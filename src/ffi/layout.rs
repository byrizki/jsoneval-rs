//! FFI layout and validation functions

use super::types::{FFIResult, JSONEvalHandle};
use std::ffi::CStr;
use std::os::raw::c_char;

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

    let eval = &mut (*handle).inner;

    let data_str = match CStr::from_ptr(data).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in data".to_string()),
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(_) => return FFIResult::error("Invalid UTF-8 in context".to_string()),
        }
    } else {
        None
    };

    let paths: Option<Vec<String>> = if !paths_json.is_null() {
        let paths_str = match CStr::from_ptr(paths_json).to_str() {
            Ok(s) => s,
            Err(_) => return FFIResult::error("Invalid UTF-8 in paths".to_string()),
        };

        match serde_json::from_str(paths_str) {
            Ok(p) => Some(p),
            Err(_) => return FFIResult::error("Invalid JSON array for paths".to_string()),
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
