use indexmap::IndexMap;
use serde_json::Value;
use std::sync::Arc;

pub(crate) fn extract_from_params(params: &mut serde_json::Map<String, Value>) -> IndexMap<String, Arc<Value>> {
    let mut static_arrays = IndexMap::new();
    extract_recursive(params, "/$params", &mut static_arrays);
    static_arrays
}

fn extract_recursive(map: &mut serde_json::Map<String, Value>, prefix: &str, static_arrays: &mut IndexMap<String, Arc<Value>>) {
    // Traverse object looking for arrays
    for (key, child) in map.iter_mut() {
        let path = format!("{}/{}", prefix, key);
        
        if let Value::Array(arr) = child {
            // Extract if large and pure-data (no actionable keys)
            if arr.len() > 10 && !crate::parse_schema::common::has_actionable_keys(child) {
                let marker = Value::Object(serde_json::json!({ "$static_array": path }).as_object().unwrap().clone());
                let array_val = std::mem::replace(child, marker);
                static_arrays.insert(path, Arc::new(array_val));
            }
        } else if let Value::Object(child_map) = child {
            // Recursively extract nested arrays (like in references or options or tables)
            extract_recursive(child_map, &path, static_arrays);
        }
    }
}
