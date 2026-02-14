use super::JSONEval;
use crate::jsoneval::json_parser;
use crate::jsoneval::path_utils;
use crate::jsoneval::types::{ValidationError, ValidationResult};
use crate::jsoneval::cancellation::CancellationToken;

use crate::time_block;

use indexmap::IndexMap;
use serde_json::Value;


impl JSONEval {
    /// Validate data against schema rules
    pub fn validate(
        &mut self,
        data: &str,
        context: Option<&str>,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<ValidationResult, String> {
        if let Some(t) = token {
            if t.is_cancelled() {
                return Err("Cancelled".to_string());
            }
        }
        time_block!("validate() [total]", {
            // Acquire lock for synchronous execution
            let _lock = self.eval_lock.lock().unwrap();

            // Save old data for comparison
            let old_data = self.eval_data.clone_data_without(&["$params"]);

             // Parse and update data
            let data_value = json_parser::parse_json_str(data)?;
            let context_value = if let Some(ctx) = context {
                json_parser::parse_json_str(ctx)?
            } else {
                Value::Object(serde_json::Map::new())
            };

            // Update eval_data with new data/context
            self.eval_data.replace_data_and_context(data_value.clone(), context_value);

            // Calculate changed paths for cache purging (root changed)
            // Selectively purge cache entries that depend on root data
            // Convert changed_paths to data pointer format for cache purging
             let changed_data_paths = vec!["/".to_string()];
            
             // Selectively purge cache entries
             self.purge_cache_for_changed_data_with_comparison(&changed_data_paths, &old_data, &data_value);
             
             if context.is_some() {
                 self.purge_cache_for_context_change();
             }
            
            // Drop lock before calling evaluate_others which needs mutable access
            drop(_lock);

            // Re-evaluate rule evaluations to ensure fresh values
            // This ensures all rule.$evaluation expressions are re-computed
            self.evaluate_others(paths, token);

            // Update evaluated_schema with fresh evaluations
            self.evaluated_schema = self.get_evaluated_schema(false);

            let mut errors: IndexMap<String, ValidationError> = IndexMap::new();

            // Use pre-parsed fields_with_rules from schema parsing (no runtime collection needed)
            // This list was collected during schema parse and contains all fields with rules
            for field_path in self.fields_with_rules.iter() {
                // Check if we should validate this path (path filtering)
                if let Some(filter_paths) = paths {
                    if !filter_paths.is_empty()
                        && !filter_paths.iter().any(|p| {
                            field_path.starts_with(p.as_str()) || p.starts_with(field_path.as_str())
                        })
                    {
                        continue;
                    }
                }

                self.validate_field(field_path, &data_value, &mut errors);

                if let Some(t) = token {
                    if t.is_cancelled() {
                        return Err("Cancelled".to_string());
                    }
                }
            }

            let has_error = !errors.is_empty();

            Ok(ValidationResult { has_error, errors })
        })
    }

    /// Validate a single field that has rules
    pub(crate) fn validate_field(
        &self,
        field_path: &str,
        data: &Value,
        errors: &mut IndexMap<String, ValidationError>,
    ) {
        // Skip if already has error
        if errors.contains_key(field_path) {
            return;
        }

        // Resolve schema for this field
        let schema_path = path_utils::dot_notation_to_schema_pointer(field_path);
        let pointer_path = schema_path.trim_start_matches('#');

        // Try to get schema, if not found, try with /properties/ prefix for standard JSON Schema
        let (field_schema, resolved_path) = match self.evaluated_schema.pointer(pointer_path) {
            Some(s) => (s, pointer_path.to_string()),
            None => {
                let alt_path = format!("/properties{}", pointer_path);
                match self.evaluated_schema.pointer(&alt_path) {
                    Some(s) => (s, alt_path),
                    None => return,
                }
            }
        };

        // Skip hidden fields
        if self.is_effective_hidden(&resolved_path) {
            return;
        }

        if let Value::Object(schema_map) = field_schema {

            // Get rules object
            let rules = match schema_map.get("rules") {
                Some(Value::Object(r)) => r,
                _ => return,
            };

            // Get field data
            let field_data = self.get_field_data(field_path, data);

            // Validate each rule
            for (rule_name, rule_value) in rules {
                self.validate_rule(
                    field_path,
                    rule_name,
                    rule_value,
                    &field_data,
                    schema_map,
                    field_schema,
                    errors,
                );
            }
        }
    }

    /// Get data value for a field path
    pub(crate) fn get_field_data(&self, field_path: &str, data: &Value) -> Value {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = data;

        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part).unwrap_or(&Value::Null);
                }
                _ => return Value::Null,
            }
        }

        current.clone()
    }

    /// Validate a single rule
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn validate_rule(
        &self,
        field_path: &str,
        rule_name: &str,
        rule_value: &Value,
        field_data: &Value,
        schema_map: &serde_json::Map<String, Value>,
        _schema: &Value,
        errors: &mut IndexMap<String, ValidationError>,
    ) {
        // Skip if already has error
        if errors.contains_key(field_path) {
            return;
        }

        let mut disabled_field = false;
        // Check if disabled
        if let Some(Value::Object(condition)) = schema_map.get("condition") {
            if let Some(Value::Bool(true)) = condition.get("disabled") {
                disabled_field = true;
            }
        }

        // Get the evaluated rule from evaluated_schema (which has $evaluation already processed)
        // Convert field_path to schema path
        let schema_path = path_utils::dot_notation_to_schema_pointer(field_path);
        let rule_path = format!(
            "{}/rules/{}",
            schema_path.trim_start_matches('#'),
            rule_name
        );

        // Look up the evaluated rule from evaluated_schema
        let evaluated_rule = if let Some(eval_rule) = self.evaluated_schema.pointer(&rule_path) {
            eval_rule.clone()
        } else {
            rule_value.clone()
        };

        // Extract rule active status, message, etc
        // Logic depends on rule structure (object with value/message or direct value)
        
        let (rule_active, rule_message, rule_code, rule_data) = match &evaluated_rule {
            Value::Object(rule_obj) => {
                let active = rule_obj.get("value").unwrap_or(&Value::Bool(false));

                // Handle message - could be string or object with "value"
                let message = match rule_obj.get("message") {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Object(msg_obj)) if msg_obj.contains_key("value") => msg_obj
                        .get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Validation failed")
                        .to_string(),
                    Some(msg_val) => msg_val.as_str().unwrap_or("Validation failed").to_string(),
                    None => "Validation failed".to_string(),
                };

                let code = rule_obj
                    .get("code")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());

                // Handle data - extract "value" from objects with $evaluation
                let data = rule_obj.get("data").map(|d| {
                    if let Value::Object(data_obj) = d {
                        let mut cleaned_data = serde_json::Map::new();
                        for (key, value) in data_obj {
                            // If value is an object with only "value" key, extract it
                            if let Value::Object(val_obj) = value {
                                if val_obj.len() == 1 && val_obj.contains_key("value") {
                                    cleaned_data.insert(key.clone(), val_obj["value"].clone());
                                } else {
                                    cleaned_data.insert(key.clone(), value.clone());
                                }
                            } else {
                                cleaned_data.insert(key.clone(), value.clone());
                            }
                        }
                        Value::Object(cleaned_data)
                    } else {
                        d.clone()
                    }
                });

                (active.clone(), message, code, data)
            }
            _ => (
                evaluated_rule.clone(),
                "Validation failed".to_string(),
                None,
                None,
            ),
        };

        // Generate default code if not provided
        let error_code = rule_code.or_else(|| Some(format!("{}.{}", field_path, rule_name)));

        let is_empty = matches!(field_data, Value::Null)
            || (field_data.is_string() && field_data.as_str().unwrap_or("").is_empty())
            || (field_data.is_array() && field_data.as_array().unwrap().is_empty());

        match rule_name {
            "required" => {
                if !disabled_field && rule_active == Value::Bool(true) {
                    if is_empty {
                        errors.insert(
                            field_path.to_string(),
                            ValidationError {
                                rule_type: "required".to_string(),
                                message: rule_message,
                                code: error_code.clone(),
                                pattern: None,
                                field_value: None,
                                data: None,
                            },
                        );
                    }
                }
            }
            "minLength" => {
                if !is_empty {
                    if let Some(min) = rule_active.as_u64() {
                        let len = match field_data {
                            Value::String(s) => s.len(),
                            Value::Array(a) => a.len(),
                            _ => 0,
                        };
                        if len < min as usize {
                            errors.insert(
                                field_path.to_string(),
                                ValidationError {
                                    rule_type: "minLength".to_string(),
                                    message: rule_message,
                                    code: error_code.clone(),
                                    pattern: None,
                                    field_value: None,
                                    data: None,
                                },
                            );
                        }
                    }
                }
            }
            "maxLength" => {
                if !is_empty {
                    if let Some(max) = rule_active.as_u64() {
                        let len = match field_data {
                            Value::String(s) => s.len(),
                            Value::Array(a) => a.len(),
                            _ => 0,
                        };
                        if len > max as usize {
                            errors.insert(
                                field_path.to_string(),
                                ValidationError {
                                    rule_type: "maxLength".to_string(),
                                    message: rule_message,
                                    code: error_code.clone(),
                                    pattern: None,
                                    field_value: None,
                                    data: None,
                                },
                            );
                        }
                    }
                }
            }
            "minValue" => {
                if !is_empty {
                    if let Some(min) = rule_active.as_f64() {
                        if let Some(val) = field_data.as_f64() {
                            if val < min {
                                errors.insert(
                                    field_path.to_string(),
                                    ValidationError {
                                        rule_type: "minValue".to_string(),
                                        message: rule_message,
                                        code: error_code.clone(),
                                        pattern: None,
                                        field_value: None,
                                        data: None,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            "maxValue" => {
                if !is_empty {
                    if let Some(max) = rule_active.as_f64() {
                        if let Some(val) = field_data.as_f64() {
                            if val > max {
                                errors.insert(
                                    field_path.to_string(),
                                    ValidationError {
                                        rule_type: "maxValue".to_string(),
                                        message: rule_message,
                                        code: error_code.clone(),
                                        pattern: None,
                                        field_value: None,
                                        data: None,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            "pattern" => {
                if !is_empty {
                    if let Some(pattern) = rule_active.as_str() {
                        if let Some(text) = field_data.as_str() {
                            let mut cache = self.regex_cache.write().unwrap();
                            let regex = cache.entry(pattern.to_string()).or_insert_with(|| {
                                regex::Regex::new(pattern).unwrap_or_else(|_| regex::Regex::new("(?:)").unwrap())
                            });
                            if !regex.is_match(text) {
                                errors.insert(
                                    field_path.to_string(),
                                    ValidationError {
                                        rule_type: "pattern".to_string(),
                                        message: rule_message,
                                        code: error_code.clone(),
                                        pattern: Some(pattern.to_string()),
                                        field_value: Some(text.to_string()),
                                        data: None,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            "evaluation" => {
                // Handle array of evaluation rules
                // Format: "evaluation": [{ "code": "...", "message": "...", "$evaluation": {...} }]
                if let Value::Array(eval_array) = &evaluated_rule {
                    for (idx, eval_item) in eval_array.iter().enumerate() {
                        if let Value::Object(eval_obj) = eval_item {
                            // Get the evaluated value (should be in "value" key after evaluation)
                            let eval_result = eval_obj.get("value").unwrap_or(&Value::Bool(true));

                            // Check if result is falsy
                            let is_falsy = match eval_result {
                                Value::Bool(false) => true,
                                Value::Null => true,
                                Value::Number(n) => n.as_f64() == Some(0.0),
                                Value::String(s) => s.is_empty(),
                                Value::Array(a) => a.is_empty(),
                                _ => false,
                            };

                            if is_falsy {
                                let eval_code = eval_obj
                                    .get("code")
                                    .and_then(|c| c.as_str())
                                    .map(|s| s.to_string())
                                    .or_else(|| Some(format!("{}.evaluation.{}", field_path, idx)));

                                let eval_message = eval_obj
                                    .get("message")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Validation failed")
                                    .to_string();

                                let eval_data = eval_obj.get("data").cloned();

                                errors.insert(
                                    field_path.to_string(),
                                    ValidationError {
                                        rule_type: "evaluation".to_string(),
                                        message: eval_message,
                                        code: eval_code,
                                        pattern: None,
                                        field_value: None,
                                        data: eval_data,
                                    },
                                );

                                // Stop at first failure
                                break;
                            }
                        }
                    }
                }
            }
            _ => {
                // Custom evaluation rules
                // In JS: if (!opt.rule.value) then error
                // This handles rules with $evaluation that return false/falsy values
                if !is_empty {
                    // Check if rule_active is falsy (false, 0, null, empty string, empty array)
                    let is_falsy = match &rule_active {
                        Value::Bool(false) => true,
                        Value::Null => true,
                        Value::Number(n) => n.as_f64() == Some(0.0),
                        Value::String(s) => s.is_empty(),
                        Value::Array(a) => a.is_empty(),
                        _ => false,
                    };

                    if is_falsy {
                        errors.insert(
                            field_path.to_string(),
                            ValidationError {
                                rule_type: "evaluation".to_string(),
                                message: rule_message,
                                code: error_code.clone(),
                                pattern: None,
                                field_value: None,
                                data: rule_data,
                            },
                        );
                    }
                }
            }
        }
    }
}
