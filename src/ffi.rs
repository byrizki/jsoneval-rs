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

/// Result type for FFI operations
#[repr(C)]
pub struct FFIResult {
    pub success: bool,
    pub data: *mut c_char,
    pub error: *mut c_char,
}

impl Default for FFIResult {
    fn default() -> Self {
        Self {
            success: false,
            data: ptr::null_mut(),
            error: ptr::null_mut(),
        }
    }
}

impl FFIResult {
    fn success(data: String) -> Self {
        Self {
            success: true,
            data: CString::new(data).unwrap().into_raw(),
            error: ptr::null_mut(),
        }
    }

    fn error(msg: String) -> Self {
        Self {
            success: false,
            data: ptr::null_mut(),
            error: CString::new(msg).unwrap_or_else(|_| CString::new("Error message contains null byte").unwrap()).into_raw(),
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
            let result = eval.get_evaluated_schema(false);
            let result_str = serde_json::to_string(&result).unwrap_or_default();
            FFIResult::success(result_str)
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
            
            let result_str = serde_json::to_string(&result_json).unwrap_or_default();
            FFIResult::success(result_str)
        }
        Err(e) => FFIResult::error(e),
    }
}

/// Evaluate dependents after data changes
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - changed_paths must be a JSON array string
/// - data must be a valid null-terminated UTF-8 string
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_evaluate_dependents(
    handle: *mut JSONEvalHandle,
    changed_paths_json: *const c_char,
    data: *const c_char,
    context: *const c_char,
    nested: bool,
) -> FFIResult {
    if handle.is_null() || changed_paths_json.is_null() || data.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let paths_json = match CStr::from_ptr(changed_paths_json).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in paths".to_string())
        }
    };

    let paths: Vec<String> = match serde_json::from_str(paths_json) {
        Ok(p) => p,
        Err(_) => {
            return FFIResult::error("Invalid JSON array for paths".to_string())
        }
    };

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

    match eval.evaluate_dependents(&paths, data_str, context_str, nested) {
        Ok(result) => {
            let result_str = serde_json::to_string(&result).unwrap_or_default();
            FFIResult::success(result_str)
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
    let result_str = serde_json::to_string(&result).unwrap_or_default();
    
    FFIResult::success(result_str)
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
    let result_str = serde_json::to_string(&result).unwrap_or_default();
    
    FFIResult::success(result_str)
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
    let result_str = serde_json::to_string(&result).unwrap_or_default();
    
    FFIResult::success(result_str)
}

/// Get a value from the evaluated schema using dotted path notation
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - path must be a valid null-terminated UTF-8 string (dotted notation)
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_value_by_path(
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

    match eval.get_value_by_path(path_str, skip_layout) {
        Some(value) => {
            let result_str = serde_json::to_string(&value).unwrap_or_default();
            FFIResult::success(result_str)
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
    if !result.data.is_null() {
        drop(CString::from_raw(result.data));
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
        Ok(_) => FFIResult::success(String::new()),
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
    
    let result_str = serde_json::to_string(&stats_json).unwrap_or_default();
    
    FFIResult::success(result_str)
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
    
    FFIResult::success(String::new())
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
    
    FFIResult::success(result_str)
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
            
            let result_str = serde_json::to_string(&result_json).unwrap_or_default();
            FFIResult::success(result_str)
        }
        Err(e) => FFIResult::error(e),
    }
}

/// Get the library version
/// 
/// # Safety
/// 
/// - Caller must call json_eval_free_string when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_version() -> *mut c_char {
    CString::new(env!("CARGO_PKG_VERSION"))
        .unwrap()
        .into_raw()
}
