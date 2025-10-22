//! FFI evaluation functions

use std::ffi::CStr;
use std::os::raw::c_char;
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

/// Compile and run logic expression
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - logic_str must be a valid null-terminated UTF-8 string (JSON Logic)
/// - data can be NULL (uses existing data)
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_compile_and_run_logic(
    handle: *mut JSONEvalHandle,
    logic_str: *const c_char,
    data: *const c_char,
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

    match eval.compile_and_run_logic(logic, data_str) {
        Ok(result) => {
            let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}
