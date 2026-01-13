// Subform methods for isolated array field evaluation

use crate::JSONEval;
use crate::ReturnFormat;
use crate::jsoneval::cancellation::CancellationToken;
use serde_json::Value;

impl JSONEval {
    /// Evaluate a subform with data
    /// Evaluate a subform with data and optional selective paths
    pub fn evaluate_subform(
        &mut self,
        subform_path: &str,
        data: &str,
        context: Option<&str>,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<(), String> {
        let subform = self
            .subforms
            .get_mut(subform_path)
            .ok_or_else(|| format!("Subform not found: {}", subform_path))?;

        subform.evaluate(data, context, paths, token)
    }

    /// Validate subform data against its schema rules
    pub fn validate_subform(
        &mut self,
        subform_path: &str,
        data: &str,
        context: Option<&str>,
        paths: Option<&[String]>,
        token: Option<&CancellationToken>,
    ) -> Result<crate::ValidationResult, String> {
        let subform = self
            .subforms
            .get_mut(subform_path)
            .ok_or_else(|| format!("Subform not found: {}", subform_path))?;

        subform.validate(data, context, paths, token)
    }

    /// Evaluate dependents in subform when a field changes
    pub fn evaluate_dependents_subform(
        &mut self,
        subform_path: &str,
        changed_paths: &[String],
        data: Option<&str>,
        context: Option<&str>,
        re_evaluate: bool,
        token: Option<&CancellationToken>,
        canceled_paths: Option<&mut Vec<String>>,
    ) -> Result<Value, String> {
        let subform = self
            .subforms
            .get_mut(subform_path)
            .ok_or_else(|| format!("Subform not found: {}", subform_path))?;

        subform.evaluate_dependents(changed_paths, data, context, re_evaluate, token, canceled_paths)
    }

    /// Resolve layout for subform
    pub fn resolve_layout_subform(
        &mut self,
        subform_path: &str,
        evaluate: bool,
    ) -> Result<(), String> {
        let subform = self
            .subforms
            .get_mut(subform_path)
            .ok_or_else(|| format!("Subform not found: {}", subform_path))?;

        let _ = subform.resolve_layout(evaluate);
        Ok(())
    }

    /// Get evaluated schema from subform
    pub fn get_evaluated_schema_subform(
        &mut self,
        subform_path: &str,
        resolve_layout: bool,
    ) -> Value {
        if let Some(subform) = self.subforms.get_mut(subform_path) {
            subform.get_evaluated_schema(resolve_layout)
        } else {
            Value::Null
        }
    }

    /// Get schema value from subform in nested object format (all .value fields).
    pub fn get_schema_value_subform(&mut self, subform_path: &str) -> Value {
        if let Some(subform) = self.subforms.get_mut(subform_path) {
            subform.get_schema_value()
        } else {
            Value::Null
        }
    }

    /// Get schema values from subform as a flat array of path-value pairs.
    pub fn get_schema_value_array_subform(&self, subform_path: &str) -> Value {
        if let Some(subform) = self.subforms.get(subform_path) {
            subform.get_schema_value_array()
        } else {
            Value::Array(vec![])
        }
    }

    /// Get schema values from subform as a flat object with dotted path keys.
    pub fn get_schema_value_object_subform(&self, subform_path: &str) -> Value {
        if let Some(subform) = self.subforms.get(subform_path) {
            subform.get_schema_value_object()
        } else {
            Value::Object(serde_json::Map::new())
        }
    }

    /// Get evaluated schema without $params from subform
    pub fn get_evaluated_schema_without_params_subform(
        &mut self,
        subform_path: &str,
        resolve_layout: bool,
    ) -> Value {
        if let Some(subform) = self.subforms.get_mut(subform_path) {
            subform.get_evaluated_schema_without_params(resolve_layout)
        } else {
            Value::Null
        }
    }

    /// Get evaluated schema by specific path from subform
    pub fn get_evaluated_schema_by_path_subform(
        &mut self,
        subform_path: &str,
        schema_path: &str,
        skip_layout: bool,
    ) -> Option<Value> {
        if let Some(subform) = self.subforms.get_mut(subform_path) {
            Some(subform.get_evaluated_schema_by_paths(&[schema_path.to_string()], skip_layout, Some(ReturnFormat::Nested)))
        } else {
            None
        }
    }

    /// Get evaluated schema by multiple paths from subform
    pub fn get_evaluated_schema_by_paths_subform(
        &mut self,
        subform_path: &str,
        schema_paths: &[String],
        skip_layout: bool,
        format: Option<crate::ReturnFormat>,
    ) -> Value {
        if let Some(subform) = self.subforms.get_mut(subform_path) {
            subform.get_evaluated_schema_by_paths(schema_paths, skip_layout, Some(format.unwrap_or(ReturnFormat::Flat)))
        } else {
            match format.unwrap_or_default() {
                crate::ReturnFormat::Array => Value::Array(vec![]),
                _ => Value::Object(serde_json::Map::new()),
            }
        }
    }

    /// Get schema by specific path from subform
    pub fn get_schema_by_path_subform(
        &self,
        subform_path: &str,
        schema_path: &str,
    ) -> Option<Value> {
        if let Some(subform) = self.subforms.get(subform_path) {
            subform.get_schema_by_path(schema_path)
        } else {
            None
        }
    }

    /// Get schema by multiple paths from subform
    pub fn get_schema_by_paths_subform(
        &self,
        subform_path: &str,
        schema_paths: &[String],
        format: Option<crate::ReturnFormat>,
    ) -> Value {
        if let Some(subform) = self.subforms.get(subform_path) {
            subform.get_schema_by_paths(schema_paths, Some(format.unwrap_or(ReturnFormat::Flat)))
        } else {
            match format.unwrap_or_default() {
                crate::ReturnFormat::Array => Value::Array(vec![]),
                _ => Value::Object(serde_json::Map::new()),
            }
        }
    }

    /// Get list of available subform paths
    pub fn get_subform_paths(&self) -> Vec<String> {
        self.subforms.keys().cloned().collect()
    }

    /// Check if a subform exists at the given path
    pub fn has_subform(&self, subform_path: &str) -> bool {
        self.subforms.contains_key(subform_path)
    }
}
