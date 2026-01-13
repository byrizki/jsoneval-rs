use super::JSONEval;
use crate::jsoneval::path_utils;
use crate::time_block;

use serde_json::Value;
use std::mem;

impl JSONEval {
    /// Resolve layout references with optional evaluation
    ///
    /// # Arguments
    ///
    /// * `evaluate` - If true, runs evaluation before resolving layout. If false, only resolves layout.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error message.
    pub fn resolve_layout(&mut self, evaluate: bool) -> Result<(), String> {
        if evaluate {
            // Use existing data
            let data_str = serde_json::to_string(&self.data)
                .map_err(|e| format!("Failed to serialize data: {}", e))?;
            self.evaluate(&data_str, None, None, None)?;
        }

        self.resolve_layout_internal();
        Ok(())
    }

    fn resolve_layout_internal(&mut self) {
        time_block!("  resolve_layout_internal()", {
            // Use cached layout paths (collected at parse time)
            let layout_paths = self.layout_paths.clone();

            time_block!("    resolve_layout_elements", {
                for layout_path in layout_paths.iter() {
                    self.resolve_layout_elements(layout_path);
                }
            });

            // After resolving all references, propagate parent hidden/disabled to children
            time_block!("    propagate_parent_conditions", {
                for layout_path in layout_paths.iter() {
                    self.propagate_parent_conditions(layout_path);
                }
            });
        });
    }

    /// Resolve $ref references in layout elements (recursively)
    fn resolve_layout_elements(&mut self, layout_elements_path: &str) {
        // Normalize path from schema format (#/) to JSON pointer format (/)
        let normalized_path = path_utils::normalize_to_json_pointer(layout_elements_path);

        // Always read elements from original schema (not evaluated_schema)
        // This ensures we get fresh $ref entries on re-evaluation
        let elements = if let Some(Value::Array(arr)) = self.schema.pointer(&normalized_path) {
            arr.clone()
        } else {
            return;
        };

        // Extract the parent path from normalized_path
        let parent_path = normalized_path
            .trim_start_matches('/')
            .replace("/elements", "")
            .replace('/', ".");

        // Process elements
        let mut resolved_elements = Vec::with_capacity(elements.len());
        for (index, element) in elements.iter().enumerate() {
            let element_path = if parent_path.is_empty() {
                format!("elements.{}", index)
            } else {
                format!("{}.elements.{}", parent_path, index)
            };
            let resolved = self.resolve_element_ref_recursive(element.clone(), &element_path);
            resolved_elements.push(resolved);
        }

        // Write back the resolved elements
        if let Some(target) = self.evaluated_schema.pointer_mut(&normalized_path) {
            *target = Value::Array(resolved_elements);
        }
    }

    /// Recursively resolve $ref in an element and its nested elements
    /// path_context: The dotted path to the current element (e.g., "form.$layout.elements.0")
    fn resolve_element_ref_recursive(&self, element: Value, path_context: &str) -> Value {
        // First resolve the current element's $ref
        let resolved = self.resolve_element_ref(element);

        // Then recursively resolve any nested elements arrays
        if let Value::Object(mut map) = resolved {
            // Ensure all layout elements have metadata fields
            if !map.contains_key("$parentHide") {
                map.insert("$parentHide".to_string(), Value::Bool(false));
            }

            // Set path metadata for direct layout elements (without $ref)
            if !map.contains_key("$fullpath") {
                map.insert("$fullpath".to_string(), Value::String(path_context.to_string()));
            }

            if !map.contains_key("$path") {
                let last_segment = path_context.split('.').last().unwrap_or(path_context);
                map.insert("$path".to_string(), Value::String(last_segment.to_string()));
            }

            // Check if this object has an "elements" array
            if let Some(Value::Array(elements)) = map.get("elements") {
                let mut resolved_nested = Vec::with_capacity(elements.len());
                for (index, nested_element) in elements.iter().enumerate() {
                    let nested_path = format!("{}.elements.{}", path_context, index);
                    resolved_nested.push(self.resolve_element_ref_recursive(nested_element.clone(), &nested_path));
                }
                map.insert("elements".to_string(), Value::Array(resolved_nested));
            }

            return Value::Object(map);
        }

        resolved
    }

    /// Resolve $ref in a single element
    fn resolve_element_ref(&self, element: Value) -> Value {
        match element {
            Value::Object(mut map) => {
                // Check if element has $ref
                if let Some(Value::String(ref_path)) = map.get("$ref").cloned() {
                    // Convert ref_path to dotted notation for metadata storage
                    let dotted_path = path_utils::pointer_to_dot_notation(&ref_path);

                    // Extract last segment for $path
                    let last_segment = dotted_path.split('.').last().unwrap_or(&dotted_path);

                    // Inject metadata fields with dotted notation
                    map.insert("$fullpath".to_string(), Value::String(dotted_path.clone()));
                    map.insert("$path".to_string(), Value::String(last_segment.to_string()));
                    map.insert("$parentHide".to_string(), Value::Bool(false));

                    // Normalize to JSON pointer for actual lookup
                    let normalized_path = if ref_path.starts_with('#') || ref_path.starts_with('/') {
                        path_utils::normalize_to_json_pointer(&ref_path)
                    } else {
                        // Try as schema path first
                        let schema_pointer = path_utils::dot_notation_to_schema_pointer(&ref_path);
                        let schema_path = path_utils::normalize_to_json_pointer(&schema_pointer);

                        // Check if it exists
                        if self.evaluated_schema.pointer(&schema_path).is_some() {
                            schema_path
                        } else {
                            // Try with /properties/ prefix
                            format!("/properties/{}", ref_path.replace('.', "/properties/"))
                        }
                    };

                    // Get the referenced value
                    if let Some(referenced_value) = self.evaluated_schema.pointer(&normalized_path) {
                        let resolved = referenced_value.clone();

                        if let Value::Object(mut resolved_map) = resolved {
                            map.remove("$ref");

                            // Special case: if resolved has $layout, flatten it
                            if let Some(Value::Object(layout_obj)) = resolved_map.remove("$layout") {
                                let mut result = layout_obj.clone();

                                // properties are now preserved and will be merged below

                                // Merge remaining resolved_map properties
                                for (key, value) in resolved_map {
                                    if key != "type" || !result.contains_key("type") {
                                        result.insert(key, value);
                                    }
                                }

                                // Finally, merge element override properties
                                for (key, value) in map {
                                    result.insert(key, value);
                                }

                                return Value::Object(result);
                            } else {
                                // Normal merge: element properties override referenced properties
                                for (key, value) in map {
                                    resolved_map.insert(key, value);
                                }

                                return Value::Object(resolved_map);
                            }
                        } else {
                            return resolved;
                        }
                    }
                }

                Value::Object(map)
            }
            _ => element,
        }
    }

    /// Propagate parent hidden/disabled conditions to children recursively
    fn propagate_parent_conditions(&mut self, layout_elements_path: &str) {
        let normalized_path = path_utils::normalize_to_json_pointer(layout_elements_path);

        // Extract elements array to avoid borrow checker issues
        let elements = if let Some(Value::Array(arr)) = self.evaluated_schema.pointer_mut(&normalized_path) {
            mem::take(arr)
        } else {
            return;
        };

        // Process elements
        let mut updated_elements = Vec::with_capacity(elements.len());
        for element in elements {
            updated_elements.push(self.apply_parent_conditions(element, false, false));
        }

        // Write back  the updated elements
        if let Some(target) = self.evaluated_schema.pointer_mut(&normalized_path) {
            *target = Value::Array(updated_elements);
        }
    }

    /// Recursively apply parent hidden/disabled conditions to an element and its children
    fn apply_parent_conditions(&self, element: Value, parent_hidden: bool, parent_disabled: bool) -> Value {
        if let Value::Object(mut map) = element {
            // Get current element's condition
            let mut element_hidden = parent_hidden;
            let mut element_disabled = parent_disabled;

            // Check condition field (from $ref elements)
            if let Some(Value::Object(condition)) = map.get("condition") {
                if let Some(Value::Bool(hidden)) = condition.get("hidden") {
                    element_hidden = element_hidden || *hidden;
                }
                if let Some(Value::Bool(disabled)) = condition.get("disabled") {
                    element_disabled = element_disabled || *disabled;
                }
            }

            // Check hideLayout field (from direct layout elements)
            if let Some(Value::Object(hide_layout)) = map.get("hideLayout") {
                if let Some(Value::Bool(all)) = hide_layout.get("all") {
                    if *all {
                        element_hidden = true;
                    }
                }
            }

            // Update condition to include parent state (for field elements)
            if parent_hidden || parent_disabled {
                // Update condition field if it exists or if this is a field element
                if map.contains_key("condition")
                    || map.contains_key("$ref")
                    || map.contains_key("$fullpath")
                {
                    let mut condition = if let Some(Value::Object(c)) = map.get("condition") {
                        c.clone()
                    } else {
                        serde_json::Map::new()
                    };

                    if parent_hidden {
                        condition.insert("hidden".to_string(), Value::Bool(true));
                        element_hidden = true;
                    }
                    if parent_disabled {
                        condition.insert("disabled".to_string(), Value::Bool(true));
                        element_disabled = true;
                    }

                    map.insert("condition".to_string(), Value::Object(condition));
                }

                // Update hideLayout for direct layout elements
                if parent_hidden && (map.contains_key("hideLayout") || map.contains_key("type")) {
                    let mut hide_layout = if let Some(Value::Object(h)) = map.get("hideLayout") {
                        h.clone()
                    } else {
                        serde_json::Map::new()
                    };

                    // Set hideLayout.all to true when parent is hidden
                    hide_layout.insert("all".to_string(), Value::Bool(true));
                    map.insert("hideLayout".to_string(), Value::Object(hide_layout));
                }
            }

            // Update $parentHide flag if element has it
            if map.contains_key("$parentHide") {
                map.insert("$parentHide".to_string(), Value::Bool(parent_hidden));
            }

            // Recursively process children if elements array exists
            if let Some(Value::Array(elements)) = map.get("elements") {
                let mut updated_children = Vec::with_capacity(elements.len());
                for child in elements {
                    updated_children.push(self.apply_parent_conditions(child.clone(), element_hidden, element_disabled));
                }
                map.insert("elements".to_string(), Value::Array(updated_children));
            }

            return Value::Object(map);
        }

        element
    }
}
