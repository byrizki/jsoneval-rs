//! FFI schema getter functions

use std::ffi::CStr;
use std::os::raw::c_char;
use super::types::{FFIResult, JSONEvalHandle};

/// Get the evaluated schema with optional layout resolution
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_evaluated_schema(
    handle: *mut JSONEvalHandle,
    skip_layout: bool,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &mut (*handle).inner;
    let result = eval.get_evaluated_schema(skip_layout);
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    
    FFIResult::success(result_bytes)
}

/// Get the evaluated schema in MessagePack format with optional layout resolution
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - Caller must call json_eval_free_result when done
/// 
/// # Zero-Copy Optimization
/// 
/// This function implements zero-copy data transfer:
/// 1. Serializes evaluated schema to MessagePack Vec<u8> (unavoidable)
/// 2. Returns raw pointer to this data via FFIResult (zero-copy)
/// 3. Caller reads directly from Rust memory (zero-copy)
/// 4. Single Marshal.Copy on caller side if needed (one copy total)
/// 
/// The MessagePack binary format is typically 20-50% smaller than JSON,
/// making it ideal for performance-critical scenarios.
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_evaluated_schema_msgpack(
    handle: *mut JSONEvalHandle,
    skip_layout: bool,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &mut (*handle).inner;
    match eval.get_evaluated_schema_msgpack(skip_layout) {
        Ok(msgpack_bytes) => FFIResult::success(msgpack_bytes),
        Err(e) => FFIResult::error(e),
    }
}

/// Get all schema values (evaluations ending with .value)
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_schema_value(
    handle: *mut JSONEvalHandle,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &mut (*handle).inner;
    let result = eval.get_schema_value();
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    
    FFIResult::success(result_bytes)
}

/// Get the evaluated schema without $params field
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_evaluated_schema_without_params(
    handle: *mut JSONEvalHandle,
    skip_layout: bool,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &mut (*handle).inner;
    let result = eval.get_evaluated_schema_without_params(skip_layout);
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    
    FFIResult::success(result_bytes)
}

/// Get a value from the evaluated schema using dotted path notation
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - path must be a valid null-terminated UTF-8 string (dotted notation)
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_evaluated_schema_by_path(
    handle: *mut JSONEvalHandle,
    path: *const c_char,
    skip_layout: bool,
) -> FFIResult {
    if handle.is_null() || path.is_null() {
        return FFIResult::error("Invalid handle or path pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in path".to_string())
        }
    };

    match eval.get_evaluated_schema_by_path(path_str, skip_layout) {
        Some(value) => {
            let result_bytes = serde_json::to_vec(&value).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        None => FFIResult::error("Path not found".to_string()),
    }
}

/// Get values from the evaluated schema using multiple dotted path notations
/// Returns a merged object containing all requested paths (skips paths that are not found)
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - paths_json must be a valid null-terminated UTF-8 string containing a JSON array of paths
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_evaluated_schema_by_paths(
    handle: *mut JSONEvalHandle,
    paths_json: *const c_char,
    skip_layout: bool,
) -> FFIResult {
    if handle.is_null() || paths_json.is_null() {
        return FFIResult::error("Invalid handle or paths pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let paths_str = match CStr::from_ptr(paths_json).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in paths".to_string())
        }
    };

    // Parse JSON array of paths
    let paths: Vec<String> = match serde_json::from_str(paths_str) {
        Ok(p) => p,
        Err(e) => {
            return FFIResult::error(format!("Failed to parse paths JSON: {}", e))
        }
    };

    let result = eval.get_evaluated_schema_by_paths(&paths, skip_layout);
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    FFIResult::success(result_bytes)
}

/// Get a value from the schema using dotted path notation
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - path must be a valid null-terminated UTF-8 string (dotted notation)
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_schema_by_path(
    handle: *mut JSONEvalHandle,
    path: *const c_char,
) -> FFIResult {
    if handle.is_null() || path.is_null() {
        return FFIResult::error("Invalid handle or path pointer".to_string());
    }

    let eval = &(*handle).inner;

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in path".to_string())
        }
    };

    match eval.get_schema_by_path(path_str) {
        Some(value) => {
            let result_bytes = serde_json::to_vec(&value).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        None => FFIResult::error("Path not found".to_string()),
    }
}
