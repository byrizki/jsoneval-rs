//! FFI evaluation functions

use std::ffi::CStr;
use std::os::raw::c_char;
use serde_json::json;
use super::types::{FFIResult, JSONEvalHandle};

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

    match eval.validate(data_str, context_str, None) {
        Ok(validation_result) => {
            let result_json = serde_json::json!({
                "hasError": validation_result.has_error,
                "errors": validation_result.errors.iter().map(|(k, v)| {
                    let mut error_obj = serde_json::json!({
                        "path": k,
                        "type": v.rule_type,
                        "message": v.message
                    });
                    
                    if let Some(code) = &v.code {
                        error_obj["code"] = json!(code);
                    }
                    if let Some(pattern) = &v.pattern {
                        error_obj["pattern"] = json!(pattern);
                    }
                    if let Some(field_value) = &v.field_value {
                        error_obj["fieldValue"] = json!(field_value);
                    }
                    if let Some(data) = &v.data {
                        error_obj["data"] = data.clone();
                    }
                    
                    error_obj
                }).collect::<Vec<_>>()
            });
            
            let result_bytes = serde_json::to_vec(&result_json).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}

/// Evaluate dependents (fields that depend on changed paths)
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - changed_paths_json must be a valid null-terminated UTF-8 string containing a JSON array of paths
/// - data can be null (uses existing data)
/// - re_evaluate: 0 = false, non-zero = true
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_evaluate_dependents(
    handle: *mut JSONEvalHandle,
    changed_paths_json: *const c_char,
    data: *const c_char,
    context: *const c_char,
    re_evaluate: i32,
) -> FFIResult {
    if handle.is_null() || changed_paths_json.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let paths_json_str = match CStr::from_ptr(changed_paths_json).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in paths".to_string())
        }
    };

    // Parse JSON array of paths
    let paths: Vec<String> = match serde_json::from_str(paths_json_str) {
        Ok(p) => p,
        Err(e) => {
            return FFIResult::error(format!("Failed to parse paths JSON: {}", e))
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

    match eval.evaluate_dependents(&paths, data_str, context_str, re_evaluate != 0) {
        Ok(result) => {
            let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}

/// Compile and run logic expression
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - logic_str must be a valid null-terminated UTF-8 string (JSON Logic)
/// - data can be NULL (uses existing data)
/// - context can be NULL (uses existing context)
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_compile_and_run_logic(
    handle: *mut JSONEvalHandle,
    logic_str: *const c_char,
    data: *const c_char,
    context: *const c_char,
) -> FFIResult {
    if handle.is_null() || logic_str.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let logic = match CStr::from_ptr(logic_str).to_str() {
        Ok(s) => s,
        Err(_) => {
            return FFIResult::error("Invalid UTF-8 in logic".to_string())
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

    match eval.compile_and_run_logic(logic, data_str, context_str) {
        Ok(result) => {
            let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}
