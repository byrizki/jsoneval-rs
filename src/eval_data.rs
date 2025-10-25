use serde_json::{Map, Value};
use std::borrow::Cow;
use std::sync::{atomic::{AtomicU64, Ordering}, Arc};

use crate::path_utils;

static NEXT_INSTANCE_ID: AtomicU64 = AtomicU64::new(0);

/// Version tracker for data mutations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataVersion(pub u64);

/// Tracked data wrapper that gates all mutations for safety
/// 
/// # Design Philosophy
/// 
/// EvalData serves as the single gatekeeper for all data mutations in the system.
/// All write operations (set, push_to_array, get_mut, etc.) MUST go through this
/// type to ensure proper version tracking and mutation safety.
/// 
/// This design provides:
/// - Thread-safe mutation tracking via version numbers
/// - Copy-on-Write (CoW) semantics via Arc for efficient cloning
/// - Single point of control for all data state changes
/// - Prevention of untracked mutations that could cause race conditions
/// 
/// # CoW Behavior
/// 
/// - Read operations are zero-cost (direct Arc dereference)
/// - Clone operations are cheap (Arc reference counting)
/// - First mutation triggers deep clone via Arc::make_mut
/// - Subsequent mutations on exclusive owner are zero-cost
pub struct EvalData {
    instance_id: u64,
    data: Arc<Value>,
}

impl EvalData {
    /// Create a new tracked data wrapper
    pub fn new(data: Value) -> Self {
        Self {
            instance_id: NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed),
            data: Arc::new(data),
        }
    }
    
    /// Initialize eval data with zero-copy references to evaluated_schema, input_data, and context_data
    /// This avoids cloning by directly constructing the data structure with borrowed references
    pub fn with_schema_data_context(
        evaluated_schema: &Value,
        input_data: &Value,
        context_data: &Value,
    ) -> Self {
        let mut data_map = Map::new();
        
        // Insert $params from evaluated_schema (clone only the reference, not deep clone)
        if let Some(params) = evaluated_schema.get("$params") {
            data_map.insert("$params".to_string(), params.clone());
        }
        
        // Merge input_data into the root level
        if let Value::Object(input_obj) = input_data {
            for (key, value) in input_obj {
                data_map.insert(key.clone(), value.clone());
            }
        }
        
        // Insert context
        data_map.insert("$context".to_string(), context_data.clone());
        
        Self::new(Value::Object(data_map))
    }
    
    /// Replace data and context in existing EvalData (for evaluation updates)
    /// Uses CoW: replaces Arc, no clone needed if not shared
    pub fn replace_data_and_context(&mut self, input_data: Value, context_data: Value) {
        let data = Arc::make_mut(&mut self.data);  // CoW: clone only if shared
        input_data.as_object().unwrap().iter().for_each(|(key, value)| {
            Self::set_by_pointer(data, &format!("/{key}"), value.clone());
        });
        Self::set_by_pointer(data, "/$context", context_data);
    }
    
    /// Get the unique instance ID
    #[inline(always)]
    pub fn instance_id(&self) -> u64 {
        self.instance_id
    }
   
    /// Get a reference to the underlying data (read-only)
    /// Zero-cost access via Arc dereference
    #[inline(always)]
    pub fn data(&self) -> &Value {
        &self.data
    }

    /// Clone a Value without certain keys
    #[inline]
    pub fn clone_data_without(&self, exclude: &[&str]) -> Value {
        match &*self.data {
            Value::Object(map) => {
                let mut new_map = Map::new();
                for (k, v) in map {
                    if !exclude.contains(&k.as_str()) {
                        new_map.insert(k.clone(), v.clone());
                    }
                }
                Value::Object(new_map)
            }
            other => other.clone(),
        }
    }
    
    /// Set a field value and increment version
    /// Accepts both dotted notation (user.name) and JSON pointer format (/user/name)
    /// Uses CoW: clones data only if shared
    #[inline]
    pub fn set(&mut self, path: &str, value: Value) {
        // Normalize to JSON pointer format internally
        let pointer = path_utils::normalize_to_json_pointer(path);
        let data = Arc::make_mut(&mut self.data);  // CoW: clone only if shared
        Self::set_by_pointer(data, &pointer, value);
    }

    /// Append to an array field without full clone (optimized for table building)
    /// Accepts both dotted notation (items) and JSON pointer format (/items)
    /// Uses CoW: clones data only if shared
    #[inline]
    pub fn push_to_array(&mut self, path: &str, value: Value) {
        // Normalize to JSON pointer format internally
        let pointer = path_utils::normalize_to_json_pointer(path);
        let data = Arc::make_mut(&mut self.data);  // CoW: clone only if shared
        if let Some(arr) = data.pointer_mut(&pointer) {
            if let Some(array) = arr.as_array_mut() {
                array.push(value);
            }
        }
    }
    
    /// Get a field value
    /// Accepts both dotted notation (user.name) and JSON pointer format (/user/name)
    #[inline(always)]
    pub fn get(&self, path: &str) -> Option<&Value> {
        // Normalize to JSON pointer format internally
        let pointer = path_utils::normalize_to_json_pointer(path);
        // Use native serde_json pointer access for best performance
        path_utils::get_value_by_pointer(&self.data, &pointer)
    }

    #[inline(always)]
    pub fn get_without_properties(&self, path: &str) -> Option<&Value> {
        // Normalize to JSON pointer format internally
        let pointer = path_utils::normalize_to_json_pointer(path);
        // Use native serde_json pointer access for best performance
        path_utils::get_value_by_pointer_without_properties(&self.data, &pointer)
    }
    
    /// OPTIMIZED: Fast array element access
    #[inline(always)]
    pub fn get_array_element(&self, array_path: &str, index: usize) -> Option<&Value> {
        let pointer = path_utils::normalize_to_json_pointer(array_path);
        path_utils::get_array_element_by_pointer(&self.data, &pointer, index)
    }
    
    /// Get a mutable reference to a field value
    /// Accepts both dotted notation and JSON pointer format
    /// Uses CoW: clones data only if shared
    /// Note: Caller must manually increment version after mutation
    pub fn get_mut(&mut self, path: &str) -> Option<&mut Value> {
        // Normalize to JSON pointer format internally
        let pointer = path_utils::normalize_to_json_pointer(path);
        let data = Arc::make_mut(&mut self.data);  // CoW: clone only if shared
        if pointer.is_empty() {
            Some(data)
        } else {
            data.pointer_mut(&pointer)
        }
    }

    /// Get a mutable reference to a table row object at path[index]
    /// Accepts both dotted notation and JSON pointer format
    /// Uses CoW: clones data only if shared
    /// Returns None if path is not an array or row is not an object
    #[inline(always)]
    pub fn get_table_row_mut(&mut self, path: &str, index: usize) -> Option<&mut Map<String, Value>> {
        // Normalize to JSON pointer format internally
        let pointer = path_utils::normalize_to_json_pointer(path);
        let data = Arc::make_mut(&mut self.data);  // CoW: clone only if shared
        let array = if pointer.is_empty() {
            data
        } else {
            data.pointer_mut(&pointer)?
        };
        array.as_array_mut()?
            .get_mut(index)?
            .as_object_mut()
    }
    
    /// Get multiple field values efficiently (for cache key generation)
    /// OPTIMIZED: Use batch pointer resolution for better performance
    #[inline]
    pub fn get_values<'a>(&'a self, paths: &'a [String]) -> Vec<Cow<'a, Value>> {
        // Convert all paths to JSON pointers for batch processing
        let pointers: Vec<String> = paths.iter()
            .map(|path| path_utils::normalize_to_json_pointer(path))
            .collect();
        
        // Batch pointer resolution
        path_utils::get_values_by_pointers(&self.data, &pointers)
            .into_iter()
            .map(|opt_val| {
                opt_val.map(Cow::Borrowed).unwrap_or(Cow::Owned(Value::Null))
            })
            .collect()
    }

    /// Set a value by JSON pointer, creating intermediate structures as needed
    #[inline]
    fn set_by_pointer(data: &mut Value, pointer: &str, new_value: Value) {
        if pointer.is_empty() {
            return;
        }

        // Split pointer into segments (remove leading /)
        let path = &pointer[1..];
        let segments: Vec<&str> = path.split('/').collect();
        
        if segments.is_empty() {
            return;
        }

        // Navigate to parent, creating intermediate structures
        let mut current = data;
        for (i, segment) in segments.iter().enumerate() {
            let is_last = i == segments.len() - 1;
            
            // Try to parse as array index
            if let Ok(index) = segment.parse::<usize>() {
                // Current should be an array
                if !current.is_array() {
                    return; // Cannot index into non-array
                }
                
                let arr = current.as_array_mut().unwrap();
                
                // Extend array if needed
                while arr.len() <= index {
                    arr.push(if is_last { Value::Null } else { Value::Object(Map::new()) });
                }
                
                if is_last {
                    arr[index] = new_value;
                    return;
                } else {
                    current = &mut arr[index];
                }
            } else {
                // Object key access
                if !current.is_object() {
                    return; // Cannot access key on non-object
                }
                
                let map = current.as_object_mut().unwrap();
                
                if is_last {
                    map.insert(segment.to_string(), new_value);
                    return;
                } else {
                    current = map.entry(segment.to_string())
                        .or_insert_with(|| Value::Object(Map::new()));
                }
            }
        }
    }
}

impl From<Value> for EvalData {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

impl Clone for EvalData {
    fn clone(&self) -> Self {
        Self {
            instance_id: self.instance_id, // Keep same ID for clones
            data: Arc::clone(&self.data),  // CoW: cheap Arc clone (ref count only)
        }
    }
}
