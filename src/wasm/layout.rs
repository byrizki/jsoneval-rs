//! WASM layout resolution functions

use super::types::JSONEvalWasm;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl JSONEvalWasm {
    /// Resolve layout with optional evaluation.
    /// Returns overlay entries (LayoutOverlayEntry array) as JavaScript object.
    ///
    /// @param evaluate - If true, runs evaluation before resolving layout
    /// @returns LayoutOverlayEntry array as JavaScript object
    #[wasm_bindgen(js_name = resolveLayout)]
    pub fn resolve_layout(&mut self, evaluate: bool) -> Result<JsValue, JsValue> {
        let result = self.inner.resolve_layout(evaluate);
        super::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
