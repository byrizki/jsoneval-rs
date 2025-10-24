// Subform methods for isolated array field evaluation

use crate::JSONEval;
use serde_json::Value;

impl JSONEval {
    /// Evaluate a subform with data
    pub fn evaluate_subform(
        &mut self,
        subform_path: &str,
        data: &str,
        context: Option<&str>,
    ) -> Result<(), String> {
        let subform = self.subforms.get_mut(subform_path)
            .ok_or_else(|| format!("Subform not found: {}", subform_path))?;
        
        subform.evaluate(data, context)
    }
    
    /// Validate subform data against its schema rules
    pub fn validate_subform(
        &mut self,
        subform_path: &str,
        data: &str,
        context: Option<&str>,
        paths: Option<&[String]>,
    ) -> Result<crate::ValidationResult, String> {
        let subform = self.subforms.get_mut(subform_path)
            .ok_or_else(|| format!("Subform not found: {}", subform_path))?;
        
        subform.validate(data, context, paths)
    }
    
    /// Evaluate dependents in subform when a field changes
    pub fn evaluate_dependents_subform(
        &mut self,
        subform_path: &str,
        changed_paths: &[String],
        data: Option<&str>,
        context: Option<&str>,
        re_evaluate: bool,
    ) -> Result<Value, String> {
        let subform = self.subforms.get_mut(subform_path)
            .ok_or_else(|| format!("Subform not found: {}", subform_path))?;
        
        subform.evaluate_dependents(changed_paths, data, context, re_evaluate)
    }
    
    /// Resolve layout for subform
    pub fn resolve_layout_subform(
        &mut self,
        subform_path: &str,
        evaluate: bool,
    ) -> Result<(), String> {
        let subform = self.subforms.get_mut(subform_path)
            .ok_or_else(|| format!("Subform not found: {}", subform_path))?;
        
        subform.resolve_layout(evaluate)
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
    
    /// Get schema value from subform (all .value fields)
    pub fn get_schema_value_subform(
        &mut self,
        subform_path: &str,
    ) -> Value {
        if let Some(subform) = self.subforms.get_mut(subform_path) {
            subform.get_schema_value()
        } else {
            Value::Null
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
            subform.get_evaluated_schema_by_path(schema_path, skip_layout)
        } else {
            None
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
