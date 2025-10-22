//! WASM schema getter functions

use wasm_bindgen::prelude::*;
use super::types::JSONEvalWasm;
use super::core::console_log;

#[wasm_bindgen]
impl JSONEvalWasm {
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
                console_log(&format!("[WASM ERROR] {}", error_msg));
                JsValue::from_str(&error_msg)
            })
    }
}
