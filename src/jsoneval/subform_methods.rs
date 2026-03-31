// Subform methods for isolated array field evaluation

use crate::jsoneval::cancellation::CancellationToken;
use crate::JSONEval;
use crate::ReturnFormat;
use serde_json::Value;

/// Decomposes a subform path that may optionally include a trailing item index,
/// and normalizes the base portion to the canonical schema-pointer key used in the
/// subform registry (e.g. `"#/illustration/properties/product_benefit/properties/riders"`).
///
/// Accepted formats for the **base** portion:
/// - Schema pointer:    `"#/illustration/properties/product_benefit/properties/riders"`
/// - Raw JSON pointer:  `"/illustration/properties/product_benefit/properties/riders"`
/// - Dot notation:      `"illustration.product_benefit.riders"`
///
/// Accepted formats for the **index** suffix (stripped before lookup):
/// - Trailing dot-index:     `"…riders.1"`
/// - Trailing slash-index:   `"…riders/1"`
/// - Bracket array index:    `"…riders[1]"` or `"…riders[1]."`
///
/// Returns `(canonical_base_path, optional_index)`.
fn resolve_subform_path(path: &str) -> (String, Option<usize>) {
    // --- Step 1: strip a trailing bracket array index, e.g. "riders[2]" or "riders[2]."
    let path = path.trim_end_matches('.');
    let (path, bracket_idx) = if let Some(bracket_start) = path.rfind('[') {
        let after = &path[bracket_start + 1..];
        if let Some(bracket_end) = after.find(']') {
            let idx_str = &after[..bracket_end];
            if let Ok(idx) = idx_str.parse::<usize>() {
                // strip everything from '[' onward (including any trailing '.')
                let base = path[..bracket_start].trim_end_matches('.');
                (base, Some(idx))
            } else {
                (path, None)
            }
        } else {
            (path, None)
        }
    } else {
        (path, None)
    };

    // --- Step 2: strip a trailing numeric segment (dot or slash separated)
    let (base_raw, trailing_idx) = if bracket_idx.is_none() {
        // Check dot-notation trailing index: "foo.bar.2"
        if let Some(dot_pos) = path.rfind('.') {
            let suffix = &path[dot_pos + 1..];
            if let Ok(idx) = suffix.parse::<usize>() {
                (&path[..dot_pos], Some(idx))
            } else {
                (path, None)
            }
        }
        // Check JSON-pointer trailing index: "#/foo/bar/0" or "/foo/bar/0"
        else if let Some(slash_pos) = path.rfind('/') {
            let suffix = &path[slash_pos + 1..];
            if let Ok(idx) = suffix.parse::<usize>() {
                (&path[..slash_pos], Some(idx))
            } else {
                (path, None)
            }
        } else {
            (path, None)
        }
    } else {
        (path, None)
    };

    let final_idx = bracket_idx.or(trailing_idx);

    // --- Step 3: normalize base_raw to a canonical schema pointer
    let canonical = normalize_to_subform_key(base_raw);

    (canonical, final_idx)
}

/// Normalize any path format to the canonical subform registry key.
///
/// The registry stores keys as `"#/field/properties/subfield/properties/…"` — exactly
/// as produced by the schema `walk()` function. This function converts all supported
/// formats into that form.
fn normalize_to_subform_key(path: &str) -> String {
    // Already a schema pointer — return as-is
    if path.starts_with("#/") {
        return path.to_string();
    }

    // Raw JSON pointer "/foo/properties/bar" → prefix with '#'
    if path.starts_with('/') {
        return format!("#{}", path);
    }

    // Dot-notation: "illustration.product_benefit.riders"
    // → "#/illustration/properties/product_benefit/properties/riders"
    crate::jsoneval::path_utils::dot_notation_to_schema_pointer(path)
}

impl JSONEval {
    /// Resolves the subform path, allowing aliases like "riders" to match the full
    /// schema pointer "#/illustration/properties/product_benefit/properties/riders".
    /// This ensures alias paths and full paths share the same underlying subform store and cache.
    pub(crate) fn resolve_subform_path_alias(&self, path: &str) -> (String, Option<usize>) {
        let (mut canonical, idx) = resolve_subform_path(path);

        if !self.subforms.contains_key(&canonical) {
            let search_suffix = if canonical.starts_with("#/") {
                format!("/properties/{}", &canonical[2..])
            } else {
                format!("/properties/{}", canonical)
            };

            for k in self.subforms.keys() {
                if k.ends_with(&search_suffix) || k == &canonical {
                    canonical = k.to_string();
                    break;
                }
            }
        }

        (canonical, idx)
    }

    /// Execute `f` on the subform at `base_path[idx]` with the parent cache swapped in.
    ///
    /// Lifecycle:
    /// 1. Set `data_value` + `context_value` on the subform's `eval_data`.
    /// 2. Compute item-level diff for `field_key` → bump `subform_caches[idx].data_versions`.
    /// 3. `mem::take` parent cache → set `active_item_index = Some(idx)` → swap into subform.
    /// 4. Execute `f(subform)` → collect result.
    /// 5. Swap parent cache back out → restore `self.eval_cache`.
    ///
    /// This ensures all three operations (evaluate / validate / evaluate_dependents)
    /// share parent-form Tier-2 cache entries, without duplicating the swap boilerplate.
    fn with_item_cache_swap<F, T>(
        &mut self,
        base_path: &str,
        idx: usize,
        data_value: Value,
        context_value: Value,
        f: F,
    ) -> Result<T, String>
    where
        F: FnOnce(&mut JSONEval) -> Result<T, String>,
    {
        let field_key = base_path
            .split('/')
            .next_back()
            .unwrap_or(base_path)
            .to_string();

        // Step 1: update subform data and extract item snapshot for targeted diff.
        // Scoped block releases the mutable borrow on `self.subforms` before we touch
        // `self.eval_cache` (they are disjoint fields, but keep it explicit).
        let (old_item_snapshot, new_item_val, subform_item_cache_opt, array_path, item_path) = {
            let subform = self
                .subforms
                .get_mut(base_path)
                .ok_or_else(|| format!("Subform not found: {}", base_path))?;

            let old_item_snapshot = subform
                .eval_cache
                .subform_caches
                .get(&idx)
                .map(|c| c.item_snapshot.clone())
                .unwrap_or(Value::Null);

            subform
                .eval_data
                .replace_data_and_context(data_value, context_value);
            let new_item_val = subform
                .eval_data
                .data()
                .get(&field_key)
                .cloned()
                .unwrap_or(Value::Null);

            // INJECT the item into the parent array location within subform's eval_data!
            // The frontend sometimes only provides the active item root but leaves the
            // corresponding slot empty or stale in the parent array tree of the wrapper.
            // Formulas that aggregate over the parent array must see the active item.
            let data_pointer = crate::jsoneval::path_utils::normalize_to_json_pointer(base_path)
                .replace("/properties/", "/");
            let array_path = data_pointer.to_string();
            let item_path = format!("{}/{}", array_path, idx);
            subform.eval_data.set(&item_path, new_item_val.clone());

            // Pull out any existing item-scoped entries from the subform's own cache
            // so they can be merged into the parent cache below.
            let existing = subform.eval_cache.subform_caches.remove(&idx);
            (
                old_item_snapshot,
                new_item_val,
                existing,
                array_path,
                item_path,
            )
        }; // subform borrow released here

        // Unified store fallback: if the subform's own per-item cache has no snapshot for this
        // index (e.g. this is the first evaluate_subform call after a full evaluate()), treat the
        // parent's eval_data slot as the canonical baseline. The parent always holds the most
        // recent array data written by evaluate() or evaluate_dependents(), so using it avoids
        // treating an already-evaluated item as brand-new and forcing full table re-evaluation.
        let parent_item = self.eval_data.get(&item_path).cloned();
        let old_item_snapshot = if old_item_snapshot == Value::Null {
            parent_item.clone().unwrap_or(Value::Null)
        } else {
            old_item_snapshot
        };

        // An item is "new" only when the parent's eval_data has no entry at the item path.
        // Using the subform's own snapshot cache as the authority (old_item_snapshot == Null)
        // is not correct after Step 6 persistence re-seeds the cache: a rider that was
        // previously evaluate_subform'd would have a snapshot but may still be absent from
        // the parent array (e.g. new rider scenario after evaluate_dependents_subform).
        let is_new_item = parent_item.is_none();

        let mut parent_cache = std::mem::take(&mut self.eval_cache);
        parent_cache.ensure_active_item_cache(idx);
        if let Some(c) = parent_cache.subform_caches.get_mut(&idx) {
            // Only inherit $params-scoped versions from the parent so that data-path
            // bumps from other items or previous calls don't contaminate this item's baseline.
            c.data_versions
                .merge_from_params(&parent_cache.params_versions);
            // Diff only the item field to find what changed (skips the 5 MB parent tree).
            crate::jsoneval::eval_cache::diff_and_update_versions(
                &mut c.data_versions,
                &format!("/{}", field_key),
                &old_item_snapshot,
                &new_item_val,
            );
            c.item_snapshot = new_item_val.clone();
        }
        parent_cache.active_item_index = Some(idx);

        // Restore cached entries that lived in the subform's own per-item cache.
        // Only restore entries whose dependency versions still match the current item
        // data_versions: if a field changed (e.g. sa bumped), entries that depended on
        // that field are stale and must not be re-inserted (they would cause false T1 hits).
        if let Some(subform_item_cache) = subform_item_cache_opt {
            if let Some(c) = parent_cache.subform_caches.get_mut(&idx) {
                let current_dv = c.data_versions.clone();
                for (k, v) in subform_item_cache.entries {
                    // Skip if entry already exists (parent-form run may have added a fresher result).
                    if c.entries.contains_key(&k) {
                        continue;
                    }
                    // Validate all dep versions against the current item data_versions.
                    let still_valid = v.dep_versions.iter().all(|(dep_path, &cached_ver)| {
                        let current_ver = if dep_path.starts_with("/$params") {
                            parent_cache.params_versions.get(dep_path)
                        } else {
                            current_dv.get(dep_path)
                        };
                        current_ver == cached_ver
                    });
                    if still_valid {
                        c.entries.insert(k, v);
                    }
                }
            }
        }

        // Insert into the parent eval_data as well (to make the item visible to global formulas on main evaluate).
        // Only write (and bump version) when the value actually changed: prevents spurious riders-array
        // version increments on repeated evaluate_subform calls where the rider data is unchanged.
        let current_at_item_path = self.eval_data.get(&item_path).cloned();
        if current_at_item_path.as_ref() != Some(&new_item_val) {
            self.eval_data.set(&item_path, new_item_val.clone());
            if is_new_item {
                parent_cache.bump_data_version(&array_path);
            }
        }

        // Re-evaluate `$params` tables that depend on subform item paths that changed.
        // This is required not just for brand-new items, but also whenever a tracked field
        // (like `riders.sa`) changes value: tables like RIDER_ZLOB_TABLE depend on rider.sa
        // and must produce updated rows that reflect the new sa before the subform's own
        // formula evaluation runs (otherwise cached old rows are reused).
        //
        // Gate: only re-evaluate tables when at least one item-level path was actually bumped
        // in the diff (to avoid unnecessary table work on pure no-op calls).
        let item_paths_bumped = parent_cache
            .subform_caches
            .get(&idx)
            .map(|c| {
                let field_prefix = format!("/{}/", field_key);
                c.data_versions.any_bumped_with_prefix(&field_prefix)
            })
            .unwrap_or(false);

        if is_new_item || item_paths_bumped {
            let params_table_keys: Vec<String> = self
                .table_metadata
                .keys()
                .filter(|k| k.starts_with("#/$params"))
                .cloned()
                .collect();
            if !params_table_keys.is_empty() {
                parent_cache.invalidate_params_tables_for_item(idx, &params_table_keys);

                let eval_data_snapshot = self.eval_data.exclusive_clone();
                for key in &params_table_keys {
                    // CRITICAL FIX: Only evaluate global tables on the parent if they do NOT
                    // depend on subform-specific item paths (like `#/riders/...`).
                    // Tables like WOP_ZLOB_PREMI_TABLE contain formulas like `#/riders/properties/code`
                    // and MUST be evaluated by the subform engine to see the subform's current data.
                    // Tables like WOP_RIDERS contain formulas like `#/illustration/product_benefit/riders`
                    // and MUST be evaluated by the parent engine to see the full parent array.
                    let depends_on_subform_item = if let Some(deps) = self.dependencies.get(key) {
                        let subform_dep_prefix = format!("#/{}/properties/", field_key);
                        let subform_dep_prefix_short = format!("#/{}/", field_key);
                        deps.iter().any(|dep| {
                            dep.starts_with(&subform_dep_prefix)
                                || dep.starts_with(&subform_dep_prefix_short)
                        })
                    } else {
                        false
                    };

                    if depends_on_subform_item {
                        continue;
                    }

                    // Evaluate the table using parent's updated data
                    if let Ok(rows) = crate::jsoneval::table_evaluate::evaluate_table(
                        self,
                        key,
                        &eval_data_snapshot,
                        None,
                    ) {
                        if std::env::var("JSONEVAL_DEBUG_CACHE").is_ok() {
                            println!("PARENT EVALUATED TABLE {} -> {} rows", key, rows.len());
                        }
                        let result_val = serde_json::Value::Array(rows);

                        // Collect external dependencies for this cache entry
                        let mut external_deps = indexmap::IndexSet::new();
                        let pointer_data_prefix =
                            crate::jsoneval::path_utils::normalize_to_json_pointer(key)
                                .replace("/properties/", "/");
                        let pointer_data_prefix_slash = format!("{}/", pointer_data_prefix);
                        if let Some(deps) = self.dependencies.get(key) {
                            for dep in deps {
                                let dep_data_path =
                                    crate::jsoneval::path_utils::normalize_to_json_pointer(dep)
                                        .replace("/properties/", "/");
                                if dep_data_path != pointer_data_prefix
                                    && !dep_data_path.starts_with(&pointer_data_prefix_slash)
                                {
                                    external_deps.insert(dep.clone());
                                }
                            }
                        }

                        // We must temporarily clear active_item_index so store_cache puts this in T2 (global)
                        // Then the subform can hit it via T2 fallback check.
                        parent_cache.active_item_index = None;
                        parent_cache.store_cache(key, &external_deps, result_val);
                        parent_cache.active_item_index = Some(idx);
                    } else {
                        if std::env::var("JSONEVAL_DEBUG_CACHE").is_ok() {
                            println!("PARENT EVALUATED TABLE {} -> ERROR", key);
                        }
                    }
                }
            }
        }

        // Step 3: swap parent cache into subform so Tier 1 + Tier 2 entries are visible.
        {
            let subform = self.subforms.get_mut(base_path).unwrap();
            std::mem::swap(&mut subform.eval_cache, &mut parent_cache);
        }

        // Step 4: run the caller-supplied operation.
        let result = {
            let subform = self.subforms.get_mut(base_path).unwrap();
            f(subform)
        };

        // Step 5: restore parent cache.
        {
            let subform = self.subforms.get_mut(base_path).unwrap();
            std::mem::swap(&mut subform.eval_cache, &mut parent_cache);
        }
        parent_cache.active_item_index = None;
        self.eval_cache = parent_cache;

        // Step 6: persist the updated T1 item cache (snapshot + entries) back into the subform's
        // own per-item cache. Without this, the next evaluate_subform call for the same idx reads
        // old_item_snapshot = Null from the subform cache (it was removed at line 183) and treats
        // the rider as brand-new, forcing a full re-diff and invalidating all T1 entries.
        {
            let subform = self.subforms.get_mut(base_path).unwrap();
            if let Some(item_cache) = self.eval_cache.subform_caches.get(&idx) {
                subform
                    .eval_cache
                    .subform_caches
                    .insert(idx, item_cache.clone());
            }
        }

        result
    }

    /// Evaluate a subform identified by `subform_path`.
    ///
    /// The path may include a trailing item index to bind the evaluation to a specific
    /// array element and enable the two-tier cache-swap strategy automatically:
    ///
    /// ```text
    /// // Evaluate riders item 1 with index-aware cache
    /// eval.evaluate_subform("illustration.product_benefit.riders.1", data, ctx, None, None)?;
    /// ```
    ///
    /// Without a trailing index, the subform is evaluated in isolation (no cache swap).
    pub fn evaluate_subform(
        &mut self,
        subform_path: &str,
        data: &str,
        context: Option<&str>,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<(), String> {
        let (base_path, idx_opt) = self.resolve_subform_path_alias(subform_path);
        if let Some(idx) = idx_opt {
            self.evaluate_subform_item(&base_path, idx, data, context, paths, token)
        } else {
            let subform = self
                .subforms
                .get_mut(base_path.as_ref() as &str)
                .ok_or_else(|| format!("Subform not found: {}", base_path))?;
            subform.evaluate(data, context, paths, token)
        }
    }

    /// Internal: evaluate a single subform item at `idx` using the cache-swap strategy.
    fn evaluate_subform_item(
        &mut self,
        base_path: &str,
        idx: usize,
        data: &str,
        context: Option<&str>,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<(), String> {
        let data_value = crate::jsoneval::json_parser::parse_json_str(data)
            .map_err(|e| format!("Failed to parse subform data: {}", e))?;
        let context_value = if let Some(ctx) = context {
            crate::jsoneval::json_parser::parse_json_str(ctx)
                .map_err(|e| format!("Failed to parse subform context: {}", e))?
        } else {
            Value::Object(serde_json::Map::new())
        };

        self.with_item_cache_swap(base_path, idx, data_value, context_value, |sf| {
            sf.evaluate_internal_pre_diffed(paths, token)
        })
    }

    /// Validate subform data against its schema rules.
    ///
    /// Supports the same trailing-index path syntax as `evaluate_subform`. When an index
    /// is present the parent cache is swapped in first, ensuring rule evaluations that
    /// depend on `$params` tables share already-computed parent-form results.
    pub fn validate_subform(
        &mut self,
        subform_path: &str,
        data: &str,
        context: Option<&str>,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<crate::ValidationResult, String> {
        let (base_path, idx_opt) = self.resolve_subform_path_alias(subform_path);
        if let Some(idx) = idx_opt {
            let data_value = crate::jsoneval::json_parser::parse_json_str(data)
                .map_err(|e| format!("Failed to parse subform data: {}", e))?;
            let context_value = if let Some(ctx) = context {
                crate::jsoneval::json_parser::parse_json_str(ctx)
                    .map_err(|e| format!("Failed to parse subform context: {}", e))?
            } else {
                Value::Object(serde_json::Map::new())
            };
            let data_for_validation = data_value.clone();
            self.with_item_cache_swap(
                base_path.as_ref(),
                idx,
                data_value,
                context_value,
                move |sf| {
                    // Warm the evaluation cache before running rule checks.
                    sf.evaluate_internal_pre_diffed(paths, token)?;
                    sf.validate_pre_set(data_for_validation, paths, token)
                },
            )
        } else {
            let subform = self
                .subforms
                .get_mut(base_path.as_ref() as &str)
                .ok_or_else(|| format!("Subform not found: {}", base_path))?;
            subform.validate(data, context, paths, token)
        }
    }

    /// Evaluate dependents in a subform when a field changes.
    ///
    /// Supports the same trailing-index path syntax as `evaluate_subform`. When an index
    /// is present the parent cache is swapped in, so dependent evaluation runs with
    /// Tier-2 entries visible and item-scoped version bumps propagate to `eval_generation`.
    pub fn evaluate_dependents_subform(
        &mut self,
        subform_path: &str,
        changed_paths: &[String],
        data: Option<&str>,
        context: Option<&str>,
        re_evaluate: bool,
        token: Option<&CancellationToken>,
        canceled_paths: Option<&mut Vec<String>>,
        include_subforms: bool,
    ) -> Result<Value, String> {
        let (base_path, idx_opt) = self.resolve_subform_path_alias(subform_path);
        if let Some(idx) = idx_opt {
            // Parse or snapshot data for the swap / diff computation.
            let (data_value, context_value) = if let Some(data_str) = data {
                let dv = crate::jsoneval::json_parser::parse_json_str(data_str)
                    .map_err(|e| format!("Failed to parse subform data: {}", e))?;
                let cv = if let Some(ctx) = context {
                    crate::jsoneval::json_parser::parse_json_str(ctx)
                        .map_err(|e| format!("Failed to parse subform context: {}", e))?
                } else {
                    Value::Object(serde_json::Map::new())
                };
                (dv, cv)
            } else {
                // No new data provided — snapshot current subform state so diff is a no-op.
                let subform = self
                    .subforms
                    .get(base_path.as_ref() as &str)
                    .ok_or_else(|| format!("Subform not found: {}", base_path))?;
                let dv = subform.eval_data.snapshot_data_clone();
                (dv, Value::Object(serde_json::Map::new()))
            };
            self.with_item_cache_swap(base_path.as_ref(), idx, data_value, context_value, |sf| {
                // Data is already set by with_item_cache_swap; pass None to avoid re-parsing.
                sf.evaluate_dependents(
                    changed_paths,
                    None,
                    None,
                    re_evaluate,
                    token,
                    None,
                    include_subforms,
                )
            })
        } else {
            let subform = self
                .subforms
                .get_mut(base_path.as_ref() as &str)
                .ok_or_else(|| format!("Subform not found: {}", base_path))?;
            subform.evaluate_dependents(
                changed_paths,
                data,
                context,
                re_evaluate,
                token,
                canceled_paths,
                include_subforms,
            )
        }
    }

    /// Resolve layout for subform.
    pub fn resolve_layout_subform(
        &mut self,
        subform_path: &str,
        evaluate: bool,
    ) -> Result<(), String> {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        let subform = self
            .subforms
            .get_mut(base_path.as_ref() as &str)
            .ok_or_else(|| format!("Subform not found: {}", base_path))?;
        let _ = subform.resolve_layout(evaluate);
        Ok(())
    }

    /// Get evaluated schema from subform.
    pub fn get_evaluated_schema_subform(
        &mut self,
        subform_path: &str,
        resolve_layout: bool,
    ) -> Value {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        if let Some(subform) = self.subforms.get_mut(base_path.as_ref() as &str) {
            subform.get_evaluated_schema(resolve_layout)
        } else {
            Value::Null
        }
    }

    /// Get schema value from subform in nested object format (all .value fields).
    pub fn get_schema_value_subform(&mut self, subform_path: &str) -> Value {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        if let Some(subform) = self.subforms.get_mut(base_path.as_ref() as &str) {
            subform.get_schema_value()
        } else {
            Value::Null
        }
    }

    /// Get schema values from subform as a flat array of path-value pairs.
    pub fn get_schema_value_array_subform(&self, subform_path: &str) -> Value {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        if let Some(subform) = self.subforms.get(base_path.as_ref() as &str) {
            subform.get_schema_value_array()
        } else {
            Value::Array(vec![])
        }
    }

    /// Get schema values from subform as a flat object with dotted path keys.
    pub fn get_schema_value_object_subform(&self, subform_path: &str) -> Value {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        if let Some(subform) = self.subforms.get(base_path.as_ref() as &str) {
            subform.get_schema_value_object()
        } else {
            Value::Object(serde_json::Map::new())
        }
    }

    /// Get evaluated schema without $params from subform.
    pub fn get_evaluated_schema_without_params_subform(
        &mut self,
        subform_path: &str,
        resolve_layout: bool,
    ) -> Value {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        if let Some(subform) = self.subforms.get_mut(base_path.as_ref() as &str) {
            subform.get_evaluated_schema_without_params(resolve_layout)
        } else {
            Value::Null
        }
    }

    /// Get evaluated schema by specific path from subform.
    pub fn get_evaluated_schema_by_path_subform(
        &mut self,
        subform_path: &str,
        schema_path: &str,
        skip_layout: bool,
    ) -> Option<Value> {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        self.subforms.get_mut(base_path.as_ref() as &str).map(|sf| {
            sf.get_evaluated_schema_by_paths(
                &[schema_path.to_string()],
                skip_layout,
                Some(ReturnFormat::Nested),
            )
        })
    }

    /// Get evaluated schema by multiple paths from subform.
    pub fn get_evaluated_schema_by_paths_subform(
        &mut self,
        subform_path: &str,
        schema_paths: &[String],
        skip_layout: bool,
        format: Option<crate::ReturnFormat>,
    ) -> Value {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        if let Some(subform) = self.subforms.get_mut(base_path.as_ref() as &str) {
            subform.get_evaluated_schema_by_paths(
                schema_paths,
                skip_layout,
                Some(format.unwrap_or(ReturnFormat::Flat)),
            )
        } else {
            match format.unwrap_or_default() {
                crate::ReturnFormat::Array => Value::Array(vec![]),
                _ => Value::Object(serde_json::Map::new()),
            }
        }
    }

    /// Get schema by specific path from subform.
    pub fn get_schema_by_path_subform(
        &self,
        subform_path: &str,
        schema_path: &str,
    ) -> Option<Value> {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        self.subforms
            .get(base_path.as_ref() as &str)
            .and_then(|sf| sf.get_schema_by_path(schema_path))
    }

    /// Get schema by multiple paths from subform.
    pub fn get_schema_by_paths_subform(
        &self,
        subform_path: &str,
        schema_paths: &[String],
        format: Option<crate::ReturnFormat>,
    ) -> Value {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        if let Some(subform) = self.subforms.get(base_path.as_ref() as &str) {
            subform.get_schema_by_paths(schema_paths, Some(format.unwrap_or(ReturnFormat::Flat)))
        } else {
            match format.unwrap_or_default() {
                crate::ReturnFormat::Array => Value::Array(vec![]),
                _ => Value::Object(serde_json::Map::new()),
            }
        }
    }

    /// Get list of available subform paths.
    pub fn get_subform_paths(&self) -> Vec<String> {
        self.subforms.keys().cloned().collect()
    }

    /// Check if a subform exists at the given path.
    pub fn has_subform(&self, subform_path: &str) -> bool {
        let (base_path, _) = self.resolve_subform_path_alias(subform_path);
        self.subforms.contains_key(base_path.as_ref() as &str)
    }
}
