//! Path utilities for JSON pointer operations
//! 
//! This module provides JSON pointer normalization and access functions
//! for efficient native serde_json operations.

use serde_json::Value;

/// Normalize path to JSON pointer format for efficient native access
/// 
/// Handles various input formats:
/// - JSON Schema refs: #/$params/constants/DEATH_SA -> /$params/constants/DEATH_SA
/// - Dotted paths: user.name -> /user/name
/// - Already normalized paths (no-op)
/// - Simple field names: field -> /field
#[inline(always)]
pub fn normalize_to_json_pointer(path: &str) -> String {
    if path.is_empty() {
        return "".to_string();
    }
    
    let mut normalized = path.to_string();
    
    // Handle JSON Schema reference format
    if normalized.starts_with("#/") {
        normalized = normalized[1..].to_string(); // Keep leading /
    } else if !normalized.starts_with('/') {
        // Handle dotted notation: user.name -> /user/name
        if normalized.contains('.') {
            normalized = format!("/{}", normalized.replace('.', "/"));
        } else {
            // Simple field name: field -> /field
            normalized = format!("/{}", normalized);
        }
    }
    
    // Clean up double slashes
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }
    
    // Return valid JSON pointer
    if normalized == "/" {
        "".to_string() // Root reference
    } else {
        normalized
    }
}

/// Convert dotted path to JSON Schema pointer format
/// 
/// This is used for schema paths where properties are nested under `/properties/`
/// 
/// Examples:
/// - "illustration.insured.name" -> "#/illustration/properties/insured/properties/name"
/// - "header.form_number" -> "#/header/properties/form_number"
/// - "#/already/formatted" -> "#/already/formatted" (no change)
#[inline(always)]
pub fn dot_notation_to_schema_pointer(path: &str) -> String {
    // If already a JSON pointer (starts with # or /), return as-is
    if path.starts_with('#') || path.starts_with('/') {
        return path.to_string();
    }
    
    // Split by dots and join with /properties/
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return "#/".to_string();
    }
    
    // Build schema path: #/part1/properties/part2/properties/part3
    // First part is root-level field, rest are under /properties/
    let mut result = String::from("#/");
    for (i, part) in parts.iter().enumerate() {
        if part.eq(&"properties") {
            continue;
        }

        if i > 0 {
            result.push_str("/properties/");
        }
        result.push_str(part);
    }
    
    result
}

/// Convert JSON pointer or schema pointer to dotted notation
/// 
/// This converts various pointer formats back to dotted notation:
/// 
/// Examples:
/// - "#/illustration/properties/insured/properties/ins_corrname" -> "illustration.properties.insured.properties.ins_corrname"
/// - "/user/name" -> "user.name"
/// - "person.name" -> "person.name" (already dotted, no change)
#[inline(always)]
pub fn pointer_to_dot_notation(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    
    // If already dotted notation (no # or / prefix), return as-is
    if !path.starts_with('#') && !path.starts_with('/') {
        return path.to_string();
    }
    
    // Remove leading # or /
    let clean_path = if path.starts_with("#/") {
        &path[2..]
    } else if path.starts_with('/') {
        &path[1..]
    } else if path.starts_with('#') {
        &path[1..]
    } else {
        path
    };
    
    // Convert slashes to dots
    clean_path.replace('/', ".")
}

/// Fast JSON pointer-based value access using serde's native implementation
/// 
/// This is significantly faster than manual path traversal for deeply nested objects
#[inline(always)]
pub fn get_value_by_pointer<'a>(data: &'a Value, pointer: &str) -> Option<&'a Value> {
    if pointer.is_empty() {
        Some(data)
    } else {
        data.pointer(pointer)
    }
}

#[inline(always)]
pub fn get_value_by_pointer_without_properties<'a>(data: &'a Value, pointer: &str) -> Option<&'a Value> {
    if pointer.is_empty() {
        Some(data)
    } else {
        data.pointer(&pointer.replace("properties/", ""))
    }
}

/// Batch pointer resolution for multiple paths
pub fn get_values_by_pointers<'a>(data: &'a Value, pointers: &[String]) -> Vec<Option<&'a Value>> {
    pointers.iter()
        .map(|pointer| get_value_by_pointer(data, pointer))
        .collect()
}

/// Fast array indexing helper for JSON arrays
/// 
/// Returns None if not an array or index out of bounds
#[inline(always)]
pub fn get_array_element<'a>(data: &'a Value, index: usize) -> Option<&'a Value> {
    data.as_array()?.get(index)
}

/// Fast array indexing with JSON pointer path
/// 
/// Example: get_array_element_by_pointer(data, "/$params/tables", 0)
#[inline(always)]
pub fn get_array_element_by_pointer<'a>(data: &'a Value, pointer: &str, index: usize) -> Option<&'a Value> {
    get_value_by_pointer(data, pointer)?
        .as_array()?
        .get(index)
}

/// Extract table metadata for fast array operations during schema parsing
#[derive(Debug, Clone)]
pub struct ArrayMetadata {
    /// Pointer to the array location
    pub pointer: String,
    /// Array length (cached for fast bounds checking)
    pub length: usize,
    /// Column names for object arrays (cached for fast field access)
    pub column_names: Vec<String>,
    /// Whether this is a uniform object array (all elements have same structure)
    pub is_uniform: bool,
}

impl ArrayMetadata {
    /// Build metadata for an array at the given pointer
    pub fn build(data: &Value, pointer: &str) -> Option<Self> {
        let array = get_value_by_pointer(data, pointer)?.as_array()?;
        
        let length = array.len();
        if length == 0 {
            return Some(ArrayMetadata {
                pointer: pointer.to_string(),
                length: 0,
                column_names: Vec::new(),
                is_uniform: true,
            });
        }
        
        // Analyze first element to determine structure
        let first_element = &array[0];
        let column_names = if let Value::Object(obj) = first_element {
            obj.keys().cloned().collect()
        } else {
            Vec::new()
        };
        
        // Check if all elements have the same structure (uniform array)
        let is_uniform = if !column_names.is_empty() {
            array.iter().all(|elem| {
                if let Value::Object(obj) = elem {
                    obj.keys().len() == column_names.len() &&
                    column_names.iter().all(|col| obj.contains_key(col))
                } else {
                    false
                }
            })
        } else {
            // Non-object arrays are considered uniform if all elements have same type
            let first_type = std::mem::discriminant(first_element);
            array.iter().all(|elem| std::mem::discriminant(elem) == first_type)
        };
        
        Some(ArrayMetadata {
            pointer: pointer.to_string(),
            length,
            column_names,
            is_uniform,
        })
    }
    
    /// Fast column access for uniform object arrays
    #[inline(always)]
    pub fn get_column_value<'a>(&self, data: &'a Value, row_index: usize, column: &str) -> Option<&'a Value> {
        if !self.is_uniform || row_index >= self.length {
            return None;
        }
        
        get_array_element_by_pointer(data, &self.pointer, row_index)?
            .as_object()?
            .get(column)
    }
    
    /// Fast bounds checking
    #[inline(always)]
    pub fn is_valid_index(&self, index: usize) -> bool {
        index < self.length
    }
}


