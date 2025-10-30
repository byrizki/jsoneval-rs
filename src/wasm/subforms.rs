//! WASM subform functions

use wasm_bindgen::prelude::*;
use super::types::{JSONEvalWasm, ValidationError, ValidationResult, create_validation_error, create_validation_result};
use super::core::console_log;

#[wasm_bindgen]
impl JSONEvalWasm {
    /// Evaluate a subform with data
    /// 
    /// @param subformPath - Path to the subform (e.g., "#/riders")
    /// @param data - JSON data string for the subform
    /// @param context - Optional context data JSON string
    /// @throws Error if evaluation fails
    #[wasm_bindgen(js_name = evaluateSubform)]
    pub fn evaluate_subform(&mut self, subform_path: &str, data: &str, context: Option<String>) -> Result<(), JsValue> {
        let ctx = context.as_deref();
        
        self.inner.evaluate_subform(subform_path, data, ctx)
            .map_err(|e| {
                let error_msg = format!("Subform evaluation failed: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                JsValue::from_str(&error_msg)
            })
    }

    /// Validate subform data against its schema rules
    /// 
    /// @param subformPath - Path to the subform
    /// @param data - JSON data string for the subform
    /// @param context - Optional context data JSON string
    /// @returns ValidationResult
    #[wasm_bindgen(js_name = validateSubform)]
    pub fn validate_subform(&mut self, subform_path: &str, data: &str, context: Option<String>) -> Result<ValidationResult, JsValue> {
        let ctx = context.as_deref();
        
        match self.inner.validate_subform(subform_path, data, ctx, None) {
            Ok(result) => {
                let errors: Vec<ValidationError> = result.errors.iter().map(|(path, error)| {
                    create_validation_error(path.clone(), error)
                }).collect();

                Ok(create_validation_result(result.has_error, errors))
            }
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Evaluate dependents in subform when a field changes
    /// 
    /// @param subformPath - Path to the subform
    /// @param changedPath - Path of the field that changed
    /// @param data - Optional updated JSON data string
    /// @param context - Optional context data JSON string
    /// @returns Array of dependent change objects as JSON string
    #[wasm_bindgen(js_name = evaluateDependentsSubform)]
    pub fn evaluate_dependents_subform(
        &mut self,
        subform_path: &str,
        changed_path: &str,
        data: Option<String>,
        context: Option<String>,
    ) -> Result<String, JsValue> {
        let data_str = data.as_deref();
        let ctx = context.as_deref();
        
        // Wrap single path in a Vec for the new API
        let paths = vec![changed_path.to_string()];
        
        match self.inner.evaluate_dependents_subform(subform_path, &paths, data_str, ctx, false) {
            Ok(result) => serde_json::to_string(&result)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Evaluate dependents in subform and return as JavaScript object
    /// 
    /// @param subformPath - Path to the subform
    /// @param changedPath - Path of the field that changed
    /// @param data - Optional updated JSON data string
    /// @param context - Optional context data JSON string
    /// @returns Array of dependent change objects as JavaScript object
    #[wasm_bindgen(js_name = evaluateDependentsSubformJS)]
    pub fn evaluate_dependents_subform_js(
        &mut self,
        subform_path: &str,
        changed_path: &str,
        data: Option<String>,
        context: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let data_str = data.as_deref();
        let ctx = context.as_deref();
        
        // Wrap single path in a Vec for the new API
        let paths = vec![changed_path.to_string()];
        
        match self.inner.evaluate_dependents_subform(subform_path, &paths, data_str, ctx, false) {
            Ok(result) => serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Resolve layout for subform
    /// 
    /// @param subformPath - Path to the subform
    /// @param evaluate - If true, runs evaluation before resolving layout
    /// @throws Error if resolve fails
    #[wasm_bindgen(js_name = resolveLayoutSubform)]
    pub fn resolve_layout_subform(&mut self, subform_path: &str, evaluate: bool) -> Result<(), JsValue> {
        self.inner.resolve_layout_subform(subform_path, evaluate)
            .map_err(|e| {
                let error_msg = format!("Failed to resolve subform layout: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                JsValue::from_str(&error_msg)
            })
    }

    /// Get evaluated schema from subform
    /// 
    /// @param subformPath - Path to the subform
    /// @param resolveLayout - Whether to resolve layout
    /// @returns Evaluated schema as JSON string
    #[wasm_bindgen(js_name = getEvaluatedSchemaSubform)]
    pub fn get_evaluated_schema_subform(&mut self, subform_path: &str, resolve_layout: bool) -> String {
        let result = self.inner.get_evaluated_schema_subform(subform_path, resolve_layout);
        serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get evaluated schema from subform as JavaScript object
    /// 
    /// @param subformPath - Path to the subform
    /// @param resolveLayout - Whether to resolve layout
    /// @returns Evaluated schema as JavaScript object
    #[wasm_bindgen(js_name = getEvaluatedSchemaSubformJS)]
    pub fn get_evaluated_schema_subform_js(&mut self, subform_path: &str, resolve_layout: bool) -> Result<JsValue, JsValue> {
        let result = self.inner.get_evaluated_schema_subform(subform_path, resolve_layout);
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get schema value from subform (all .value fields)
    /// 
    /// @param subformPath - Path to the subform
    /// @returns Modified data as JavaScript object
    #[wasm_bindgen(js_name = getSchemaValueSubform)]
    pub fn get_schema_value_subform(&mut self, subform_path: &str) -> Result<JsValue, JsValue> {
        let result = self.inner.get_schema_value_subform(subform_path);
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get evaluated schema without $params from subform
    /// 
    /// @param subformPath - Path to the subform
    /// @param resolveLayout - Whether to resolve layout
    /// @returns Evaluated schema as JSON string
    #[wasm_bindgen(js_name = getEvaluatedSchemaWithoutParamsSubform)]
    pub fn get_evaluated_schema_without_params_subform(&mut self, subform_path: &str, resolve_layout: bool) -> String {
        let result = self.inner.get_evaluated_schema_without_params_subform(subform_path, resolve_layout);
        serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get evaluated schema without $params from subform as JavaScript object
    /// 
    /// @param subformPath - Path to the subform
    /// @param resolveLayout - Whether to resolve layout
    /// @returns Evaluated schema as JavaScript object
    #[wasm_bindgen(js_name = getEvaluatedSchemaWithoutParamsSubformJS)]
    pub fn get_evaluated_schema_without_params_subform_js(&mut self, subform_path: &str, resolve_layout: bool) -> Result<JsValue, JsValue> {
        let result = self.inner.get_evaluated_schema_without_params_subform(subform_path, resolve_layout);
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get evaluated schema by specific path from subform
    /// 
    /// @param subformPath - Path to the subform
    /// @param schemaPath - Dotted path to the value within the subform
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Value as JSON string or null if not found
    #[wasm_bindgen(js_name = getEvaluatedSchemaByPathSubform)]
    pub fn get_evaluated_schema_by_path_subform(&mut self, subform_path: &str, schema_path: &str, skip_layout: bool) -> Option<String> {
        self.inner.get_evaluated_schema_by_path_subform(subform_path, schema_path, skip_layout)
            .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string()))
    }

    /// Get evaluated schema by specific path from subform as JavaScript object
    /// 
    /// @param subformPath - Path to the subform
    /// @param schemaPath - Dotted path to the value within the subform
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Value as JavaScript object or null if not found
    #[wasm_bindgen(js_name = getEvaluatedSchemaByPathSubformJS)]
    pub fn get_evaluated_schema_by_path_subform_js(&mut self, subform_path: &str, schema_path: &str, skip_layout: bool) -> Result<JsValue, JsValue> {
        match self.inner.get_evaluated_schema_by_path_subform(subform_path, schema_path, skip_layout) {
            Some(value) => serde_wasm_bindgen::to_value(&value)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            None => Ok(JsValue::NULL),
        }
    }

    /// Get values from the evaluated schema of a subform using multiple dotted path notations (returns JSON string)
    /// @param subformPath - Path to the subform
    /// @param pathsJson - JSON array of dotted paths
    /// @param skipLayout - Whether to skip layout resolution
    /// @param format - Return format (0=Nested, 1=Flat, 2=Array)
    /// @returns Data in specified format as JSON string
    #[wasm_bindgen(js_name = getEvaluatedSchemaByPathsSubform)]
    pub fn get_evaluated_schema_by_paths_subform(&mut self, subform_path: &str, paths_json: &str, skip_layout: bool, format: u8) -> Result<String, JsValue> {
        // Parse JSON array of paths
        let paths: Vec<String> = serde_json::from_str(paths_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse paths JSON: {}", e)))?;
        
        let return_format = match format {
            1 => crate::ReturnFormat::Flat,
            2 => crate::ReturnFormat::Array,
            _ => crate::ReturnFormat::Nested,
        };
        
        let result = self.inner.get_evaluated_schema_by_paths_subform(subform_path, &paths, skip_layout, Some(return_format));
        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get values from the evaluated schema of a subform using multiple dotted path notations (returns JS object)
    /// @param subformPath - Path to the subform
    /// @param pathsJson - JSON array of dotted paths
    /// @param skipLayout - Whether to skip layout resolution
    /// @param format - Return format (0=Nested, 1=Flat, 2=Array)
    /// @returns Data in specified format as JavaScript object
    #[wasm_bindgen(js_name = getEvaluatedSchemaByPathsSubformJS)]
    pub fn get_evaluated_schema_by_paths_subform_js(&mut self, subform_path: &str, paths_json: &str, skip_layout: bool, format: u8) -> Result<JsValue, JsValue> {
        // Parse JSON array of paths
        let paths: Vec<String> = serde_json::from_str(paths_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse paths JSON: {}", e)))?;
        
        let return_format = match format {
            1 => crate::ReturnFormat::Flat,
            2 => crate::ReturnFormat::Array,
            _ => crate::ReturnFormat::Nested,
        };
        
        let result = self.inner.get_evaluated_schema_by_paths_subform(subform_path, &paths, skip_layout, Some(return_format));
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get schema by specific path from subform (returns JSON string)
    /// @param subformPath - Path to the subform
    /// @param schemaPath - Path within the subform
    /// @returns Value as JSON string or null if not found
    #[wasm_bindgen(js_name = getSchemaByPathSubform)]
    pub fn get_schema_by_path_subform(&self, subform_path: &str, schema_path: &str) -> Option<String> {
        self.inner.get_schema_by_path_subform(subform_path, schema_path)
            .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string()))
    }

    /// Get schema by specific path from subform (returns JS object)
    /// @param subformPath - Path to the subform
    /// @param schemaPath - Path within the subform
    /// @returns Value as JavaScript object or null if not found
    #[wasm_bindgen(js_name = getSchemaByPathSubformJS)]
    pub fn get_schema_by_path_subform_js(&self, subform_path: &str, schema_path: &str) -> Result<JsValue, JsValue> {
        match self.inner.get_schema_by_path_subform(subform_path, schema_path) {
            Some(value) => serde_wasm_bindgen::to_value(&value)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            None => Ok(JsValue::NULL),
        }
    }

    /// Get schema by multiple paths from subform
    /// @param subformPath - Path to the subform
    /// @param pathsJson - JSON array of dotted paths
    /// @param format - Return format (0=Nested, 1=Flat, 2=Array)
    /// @returns Data in specified format as JSON string
    #[wasm_bindgen(js_name = getSchemaByPathsSubform)]
    pub fn get_schema_by_paths_subform(&self, subform_path: &str, paths_json: &str, format: u8) -> Result<String, JsValue> {
        // Parse JSON array of paths
        let paths: Vec<String> = serde_json::from_str(paths_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse paths JSON: {}", e)))?;
        
        let return_format = match format {
            1 => crate::ReturnFormat::Flat,
            2 => crate::ReturnFormat::Array,
            _ => crate::ReturnFormat::Nested,
        };
        
        let result = self.inner.get_schema_by_paths_subform(subform_path, &paths, Some(return_format));
        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get schema by multiple paths from subform (JS object)
    /// @param subformPath - Path to the subform
    /// @param pathsJson - JSON array of dotted paths
    /// @param format - Return format (0=Nested, 1=Flat, 2=Array)
    /// @returns Data in specified format as JavaScript object
    #[wasm_bindgen(js_name = getSchemaByPathsSubformJS)]
    pub fn get_schema_by_paths_subform_js(&self, subform_path: &str, paths_json: &str, format: u8) -> Result<JsValue, JsValue> {
        // Parse JSON array of paths
        let paths: Vec<String> = serde_json::from_str(paths_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse paths JSON: {}", e)))?;
        
        let return_format = match format {
            1 => crate::ReturnFormat::Flat,
            2 => crate::ReturnFormat::Array,
            _ => crate::ReturnFormat::Nested,
        };
        
        let result = self.inner.get_schema_by_paths_subform(subform_path, &paths, Some(return_format));
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get list of available subform paths
    /// 
    /// @returns Array of subform paths
    #[wasm_bindgen(js_name = getSubformPaths)]
    pub fn get_subform_paths(&self) -> Vec<String> {
        self.inner.get_subform_paths()
    }

    /// Check if a subform exists at the given path
    /// 
    /// @param subformPath - Path to check
    /// @returns True if subform exists, false otherwise
    #[wasm_bindgen(js_name = hasSubform)]
    pub fn has_subform(&self, subform_path: &str) -> bool {
        self.inner.has_subform(subform_path)
    }
}
