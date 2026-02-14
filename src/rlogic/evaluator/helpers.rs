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

/// OPTIMIZED: Fast variable access - paths are pre-normalized during compilation
/// This method is now only used for legacy/fallback cases
#[inline(always)]
pub fn get_var<'a>(data: &'a Value, name: &str) -> Option<&'a Value> {
    if name.is_empty() {
        return Some(data);
    }

    // OPTIMIZED: Assume path is already normalized (from compilation)
    // Direct JSON pointer access without re-normalization
    path_utils::get_value_by_pointer_without_properties(data, name)
}

/// Get variable with layered context (primary first, then fallback)
#[inline]
pub fn get_var_layered<'a>(
    primary: &'a Value,
    fallback: &'a Value,
    name: &str,
) -> Option<&'a Value> {
    get_var(primary, name).or_else(|| get_var(fallback, name))
}

/// Check if key is missing (null or not present)
#[inline]
pub fn is_key_missing(data: &Value, key: &str) -> bool {
    if key.is_empty() {
        return false;
    }
    let pointer = path_utils::normalize_to_json_pointer(key);
    if pointer.is_empty() {
        return false;
    }
    get_var(data, &pointer).map(|v| v.is_null()).unwrap_or(true)
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
