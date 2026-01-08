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
        super::to_value(&stats_obj)
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

    /// Enable evaluation caching
    /// Useful for reusing JSONEval instances with different data
    #[wasm_bindgen(js_name = enableCache)]
    pub fn enable_cache(&mut self) {
        self.inner.enable_cache();
    }

    /// Disable evaluation caching
    /// Useful for web API usage where each request creates a new JSONEval instance
    /// Improves performance by skipping cache operations that have no benefit for single-use instances
    #[wasm_bindgen(js_name = disableCache)]
    pub fn disable_cache(&mut self) {
        self.inner.disable_cache();
    }

    /// Check if evaluation caching is enabled
    /// 
    /// @returns true if caching is enabled, false otherwise
    #[wasm_bindgen(js_name = isCacheEnabled)]
    pub fn is_cache_enabled(&self) -> bool {
        self.inner.is_cache_enabled()
    }
}
