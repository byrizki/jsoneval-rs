/// Hybrid SIMD-accelerated JSON parser with fallback to serde_json
/// Uses simd-json's serde integration for direct deserialization into serde_json::Value
use serde_json::Value;

/// Parse JSON from a string slice using SIMD acceleration when possible
/// Falls back to serde_json for compatibility
#[inline]
pub fn parse_json_str(json: &str) -> Result<Value, String> {
    let mut bytes = json.as_bytes().to_vec();
    parse_simd_bytes(&mut bytes).or_else(|_| {
        serde_json::from_str(json).map_err(|e| e.to_string())
    })
}

/// Parse JSON from bytes using SIMD acceleration when possible
/// This is the fastest path as simd-json works on mutable byte slices
#[inline]
pub fn parse_json_bytes(mut bytes: Vec<u8>) -> Result<Value, String> {
    parse_simd_bytes(&mut bytes).or_else(|_| {
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    })
}

/// Internal SIMD parser — deserializes directly into serde_json::Value
/// via simd-json's serde integration (no intermediate BorrowedValue)
#[inline]
fn parse_simd_bytes(bytes: &mut [u8]) -> Result<Value, String> {
    simd_json::serde::from_slice(bytes).map_err(|e| e.to_string())
}

/// Read JSON file using SIMD acceleration
/// Reads file into memory and uses fast SIMD parsing
pub fn read_json_file(path: &str) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read file {}: {}", path, e))?;
    parse_json_bytes(bytes)
}

