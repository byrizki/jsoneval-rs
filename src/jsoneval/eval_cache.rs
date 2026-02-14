#[cfg(feature = "parallel")]
use dashmap::DashMap;
#[cfg(not(feature = "parallel"))]
use std::cell::RefCell;
#[cfg(not(feature = "parallel"))]
use std::collections::HashMap;

use ahash::AHasher;
use indexmap::IndexSet;
use serde_json::Value;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;



/// Hash a serde_json::Value directly without intermediate string allocation
#[inline]
fn hash_value(value: &Value, hasher: &mut AHasher) {
    match value {
        Value::Null => 0u8.hash(hasher),
        Value::Bool(b) => {
            1u8.hash(hasher);
            b.hash(hasher);
        }
        Value::Number(n) => {
            2u8.hash(hasher);
            n.as_f64().unwrap_or(0.0).to_bits().hash(hasher);
        }
        Value::String(s) => {
            3u8.hash(hasher);
            s.hash(hasher);
        }
        Value::Array(arr) => {
            4u8.hash(hasher);
            arr.len().hash(hasher);
            for v in arr {
                hash_value(v, hasher);
            }
        }
        Value::Object(map) => {
            5u8.hash(hasher);
            map.len().hash(hasher);
            for (k, v) in map {
                k.hash(hasher);
                hash_value(v, hasher);
            }
        }
    }
}


/// Cache key: combines evaluation logic ID with hash of all dependent values
/// Zero-copy design: stores references to logic and dependency paths
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Evaluation key (e.g., "$params.foo")
    pub eval_key: String,
    /// Single hash of all dependency values combined (for efficiency)
    pub deps_hash: u64,
}

impl CacheKey {
    /// Create cache key from evaluation key and dependency values
    /// Dependencies are pre-filtered by caller (JSONEval)
    /// Hashes all dependency values together in one pass for efficiency
    pub fn new(
        eval_key: String,
        dependencies: &IndexSet<String>,
        values: &[(String, &Value)],
    ) -> Self {
        let value_map: std::collections::HashMap<&str, &Value> =
            values.iter().map(|(k, v)| (k.as_str(), *v)).collect();

        let mut hasher = AHasher::default();
        for dep_key in dependencies.iter() {
            dep_key.hash(&mut hasher);
            if let Some(value) = value_map.get(dep_key.as_str()) {
                hash_value(value, &mut hasher);
            } else {
                0u8.hash(&mut hasher);
            }
        }

        let deps_hash = hasher.finish();

        Self {
            eval_key,
            deps_hash,
        }
    }


    /// Create a simple cache key without dependencies (for evaluations with no deps)
    pub fn simple(eval_key: String) -> Self {
        Self {
            eval_key,
            deps_hash: 0, // No dependencies = hash of 0
        }
    }
}

/// Zero-copy cache store
/// With parallel feature: Uses DashMap for thread-safe concurrent access
/// Without parallel feature: Uses HashMap + RefCell for ultra-fast single-threaded access
/// Values are stored behind Arc to enable cheap cloning
pub struct EvalCache {
    #[cfg(feature = "parallel")]
    /// Cache storage: DashMap for thread-safe concurrent access
    cache: DashMap<CacheKey, Arc<Value>>,

    #[cfg(not(feature = "parallel"))]
    /// Cache storage: HashMap + RefCell for ultra-fast single-threaded access
    cache: RefCell<HashMap<CacheKey, Arc<Value>>>,

    /// Cache statistics (atomic for thread safety)
    hits: AtomicUsize,
    misses: AtomicUsize,
}

impl EvalCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "parallel")]
            cache: DashMap::new(),
            #[cfg(not(feature = "parallel"))]
            cache: RefCell::new(HashMap::new()),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    /// Create cache with preallocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            #[cfg(feature = "parallel")]
            cache: DashMap::with_capacity(capacity),
            #[cfg(not(feature = "parallel"))]
            cache: RefCell::new(HashMap::with_capacity(capacity)),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    /// Get cached result (zero-copy via Arc clone)
    /// Returns None if not cached
    #[cfg(feature = "parallel")]
    /// Thread-safe: can be called concurrently
    #[inline]
    pub fn get(&self, key: &CacheKey) -> Option<Arc<Value>> {
        if let Some(value) = self.cache.get(key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            Some(Arc::clone(value.value()))
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Get cached result (zero-copy via Arc clone)
    /// Returns None if not cached
    #[cfg(not(feature = "parallel"))]
    /// Ultra-fast single-threaded access
    #[inline]
    pub fn get(&self, key: &CacheKey) -> Option<Arc<Value>> {
        if let Some(value) = self.cache.borrow().get(key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            Some(Arc::clone(value))
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Insert result into cache (wraps in Arc for zero-copy sharing)
    #[cfg(feature = "parallel")]
    /// Thread-safe: can be called concurrently
    #[inline]
    pub fn insert(&self, key: CacheKey, value: Value) {
        self.cache.insert(key, Arc::new(value));
    }

    /// Insert result into cache (wraps in Arc for zero-copy sharing)
    #[cfg(not(feature = "parallel"))]
    /// Ultra-fast single-threaded access
    #[inline]
    pub fn insert(&self, key: CacheKey, value: Value) {
        self.cache.borrow_mut().insert(key, Arc::new(value));
    }

    /// Insert with Arc-wrapped value (zero-copy if already Arc)
    #[cfg(feature = "parallel")]
    /// Thread-safe: can be called concurrently
    #[inline]
    pub fn insert_arc(&self, key: CacheKey, value: Arc<Value>) {
        self.cache.insert(key, value);
    }

    /// Insert with Arc-wrapped value (zero-copy if already Arc)
    #[cfg(not(feature = "parallel"))]
    /// Ultra-fast single-threaded access
    #[inline]
    pub fn insert_arc(&self, key: CacheKey, value: Arc<Value>) {
        self.cache.borrow_mut().insert(key, value);
    }

    /// Clear all cached entries
    #[cfg(feature = "parallel")]
    pub fn clear(&self) {
        self.cache.clear();
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }

    /// Clear all cached entries
    #[cfg(not(feature = "parallel"))]
    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get cache statistics
    #[cfg(feature = "parallel")]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            entries: self.cache.len(),
            hit_rate: self.hit_rate(),
        }
    }

    /// Get cache statistics
    #[cfg(not(feature = "parallel"))]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            entries: self.cache.borrow().len(),
            hit_rate: self.hit_rate(),
        }
    }

    /// Get number of cached entries
    #[cfg(feature = "parallel")]
    #[inline]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Get number of cached entries
    #[cfg(not(feature = "parallel"))]
    #[inline]
    pub fn len(&self) -> usize {
        self.cache.borrow().len()
    }

    /// Check if cache is empty
    #[cfg(feature = "parallel")]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Check if cache is empty
    #[cfg(not(feature = "parallel"))]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cache.borrow().is_empty()
    }

    /// Remove specific entry
    #[cfg(feature = "parallel")]
    #[inline]
    pub fn remove(&self, key: &CacheKey) -> Option<Arc<Value>> {
        self.cache.remove(key).map(|(_, v)| v)
    }

    /// Remove specific entry
    #[cfg(not(feature = "parallel"))]
    #[inline]
    pub fn remove(&self, key: &CacheKey) -> Option<Arc<Value>> {
        self.cache.borrow_mut().remove(key)
    }

    /// Remove entries based on a predicate function
    /// Predicate returns true to keep the entry, false to remove it
    #[cfg(feature = "parallel")]
    pub fn retain<F>(&self, predicate: F)
    where
        F: Fn(&CacheKey, &Arc<Value>) -> bool,
    {
        self.cache.retain(|k, v| predicate(k, v));
    }

    /// Remove entries based on a predicate function
    /// Predicate returns true to keep the entry, false to remove it
    #[cfg(not(feature = "parallel"))]
    pub fn retain<F>(&self, predicate: F)
    where
        F: Fn(&CacheKey, &Arc<Value>) -> bool,
    {
        self.cache.borrow_mut().retain(|k, v| predicate(k, v));
    }

    /// Invalidate cache entries that depend on changed paths
    /// Efficiently removes only affected entries
    #[cfg(feature = "parallel")]
    pub fn invalidate_dependencies(&self, changed_paths: &[String]) {
        // Build a set of changed path hashes for fast lookup
        let changed_hashes: IndexSet<String> = changed_paths.iter().cloned().collect();

        // Remove cache entries whose eval_key is in the changed set
        self.cache
            .retain(|cache_key, _| !changed_hashes.contains(&cache_key.eval_key));
    }

    /// Invalidate cache entries that depend on changed paths
    /// Efficiently removes only affected entries
    #[cfg(not(feature = "parallel"))]
    pub fn invalidate_dependencies(&self, changed_paths: &[String]) {
        // Build a set of changed path hashes for fast lookup
        let changed_hashes: IndexSet<String> = changed_paths.iter().cloned().collect();

        // Remove cache entries whose eval_key is in the changed set
        self.cache
            .borrow_mut()
            .retain(|cache_key, _| !changed_hashes.contains(&cache_key.eval_key));
    }
}

impl Default for EvalCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub entries: usize,
    pub hit_rate: f64,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats: {} entries, {} hits, {} misses, {:.2}% hit rate",
            self.entries,
            self.hits,
            self.misses,
            self.hit_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cache_key_creation() {
        let eval_key = "$params.foo".to_string();
        let mut deps = IndexSet::new();
        deps.insert("$params.bar".to_string());
        deps.insert("data.value".to_string());

        let val1 = json!(42);
        let val2 = json!("test");
        let values = vec![
            ("$params.bar".to_string(), &val1),
            ("data.value".to_string(), &val2),
        ];

        let key1 = CacheKey::new(eval_key.clone(), &deps, &values);
        let key2 = CacheKey::new(eval_key.clone(), &deps, &values);

        // Same inputs should produce same key
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_values() {
        let eval_key = "$params.foo".to_string();
        let mut deps = IndexSet::new();
        deps.insert("data.value".to_string());

        let val1 = json!(42);
        let val2 = json!(43);
        let values1 = vec![("data.value".to_string(), &val1)];
        let values2 = vec![("data.value".to_string(), &val2)];

        let key1 = CacheKey::new(eval_key.clone(), &deps, &values1);
        let key2 = CacheKey::new(eval_key.clone(), &deps, &values2);

        // Different values should produce different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_operations() {
        let cache = EvalCache::new();

        let key = CacheKey::simple("test".to_string());
        let value = json!({"result": 42});

        // Test miss
        assert!(cache.get(&key).is_none());
        assert_eq!(cache.stats().misses, 1);

        // Insert and test hit
        cache.insert(key.clone(), value.clone());
        assert_eq!(cache.get(&key).unwrap().as_ref(), &value);
        assert_eq!(cache.stats().hits, 1);

        // Test stats
        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.hit_rate, 0.5); // 1 hit, 1 miss
    }

    #[test]
    fn test_cache_clear() {
        let cache = EvalCache::new();
        cache.insert(CacheKey::simple("test".to_string()), json!(42));

        assert_eq!(cache.len(), 1);
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.stats().hits, 0);
    }

    #[test]
    fn test_invalidate_dependencies() {
        let cache = EvalCache::new();

        // Add cache entries
        cache.insert(CacheKey::simple("$params.foo".to_string()), json!(1));
        cache.insert(CacheKey::simple("$params.bar".to_string()), json!(2));
        cache.insert(CacheKey::simple("$params.baz".to_string()), json!(3));

        assert_eq!(cache.len(), 3);

        // Invalidate one path
        cache.invalidate_dependencies(&["$params.foo".to_string()]);

        assert_eq!(cache.len(), 2);
        assert!(cache
            .get(&CacheKey::simple("$params.foo".to_string()))
            .is_none());
        assert!(cache
            .get(&CacheKey::simple("$params.bar".to_string()))
            .is_some());
    }
}
