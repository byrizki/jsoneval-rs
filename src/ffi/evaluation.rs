//! FFI evaluation functions

use super::types::{FFIResult, JSONEvalHandle};
use std::ffi::CStr;
use std::os::raw::c_char;

/// Evaluate the schema with provided data
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - data must be a valid null-terminated UTF-8 string
/// - context can be NULL
/// - paths_json can be NULL or a valid null-terminated string containing a JSON array of path strings
/// - Caller must call json_eval_free_result when done with the result
#[no_mangle]
pub unsafe extern "C" fn json_eval_evaluate(
    handle: *mut JSONEvalHandle,
    data: *const c_char,
    context: *const c_char,
    paths_json: *const c_char,
) -> FFIResult {
    if handle.is_null() || data.is_null() {
        return FFIResult::error("Invalid handle or data pointer".to_string());
    }

    let handle_ref = &mut *handle;
    let token = handle_ref.reset_token();
    let eval = &mut handle_ref.inner;

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

    let paths = if !paths_json.is_null() {
        match CStr::from_ptr(paths_json).to_str() {
            Ok(s) => match serde_json::from_str::<Vec<String>>(s) {
                Ok(p) => Some(p),
                Err(e) => return FFIResult::error(format!("Failed to parse paths JSON: {}", e)),
            },
            Err(_) => return FFIResult::error("Invalid UTF-8 in paths".to_string()),
        }
    } else {
        None
    };

    match eval.evaluate(data_str, context_str, paths.as_deref(), token.as_ref()) {
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

    let handle_ref = &mut *handle;
    let token = handle_ref.reset_token();
    let eval = &mut handle_ref.inner;

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

    match eval.validate(data_str, context_str, None, token.as_ref()) {
        Ok(validation_result) => {
            let mut errors_map = serde_json::Map::new();
            for (path, err) in &validation_result.errors {
                errors_map.insert(
                    path.clone(),
                    serde_json::json!({
                    "path": path,
                    "type": err.rule_type,
                    "message": err.message,
                    "code": err.code,
                    "pattern": err.pattern,
                    "fieldValue": err.field_value,
                    "data": err.data,
                    }),
                );
            }

            let result_json = serde_json::json!({
                "hasError": validation_result.has_error,
                "error": errors_map
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

    let handle_ref = &mut *handle;
    let token = handle_ref.reset_token();
    let eval = &mut handle_ref.inner;

    let paths_json_str = match CStr::from_ptr(changed_paths_json).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in paths".to_string()),
    };

    // Parse JSON array of paths
    let paths: Vec<String> = match serde_json::from_str(paths_json_str) {
        Ok(p) => p,
        Err(e) => return FFIResult::error(format!("Failed to parse paths JSON: {}", e)),
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

    match eval.evaluate_dependents(&paths, data_str, context_str, re_evaluate != 0, token.as_ref(), None) {
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

    match eval.compile_and_run_logic(logic, data_str, context_str) {
        Ok(result) => {
            let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}
