use crate::jsoneval::path_utils;
use crate::jsoneval::table_metadata::ColumnMetadata;
/// Shared utilities for schema parsing (used by both legacy and parsed implementations)
use indexmap::IndexSet;
use serde_json::Value;

/// Collect $ref dependencies from a JSON value recursively
pub fn collect_refs(value: &Value, refs: &mut IndexSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(path) = map.get("$ref").and_then(Value::as_str) {
                refs.insert(path_utils::normalize_to_json_pointer(path).into_owned());
            }
            if let Some(path) = map.get("ref").and_then(Value::as_str) {
                refs.insert(path_utils::normalize_to_json_pointer(path).into_owned());
            }
            if let Some(var_val) = map.get("var") {
                match var_val {
                    Value::String(s) => {
                        refs.insert(s.clone());
                    }
                    Value::Array(arr) => {
                        if let Some(path) = arr.get(0).and_then(Value::as_str) {
                            refs.insert(path.to_string());
                        }
                    }
                    _ => {}
                }
            }
            for val in map.values() {
                collect_refs(val, refs);
            }
        }
        Value::Array(arr) => {
            for val in arr {
                collect_refs(val, refs);
            }
        }
        _ => {}
    }
}

/// Check if a value contains any actionable schema keys recursively (with depth limit for arrays)
/// used to skip large pure-data arrays during schema walking
#[inline]
pub fn has_actionable_keys(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            if map.contains_key("$evaluation")
                || map.contains_key("$table")
                || map.contains_key("dependents")
                || map.contains_key("$layout")
            {
                return true;
            }

            // Check for conditional hidden/disabled fields
            if let Some(Value::Object(condition)) = map.get("condition") {
                if condition.contains_key("hidden") || condition.contains_key("disabled") {
                    return true;
                }
            }

            // Check for rules object
            if map.contains_key("rules") {
                return true;
            }

            // Check for type="array" with items (subforms)
            if let Some(Value::String(type_str)) = map.get("type") {
                if type_str == "array" && map.contains_key("items") {
                    return true;
                }
            }

            // Check for options with URL templates
            if let Some(Value::String(url)) = map.get("url") {
                if url.contains('{') && url.contains('}') {
                    return true;
                }
            }

            map.values().any(has_actionable_keys)
        }
        Value::Array(arr) => arr.iter().take(5).any(has_actionable_keys),
        _ => false,
    }
}

/// Compute forward/normal column partitions with transitive closure
///
/// This function identifies which columns have forward references (dependencies on later columns)
/// and separates them from normal columns for proper evaluation order.
pub fn compute_column_partitions(columns: &[ColumnMetadata]) -> (Vec<usize>, Vec<usize>) {
    use std::collections::HashSet;

    // Build set of all forward-referencing column names (direct + transitive)
    let mut fwd_cols = HashSet::new();
    for col in columns {
        if col.has_forward_ref {
            fwd_cols.insert(col.name.as_ref());
        }
    }

    // Transitive closure: any column that depends on forward columns is also forward
    loop {
        let mut changed = false;
        for col in columns {
            if !fwd_cols.contains(col.name.as_ref()) {
                // Check if this column depends on any forward column
                for dep in col.dependencies.iter() {
                    // Strip $ prefix from dependency name for comparison
                    let dep_name = dep.trim_start_matches('$');
                    if fwd_cols.contains(dep_name) {
                        fwd_cols.insert(col.name.as_ref());
                        changed = true;
                        break;
                    }
                }
            }
        }
        // Stop when no more changes
        if !changed {
            break;
        }
    }

    // Separate into forward and normal indices
    let mut forward_indices = Vec::new();
    let mut normal_indices = Vec::new();

    for (idx, col) in columns.iter().enumerate() {
        if fwd_cols.contains(col.name.as_ref()) {
            forward_indices.push(idx);
        } else {
            normal_indices.push(idx);
        }
    }

    (forward_indices, normal_indices)
}
