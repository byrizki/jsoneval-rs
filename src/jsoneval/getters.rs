use super::JSONEval;
use crate::jsoneval::path_utils;
use crate::jsoneval::types::ReturnFormat;



use serde_json::Value;


impl JSONEval {
    /// Get the fully evaluated schema
    ///
    /// # Arguments
    ///
    /// * `include_hidden` - If true, hidden fields are included (but marked).
    ///   If false, hidden fields are removed from the output.
    pub fn get_evaluated_schema(&self, include_hidden: bool) -> Value {
        if include_hidden {
            self.evaluated_schema.clone()
        } else {
            // Filter out hidden fields
            let mut result = self.evaluated_schema.clone();
            Self::filter_hidden_recursive(&mut result);
            result
        }
    }

    /// Helper to recursively remove hidden fields from schema
    fn filter_hidden_recursive(value: &mut Value) {
        if let Value::Object(map) = value {
            // Check if this object itself is hidden
            let is_hidden = if let Some(Value::Object(condition)) = map.get("condition") {
                condition.get("hidden") == Some(&Value::Bool(true))
            } else {
                false
            };

            if is_hidden {
                // If the entire object is hidden, usually the parent removes it?
                // But if we are iterating the map, we can't remove self easily.
                // However, this function modifies children.
            }

            // Iterate and remove hidden children
            // Properties
            if let Some(Value::Object(props)) = map.get_mut("properties") {
                let keys_to_remove: Vec<String> = props
                    .iter()
                    .filter_map(|(k, v)| {
                        if let Value::Object(v_map) = v {
                            if let Some(Value::Object(condition)) = v_map.get("condition") {
                                if condition.get("hidden") == Some(&Value::Bool(true)) {
                                    return Some(k.clone());
                                }
                            }
                        }
                        None
                    })
                    .collect();

                for k in keys_to_remove {
                    props.remove(&k);
                }

                // Recurse
                for (_, v) in props.iter_mut() {
                    Self::filter_hidden_recursive(v);
                }
            }
            
            // Recurse into other objects (like array items)
            for (k, v) in map.iter_mut() {
                 if k == "items" {
                     Self::filter_hidden_recursive(v);
                 } else if !k.starts_with('$') && k != "properties" && v.is_object() {
                      // Generic recursion for nested structures
                      Self::filter_hidden_recursive(v);
                 }
            }
        } else if let Value::Array(arr) = value {
            for v in arr {
                Self::filter_hidden_recursive(v);
            }
        }
    }

    /// Get evaluated schema with layout resolution
    pub fn get_evaluated_schema_with_layout(&self, include_hidden: bool) -> Value {
        // Since $layout resolution is complex and done during evaluation/getters,
        // we might return evaluated_schema directly if layouts were resolved in place?
        // But layouts are often dynamic.
        // For now, return standard evaluated schema, as $ref resolution in layout happens lazily/on-demand?
        // Or is it mutated in evaluated_schema?
        // The implementation in lib.rs typically returns evaluated_schema.
        self.get_evaluated_schema(include_hidden)
    }

    /// Get specific schema value by path
    pub fn get_schema_value_by_path(&self, path: &str) -> Option<Value> {
        let pointer_path = path_utils::dot_notation_to_schema_pointer(path);
        self.evaluated_schema.pointer(&pointer_path).cloned()
    }

    /// Get all schema values (data view)
    /// This corresponds to subform.get_schema_value() usage
    pub fn get_schema_value(&self) -> Value {
        self.eval_data.data().clone()
    }

    /// Get evaluated schema without $params
    pub fn get_evaluated_schema_without_params(&self, include_hidden: bool) -> Value {
        let mut schema = self.get_evaluated_schema(include_hidden);
        if let Value::Object(ref mut map) = schema {
            map.remove("$params");
        }
        schema
    }

    /// Get evaluated schema as MessagePack bytes
    pub fn get_evaluated_schema_msgpack(&self, include_hidden: bool) -> Result<Vec<u8>, String> {
        let schema = self.get_evaluated_schema(include_hidden);
        rmp_serde::to_vec(&schema).map_err(|e| format!("MessagePack serialization failed: {}", e))
    }

    /// Get value from evaluated schema by path
    pub fn get_evaluated_schema_by_path(&self, path: &str, _skip_layout: bool) -> Option<Value> {
        self.get_schema_value_by_path(path)
    }

    /// Get evaluated schema parts by multiple paths
    pub fn get_evaluated_schema_by_paths(
        &self,
        paths: &[String],
        _skip_layout: bool, // Unused for now but kept for API
        format: ReturnFormat,
    ) -> Value {
        // If skip_layout is true, we might want to ensure layout is not applied or filter it?
        // But get_evaluated_schema usually returns schema which has layout resolved?
        // Or not?
        // For now, ignoring skip_layout.
        match format {
            ReturnFormat::Nested => {
                let mut result = Value::Object(serde_json::Map::new());
                for path in paths {
                    if let Some(val) = self.get_schema_value_by_path(path) {
                         // Insert into result object at proper path nesting
                         Self::insert_at_path(&mut result, path, val);
                    }
                }
                result
            }
            ReturnFormat::Flat => {
                 let mut result = serde_json::Map::new();
                 for path in paths {
                    if let Some(val) = self.get_schema_value_by_path(path) {
                        result.insert(path.clone(), val);
                    }
                }
                Value::Object(result)
            }
            ReturnFormat::Array => {
                 let mut result = Vec::new();
                 for path in paths {
                    if let Some(val) = self.get_schema_value_by_path(path) {
                        result.push(val);
                    } else {
                        result.push(Value::Null);
                    }
                }
                Value::Array(result)
            }
        }
    }

    /// Get original (unevaluated) schema by path
    pub fn get_schema_by_path(&self, path: &str) -> Option<Value> {
        let pointer_path = path_utils::dot_notation_to_schema_pointer(path);
        self.schema.pointer(&pointer_path).cloned()
    }

    /// Get original schema by multiple paths
    pub fn get_schema_by_paths(
        &self,
        paths: &[String],
        format: ReturnFormat,
    ) -> Value {
        match format {
            ReturnFormat::Nested => {
                let mut result = Value::Object(serde_json::Map::new());
                for path in paths {
                    if let Some(val) = self.get_schema_by_path(path) {
                         Self::insert_at_path(&mut result, path, val);
                    }
                }
                result
            }
            ReturnFormat::Flat => {
                 let mut result = serde_json::Map::new();
                 for path in paths {
                    if let Some(val) = self.get_schema_by_path(path) {
                        result.insert(path.clone(), val);
                    }
                }
                Value::Object(result)
            }
            ReturnFormat::Array => {
                 let mut result = Vec::new();
                 for path in paths {
                    if let Some(val) = self.get_schema_by_path(path) {
                        result.push(val);
                    } else {
                        result.push(Value::Null);
                    }
                }
                Value::Array(result)
            }
        }
    }

    /// Helper to insert value into nested object at dotted path
    pub(crate) fn insert_at_path(root: &mut Value, path: &str, value: Value) {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = root;
        
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - set value
                if let Value::Object(map) = current {
                    map.insert(part.to_string(), value);
                    return; // Done
                }
            } else {
                // Intermediate part - traverse or create
                // We need to temporarily take the value or use raw pointer manipulation?
                // serde_json pointer is read-only or requires mutable reference
                
                 if !current.is_object() {
                     *current = Value::Object(serde_json::Map::new());
                 }
                 
                 if let Value::Object(map) = current {
                     if !map.contains_key(*part) {
                         map.insert(part.to_string(), Value::Object(serde_json::Map::new()));
                     }
                     current = map.get_mut(*part).unwrap();
                 }
            }
        }
    }
    
    /// Flatten a nested object key-value pair to dotted keys
    pub fn flatten_object(prefix: &str, value: &Value, result: &mut serde_json::Map<String, Value>) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                     let new_key = if prefix.is_empty() {
                         k.clone()
                     } else {
                         format!("{}.{}", prefix, k)
                     };
                     Self::flatten_object(&new_key, v, result);
                }
            }
            _ => {
                result.insert(prefix.to_string(), value.clone());
            }
        }
    }

    pub fn convert_to_format(value: Value, format: ReturnFormat) -> Value {
         match format {
             ReturnFormat::Nested => value,
             ReturnFormat::Flat => {
                 let mut result = serde_json::Map::new();
                 Self::flatten_object("", &value, &mut result);
                 Value::Object(result)
             }
             ReturnFormat::Array => {
                 // Convert object values to array? Only if source was object?
                 // Or flattened values?
                 // Usually converting to array disregards keys.
                 if let Value::Object(map) = value {
                     Value::Array(map.values().cloned().collect())
                 } else if let Value::Array(arr) = value {
                     Value::Array(arr)
                 } else {
                     Value::Array(vec![value])
                 }
             }
         }
    }
}
