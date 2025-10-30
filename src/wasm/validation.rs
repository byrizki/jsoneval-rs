//! WASM validation functions

use wasm_bindgen::prelude::*;
use super::types::{JSONEvalWasm, ValidationError, ValidationResult, create_validation_error, create_validation_result};
use super::core::console_log;

#[wasm_bindgen]
impl JSONEvalWasm {
    /// Validate data against schema rules
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @returns ValidationResult
    #[wasm_bindgen]
    pub fn validate(&mut self, data: &str, context: Option<String>) -> Result<ValidationResult, JsValue> {
        let ctx = context.as_deref();
        
        match self.inner.validate(data, ctx, None) {
            Ok(result) => {
                let errors: Vec<ValidationError> = result.errors.iter().map(|(path, error)| {
                    create_validation_error(path.clone(), error)
                }).collect();

                Ok(create_validation_result(result.has_error, errors))
            }
            Err(e) => {
                let error_msg = format!("Validation failed: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
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
    pub fn validate_js(&mut self, data: &str, context: Option<String>) -> Result<JsValue, JsValue> {
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
                    .map_err(|e| {
                        let error_msg = format!("Failed to serialize validation result: {}", e);
                        console_log(&format!("[WASM ERROR] {}", error_msg));
                        JsValue::from_str(&error_msg)
                    })?;
                
                js_sys::JSON::parse(&json_string)
                    .map_err(|e| {
                        let error_msg = format!("JSON parse error: {:?}", e);
                        console_log(&format!("[WASM ERROR] {}", error_msg));
                        JsValue::from_str(&error_msg)
                    })
            }
            Err(e) => {
                let error_msg = format!("Validation failed: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Validate data against schema rules with optional path filtering
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @param paths - Optional array of paths to validate (null for all)
    /// @returns ValidationResult
    #[wasm_bindgen(js_name = validatePaths)]
    pub fn validate_paths(&mut self, data: &str, context: Option<String>, paths: Option<Vec<String>>) -> Result<ValidationResult, JsValue> {
        let ctx = context.as_deref();
        let paths_ref = paths.as_ref().map(|v| v.as_slice());
        
        match self.inner.validate(data, ctx, paths_ref) {
            Ok(result) => {
                let errors: Vec<ValidationError> = result.errors.iter().map(|(path, error)| {
                    create_validation_error(path.clone(), error)
                }).collect();

                Ok(create_validation_result(result.has_error, errors))
            }
            Err(e) => {
                let error_msg = format!("Validation failed: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Validate with path filtering and return as plain JavaScript object (Worker-safe)
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @param paths - Optional array of paths to validate (null for all)
    /// @returns Plain JavaScript object with validation result
    #[wasm_bindgen(js_name = validatePathsJS)]
    pub fn validate_paths_js(&mut self, data: &str, context: Option<String>, paths: Option<Vec<String>>) -> Result<JsValue, JsValue> {
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
                    .map_err(|e| {
                        let error_msg = format!("Failed to serialize validation result: {}", e);
                        console_log(&format!("[WASM ERROR] {}", error_msg));
                        JsValue::from_str(&error_msg)
                    })?;
                
                js_sys::JSON::parse(&json_string)
                    .map_err(|e| {
                        let error_msg = format!("JSON parse error: {:?}", e);
                        console_log(&format!("[WASM ERROR] {}", error_msg));
                        JsValue::from_str(&error_msg)
                    })
            }
            Err(e) => {
                let error_msg = format!("Validation failed: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }
}
