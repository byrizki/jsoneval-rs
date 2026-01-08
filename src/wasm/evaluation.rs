//! WASM evaluation functions

use wasm_bindgen::prelude::*;
use super::types::JSONEvalWasm;
use super::core::console_log;

#[wasm_bindgen]
impl JSONEvalWasm {
    /// Evaluate schema with provided data (does not return schema - use getEvaluatedSchema() for that)
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @throws Error if evaluation fails
    #[wasm_bindgen]
    pub fn evaluate(&mut self, data: &str, context: Option<String>, paths: Option<Vec<String>>) -> Result<(), JsValue> {
        let ctx = context.as_deref();
        let paths_refs: Option<Vec<String>> = paths;
        // Convert Vec<String> to &[String] for evaluate
        // We need to keep the Vec alive if it exists
        
        match self.inner.evaluate(data, ctx, paths_refs.as_deref()) {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_msg = format!("Evaluation failed: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Evaluate and return as JsValue for direct JavaScript object access
    /// 
    /// @param data - JSON data string
    /// @param context - Optional context data JSON string
    /// @returns Evaluated schema as JavaScript object
    #[wasm_bindgen(js_name = evaluateJS)]
    pub fn evaluate_js(&mut self, data: &str, context: Option<String>, paths: Option<Vec<String>>) -> Result<JsValue, JsValue> {
        let ctx = context.as_deref();
        let paths_refs: Option<Vec<String>> = paths;
        
        match self.inner.evaluate(data, ctx, paths_refs.as_deref()) {
            Ok(_) => {
                let result = self.inner.get_evaluated_schema(false);
                super::to_value(&result)
                    .map_err(|e| {
                        let error_msg = format!("Failed to convert evaluation result to JsValue: {}", e);
                        console_log(&format!("[WASM ERROR] {}", error_msg));
                        JsValue::from_str(&error_msg)
                    })
            },
            Err(e) => {
                let error_msg = format!("Evaluation failed: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Evaluate dependents when a field changes (returns array of changes as JSON string)
    /// 
    /// @param changedPath - Path of the field that changed
    /// @param data - Optional updated JSON data string
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
        
        // Wrap single path in a Vec for the new API
        let paths = vec![changed_path.to_string()];
        
        match self.inner.evaluate_dependents(&paths, data_str, ctx, false) {
            Ok(result) => serde_json::to_string(&result)
                .map_err(|e| {
                    let error_msg = format!("Failed to serialize dependents: {}", e);
                    console_log(&format!("[WASM ERROR] {}", error_msg));
                    JsValue::from_str(&error_msg)
                }),
            Err(e) => {
                let error_msg = format!("Failed to evaluate dependents: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Evaluate dependents and return as JavaScript object
    /// 
    /// @param changedPathsJson - JSON array of field paths that changed
    /// @param data - Optional updated JSON data string
    /// @param context - Optional context data JSON string
    /// @param reEvaluate - If true, performs full evaluation after processing dependents
    /// @returns Array of dependent change objects as JavaScript object
    #[wasm_bindgen(js_name = evaluateDependentsJS)]
    pub fn evaluate_dependents_js(
        &mut self,
        changed_paths_json: &str,
        data: Option<String>,
        context: Option<String>,
        re_evaluate: bool,
    ) -> Result<JsValue, JsValue> {
        // Parse JSON array of paths
        let paths: Vec<String> = serde_json::from_str(changed_paths_json)
            .map_err(|e| {
                let error_msg = format!("Failed to parse paths JSON: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                JsValue::from_str(&error_msg)
            })?;
        
        let data_str = data.as_deref();
        let ctx = context.as_deref();
        
        match self.inner.evaluate_dependents(&paths, data_str, ctx, re_evaluate) {
            Ok(result) => serde_wasm_bindgen::to_value(&result)
                .map_err(|e| {
                    let error_msg = format!("Failed to serialize dependents: {}", e);
                    console_log(&format!("[WASM ERROR] {}", error_msg));
                    JsValue::from_str(&error_msg)
                }),
            Err(e) => {
                let error_msg = format!("Failed to evaluate dependents: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Compile and run JSON logic from a JSON logic string
    /// @param logic_str - JSON logic expression as a string
    /// @param data - Optional JSON data string
    /// @param context - Optional JSON context string
    /// @returns Result as JavaScript object
    #[wasm_bindgen(js_name = compileAndRunLogic)]
    pub fn compile_and_run_logic(&mut self, logic_str: &str, data: Option<String>, context: Option<String>) -> Result<JsValue, JsValue> {
        let data_str = data.as_deref();
        let context_str = context.as_deref();
        
        match self.inner.compile_and_run_logic(logic_str, data_str, context_str) {
            Ok(result) => serde_wasm_bindgen::to_value(&result)
                .map_err(|e| {
                    let error_msg = format!("Failed to convert logic result: {}", e);
                    JsValue::from_str(&error_msg)
                }),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Compile JSON logic and return a global ID
    /// @param logic_str - JSON logic expression as a string
    /// @returns Compiled logic ID as number (u64)
    #[wasm_bindgen(js_name = compileLogic)]
    pub fn compile_logic(&self, logic_str: &str) -> Result<f64, JsValue> {
        match self.inner.compile_logic(logic_str) {
            Ok(id) => Ok(id.as_u64() as f64), // JavaScript number
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Run pre-compiled logic by ID
    /// @param logic_id - Compiled logic ID from compileLogic
    /// @param data - Optional JSON data string
    /// @param context - Optional JSON context string
    /// @returns Result as JavaScript object
    #[wasm_bindgen(js_name = runLogic)]
    pub fn run_logic(&mut self, logic_id: f64, data: Option<String>, context: Option<String>) -> Result<JsValue, JsValue> {
        let id = crate::CompiledLogicId::from_u64(logic_id as u64);
        
        let data_value = if let Some(data_str) = data {
            match serde_json::from_str(&data_str) {
                Ok(v) => Some(v),
                Err(e) => return Err(JsValue::from_str(&format!("Failed to parse data: {}", e))),
            }
        } else {
            None
        };
        
        let context_value = if let Some(ctx_str) = context {
            match serde_json::from_str(&ctx_str) {
                Ok(v) => Some(v),
                Err(e) => return Err(JsValue::from_str(&format!("Failed to parse context: {}", e))),
            }
        } else {
            None
        };
        
        match self.inner.run_logic(id, data_value.as_ref(), context_value.as_ref()) {
            Ok(result) => serde_wasm_bindgen::to_value(&result)
                .map_err(|e| {
                    let error_msg = format!("Failed to convert logic result: {}", e);
                    JsValue::from_str(&error_msg)
                }),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }
}
