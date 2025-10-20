//! FFI (Foreign Function Interface) bindings for C#, C++, and other languages
//! 
//! This module provides a C-compatible API for the JSON evaluation library.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use crate::JSONEval;

/// Opaque pointer type for JSONEval instances
pub struct JSONEvalHandle {
    inner: Box<JSONEval>,
}

/// Zero-copy result type for FFI operations
/// 
/// # Zero-Copy Architecture
/// 
/// This structure implements true zero-copy data transfer across the FFI boundary:
/// 
/// 1. **Rust Side**: Serialized data (JSON/MessagePack) is allocated in a Vec<u8>
/// 2. **Boxing**: Vec is boxed and converted to raw pointer via Box::into_raw()
/// 3. **Transfer**: Raw pointer and length are passed to caller (NO COPY)
/// 4. **Caller Side**: Reads data directly from Rust-owned memory (NO COPY)
/// 5. **Cleanup**: Caller must call json_eval_free_result() to drop the Box
/// 
/// The data remains valid and owned by Rust until explicitly freed. The caller
/// accesses Rust's memory directly without any intermediate copies.
/// 
/// # Memory Layout
/// 
/// ```text
/// Rust Memory:        FFI Boundary:      Caller Memory:
/// +-----------+       data_ptr ------>  Direct read (zero-copy)
/// | Vec<u8>   |       data_len          No allocation needed
/// | [1,2,3..] |                         Marshal.Copy if needed
/// +-----------+
/// ```
#[repr(C)]
pub struct FFIResult {
    pub success: bool,
    pub data_ptr: *const u8,
    pub data_len: usize,
    pub error: *mut c_char,
    // Internal pointer to owned data for cleanup
    _owned_data: *mut Vec<u8>,
}

impl Default for FFIResult {
    fn default() -> Self {
        Self {
            success: false,
            data_ptr: ptr::null(),
            data_len: 0,
            error: ptr::null_mut(),
            _owned_data: ptr::null_mut(),
        }
    }
}

impl FFIResult {
    fn success(data: Vec<u8>) -> Self {
        let boxed_data = Box::new(data);
        let data_ptr = boxed_data.as_ptr();
        let data_len = boxed_data.len();
        Self {
            success: true,
            data_ptr,
            data_len,
            error: ptr::null_mut(),
            _owned_data: Box::into_raw(boxed_data),
        }
    }

    fn error(msg: String) -> Self {
        Self {
            success: false,
            data_ptr: ptr::null(),
            data_len: 0,
            error: CString::new(msg).unwrap_or_else(|_| CString::new("Error message contains null byte").unwrap()).into_raw(),
            _owned_data: ptr::null_mut(),
        }
    }
}

/// Get the library version
/// 
/// Returns a pointer to a static null-terminated string containing the version.
/// This pointer does not need to be freed.
#[no_mangle]
pub extern "C" fn json_eval_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Create a new JSONEval instance from MessagePack-encoded schema
/// 
/// # Safety
/// 
/// - schema_msgpack must be a valid pointer to MessagePack-encoded bytes
/// - schema_len must be the exact length of the MessagePack data
/// - context can be NULL for no context
/// - data can be NULL for no initial data
/// - Caller must call json_eval_free when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_new_from_msgpack(
    schema_msgpack: *const u8,
    schema_len: usize,
    context: *const c_char,
    data: *const c_char,
) -> *mut JSONEvalHandle {
    if schema_msgpack.is_null() || schema_len == 0 {
        eprintln!("[FFI ERROR] json_eval_new_from_msgpack: invalid schema pointer or length");
        return ptr::null_mut();
    }

    // Convert raw pointer to slice
    let schema_bytes = std::slice::from_raw_parts(schema_msgpack, schema_len);

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("[FFI ERROR] json_eval_new_from_msgpack: invalid UTF-8 in context: {}", e);
                return ptr::null_mut();
            }
        }
    } else {
        None
    };

    let data_str = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("[FFI ERROR] json_eval_new_from_msgpack: invalid UTF-8 in data: {}", e);
                return ptr::null_mut();
            }
        }
    } else {
        None
    };

    match JSONEval::new_from_msgpack(schema_bytes, context_str, data_str) {
        Ok(eval) => {
            let handle = Box::new(JSONEvalHandle {
                inner: Box::new(eval),
            });
            Box::into_raw(handle)
        }
        Err(e) => {
            let error_msg = format!("Failed to create JSONEval instance from MessagePack: {}", e);
            eprintln!("[FFI ERROR] json_eval_new_from_msgpack: {}", error_msg);
            ptr::null_mut()
        }
    }
}

/// Create a new JSONEval instance
/// 
/// # Safety
/// 
/// - schema must be a valid null-terminated UTF-8 string
/// - context can be NULL for no context
/// - data can be NULL for no initial data
/// - Caller must call json_eval_free when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_new(
    schema: *const c_char,
    context: *const c_char,
    data: *const c_char,
) -> *mut JSONEvalHandle {
    if schema.is_null() {
        eprintln!("[FFI ERROR] json_eval_new: schema pointer is null");
        return ptr::null_mut();
    }

    let schema_str = match CStr::from_ptr(schema).to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[FFI ERROR] json_eval_new: invalid UTF-8 in schema: {}", e);
            return ptr::null_mut();
        }
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("[FFI ERROR] json_eval_new: invalid UTF-8 in context: {}", e);
                return ptr::null_mut();
            }
        }
    } else {
        None
    };

    let data_str = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("[FFI ERROR] json_eval_new: invalid UTF-8 in data: {}", e);
                return ptr::null_mut();
            }
        }
    } else {
        None
    };

    match JSONEval::new(schema_str, context_str, data_str) {
        Ok(eval) => {
            let handle = Box::new(JSONEvalHandle {
                inner: Box::new(eval),
            });
            Box::into_raw(handle)
        }
        Err(e) => {
            let error_msg = format!("Failed to create JSONEval instance: {}", e);
            eprintln!("[FFI ERROR] json_eval_new: {}", error_msg);
            ptr::null_mut()
        }
    }
}

/// Create a new JSONEval instance with detailed error reporting
/// 
/// # Safety
/// 
/// - schema must be a valid null-terminated UTF-8 string
/// - context can be NULL for no context
/// - data can be NULL for no initial data
/// - error_out must be a valid pointer to store error message (caller owns the string)
/// - Returns non-null handle on success, null on failure (check error_out for details)
#[no_mangle]
pub unsafe extern "C" fn json_eval_new_with_error(
    schema: *const c_char,
    context: *const c_char,
    data: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut JSONEvalHandle {
    if schema.is_null() {
        if !error_out.is_null() {
            *error_out = CString::new("Schema pointer is null").unwrap().into_raw();
        }
        return ptr::null_mut();
    }

    let schema_str = match CStr::from_ptr(schema).to_str() {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                let msg = format!("Invalid UTF-8 in schema: {}", e);
                *error_out = CString::new(msg).unwrap().into_raw();
            }
            return ptr::null_mut();
        }
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                if !error_out.is_null() {
                    let msg = format!("Invalid UTF-8 in context: {}", e);
                    *error_out = CString::new(msg).unwrap().into_raw();
                }
                return ptr::null_mut();
            }
        }
    } else {
        None
    };

    let data_str = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                if !error_out.is_null() {
                    let msg = format!("Invalid UTF-8 in data: {}", e);
                    *error_out = CString::new(msg).unwrap().into_raw();
                }
                return ptr::null_mut();
            }
        }
    } else {
        None
    };

    match JSONEval::new(schema_str, context_str, data_str) {
        Ok(eval) => {
            let handle = Box::new(JSONEvalHandle {
                inner: Box::new(eval),
            });
            Box::into_raw(handle)
        }
        Err(e) => {
            if !error_out.is_null() {
                let msg = format!("Failed to create JSONEval instance: {}", e);
                *error_out = CString::new(msg).unwrap().into_raw();
            }
            ptr::null_mut()
        }
    }
}

/// Evaluate the schema with provided data
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - data must be a valid null-terminated UTF-8 string
/// - context can be NULL
/// - Caller must call json_eval_free_result when done with the result
#[no_mangle]
pub unsafe extern "C" fn json_eval_evaluate(
    handle: *mut JSONEvalHandle,
    data: *const c_char,
    context: *const c_char,
) -> FFIResult {
    if handle.is_null() || data.is_null() {
        return FFIResult::error("Invalid handle or data pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let data_str = match CStr::from_ptr(data).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in data".to_string())
        }
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                return FFIResult::error("Invalid UTF-8 in context".to_string())
            }
        }
    } else {
        None
    };

    match eval.evaluate(data_str, context_str) {
        Ok(_) => {
            // Don't serialize the schema here - massive performance waste!
            // C# can call get_evaluated_schema() explicitly if needed
            FFIResult::success(Vec::new())
        }
        Err(e) => FFIResult::error(e),
    }
}

/// Validate data against schema rules
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - data must be a valid null-terminated UTF-8 string
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_validate(
    handle: *mut JSONEvalHandle,
    data: *const c_char,
    context: *const c_char,
) -> FFIResult {
    if handle.is_null() || data.is_null() {
        return FFIResult::error("Invalid handle or data pointer".to_string());
    }

    let eval = &(*handle).inner;

    let data_str = match CStr::from_ptr(data).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in data".to_string())
        }
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                return FFIResult::error("Invalid UTF-8 in context".to_string())
            }
        }
    } else {
        None
    };

    match eval.validate(data_str, context_str, None) {
        Ok(validation_result) => {
            let result_json = serde_json::json!({
                "hasError": validation_result.has_error,
                "errors": validation_result.errors.iter().map(|(k, v)| {
                    serde_json::json!({
                        "path": k,
                        "ruleType": v.rule_type,
                        "message": v.message
                    })
                }).collect::<Vec<_>>()
            });
            
            let result_bytes = serde_json::to_vec(&result_json).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}

/// Evaluate dependents (fields that depend on a changed path)
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - changed_path must be a valid null-terminated UTF-8 string
/// - data can be null (uses existing data)
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_evaluate_dependents(
    handle: *mut JSONEvalHandle,
    changed_path: *const c_char,
    data: *const c_char,
    context: *const c_char,
) -> FFIResult {
    if handle.is_null() || changed_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let path_str = match CStr::from_ptr(changed_path).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in path".to_string())
        }
    };

    let data_str = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                return FFIResult::error("Invalid UTF-8 in data".to_string())
            }
        }
    } else {
        None
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                return FFIResult::error("Invalid UTF-8 in context".to_string())
            }
        }
    } else {
        None
    };

    match eval.evaluate_dependents(path_str, data_str, context_str) {
        Ok(result) => {
            let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}

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

/// Free an FFIResult
/// 
/// # Safety
/// 
/// - result must be a valid FFIResult from one of the evaluate functions
/// - result should not be used after calling this function
#[no_mangle]
pub unsafe extern "C" fn json_eval_free_result(result: FFIResult) {
    if !result._owned_data.is_null() {
        drop(Box::from_raw(result._owned_data));
    }
    if !result.error.is_null() {
        drop(CString::from_raw(result.error));
    }
}

/// Free a string returned by the library
/// 
/// # Safety
/// 
/// - ptr must be a valid pointer from a library function
#[no_mangle]
pub unsafe extern "C" fn json_eval_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Free a JSONEval instance
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - handle should not be used after calling this function
#[no_mangle]
pub unsafe extern "C" fn json_eval_free(handle: *mut JSONEvalHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Reload schema with new data
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - schema must be a valid null-terminated UTF-8 string
/// - context and data can be NULL
#[no_mangle]
pub unsafe extern "C" fn json_eval_reload_schema(
    handle: *mut JSONEvalHandle,
    schema: *const c_char,
    context: *const c_char,
    data: *const c_char,
) -> FFIResult {
    if handle.is_null() || schema.is_null() {
        return FFIResult::error("Invalid handle or schema pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let schema_str = match CStr::from_ptr(schema).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in schema".to_string())
        }
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                return FFIResult::error("Invalid UTF-8 in context".to_string())
            }
        }
    } else {
        None
    };

    let data_str = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                return FFIResult::error("Invalid UTF-8 in data".to_string())
            }
        }
    } else {
        None
    };

    match eval.reload_schema(schema_str, context_str, data_str) {
        Ok(_) => FFIResult::success(Vec::new()),
        Err(e) => FFIResult::error(e),
    }
}

/// Get cache statistics
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_cache_stats(
    handle: *mut JSONEvalHandle,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &(*handle).inner;
    let stats = eval.cache_stats();
    
    let stats_json = serde_json::json!({
        "hits": stats.hits,
        "misses": stats.misses,
        "entries": stats.entries,
    });
    
    let result_bytes = serde_json::to_vec(&stats_json).unwrap_or_default();
    
    FFIResult::success(result_bytes)
}

/// Clear the evaluation cache
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
#[no_mangle]
pub unsafe extern "C" fn json_eval_clear_cache(
    handle: *mut JSONEvalHandle,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &mut (*handle).inner;
    eval.clear_cache();
    
    FFIResult::success(Vec::new())
}

/// Get the number of cached entries
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_cache_len(
    handle: *mut JSONEvalHandle,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &(*handle).inner;
    let len = eval.cache_len();
    
    let result_str = len.to_string();
    let result_bytes = result_str.into_bytes();
    
    FFIResult::success(result_bytes)
}

/// Validate data against schema rules with optional path filtering
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - data must be a valid null-terminated UTF-8 string
/// - paths_json can be NULL for no filtering, or a JSON array string
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_validate_paths(
    handle: *mut JSONEvalHandle,
    data: *const c_char,
    context: *const c_char,
    paths_json: *const c_char,
) -> FFIResult {
    if handle.is_null() || data.is_null() {
        return FFIResult::error("Invalid handle or data pointer".to_string());
    }

    let eval = &(*handle).inner;

    let data_str = match CStr::from_ptr(data).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in data".to_string())
        }
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                return FFIResult::error("Invalid UTF-8 in context".to_string())
            }
        }
    } else {
        None
    };

    let paths: Option<Vec<String>> = if !paths_json.is_null() {
        let paths_str = match CStr::from_ptr(paths_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                return FFIResult::error("Invalid UTF-8 in paths".to_string())
            }
        };
        
        match serde_json::from_str(paths_str) {
            Ok(p) => Some(p),
            Err(_) => {
                return FFIResult::error("Invalid JSON array for paths".to_string())
            }
        }
    } else {
        None
    };

    let paths_ref = paths.as_ref().map(|v| v.as_slice());

    match eval.validate(data_str, context_str, paths_ref) {
        Ok(validation_result) => {
            let result_json = serde_json::json!({
                "hasError": validation_result.has_error,
                "errors": validation_result.errors.iter().map(|(k, v)| {
                    serde_json::json!({
                        "path": k,
                        "ruleType": v.rule_type,
                        "message": v.message
                    })
                }).collect::<Vec<_>>()
            });
            
            let result_bytes = serde_json::to_vec(&result_json).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}

/// Resolve layout with optional evaluation
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - evaluate: if true, runs evaluation before resolving layout
#[no_mangle]
pub unsafe extern "C" fn json_eval_resolve_layout(
    handle: *mut JSONEvalHandle,
    evaluate: bool,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &mut (*handle).inner;
    match eval.resolve_layout(evaluate) {
        Ok(_) => FFIResult::success(Vec::new()),
        Err(e) => FFIResult::error(e),
    }
}

/// Compile and run JSON logic from a JSON logic string
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - logic_str must be a valid null-terminated UTF-8 string containing JSON logic
/// - data can be NULL (uses existing data)
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_compile_and_run_logic(
    handle: *mut JSONEvalHandle,
    logic_str: *const c_char,
    data: *const c_char,
) -> FFIResult {
    if handle.is_null() || logic_str.is_null() {
        return FFIResult::error("Invalid handle or logic_str pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let logic = match CStr::from_ptr(logic_str).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in logic_str".to_string())
        }
    };

    let data_str = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                return FFIResult::error("Invalid UTF-8 in data".to_string())
            }
        }
    } else {
        None
    };

    match eval.compile_and_run_logic(logic, data_str) {
        Ok(result) => {
            let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}

