use std::sync::Arc;

use super::JSONEval;
use crate::jsoneval::cancellation::CancellationToken;
use crate::jsoneval::eval_data::EvalData;
use crate::jsoneval::json_parser;
use crate::jsoneval::path_utils;
use crate::jsoneval::table_evaluate;
use crate::time_block;
use crate::utils::clean_float_noise_scalar;

use serde_json::Value;

/// Returns `true` if `new_item` (raw user input) is identity-compatible with `old_item`
/// (snapshot that may contain computed formula outputs alongside raw input fields).
///
/// A full `==` comparison fails when `old_item` has extra keys written by formula evaluation
/// (e.g., `wop_rider_premi`, `first_prem`) that are absent from the raw `new_item`. This helper
/// compares only the fields present in `new_item`, ignoring extra keys in `old_item`:
///
/// - If both are objects: every key in `new` must match the same key in `old`.
/// - Otherwise: standard equality (covers Null, scalar, array cases).
///
/// Used by `invalidate_subform_caches_on_structural_change` to detect genuine order/identity
/// shifts without false positives from computed formula output fields in the snapshot.
fn items_same_input_identity(old: Option<&Value>, new: Option<&Value>) -> bool {
    match (old, new) {
        (Some(Value::Object(old_map)), Some(Value::Object(new_map))) => new_map
            .iter()
            .all(|(k, new_val)| old_map.get(k).map_or(false, |old_val| old_val == new_val)),
        (old, new) => old == new,
    }
}

impl JSONEval {
    /// Evaluate the schema with the given data and context.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to evaluate.
    /// * `context` - The context to evaluate.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error message.
    pub fn evaluate(
        &mut self,
        data: &str,
        context: Option<&str>,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<(), String> {
        if let Some(t) = token {
            if t.is_cancelled() {
                return Err("Cancelled".to_string());
            }
        }
        time_block!("evaluate() [total]", {
            // Use SIMD-accelerated JSON parsing
            // Parse and update data/context
            let data_value = time_block!("  parse data", { json_parser::parse_json_str(data)? });
            let context_value = time_block!("  parse context", {
                if let Some(ctx) = context {
                    json_parser::parse_json_str(ctx)?
                } else {
                    Value::Object(serde_json::Map::new())
                }
            });
            self.evaluate_internal_with_new_data(data_value, context_value, paths, token)
        })
    }

    /// Internal helper to evaluate with all data/context provided as Values.
    /// `pub(crate)` so the cache-swap path in `evaluate_subform` can call it directly
    /// after swapping the parent cache in, bypassing the string-parsing overhead.
    pub(crate) fn evaluate_internal_with_new_data(
        &mut self,
        data: Value,
        context: Value,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<(), String> {
        time_block!("  evaluate_internal_with_new_data", {
            // Reuse the previously stored snapshot as `old_data` to avoid an O(n) deep clone
            // on every main-form evaluation call.
            let has_previous_eval = self.eval_cache.main_form_snapshot.is_some();
            let old_data = self
                .eval_cache
                .main_form_snapshot
                .take()
                .unwrap_or_else(|| self.eval_data.snapshot_data_clone());

            let old_context = self
                .eval_data
                .data()
                .get("$context")
                .cloned()
                .unwrap_or(Value::Null);

            // Store data, context and replace in eval_data (clone once instead of twice)
            self.data = data.clone();
            self.context = context.clone();
            time_block!("  replace_data_and_context", {
                self.eval_data.replace_data_and_context(data, context);
            });

            let new_data = self.eval_data.snapshot_data_clone();
            let new_context = self
                .eval_data
                .data()
                .get("$context")
                .cloned()
                .unwrap_or(Value::Null);

            if has_previous_eval
                && old_data == new_data
                && old_context == new_context
                && paths.is_none()
            {
                // Perfect cache hit for unmodified payload: fully skip tree traversal.
                // Restore snapshot since nothing changed.
                self.eval_cache.main_form_snapshot = Some(new_data);
                return Ok(());
            }

            // Proactively populate per-item caches for all existing subform items from the loaded data.
            // When a user opens an existing form (e.g. reload from DB), the main `evaluate(data)`
            // establishes the baseline state. If we don't populate subform caches here, the first
            // time the user opens a rider (`evaluate_subform`), the cache is empty (item_snapshot=Null).
            // The diff between Null and the full rider data will then mark EVERY field (sa, code, etc.)
            // as "changed", spuriously bumping secondary trackers and causing false T2 table misses.
            for (subform_path, subform) in &mut self.subforms {
                let subform_ptr =
                    crate::jsoneval::path_utils::normalize_to_json_pointer(subform_path);
                if let Some(items) = new_data.pointer(&subform_ptr).and_then(|v| v.as_array()) {
                    for (idx, item_val) in items.iter().enumerate() {
                        self.eval_cache.ensure_active_item_cache(idx);
                        if let Some(c) = self.eval_cache.subform_caches.get_mut(&idx) {
                            c.item_snapshot = item_val.clone();
                        }
                        subform.eval_cache.ensure_active_item_cache(idx);
                        if let Some(c) = subform.eval_cache.subform_caches.get_mut(&idx) {
                            c.item_snapshot = item_val.clone();
                        }
                    }
                }
            }

            self.eval_cache
                .store_snapshot_and_diff_versions(&old_data, &new_data);
            // Save snapshot for the next evaluation cycle (avoids one snapshot_data_clone() call)
            self.eval_cache.main_form_snapshot = Some(new_data.clone());

            // Detect subform array structural changes: length differences OR item identity shifts
            // (e.g., rider reorder). When items move indices their per-index T1 caches are misaligned,
            // and T2 global entries keyed on subform-local dep paths (e.g., `/riders/code`) must be
            // evicted — the parent diff only bumps indexed full paths like
            // `/illustration/product_benefit/riders/2/code`, which never match the stored dep key.
            self.invalidate_subform_caches_on_structural_change(&old_data, &new_data);

            // Generation-based fast skip: diff_and_update_versions bumps data_versions.versions
            // but does NOT increment eval_generation. Only bump_data_version / bump_params_version
            // (called from formula stores) advance eval_generation.
            // If eval_generation == last_evaluated_generation after the diff, no formula's cached
            // deps are actually stale — all batches would be cache hits. Skip the full traversal.
            // Safe only in the external evaluate() path; run_re_evaluate_pass must always evaluate.
            if paths.is_none() && !self.eval_cache.needs_full_evaluation() {
                self.evaluate_others(paths, token, false);
                return Ok(());
            }

            // Call internal evaluate (uses existing data if not provided)
            self.evaluate_internal(paths, token)
        })
    }

    /// Detect structural changes in subform arrays between `old_data` and `new_data`
    /// and evict stale caches accordingly.
    pub(crate) fn invalidate_subform_caches_on_structural_change(
        &mut self,
        old_data: &Value,
        new_data: &Value,
    ) {
        use crate::jsoneval::path_utils::normalize_to_json_pointer;

        for (subform_path, _) in &self.subforms {
            // Resolve the data pointer for this subform
            // (e.g., `/illustration/product_benefit/riders`)
            let subform_ptr = normalize_to_json_pointer(subform_path).replace("/properties/", "/");

            let old_items = old_data.pointer(&subform_ptr).and_then(Value::as_array);
            let new_items = new_data.pointer(&subform_ptr).and_then(Value::as_array);

            let old_len = old_items.map(Vec::len).unwrap_or(0);
            let new_len = new_items.map(Vec::len).unwrap_or(0);
            let min_len = old_len.min(new_len);

            // Detect identity shift in the overlapping index range using subset comparison.
            // We check whether the raw input fields of new_items[i] all match old_items[i],
            // ignoring extra computed keys that only exist in the old snapshot.
            let identities_shifted = (0..min_len).any(|i| {
                let old_item = old_items.and_then(|a| a.get(i));
                let new_item = new_items.and_then(|a| a.get(i));
                !items_same_input_identity(old_item, new_item)
            });

            if old_len == new_len && !identities_shifted {
                continue; // No structural change for this subform
            }

            // Build the subform-local dep-path prefix stored in T2 dep_versions
            // (e.g., `/riders/` for a riders subform). T2 dep keys are normalized data
            // paths — never schema paths — so only one prefix is needed.
            let field_key = subform_ptr
                .split('/')
                .next_back()
                .unwrap_or(subform_ptr.as_str());
            let subform_dep_prefix = format!("/{}/", field_key);

            // Evict T2 global entries whose deps include any subform-local path.
            // `retain` evicts inline (no intermediate Vec allocation).
            // Collect the normalized path of each evicted key for the params_versions bump.
            let mut evicted_paths: Vec<String> = Vec::new();
            self.eval_cache.entries.retain(|eval_key, entry| {
                let has_subform_dep = entry
                    .dep_versions
                    .keys()
                    .any(|dep| dep.starts_with(&subform_dep_prefix));

                if has_subform_dep {
                    // Normalize eval_key → params data path once, at eviction time
                    let raw = normalize_to_json_pointer(eval_key).replace("/properties/", "/");
                    let normalized = raw.trim_start_matches('#');
                    evicted_paths.push(if normalized.starts_with('/') {
                        normalized.to_string()
                    } else {
                        format!("/{}", normalized)
                    });
                    false // remove entry
                } else {
                    true // keep
                }
            });

            // Bump params_versions for every evicted T2 entry so downstream $params formulas
            // (SA_WOP_RIDER, TOTAL_WOP_SA, etc.) correctly miss their caches.
            for path in &evicted_paths {
                self.eval_cache.params_versions.bump(path);
            }

            // Clear T1 per-item caches for indices where item identity has shifted.
            // This prevents stale per-rider results being reused for a different rider
            // occupying the same array slot after a reorder.
            for idx in 0..min_len {
                let old_item = old_items.and_then(|a| a.get(idx));
                let new_item = new_items.and_then(|a| a.get(idx));
                if !items_same_input_identity(old_item, new_item) {
                    if let Some(c) = self.eval_cache.subform_caches.get_mut(&idx) {
                        c.entries.clear();
                        c.data_versions = crate::jsoneval::eval_cache::VersionTracker::new();
                    }
                }
            }
            // Prune T1 caches for indices that no longer exist (removed items)
            self.eval_cache.prune_subform_caches(new_len);

            if !evicted_paths.is_empty() || old_len != new_len {
                self.eval_cache.eval_generation += 1;
            }
        }
    }

    /// Fast variant of `evaluate_internal_with_new_data` for the cache-swap path.
    ///
    /// The caller (e.g. `run_subform_pass` / `evaluate_subform_item`) has **already**:
    /// 1. Called `replace_data_and_context` on `subform.eval_data` with the merged payload.
    /// 2. Computed the item-level diff and bumped `subform_caches[idx].data_versions` accordingly.
    /// 3. Swapped the parent cache into `subform.eval_cache` so Tier 2 entries are visible.
    /// 4. Set `active_item_index = Some(idx)` on the swapped-in cache.
    ///
    /// Skipping the expensive `snapshot_data_clone()` × 2 and `diff_and_update_versions`
    /// saves ~40–80ms per rider on a 5 MB parent payload.
    pub(crate) fn evaluate_internal_pre_diffed(
        &mut self,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<(), String> {
        debug_assert!(
            self.eval_cache.active_item_index.is_some(),
            "evaluate_internal_pre_diffed called without active_item_index — \
             caller must set up the cache-swap before calling this method"
        );

        // Always delegate to evaluate_internal so that evaluated_schema is populated correctly
        // for every item. The previous generation-based skip here left evaluated_schema stale
        // (with the prior rider's values) when no deps changed — causing get_evaluated_schema_subform
        // to return wrong values for all but the last-evaluated rider.
        //
        // evaluate_internal's all-hit fast path (lines ~314–338) handles the no-change case
        // efficiently: it writes eval_data + evaluated_schema per formula from T1 cache and
        // skips the expensive formula engine entirely.
        self.evaluate_internal(paths, token)
    }

    /// Internal evaluate that can be called when data is already set
    /// This avoids double-locking and unnecessary data cloning for re-evaluation from evaluate_dependents
    pub(crate) fn evaluate_internal(
        &mut self,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<(), String> {
        if let Some(t) = token {
            if t.is_cancelled() {
                return Err("Cancelled".to_string());
            }
        }
        time_block!("  evaluate_internal() [total]", {
            // Acquire lock for synchronous execution
            let _lock = self.eval_lock.lock().unwrap();

            // Normalize paths to schema pointers for correct filtering
            let normalized_paths_storage; // Keep alive
            let normalized_paths = if let Some(p_list) = paths {
                normalized_paths_storage = p_list
                    .iter()
                    .flat_map(|p| {
                        let normalized = if p.starts_with("#/") {
                            p.to_string()
                        } else if p.starts_with('/') {
                            format!("#{}", p)
                        } else {
                            format!("#/{}", p.replace('.', "/"))
                        };
                        vec![normalized]
                    })
                    .collect::<Vec<_>>();
                Some(normalized_paths_storage.as_slice())
            } else {
                None
            };

            // Borrow sorted_evaluations via Arc (avoid deep-cloning Vec<Vec<String>>)
            let eval_batches = self.sorted_evaluations.clone();

            // Track whether any entry was a cache miss (required an actual formula run).
            // When false (all hits), evaluate_others can skip resolve_layout because no
            // values changed and the layout state is guaranteed identical.
            // On the very first evaluation (last_evaluated_generation == u64::MAX), we MUST
            // force a cache miss so that static schemas (with no formulas) still process
            // URL templates and layout resolution once.
            let mut had_cache_miss = self.eval_cache.last_evaluated_generation == u64::MAX;

            // Process each batch - sequentially
            // Batches are processed sequentially to maintain dependency order
            // Process value evaluations (simple computed fields with no dependencies)
            let eval_data_values = self.eval_data.clone();
            time_block!("      evaluate values", {
                for eval_key in self.value_evaluations.iter() {
                    if let Some(t) = token {
                        if t.is_cancelled() {
                            return Err("Cancelled".to_string());
                        }
                    }
                    // Skip if has dependencies (handled in sorted batches with correct ordering)
                    if let Some(deps) = self.dependencies.get(eval_key) {
                        if !deps.is_empty() {
                            continue;
                        }
                    }

                    // Filter items if paths are provided
                    if let Some(filter_paths) = normalized_paths {
                        if !filter_paths.is_empty()
                            && !filter_paths.iter().any(|p| {
                                eval_key.starts_with(p.as_str()) || p.starts_with(eval_key.as_str())
                            })
                        {
                            continue;
                        }
                    }

                    let pointer_path = path_utils::normalize_to_json_pointer(eval_key).into_owned();
                    let empty_deps = indexmap::IndexSet::new();
                    let deps = self.dependencies.get(eval_key).unwrap_or(&empty_deps);

                    // Cache hit check
                    if let Some(_cached_result) = self.eval_cache.check_cache(eval_key, deps) {
                        continue;
                    }

                    had_cache_miss = true;
                    // Cache miss - evaluate
                    if let Some(logic_id) = self.evaluations.get(eval_key) {
                        if let Ok(val) = self.engine.run(logic_id, eval_data_values.data()) {
                            let cleaned_val = clean_float_noise_scalar(val);
                            self.eval_cache
                                .store_cache(eval_key, deps, cleaned_val.clone());

                            if let Some(pointer_value) =
                                self.evaluated_schema.pointer_mut(&pointer_path)
                            {
                                *pointer_value = cleaned_val;
                            }
                        }
                    }
                }
            });

            time_block!("    process batches", {
                for batch in eval_batches.iter() {
                    if let Some(t) = token {
                        if t.is_cancelled() {
                            return Err("Cancelled".to_string());
                        }
                    }
                    // Skip empty batches
                    if batch.is_empty() {
                        continue;
                    }

                    // Check if we can skip this entire batch optimization
                    let batch_skipped = time_block!("      batch filter check", {
                        if let Some(filter_paths) = normalized_paths {
                            if !filter_paths.is_empty() {
                                let batch_has_match = batch.iter().any(|eval_key| {
                                    filter_paths.iter().any(|p| {
                                        eval_key.starts_with(p.as_str())
                                            || (p.starts_with(eval_key.as_str())
                                                && !eval_key.contains("/$params/"))
                                    })
                                });
                                !batch_has_match
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    });
                    if batch_skipped {
                        continue;
                    }

                    // Fast path: try to resolve every eval_key in this batch from cache.
                    // If all hit, skip the expensive exclusive_clone() of the full eval_data tree.
                    // This is critical for subforms where eval_data contains the full parent payload.
                    let all_cache_hit = time_block!("      batch cache fast path", {
                        let mut batch_hits: Vec<(String, Value)> = Vec::with_capacity(batch.len());
                        let all_hit = batch.iter().all(|eval_key| {
                            let empty_deps = indexmap::IndexSet::new();
                            let deps = self.dependencies.get(eval_key).unwrap_or(&empty_deps);
                            if let Some(cached) = self.eval_cache.check_cache(eval_key, deps) {
                                let pointer_path =
                                    path_utils::normalize_to_json_pointer(eval_key).into_owned();
                                batch_hits.push((pointer_path, cached));
                                true
                            } else {
                                false
                            }
                        });

                        if all_hit {
                            // Populate eval_data AND evaluated_schema so both downstream batches
                            // and get_evaluated_schema callers see the correct per-item values.
                            // Previously only eval_data was written here, leaving evaluated_schema
                            // with stale values from the last full-miss evaluation (e.g. the first
                            // rider), causing all riders to report the same schema outputs.
                            for (ptr, val) in batch_hits {
                                self.eval_data.set(&ptr, val.clone());
                                if let Some(schema_value) = self.evaluated_schema.pointer_mut(&ptr)
                                {
                                    *schema_value = val;
                                }
                            }
                        }
                        // Partial or full miss — fall through to the normal exclusive_clone path below.
                        // batch_hits is dropped here; cache lookups will repeat but that's cheap.
                        all_hit
                    });
                    if all_cache_hit {
                        continue;
                    }
                    had_cache_miss = true;

                    // Sequential execution.
                    // For each formula miss, snapshot_data() gives an O(1) Arc::clone
                    // as a stable read view. The Arc is dropped before self.eval_data.set()
                    // so Arc::make_mut always finds rc=1 — zero deep copy, zero latency.
                    time_block!("      batch sequential eval", {
                        for eval_key in batch {
                            if let Some(t) = token {
                                if t.is_cancelled() {
                                    return Err("Cancelled".to_string());
                                }
                            }
                            // Filter individual items if paths are provided
                            if let Some(filter_paths) = normalized_paths {
                                if !filter_paths.is_empty()
                                    && !filter_paths.iter().any(|p| {
                                        eval_key.starts_with(p.as_str())
                                            || (p.starts_with(eval_key.as_str())
                                                && !eval_key.contains("/$params/"))
                                    })
                                {
                                    continue;
                                }
                            }

                            let pointer_path =
                                path_utils::normalize_to_json_pointer(eval_key).into_owned();

                            // Cache miss - evaluate
                            let is_table = self.table_metadata.contains_key(eval_key);

                            if is_table {
                                time_block!("        table eval", {
                                    // Snapshot for table read access: Arc::clone is O(1).
                                    // Scoped so it's dropped before self.eval_data.set() below,
                                    // keeping self.eval_data.data at rc=1 so Arc::make_mut is free.
                                    let table_result = {
                                        let table_scope =
                                            EvalData::from_arc(self.eval_data.snapshot_data());
                                        table_evaluate::evaluate_table(
                                            self,
                                            eval_key,
                                            &table_scope,
                                            token,
                                        )
                                        // table_scope dropped here → rc back to 1
                                    };
                                    if let Ok((rows, external_deps_opt)) = table_result {
                                        let result_val = Value::Array(rows);
                                        if let Some(external_deps) = external_deps_opt {
                                            self.eval_cache.store_cache(
                                                eval_key,
                                                &external_deps,
                                                result_val.clone(),
                                            );
                                        }

                                        // NOTE: bump_params_version / bump_data_version for table results
                                        // is now handled inside store_cache (conditional on value change).
                                        // The separate bump here was double-counting: store_cache uses T2
                                        // comparison while this block used eval_data as reference point,
                                        // causing two version increments per changed table.

                                        let static_key = format!("/$table{}", pointer_path);
                                        let arc_value = std::sync::Arc::new(result_val);

                                        Arc::make_mut(&mut self.static_arrays).insert(
                                            static_key.clone(),
                                            std::sync::Arc::clone(&arc_value),
                                        );

                                        self.eval_data.set(&pointer_path, Value::clone(&arc_value));

                                        let marker =
                                            serde_json::json!({ "$static_array": static_key });
                                        if let Some(schema_value) =
                                            self.evaluated_schema.pointer_mut(&pointer_path)
                                        {
                                            *schema_value = marker;
                                        }
                                    }
                                });
                            } else {
                                let empty_deps = indexmap::IndexSet::new();
                                let deps = self.dependencies.get(eval_key).unwrap_or(&empty_deps);
                                let cached_result = self.eval_cache.check_cache(eval_key, &deps);

                                time_block!("        formula eval", {
                                    if let Some(cached_result) = cached_result {
                                        // Must still populate eval_data out of cache so subsequent formulas
                                        // referencing this path in the same iteration can read the exact value
                                        self.eval_data.set(&pointer_path, cached_result.clone());
                                        if let Some(schema_value) =
                                            self.evaluated_schema.pointer_mut(&pointer_path)
                                        {
                                            *schema_value = cached_result;
                                        }
                                    } else if let Some(logic_id) = self.evaluations.get(eval_key) {
                                        // snapshot_data() is O(1) Arc::clone — no deep copy.
                                        // Arc is moved into `snap` and lives only for the
                                        // engine.run() call, then dropped before set() below.
                                        // This keeps self.eval_data.data at rc=1 when set()
                                        // calls Arc::make_mut, so no deep clone ever occurs.
                                        let val = {
                                            let snap = self.eval_data.snapshot_data();
                                            self.engine.run(logic_id, &*snap)
                                            // snap dropped here → rc back to 1
                                        };
                                        if let Ok(val) = val {
                                            let cleaned_val = clean_float_noise_scalar(val);
                                            let data_path =
                                                pointer_path.replace("/properties/", "/");
                                            self.eval_cache.store_cache(
                                                eval_key,
                                                &deps,
                                                cleaned_val.clone(),
                                            );

                                            // Bump data_versions when non-$params field value changes.
                                            // $params bumps are handled inside store_cache (conditional).
                                            let old_val = self
                                                .eval_data
                                                .get(&data_path)
                                                .cloned()
                                                .unwrap_or(Value::Null);
                                            if cleaned_val != old_val
                                                && !data_path.starts_with("/$params")
                                            {
                                                self.eval_cache.bump_data_version(&data_path);
                                            }

                                            self.eval_data.set(&pointer_path, cleaned_val.clone());
                                            if let Some(schema_value) =
                                                self.evaluated_schema.pointer_mut(&pointer_path)
                                            {
                                                *schema_value = cleaned_val;
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    });
                }
            });

            // Drop lock before calling evaluate_others
            drop(_lock);

            // Mark generation stable so the next evaluate_internal call can detect whether
            // any formula was actually re-stored (via bump_data/params_version) since this run.
            self.eval_cache.mark_evaluated();

            self.evaluate_others(paths, token, had_cache_miss);

            Ok(())
        })
    }

    pub(crate) fn evaluate_others(
        &mut self,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
        had_cache_miss: bool,
    ) {
        if let Some(t) = token {
            if t.is_cancelled() {
                return;
            }
        }
        time_block!("    evaluate_others()", {
            // Step 1: Evaluate "rules" and "others" categories with caching
            // Rules are evaluated here so their values are available in evaluated_schema
            let combined_count = self.rules_evaluations.len() + self.others_evaluations.len();
            if combined_count > 0 {
                time_block!("      evaluate rules+others", {
                    let eval_data_snapshot = self.eval_data.clone();

                    let normalized_paths: Option<Vec<String>> = paths.map(|p_list| {
                        p_list
                            .iter()
                            .flat_map(|p| {
                                let ptr = path_utils::dot_notation_to_schema_pointer(p);
                                // Also support version with /properties/ prefix for root match
                                let with_props = if ptr.starts_with("#/") {
                                    format!("#/properties/{}", &ptr[2..])
                                } else {
                                    ptr.clone()
                                };
                                vec![ptr, with_props]
                            })
                            .collect()
                    });

                    // Sequential evaluation
                    let combined_evals: Vec<&String> = self
                        .rules_evaluations
                        .iter()
                        .chain(self.others_evaluations.iter())
                        .collect();

                    for eval_key in combined_evals {
                        if let Some(t) = token {
                            if t.is_cancelled() {
                                return;
                            }
                        }
                        // Filter items if paths are provided
                        if let Some(filter_paths) = normalized_paths.as_ref() {
                            if !filter_paths.is_empty()
                                && !filter_paths.iter().any(|p| {
                                    eval_key.starts_with(p.as_str())
                                        || (p.starts_with(eval_key.as_str())
                                            && !eval_key.contains("/$params/"))
                                })
                            {
                                continue;
                            }
                        }

                        let pointer_path =
                            path_utils::normalize_to_json_pointer(eval_key).into_owned();
                        let empty_deps = indexmap::IndexSet::new();
                        let deps = self.dependencies.get(eval_key).unwrap_or(&empty_deps);

                        if let Some(cached_result) = self.eval_cache.check_cache(eval_key, &deps) {
                            if let Some(pointer_value) =
                                self.evaluated_schema.pointer_mut(&pointer_path)
                            {
                                if !pointer_path.starts_with("$")
                                    && pointer_path.contains("/rules/")
                                    && !pointer_path.ends_with("/value")
                                {
                                    if let Some(pointer_obj) = pointer_value.as_object_mut() {
                                        pointer_obj.remove("$evaluation");
                                        pointer_obj
                                            .insert("value".to_string(), cached_result.clone());
                                    }
                                } else {
                                    *pointer_value = cached_result.clone();
                                }
                            }
                            continue;
                        }
                        if let Some(logic_id) = self.evaluations.get(eval_key) {
                            if let Ok(val) = self.engine.run(logic_id, eval_data_snapshot.data()) {
                                let cleaned_val = clean_float_noise_scalar(val);
                                self.eval_cache
                                    .store_cache(eval_key, &deps, cleaned_val.clone());

                                if let Some(pointer_value) =
                                    self.evaluated_schema.pointer_mut(&pointer_path)
                                {
                                    if !pointer_path.starts_with("$")
                                        && pointer_path.contains("/rules/")
                                        && !pointer_path.ends_with("/value")
                                    {
                                        match pointer_value.as_object_mut() {
                                            Some(pointer_obj) => {
                                                pointer_obj.remove("$evaluation");
                                                pointer_obj
                                                    .insert("value".to_string(), cleaned_val);
                                            }
                                            None => continue,
                                        }
                                    } else {
                                        *pointer_value = cleaned_val;
                                    }
                                }
                            }
                        }
                    }
                });
            }
        });

        // Step 2: Evaluate options URL templates (handles {variable} patterns)
        // Skip when all entries were cache hits — template inputs can't have changed.
        if had_cache_miss {
            time_block!("      evaluate_options_templates", {
                self.evaluate_options_templates(paths);
            });

            // Step 3: Resolve layout logic (metadata injection, hidden propagation)
            // Skip when no values changed — layout state is guaranteed identical.
            time_block!("      resolve_layout", {
                let _ = self.resolve_layout(false);
            });
        }
    }

    /// Evaluate options URL templates (handles {variable} patterns)
    fn evaluate_options_templates(&mut self, paths: Option<&[String]>) {
        // Use pre-collected options templates from parsing (Arc clone is cheap)
        let templates_to_eval = self.options_templates.clone();

        // Evaluate each template
        for (path, template_str, params_path) in templates_to_eval.iter() {
            // Filter items if paths are provided
            // 'path' here is the schema path to the field (dot notation or similar, need to check)
            // It seems to be schema pointer based on usage in other methods
            if let Some(filter_paths) = paths {
                if !filter_paths.is_empty()
                    && !filter_paths
                        .iter()
                        .any(|p| path.starts_with(p.as_str()) || p.starts_with(path.as_str()))
                {
                    continue;
                }
            }

            if let Some(params) = self.evaluated_schema.pointer(&params_path) {
                if let Ok(evaluated) = self.evaluate_template(&template_str, params) {
                    if let Some(target) = self.evaluated_schema.pointer_mut(&path) {
                        *target = Value::String(evaluated);
                    }
                }
            }
        }
    }

    /// Evaluate a template string like "api/users/{id}" with params
    fn evaluate_template(&self, template: &str, params: &Value) -> Result<String, String> {
        let mut result = template.to_string();

        // Simple template evaluation: replace {key} with params.key
        if let Value::Object(params_map) = params {
            for (key, value) in params_map {
                let placeholder = format!("{{{}}}", key);
                if let Some(str_val) = value.as_str() {
                    result = result.replace(&placeholder, str_val);
                } else {
                    // Convert non-string values to strings
                    result = result.replace(&placeholder, &value.to_string());
                }
            }
        }

        Ok(result)
    }
}
