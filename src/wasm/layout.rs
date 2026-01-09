//! WASM layout resolution functions

use super::core::console_log;
use super::types::JSONEvalWasm;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl JSONEvalWasm {
    /// Resolve layout with optional evaluation
    ///
    /// @param evaluate - If true, runs evaluation before resolving layout
    /// @throws Error if resolve fails
    #[wasm_bindgen(js_name = resolveLayout)]
    pub fn resolve_layout(&mut self, evaluate: bool) -> Result<(), JsValue> {
        self.inner.resolve_layout(evaluate).map_err(|e| {
            let error_msg = format!("Failed to resolve layout: {}", e);
            console_log(&format!("[WASM ERROR] {}", error_msg));
            JsValue::from_str(&error_msg)
        })
    }
}
