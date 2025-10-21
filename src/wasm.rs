//! WebAssembly bindings for browser and Node.js
//! 
//! This module provides JavaScript/TypeScript compatible bindings

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use crate::JSONEval;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Validation error for JavaScript
#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
pub struct ValidationError {
    path: String,
    rule_type: String,
    message: String,
}

#[wasm_bindgen]
impl ValidationError {
    #[wasm_bindgen(getter)]
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn rule_type(&self) -> String {
        self.rule_type.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

/// Validation result for JavaScript
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct ValidationResult {
    has_error: bool,
    errors: Vec<ValidationError>,
}

#[wasm_bindgen]
impl ValidationResult {
    #[wasm_bindgen(getter)]
    pub fn has_error(&self) -> bool {
        self.has_error
    }

    #[wasm_bindgen(getter)]
    pub fn errors(&self) -> Vec<ValidationError> {
        self.errors.clone()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Get the library version
#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// WebAssembly wrapper for JSONEval
#[wasm_bindgen]
pub struct JSONEvalWasm {
    inner: JSONEval,
}

#[wasm_bindgen]
impl JSONEvalWasm {
    /// Create a new JSONEval instance
    /// 
    /// @param schema - JSON schema string
    /// @param context - Optional context data JSON string
    /// @param data - Optional initial data JSON string
    #[wasm_bindgen(constructor)]
    pub fn new(schema: &str, context: Option<String>, data: Option<String>) -> Result<JSONEvalWasm, JsValue> {
        console_error_panic_hook::set_once();
        
        let ctx = context.as_deref();
        let dt = data.as_deref();
        
        match JSONEval::new(schema, ctx, dt) {
            Ok(eval) => Ok(JSONEvalWasm { inner: eval }),
            Err(e) => {
                let error_msg = format!("Failed to create JSONEval instance: {}", e);
                log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Create a new JSONEval instance from MessagePack-encoded schema
    /// 
    /// @param schemaMsgpack - MessagePack-encoded schema bytes (Uint8Array)
    /// @param context - Optional context data JSON string
    /// @param data - Optional initial data JSON string
    #[wasm_bindgen(js_name = newFromMsgpack)]
    pub fn new_from_msgpack(schema_msgpack: &[u8], context: Option<String>, data: Option<String>) -> Result<JSONEvalWasm, JsValue> {
        console_error_panic_hook::set_once();
        
        let ctx = context.as_deref();
        let dt = data.as_deref();
        
        match JSONEval::new_from_msgpack(schema_msgpack, ctx, dt) {
            Ok(eval) => Ok(JSONEvalWasm { inner: eval }),
            Err(e) => {
                let error_msg = format!("Failed to create JSONEval instance from MessagePack: {}", e);
                log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Evaluate schema with provided data (does not return schema - use getEvaluatedSchema() for that)
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @throws Error if evaluation fails
    #[wasm_bindgen]
    pub fn evaluate(&mut self, data: &str, context: Option<String>) -> Result<(), JsValue> {
        let ctx = context.as_deref();
        
        self.inner.evaluate(data, ctx)
            .map_err(|e| {
                let error_msg = format!("Evaluation failed: {}", e);
                log(&format!("[WASM ERROR] {}", error_msg));
                JsValue::from_str(&error_msg)
            })
    }

    /// Evaluate and return as JsValue for direct JavaScript object access
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @returns Evaluated schema as JavaScript object
    #[wasm_bindgen(js_name = evaluateJS)]
    pub fn evaluate_js(&mut self, data: &str, context: Option<String>) -> Result<JsValue, JsValue> {
        let ctx = context.as_deref();
        
        match self.inner.evaluate(data, ctx) {
            Ok(_) => {
                let result = self.inner.get_evaluated_schema(false);
                serde_wasm_bindgen::to_value(&result)
                    .map_err(|e| {
                        let error_msg = format!("Failed to convert evaluation result to JsValue: {}", e);
                        log(&format!("[WASM ERROR] {}", error_msg));
                        JsValue::from_str(&error_msg)
                    })
            },
            Err(e) => {
                let error_msg = format!("Evaluation failed: {}", e);
                log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Validate data against schema rules
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @returns ValidationResult
    #[wasm_bindgen]
    pub fn validate(&self, data: &str, context: Option<String>) -> Result<ValidationResult, JsValue> {
        let ctx = context.as_deref();
        
        match self.inner.validate(data, ctx, None) {
            Ok(result) => {
                let errors: Vec<ValidationError> = result.errors.iter().map(|(path, error)| {
                    ValidationError {
                        path: path.clone(),
                        rule_type: error.rule_type.clone(),
                        message: error.message.clone(),
                    }
                }).collect();

                Ok(ValidationResult {
                    has_error: result.has_error,
                    errors,
                })
            }
            Err(e) => {
                let error_msg = format!("Validation failed: {}", e);
                log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Validate data and return as plain JavaScript object (Worker-safe)
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @returns Plain JavaScript object with validation result
    #[wasm_bindgen(js_name = validateJS)]
    pub fn validate_js(&self, data: &str, context: Option<String>) -> Result<JsValue, JsValue> {
        let ctx = context.as_deref();
        
        match self.inner.validate(data, ctx, None) {
            Ok(result) => {
                let errors: Vec<serde_json::Value> = result.errors.iter().map(|(path, error)| {
                    serde_json::json!({
                        "path": path,
                        "rule_type": error.rule_type,
                        "message": error.message,
                    })
                }).collect();

                let validation_result = serde_json::json!({
                    "has_error": result.has_error,
                    "errors": errors,
                });

                // Use JSON.parse to ensure plain object (not Map)
                let json_string = serde_json::to_string(&validation_result)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                
                js_sys::JSON::parse(&json_string)
                    .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))
            }
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Re-evaluate dependents when a field changes (processes transitively)
    /// 
    /// @param changedPath - Path of the field that changed
    /// @param data - Optional updated JSON data string (null to use existing data)
    /// @param context - Optional context data JSON string
    /// @returns Array of dependent change objects as JSON string
    #[wasm_bindgen(js_name = evaluateDependents)]
    pub fn evaluate_dependents(
        &mut self,
        changed_path: &str,
        data: Option<String>,
        context: Option<String>,
    ) -> Result<String, JsValue> {
        let data_str = data.as_deref();
        let ctx = context.as_deref();
        
        match self.inner.evaluate_dependents(changed_path, data_str, ctx) {
            Ok(result) => serde_json::to_string(&result)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Re-evaluate dependents and return as JsValue
    /// 
    /// @param changedPath - Path of the field that changed
    /// @param data - Optional updated JSON data string (null to use existing data)
    /// @param context - Optional context data JSON string
    /// @returns Array of dependent change objects as JavaScript object
    #[wasm_bindgen(js_name = evaluateDependentsJS)]
    pub fn evaluate_dependents_js(
        &mut self,
        changed_path: &str,
        data: Option<String>,
        context: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let data_str = data.as_deref();
        let ctx = context.as_deref();
        
        match self.inner.evaluate_dependents(changed_path, data_str, ctx) {
            Ok(result) => serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Get the evaluated schema with optional layout resolution
    /// 
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Evaluated schema as JSON string
    #[wasm_bindgen(js_name = getEvaluatedSchema)]
    pub fn get_evaluated_schema(&mut self, skip_layout: bool) -> String {
        let result = self.inner.get_evaluated_schema(skip_layout);
        serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get the evaluated schema as JavaScript object
    /// 
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Evaluated schema as JavaScript object
    #[wasm_bindgen(js_name = getEvaluatedSchemaJS)]
    pub fn get_evaluated_schema_js(&mut self, skip_layout: bool) -> Result<JsValue, JsValue> {
        let result = self.inner.get_evaluated_schema(skip_layout);
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get the evaluated schema in MessagePack format
    /// 
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Evaluated schema as MessagePack bytes (Uint8Array)
    /// 
    /// # Zero-Copy Optimization
    /// 
    /// This method returns MessagePack binary data with minimal copying:
    /// 1. Serializes schema to Vec<u8> in Rust (unavoidable)
    /// 2. wasm-bindgen transfers Vec<u8> to JS as Uint8Array (optimized)
    /// 3. Result is a Uint8Array view (minimal overhead)
    /// 
    /// MessagePack format is 20-50% smaller than JSON, ideal for web/WASM.
    #[wasm_bindgen(js_name = getEvaluatedSchemaMsgpack)]
    pub fn get_evaluated_schema_msgpack(&mut self, skip_layout: bool) -> Result<Vec<u8>, JsValue> {
        self.inner.get_evaluated_schema_msgpack(skip_layout)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get all schema values (evaluations ending with .value)
    /// Mutates internal data by overriding with values from value evaluations
    /// 
    /// @returns Modified data as JavaScript object
    #[wasm_bindgen(js_name = getSchemaValue)]
    pub fn get_schema_value(&mut self) -> Result<JsValue, JsValue> {
        let result = self.inner.get_schema_value();
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get the evaluated schema without $params field
    /// 
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Evaluated schema as JSON string
    #[wasm_bindgen(js_name = getEvaluatedSchemaWithoutParams)]
    pub fn get_evaluated_schema_without_params(&mut self, skip_layout: bool) -> String {
        let result = self.inner.get_evaluated_schema_without_params(skip_layout);
        serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get the evaluated schema without $params as JavaScript object
    /// 
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Evaluated schema as JavaScript object
    #[wasm_bindgen(js_name = getEvaluatedSchemaWithoutParamsJS)]
    pub fn get_evaluated_schema_without_params_js(&mut self, skip_layout: bool) -> Result<JsValue, JsValue> {
        let result = self.inner.get_evaluated_schema_without_params(skip_layout);
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get a value from the evaluated schema using dotted path notation
    /// 
    /// @param path - Dotted path to the value (e.g., "properties.field.value")
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Value as JSON string or null if not found
    #[wasm_bindgen(js_name = getEvaluatedSchemaByPath)]
    pub fn get_evaluated_schema_by_path(&mut self, path: &str, skip_layout: bool) -> Option<String> {
        self.inner.get_evaluated_schema_by_path(path, skip_layout)
            .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string()))
    }

    /// Get a value from the evaluated schema using dotted path notation as JavaScript object
    /// 
    /// @param path - Dotted path to the value (e.g., "properties.field.value")
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Value as JavaScript object or null if not found
    #[wasm_bindgen(js_name = getEvaluatedSchemaByPathJS)]
    pub fn get_evaluated_schema_by_path_js(&mut self, path: &str, skip_layout: bool) -> Result<JsValue, JsValue> {
        match self.inner.get_evaluated_schema_by_path(path, skip_layout) {
            Some(value) => serde_wasm_bindgen::to_value(&value)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            None => Ok(JsValue::NULL),
        }
    }

    /// Reload schema with new data
    /// 
    /// @param schema - New JSON schema string
    /// @param context - Optional context data JSON string
    /// @param data - Optional initial data JSON string
    #[wasm_bindgen(js_name = reloadSchema)]
    pub fn reload_schema(&mut self, schema: &str, context: Option<String>, data: Option<String>) -> Result<(), JsValue> {
        let ctx = context.as_deref();
        let dt = data.as_deref();
        
        self.inner.reload_schema(schema, ctx, dt)
            .map_err(|e| {
                let error_msg = format!("Failed to reload schema: {}", e);
                log(&format!("[WASM ERROR] {}", error_msg));
                JsValue::from_str(&error_msg)
            })
    }

    /// Get cache statistics
    /// 
    /// @returns Cache statistics as JavaScript object with hits, misses, and entries
    #[wasm_bindgen(js_name = cacheStats)]
    pub fn cache_stats(&self) -> Result<JsValue, JsValue> {
        let stats = self.inner.cache_stats();
        let stats_obj = serde_json::json!({
            "hits": stats.hits,
            "misses": stats.misses,
            "entries": stats.entries,
        });
        serde_wasm_bindgen::to_value(&stats_obj)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Clear the evaluation cache
    #[wasm_bindgen(js_name = clearCache)]
    pub fn clear_cache(&mut self) {
        self.inner.clear_cache();
    }

    /// Get the number of cached entries
    /// 
    /// @returns Number of cached entries
    #[wasm_bindgen(js_name = cacheLen)]
    pub fn cache_len(&self) -> usize {
        self.inner.cache_len()
    }

    /// Validate data against schema rules with optional path filtering
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @param paths - Optional array of paths to validate (null for all)
    /// @returns ValidationResult
    #[wasm_bindgen(js_name = validatePaths)]
    pub fn validate_paths(&self, data: &str, context: Option<String>, paths: Option<Vec<String>>) -> Result<ValidationResult, JsValue> {
        let ctx = context.as_deref();
        let paths_ref = paths.as_ref().map(|v| v.as_slice());
        
        match self.inner.validate(data, ctx, paths_ref) {
            Ok(result) => {
                let errors: Vec<ValidationError> = result.errors.iter().map(|(path, error)| {
                    ValidationError {
                        path: path.clone(),
                        rule_type: error.rule_type.clone(),
                        message: error.message.clone(),
                    }
                }).collect();

                Ok(ValidationResult {
                    has_error: result.has_error,
                    errors,
                })
            }
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Validate with path filtering and return as plain JavaScript object (Worker-safe)
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @param paths - Optional array of paths to validate (null for all)
    /// @returns Plain JavaScript object with validation result
    #[wasm_bindgen(js_name = validatePathsJS)]
    pub fn validate_paths_js(&self, data: &str, context: Option<String>, paths: Option<Vec<String>>) -> Result<JsValue, JsValue> {
        let ctx = context.as_deref();
        let paths_ref = paths.as_ref().map(|v| v.as_slice());
        
        match self.inner.validate(data, ctx, paths_ref) {
            Ok(result) => {
                let errors: Vec<serde_json::Value> = result.errors.iter().map(|(path, error)| {
                    serde_json::json!({
                        "path": path,
                        "rule_type": error.rule_type,
                        "message": error.message,
                    })
                }).collect();

                let validation_result = serde_json::json!({
                    "has_error": result.has_error,
                    "errors": errors,
                });

                // Use JSON.parse to ensure plain object (not Map)
                let json_string = serde_json::to_string(&validation_result)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                
                js_sys::JSON::parse(&json_string)
                    .map_err(|e| JsValue::from_str(&format!("JSON parse error: {:?}", e)))
            }
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Resolve layout with optional evaluation
    /// 
    /// @param evaluate - If true, runs evaluation before resolving layout
    /// @throws Error if resolve fails
    #[wasm_bindgen(js_name = resolveLayout)]
    pub fn resolve_layout(&mut self, evaluate: bool) -> Result<(), JsValue> {
        self.inner.resolve_layout(evaluate)
            .map_err(|e| {
                let error_msg = format!("Failed to resolve layout: {}", e);
                log(&format!("[WASM ERROR] {}", error_msg));
                JsValue::from_str(&error_msg)
            })
    }

    /// Compile and run JSON logic from a JSON logic string
    /// 
    /// @param logicStr - JSON logic expression as a string
    /// @param data - Optional JSON data string (null to use existing data)
    /// @returns Result as JSON string
    /// @throws Error if compilation or evaluation fails
    #[wasm_bindgen(js_name = compileAndRunLogic)]
    pub fn compile_and_run_logic(&mut self, logic_str: &str, data: Option<String>) -> Result<String, JsValue> {
        let data_str = data.as_deref();
        
        match self.inner.compile_and_run_logic(logic_str, data_str) {
            Ok(result) => serde_json::to_string(&result)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => {
                let error_msg = format!("Failed to compile and run logic: {}", e);
                log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Compile and run JSON logic and return as JavaScript object
    /// 
    /// @param logicStr - JSON logic expression as a string
    /// @param data - Optional JSON data string (null to use existing data)
    /// @returns Result as JavaScript object
    /// @throws Error if compilation or evaluation fails
    #[wasm_bindgen(js_name = compileAndRunLogicJS)]
    pub fn compile_and_run_logic_js(&mut self, logic_str: &str, data: Option<String>) -> Result<JsValue, JsValue> {
        let data_str = data.as_deref();
        
        match self.inner.compile_and_run_logic(logic_str, data_str) {
            Ok(result) => serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => {
                let error_msg = format!("Failed to compile and run logic: {}", e);
                log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    // ============================================================================
    // Subform Methods
    // ============================================================================

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
                log(&format!("[WASM ERROR] {}", error_msg));
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
    pub fn validate_subform(&self, subform_path: &str, data: &str, context: Option<String>) -> Result<ValidationResult, JsValue> {
        let ctx = context.as_deref();
        
        match self.inner.validate_subform(subform_path, data, ctx, None) {
            Ok(result) => {
                let errors: Vec<ValidationError> = result.errors.iter().map(|(path, error)| {
                    ValidationError {
                        path: path.clone(),
                        rule_type: error.rule_type.clone(),
                        message: error.message.clone(),
                    }
                }).collect();

                Ok(ValidationResult {
                    has_error: result.has_error,
                    errors,
                })
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
        
        match self.inner.evaluate_dependents_subform(subform_path, changed_path, data_str, ctx) {
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
        
        match self.inner.evaluate_dependents_subform(subform_path, changed_path, data_str, ctx) {
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
                log(&format!("[WASM ERROR] {}", error_msg));
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

/// Get library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Initialize the library (sets up panic hook)
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}
