use super::JSONEval;
use crate::jsoneval::path_utils;
use crate::jsoneval::types::ReturnFormat;



use serde_json::Value;
use crate::time_block;


impl JSONEval {
    /// Get the evaluated schema with optional layout resolution.
    ///
    /// # Arguments
    ///
    /// * `skip_layout` - Whether to skip layout resolution.
    ///
    /// # Returns
    ///
    /// The evaluated schema as a JSON value.
    pub fn get_evaluated_schema(&mut self, skip_layout: bool) -> Value {
        time_block!("get_evaluated_schema()", {
            if !skip_layout {
                if let Err(e) = self.resolve_layout(false) {
                    eprintln!("Warning: Layout resolution failed in get_evaluated_schema: {}", e);
                }
            }
            self.evaluated_schema.clone()
        })
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
    pub fn get_evaluated_schema_without_params(&mut self, skip_layout: bool) -> Value {
        let mut schema = self.get_evaluated_schema(skip_layout);
        if let Value::Object(ref mut map) = schema {
            map.remove("$params");
        }
        schema
    }

    /// Get evaluated schema as MessagePack bytes
    pub fn get_evaluated_schema_msgpack(&mut self, skip_layout: bool) -> Result<Vec<u8>, String> {
        let schema = self.get_evaluated_schema(skip_layout);
        rmp_serde::to_vec(&schema).map_err(|e| format!("MessagePack serialization failed: {}", e))
    }

    /// Get value from evaluated schema by path
    pub fn get_evaluated_schema_by_path(&mut self, path: &str, skip_layout: bool) -> Option<Value> {
        if !skip_layout {
            if let Err(e) = self.resolve_layout(false) {
                eprintln!("Warning: Layout resolution failed in get_evaluated_schema_by_path: {}", e);
            }
        }
        self.get_schema_value_by_path(path)
    }

    /// Get evaluated schema parts by multiple paths
    pub fn get_evaluated_schema_by_paths(
        &mut self,
        paths: &[String],
        skip_layout: bool,
        format: Option<ReturnFormat>,
    ) -> Value {
        if !skip_layout {
            if let Err(e) = self.resolve_layout(false) {
                eprintln!("Warning: Layout resolution failed in get_evaluated_schema_by_paths: {}", e);
            }
        }

        match format.unwrap_or(ReturnFormat::Nested) {
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
        format: Option<ReturnFormat>,
    ) -> Value {
        match format.unwrap_or(ReturnFormat::Nested) {
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
