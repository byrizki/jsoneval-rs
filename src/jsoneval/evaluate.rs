use super::JSONEval;
use crate::json_parser;
use crate::path_utils;
use crate::table_evaluate;
use crate::utils::clean_float_noise;
use crate::time_block;

use serde_json::Value;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

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
    ) -> Result<(), String> {
        time_block!("evaluate() [total]", {
            let context_provided = context.is_some();

            // Use SIMD-accelerated JSON parsing
            let data: Value = time_block!("  parse data", { json_parser::parse_json_str(data)? });
            let context: Value = time_block!("  parse context", {
                json_parser::parse_json_str(context.unwrap_or("{}"))?
            });

            self.data = data.clone();

            // Collect top-level data keys to selectively purge cache
            let changed_data_paths: Vec<String> = if let Some(obj) = data.as_object() {
                obj.keys().map(|k| format!("/{}", k)).collect()
            } else {
                Vec::new()
            };

            // Replace data and context in existing eval_data
            time_block!("  replace_data_and_context", {
                self.eval_data.replace_data_and_context(data, context);
            });

            // Selectively purge cache entries that depend on changed top-level data keys
            // This is more efficient than clearing entire cache
            time_block!("  purge_cache", {
                self.purge_cache_for_changed_data(&changed_data_paths);

                // Also purge context-dependent cache if context was provided
                if context_provided {
                    self.purge_cache_for_context_change();
                }
            });

            // Call internal evaluate (uses existing data if not provided)
            self.evaluate_internal(paths)
        })
    }

    /// Internal evaluate that can be called when data is already set
    /// This avoids double-locking and unnecessary data cloning for re-evaluation from evaluate_dependents
    pub(crate) fn evaluate_internal(&mut self, paths: Option<&[String]>) -> Result<(), String> {
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
                            // Case 1: JSON Schema path (e.g. #/properties/foo) - keep as is
                            p.to_string()
                        } else if p.starts_with('/') {
                            // Case 2: Rust Pointer path (e.g. /properties/foo) - ensure # prefix
                            format!("#{}", p)
                        } else {
                            // Case 3: Dot notation (e.g. properties.foo) - replace dots with slashes and add prefix
                            format!("#/{}", p.replace('.', "/"))
                        };

                        vec![normalized]
                    })
                    .collect::<Vec<_>>();
                Some(normalized_paths_storage.as_slice())
            } else {
                None
            };

            // Clone sorted_evaluations (Arc clone is cheap, then clone inner Vec)
            let eval_batches: Vec<Vec<String>> = (*self.sorted_evaluations).clone();

            // Process each batch - parallelize evaluations within each batch
            // Batches are processed sequentially to maintain dependency order
            // Process value evaluations (simple computed fields)
            // These are independent of rule batches and should always run
            let eval_data_values = self.eval_data.clone();
            time_block!("      evaluate values", {
                #[cfg(feature = "parallel")]
                if self.value_evaluations.len() > 100 {
                    let value_results: Mutex<Vec<(String, Value)>> =
                        Mutex::new(Vec::with_capacity(self.value_evaluations.len()));

                    self.value_evaluations.par_iter().for_each(|eval_key| {
                        // Skip if has dependencies (will be handled in sorted batches)
                        if let Some(deps) = self.dependencies.get(eval_key) {
                            if !deps.is_empty() {
                                return;
                            }
                        }

                        // Filter items if paths are provided
                        if let Some(filter_paths) = normalized_paths {
                            if !filter_paths.is_empty()
                                && !filter_paths.iter().any(|p| {
                                    eval_key.starts_with(p.as_str())
                                        || p.starts_with(eval_key.as_str())
                                })
                            {
                                return;
                            }
                        }

                        // For value evaluations (e.g. /properties/foo/value), we want the value at that path
                        // The path in eval_key is like "#/properties/foo/value"
                        let pointer_path = path_utils::normalize_to_json_pointer(eval_key);

                        // Try cache first (thread-safe)
                        if let Some(_) = self.try_get_cached(eval_key, &eval_data_values) {
                            return;
                        }

                        // Cache miss - evaluate
                        if let Some(logic_id) = self.evaluations.get(eval_key) {
                            if let Ok(val) = self.engine.run(logic_id, eval_data_values.data()) {
                                let cleaned_val = clean_float_noise(val);
                                // Cache result (thread-safe)
                                self.cache_result(eval_key, Value::Null, &eval_data_values);
                                value_results
                                    .lock()
                                    .unwrap()
                                    .push((pointer_path, cleaned_val));
                            }
                        }
                    });

                    // Write results to evaluated_schema
                    for (result_path, value) in value_results.into_inner().unwrap() {
                        if let Some(pointer_value) = self.evaluated_schema.pointer_mut(&result_path)
                        {
                            *pointer_value = value;
                        }
                    }
                }

                // Sequential execution for values (if not parallel or small count)
                #[cfg(feature = "parallel")]
                let value_eval_items = if self.value_evaluations.len() > 100 {
                    &self.value_evaluations[0..0]
                } else {
                    &self.value_evaluations
                };

                #[cfg(not(feature = "parallel"))]
                let value_eval_items = &self.value_evaluations;

                for eval_key in value_eval_items.iter() {
                    // Skip if has dependencies (will be handled in sorted batches)
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

                    let pointer_path = path_utils::normalize_to_json_pointer(eval_key);

                    // Try cache first
                    if let Some(_) = self.try_get_cached(eval_key, &eval_data_values) {
                        continue;
                    }

                    // Cache miss - evaluate
                    if let Some(logic_id) = self.evaluations.get(eval_key) {
                        if let Ok(val) = self.engine.run(logic_id, eval_data_values.data()) {
                            let cleaned_val = clean_float_noise(val);
                            // Cache result
                            self.cache_result(eval_key, Value::Null, &eval_data_values);

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
                for batch in eval_batches {
                    // Skip empty batches
                    if batch.is_empty() {
                        continue;
                    }

                    // Check if we can skip this entire batch optimization
                    // If paths are provided, we can check if ANY item in batch matches ANY path
                    if let Some(filter_paths) = normalized_paths {
                        if !filter_paths.is_empty() {
                            let batch_has_match = batch.iter().any(|eval_key| {
                                filter_paths.iter().any(|p| {
                                    eval_key.starts_with(p.as_str())
                                        || p.starts_with(eval_key.as_str())
                                })
                            });
                            if !batch_has_match {
                                continue;
                            }
                        }
                    }

                    // No pre-checking cache - we'll check inside parallel execution
                    // This allows thread-safe cache access during parallel evaluation

                    // Parallel execution within batch (no dependencies between items)
                    // Use Mutex for thread-safe result collection
                    // Store both eval_key and result for cache storage
                    let eval_data_snapshot = self.eval_data.clone();

                    // Parallelize only if batch has multiple items (overhead not worth it for single item)

                    #[cfg(feature = "parallel")]
                    if batch.len() > 1000 {
                        let results: Mutex<Vec<(String, String, Value)>> =
                            Mutex::new(Vec::with_capacity(batch.len()));
                        batch.par_iter().for_each(|eval_key| {
                            // Filter individual items if paths are provided
                            if let Some(filter_paths) = normalized_paths {
                                if !filter_paths.is_empty()
                                    && !filter_paths.iter().any(|p| {
                                        eval_key.starts_with(p.as_str())
                                            || p.starts_with(eval_key.as_str())
                                    })
                                {
                                    return;
                                }
                            }

                            let pointer_path = path_utils::normalize_to_json_pointer(eval_key);

                            // Try cache first (thread-safe)
                            if let Some(_) = self.try_get_cached(eval_key, &eval_data_snapshot) {
                                return;
                            }

                            // Cache miss - evaluate
                            let is_table = self.table_metadata.contains_key(eval_key);

                            if is_table {
                                // Evaluate table using sandboxed metadata (parallel-safe, immutable parent scope)
                                if let Ok(rows) = table_evaluate::evaluate_table(
                                    self,
                                    eval_key,
                                    &eval_data_snapshot,
                                ) {
                                    let value = Value::Array(rows);
                                    // Cache result (thread-safe)
                                    self.cache_result(eval_key, Value::Null, &eval_data_snapshot);
                                    results.lock().unwrap().push((
                                        eval_key.clone(),
                                        pointer_path,
                                        value,
                                    ));
                                }
                            } else {
                                if let Some(logic_id) = self.evaluations.get(eval_key) {
                                    // Evaluate directly with snapshot
                                    if let Ok(val) =
                                        self.engine.run(logic_id, eval_data_snapshot.data())
                                    {
                                        let cleaned_val = clean_float_noise(val);
                                        // Cache result (thread-safe)
                                        self.cache_result(
                                            eval_key,
                                            Value::Null,
                                            &eval_data_snapshot,
                                        );
                                        results.lock().unwrap().push((
                                            eval_key.clone(),
                                            pointer_path,
                                            cleaned_val,
                                        ));
                                    }
                                }
                            }
                        });

                        // Write all results back sequentially (already cached in parallel execution)
                        for (_eval_key, path, value) in results.into_inner().unwrap() {
                            let cleaned_value = clean_float_noise(value);

                            self.eval_data.set(&path, cleaned_value.clone());
                            // Also write to evaluated_schema
                            if let Some(schema_value) = self.evaluated_schema.pointer_mut(&path) {
                                *schema_value = cleaned_value;
                            }
                        }
                        continue;
                    }

                    // Sequential execution (single item or parallel feature disabled)
                    #[cfg(not(feature = "parallel"))]
                    let batch_items = &batch;

                    #[cfg(feature = "parallel")]
                    let batch_items = if batch.len() > 1000 {
                        &batch[0..0]
                    } else {
                        &batch
                    }; // Empty slice if already processed in parallel

                    for eval_key in batch_items {
                        // Filter individual items if paths are provided
                        if let Some(filter_paths) = normalized_paths {
                            if !filter_paths.is_empty()
                                && !filter_paths.iter().any(|p| {
                                    eval_key.starts_with(p.as_str())
                                        || p.starts_with(eval_key.as_str())
                                })
                            {
                                continue;
                            }
                        }

                        let pointer_path = path_utils::normalize_to_json_pointer(eval_key);

                        // Try cache first
                        if let Some(_) = self.try_get_cached(eval_key, &eval_data_snapshot) {
                            continue;
                        }

                        // Cache miss - evaluate
                        let is_table = self.table_metadata.contains_key(eval_key);

                        if is_table {
                            if let Ok(rows) =
                                table_evaluate::evaluate_table(self, eval_key, &eval_data_snapshot)
                            {
                                let value = Value::Array(rows);
                                // Cache result
                                self.cache_result(eval_key, Value::Null, &eval_data_snapshot);

                                let cleaned_value = clean_float_noise(value);
                                self.eval_data.set(&pointer_path, cleaned_value.clone());
                                if let Some(schema_value) =
                                    self.evaluated_schema.pointer_mut(&pointer_path)
                                {
                                    *schema_value = cleaned_value;
                                }
                            }
                        } else {
                            if let Some(logic_id) = self.evaluations.get(eval_key) {
                                if let Ok(val) =
                                    self.engine.run(logic_id, eval_data_snapshot.data())
                                {
                                    let cleaned_val = clean_float_noise(val);
                                    // Cache result
                                    self.cache_result(eval_key, Value::Null, &eval_data_snapshot);

                                    self.eval_data.set(&pointer_path, cleaned_val.clone());
                                    if let Some(schema_value) =
                                        self.evaluated_schema.pointer_mut(&pointer_path)
                                    {
                                        *schema_value = cleaned_val;
                                    }
                                }
                            }
                        }
                    }
                }
            });

            // Drop lock before calling evaluate_others
            drop(_lock);

            self.evaluate_others(paths);

            Ok(())
        })
    }

    pub(crate) fn evaluate_others(&mut self, paths: Option<&[String]>) {
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

                    #[cfg(feature = "parallel")]
                    {
                        let combined_results: Mutex<Vec<(String, Value)>> =
                            Mutex::new(Vec::with_capacity(combined_count));

                        self.rules_evaluations
                            .par_iter()
                            .chain(self.others_evaluations.par_iter())
                            .for_each(|eval_key| {
                                // Filter items if paths are provided
                                if let Some(filter_paths) = normalized_paths.as_ref() {
                                    if !filter_paths.is_empty()
                                        && !filter_paths.iter().any(|p| {
                                            eval_key.starts_with(p.as_str())
                                                || p.starts_with(eval_key.as_str())
                                        })
                                    {
                                        return;
                                    }
                                }

                                let pointer_path = path_utils::normalize_to_json_pointer(eval_key);

                                // Try cache first (thread-safe)
                                if let Some(_) = self.try_get_cached(eval_key, &eval_data_snapshot)
                                {
                                    return;
                                }

                                // Cache miss - evaluate
                                if let Some(logic_id) = self.evaluations.get(eval_key) {
                                    if let Ok(val) =
                                        self.engine.run(logic_id, eval_data_snapshot.data())
                                    {
                                        let cleaned_val = clean_float_noise(val);
                                        // Cache result (thread-safe)
                                        self.cache_result(
                                            eval_key,
                                            Value::Null,
                                            &eval_data_snapshot,
                                        );
                                        combined_results
                                            .lock()
                                            .unwrap()
                                            .push((pointer_path, cleaned_val));
                                    }
                                }
                            });

                        // Write results to evaluated_schema
                        for (result_path, value) in combined_results.into_inner().unwrap() {
                            if let Some(pointer_value) =
                                self.evaluated_schema.pointer_mut(&result_path)
                            {
                                // Special handling for rules with $evaluation
                                // This includes both direct rules and array items: /rules/evaluation/0/$evaluation
                                if !result_path.starts_with("$")
                                    && result_path.contains("/rules/")
                                    && !result_path.ends_with("/value")
                                {
                                    match pointer_value.as_object_mut() {
                                        Some(pointer_obj) => {
                                            pointer_obj.remove("$evaluation");
                                            pointer_obj.insert("value".to_string(), value);
                                        }
                                        None => continue,
                                    }
                                } else {
                                    *pointer_value = value;
                                }
                            }
                        }
                    }

                    #[cfg(not(feature = "parallel"))]
                    {
                        // Sequential evaluation
                        let combined_evals: Vec<&String> = self
                            .rules_evaluations
                            .iter()
                            .chain(self.others_evaluations.iter())
                            .collect();

                        for eval_key in combined_evals {
                            // Filter items if paths are provided
                            if let Some(filter_paths) = normalized_paths.as_ref() {
                                if !filter_paths.is_empty()
                                    && !filter_paths.iter().any(|p| {
                                        eval_key.starts_with(p.as_str())
                                            || p.starts_with(eval_key.as_str())
                                    })
                                {
                                    continue;
                                }
                            }

                            let pointer_path = path_utils::normalize_to_json_pointer(eval_key);

                            // Try cache first
                            if let Some(_) = self.try_get_cached(eval_key, &eval_data_snapshot) {
                                continue;
                            }

                            // Cache miss - evaluate
                            if let Some(logic_id) = self.evaluations.get(eval_key) {
                                if let Ok(val) =
                                    self.engine.run(logic_id, eval_data_snapshot.data())
                                {
                                    let cleaned_val = clean_float_noise(val);
                                    // Cache result
                                    self.cache_result(eval_key, Value::Null, &eval_data_snapshot);

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
                    }
                });
            }
        });

        // Step 2: Evaluate options URL templates (handles {variable} patterns)
        time_block!("      evaluate_options_templates", {
            self.evaluate_options_templates(paths);
        });

        // Step 3: Resolve layout logic (metadata injection, hidden propagation)
        time_block!("      resolve_layout", {
            self.resolve_layout(false);
        });
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
