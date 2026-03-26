use super::JSONEval;
use crate::jsoneval::json_parser;
use crate::jsoneval::path_utils;
use crate::jsoneval::table_evaluate;
use crate::jsoneval::cancellation::CancellationToken;
use crate::utils::clean_float_noise_scalar;
use crate::time_block;

use serde_json::Value;



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

    /// Internal helper to evaluate with all data/context provided as Values
    fn evaluate_internal_with_new_data(
        &mut self,
        data: Value,
        context: Value,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<(), String> {
        time_block!("  evaluate_internal_with_new_data", {
            // Store data, context and replace in eval_data (clone once instead of twice)
            self.data = data.clone();
            self.context = context.clone();
            time_block!("  replace_data_and_context", {
                self.eval_data.replace_data_and_context(data, context);
            });

            // Call internal evaluate (uses existing data if not provided)
            self.evaluate_internal(paths, token)
        })
    }

    /// Internal evaluate that can be called when data is already set
    /// This avoids double-locking and unnecessary data cloning for re-evaluation from evaluate_dependents
    pub(crate) fn evaluate_internal(&mut self, paths: Option<&[String]>, token: Option<&CancellationToken>) -> Result<(), String> {
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

            // Borrow sorted_evaluations via Arc (avoid deep-cloning Vec<Vec<String>>)
            let eval_batches = self.sorted_evaluations.clone();
            
            // Track cache misses across batches to prevent false hits from large skipped arrays
            // Use persisted missed_keys from JSONEval

            // Process each batch - sequentially
            // Batches are processed sequentially to maintain dependency order
            // Process value evaluations (simple computed fields)
            // These are independent of rule batches and should always run
            let eval_data_values = self.eval_data.clone();
            time_block!("      evaluate values", {
                for eval_key in self.value_evaluations.iter() {
                    if let Some(t) = token {
                        if t.is_cancelled() {
                            return Err("Cancelled".to_string());
                        }
                    }
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

                    let pointer_path = path_utils::normalize_to_json_pointer(eval_key).into_owned();

                    // Cache miss - evaluate
                    if let Some(logic_id) = self.evaluations.get(eval_key) {
                        if let Ok(val) = self.engine.run(logic_id, eval_data_values.data()) {
                             let cleaned_val = clean_float_noise_scalar(val);

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
                    if let Some(filter_paths) = normalized_paths {
                        if !filter_paths.is_empty() {
                            let batch_has_match = batch.iter().any(|eval_key| {
                                filter_paths.iter().any(|p| {
                                    eval_key.starts_with(p.as_str())
                                        || (p.starts_with(eval_key.as_str())
                                            && !eval_key.contains("/$params/"))
                                })
                            });
                            if !batch_has_match {
                                continue;
                            }
                        }
                    }

                    // Sequential execution
                    let eval_data_snapshot = self.eval_data.clone();

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

                        let pointer_path = path_utils::normalize_to_json_pointer(eval_key).into_owned();

                        // Cache miss - evaluate
                        let is_table = self.table_metadata.contains_key(eval_key);

                        if is_table {
                            if let Ok(rows) =
                                table_evaluate::evaluate_table(self, eval_key, &eval_data_snapshot, token)
                            {
                                let value = Value::Array(rows);
                                self.eval_data.set(&pointer_path, value.clone());
                                if let Some(schema_value) =
                                    self.evaluated_schema.pointer_mut(&pointer_path)
                                {
                                    *schema_value = value;
                                }
                            }
                        } else {
                            if let Some(logic_id) = self.evaluations.get(eval_key) {
                                if let Ok(val) =
                                    self.engine.run(logic_id, eval_data_snapshot.data())
                                {
                                    let cleaned_val = clean_float_noise_scalar(val);
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

            self.evaluate_others(paths, token);

            Ok(())
        })
    }

    pub(crate) fn evaluate_others(&mut self, paths: Option<&[String]>, token: Option<&CancellationToken>) {
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

                        let pointer_path = path_utils::normalize_to_json_pointer(eval_key).into_owned();

                        // Cache miss - evaluate
                        if let Some(logic_id) = self.evaluations.get(eval_key) {
                            if let Ok(val) =
                                self.engine.run(logic_id, eval_data_snapshot.data())
                            {
                                let cleaned_val = clean_float_noise_scalar(val);

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
        time_block!("      evaluate_options_templates", {
            self.evaluate_options_templates(paths);
        });

        // Step 3: Resolve layout logic (metadata injection, hidden propagation)
        time_block!("      resolve_layout", {
            let _ = self.resolve_layout(false);
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
