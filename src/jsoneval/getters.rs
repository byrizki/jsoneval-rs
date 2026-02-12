use super::JSONEval;
use crate::jsoneval::path_utils;
use crate::jsoneval::types::ReturnFormat;



use serde_json::Value;
use crate::time_block;


impl JSONEval {
    /// Check if a field is effectively hidden by checking its condition and all parents
    /// Also checks for $layout.hideLayout.all on parents
    fn is_effective_hidden(&self, schema_pointer: &str) -> bool {
        let mut current_path = schema_pointer.to_string();

        // Loop until broken (when we checked root)
        loop {
            // Check if current node exists
            if let Some(schema_node) = self.evaluated_schema.pointer(&current_path) {
                if let Value::Object(map) = schema_node {
                    // 1. Check condition.hidden
                    if let Some(Value::Object(condition)) = map.get("condition") {
                        if let Some(Value::Bool(hidden)) = condition.get("hidden") {
                            if *hidden {
                                return true;
                            }
                        }
                    }

                    // 2. Check $layout.hideLayout.all (on the object itself, as $layout is sibling to properties)
                    // Note: This check applies to the container (e.g., "header" object), hiding all its children
                    // But here we are traversing up. If "header" has hideLayout.all=true, then "header" itself is hidden
                    // and its children are hidden.
                    if let Some(Value::Object(layout)) = map.get("$layout") {
                        if let Some(Value::Object(hide_layout)) = layout.get("hideLayout") {
                            if let Some(Value::Bool(all)) = hide_layout.get("all") {
                                if *all {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            
            // If we just checked root, break
            if current_path.is_empty() {
                break;
            }

            // Move to parent
            // Path format: /properties/foo/properties/bar or /items/0/properties/foo
            // We want to strip the last key AND the /properties or /items segment
            
            // Find last slash
            if let Some(last_slash_idx) = current_path.rfind('/') {
                let parent_path_raw = &current_path[..last_slash_idx];
                
                // If empty, next path is root
                if parent_path_raw.is_empty() {
                    current_path = "".to_string();
                    continue;
                }
                
                // Check if parent path ends with /properties or /items and strip it
                if parent_path_raw.ends_with("/properties") {
                    current_path = parent_path_raw[..parent_path_raw.len() - "/properties".len()].to_string();
                } else if parent_path_raw.ends_with("/items") { // arrays not fully supported yet in this traversal but good to handle
                     current_path = parent_path_raw[..parent_path_raw.len() - "/items".len()].to_string();
                } else {
                     // Fallback or intermediate path - just go up one level?
                     // If we have structure like /foo/bar without properties (not standard schema), just strip last segment again?
                     // BUT wait, parent_path_raw IS the parent path if not ending in special.
                     current_path = parent_path_raw.to_string();
                }
            } else {
                // No slash found but not empty? Should not happen if normalized.
                // Assuming root checked above, break.
                break;
            }
        }

        false
    }
    
    /// Prune hidden values from data object recursively
    fn prune_hidden_values(&self, data: &mut Value, current_path: &str) {
        if let Value::Object(map) = data {
            // Collect keys to remove to avoid borrow checker issues
            let mut keys_to_remove = Vec::new();
            
            for (key, value) in map.iter_mut() {
                // Skip special keys
                if key == "$params" || key == "$context" {
                    continue;
                }
                
                // Construct schema path for this key
                // For root fields: /properties/key
                // For nested fields: current_path/properties/key
                let schema_path = if current_path.is_empty() {
                    format!("/properties/{}", key)
                } else {
                    format!("{}/properties/{}", current_path, key)
                };
                
                // Check if hidden
                if self.is_effective_hidden(&schema_path) {
                    keys_to_remove.push(key.clone());
                } else {
                    // Recurse if object
                    if value.is_object() {
                        self.prune_hidden_values(value, &schema_path);
                    }
                }
            }
            
            // Remove hidden keys
            for key in keys_to_remove {
                map.remove(&key);
            }
        }
    }

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
        self.evaluated_schema.pointer(&pointer_path.trim_start_matches('#')).cloned()
    }

    /// Get all schema values (data view)
    /// Mutates internal data state by overriding with values from value evaluations
    /// This corresponds to subform.get_schema_value() usage
    pub fn get_schema_value(&mut self) -> Value {
        // Start with current authoritative data from eval_data
        let mut current_data = self.eval_data.data().clone();

        // Ensure it's an object
        if !current_data.is_object() {
            current_data = Value::Object(serde_json::Map::new());
        }

        // Strip $params and $context from data
        if let Some(obj) = current_data.as_object_mut() {
            obj.remove("$params");
            obj.remove("$context");
        }
        
        // Prune hidden values from current_data (to remove user input in hidden fields)
        self.prune_hidden_values(&mut current_data, "");

        // Override data with values from value evaluations
        // We use value_evaluations which stores the paths of fields with .value
        for eval_key in self.value_evaluations.iter() {
            let clean_key = eval_key.replace('#', "");

            // Exclude rules.*.value, options.*.value, and $params
            if clean_key.starts_with("/$params")
                || (clean_key.ends_with("/value")
                    && (clean_key.contains("/rules/") || clean_key.contains("/options/")))
            {
                continue;
            }

            let path = clean_key.replace("/properties", "").replace("/value", "");
            
            // Check if field is effectively hidden
            // Schema path is clean_key without /value
            let schema_path = clean_key.strip_suffix("/value").unwrap_or(&clean_key);
            if self.is_effective_hidden(schema_path) {
                continue;
            }

            // Get the value from evaluated_schema
            let value = match self.evaluated_schema.pointer(&clean_key) {
                Some(v) => v.clone(),
                None => continue,
            };

            // Parse the path and create nested structure as needed
            let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

            if path_parts.is_empty() {
                continue;
            }

            // Navigate/create nested structure
            let mut current = &mut current_data;
            for (i, part) in path_parts.iter().enumerate() {
                let is_last = i == path_parts.len() - 1;

                if is_last {
                    // Set the value at the final key
                    if let Some(obj) = current.as_object_mut() {
                        obj.insert(part.to_string(), crate::utils::clean_float_noise(value.clone()));
                    }
                } else {
                    // Ensure current is an object, then navigate/create intermediate objects
                    if let Some(obj) = current.as_object_mut() {
                        // Use raw entry API or standard entry if possible, but borrowing is tricky
                        // We need to re-borrow `current` for the next iteration
                        // Since `entry` API consumes check, we might need a different approach or careful usage
                        
                        // Check presence first to avoid borrow issues if simpler
                        if !obj.contains_key(*part) {
                            obj.insert((*part).to_string(), Value::Object(serde_json::Map::new()));
                        }
                        
                        current = obj.get_mut(*part).unwrap();
                    } else {
                        // Skip this path if current is not an object and can't be made into one
                        break;
                    }
                }
            }
        }
        
        // Update self.data to persist the view changes (matching backup behavior)
        self.data = current_data.clone();
        
        crate::utils::clean_float_noise(current_data)
    }

    /// Get all schema values as array of path-value pairs
    /// Returns [{path: "", value: ""}, ...]
    ///
    /// # Returns
    ///
    /// Array of objects containing path (dotted notation) and value pairs from value evaluations
    pub fn get_schema_value_array(&self) -> Value {
        let mut result = Vec::new();
        
        for eval_key in self.value_evaluations.iter() {
            let clean_key = eval_key.replace('#', "");

            // Exclude rules.*.value, options.*.value, and $params
            if clean_key.starts_with("/$params")
                || (clean_key.ends_with("/value")
                    && (clean_key.contains("/rules/") || clean_key.contains("/options/")))
            {
                continue;
            }
            
            // Check if field is effectively hidden
            let schema_path = clean_key.strip_suffix("/value").unwrap_or(&clean_key);
            if self.is_effective_hidden(schema_path) {
                continue;
            }

            // Convert JSON pointer to dotted notation
            let dotted_path = clean_key
                .replace("/properties", "")
                .replace("/value", "")
                .trim_start_matches('/')
                .replace('/', ".");

            if dotted_path.is_empty() {
                continue;
            }

            // Get the value from evaluated_schema
            let value = match self.evaluated_schema.pointer(&clean_key) {
                Some(v) => crate::utils::clean_float_noise(v.clone()),
                None => continue,
            };

            // Create {path, value} object
            let mut item = serde_json::Map::new();
            item.insert("path".to_string(), Value::String(dotted_path));
            item.insert("value".to_string(), value);
            result.push(Value::Object(item));
        }
        
        Value::Array(result)
    }

    /// Get all schema values as object with dotted path keys
    /// Returns {path: value, ...}
    ///
    /// # Returns
    ///
    /// Flat object with dotted notation paths as keys and evaluated values
    pub fn get_schema_value_object(&self) -> Value {
        let mut result = serde_json::Map::new();
        
        for eval_key in self.value_evaluations.iter() {
            let clean_key = eval_key.replace('#', "");

            // Exclude rules.*.value, options.*.value, and $params
            if clean_key.starts_with("/$params")
                || (clean_key.ends_with("/value")
                    && (clean_key.contains("/rules/") || clean_key.contains("/options/")))
            {
                continue;
            }
            
            // Check if field is effectively hidden
            let schema_path = clean_key.strip_suffix("/value").unwrap_or(&clean_key);
            if self.is_effective_hidden(schema_path) {
                continue;
            }

            // Convert JSON pointer to dotted notation
            let dotted_path = clean_key
                .replace("/properties", "")
                .replace("/value", "")
                .trim_start_matches('/')
                .replace('/', ".");

            if dotted_path.is_empty() {
                continue;
            }

            // Get the value from evaluated_schema
            let value = match self.evaluated_schema.pointer(&clean_key) {
                Some(v) => crate::utils::clean_float_noise(v.clone()),
                None => continue,
            };

            result.insert(dotted_path, value);
        }
        
        Value::Object(result)
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
        self.schema.pointer(&pointer_path.trim_start_matches('#')).cloned()
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
