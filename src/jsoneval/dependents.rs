use super::JSONEval;
use crate::jsoneval::cancellation::CancellationToken;
use crate::jsoneval::json_parser;
use crate::jsoneval::path_utils;
use crate::jsoneval::path_utils::get_value_by_pointer_without_properties;
use crate::jsoneval::path_utils::normalize_to_json_pointer;
use crate::jsoneval::types::DependentItem;
use crate::rlogic::{LogicId, RLogic};
use crate::time_block;
use crate::utils::clean_float_noise_scalar;
use crate::EvalData;

use indexmap::{IndexMap, IndexSet};
use serde_json::Value;

impl JSONEval {
    /// Evaluate fields that depend on a changed path.
    /// Processes all dependent fields transitively, then optionally performs a full
    /// re-evaluation pass (for read-only / hide effects) and cascades into subforms.
    pub fn evaluate_dependents(
        &mut self,
        changed_paths: &[String],
        data: Option<&str>,
        context: Option<&str>,
        re_evaluate: bool,
        token: Option<&CancellationToken>,
        mut canceled_paths: Option<&mut Vec<String>>,
        include_subforms: bool,
    ) -> Result<Value, String> {
        if let Some(t) = token {
            if t.is_cancelled() {
                return Err("Cancelled".to_string());
            }
        }
        let _lock = self.eval_lock.lock().unwrap();
        let mut structural_change_data = None;

        // Update data if provided, diff versions
        if let Some(data_str) = data {
            let data_value = json_parser::parse_json_str(data_str)?;
            let context_value = if let Some(ctx) = context {
                json_parser::parse_json_str(ctx)?
            } else {
                Value::Object(serde_json::Map::new())
            };
            let old_data = self.eval_data.snapshot_data_clone();
            time_block!("  [dep] data_replace_and_context", {
                self.eval_data
                    .replace_data_and_context(data_value, context_value);
            });
            let new_data = self.eval_data.snapshot_data_clone();
            time_block!("  [dep] data_diff_versions", {
                self.eval_cache
                    .store_snapshot_and_diff_versions(&old_data, &new_data);
            });
            structural_change_data = Some((old_data, new_data));
        }

        // Drop the lock before calling sub-methods that need &mut self
        drop(_lock);

        // When a subform array changes structurally (riders added/removed/reordered),
        // evict stale T2 global cache entries whose dep paths use the subform-local key
        // format that is never bumped by the parent-level diff.
        if let Some((old_data, new_data)) = structural_change_data {
            time_block!("  [dep] invalidate_subform_structural", {
                self.invalidate_subform_caches_on_structural_change(&old_data, &new_data);
            });
        }

        let mut result = Vec::new();
        let mut processed = std::collections::HashMap::new();
        let mut to_process: Vec<(String, bool, Option<Vec<usize>>)> = changed_paths
            .iter()
            .map(|path| {
                (
                    path_utils::dot_notation_to_schema_pointer(path),
                    false,
                    None,
                )
            })
            .collect();

        time_block!("  [dep] process_dependents_queue", {
            Self::process_dependents_queue(
                &self.engine,
                &self.evaluations,
                &mut self.eval_data,
                &mut self.eval_cache,
                &self.dependents_evaluations,
                &self.dep_formula_triggers,
                &self.evaluated_schema,
                &mut to_process,
                &mut processed,
                &mut result,
                token,
                canceled_paths.as_mut().map(|v| &mut **v),
            )?;
        });

        if re_evaluate {
            time_block!("  [dep] run_re_evaluate_pass", {
                self.run_re_evaluate_pass(
                    token,
                    &mut to_process,
                    &mut processed,
                    &mut result,
                    canceled_paths.as_mut().map(|v| &mut **v),
                )?;
            });
        }

        if include_subforms {
            // Augment changed_paths with every subform item field already written into result
            // by the dependents queue and re-evaluate pass. Without this, when a main-form
            // dependent rule writes to e.g. `riders.0.benefit`, that path never appears in
            // `item_changed_paths` inside run_subform_pass → the item is skipped → the
            // subform item's own `benefit.dependents` never fire.
            let extended_paths: Vec<String> = {
                let mut paths = changed_paths.to_vec();
                for item in &result {
                    if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                        let s = ref_val.to_string();
                        if !paths.contains(&s) {
                            paths.push(s);
                        }
                    }
                }
                paths
            };
            time_block!("  [dep] run_subform_pass", {
                self.run_subform_pass(&extended_paths, re_evaluate, token, &mut result)?;
            });
        }

        // Deduplicate by $ref — keep the last entry for each path.
        // Multiple passes (dependents queue, re-evaluate, subform) may independently emit
        // the same $ref when cache versions cause overlapping detections. The subform pass
        // result is most specific and wins because it is appended last.
        let deduped = {
            let mut seen: IndexMap<String, usize> = IndexMap::new();
            for (i, item) in result.iter().enumerate() {
                if let Some(r) = item.get("$ref").and_then(|v| v.as_str()) {
                    seen.insert(r.to_string(), i);
                }
            }
            let last_indices: IndexSet<usize> = seen.values().copied().collect();
            let out: Vec<Value> = result
                .into_iter()
                .enumerate()
                .filter(|(i, _)| last_indices.contains(i))
                .map(|(_, item)| item)
                .collect();
            out
        };

        // Refresh main_form_snapshot so the next evaluate() call computes diffs from the
        // post-dependents state instead of the old pre-dependents snapshot.
        // Without this, evaluate() re-diffs every field that evaluate_dependents already
        // processed, double-bumping data_versions and causing spurious cache misses in
        // evaluate_internal (observed as unexpected ~550ms "cache hit" full evaluates).
        // Only update when no subform item is active — subform evaluate_dependents calls
        // must not overwrite the parent's snapshot.
        if self.eval_cache.active_item_index.is_none() {
            let current_snapshot = self.eval_data.snapshot_data_clone();
            self.eval_cache.main_form_snapshot = Some(current_snapshot);
        }

        Ok(Value::Array(deduped))
    }

    /// Full re-evaluation pass: runs `evaluate_internal`, then applies read-only fixes and
    /// recursive hide effects, feeding any newly-generated changes back into the dependents queue.
    fn run_re_evaluate_pass(
        &mut self,
        token: Option<&CancellationToken>,
        to_process: &mut Vec<(String, bool, Option<Vec<usize>>)>,
        processed: &mut std::collections::HashMap<String, Option<std::collections::HashSet<usize>>>,
        result: &mut Vec<Value>,
        mut canceled_paths: Option<&mut Vec<String>>,
    ) -> Result<(), String> {
        // --- Schema Default Value Pass (Before Eval) ---
        self.run_schema_default_value_pass(
            token,
            to_process,
            processed,
            result,
            canceled_paths.as_mut().map(|v| &mut **v),
        )?;

        // Resolve the correct data_versions tracker before snapshotting.
        // When active_item_index is Some(idx), evaluate_internal bumps
        // subform_caches[idx].data_versions — NOT the main data_versions.
        // Using the main tracker for both snapshot and post-eval lookup would make
        // old_ver == new_ver always, so no changed values would ever be emitted.
        let pre_eval_versions = if let Some(idx) = self.eval_cache.active_item_index {
            self.eval_cache
                .subform_caches
                .get(&idx)
                .map(|c| c.data_versions.clone())
                .unwrap_or_else(|| self.eval_cache.data_versions.clone())
        } else {
            self.eval_cache.data_versions.clone()
        };

        self.evaluate_internal(None, token)?;

        // --- Schema Default Value Pass (After Eval) ---
        self.run_schema_default_value_pass(
            token,
            to_process,
            processed,
            result,
            canceled_paths.as_mut().map(|v| &mut **v),
        )?;

        // Emit result entries for every sorted-evaluation whose version uniquely bumped.
        let active_idx = self.eval_cache.active_item_index;
        for eval_key in self.sorted_evaluations.iter().flatten() {
            if eval_key.contains("/$params/") || eval_key.contains("/$") {
                continue;
            }

            let schema_ptr = path_utils::normalize_to_json_pointer(eval_key);
            let data_path = schema_ptr
                .replace("/properties/", "/")
                .trim_start_matches('#')
                .trim_start_matches('/')
                .to_string();

            let version_path = format!("/{}", data_path);
            let old_ver = pre_eval_versions.get(&version_path);
            let new_ver = if let Some(idx) = active_idx {
                self.eval_cache
                    .subform_caches
                    .get(&idx)
                    .map(|c| c.data_versions.get(&version_path))
                    .unwrap_or_else(|| self.eval_cache.data_versions.get(&version_path))
            } else {
                self.eval_cache.data_versions.get(&version_path)
            };

            if new_ver > old_ver {
                if let Some(new_val) = self.evaluated_schema.pointer(&schema_ptr) {
                    let dot_path = data_path.trim_end_matches("/value").replace('/', ".");
                    let mut obj = serde_json::Map::new();
                    obj.insert("$ref".to_string(), Value::String(dot_path));
                    let is_clear = new_val == &Value::Null || new_val.as_str() == Some("");
                    if is_clear {
                        obj.insert("clear".to_string(), Value::Bool(true));
                    } else {
                        obj.insert("value".to_string(), new_val.clone());
                    }
                    result.push(Value::Object(obj));
                }
            }
        }

        // Re-acquire lock for post-eval passes
        let _lock = self.eval_lock.lock().unwrap();

        // --- Read-Only Pass ---
        let mut readonly_changes = Vec::new();
        let mut readonly_values = Vec::new();
        for path in self.conditional_readonly_fields.iter() {
            let normalized = path_utils::normalize_to_json_pointer(path);
            if let Some(schema_el) = self.evaluated_schema.pointer(&normalized) {
                self.check_readonly_for_dependents(
                    schema_el,
                    path,
                    &mut readonly_changes,
                    &mut readonly_values,
                );
            }
        }
        // Capture count before drain so had_actual_readonly_changes is not confused by
        // pre-existing to_process entries from schema_default_before or process_dependents_queue.
        let had_actual_readonly_changes = !readonly_changes.is_empty();
        for (path, schema_value) in readonly_changes {
            let data_path = path_utils::normalize_to_json_pointer(&path)
                .replace("/properties/", "/")
                .trim_start_matches('#')
                .to_string();
            self.eval_data.set(&data_path, schema_value.clone());
            self.eval_cache.bump_data_version(&data_path);
            to_process.push((path, true, None));
        }
        for (path, schema_value) in readonly_values {
            let data_path = path_utils::normalize_to_json_pointer(&path)
                .replace("/properties/", "/")
                .trim_start_matches('#')
                .to_string();
            let mut obj = serde_json::Map::new();
            obj.insert(
                "$ref".to_string(),
                Value::String(path_utils::pointer_to_dot_notation(&data_path)),
            );
            obj.insert("$readonly".to_string(), Value::Bool(true));
            let is_clear = schema_value == Value::Null || schema_value.as_str() == Some("");
            if is_clear {
                obj.insert("clear".to_string(), Value::Bool(true));
            } else {
                obj.insert("value".to_string(), schema_value);
            }
            result.push(Value::Object(obj));
        }

        // When readonly fields were updated, re-run evaluate_internal only for the subset of
        // $params tables that actually depend on the changed readonly fields.
        // Use had_actual_readonly_changes (captured before draining into to_process) — NOT
        // `!to_process.is_empty()`, which would fire whenever any dependents were queued,
        // triggering a spurious second evaluate_internal per rider costing ~240ms each.
        if had_actual_readonly_changes {
            if let Some(active_idx) = self.eval_cache.active_item_index {
                // Collect the schema-dep paths for the readonly-changed fields so we can
                // filter tables to only those that actually depend on these fields.
                // Bumping ALL $params tables unconditionally forces a full re-evaluate of
                // every WOP/RIDER table even when they don't read wop_rider_premi etc.
                let readonly_dep_prefixes: Vec<String> =
                    to_process.iter().map(|(path, _, _)| path.clone()).collect();

                let params_table_keys: Vec<String> = self
                    .table_metadata
                    .keys()
                    .filter(|k| {
                        if !k.starts_with("#/$params") {
                            return false;
                        }
                        // Only invalidate tables that depend on one of the readonly fields
                        if let Some(deps) = self.dependencies.get(*k) {
                            deps.iter().any(|dep| {
                                readonly_dep_prefixes
                                    .iter()
                                    .any(|ro| dep == ro || dep.starts_with(ro.as_str()))
                            })
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();

                if !params_table_keys.is_empty() {
                    self.eval_cache
                        .invalidate_params_tables_for_item(active_idx, &params_table_keys);
                    drop(_lock);
                    self.evaluate_internal(None, token)?;
                }
            }
        }

        if !to_process.is_empty() {
            Self::process_dependents_queue(
                &self.engine,
                &self.evaluations,
                &mut self.eval_data,
                &mut self.eval_cache,
                &self.dependents_evaluations,
                &self.dep_formula_triggers,
                &self.evaluated_schema,
                to_process,
                processed,
                result,
                token,
                canceled_paths.as_mut().map(|v| &mut **v),
            )?;
        }

        // --- Recursive Hide Pass ---
        let mut hidden_fields = Vec::new();
        for path in self.conditional_hidden_fields.iter() {
            let normalized = path_utils::normalize_to_json_pointer(path);
            if let Some(schema_el) = self.evaluated_schema.pointer(&normalized) {
                self.check_hidden_field(schema_el, path, &mut hidden_fields);
            }
        }
        if !hidden_fields.is_empty() {
            Self::recursive_hide_effect(
                &self.engine,
                &self.evaluations,
                &self.reffed_by,
                &mut self.eval_data,
                &mut self.eval_cache,
                hidden_fields,
                to_process,
                result,
            );
        }
        if !to_process.is_empty() {
            Self::process_dependents_queue(
                &self.engine,
                &self.evaluations,
                &mut self.eval_data,
                &mut self.eval_cache,
                &self.dependents_evaluations,
                &self.dep_formula_triggers,
                &self.evaluated_schema,
                to_process,
                processed,
                result,
                token,
                canceled_paths.as_mut().map(|v| &mut **v),
            )?;
        }

        Ok(())
    }

    /// Internal method to run the schema default value pass.
    /// Filters for only primitive schema values (not $evaluation objects).
    fn run_schema_default_value_pass(
        &mut self,
        token: Option<&CancellationToken>,
        to_process: &mut Vec<(String, bool, Option<Vec<usize>>)>,
        processed: &mut std::collections::HashMap<String, Option<std::collections::HashSet<usize>>>,
        result: &mut Vec<Value>,
        mut canceled_paths: Option<&mut Vec<String>>,
    ) -> Result<(), String> {
        let mut default_value_changes = Vec::new();
        let schema_values = self.get_schema_value_array();

        if let Value::Array(values) = schema_values {
            for item in values {
                if let Value::Object(map) = item {
                    if let (Some(Value::String(dot_path)), Some(schema_val)) =
                        (map.get("path"), map.get("value"))
                    {
                        let schema_ptr = path_utils::dot_notation_to_schema_pointer(dot_path);
                        if let Some(Value::Object(schema_node)) = self
                            .evaluated_schema
                            .pointer(schema_ptr.trim_start_matches('#'))
                        {
                            if let Some(Value::Object(condition)) = schema_node.get("condition") {
                                if let Some(hidden_val) = condition.get("hidden") {
                                    // Skip if hidden is true OR if it's a non-primitive value (formula object)
                                    if !hidden_val.is_boolean()
                                        || hidden_val.as_bool() == Some(true)
                                    {
                                        continue;
                                    }
                                }
                            }
                        }

                        let data_path = dot_path.replace('.', "/");
                        let current_data = self
                            .eval_data
                            .data()
                            .pointer(&format!("/{}", data_path))
                            .unwrap_or(&Value::Null);

                        let is_empty = match current_data {
                            Value::Null => true,
                            Value::String(s) if s.is_empty() => true,
                            _ => false,
                        };

                        let is_schema_val_empty = match schema_val {
                            Value::Null => true,
                            Value::String(s) if s.is_empty() => true,
                            Value::Object(map) if map.contains_key("$evaluation") => true,
                            _ => false,
                        };

                        if is_empty && !is_schema_val_empty && current_data != schema_val {
                            default_value_changes.push((
                                data_path,
                                schema_val.clone(),
                                dot_path.clone(),
                            ));
                        }
                    }
                }
            }
        }

        let mut has_changes = false;
        for (data_path, schema_val, dot_path) in default_value_changes {
            self.eval_data
                .set(&format!("/{}", data_path), schema_val.clone());
            self.eval_cache
                .bump_data_version(&format!("/{}", data_path));

            let mut change_obj = serde_json::Map::new();
            change_obj.insert("$ref".to_string(), Value::String(dot_path));
            let is_clear = schema_val == Value::Null || schema_val.as_str() == Some("");
            if is_clear {
                change_obj.insert("clear".to_string(), Value::Bool(true));
            } else {
                change_obj.insert("value".to_string(), schema_val);
            }
            result.push(Value::Object(change_obj));

            let schema_ptr = format!("#/{}", data_path.replace('/', "/properties/"));
            to_process.push((schema_ptr, true, None));
            has_changes = true;
        }

        if has_changes {
            Self::process_dependents_queue(
                &self.engine,
                &self.evaluations,
                &mut self.eval_data,
                &mut self.eval_cache,
                &self.dependents_evaluations,
                &self.dep_formula_triggers,
                &self.evaluated_schema,
                to_process,
                processed,
                result,
                token,
                canceled_paths.as_mut().map(|v| &mut **v),
            )?;
        }

        Ok(())
    }

    /// Cascade dependency evaluation into each subform item.
    ///
    /// For every registered subform, this method iterates over its array items and runs
    /// `evaluate_dependents` on the subform using the cache-swap strategy so the subform
    /// can see global main-form Tier 2 cache entries (avoiding redundant table re-evaluation).
    ///
    /// `sub_re_evaluate` is set **only** when the parent's bumped `data_versions` intersect
    /// with paths the subform actually depends on — preventing expensive full re-evals on
    /// subform items whose dependencies did not change.
    fn run_subform_pass(
        &mut self,
        changed_paths: &[String],
        re_evaluate: bool,
        token: Option<&CancellationToken>,
        result: &mut Vec<Value>,
    ) -> Result<(), String> {
        // Collect subform paths once (avoids holding borrow on self.subforms during mutation)
        let subform_paths: Vec<String> = self.subforms.keys().cloned().collect();

        for subform_path in subform_paths {
            let field_key = subform_field_key(&subform_path);
            // Compute dotted path and prefix strings once per subform, not per item
            let subform_dot_path =
                path_utils::pointer_to_dot_notation(&subform_path).replace(".properties.", ".");
            let field_prefix = format!("{}.", field_key);
            let subform_ptr = normalize_to_json_pointer(&subform_path);

            // Borrow only the item count first — avoid cloning the full array
            let item_count =
                get_value_by_pointer_without_properties(self.eval_data.data(), &subform_ptr)
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);

            if item_count == 0 {
                continue;
            }

            // Evict stale per-item caches for indices that no longer exist in the array.
            // This prevents memory leaks when riders are removed and the array shrinks.
            self.eval_cache.prune_subform_caches(item_count);

            // When the parent ran a re_evaluate pass, always pass re_evaluate:true to subforms.
            // The parent's evaluate_internal may have updated $params or other referenced values
            // that the subform formulas read, even if none of the subform's own dep paths bumped.
            let global_sub_re_evaluate = re_evaluate;

            // Snapshot the parent's version trackers once, before iterating any riders.
            // Using the live `parent_cache.data_versions` inside the loop would let rider N's
            // evaluation bumps contaminate the merge_from baseline for rider M (M ≠ N),
            // causing cache misses and wrong re-evaluations on subsequent visits to rider M.
            let parent_data_versions_snapshot = self.eval_cache.data_versions.clone();
            let parent_params_versions_snapshot = self.eval_cache.params_versions.clone();

            for idx in 0..item_count {
                // Map absolute changed paths → subform-internal paths for this item index
                let prefix_dot = format!("{}.{}.", subform_dot_path, idx);
                let prefix_bracket = format!("{}[{}].", subform_dot_path, idx);
                let prefix_field_bracket = format!("{}[{}].", field_key, idx);

                let item_changed_paths: Vec<String> = changed_paths
                    .iter()
                    .filter_map(|p| {
                        if p.starts_with(&prefix_bracket) {
                            Some(p.replacen(&prefix_bracket, &field_prefix, 1))
                        } else if p.starts_with(&prefix_dot) {
                            Some(p.replacen(&prefix_dot, &field_prefix, 1))
                        } else if p.starts_with(&prefix_field_bracket) {
                            Some(p.replacen(&prefix_field_bracket, &field_prefix, 1))
                        } else {
                            None
                        }
                    })
                    .collect();

                let sub_re_evaluate = global_sub_re_evaluate || !item_changed_paths.is_empty();

                // Skip entirely if there's nothing to do for this item
                if !sub_re_evaluate && item_changed_paths.is_empty() {
                    continue;
                }

                // Build minimal merged data: clone only item at idx, share $params shallowly.
                // This avoids cloning the full 5MB parent payload for every item.
                let item_val =
                    get_value_by_pointer_without_properties(self.eval_data.data(), &subform_ptr)
                        .and_then(|v| v.as_array())
                        .and_then(|a| a.get(idx))
                        .cloned()
                        .unwrap_or(Value::Null);

                // Build a minimal parent object with only the fields the subform needs:
                // the item under field_key, plus all non-array top-level parent fields
                // ($params markers, scalars). Large arrays are already stripped to static_arrays.
                let merged_data = {
                    let parent = self.eval_data.data();
                    let mut map = serde_json::Map::new();
                    if let Value::Object(parent_map) = parent {
                        for (k, v) in parent_map {
                            if k == &field_key {
                                // Will be overridden with the single item below
                                continue;
                            }
                            // Include scalars, objects ($params markers, etc.) but skip
                            // other large array fields that aren't this subform
                            if !v.is_array() {
                                map.insert(k.clone(), v.clone());
                            }
                        }
                    }
                    map.insert(field_key.clone(), item_val.clone());
                    Value::Object(map)
                };

                let Some(subform) = self.subforms.get_mut(&subform_path) else {
                    continue;
                };

                // Prepare cache state for this item
                self.eval_cache.ensure_active_item_cache(idx);
                let old_item_val = self
                    .eval_cache
                    .subform_caches
                    .get(&idx)
                    .map(|c| c.item_snapshot.clone())
                    .unwrap_or(Value::Null);

                subform.eval_data.replace_data_and_context(
                    merged_data,
                    self.eval_data
                        .data()
                        .get("$context")
                        .cloned()
                        .unwrap_or(Value::Null),
                );
                let new_item_val = subform
                    .eval_data
                    .data()
                    .get(&field_key)
                    .cloned()
                    .unwrap_or(Value::Null);

                // Cache-swap: lend parent cache to subform
                let mut parent_cache = std::mem::take(&mut self.eval_cache);
                parent_cache.ensure_active_item_cache(idx);
                if let Some(c) = parent_cache.subform_caches.get_mut(&idx) {
                    // Merge all data versions from the parent snapshot. We must include non-$params
                    // paths so that parent field updates (like wop_basic_benefit changing) correctly
                    // invalidate subform per-item cache entries that depend on them.
                    c.data_versions.merge_from(&parent_data_versions_snapshot);
                    // Always reflect the latest $params (schema-level, index-independent).
                    c.data_versions
                        .merge_from_params(&parent_params_versions_snapshot);
                    crate::jsoneval::eval_cache::diff_and_update_versions(
                        &mut c.data_versions,
                        &format!("/{}", field_key),
                        &old_item_val,
                        &new_item_val,
                    );
                    c.item_snapshot = new_item_val;
                }
                parent_cache.set_active_item(idx);
                std::mem::swap(&mut subform.eval_cache, &mut parent_cache);

                let subform_result = time_block!("    [subform_pass] rider evaluate_dependents", {
                    subform.evaluate_dependents(
                        &item_changed_paths,
                        None,
                        None,
                        sub_re_evaluate,
                        token,
                        None,
                        false,
                    )
                });

                // Restore parent cache
                std::mem::swap(&mut subform.eval_cache, &mut parent_cache);
                parent_cache.clear_active_item();

                // Propagate the updated item_snapshot from the parent's T1 cache into the
                // subform's own eval_cache. Without this, subsequent evaluate_subform() calls
                // for this idx read the OLD snapshot (pre-run_subform_pass) and see a diff
                // against the new data → item_paths_bumped = true → spurious table invalidation.
                if let Some(parent_item_cache) = self.eval_cache.subform_caches.get(&idx) {
                    let snapshot = parent_item_cache.item_snapshot.clone();
                    subform.eval_cache.ensure_active_item_cache(idx);
                    if let Some(sub_cache) = subform.eval_cache.subform_caches.get_mut(&idx) {
                        sub_cache.item_snapshot = snapshot;
                    }
                }

                self.eval_cache = parent_cache;

                if let Ok(Value::Array(changes)) = subform_result {
                    let mut had_any_change = false;
                    for change in changes {
                        if let Some(obj) = change.as_object() {
                            if let Some(Value::String(ref_path)) = obj.get("$ref") {
                                // Remap the $ref path to include the parent path + item index
                                let new_ref = if ref_path.starts_with(&field_prefix) {
                                    format!(
                                        "{}.{}.{}",
                                        subform_dot_path,
                                        idx,
                                        &ref_path[field_prefix.len()..]
                                    )
                                } else {
                                    format!("{}.{}.{}", subform_dot_path, idx, ref_path)
                                };

                                // Write the computed value back to parent eval_data so subsequent
                                // evaluate_subform calls see an up-to-date old_item_snapshot.
                                // Without this, the diff in with_item_cache_swap sees stale parent
                                // data vs the new call's apply_changes values → spurious item bumps
                                // → invalidate_params_tables_for_item fires → eval_generation bumps.
                                if let Some(val) = obj.get("value") {
                                    let data_ptr = format!("/{}", new_ref.replace('.', "/"));
                                    self.eval_data.set(&data_ptr, val.clone());
                                    had_any_change = true;
                                } else if obj.get("clear").and_then(Value::as_bool) == Some(true) {
                                    let data_ptr = format!("/{}", new_ref.replace('.', "/"));
                                    self.eval_data.set(&data_ptr, Value::Null);
                                    had_any_change = true;
                                }

                                let mut new_obj = obj.clone();
                                new_obj.insert("$ref".to_string(), Value::String(new_ref));
                                result.push(Value::Object(new_obj));
                            } else {
                                // No $ref rewrite needed — push as-is without cloning the map
                                result.push(change);
                            }
                        }
                    }

                    // After writing computed outputs (first_prem, wop_rider_premi, etc.) back to
                    // parent eval_data, refresh the item_snapshot so that subsequent evaluate_subform
                    // calls see the post-computation state as their baseline. Without this, the
                    // snapshot only contains the raw input (before apply_changes), so the next
                    // with_item_cache_swap diff detects ALL computed fields (first_prem, code, sa ...)
                    // as "changed" even when only genuinely-new data arrived, causing spurious
                    // secondary version bumps → false T2 table misses for RIDER_ZLOB_TABLE etc.
                    if had_any_change {
                        let item_path = format!("{}/{}", subform_ptr, idx);
                        let updated_item = self
                            .eval_data
                            .get(&item_path)
                            .cloned()
                            .unwrap_or(Value::Null);
                        // Update parent T1 cache snapshot
                        if let Some(c) = self.eval_cache.subform_caches.get_mut(&idx) {
                            c.item_snapshot = updated_item.clone();
                        }
                        // Update subform's own per-item snapshot used as old_item_snapshot
                        // on the next evaluate_subform call.
                        subform.eval_cache.ensure_active_item_cache(idx);
                        if let Some(sub_cache) = subform.eval_cache.subform_caches.get_mut(&idx) {
                            sub_cache.item_snapshot = updated_item;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Helper to evaluate a dependent value - uses pre-compiled eval keys for fast lookup
    pub(crate) fn evaluate_dependent_value_static(
        engine: &RLogic,
        evaluations: &IndexMap<String, LogicId>,
        eval_data: &EvalData,
        value: &Value,
        changed_field_value: &Value,
        changed_field_ref_value: &Value,
    ) -> Result<Value, String> {
        match value {
            // If it's a String, check if it's an eval key reference
            Value::String(eval_key) => {
                if let Some(logic_id) = evaluations.get(eval_key) {
                    // It's a pre-compiled evaluation - run it with scoped context
                    // Create internal context with $value and $refValue
                    let mut internal_context = serde_json::Map::new();
                    internal_context.insert("$value".to_string(), changed_field_value.clone());
                    internal_context.insert("$refValue".to_string(), changed_field_ref_value.clone());
                    let context_value = Value::Object(internal_context);

                    let result = engine.run_with_context(logic_id, eval_data.data(), &context_value)
                        .map_err(|e| format!("Failed to evaluate dependent logic '{}': {}", eval_key, e))?;
                    Ok(result)
                } else {
                    // It's a regular string value
                    Ok(value.clone())
                }
            }
            // For backwards compatibility: compile $evaluation on-the-fly
            // This shouldn't happen with properly parsed schemas
            Value::Object(map) if map.contains_key("$evaluation") => {
                Err("Dependent evaluation contains unparsed $evaluation - schema was not properly parsed".to_string())
            }
            // Primitive value - return as-is
            _ => Ok(value.clone()),
        }
    }

    /// Check if a single field is readonly and populate vectors for both changes and all values
    pub(crate) fn check_readonly_for_dependents(
        &self,
        schema_element: &Value,
        path: &str,
        changes: &mut Vec<(String, Value)>,
        all_values: &mut Vec<(String, Value)>,
    ) {
        match schema_element {
            Value::Object(map) => {
                // Check if field is disabled (ReadOnly)
                let mut is_disabled = false;
                if let Some(Value::Object(condition)) = map.get("condition") {
                    if let Some(Value::Bool(d)) = condition.get("disabled") {
                        is_disabled = *d;
                    }
                }

                // Check skipReadOnlyValue config
                let mut skip_readonly = false;
                if let Some(Value::Object(config)) = map.get("config") {
                    if let Some(Value::Object(all)) = config.get("all") {
                        if let Some(Value::Bool(skip)) = all.get("skipReadOnlyValue") {
                            skip_readonly = *skip;
                        }
                    }
                }

                if is_disabled && !skip_readonly {
                    if let Some(schema_value) = map.get("value") {
                        let data_path = path_utils::normalize_to_json_pointer(path)
                            .replace("/properties/", "/")
                            .trim_start_matches('#')
                            .to_string();

                        let current_data = self
                            .eval_data
                            .data()
                            .pointer(&data_path)
                            .unwrap_or(&Value::Null);

                        // Add to all_values (include in dependents result regardless of change)
                        all_values.push((path.to_string(), schema_value.clone()));

                        // Only add to changes if value doesn't match
                        if current_data != schema_value {
                            changes.push((path.to_string(), schema_value.clone()));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Recursively collect read-only fields that need updates (Legacy/Full-Scan)
    #[allow(dead_code)]
    pub(crate) fn collect_readonly_fixes(
        &self,
        schema_element: &Value,
        path: &str,
        changes: &mut Vec<(String, Value)>,
    ) {
        match schema_element {
            Value::Object(map) => {
                // Check if field is disabled (ReadOnly)
                let mut is_disabled = false;
                if let Some(Value::Object(condition)) = map.get("condition") {
                    if let Some(Value::Bool(d)) = condition.get("disabled") {
                        is_disabled = *d;
                    }
                }

                // Check skipReadOnlyValue config
                let mut skip_readonly = false;
                if let Some(Value::Object(config)) = map.get("config") {
                    if let Some(Value::Object(all)) = config.get("all") {
                        if let Some(Value::Bool(skip)) = all.get("skipReadOnlyValue") {
                            skip_readonly = *skip;
                        }
                    }
                }

                if is_disabled && !skip_readonly {
                    // Check if it's a value field (has "value" property or implicit via path?)
                    // In JS: "const readOnlyValues = this.getSchemaValues();"
                    // We only care if data != schema value
                    if let Some(schema_value) = map.get("value") {
                        let data_path = path_utils::normalize_to_json_pointer(path)
                            .replace("/properties/", "/")
                            .trim_start_matches('#')
                            .to_string();

                        let current_data = self
                            .eval_data
                            .data()
                            .pointer(&data_path)
                            .unwrap_or(&Value::Null);

                        if current_data != schema_value {
                            changes.push((path.to_string(), schema_value.clone()));
                        }
                    }
                }

                // Recurse into properties
                if let Some(Value::Object(props)) = map.get("properties") {
                    for (key, val) in props {
                        let next_path = if path == "#" {
                            format!("#/properties/{}", key)
                        } else {
                            format!("{}/properties/{}", path, key)
                        };
                        self.collect_readonly_fixes(val, &next_path, changes);
                    }
                }
            }
            _ => {}
        }
    }

    /// Check if a single field is hidden and needs clearing (Optimized non-recursive)
    pub(crate) fn check_hidden_field(
        &self,
        schema_element: &Value,
        path: &str,
        hidden_fields: &mut Vec<String>,
    ) {
        match schema_element {
            Value::Object(map) => {
                // Check if field is hidden
                let mut is_hidden = false;
                if let Some(Value::Object(condition)) = map.get("condition") {
                    if let Some(Value::Bool(h)) = condition.get("hidden") {
                        is_hidden = *h;
                    }
                }

                // Check keepHiddenValue config
                let mut keep_hidden = false;
                if let Some(Value::Object(config)) = map.get("config") {
                    if let Some(Value::Object(all)) = config.get("all") {
                        if let Some(Value::Bool(keep)) = all.get("keepHiddenValue") {
                            keep_hidden = *keep;
                        }
                    }
                }

                if is_hidden && !keep_hidden {
                    let data_path = path_utils::normalize_to_json_pointer(path)
                        .replace("/properties/", "/")
                        .trim_start_matches('#')
                        .to_string();

                    let current_data = self
                        .eval_data
                        .data()
                        .pointer(&data_path)
                        .unwrap_or(&Value::Null);

                    // If hidden and has non-empty value, add to list
                    if current_data != &Value::Null && current_data != "" {
                        hidden_fields.push(path.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    /// Recursively collect hidden fields that have values (candidates for clearing) (Legacy/Full-Scan)
    #[allow(dead_code)]
    pub(crate) fn collect_hidden_fields(
        &self,
        schema_element: &Value,
        path: &str,
        hidden_fields: &mut Vec<String>,
    ) {
        match schema_element {
            Value::Object(map) => {
                // Check if field is hidden
                let mut is_hidden = false;
                if let Some(Value::Object(condition)) = map.get("condition") {
                    if let Some(Value::Bool(h)) = condition.get("hidden") {
                        is_hidden = *h;
                    }
                }

                // Check keepHiddenValue config
                let mut keep_hidden = false;
                if let Some(Value::Object(config)) = map.get("config") {
                    if let Some(Value::Object(all)) = config.get("all") {
                        if let Some(Value::Bool(keep)) = all.get("keepHiddenValue") {
                            keep_hidden = *keep;
                        }
                    }
                }

                if is_hidden && !keep_hidden {
                    let data_path = path_utils::normalize_to_json_pointer(path)
                        .replace("/properties/", "/")
                        .trim_start_matches('#')
                        .to_string();

                    let current_data = self
                        .eval_data
                        .data()
                        .pointer(&data_path)
                        .unwrap_or(&Value::Null);

                    // If hidden and has non-empty value, add to list
                    if current_data != &Value::Null && current_data != "" {
                        hidden_fields.push(path.to_string());
                    }
                }

                // Recurse into children
                for (key, val) in map {
                    if key == "properties" {
                        if let Value::Object(props) = val {
                            for (p_key, p_val) in props {
                                let next_path = if path == "#" {
                                    format!("#/properties/{}", p_key)
                                } else {
                                    format!("{}/properties/{}", path, p_key)
                                };
                                self.collect_hidden_fields(p_val, &next_path, hidden_fields);
                            }
                        }
                    } else if let Value::Object(_) = val {
                        // Skip known metadata keys and explicitly handled keys
                        if key == "condition"
                            || key == "config"
                            || key == "rules"
                            || key == "dependents"
                            || key == "hideLayout"
                            || key == "$layout"
                            || key == "$params"
                            || key == "definitions"
                            || key == "$defs"
                            || key.starts_with('$')
                        {
                            continue;
                        }

                        let next_path = if path == "#" {
                            format!("#/{}", key)
                        } else {
                            format!("{}/{}", path, key)
                        };
                        self.collect_hidden_fields(val, &next_path, hidden_fields);
                    }
                }
            }
            _ => {}
        }
    }

    /// Perform recursive hiding effect using reffed_by graph.
    /// Collects every data path that gets nulled into `invalidated_paths`.
    pub(crate) fn recursive_hide_effect(
        engine: &RLogic,
        evaluations: &IndexMap<String, LogicId>,
        reffed_by: &IndexMap<String, Vec<String>>,
        eval_data: &mut EvalData,
        eval_cache: &mut crate::jsoneval::eval_cache::EvalCache,
        mut hidden_fields: Vec<String>,
        queue: &mut Vec<(String, bool, Option<Vec<usize>>)>,
        result: &mut Vec<Value>,
    ) {
        while let Some(hf) = hidden_fields.pop() {
            let data_path = path_utils::normalize_to_json_pointer(&hf)
                .replace("/properties/", "/")
                .trim_start_matches('#')
                .to_string();

            // clear data
            eval_data.set(&data_path, Value::Null);
            eval_cache.bump_data_version(&data_path);

            // Create dependent object for result
            let mut change_obj = serde_json::Map::new();
            change_obj.insert(
                "$ref".to_string(),
                Value::String(path_utils::pointer_to_dot_notation(&data_path)),
            );
            change_obj.insert("$hidden".to_string(), Value::Bool(true));
            change_obj.insert("clear".to_string(), Value::Bool(true));
            result.push(Value::Object(change_obj));

            // Add to queue for standard dependent processing
            queue.push((hf.clone(), true, None));

            // Check reffed_by to find other fields that might become hidden
            if let Some(referencing_fields) = reffed_by.get(&data_path) {
                for rb in referencing_fields {
                    // Evaluate condition.hidden for rb
                    // We need a way to run specific evaluation?
                    // We can check if rb has a hidden evaluation in self.evaluations
                    let hidden_eval_key = format!("{}/condition/hidden", rb);

                    if let Some(logic_id) = evaluations.get(&hidden_eval_key) {
                        // Run evaluation
                        // Context: $value = current field (rb) value? No, $value usually refers to changed field in deps.
                        // But here we are just re-evaluating the rule.
                        // In JS logic: "const result = hiddenFn(runnerCtx);"
                        // runnerCtx has the updated data (we just set hf to null).

                        let rb_data_path = path_utils::normalize_to_json_pointer(rb)
                            .replace("/properties/", "/")
                            .trim_start_matches('#')
                            .to_string();
                        let rb_value = eval_data
                            .data()
                            .pointer(&rb_data_path)
                            .cloned()
                            .unwrap_or(Value::Null);

                        // We can use engine.run w/ eval_data
                        if let Ok(Value::Bool(is_hidden)) = engine.run(logic_id, eval_data.data()) {
                            if is_hidden {
                                // Check if rb is not already in hidden_fields and has value
                                // rb is &String, hidden_fields is Vec<String>
                                if !hidden_fields.contains(rb) {
                                    let has_value = rb_value != Value::Null && rb_value != "";
                                    if has_value {
                                        hidden_fields.push(rb.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Process the dependents queue.
    /// Collects every data path written into `eval_data` into `invalidated_paths`.
    pub(crate) fn process_dependents_queue(
        engine: &RLogic,
        evaluations: &IndexMap<String, LogicId>,
        eval_data: &mut EvalData,
        eval_cache: &mut crate::jsoneval::eval_cache::EvalCache,
        dependents_evaluations: &IndexMap<String, Vec<DependentItem>>,
        dep_formula_triggers: &IndexMap<String, Vec<(String, usize)>>,
        evaluated_schema: &Value,
        queue: &mut Vec<(String, bool, Option<Vec<usize>>)>,
        processed: &mut std::collections::HashMap<String, Option<std::collections::HashSet<usize>>>,
        result: &mut Vec<Value>,
        token: Option<&CancellationToken>,
        canceled_paths: Option<&mut Vec<String>>,
    ) -> Result<(), String> {
        while let Some((current_path, is_transitive, target_indices)) = queue.pop() {
            if let Some(t) = token {
                if t.is_cancelled() {
                    if let Some(cp) = canceled_paths {
                        cp.push(current_path.clone());
                        for (path, _, _) in queue.iter() {
                            cp.push(path.clone());
                        }
                    }
                    return Err("Cancelled".to_string());
                }
            }

            let (should_run, indices_to_run) = match processed.get(&current_path) {
                Some(None) => {
                    // Already fully processed, skip
                    continue;
                }
                Some(Some(already_processed_indices)) => {
                    if let Some(targets) = &target_indices {
                        let new_targets: std::collections::HashSet<usize> = targets
                            .iter()
                            .copied()
                            .filter(|i| !already_processed_indices.contains(i))
                            .collect();
                        if new_targets.is_empty() {
                            continue;
                        }
                        (true, Some(new_targets))
                    } else {
                        (true, None)
                    }
                }
                None => (
                    true,
                    target_indices.clone().map(|t| t.into_iter().collect()),
                ),
            };

            if !should_run {
                continue;
            }

            let new_processed_state = if let Some(targets_to_run) = &indices_to_run {
                match processed.get(&current_path) {
                    Some(Some(existing_targets)) => {
                        let mut copy = existing_targets.clone();
                        for t in targets_to_run {
                            copy.insert(*t);
                        }
                        Some(copy)
                    }
                    _ => Some(targets_to_run.clone()),
                }
            } else {
                None
            };
            processed.insert(current_path.clone(), new_processed_state);

            // Get the value of the changed field for $value context
            let current_data_path = path_utils::normalize_to_json_pointer(&current_path)
                .replace("/properties/", "/")
                .trim_start_matches('#')
                .to_string();
            let mut current_value = eval_data
                .data()
                .pointer(&current_data_path)
                .cloned()
                .unwrap_or(Value::Null);

            // Re-enqueue source fields whose dependent formulas reference this changed field.
            // These are fields that have a dependent formula that checks `current_path` as a
            // contextual condition (e.g., ins_occ's formula for ph_occupation checks phins_relation).
            // When `current_path` changes, we need to re-evaluate those source fields' dependents.
            if let Some(formula_sources) = dep_formula_triggers.get(&current_data_path) {
                let mut targets_by_source: std::collections::HashMap<String, Vec<usize>> =
                    std::collections::HashMap::new();
                for (source_schema_path, dep_idx) in formula_sources {
                    let source_ptr = path_utils::dot_notation_to_schema_pointer(source_schema_path);
                    targets_by_source
                        .entry(source_ptr)
                        .or_default()
                        .push(*dep_idx);
                }
                for (source_ptr, targets) in targets_by_source {
                    // Check if it's already entirely processed
                    if let Some(None) = processed.get(&source_ptr) {
                        continue;
                    }
                    queue.push((source_ptr, true, Some(targets)));
                }
            }

            // Find dependents for this path
            if let Some(dependent_items) = dependents_evaluations.get(&current_path) {
                for (dep_idx, dep_item) in dependent_items.iter().enumerate() {
                    if let Some(targets) = &indices_to_run {
                        if !targets.contains(&dep_idx) {
                            continue;
                        }
                    }
                    let ref_path = &dep_item.ref_path;
                    let pointer_path = path_utils::normalize_to_json_pointer(ref_path);
                    // Data paths don't include /properties/, strip it for data access
                    let data_path = pointer_path.replace("/properties/", "/");

                    let current_ref_value = eval_data
                        .data()
                        .pointer(&data_path)
                        .cloned()
                        .unwrap_or(Value::Null);

                    // Get field and parent field from schema
                    let field = evaluated_schema.pointer(&pointer_path).cloned();

                    // Get parent field - skip /properties/ to get actual parent object
                    let parent_path = if let Some(last_slash) = pointer_path.rfind("/properties") {
                        &pointer_path[..last_slash]
                    } else {
                        "/"
                    };
                    let mut parent_field = if parent_path.is_empty() || parent_path == "/" {
                        evaluated_schema.clone()
                    } else {
                        evaluated_schema
                            .pointer(parent_path)
                            .cloned()
                            .unwrap_or_else(|| Value::Object(serde_json::Map::new()))
                    };

                    // omit properties to minimize size of parent field
                    if let Value::Object(ref mut map) = parent_field {
                        map.remove("properties");
                        map.remove("$layout");
                    }

                    let mut change_obj = serde_json::Map::new();
                    change_obj.insert(
                        "$ref".to_string(),
                        Value::String(path_utils::pointer_to_dot_notation(&data_path)),
                    );
                    if let Some(f) = field {
                        change_obj.insert("$field".to_string(), f);
                    }
                    change_obj.insert("$parentField".to_string(), parent_field);
                    change_obj.insert("transitive".to_string(), Value::Bool(is_transitive));

                    // Skip writing back to a field that has already been processed.
                    // This prevents formula-triggered re-enqueues from creating circular writes:
                    // e.g., ins_gender → triggers phins_relation (via dep_formula_triggers) →
                    // phins_relation has a dep that writes back to ins_gender → we must not let that happen.
                    if processed.contains_key(ref_path) {
                        continue;
                    }

                    let mut add_transitive = false;
                    let mut add_deps = false;
                    // Process clear
                    if let Some(clear_val) = &dep_item.clear {
                        let should_clear = Self::evaluate_dependent_value_static(
                            engine,
                            evaluations,
                            eval_data,
                            clear_val,
                            &current_value,
                            &current_ref_value,
                        )?;
                        let clear_bool = match should_clear {
                            Value::Bool(b) => b,
                            _ => false,
                        };

                        if clear_bool {
                            if data_path == current_data_path {
                                current_value = Value::Null;
                            }
                            eval_data.set(&data_path, Value::Null);
                            eval_cache.bump_data_version(&data_path);
                            change_obj.insert("clear".to_string(), Value::Bool(true));
                            add_transitive = true;
                            add_deps = true;
                        }
                    }

                    // Process value
                    if let Some(value_val) = &dep_item.value {
                        let computed_value = Self::evaluate_dependent_value_static(
                            engine,
                            evaluations,
                            eval_data,
                            value_val,
                            &current_value,
                            &current_ref_value,
                        )?;
                        let cleaned_val = clean_float_noise_scalar(computed_value);

                        let is_clear =
                            cleaned_val == Value::Null || cleaned_val.as_str() == Some("");

                        if cleaned_val != current_ref_value && !is_clear {
                            if data_path == current_data_path {
                                current_value = cleaned_val.clone();
                            }
                            eval_data.set(&data_path, cleaned_val.clone());
                            eval_cache.bump_data_version(&data_path);
                            change_obj.insert("value".to_string(), cleaned_val);
                            add_transitive = true;
                            add_deps = true;
                        }
                    }

                    // add only when has clear / value
                    if add_deps {
                        result.push(Value::Object(change_obj));
                    }

                    // Add this dependent to queue for transitive processing
                    if add_transitive {
                        queue.push((ref_path.clone(), true, None));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Extract the field key from a subform path.
///
/// Examples:
/// - `#/riders`                               → `riders`
/// - `#/properties/form/properties/riders`    → `riders`
/// - `#/items`                                → `items`
fn subform_field_key(subform_path: &str) -> String {
    // Strip leading `#/`
    let stripped = subform_path.trim_start_matches('#').trim_start_matches('/');

    // The last non-"properties" segment is the field key
    stripped
        .split('/')
        .filter(|seg| !seg.is_empty() && *seg != "properties")
        .last()
        .unwrap_or(stripped)
        .to_string()
}
