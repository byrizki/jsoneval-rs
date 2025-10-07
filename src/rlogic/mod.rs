//! RLogic - High-performance JSON Logic compiler and evaluator with caching
//! 
//! This module provides a complete implementation of JSON Logic with:
//! - Pre-compilation of logic expressions for fast repeated evaluation
//! - Automatic result caching with invalidation on data mutation
//! - Mutation tracking via proxy-like data wrapper
//! - Zero external dependencies (uses only standard library and serde_json)

pub mod compiled;
pub mod evaluator;
pub mod cache;
pub mod data_wrapper;
pub mod custom_ops;
pub mod config;
pub mod path;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod custom_tests;

#[cfg(test)]
mod advanced_tests;

#[cfg(test)]
mod config_tests;

#[cfg(test)]
mod ref_tests;

use serde_json::Value;
use std::sync::Arc;

pub use compiled::{CompiledLogic, CompiledLogicStore, LogicId};
pub use evaluator::Evaluator;
pub use cache::{EvalCache, CacheKey, CacheStats};
pub use data_wrapper::{TrackedData, TrackedDataBuilder, DataVersion};
pub use config::RLogicConfig;

/// Main RLogic engine combining compilation, evaluation, and caching
pub struct RLogic {
    config: RLogicConfig,
    store: CompiledLogicStore,
    evaluator: Evaluator,
    cache: EvalCache,
}

impl RLogic {
    /// Create a new RLogic instance with default configuration
    pub fn new() -> Self {
        Self::with_config(RLogicConfig::default())
    }
    
    /// Create a new RLogic instance with custom configuration
    pub fn with_config(config: RLogicConfig) -> Self {
        Self {
            config,
            store: CompiledLogicStore::new(),
            evaluator: Evaluator::new().with_config(config),
            cache: EvalCache::new(),
        }
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &RLogicConfig {
        &self.config
    }
    
    /// Compile a JSON Logic expression
    pub fn compile(&mut self, logic: &Value) -> Result<LogicId, String> {
        self.store.compile(logic)
    }
    
    /// Evaluate a compiled logic expression against tracked data
    pub fn evaluate(&mut self, logic_id: &LogicId, data: &TrackedData) -> Result<Arc<Value>, String> {
        // Skip cache if disabled
        if !self.config.enable_cache {
            return self.evaluate_uncached(logic_id, data).map(Arc::new);
        }
        
        // Get dependencies for this logic
        let deps = self.store.get_dependencies(logic_id)
            .ok_or_else(|| "Logic ID not found".to_string())?;

        // Use tracked versions instead of whole values for cache key
        // Note: We use instance_id instead of global version since we track field-specific versions
        let dep_versions = data.dependency_versions(deps);
        let cache_key = CacheKey::from_dependencies(*logic_id, data.instance_id(), &dep_versions);
        
        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached);
        }
        
        // Cache miss - evaluate
        let logic = self.store.get(logic_id)
            .ok_or_else(|| "Logic ID not found".to_string())?;

        let result = self.evaluator.evaluate(logic, data.data())?;

        // Store in cache only if result is not a large array (avoid caching huge tables)
        let should_cache = match &result {
            Value::Array(arr) if arr.len() > 100 => false,  // Skip caching large arrays
            Value::Object(obj) if obj.len() > 50 => false,  // Skip caching large objects
            _ => true,
        };
        
        if should_cache {
            self.cache.insert(cache_key, result.clone());
        }
        
        Ok(Arc::new(result))
    }
    
    /// Evaluate without caching (useful when data changes frequently)
    pub fn evaluate_uncached(&self, logic_id: &LogicId, data: &TrackedData) -> Result<Value, String> {
        let logic = self.store.get(logic_id)
            .ok_or_else(|| "Logic ID not found".to_string())?;
        
        self.evaluator.evaluate(logic, data.data())
    }
    
    /// Evaluate without caching or wrapping (fastest for one-off evaluations)
    pub fn evaluate_raw(&self, logic_id: &LogicId, data: &Value) -> Result<Value, String> {
        let logic = self.store.get(logic_id)
            .ok_or_else(|| "Logic ID not found".to_string())?;
        
        self.evaluator.evaluate(logic, data)
    }
    
    /// Evaluate a logic expression directly (compile + evaluate)
    pub fn evaluate_direct(&mut self, logic: &Value, data: &Value) -> Result<Value, String> {
        let compiled = CompiledLogic::compile(logic)?;
        self.evaluator.evaluate(&compiled, data)
    }
    
    /// Invalidate cache for a specific logic ID
    pub fn invalidate_logic(&mut self, logic_id: &LogicId) {
        self.cache.invalidate_logic(logic_id);
    }
    
    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }
    
    /// Get the referenced variables in a compiled logic
    pub fn get_referenced_vars(&self, logic_id: &LogicId) -> Option<Vec<String>> {
        self.store.get(logic_id).map(|logic| logic.referenced_vars())
    }
    
    /// Check if a compiled logic has forward references (e.g., references future iterations)
    pub fn has_forward_reference(&self, logic_id: &LogicId) -> bool {
        self.store.get(logic_id)
            .map(|logic| logic.has_forward_reference())
            .unwrap_or(false)
    }
}

impl Default for RLogic {
    fn default() -> Self {
        Self::new()
    }
}
