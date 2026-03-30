use super::JSONEval;
use crate::jsoneval::json_parser;
use crate::jsoneval::path_utils;
use crate::jsoneval::path_utils::get_value_by_pointer_without_properties;
use crate::jsoneval::path_utils::normalize_to_json_pointer;
use crate::rlogic::{LogicId, RLogic};
use crate::jsoneval::types::DependentItem;
use crate::jsoneval::cancellation::CancellationToken;
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

        // Update data if provided, diff versions
        if let Some(data_str) = data {
            let data_value = json_parser::parse_json_str(data_str)?;
            let context_value = if let Some(ctx) = context {
                json_parser::parse_json_str(ctx)?
            } else {
                Value::Object(serde_json::Map::new())
            };
            let old_data = self.eval_data.snapshot_data_clone();
            self.eval_data.replace_data_and_context(data_value, context_value);
            let new_data = self.eval_data.snapshot_data_clone();
            self.eval_cache.store_snapshot_and_diff_versions(&old_data, &new_data);
        }

        let mut result = Vec::new();
        let mut processed = IndexSet::new();
        let mut to_process: Vec<(String, bool)> = changed_paths
            .iter()
            .map(|path| (path_utils::dot_notation_to_schema_pointer(path), false))
            .collect();

        Self::process_dependents_queue(
            &self.engine,
            &self.evaluations,
            &mut self.eval_data,
            &mut self.eval_cache,
            &self.dependents_evaluations,
            &self.evaluated_schema,
            &mut to_process,
            &mut processed,
            &mut result,
            token,
            canceled_paths.as_mut().map(|v| &mut **v),
        )?;

        // Drop the lock before calling sub-methods that may re-acquire it
        drop(_lock);

        if re_evaluate {
            self.run_re_evaluate_pass(token, &mut to_process, &mut processed, &mut result, canceled_paths.as_mut().map(|v| &mut **v))?;
        }

        if include_subforms {
            self.run_subform_pass(changed_paths, re_evaluate, token, &mut result)?;
        }

        Ok(Value::Array(result))
    }

    /// Full re-evaluation pass: runs `evaluate_internal`, then applies read-only fixes and
    /// recursive hide effects, feeding any newly-generated changes back into the dependents queue.
    fn run_re_evaluate_pass(
        &mut self,
        token: Option<&CancellationToken>,
        to_process: &mut Vec<(String, bool)>,
        processed: &mut IndexSet<String>,
        result: &mut Vec<Value>,
        mut canceled_paths: Option<&mut Vec<String>>,
    ) -> Result<(), String> {
        // --- Schema Default Value Pass ---
        let mut default_value_changes = Vec::new();
        let schema_values = self.get_schema_value_array();
        
        if let Value::Array(values) = schema_values {
            for item in values {
                if let Value::Object(map) = item {
                    if let (Some(Value::String(dot_path)), Some(schema_val)) = (map.get("path"), map.get("value")) {
                        let data_path = dot_path.replace('.', "/");
                        let current_data = self.eval_data.data().pointer(&format!("/{}", data_path)).unwrap_or(&Value::Null);
                        
                        let is_empty = match current_data {
                            Value::Null => true,
                            Value::String(s) if s.is_empty() => true,
                            _ => false,
                        };

                        let is_schema_val_empty = match schema_val {
                            Value::Null => true,
                            Value::String(s) if s.is_empty() => true,
                            _ => false,
                        };
                        
                        if is_empty && !is_schema_val_empty && current_data != schema_val {
                             default_value_changes.push((data_path, schema_val.clone(), dot_path.clone()));
                        }
                    }
                }
            }
        }
        
        for (data_path, schema_val, dot_path) in default_value_changes {
             self.eval_data.set(&format!("/{}", data_path), schema_val.clone());
             self.eval_cache.bump_data_version(&format!("/{}", data_path));
             
             let mut change_obj = serde_json::Map::new();
             change_obj.insert("$ref".to_string(), Value::String(dot_path));
             change_obj.insert("value".to_string(), schema_val);
             result.push(Value::Object(change_obj));
             
             let schema_ptr = format!("#/{}", data_path.replace('/', "/properties/"));
             to_process.push((schema_ptr, true));
        }

        if !to_process.is_empty() {
            Self::process_dependents_queue(
                &self.engine,
                &self.evaluations,
                &mut self.eval_data,
                &mut self.eval_cache,
                &self.dependents_evaluations,
                &self.evaluated_schema,
                to_process,
                processed,
                result,
                token,
                canceled_paths.as_mut().map(|v| &mut **v),
            )?;
        }

        // Snapshot lightweight internal version map before re-evaluation.
        // This completely skips the previous O(N) memory-heavy deep cloning of all JSON node data!
        let pre_eval_versions = self.eval_cache.data_versions.clone();

        self.evaluate_internal(None, token)?;

        // Emit result entries for every sorted-evaluation whose version uniquely bumped
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

            let old_ver = pre_eval_versions.get(&format!("/{}", data_path));
            let new_ver = self.eval_cache.data_versions.get(&format!("/{}", data_path));

            if new_ver > old_ver {
                if let Some(new_val) = self.evaluated_schema.pointer(&schema_ptr) {
                    let dot_path = data_path.trim_end_matches("/value").replace('/', ".");
                    let mut obj = serde_json::Map::new();
                    obj.insert("$ref".to_string(), Value::String(dot_path));
                    obj.insert("value".to_string(), new_val.clone());
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
                self.check_readonly_for_dependents(schema_el, path, &mut readonly_changes, &mut readonly_values);
            }
        }
        for (path, schema_value) in readonly_changes {
            let data_path = path_utils::normalize_to_json_pointer(&path)
                .replace("/properties/", "/")
                .trim_start_matches('#')
                .to_string();
            self.eval_data.set(&data_path, schema_value.clone());
            self.eval_cache.bump_data_version(&data_path);
            to_process.push((path, true));
        }
        for (path, schema_value) in readonly_values {
            let data_path = path_utils::normalize_to_json_pointer(&path)
                .replace("/properties/", "/")
                .trim_start_matches('#')
                .to_string();
            let mut obj = serde_json::Map::new();
            obj.insert("$ref".to_string(), Value::String(path_utils::pointer_to_dot_notation(&data_path)));
            obj.insert("$readonly".to_string(), Value::Bool(true));
            obj.insert("value".to_string(), schema_value);
            result.push(Value::Object(obj));
        }
        if !to_process.is_empty() {
            Self::process_dependents_queue(
                &self.engine,
                &self.evaluations,
                &mut self.eval_data,
                &mut self.eval_cache,
                &self.dependents_evaluations,
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
            let subform_dot_path = path_utils::pointer_to_dot_notation(&subform_path)
                .replace(".properties.", ".");
            let field_prefix = format!("{}.", field_key);
            let subform_ptr = normalize_to_json_pointer(&subform_path);

            // Borrow only the item count first — avoid cloning the full array
            let item_count = get_value_by_pointer_without_properties(
                self.eval_data.data(),
                &subform_ptr,
            )
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

            if item_count == 0 {
                continue;
            }

            // When the parent ran a re_evaluate pass, always pass re_evaluate:true to subforms.
            // The parent's evaluate_internal may have updated $params or other referenced values
            // that the subform formulas read, even if none of the subform's own dep paths bumped.
            let global_sub_re_evaluate = re_evaluate;

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
                let item_val = get_value_by_pointer_without_properties(
                    self.eval_data.data(),
                    &subform_ptr,
                )
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
                let old_item_val = self.eval_cache.subform_caches
                    .get(&idx)
                    .map(|c| c.item_snapshot.clone())
                    .unwrap_or(Value::Null);

                subform.eval_data.replace_data_and_context(
                    merged_data,
                    self.eval_data.data().get("$context").cloned().unwrap_or(Value::Null),
                );
                let new_item_val = subform.eval_data.data()
                    .get(&field_key)
                    .cloned()
                    .unwrap_or(Value::Null);

                // Cache-swap: lend parent cache to subform
                let mut parent_cache = std::mem::take(&mut self.eval_cache);
                parent_cache.ensure_active_item_cache(idx);
                if let Some(c) = parent_cache.subform_caches.get_mut(&idx) {
                    c.data_versions.merge_from(&parent_cache.data_versions);
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

                let subform_result = subform.evaluate_dependents(
                    &item_changed_paths,
                    None,
                    None,
                    sub_re_evaluate,
                    token,
                    None,
                    false,
                );

                // Restore parent cache
                std::mem::swap(&mut subform.eval_cache, &mut parent_cache);
                parent_cache.clear_active_item();
                self.eval_cache = parent_cache;

                if let Ok(Value::Array(changes)) = subform_result {
                    for change in changes {
                        if let Some(obj) = change.as_object() {
                            if let Some(Value::String(ref_path)) = obj.get("$ref") {
                                // Remap the $ref path to include the parent path + item index
                                let new_ref = if ref_path.starts_with(&field_prefix) {
                                    format!("{}.{}.{}", subform_dot_path, idx, &ref_path[field_prefix.len()..])
                                } else {
                                    format!("{}.{}.{}", subform_dot_path, idx, ref_path)
                                };
                                let mut new_obj = obj.clone();
                                new_obj.insert("$ref".to_string(), Value::String(new_ref));
                                result.push(Value::Object(new_obj));
                            } else {
                                // No $ref rewrite needed — push as-is without cloning the map
                                result.push(change);
                            }
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
                         
                         let current_data = self.eval_data.data().pointer(&data_path).unwrap_or(&Value::Null);
                         
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
                         
                         let current_data = self.eval_data.data().pointer(&data_path).unwrap_or(&Value::Null);
                         
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

                     let current_data = self.eval_data.data().pointer(&data_path).unwrap_or(&Value::Null);
                     
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

                     let current_data = self.eval_data.data().pointer(&data_path).unwrap_or(&Value::Null);
                     
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
        queue: &mut Vec<(String, bool)>, 
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
            change_obj.insert("$ref".to_string(), Value::String(path_utils::pointer_to_dot_notation(&data_path)));
            change_obj.insert("$hidden".to_string(), Value::Bool(true));
            change_obj.insert("clear".to_string(), Value::Bool(true));
            result.push(Value::Object(change_obj));
            
            // Add to queue for standard dependent processing
            queue.push((hf.clone(), true));

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
                         let rb_value = eval_data.data().pointer(&rb_data_path).cloned().unwrap_or(Value::Null);
                         
                         // We can use engine.run w/ eval_data
                         if let Ok(Value::Bool(is_hidden)) = engine.run(
                             logic_id, 
                             eval_data.data()
                         ) {
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
        evaluated_schema: &Value,
        queue: &mut Vec<(String, bool)>,
        processed: &mut IndexSet<String>,
        result: &mut Vec<Value>,
        token: Option<&CancellationToken>,
        canceled_paths: Option<&mut Vec<String>>,
    ) -> Result<(), String> {
        while let Some((current_path, is_transitive)) = queue.pop() {
            if let Some(t) = token {
                if t.is_cancelled() {
                    // Accumulate canceled paths if buffer provided
                    if let Some(cp) = canceled_paths {
                        cp.push(current_path.clone());
                        // Also push remaining items in queue?
                        // The user request says "accumulate canceled path if provided", usually implies what was actively cancelled 
                        // or what was pending. Since we pop one by one, we can just dump the queue back or just push pending.
                        // But since we just popped `current_path`, it is the one being cancelled on.
                        // Let's also drain the queue.
                        for (path, _) in queue.iter() {
                             cp.push(path.clone());
                        }
                    }
                    return Err("Cancelled".to_string());
                }
            }
            if processed.contains(&current_path) {
                continue;
            }
            processed.insert(current_path.clone());

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

            // Find dependents for this path
            if let Some(dependent_items) = dependents_evaluations.get(&current_path) {
                for dep_item in dependent_items {
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

                        if cleaned_val != current_ref_value && cleaned_val != Value::Null {
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
                        queue.push((ref_path.clone(), true));
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
