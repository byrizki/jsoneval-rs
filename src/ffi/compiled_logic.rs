//! FFI functions for compiled logic operations

use super::types::{FFIResult, JSONEvalHandle};
use std::ffi::CStr;
use std::os::raw::c_char;

/// Compile logic and return a global ID
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - logic_str must be a valid null-terminated UTF-8 string (JSON Logic)
/// - Returns 0 on error (check error via json_eval_get_last_error if needed)
#[no_mangle]
pub unsafe extern "C" fn json_eval_compile_logic(
    handle: *mut JSONEvalHandle,
    logic_str: *const c_char,
) -> u64 {
    if handle.is_null() || logic_str.is_null() {
        return 0;
    }

    let eval = &(*handle).inner;

    let logic = match CStr::from_ptr(logic_str).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    match eval.compile_logic(logic) {
        Ok(id) => id.as_u64(),
        Err(_) => 0,
    }
}

/// Run pre-compiled logic by ID
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - logic_id must be a valid ID from json_eval_compile_logic
/// - data can be NULL (uses existing data)
/// - context can be NULL (uses existing context)
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_run_logic(
    handle: *mut JSONEvalHandle,
    logic_id: u64,
    data: *const c_char,
    context: *const c_char,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &mut (*handle).inner;
    let id = crate::CompiledLogicId::from_u64(logic_id);

    let data_value = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => match crate::jsoneval::json_parser::parse_json_str(s) {
                Ok(v) => Some(v),
                Err(e) => return FFIResult::error(format!("Failed to parse data: {}", e)),
            },
            Err(_) => return FFIResult::error("Invalid UTF-8 in data".to_string()),
        }
    } else {
        None
    };

    let context_value = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => match crate::jsoneval::json_parser::parse_json_str(s) {
                Ok(v) => Some(v),
                Err(e) => return FFIResult::error(format!("Failed to parse context: {}", e)),
            },
            Err(_) => return FFIResult::error("Invalid UTF-8 in context".to_string()),
        }
    } else {
        None
    };

    match eval.run_logic(id, data_value.as_ref(), context_value.as_ref()) {
        Ok(result) => {
            let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}

/// Run logic evaluation without an instance handle (pure/static)
/// 
/// # Safety
/// 
/// - logic_str must be a valid null-terminated UTF-8 string (JSON Logic)
/// - data can be NULL
/// - context can be NULL
/// - Caller must call json_eval_free_result when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_evaluate_logic_pure(
    logic_str: *const c_char,
    data: *const c_char,
    context: *const c_char,
) -> FFIResult {
    if logic_str.is_null() {
        return FFIResult::error("Logic string is null".to_string());
    }

    let logic = match CStr::from_ptr(logic_str).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in logic".to_string()),
    };

    let data_str = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => Some(s),
            Err(_) => return FFIResult::error("Invalid UTF-8 in data".to_string()),
        }
    } else {
        None
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(_) => return FFIResult::error("Invalid UTF-8 in context".to_string()),
        }
    } else {
        None
    };

    match crate::jsoneval::logic::evaluate_logic_pure(logic, data_str, context_str) {
        Ok(result) => {
             let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
             FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}
