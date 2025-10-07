use serde_json::Value;
use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};
use rustc_hash::FxHasher;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::compiled::LogicId;

/// Cache key combining logic ID, data version, and dependency fingerprint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey {
    logic_id: LogicId,
    data_version: u64,
    dependency_hash: u64,
}

impl CacheKey {
    /// Create a cache key from logic ID and dependency versions
    #[inline]
    pub fn from_dependencies(logic_id: LogicId, data_version: u64, dependency_versions: &[u64]) -> Self {
        let mut hasher = FxHasher::default();
        logic_id.hash(&mut hasher);
        data_version.hash(&mut hasher);
        for version in dependency_versions {
            version.hash(&mut hasher);
        }

        Self {
            logic_id,
            data_version,
            dependency_hash: hasher.finish(),
        }
    }

    pub fn logic_id(&self) -> LogicId {
        self.logic_id
    }
}

#[derive(Clone)]
struct CacheNode {
    value: Arc<Value>,
    prev: Option<CacheKey>,
    next: Option<CacheKey>,
}

/// High-performance evaluation result cache with optional LRU eviction
pub struct EvalCache {
    entries: FxHashMap<CacheKey, CacheNode>,
    head: Option<CacheKey>,
    tail: Option<CacheKey>,
    capacity: Option<usize>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl EvalCache {
    pub fn new() -> Self {
        Self::with_capacity(None)
    }

    pub fn with_capacity(capacity: Option<usize>) -> Self {
        Self {
            entries: FxHashMap::default(),
            head: None,
            tail: None,
            capacity,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    pub fn capacity(&self) -> Option<usize> {
        self.capacity
    }
    
    /// Get a cached result if available
    pub fn get(&mut self, key: &CacheKey) -> Option<Arc<Value>> {
        if let Some(node) = self.entries.get(key) {
            let hit_count = self.hits.fetch_add(1, Ordering::Relaxed);
            let value = Arc::clone(&node.value);
            // Throttle LRU updates: only update every 16th hit to reduce HashMap overhead
            if hit_count & 15 == 0 {
                self.move_to_tail(*key);
            }
            Some(value)
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }
    
    /// Store a result in the cache
    pub fn insert(&mut self, key: CacheKey, value: Value) {
        let arc = Arc::new(value);
        if let Some(node) = self.entries.get_mut(&key) {
            node.value = Arc::clone(&arc);
            self.move_to_tail(key);
            return;
        }

        self.entries.insert(
            key,
            CacheNode {
                value: Arc::clone(&arc),
                prev: None,
                next: None,
            },
        );
        self.attach_to_tail(key);
        self.enforce_capacity();
    }
    
    /// Invalidate all cache entries for a specific logic ID
    pub fn invalidate_logic(&mut self, logic_id: &LogicId) {
        let keys: Vec<_> = self
            .entries
            .keys()
            .filter(|k| &k.logic_id() == logic_id)
            .copied()
            .collect();
        for key in keys {
            self.remove_node(key);
        }
    }
    
    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.head = None;
        self.tail = None;
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };
        
        CacheStats {
            hits,
            misses,
            hit_rate,
            size: self.entries.len(),
        }
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }

    fn move_to_tail(&mut self, key: CacheKey) {
        if self.tail == Some(key) {
            return;
        }
        self.detach(key);
        self.attach_to_tail(key);
    }

    fn attach_to_tail(&mut self, key: CacheKey) {
        match self.tail {
            Some(tail_key) => {
                if let Some(tail_node) = self.entries.get_mut(&tail_key) {
                    tail_node.next = Some(key);
                }
                if let Some(node) = self.entries.get_mut(&key) {
                    node.prev = Some(tail_key);
                    node.next = None;
                }
            }
            None => {
                if let Some(node) = self.entries.get_mut(&key) {
                    node.prev = None;
                    node.next = None;
                }
                self.head = Some(key);
            }
        }
        self.tail = Some(key);
        if self.head.is_none() {
            self.head = Some(key);
        }
    }

    fn detach(&mut self, key: CacheKey) {
        let links = match self.entries.get(&key) {
            Some(node) => (node.prev, node.next),
            None => return,
        };

        let (prev, next) = links;
        if let Some(prev_key) = prev {
            if let Some(prev_node) = self.entries.get_mut(&prev_key) {
                prev_node.next = next;
            }
        } else {
            self.head = next;
        }

        if let Some(next_key) = next {
            if let Some(next_node) = self.entries.get_mut(&next_key) {
                next_node.prev = prev;
            }
        } else {
            self.tail = prev;
        }

        if let Some(node) = self.entries.get_mut(&key) {
            node.prev = None;
            node.next = None;
        }

        if self.entries.is_empty() {
            self.head = None;
            self.tail = None;
        } else {
            if self.head.is_none() {
                self.head = Some(key);
            }
            if self.tail.is_none() {
                self.tail = Some(key);
            }
        }
    }

    fn remove_node(&mut self, key: CacheKey) {
        let links = match self.entries.get(&key) {
            Some(node) => (node.prev, node.next),
            None => return,
        };

        let (prev, next) = links;
        if let Some(prev_key) = prev {
            if let Some(prev_node) = self.entries.get_mut(&prev_key) {
                prev_node.next = next;
            }
        } else {
            self.head = next;
        }

        if let Some(next_key) = next {
            if let Some(next_node) = self.entries.get_mut(&next_key) {
                next_node.prev = prev;
            }
        } else {
            self.tail = prev;
        }

        self.entries.remove(&key);

        if self.entries.is_empty() {
            self.head = None;
            self.tail = None;
        }
    }

    fn enforce_capacity(&mut self) {
        if let Some(limit) = self.capacity {
            while self.entries.len() > limit {
                if let Some(oldest) = self.head {
                    self.remove_node(oldest);
                } else {
                    break;
                }
            }
        }
    }
}

impl Default for EvalCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,
}
