use crate::jsoneval::path_utils;
use serde_json::{Number, Value};

/// Convert f64 to JSON number
#[inline(always)]
pub fn f64_to_json(f: f64, safe_nan_handling: bool) -> Value {
    if f.is_finite() {
        // Check if it's an integer value (within safe precision range)
        if f == f.floor() && f.abs() < 9007199254740991.0 {
            // MAX_SAFE_INTEGER
            return Value::Number(Number::from(f as i64));
        }
        Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null)
    } else if safe_nan_handling {
        Value::Number(Number::from(0))
    } else {
        Value::Null
    }
}

/// Convert JSON value to f64
#[inline(always)]
pub fn to_f64(value: &Value) -> f64 {
    match value {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::Bool(true) => 1.0,
        Value::Bool(false) => 0.0,
        Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        Value::Array(arr) => {
            if arr.len() == 1 {
                to_f64(&arr[0])
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

/// Legacy to_number for f64 (only used for Power operations)
#[inline]
pub fn to_number(value: &Value) -> f64 {
    to_f64(value)
}

/// Helper to parse string to f64 (empty string = 0.0)
#[inline]
pub fn parse_string_to_f64(s: &str) -> Option<f64> {
    if s.is_empty() {
        Some(0.0)
    } else {
        s.parse::<f64>().ok()
    }
}

/// Convert value to string
#[inline]
pub fn to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            // JavaScript-like number to string conversion:
            // Integer-valued numbers should not have decimal point
            if let Some(f) = n.as_f64() {
                if f.is_finite() && f == f.floor() && f.abs() < 1e15 {
                    // It's an integer value, format without decimal
                    format!("{}", f as i64)
                } else {
                    n.to_string()
                }
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

/// Use centralized path normalization for consistent $ref/var handling
#[inline]
pub fn normalize_ref_path(path: &str) -> String {
    path_utils::normalize_to_json_pointer(path).into_owned()
}

use crate::rlogic::Evaluator;

impl Evaluator {
    /// OPTIMIZED: Fast variable access - paths are pre-normalized during compilation
    ///
    /// Resolution order:
    /// 1. Active table scope (for self-table references during table evaluation)
    /// 2. Static arrays store (for large pre-extracted $params arrays)
    /// 3. Normal JSON pointer access on data
    /// 4. $static_array marker transparency
    #[inline(always)]
    pub(crate) fn get_var<'a>(&'a self, data: &'a Value, name: &str) -> Option<&'a Value> {
        if name.is_empty() {
            return Some(data);
        }

        // Fast intercept for $iteration and $threshold in active table scope
        if name == "/$iteration" || name == "$iteration" {
            // SAFETY: single-threaded (eval_lock held during this scope), UnsafeCell
            let scope = unsafe { &*self.table_scope.get() };
            if let Some(ts) = scope.as_ref() {
                if let Some(ref val) = ts.iteration_val {
                    return Some(val);
                }
            }
        } else if name == "/$threshold" || name == "$threshold" {
            // SAFETY: single-threaded (eval_lock held during this scope), UnsafeCell
            let scope = unsafe { &*self.table_scope.get() };
            if let Some(ts) = scope.as_ref() {
                if let Some(ref val) = ts.threshold_val {
                    return Some(val);
                }
            }
        } else if name == "/$loopIteration" || name == "$loopIteration" {
            if let Value::Object(obj) = data {
                if let Some(val) = obj.get("$loopIteration") {
                    return Some(val);
                }
            }
        }

        // Fast intercept for self-table current row reference
        // Column references are of the form "/$COLUMN_NAME" or "$COLUMN_NAME" (single segment, no slashes)
        let col_ref = if name.starts_with("/$") && !name[2..].contains('/') {
            Some(&name[2..])
        } else if name.starts_with('$') && !name.starts_with("/$") && !name[1..].contains('/') {
            Some(&name[1..])
        } else {
            None
        };

        if let Some(field) = col_ref {
            // SAFETY: single-threaded (eval_lock held during this scope), UnsafeCell
            let scope = unsafe { &*self.table_scope.get() };
            if let Some(ts) = scope.as_ref() {
                // Ultra-fast path: direct flat_cells lookup via precomputed current_row_base
                if !ts.current_row_base.is_null() {
                    if let Some(col_idx) = ts.get_col_idx(field) {
                        let cell = unsafe { &*ts.current_row_base.add(col_idx) };
                        return Some(cell);
                    }
                }
                if let Some(row_idx) = ts.current_row {
                    // Fallback for flat_cells if current_row_base was null
                    if ts.col_count > 0 && !ts.flat_cells.is_null() {
                        if row_idx >= ts.existing_row_count
                            && row_idx < ts.existing_row_count + ts.total_rows
                        {
                            if let Some(col_idx) = ts.get_col_idx(field) {
                                let row_offset = row_idx - ts.existing_row_count;
                                let cell = unsafe {
                                    &*ts.flat_cells.add(row_offset * ts.col_count + col_idx)
                                };
                                return Some(cell);
                            }
                        }
                    }
                    // Fallback for static rows
                    // SAFETY: local_rows outlives this evaluation frame
                    let rows = unsafe { &*ts.rows };
                    if let Some(row) = rows.get(row_idx) {
                        if let Value::Object(obj) = row {
                            if let Some(cell) = obj.get(field) {
                                return Some(cell);
                            }
                        }
                    }
                }
            }
        }

        // Fast intercept for static arrays to handle deep lookup paths
        // e.g., name = "/$params/references/WOP_BENEFIT" or "/$params/R_PROD_RIDER/0/PLAN_NAME" or "/$table/..."
        let static_arrays = unsafe { &*self.static_arrays.get() };
        if let Some(arrays) = static_arrays {
            if name.starts_with("/$params/references/") && name.len() > 20 {
                let end_idx = name[20..].find('/').map(|i| i + 20).unwrap_or(name.len());
                let base_path = &name[..end_idx];
                if let Some(arc_val) = arrays.get(base_path) {
                    if end_idx == name.len() {
                        return Some(&**arc_val);
                    } else {
                        let remainder = &name[end_idx..];
                        return path_utils::get_value_by_pointer_without_properties(
                            arc_val, remainder,
                        );
                    }
                }
            } else if name.starts_with("/$params/") && name.len() > 9 {
                let end_idx = name[9..].find('/').map(|i| i + 9).unwrap_or(name.len());
                let base_path = &name[..end_idx];
                if let Some(arc_val) = arrays.get(base_path) {
                    if end_idx == name.len() {
                        return Some(&**arc_val);
                    } else {
                        let remainder = &name[end_idx..];
                        return path_utils::get_value_by_pointer_without_properties(
                            arc_val, remainder,
                        );
                    }
                }
            } else {
                // Table static array intercept: e.g. POL_TABLE, /properties/POL_TABLE, or subpaths
                for (static_key, arc_val) in arrays.iter() {
                    if let Some(table_path) = static_key.strip_prefix("/$table") {
                        let path_no_prop =
                            table_path.strip_prefix("/properties").unwrap_or(table_path);
                        let raw_name = path_no_prop.trim_start_matches('/');
                        let matched_len = if name.starts_with(table_path) {
                            Some(table_path.len())
                        } else if name.starts_with(path_no_prop) {
                            Some(path_no_prop.len())
                        } else if name.starts_with(raw_name) {
                            Some(raw_name.len())
                        } else {
                            None
                        };
                        if let Some(prefix_len) = matched_len {
                            if name.len() == prefix_len {
                                return Some(&**arc_val);
                            } else if name.as_bytes().get(prefix_len) == Some(&b'/') {
                                let remainder = &name[prefix_len..];
                                return path_utils::get_value_by_pointer_without_properties(
                                    arc_val, remainder,
                                );
                            }
                        }
                    }
                }
            }
        }

        // Direct JSON pointer access without re-normalization
        let val = path_utils::get_value_by_pointer_without_properties(data, name);

        // Resolve $static_array transparency for exact marker matches
        if let Some(v) = val {
            if let Some(obj) = v.as_object() {
                if let Some(Value::String(static_path)) = obj.get("$static_array") {
                    if let Some(arrays) = static_arrays {
                        if let Some(arc_val) = arrays.get(static_path) {
                            return Some(&**arc_val);
                        }
                    }
                }
            }
        }
        val
    }

    /// Check if key is missing (null or not present)
    #[inline]
    pub(crate) fn is_key_missing(&self, data: &Value, key: &str) -> bool {
        if key.is_empty() {
            return false;
        }
        let pointer = path_utils::normalize_to_json_pointer(key);
        if pointer.is_empty() {
            return false;
        }
        self.get_var(data, &pointer)
            .map(|v| v.is_null())
            .unwrap_or(true)
    }
}

/// Check if value is truthy
#[inline]
pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Array(arr) => !arr.is_empty(),
        Value::Object(_) => true,
    }
}

/// Check if value is null-like (Null, empty string, or NaN)
#[inline]
pub fn is_null_like(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) if s.is_empty() => true,
        Value::Number(n) if n.is_f64() && n.as_f64().unwrap().is_nan() => true,
        _ => false,
    }
}

/// Build ISO date string from NaiveDate
#[inline]
pub fn build_iso_date_string(date: chrono::NaiveDate) -> String {
    let mut result = String::with_capacity(24);
    result.push_str(&date.format("%Y-%m-%d").to_string());
    result.push_str("T00:00:00.000Z");
    result
}

/// Compare two values as numbers
#[inline]
pub fn compare(a: &Value, b: &Value) -> f64 {
    let num_a = to_f64(a);
    let num_b = to_f64(b);
    num_a - num_b
}

/// Create option object from label and value
#[inline]
pub fn create_option(label: &Value, value: &Value) -> Value {
    serde_json::json!({"label": label, "value": value})
}

/// Generate scalar hash key for Value (used in In operations)
#[inline]
pub fn scalar_hash_key(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some(String::from("null")),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

/// JavaScript-like loose equality
pub fn loose_equal(a: &Value, b: &Value) -> bool {
    // JavaScript-like type coercion for loose equality (==)
    match (a, b) {
        // Same type comparisons
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => {
            let a_f64 = a.as_f64().unwrap_or(0.0);
            let b_f64 = b.as_f64().unwrap_or(0.0);
            a_f64 == b_f64
        }
        (Value::String(a), Value::String(b)) => a == b,

        // Number and String: convert string to number
        (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
            let n_val = n.as_f64().unwrap_or(0.0);
            parse_string_to_f64(s)
                .map(|parsed| n_val == parsed)
                .unwrap_or(false)
        }

        // Boolean and Number: convert boolean to number (true=1, false=0)
        (Value::Bool(b), Value::Number(n)) | (Value::Number(n), Value::Bool(b)) => {
            let b_num = if *b { 1.0 } else { 0.0 };
            b_num == n.as_f64().unwrap_or(0.0)
        }

        // Boolean and String: convert both to number
        (Value::Bool(b), Value::String(s)) | (Value::String(s), Value::Bool(b)) => {
            let b_num = if *b { 1.0 } else { 0.0 };
            parse_string_to_f64(s)
                .map(|parsed| b_num == parsed)
                .unwrap_or(false)
        }

        // Null comparisons: null only equals null (and undefined, but we don't have that)
        (Value::Null, _) | (_, Value::Null) => false,

        // Default: strict equality
        _ => a == b,
    }
}
