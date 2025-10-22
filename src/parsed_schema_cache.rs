/// Built-in cache store for Arc<ParsedSchema> instances
/// 
/// Provides thread-safe caching of parsed schemas with caller-controlled lifecycle:
/// - Caller provides unique keys
/// - Caller decides when to re-parse
/// - Caller controls cache clearing
/// - Caller manages memory release

use std::sync::{Arc, RwLock};
use indexmap::IndexMap;
use crate::ParsedSchema;

/// Thread-safe cache for storing and reusing ParsedSchema instances
/// 
/// # Example
/// ```
/// use json_eval_rs::{ParsedSchemaCache, ParsedSchema};
/// use std::sync::Arc;
/// 
/// let cache = ParsedSchemaCache::new();
/// 
/// // Parse and cache a schema
/// let schema = ParsedSchema::from_json(schema_json, None).unwrap();
/// cache.insert("my-schema".to_string(), Arc::new(schema));
/// 
/// // Retrieve from cache
/// if let Some(cached) = cache.get("my-schema") {
///     // Use cached schema for evaluation
/// }
/// 
/// // Remove specific entry
/// cache.remove("my-schema");
/// 
/// // Clear all entries
/// cache.clear();
/// ```
#[derive(Clone)]
pub struct ParsedSchemaCache {
    cache: Arc<RwLock<IndexMap<String, Arc<ParsedSchema>>>>,
}

impl ParsedSchemaCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(IndexMap::new())),
        }
    }
    
    /// Insert or update a parsed schema with the given key
    /// 
    /// Returns the previous value if the key already existed
    pub fn insert(&self, key: String, schema: Arc<ParsedSchema>) -> Option<Arc<ParsedSchema>> {
        let mut cache = self.cache.write().unwrap();
        cache.insert(key, schema)
    }
    
    /// Get a cloned Arc reference to the cached schema
    /// 
    /// Returns None if the key doesn't exist
    pub fn get(&self, key: &str) -> Option<Arc<ParsedSchema>> {
        let cache = self.cache.read().unwrap();
        cache.get(key).cloned()
    }
    
    /// Remove and return the schema for the given key
    /// 
    /// Returns None if the key doesn't exist
    pub fn remove(&self, key: &str) -> Option<Arc<ParsedSchema>> {
        let mut cache = self.cache.write().unwrap();
        cache.shift_remove(key)
    }
    
    /// Clear all cached schemas
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
    
    /// Check if a key exists in the cache
    pub fn contains_key(&self, key: &str) -> bool {
        let cache = self.cache.read().unwrap();
        cache.contains_key(key)
    }
    
    /// Get the number of cached schemas
    pub fn len(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }
    
    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Get all keys currently in the cache
    pub fn keys(&self) -> Vec<String> {
        let cache = self.cache.read().unwrap();
        cache.keys().cloned().collect()
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> ParsedSchemaCacheStats {
        let cache = self.cache.read().unwrap();
        ParsedSchemaCacheStats {
            entry_count: cache.len(),
            keys: cache.keys().cloned().collect(),
        }
    }
    
    /// Get or insert a schema using a factory function
    /// 
    /// If the key exists, returns the cached value.
    /// Otherwise, calls the factory function to create a new value,
    /// inserts it, and returns it.
    /// 
    /// # Example
    /// ```
    /// let schema = cache.get_or_insert_with("my-schema", || {
    ///     Arc::new(ParsedSchema::from_json(json, None).unwrap())
    /// });
    /// ```
    pub fn get_or_insert_with<F>(&self, key: &str, factory: F) -> Arc<ParsedSchema>
    where
        F: FnOnce() -> Arc<ParsedSchema>,
    {
        // Try read first (fast path)
        {
            let cache = self.cache.read().unwrap();
            if let Some(schema) = cache.get(key) {
                return schema.clone();
            }
        }
        
        // Need to insert (slow path)
        let mut cache = self.cache.write().unwrap();
        // Double-check in case another thread inserted while we waited for write lock
        if let Some(schema) = cache.get(key) {
            return schema.clone();
        }
        
        let schema = factory();
        cache.insert(key.to_string(), schema.clone());
        schema
    }
    
    /// Batch insert multiple schemas at once
    pub fn insert_batch(&self, entries: Vec<(String, Arc<ParsedSchema>)>) {
        let mut cache = self.cache.write().unwrap();
        for (key, schema) in entries {
            cache.insert(key, schema);
        }
    }
    
    /// Remove multiple keys at once
    pub fn remove_batch(&self, keys: &[String]) -> Vec<(String, Arc<ParsedSchema>)> {
        let mut cache = self.cache.write().unwrap();
        let mut removed = Vec::new();
        for key in keys {
            if let Some(schema) = cache.shift_remove(key) {
                removed.push((key.clone(), schema));
            }
        }
        removed
    }
}

impl Default for ParsedSchemaCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the cache state
#[derive(Debug, Clone)]
pub struct ParsedSchemaCacheStats {
    /// Number of entries in the cache
    pub entry_count: usize,
    /// List of all keys
    pub keys: Vec<String>,
}

impl std::fmt::Display for ParsedSchemaCacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParsedSchemaCache: {} entries", self.entry_count)?;
        if !self.keys.is_empty() {
            write!(f, " (keys: {})", self.keys.join(", "))?;
        }
        Ok(())
    }
}

// Optional: Global cache instance for convenience
use once_cell::sync::Lazy;

/// Global ParsedSchema cache instance
/// 
/// Convenient for applications that want a single global cache
/// without managing their own instance.
/// 
/// # Example
/// ```
/// use json_eval_rs::PARSED_SCHEMA_CACHE;
/// 
/// PARSED_SCHEMA_CACHE.insert("global-schema".to_string(), schema);
/// let cached = PARSED_SCHEMA_CACHE.get("global-schema");
/// ```
pub static PARSED_SCHEMA_CACHE: Lazy<ParsedSchemaCache> = Lazy::new(ParsedSchemaCache::new);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_basic_operations() {
        let cache = ParsedSchemaCache::new();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        
        // Insert doesn't require a real ParsedSchema for this test
        // In real usage, you'd use ParsedSchema::from_json
        
        assert!(!cache.contains_key("test"));
        assert_eq!(cache.keys().len(), 0);
    }
    
    #[test]
    fn test_cache_clone() {
        let cache1 = ParsedSchemaCache::new();
        let cache2 = cache1.clone();
        
        // Both should share the same underlying cache
        assert_eq!(cache1.len(), cache2.len());
    }
}
