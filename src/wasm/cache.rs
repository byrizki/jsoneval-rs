//! WASM cache management functions

use wasm_bindgen::prelude::*;
use super::types::JSONEvalWasm;

#[wasm_bindgen]
impl JSONEvalWasm {
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
}
