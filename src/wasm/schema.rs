//! WASM schema getter functions

use super::core::console_log;
use super::types::JSONEvalWasm;
use wasm_bindgen::prelude::*;

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
        super::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
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
        self.inner
            .get_evaluated_schema_msgpack(skip_layout)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get all schema values (evaluations ending with .value)
    /// Mutates internal data by overriding with values from value evaluations
    ///
    /// @returns Modified data as JavaScript object
    #[wasm_bindgen(js_name = getSchemaValue)]
    pub fn get_schema_value(&mut self) -> Result<JsValue, JsValue> {
        let result = self.inner.get_schema_value();
        super::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
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
    pub fn get_evaluated_schema_without_params_js(
        &mut self,
        skip_layout: bool,
    ) -> Result<JsValue, JsValue> {
        let result = self.inner.get_evaluated_schema_without_params(skip_layout);
        super::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get a value from the evaluated schema using dotted path notation
    ///
    /// @param path - Dotted path to the value (e.g., "properties.field.value")
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Value as JSON string or null if not found
    #[wasm_bindgen(js_name = getEvaluatedSchemaByPath)]
    pub fn get_evaluated_schema_by_path(
        &mut self,
        path: &str,
        skip_layout: bool,
    ) -> Option<String> {
        self.inner
            .get_evaluated_schema_by_path(path, skip_layout)
            .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string()))
    }

    /// Get a value from the evaluated schema using dotted path notation as JavaScript object
    ///
    /// @param path - Dotted path to the value (e.g., "properties.field.value")
    /// @param skipLayout - Whether to skip layout resolution
    /// @returns Value as JavaScript object or null if not found
    #[wasm_bindgen(js_name = getEvaluatedSchemaByPathJS)]
    pub fn get_evaluated_schema_by_path_js(
        &mut self,
        path: &str,
        skip_layout: bool,
    ) -> Result<JsValue, JsValue> {
        match self.inner.get_evaluated_schema_by_path(path, skip_layout) {
            Some(value) => super::to_value(&value).map_err(|e| JsValue::from_str(&e.to_string())),
            None => Ok(JsValue::NULL),
        }
    }

    /// Get values from evaluated schema using multiple dotted paths
    /// @param pathsJson - JSON array of dotted paths
    /// @param skipLayout - Whether to skip layout resolution
    /// @param format - Return format (0=Nested, 1=Flat, 2=Array)
    /// @returns Data in specified format as JSON string
    #[wasm_bindgen(js_name = getEvaluatedSchemaByPaths)]
    pub fn get_evaluated_schema_by_paths(
        &mut self,
        paths_json: &str,
        skip_layout: bool,
        format: u8,
    ) -> Result<String, JsValue> {
        // Parse JSON array of paths
        let paths: Vec<String> = serde_json::from_str(paths_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse paths JSON: {}", e)))?;

        let return_format = match format {
            1 => crate::ReturnFormat::Flat,
            2 => crate::ReturnFormat::Array,
            _ => crate::ReturnFormat::Nested,
        };

        let result =
            self.inner
                .get_evaluated_schema_by_paths(&paths, skip_layout, Some(return_format));
        serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get values from evaluated schema using multiple dotted paths (JS object)
    /// @param pathsJson - JSON array of dotted paths
    /// @param skipLayout - Whether to skip layout resolution
    /// @param format - Return format (0=Nested, 1=Flat, 2=Array)
    /// @returns Data in specified format as JavaScript object
    #[wasm_bindgen(js_name = getEvaluatedSchemaByPathsJS)]
    pub fn get_evaluated_schema_by_paths_js(
        &mut self,
        paths_json: &str,
        skip_layout: bool,
        format: u8,
    ) -> Result<JsValue, JsValue> {
        // Parse JSON array of paths
        let paths: Vec<String> = serde_json::from_str(paths_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse paths JSON: {}", e)))?;

        let return_format = match format {
            1 => crate::ReturnFormat::Flat,
            2 => crate::ReturnFormat::Array,
            _ => crate::ReturnFormat::Nested,
        };

        let result =
            self.inner
                .get_evaluated_schema_by_paths(&paths, skip_layout, Some(return_format));
        super::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get a value from the schema using dotted path notation
    ///
    /// @param path - Dotted path to the value (e.g., "properties.field.value")
    /// @returns Value as JSON string or null if not found
    #[wasm_bindgen(js_name = getSchemaByPath)]
    pub fn get_schema_by_path(&self, path: &str) -> Option<String> {
        self.inner
            .get_schema_by_path(path)
            .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string()))
    }

    /// Get a value from the schema using dotted path notation as JavaScript object
    ///
    /// @param path - Dotted path to the value (e.g., "properties.field.value")
    /// @returns Value as JavaScript object or null if not found
    #[wasm_bindgen(js_name = getSchemaByPathJS)]
    pub fn get_schema_by_path_js(&self, path: &str) -> Result<JsValue, JsValue> {
        match self.inner.get_schema_by_path(path) {
            Some(value) => super::to_value(&value).map_err(|e| JsValue::from_str(&e.to_string())),
            None => Ok(JsValue::NULL),
        }
    }

    /// Get values from schema using multiple dotted paths
    /// @param pathsJson - JSON array of dotted paths
    /// @param format - Return format (0=Nested, 1=Flat, 2=Array)
    /// @returns Data in specified format as JSON string
    #[wasm_bindgen(js_name = getSchemaByPaths)]
    pub fn get_schema_by_paths(&self, paths_json: &str, format: u8) -> Result<String, JsValue> {
        // Parse JSON array of paths
        let paths: Vec<String> = serde_json::from_str(paths_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse paths JSON: {}", e)))?;

        let return_format = match format {
            1 => crate::ReturnFormat::Flat,
            2 => crate::ReturnFormat::Array,
            _ => crate::ReturnFormat::Nested,
        };

        let result = self.inner.get_schema_by_paths(&paths, Some(return_format));
        serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get values from schema using multiple dotted paths (JS object)
    /// @param pathsJson - JSON array of dotted paths
    /// @param format - Return format (0=Nested, 1=Flat, 2=Array)
    /// @returns Data in specified format as JavaScript object
    #[wasm_bindgen(js_name = getSchemaByPathsJS)]
    pub fn get_schema_by_paths_js(&self, paths_json: &str, format: u8) -> Result<JsValue, JsValue> {
        // Parse JSON array of paths
        let paths: Vec<String> = serde_json::from_str(paths_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse paths JSON: {}", e)))?;

        let return_format = match format {
            1 => crate::ReturnFormat::Flat,
            2 => crate::ReturnFormat::Array,
            _ => crate::ReturnFormat::Nested,
        };

        let result = self.inner.get_schema_by_paths(&paths, Some(return_format));
        super::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Reload schema with new data
    ///
    /// @param schema - New JSON schema string
    /// @param context - Optional context data JSON string
    /// @param data - Optional initial data JSON string
    #[wasm_bindgen(js_name = reloadSchema)]
    pub fn reload_schema(
        &mut self,
        schema: &str,
        context: Option<String>,
        data: Option<String>,
    ) -> Result<(), JsValue> {
        let ctx = context.as_deref();
        let dt = data.as_deref();

        self.inner.reload_schema(schema, ctx, dt).map_err(|e| {
            let error_msg = format!("Failed to reload schema: {}", e);
            console_log(&format!("[WASM ERROR] {}", error_msg));
            JsValue::from_str(&error_msg)
        })
    }

    /// Reload schema from MessagePack-encoded bytes
    ///
    /// @param schemaMsgpack - MessagePack-encoded schema bytes (Uint8Array)
    /// @param context - Optional context data JSON string
    /// @param data - Optional initial data JSON string
    #[wasm_bindgen(js_name = reloadSchemaMsgpack)]
    pub fn reload_schema_msgpack(
        &mut self,
        schema_msgpack: &[u8],
        context: Option<String>,
        data: Option<String>,
    ) -> Result<(), JsValue> {
        let ctx = context.as_deref();
        let dt = data.as_deref();

        self.inner
            .reload_schema_msgpack(schema_msgpack, ctx, dt)
            .map_err(|e| {
                let error_msg = format!("Failed to reload schema from MessagePack: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                JsValue::from_str(&error_msg)
            })
    }

    /// Reload schema from ParsedSchemaCache using a cache key
    ///
    /// @param cacheKey - Cache key to lookup in the global ParsedSchemaCache
    /// @param context - Optional context data JSON string
    /// @param data - Optional initial data JSON string
    #[wasm_bindgen(js_name = reloadSchemaFromCache)]
    pub fn reload_schema_from_cache(
        &mut self,
        cache_key: &str,
        context: Option<String>,
        data: Option<String>,
    ) -> Result<(), JsValue> {
        let ctx = context.as_deref();
        let dt = data.as_deref();

        self.inner
            .reload_schema_from_cache(cache_key, ctx, dt)
            .map_err(|e| {
                let error_msg = format!("Failed to reload schema from cache: {}", e);
                console_log(&format!("[WASM ERROR] {}", error_msg));
                JsValue::from_str(&error_msg)
            })
    }
}
