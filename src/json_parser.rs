/// Hybrid SIMD-accelerated JSON parser with fallback to serde_json
/// Uses simd-json for maximum performance on large JSON data
use serde_json::Value;

/// Parse JSON from a string slice using SIMD acceleration when possible
/// Falls back to serde_json for compatibility
#[inline]
pub fn parse_json_str(json: &str) -> Result<Value, String> {
    // Try SIMD-JSON first for performance
    parse_simd_str(json).or_else(|_| {
        // Fallback to serde_json if SIMD fails
        serde_json::from_str(json).map_err(|e| e.to_string())
    })
}

/// Parse JSON from bytes using SIMD acceleration when possible
/// This is the fastest path as simd-json works on mutable byte slices
#[inline]
pub fn parse_json_bytes(mut bytes: Vec<u8>) -> Result<Value, String> {
    // SIMD-JSON requires mutable bytes for in-place parsing
    parse_simd_bytes(&mut bytes).or_else(|_| {
        // Fallback to serde_json
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    })
}

/// Internal SIMD parser for string slices
#[inline]
fn parse_simd_str(json: &str) -> Result<Value, String> {
    let mut bytes = json.as_bytes().to_vec();
    parse_simd_bytes(&mut bytes)
}

/// Internal SIMD parser for mutable byte slices (fastest path)
#[inline]
fn parse_simd_bytes(bytes: &mut [u8]) -> Result<Value, String> {
    // Use simd-json for SIMD-accelerated parsing
    match simd_json::to_borrowed_value(bytes) {
        Ok(borrowed_value) => {
            // Convert simd_json::BorrowedValue to serde_json::Value
            // This is efficient as simd-json value types are compatible
            convert_simd_to_serde(&borrowed_value)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Convert simd_json::BorrowedValue to serde_json::Value
/// Optimized for zero-copy where possible
#[inline]
fn convert_simd_to_serde(value: &simd_json::BorrowedValue) -> Result<Value, String> {
    use simd_json::prelude::*;

    match value.value_type() {
        simd_json::ValueType::Null => Ok(Value::Null),
        simd_json::ValueType::Bool => Ok(Value::Bool(value.as_bool().unwrap_or(false))),
        simd_json::ValueType::I64 => {
            if let Some(i) = value.as_i64() {
                Ok(serde_json::Number::from(i).into())
            } else {
                Ok(Value::Null)
            }
        }
        simd_json::ValueType::I128 => {
            // For i128, convert to f64 as serde_json doesn't support i128 directly
            if let Some(i) = value.as_i128() {
                Ok(serde_json::Number::from_f64(i as f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }
        simd_json::ValueType::U64 => {
            if let Some(u) = value.as_u64() {
                Ok(serde_json::Number::from(u).into())
            } else {
                Ok(Value::Null)
            }
        }
        simd_json::ValueType::U128 => {
            // For u128, convert to f64 as serde_json doesn't support u128 directly
            if let Some(u) = value.as_u128() {
                Ok(serde_json::Number::from_f64(u as f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            }
        }
        simd_json::ValueType::F64 => {
            if let Some(f) = value.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .ok_or_else(|| "Invalid float value".to_string())
            } else {
                Ok(Value::Null)
            }
        }
        simd_json::ValueType::String => {
            if let Some(s) = value.as_str() {
                Ok(Value::String(s.to_string()))
            } else {
                Ok(Value::Null)
            }
        }
        simd_json::ValueType::Array => {
            if let Some(arr) = value.as_array() {
                let mut result = Vec::with_capacity(arr.len());
                for item in arr {
                    result.push(convert_simd_to_serde(item)?);
                }
                Ok(Value::Array(result))
            } else {
                Ok(Value::Array(Vec::new()))
            }
        }
        simd_json::ValueType::Object => {
            if let Some(obj) = value.as_object() {
                let mut result = serde_json::Map::with_capacity(obj.len());
                for (key, val) in obj.iter() {
                    result.insert(key.to_string(), convert_simd_to_serde(val)?);
                }
                Ok(Value::Object(result))
            } else {
                Ok(Value::Object(serde_json::Map::new()))
            }
        }
        simd_json::ValueType::Extended(_) => {
            // Extended types - fallback to null
            Ok(Value::Null)
        }
        _ => {
            // Any other types - fallback to null
            Ok(Value::Null)
        }
    }
}

/// Read JSON file using SIMD acceleration
/// Reads file into memory and uses fast SIMD parsing
pub fn read_json_file(path: &str) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read file {}: {}", path, e))?;
    parse_json_bytes(bytes)
}
