//! WASM core functions: constructors and utilities

use wasm_bindgen::prelude::*;
use crate::JSONEval;
use super::types::JSONEvalWasm;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Get the library version
#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get library version (alias)
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Initialize the library (sets up panic hook)
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
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

    /// Create a new JSONEval instance from a cached ParsedSchema
    /// 
    /// @param cacheKey - Cache key to lookup in the global ParsedSchemaCache
    /// @param context - Optional context data JSON string
    /// @param data - Optional initial data JSON string
    #[wasm_bindgen(js_name = newFromCache)]
    pub fn new_from_cache(cache_key: &str, context: Option<String>, data: Option<String>) -> Result<JSONEvalWasm, JsValue> {
        console_error_panic_hook::set_once();
        
        let ctx = context.as_deref();
        let dt = data.as_deref();
        
        // Get the cached ParsedSchema
        let parsed = crate::PARSED_SCHEMA_CACHE.get(cache_key)
            .ok_or_else(|| {
                let error_msg = format!("Schema '{}' not found in cache", cache_key);
                log(&format!("[WASM ERROR] {}", error_msg));
                JsValue::from_str(&error_msg)
            })?;
        
        // Create JSONEval from the cached ParsedSchema
        match JSONEval::with_parsed_schema(parsed, ctx, dt) {
            Ok(eval) => Ok(JSONEvalWasm { inner: eval }),
            Err(e) => {
                let error_msg = format!("Failed to create JSONEval from cache: {}", e);
                log(&format!("[WASM ERROR] {}", error_msg));
                Err(JsValue::from_str(&error_msg))
            }
        }
    }
}

// Make log available for other modules
pub(super) fn console_log(msg: &str) {
    log(msg);
}
