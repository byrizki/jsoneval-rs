//! FFI subform functions

use super::types::{FFIResult, JSONEvalHandle};
use serde_json::json;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Evaluate a subform with data
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
/// - data must be a valid null-terminated UTF-8 string
/// - context can be NULL
/// - paths_json can be NULL (if NULL, full evaluation)
#[no_mangle]
pub unsafe extern "C" fn json_eval_evaluate_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
    data: *const c_char,
    context: *const c_char,
    paths_json: *const c_char,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() || data.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let handle_ref = &mut *handle;
    let token = handle_ref.reset_token();
    let eval = &mut handle_ref.inner;

    let path_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

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
        match CStr::from_ptr(paths_json).to_str() {
            Ok(s) => match serde_json::from_str(s) {
                Ok(p) => Some(p),
                Err(e) => return FFIResult::error(format!("Failed to parse paths JSON: {}", e)),
            },
            Err(_) => return FFIResult::error("Invalid UTF-8 in paths_json".to_string()),
        }
    } else {
        None
    };

    match eval.evaluate_subform(path_str, data_str, context_str, paths.as_deref(), token.as_ref()) {
        Ok(_) => FFIResult::success(Vec::new()),
        Err(e) => FFIResult::error(e),
    }
}

/// Validate subform data against its schema rules
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
/// - data must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn json_eval_validate_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
    data: *const c_char,
    context: *const c_char,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() || data.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let handle_ref = &mut *handle;
    let token = handle_ref.reset_token();
    let eval = &mut handle_ref.inner;

    let path_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

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

    match eval.validate_subform(path_str, data_str, context_str, None, token.as_ref()) {
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

/// Evaluate dependents in subform when a field changes
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
/// - changed_path must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn json_eval_evaluate_dependents_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
    changed_path: *const c_char,
    data: *const c_char,
    context: *const c_char,
    re_evaluate: i32,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() || changed_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let handle_ref = &mut *handle;
    let token = handle_ref.reset_token();
    let eval = &mut handle_ref.inner;

    let subform_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let path_str = match CStr::from_ptr(changed_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in changed_path".to_string()),
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

    // Wrap single path in a Vec for the new API
    let paths = vec![path_str.to_string()];

    match eval.evaluate_dependents_subform(subform_str, &paths, data_str, context_str, re_evaluate != 0, token.as_ref(), None) {
        Ok(result) => {
            let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        Err(e) => FFIResult::error(e),
    }
}

/// Resolve layout for subform
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn json_eval_resolve_layout_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
    evaluate: bool,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let path_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    match eval.resolve_layout_subform(path_str, evaluate) {
        Ok(_) => FFIResult::success(Vec::new()),
        Err(e) => FFIResult::error(e),
    }
}

/// Get evaluated schema from subform
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_evaluated_schema_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
    resolve_layout: bool,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let path_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let result = eval.get_evaluated_schema_subform(path_str, resolve_layout);
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    FFIResult::success(result_bytes)
}

/// Get schema value from subform (all .value fields)
///
/// # Safety
/// Get schema value from subform in nested object format
///
/// Returns a hierarchical object structure mimicking the schema.
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_schema_value_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let path_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let result = eval.get_schema_value_subform(path_str);
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    FFIResult::success(result_bytes)
}

/// Get schema values from subform as a flat array of path-value pairs
///
/// Returns keys as "path" and values as "value".
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid UTF-8 string pointer
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_schema_value_array_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let path_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let result = eval.get_schema_value_array_subform(path_str);
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    FFIResult::success(result_bytes)
}

/// Get schema values from subform as a flat object with dotted path keys
///
/// Returns object with dotted paths as keys.
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid UTF-8 string pointer
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_schema_value_object_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let path_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let result = eval.get_schema_value_object_subform(path_str);
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    FFIResult::success(result_bytes)
}


/// Get evaluated schema without $params from subform
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_evaluated_schema_without_params_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
    resolve_layout: bool,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let path_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let result = eval.get_evaluated_schema_without_params_subform(path_str, resolve_layout);
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    FFIResult::success(result_bytes)
}

/// Get evaluated schema by specific path from subform
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
/// - schema_path must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_evaluated_schema_by_path_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
    schema_path: *const c_char,
    skip_layout: bool,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() || schema_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let subform_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let path_str = match CStr::from_ptr(schema_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in schema_path".to_string()),
    };

    match eval.get_evaluated_schema_by_path_subform(subform_str, path_str, skip_layout) {
        Some(value) => {
            let result_bytes = serde_json::to_vec(&value).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        None => FFIResult::error("Path not found in subform".to_string()),
    }
}

/// Get values from the evaluated schema of a subform using multiple dotted path notations
/// Returns data in the specified format (skips paths that are not found)
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
/// - schema_paths_json must be a valid null-terminated UTF-8 string containing a JSON array of paths
/// - format: 0 = Nested (default), 1 = Flat, 2 = Array
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_evaluated_schema_by_paths_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
    schema_paths_json: *const c_char,
    skip_layout: bool,
    format: u8,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() || schema_paths_json.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &mut (*handle).inner;

    let subform_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let paths_str = match CStr::from_ptr(schema_paths_json).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in schema_paths".to_string()),
    };

    // Parse JSON array of paths
    let paths: Vec<String> = match serde_json::from_str(paths_str) {
        Ok(p) => p,
        Err(e) => return FFIResult::error(format!("Failed to parse paths JSON: {}", e)),
    };

    let return_format = match format {
        1 => crate::ReturnFormat::Flat,
        2 => crate::ReturnFormat::Array,
        _ => crate::ReturnFormat::Nested,
    };

    let result = eval.get_evaluated_schema_by_paths_subform(
        subform_str,
        &paths,
        skip_layout,
        Some(return_format),
    );
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    FFIResult::success(result_bytes)
}

/// Get schema by specific path from subform
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
/// - schema_path must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_schema_by_path_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
    schema_path: *const c_char,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() || schema_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &(*handle).inner;

    let subform_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let path_str = match CStr::from_ptr(schema_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in schema_path".to_string()),
    };

    match eval.get_schema_by_path_subform(subform_str, path_str) {
        Some(value) => {
            let result_bytes = serde_json::to_vec(&value).unwrap_or_default();
            FFIResult::success(result_bytes)
        }
        None => FFIResult::error("Path not found in subform".to_string()),
    }
}

/// Get schema by multiple paths from subform
/// Returns data in the specified format (skips paths that are not found)
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
/// - schema_paths_json must be a valid null-terminated UTF-8 string containing a JSON array of paths
/// - format: 0 = Nested (default), 1 = Flat, 2 = Array
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_schema_by_paths_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
    schema_paths_json: *const c_char,
    format: u8,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() || schema_paths_json.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &(*handle).inner;

    let subform_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let paths_str = match CStr::from_ptr(schema_paths_json).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in schema_paths".to_string()),
    };

    // Parse JSON array of paths
    let paths: Vec<String> = match serde_json::from_str(paths_str) {
        Ok(p) => p,
        Err(e) => return FFIResult::error(format!("Failed to parse paths JSON: {}", e)),
    };

    let return_format = match format {
        1 => crate::ReturnFormat::Flat,
        2 => crate::ReturnFormat::Array,
        _ => crate::ReturnFormat::Nested,
    };

    let result = eval.get_schema_by_paths_subform(subform_str, &paths, Some(return_format));
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    FFIResult::success(result_bytes)
}

/// Get list of available subform paths
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
#[no_mangle]
pub unsafe extern "C" fn json_eval_get_subform_paths(handle: *mut JSONEvalHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &(*handle).inner;
    let paths = eval.get_subform_paths();
    let result_bytes = serde_json::to_vec(&paths).unwrap_or_default();
    FFIResult::success(result_bytes)
}

/// Check if a subform exists at the given path
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - subform_path must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn json_eval_has_subform(
    handle: *mut JSONEvalHandle,
    subform_path: *const c_char,
) -> FFIResult {
    if handle.is_null() || subform_path.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let eval = &(*handle).inner;

    let path_str = match CStr::from_ptr(subform_path).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in subform_path".to_string()),
    };

    let has_subform = eval.has_subform(path_str);
    let result = if has_subform { "true" } else { "false" };
    let result_bytes = result.as_bytes().to_vec();
    FFIResult::success(result_bytes)
}
