// Subform methods for isolated array field evaluation

use crate::JSONEval;
use crate::ReturnFormat;
use crate::jsoneval::cancellation::CancellationToken;
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
        let field_key = base_path.split('/').next_back().unwrap_or(base_path).to_string();

        // Step 1: update subform data and extract item snapshot for targeted diff.
        // Scoped block releases the mutable borrow on `self.subforms` before we touch
        // `self.eval_cache` (they are disjoint fields, but keep it explicit).
        let (old_item_snapshot, new_item_val, subform_item_cache_opt) = {
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

            subform.eval_data.replace_data_and_context(data_value, context_value);
            let new_item_val = subform
                .eval_data
                .data()
                .get(&field_key)
                .cloned()
                .unwrap_or(Value::Null);

            // Pull out any existing item-scoped entries from the subform's own cache
            // so they can be merged into the parent cache below.
            let existing = subform.eval_cache.subform_caches.remove(&idx);
            (old_item_snapshot, new_item_val, existing)
        }; // subform borrow released here

        // Step 2: build parent cache with item-scoped diff applied.
        let mut parent_cache = std::mem::take(&mut self.eval_cache);
        parent_cache.ensure_active_item_cache(idx);
        if let Some(c) = parent_cache.subform_caches.get_mut(&idx) {
            // Inherit parent-level version counters so Tier-2 dep checks are correct.
            c.data_versions.merge_from(&parent_cache.data_versions);
            // Diff only the item field to find what changed (skips the 5 MB parent tree).
            crate::jsoneval::eval_cache::diff_and_update_versions(
                &mut c.data_versions,
                &format!("/{}", field_key),
                &old_item_snapshot,
                &new_item_val,
            );
            c.item_snapshot = new_item_val;
        }
        parent_cache.active_item_index = Some(idx);

        // Migrate any item-scoped entries that lived in the subform's own cache.
        // `or_insert` preserves the parent's newly-diffed entry if it already exists.
        if let Some(subform_item_cache) = subform_item_cache_opt {
            parent_cache.subform_caches.entry(idx).or_insert(subform_item_cache);
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
        let (base_path, idx_opt) = resolve_subform_path(subform_path);
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
        let (base_path, idx_opt) = resolve_subform_path(subform_path);
        if let Some(idx) = idx_opt {
            let data_value = crate::jsoneval::json_parser::parse_json_str(data)
                .map_err(|e| format!("Failed to parse subform data: {}", e))?;
            let context_value = if let Some(ctx) = context {
                crate::jsoneval::json_parser::parse_json_str(ctx)
                    .map_err(|e| format!("Failed to parse subform context: {}", e))?
            } else {
                Value::Object(serde_json::Map::new())
            };
            // Clone data before it is consumed by with_item_cache_swap so we can pass it
            // to validate_pre_set which needs to run validate_field with the raw data.
            let data_for_validation = data_value.clone();
            self.with_item_cache_swap(base_path.as_ref(), idx, data_value, context_value, move |sf| {
                // Warm the evaluation cache before running rule checks.
                sf.evaluate_internal_pre_diffed(paths, token)?;
                sf.validate_pre_set(data_for_validation, paths, token)
            })
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
        let (base_path, idx_opt) = resolve_subform_path(subform_path);
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
                sf.evaluate_dependents(changed_paths, None, None, re_evaluate, token, None, include_subforms)
            })
        } else {
            let subform = self
                .subforms
                .get_mut(base_path.as_ref() as &str)
                .ok_or_else(|| format!("Subform not found: {}", base_path))?;
            subform.evaluate_dependents(changed_paths, data, context, re_evaluate, token, canceled_paths, include_subforms)
        }
    }

    /// Resolve layout for subform.
    pub fn resolve_layout_subform(
        &mut self,
        subform_path: &str,
        evaluate: bool,
    ) -> Result<(), String> {
        let (base_path, _) = resolve_subform_path(subform_path);
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
        let (base_path, _) = resolve_subform_path(subform_path);
        if let Some(subform) = self.subforms.get_mut(base_path.as_ref() as &str) {
            subform.get_evaluated_schema(resolve_layout)
        } else {
            Value::Null
        }
    }

    /// Get schema value from subform in nested object format (all .value fields).
    pub fn get_schema_value_subform(&mut self, subform_path: &str) -> Value {
        let (base_path, _) = resolve_subform_path(subform_path);
        if let Some(subform) = self.subforms.get_mut(base_path.as_ref() as &str) {
            subform.get_schema_value()
        } else {
            Value::Null
        }
    }

    /// Get schema values from subform as a flat array of path-value pairs.
    pub fn get_schema_value_array_subform(&self, subform_path: &str) -> Value {
        let (base_path, _) = resolve_subform_path(subform_path);
        if let Some(subform) = self.subforms.get(base_path.as_ref() as &str) {
            subform.get_schema_value_array()
        } else {
            Value::Array(vec![])
        }
    }

    /// Get schema values from subform as a flat object with dotted path keys.
    pub fn get_schema_value_object_subform(&self, subform_path: &str) -> Value {
        let (base_path, _) = resolve_subform_path(subform_path);
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
        let (base_path, _) = resolve_subform_path(subform_path);
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
        let (base_path, _) = resolve_subform_path(subform_path);
        self.subforms.get_mut(base_path.as_ref() as &str).map(|sf| {
            sf.get_evaluated_schema_by_paths(&[schema_path.to_string()], skip_layout, Some(ReturnFormat::Nested))
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
        let (base_path, _) = resolve_subform_path(subform_path);
        if let Some(subform) = self.subforms.get_mut(base_path.as_ref() as &str) {
            subform.get_evaluated_schema_by_paths(schema_paths, skip_layout, Some(format.unwrap_or(ReturnFormat::Flat)))
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
        let (base_path, _) = resolve_subform_path(subform_path);
        self.subforms.get(base_path.as_ref() as &str).and_then(|sf| sf.get_schema_by_path(schema_path))
    }

    /// Get schema by multiple paths from subform.
    pub fn get_schema_by_paths_subform(
        &self,
        subform_path: &str,
        schema_paths: &[String],
        format: Option<crate::ReturnFormat>,
    ) -> Value {
        let (base_path, _) = resolve_subform_path(subform_path);
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
        let (base_path, _) = resolve_subform_path(subform_path);
        self.subforms.contains_key(base_path.as_ref() as &str)
    }
}
