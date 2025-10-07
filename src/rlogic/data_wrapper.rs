use serde_json::{Map, Value};
use std::borrow::Cow;
use std::sync::{atomic::{AtomicU64, Ordering}, RwLock};
use rustc_hash::FxHashMap;

use super::path::{parse_path, traverse, traverse_mut, PathSegment, PathVec};

static NEXT_INSTANCE_ID: AtomicU64 = AtomicU64::new(0);

/// Version tracker for data mutations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataVersion(pub u64);

/// Tracked data wrapper that invalidates cache on mutation (like JS Proxy)
pub struct TrackedData {
    instance_id: u64,
    data: Value,
    version: AtomicU64,
    field_versions: FxHashMap<String, u64>,
    path_cache: RwLock<FxHashMap<String, PathVec>>,
}

impl TrackedData {
    /// Create a new tracked data wrapper
    pub fn new(data: Value) -> Self {
        Self {
            instance_id: NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed),
            data,
            version: AtomicU64::new(0),
            field_versions: FxHashMap::default(),
            path_cache: RwLock::new(FxHashMap::default()),
        }
    }
    
    /// Get the unique instance ID
    pub fn instance_id(&self) -> u64 {
        self.instance_id
    }
    
    #[inline]
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }
    
    /// Get a reference to the underlying data (read-only)
    #[inline]
    pub fn data(&self) -> &Value {
        &self.data
    }
    
    /// Set a field value and increment version
    pub fn set(&mut self, path: &str, value: Value) -> u64 {
        let segments = self.cached_segments(path);
        Self::set_value_with_segments(&segments, &mut self.data, value);
        let new_version = self.version.fetch_add(1, Ordering::AcqRel) + 1;
        self.field_versions.insert(path.to_string(), new_version);
        new_version
    }

    /// Append to an array field without full clone (optimized for table building)
    pub fn push_to_array(&mut self, path: &str, value: Value) -> u64 {
        let segments = self.cached_segments(path);
        if let Some(arr) = traverse_mut(&mut self.data, &segments) {
            if let Some(array) = arr.as_array_mut() {
                array.push(value);
            }
        }
        let new_version = self.version.fetch_add(1, Ordering::AcqRel) + 1;
        self.field_versions.insert(path.to_string(), new_version);
        new_version
    }
    
    /// Get a field value
    pub fn get(&self, path: &str) -> Option<&Value> {
        let segments = self.cached_segments(path);
        traverse(&self.data, &segments)
    }
    
    /// Get a mutable reference to a field value
    /// Note: Caller must manually increment version after mutation
    pub fn get_mut(&mut self, path: &str) -> Option<&mut Value> {
        let segments = self.cached_segments(path);
        traverse_mut(&mut self.data, &segments)
    }
    
    /// Increment version after manual mutation via get_mut
    pub fn mark_modified(&mut self, path: &str) -> u64 {
        let new_version = self.version.fetch_add(1, Ordering::AcqRel) + 1;
        self.field_versions.insert(path.to_string(), new_version);
        new_version
    }
    
    /// Get multiple field values efficiently (for cache key generation)
    pub fn get_values<'a>(&'a self, paths: &'a [String]) -> Vec<Cow<'a, Value>> {
        paths
            .iter()
            .map(|path| {
                let segments = self.cached_segments(path);
                traverse(&self.data, &segments)
                    .map(Cow::Borrowed)
                    .unwrap_or(Cow::Owned(Value::Null))
            })
            .collect()
    }

    /// Get version fingerprints for dependency paths used in cache keys
    pub fn dependency_versions(&self, paths: &[String]) -> Vec<u64> {
        // Use 0 as default for unmodified fields to ensure cache stability
        // Fields that haven't been explicitly set via .set() get version 0
        paths
            .iter()
            .map(|path| self.field_versions.get(path).copied().unwrap_or(0))
            .collect()
    }
    
    /// Check if a field has been modified since a specific version
    pub fn field_modified_since(&self, path: &str, version: u64) -> bool {
        if let Some(&field_version) = self.field_versions.get(path) {
            field_version > version
        } else {
            false
        }
    }

    /// Deep merge another Value into this TrackedData
    /// Only updates values that are different, preserving unchanged values
    /// Returns the number of fields that were actually changed
    pub fn deep_merge(&mut self, other: &Value) -> usize {
        let mut changes = 0;
        Self::deep_merge_recursive(&mut self.data, other, "", &mut changes, &mut self.field_versions, &mut self.version);
        changes
    }
    
    fn deep_merge_recursive(
        target: &mut Value,
        source: &Value,
        path: &str,
        changes: &mut usize,
        field_versions: &mut FxHashMap<String, u64>,
        version: &mut AtomicU64,
    ) {
        match (target, source) {
            (Value::Object(target_map), Value::Object(source_map)) => {
                for (key, source_value) in source_map {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    
                    if let Some(target_value) = target_map.get_mut(key) {
                        // Key exists, recursively merge
                        Self::deep_merge_recursive(
                            target_value,
                            source_value,
                            &new_path,
                            changes,
                            field_versions,
                            version,
                        );
                    } else {
                        // Key doesn't exist, insert new value
                        target_map.insert(key.clone(), source_value.clone());
                        *changes += 1;
                        let new_version = version.fetch_add(1, Ordering::AcqRel) + 1;
                        field_versions.insert(new_path, new_version);
                    }
                }
            }
            (target_val, source_val) => {
                // For non-object values, only update if different
                if target_val != source_val {
                    *target_val = source_val.clone();
                    *changes += 1;
                    if !path.is_empty() {
                        let new_version = version.fetch_add(1, Ordering::AcqRel) + 1;
                        field_versions.insert(path.to_string(), new_version);
                    }
                }
            }
        }
    }
    
    fn set_value_with_segments(segments: &[PathSegment], value: &mut Value, new_value: Value) {
        if segments.is_empty() {
            return;
        }

        if segments.len() == 1 {
            match (&segments[0], value) {
                (PathSegment::Key(key), Value::Object(map)) => {
                    map.insert(key.clone(), new_value);
                }
                (PathSegment::Index(index), Value::Array(arr)) => {
                    while arr.len() <= *index {
                        arr.push(Value::Null);
                    }
                    arr[*index] = new_value;
                }
                _ => {}
            }
            return;
        }

        let mut current = value;
        for segment in &segments[..segments.len() - 1] {
            match (segment, current) {
                (PathSegment::Key(key), Value::Object(map)) => {
                    current = map.entry(key.clone()).or_insert_with(|| Value::Object(Map::new()));
                }
                (PathSegment::Index(index), Value::Array(arr)) => {
                    while arr.len() <= *index {
                        arr.push(Value::Object(Map::new()));
                    }
                    current = &mut arr[*index];
                }
                _ => return,
            }
        }

        Self::set_value_with_segments(&segments[segments.len() - 1..], current, new_value);
    }

    #[inline]
    fn cached_segments(&self, path: &str) -> PathVec {
        if path.is_empty() {
            return PathVec::new();
        }

        if let Some(existing) = self.path_cache.read().unwrap().get(path) {
            return existing.clone();
        }

        let parsed = parse_path(path);
        let mut cache = self.path_cache.write().unwrap();
        cache.entry(path.to_string()).or_insert_with(|| parsed.clone());
        parsed
    }
}

impl From<Value> for TrackedData {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

impl Clone for TrackedData {
    fn clone(&self) -> Self {
        Self {
            instance_id: self.instance_id, // Keep same ID for clones
            data: self.data.clone(),
            version: AtomicU64::new(self.version.load(Ordering::Acquire)),
            field_versions: self.field_versions.clone(),
            path_cache: RwLock::new(FxHashMap::default()),
        }
    }
}

/// Builder for tracked data with mutation tracking
pub struct TrackedDataBuilder {
    data: Value,
}

impl TrackedDataBuilder {
    pub fn new() -> Self {
        Self {
            data: Value::Object(Map::new()),
        }
    }
    
    pub fn from_value(value: Value) -> Self {
        Self { data: value }
    }
    
    pub fn set(mut self, key: &str, value: Value) -> Self {
        if let Value::Object(map) = &mut self.data {
            map.insert(key.to_string(), value);
        }
        self
    }
    
    pub fn build(self) -> TrackedData {
        TrackedData::new(self.data)
    }
}

impl Default for TrackedDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_version_increment() {
        let mut data = TrackedData::new(json!({"name": "test"}));
        let v1 = data.version();
        data.set("name", json!("updated"));
        let v2 = data.version();
        assert!(v2 > v1);
    }
    
    #[test]
    fn test_nested_path() {
        let mut data = TrackedData::new(json!({"user": {"name": "John"}}));
        assert_eq!(data.get("user.name"), Some(&json!("John")));
        
        data.set("user.age", json!(30));
        assert_eq!(data.get("user.age"), Some(&json!(30)));
    }
    
    #[test]
    fn test_field_modification_tracking() {
        let mut data = TrackedData::new(json!({"a": 1, "b": 2}));
        let v1 = data.version();
        
        data.set("a", json!(10));
        
        assert!(data.field_modified_since("a", v1));
        assert!(!data.field_modified_since("b", v1));
    }
    
    #[test]
    fn test_deep_merge_preserves_unchanged() {
        let mut data = TrackedData::new(json!({
            "user": {
                "name": "John",
                "age": 30
            },
            "settings": {
                "theme": "dark"
            }
        }));
        
        let v1 = data.version();
        
        // Merge with same values and one new value
        let other = json!({
            "user": {
                "name": "John",  // Same
                "age": 31,       // Different
                "email": "john@example.com"  // New
            },
            "settings": {
                "theme": "dark"  // Same
            }
        });
        
        let changes = data.deep_merge(&other);
        
        // Should only count actual changes (age changed, email added)
        assert_eq!(changes, 2);
        
        // Verify the merge
        assert_eq!(data.get("user.name"), Some(&json!("John")));
        assert_eq!(data.get("user.age"), Some(&json!(31)));
        assert_eq!(data.get("user.email"), Some(&json!("john@example.com")));
        assert_eq!(data.get("settings.theme"), Some(&json!("dark")));
        
        // Version should have increased
        assert!(data.version() > v1);
    }
    
    #[test]
    fn test_deep_merge_no_changes() {
        let mut data = TrackedData::new(json!({"a": 1, "b": {"c": 2}}));
        let v1 = data.version();
        
        // Merge with identical data
        let other = json!({"a": 1, "b": {"c": 2}});
        let changes = data.deep_merge(&other);
        
        // No changes should be detected
        assert_eq!(changes, 0);
        
        // Version should remain the same
        assert_eq!(data.version(), v1);
    }
}
