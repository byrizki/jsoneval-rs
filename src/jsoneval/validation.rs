use super::JSONEval;
use crate::jsoneval::cancellation::CancellationToken;
use crate::jsoneval::json_parser;
use crate::jsoneval::path_utils;
use crate::jsoneval::types::{ValidationError, ValidationResult};

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

            // Parse and update data
            let data_value = json_parser::parse_json_str(data)?;
            let context_value = if let Some(ctx) = context {
                json_parser::parse_json_str(ctx)?
            } else {
                Value::Object(serde_json::Map::new())
            };

            // Update context
            self.context = context_value.clone();

            // Update eval_data with new data/context
            self.eval_data
                .replace_data_and_context(data_value.clone(), context_value);

            // Drop lock before calling evaluate_others which needs mutable access
            drop(_lock);

            // Re-evaluate rule evaluations to ensure fresh values
            // This ensures all rule.$evaluation expressions are re-computed
            // Always pass had_cache_miss=true for validation: rules must always re-run.
            self.evaluate_others(paths, token, true);

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

    /// Validate using the data already present in `eval_data` (set by `with_item_cache_swap`).
    ///
    /// Skips JSON parsing and `replace_data_and_context` — use this inside the
    /// cache-swap closure to avoid redundant work when the subform data is already set.
    pub(crate) fn validate_pre_set(
        &mut self,
        data_value: Value,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<crate::ValidationResult, String> {
        // Re-evaluate rule evaluations with the current (already-set) data.
        self.evaluate_others(paths, token, true);
        self.evaluated_schema = self.get_evaluated_schema(false);

        let mut errors: IndexMap<String, ValidationError> = IndexMap::new();

        let fields: Vec<String> = self.fields_with_rules.iter().cloned().collect();
        for field_path in &fields {
            if let Some(filter_paths) = paths {
                if !filter_paths.is_empty()
                    && !filter_paths.iter().any(|p| {
                        field_path.starts_with(p.as_str()) || p.starts_with(field_path.as_str())
                    })
                {
                    continue;
                }
            }
            if let Some(t) = token {
                if t.is_cancelled() {
                    return Err("Cancelled".to_string());
                }
            }
            self.validate_field(field_path, &data_value, &mut errors);
        }

        let has_error = !errors.is_empty();
        Ok(crate::ValidationResult { has_error, errors })
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

        let schema_type = schema_map
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("");

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
            "minLength" | "maxLength" | "minValue" | "maxValue" => {
                if rule_value_fails(rule_name, &rule_active, field_data, is_empty, schema_type) {
                    errors.insert(
                        field_path.to_string(),
                        ValidationError {
                            rule_type: rule_name.to_string(),
                            message: rule_message,
                            code: error_code.clone(),
                            pattern: None,
                            field_value: None,
                            data: None,
                        },
                    );
                }
            }

            "pattern" => {
                if !is_empty {
                    if let Some(pattern) = rule_active.as_str() {
                        if let Some(text) = field_data.as_str() {
                            let mut cache = self.regex_cache.write().unwrap();
                            let regex = cache.entry(pattern.to_string()).or_insert_with(|| {
                                regex::Regex::new(pattern)
                                    .unwrap_or_else(|_| regex::Regex::new("(?:)").unwrap())
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
                if rule_value_fails(rule_name, &rule_active, field_data, is_empty, schema_type) {
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

    /// Returns `true` if `field_data` fails any of the dep field's schema rules.
    ///
    /// Rules are evaluated on-demand: compiled `LogicId`s from `self.evaluations` (set at
    /// construction time) are executed directly against `scope_data`, completely bypassing
    /// `evaluated_schema`. This avoids stale-cache issues during table dependency checks.
    /// Unlike `validate_field`, this also evaluates the `required` rule on-demand.
    pub(crate) fn dep_fails_schema_rules(
        &self,
        field_path: &str,
        field_data: &Value,
        scope_data: &Value,
    ) -> bool {
        let schema_pointer = path_utils::dot_notation_to_schema_pointer(field_path);
        let pointer = schema_pointer.trim_start_matches('#');

        let field_schema = match self.schema.pointer(pointer) {
            Some(s) => s,
            None => {
                let alt_pointer = format!("/properties{}", pointer);
                match self.schema.pointer(&alt_pointer) {
                    Some(s) => s,
                    None => return false,
                }
            }
        };

        let schema_map = match field_schema.as_object() {
            Some(m) => m,
            None => return false,
        };

        let rules = match schema_map.get("rules") {
            Some(Value::Object(r)) => r,
            _ => return false,
        };

        let schema_type = schema_map
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let is_empty = matches!(field_data, Value::Null)
            || field_data.as_str().map_or(false, |s| s.is_empty())
            || field_data.as_array().map_or(false, |a| a.is_empty());

        for (rule_name, rule_value) in rules {
            // Resolve the rule's active value on-demand.
            // If a compiled LogicId exists in self.evaluations for this rule path, run it fresh
            // against scope_data. Otherwise fall back to the static "value" from the raw schema.
            let rule_eval_key = format!("#{}/rules/{}", pointer, rule_name);
            let rule_active: Value = if let Some(logic_id) = self.evaluations.get(&rule_eval_key) {
                let empty_ctx = Value::Object(serde_json::Map::new());
                self.engine
                    .run_with_context(logic_id, scope_data, &empty_ctx)
                    .unwrap_or(Value::Null)
            } else {
                match rule_value {
                    Value::Object(obj) => obj.get("value").cloned().unwrap_or(Value::Null),
                    other => other.clone(),
                }
            };

            if rule_value_fails(rule_name, &rule_active, field_data, is_empty, schema_type) {
                return true;
            }
        }

        false
    }
}

/// Pure rule-check: returns `true` if `rule_active` indicates `field_data` fails the rule.
///
/// This is the shared comparison kernel used by both `validate_rule` (full validation path)
/// and `dep_fails_schema_rules` (on-demand dep checking). It is intentionally free of any
/// schema/cache lookups — callers are responsible for resolving `rule_active` beforehand.
///
/// Handles: `required`, `minLength`, `maxLength`, `minValue`, `maxValue`, and custom/dynamic.
/// Does NOT handle: `pattern` (needs regex cache), `evaluation` array format (complex structure).
fn rule_value_fails(
    rule_name: &str,
    rule_active: &Value,
    field_data: &Value,
    is_empty: bool,
    schema_type: &str,
) -> bool {
    let coerce_num = |v: &Value| -> Option<f64> {
        if let Some(n) = v.as_f64() {
            return Some(n);
        }
        if matches!(schema_type, "number" | "integer") {
            if let Some(s) = v.as_str() {
                return s.trim().parse::<f64>().ok();
            }
        }
        None
    };

    match rule_name {
        "required" => is_empty && matches!(rule_active, Value::Bool(true)),
        "minLength" => {
            if is_empty {
                false
            } else if let Some(min) = rule_active.as_u64() {
                let len = match field_data {
                    Value::String(s) => s.len(),
                    Value::Array(a) => a.len(),
                    _ => 0,
                };
                len < min as usize
            } else {
                false
            }
        }
        "maxLength" => {
            if is_empty {
                false
            } else if let Some(max) = rule_active.as_u64() {
                let len = match field_data {
                    Value::String(s) => s.len(),
                    Value::Array(a) => a.len(),
                    _ => 0,
                };
                len > max as usize
            } else {
                false
            }
        }
        "minValue" => {
            if is_empty {
                false
            } else if let Some(min) = rule_active.as_f64() {
                coerce_num(field_data).map_or(false, |v| v < min)
            } else {
                false
            }
        }
        "maxValue" => {
            if is_empty {
                false
            } else if let Some(max) = rule_active.as_f64() {
                coerce_num(field_data).map_or(false, |v| v > max)
            } else {
                false
            }
        }
        // pattern and evaluation array are handled by their specific callers
        "pattern" | "evaluation" => false,
        _ => {
            // Custom/dynamic rule: falsy rule_active = constraint not met = field invalid
            if is_empty {
                false
            } else {
                matches!(rule_active, Value::Bool(false) | Value::Null)
                    || rule_active.as_f64() == Some(0.0)
                    || rule_active.as_str().map_or(false, |s| s.is_empty())
                    || rule_active.as_array().map_or(false, |a| a.is_empty())
            }
        }
    }
}
