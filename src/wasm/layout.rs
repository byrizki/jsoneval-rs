//! WASM layout resolution functions

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
        self.inner.resolve_layout(evaluate);
        Ok(())
    }
}
