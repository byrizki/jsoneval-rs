use super::JSONEval;
use crate::jsoneval::json_parser;
use crate::jsoneval::path_utils;
use crate::rlogic::{LogicId, RLogic};
use crate::jsoneval::types::DependentItem;
use crate::jsoneval::cancellation::CancellationToken;
use crate::utils::clean_float_noise_scalar;
use crate::EvalData;

use indexmap::{IndexMap, IndexSet};
use serde_json::Value;


impl JSONEval {
    /// Evaluate fields that depend on a changed path
    /// This processes all dependent fields transitively when a source field changes
    pub fn evaluate_dependents(
        &mut self,
        changed_paths: &[String],
        data: Option<&str>,
        context: Option<&str>,
        re_evaluate: bool,
        token: Option<&CancellationToken>,
        mut canceled_paths: Option<&mut Vec<String>>,
    ) -> Result<Value, String> {
        // Check cancellation
        if let Some(t) = token {
            if t.is_cancelled() {
                return Err("Cancelled".to_string());
            }
        }
        // Acquire lock for synchronous execution
        let _lock = self.eval_lock.lock().unwrap();

        // Update data if provided
        if let Some(data_str) = data {
            // Save old data for comparison
            let old_data = self.eval_data.snapshot_data();

            let data_value = json_parser::parse_json_str(data_str)?;
            let context_value = if let Some(ctx) = context {
                json_parser::parse_json_str(ctx)?
            } else {
                Value::Object(serde_json::Map::new())
            };
            self.eval_data
                .replace_data_and_context(data_value.clone(), context_value);

            // Selectively purge cache entries that depend on changed data
            // Only purge if values actually changed
            // Convert changed_paths to data pointer format for cache purging
            let data_paths: Vec<String> = changed_paths
                .iter()
                .map(|path| {
                    // Robust normalization: normalize to schema pointer first, then strip schema-specific parts
                    // This handles both "illustration.insured.name" and "#/illustration/properties/insured/properties/name"
                    let schema_ptr = path_utils::dot_notation_to_schema_pointer(path);

                    // Remove # prefix and /properties/ segments to get pure data location
                    let normalized = schema_ptr
                        .trim_start_matches('#')
                        .replace("/properties/", "/");

                    // Ensure it starts with / for data pointer
                    if normalized.starts_with('/') {
                        normalized
                    } else {
                        format!("/{}", normalized)
                    }
                })
                .collect();
            self.purge_cache_for_changed_data_with_comparison(&data_paths, &old_data, &data_value);
        }

        let mut result = Vec::new();
        let mut processed = IndexSet::new();

        // Normalize all changed paths and add to processing queue
        // Converts: "illustration.insured.name" -> "#/illustration/properties/insured/properties/name"
        let mut to_process: Vec<(String, bool)> = changed_paths
            .iter()
            .map(|path| (path_utils::dot_notation_to_schema_pointer(path), false))
            .collect(); // (path, is_transitive)

        // Process dependents recursively (always nested/transitive)
        Self::process_dependents_queue(
            &self.engine,
            &self.evaluations,
            &mut self.eval_data,
            &self.dependents_evaluations,
            &self.evaluated_schema,
            &mut to_process,
            &mut processed,
            &mut result,
            token,
            canceled_paths.as_mut().map(|v| &mut **v)
        )?;

        // If re_evaluate is true, perform full evaluation with the mutated eval_data
        // Then perform post-evaluation checks (ReadOnly, Hidden)
        if re_evaluate {
            // Drop lock for evaluate_internal
            drop(_lock); 

            // Clear the entire eval cache before re-evaluation.
            // The dependents graph processing above may have modified eval_data for
            // many fields beyond the user's changed_paths, but those changes were not
            // tracked for cache purging. Clearing ensures evaluate_internal doesn't
            // return stale cached results for fields whose dependencies were updated.
            self.eval_cache.clear();

            self.evaluate_internal(None, token)?;
            
            // Re-acquire lock for ReadOnly/Hidden processing
            let _lock = self.eval_lock.lock().unwrap();

            // 1. Read-Only Pass
            // Collect read-only fields - include ALL readonly values in the result
            let mut readonly_changes = Vec::new();
            let mut readonly_values = Vec::new();  // Track all readonly values (including unchanged)
            
            // OPTIMIZATION: Use conditional_readonly_fields cache instead of recursing whole schema
            // self.collect_readonly_fixes(&self.evaluated_schema, "#", &mut readonly_changes);
            for path in self.conditional_readonly_fields.iter() {
                let normalized = path_utils::normalize_to_json_pointer(path);
                if let Some(schema_element) = self.evaluated_schema.pointer(&normalized) {
                    self.check_readonly_for_dependents(schema_element, path, &mut readonly_changes, &mut readonly_values);
                }
            }

            // Apply fixes for changed values and add to queue
            for (path, schema_value) in readonly_changes {
                // Set data to match schema value
                let data_path = path_utils::normalize_to_json_pointer(&path)
                    .replace("/properties/", "/")
                    .trim_start_matches('#')
                    .to_string();
                
                self.eval_data.set(&data_path, schema_value.clone());
                
                // Add to process queue for changed values
                to_process.push((path, true));
            }
            
            // Add ALL readonly values to result (both changed and unchanged)
            for (path, schema_value) in readonly_values {
                let data_path = path_utils::normalize_to_json_pointer(&path)
                    .replace("/properties/", "/")
                    .trim_start_matches('#')
                    .to_string();
                
                let mut change_obj = serde_json::Map::new();
                change_obj.insert("$ref".to_string(), Value::String(path_utils::pointer_to_dot_notation(&data_path)));
                change_obj.insert("$readonly".to_string(), Value::Bool(true));
                change_obj.insert("value".to_string(), schema_value);
                
                result.push(Value::Object(change_obj));
            }
            
            // Refund process queue for ReadOnly effects
            if !to_process.is_empty() {
                 Self::process_dependents_queue(
                    &self.engine,
                    &self.evaluations,
                    &mut self.eval_data,
                    &self.dependents_evaluations,
                    &self.evaluated_schema,
                    &mut to_process,
                    &mut processed,
                    &mut result,
                    token,
                    canceled_paths.as_mut().map(|v| &mut **v)
                )?;
            }

            // 2. Recursive Hide Pass
            // Collect hidden fields that have values
            let mut hidden_fields = Vec::new();
            // OPTIMIZATION: Use conditional_hidden_fields cache instead of recursing whole schema
            // self.collect_hidden_fields(&self.evaluated_schema, "#", &mut hidden_fields);
            for path in self.conditional_hidden_fields.iter() {
                let normalized = path_utils::normalize_to_json_pointer(path);
                 if let Some(schema_element) = self.evaluated_schema.pointer(&normalized) {
                    self.check_hidden_field(schema_element, path, &mut hidden_fields);
                }
            }
            
            // Logic for recursive hiding (using reffed_by)
            if !hidden_fields.is_empty() {
                Self::recursive_hide_effect(
                    &self.engine,
                    &self.evaluations,
                    &self.reffed_by,
                    &mut self.eval_data,
                    hidden_fields, 
                    &mut to_process, 
                    &mut result
                );
            }
            
            // Process queue for Hidden effects
             if !to_process.is_empty() {
                 Self::process_dependents_queue(
                    &self.engine,
                    &self.evaluations,
                    &mut self.eval_data,
                    &self.dependents_evaluations,
                    &self.evaluated_schema,
                    &mut to_process,
                    &mut processed,
                    &mut result,
                    token,
                    canceled_paths.as_mut().map(|v| &mut **v)
                )?;
            }
        }

        Ok(Value::Array(result))
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

    /// Perform recursive hiding effect using reffed_by graph
    pub(crate) fn recursive_hide_effect(
        engine: &RLogic,
        evaluations: &IndexMap<String, LogicId>,
        reffed_by: &IndexMap<String, Vec<String>>,
        eval_data: &mut EvalData,
        mut hidden_fields: Vec<String>,
        queue: &mut Vec<(String, bool)>, 
        result: &mut Vec<Value>
    ) {
        while let Some(hf) = hidden_fields.pop() {
            let data_path = path_utils::normalize_to_json_pointer(&hf)
                .replace("/properties/", "/")
                .trim_start_matches('#')
                .to_string();
            
            // clear data
            eval_data.set(&data_path, Value::Null);
            
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

    /// Process the dependents queue
    /// This handles the transitive propagation of changes based on the dependents graph
    pub(crate) fn process_dependents_queue(
        engine: &RLogic,
        evaluations: &IndexMap<String, LogicId>,
        eval_data: &mut EvalData,
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
                            // Clear the field
                            if data_path == current_data_path {
                                current_value = Value::Null;
                            }
                            eval_data.set(&data_path, Value::Null);
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
                            // Set the value
                            if data_path == current_data_path {
                                current_value = cleaned_val.clone();
                            }
                            eval_data.set(&data_path, cleaned_val.clone());
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
