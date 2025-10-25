use dashmap::DashMap;
use serde_json::Value;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use ahash::AHasher;
use indexmap::IndexSet;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Fast hash computation for cache keys
/// Uses AHash for performance and FxHash-like quality
#[inline]
fn compute_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = AHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
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
    pub fn new(eval_key: String, dependencies: &IndexSet<String>, values: &[(String, &Value)]) -> Self {
        // Build hash map for fast lookup
        let value_map: std::collections::HashMap<&str, &Value> = values
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();

        // Combine all dependency values into a single string for hashing
        // This is more efficient than hashing each value separately
        let mut combined = String::new();
        for dep_key in dependencies.iter() {
            combined.push_str(dep_key);
            combined.push(':');
            if let Some(value) = value_map.get(dep_key.as_str()) {
                combined.push_str(&value.to_string());
            } else {
                combined.push_str("null");
            }
            combined.push(';');
        }
        
        // Compute single hash for all dependencies
        let deps_hash = compute_hash(&combined);

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

/// Zero-copy cache store using DashMap for thread-safe concurrent access
/// Values are stored behind Arc to enable cheap cloning
pub struct EvalCache {
    /// Cache storage: DashMap for thread-safe concurrent access
    /// Arc enables zero-copy reads across threads
    cache: DashMap<CacheKey, Arc<Value>>,
    /// Cache statistics (atomic for thread safety)
    hits: AtomicUsize,
    misses: AtomicUsize,
}

impl EvalCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    /// Create cache with preallocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: DashMap::with_capacity(capacity),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    /// Get cached result (zero-copy via Arc clone)
    /// Returns None if not cached
    /// Thread-safe: can be called concurrently
    #[inline(always)]
    pub fn get(&self, key: &CacheKey) -> Option<Arc<Value>> {
        if let Some(value) = self.cache.get(key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            Some(Arc::clone(value.value()))
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Insert result into cache (wraps in Arc for zero-copy sharing)
    /// Thread-safe: can be called concurrently
    #[inline(always)]
    pub fn insert(&self, key: CacheKey, value: Value) {
        self.cache.insert(key, Arc::new(value));
    }

    /// Insert with Arc-wrapped value (zero-copy if already Arc)
    /// Thread-safe: can be called concurrently
    #[inline(always)]
    pub fn insert_arc(&self, key: CacheKey, value: Arc<Value>) {
        self.cache.insert(key, value);
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        self.cache.clear();
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
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            entries: self.cache.len(),
            hit_rate: self.hit_rate(),
        }
    }

    /// Get number of cached entries
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Remove specific entry
    #[inline(always)]
    pub fn remove(&self, key: &CacheKey) -> Option<Arc<Value>> {
        self.cache.remove(key).map(|(_, v)| v)
    }
    
    /// Remove entries based on a predicate function
    /// Predicate returns true to keep the entry, false to remove it
    pub fn retain<F>(&self, predicate: F)
    where
        F: Fn(&CacheKey, &Arc<Value>) -> bool,
    {
        self.cache.retain(|k, v| predicate(k, v));
    }

    /// Invalidate cache entries that depend on changed paths
    /// Efficiently removes only affected entries
    pub fn invalidate_dependencies(&self, changed_paths: &[String]) {
        // Build a set of changed path hashes for fast lookup
        let changed_hashes: IndexSet<String> = changed_paths.iter().cloned().collect();

        // Remove cache entries whose eval_key is in the changed set
        self.cache.retain(|cache_key, _| {
            !changed_hashes.contains(&cache_key.eval_key)
        });
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
        assert!(cache.get(&CacheKey::simple("$params.foo".to_string())).is_none());
        assert!(cache.get(&CacheKey::simple("$params.bar".to_string())).is_some());
    }
}
