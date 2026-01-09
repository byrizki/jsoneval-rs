//! Global storage for compiled logic expressions
//!
//! This module provides a thread-safe global store for compiled logic that can be shared
//! across different JSONEval instances and across FFI boundaries.

use super::CompiledLogic;
use ahash::AHasher;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for a compiled logic expression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompiledLogicId(u64);

impl CompiledLogicId {
    /// Get the underlying u64 value
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Create from u64 value
    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }
}

/// Global storage for compiled logic expressions
static COMPILED_LOGIC_STORE: Lazy<CompiledLogicStore> = Lazy::new(|| {
    CompiledLogicStore {
        store: DashMap::new(),
        id_map: DashMap::new(),
        next_id: AtomicU64::new(1), // Start from 1, 0 reserved for invalid
    }
});

/// Thread-safe global store for compiled logic
struct CompiledLogicStore {
    /// Map from hash to (ID, CompiledLogic)
    store: DashMap<u64, (CompiledLogicId, CompiledLogic)>,
    /// Reverse map from ID to CompiledLogic for fast lookup
    id_map: DashMap<u64, CompiledLogic>,
    /// Next available ID
    next_id: AtomicU64,
}

impl CompiledLogicStore {
    /// Compile logic from a Value and return an ID
    /// If the same logic was compiled before, returns the existing ID
    fn compile_value(&self, logic: &serde_json::Value) -> Result<CompiledLogicId, String> {
        // Hash the logic value for deduplication
        let logic_str = serde_json::to_string(logic)
            .map_err(|e| format!("Failed to serialize logic: {}", e))?;
        let mut hasher = AHasher::default();
        logic_str.hash(&mut hasher);
        let hash = hasher.finish();

        // Check if already compiled
        if let Some(entry) = self.store.get(&hash) {
            return Ok(entry.0);
        }

        // Compile using the shared CompiledLogic::compile method
        let compiled = CompiledLogic::compile(logic)?;

        // Generate new ID
        let id = CompiledLogicId(self.next_id.fetch_add(1, Ordering::SeqCst));

        // Store in both maps
        self.store.insert(hash, (id, compiled.clone()));
        self.id_map.insert(id.0, compiled);

        Ok(id)
    }

    /// Compile logic from a JSON string and return an ID
    /// If the same logic was compiled before, returns the existing ID
    fn compile(&self, logic_json: &str) -> Result<CompiledLogicId, String> {
        // Parse JSON
        let logic: serde_json::Value = serde_json::from_str(logic_json)
            .map_err(|e| format!("Failed to parse logic JSON: {}", e))?;

        // Use shared compile_value method
        self.compile_value(&logic)
    }

    /// Get compiled logic by ID (O(1) lookup)
    fn get(&self, id: CompiledLogicId) -> Option<CompiledLogic> {
        self.id_map.get(&id.0).map(|v| v.clone())
    }

    /// Get statistics about the store
    fn stats(&self) -> CompiledLogicStoreStats {
        CompiledLogicStoreStats {
            compiled_count: self.store.len(),
            next_id: self.next_id.load(Ordering::SeqCst),
        }
    }

    /// Clear all compiled logic (useful for testing)
    #[allow(dead_code)]
    fn clear(&self) {
        self.store.clear();
        self.id_map.clear();
        self.next_id.store(1, Ordering::SeqCst);
    }
}

/// Statistics about the compiled logic store
#[derive(Debug, Clone)]
pub struct CompiledLogicStoreStats {
    /// Number of compiled logic expressions stored
    pub compiled_count: usize,
    /// Next ID that will be assigned
    pub next_id: u64,
}

/// Compile logic from a JSON string and return a unique ID
///
/// The compiled logic is stored in a global thread-safe cache.
/// If the same logic was compiled before, returns the existing ID.
pub fn compile_logic(logic_json: &str) -> Result<CompiledLogicId, String> {
    COMPILED_LOGIC_STORE.compile(logic_json)
}

/// Compile logic from a Value and return a unique ID
///
/// The compiled logic is stored in a global thread-safe cache.
/// If the same logic was compiled before, returns the existing ID.
pub fn compile_logic_value(logic: &serde_json::Value) -> Result<CompiledLogicId, String> {
    COMPILED_LOGIC_STORE.compile_value(logic)
}

/// Get compiled logic by ID
pub fn get_compiled_logic(id: CompiledLogicId) -> Option<CompiledLogic> {
    COMPILED_LOGIC_STORE.get(id)
}

/// Get statistics about the global compiled logic store
pub fn get_store_stats() -> CompiledLogicStoreStats {
    COMPILED_LOGIC_STORE.stats()
}

/// Clear all compiled logic from the global store
///
/// **Warning**: This will invalidate all existing CompiledLogicIds
#[cfg(test)]
pub fn clear_store() {
    COMPILED_LOGIC_STORE.clear()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Test mutex to serialize access to the global store during tests
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_compile_and_get() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_store(); // Ensure clean state

        let logic = r#"{"==": [{"var": "x"}, 10]}"#;
        let id = compile_logic(logic).expect("Failed to compile");

        let compiled = get_compiled_logic(id);
        assert!(compiled.is_some());
    }

    #[test]
    fn test_deduplication() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_store(); // Ensure clean state

        let logic = r#"{"*": [{"var": "a"}, 2]}"#;

        let id1 = compile_logic(logic).expect("Failed to compile");
        let id2 = compile_logic(logic).expect("Failed to compile");

        // Same logic should return same ID
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_different_logic() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_store(); // Ensure clean state

        let logic1 = r#"{"*": [{"var": "a"}, 2]}"#;
        let logic2 = r#"{"*": [{"var": "b"}, 3]}"#;

        let id1 = compile_logic(logic1).expect("Failed to compile");
        let id2 = compile_logic(logic2).expect("Failed to compile");

        // Different logic should return different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_stats() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_store(); // Ensure clean state

        // Compile some logic to populate the store
        let logic = r#"{"+": [1, 2, 3]}"#;
        let _ = compile_logic(logic).expect("Failed to compile");

        let stats = get_store_stats();
        assert_eq!(stats.compiled_count, 1);
        assert_eq!(stats.next_id, 2);
    }
}
