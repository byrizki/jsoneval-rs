//! Core FFI functions: version, constructors, memory management

use super::types::{FFIResult, JSONEvalHandle};
use crate::JSONEval;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

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
                eprintln!(
                    "[FFI ERROR] json_eval_new_from_msgpack: invalid UTF-8 in context: {}",
                    e
                );
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
                eprintln!(
                    "[FFI ERROR] json_eval_new_from_msgpack: invalid UTF-8 in data: {}",
                    e
                );
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

/// Free an FFIResult
///
/// # Safety
///
/// - result must be a valid FFIResult from one of the evaluate functions
/// - result should not be used after calling this function
#[no_mangle]
pub unsafe extern "C" fn json_eval_free_result(result: super::types::FFIResult) {
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
        Err(_) => return FFIResult::error("Invalid UTF-8 in schema".to_string()),
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(_) => return FFIResult::error("Invalid UTF-8 in context".to_string()),
        }
    } else {
        None
    };

    let data_str = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => Some(s),
            Err(_) => return FFIResult::error("Invalid UTF-8 in data".to_string()),
        }
    } else {
        None
    };

    match eval.reload_schema(schema_str, context_str, data_str) {
        Ok(_) => FFIResult::success(Vec::new()),
        Err(e) => FFIResult::error(e),
    }
}

/// Reload schema from MessagePack-encoded bytes
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - schema_msgpack must be a valid pointer to MessagePack bytes
/// - schema_len must be the exact length of the MessagePack data
/// - context and data can be NULL
#[no_mangle]
pub unsafe extern "C" fn json_eval_reload_schema_msgpack(
    handle: *mut JSONEvalHandle,
    schema_msgpack: *const u8,
    schema_len: usize,
    context: *const c_char,
    data: *const c_char,
) -> FFIResult {
    if handle.is_null() || schema_msgpack.is_null() || schema_len == 0 {
        return FFIResult::error("Invalid handle, schema pointer, or length".to_string());
    }

    let eval = &mut (*handle).inner;

    let schema_bytes = std::slice::from_raw_parts(schema_msgpack, schema_len);

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(_) => return FFIResult::error("Invalid UTF-8 in context".to_string()),
        }
    } else {
        None
    };

    let data_str = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => Some(s),
            Err(_) => return FFIResult::error("Invalid UTF-8 in data".to_string()),
        }
    } else {
        None
    };

    match eval.reload_schema_msgpack(schema_bytes, context_str, data_str) {
        Ok(_) => FFIResult::success(Vec::new()),
        Err(e) => FFIResult::error(e),
    }
}

/// Create a new JSONEval instance from a cached ParsedSchema
///
/// # Safety
///
/// - cache_key must be a valid null-terminated UTF-8 string
/// - context and data can be NULL
/// - Returns non-null handle on success, null on failure
/// - Caller must call json_eval_free when done
#[no_mangle]
pub unsafe extern "C" fn json_eval_new_from_cache(
    cache_key: *const c_char,
    context: *const c_char,
    data: *const c_char,
) -> *mut JSONEvalHandle {
    if cache_key.is_null() {
        eprintln!("[FFI ERROR] json_eval_new_from_cache: cache_key pointer is null");
        return ptr::null_mut();
    }

    let key_str = match CStr::from_ptr(cache_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "[FFI ERROR] json_eval_new_from_cache: invalid UTF-8 in cache_key: {}",
                e
            );
            return ptr::null_mut();
        }
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!(
                    "[FFI ERROR] json_eval_new_from_cache: invalid UTF-8 in context: {}",
                    e
                );
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
                eprintln!(
                    "[FFI ERROR] json_eval_new_from_cache: invalid UTF-8 in data: {}",
                    e
                );
                return ptr::null_mut();
            }
        }
    } else {
        None
    };

    // Get the cached ParsedSchema
    let parsed = match crate::PARSED_SCHEMA_CACHE.get(key_str) {
        Some(p) => p,
        None => {
            eprintln!(
                "[FFI ERROR] json_eval_new_from_cache: schema '{}' not found in cache",
                key_str
            );
            return ptr::null_mut();
        }
    };

    // Create JSONEval from the cached ParsedSchema
    match crate::JSONEval::with_parsed_schema(parsed, context_str, data_str) {
        Ok(eval) => {
            let handle = Box::new(JSONEvalHandle {
                inner: Box::new(eval),
            });
            Box::into_raw(handle)
        }
        Err(e) => {
            let error_msg = format!("Failed to create JSONEval instance from cache: {}", e);
            eprintln!("[FFI ERROR] json_eval_new_from_cache: {}", error_msg);
            ptr::null_mut()
        }
    }
}

/// Create a new JSONEval instance from cache with detailed error reporting
///
/// # Safety
///
/// - cache_key must be a valid null-terminated UTF-8 string
/// - context and data can be NULL
/// - error_out must be a valid pointer to store error message (caller owns the string)
/// - Returns non-null handle on success, null on failure (check error_out for details)
#[no_mangle]
pub unsafe extern "C" fn json_eval_new_from_cache_with_error(
    cache_key: *const c_char,
    context: *const c_char,
    data: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut JSONEvalHandle {
    if cache_key.is_null() {
        if !error_out.is_null() {
            *error_out = CString::new("cache_key pointer is null")
                .unwrap()
                .into_raw();
        }
        return ptr::null_mut();
    }

    let key_str = match CStr::from_ptr(cache_key).to_str() {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                let error_msg = format!("Invalid UTF-8 in cache_key: {}", e);
                *error_out = CString::new(error_msg).unwrap().into_raw();
            }
            return ptr::null_mut();
        }
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                if !error_out.is_null() {
                    let error_msg = format!("Invalid UTF-8 in context: {}", e);
                    *error_out = CString::new(error_msg).unwrap().into_raw();
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
                    let error_msg = format!("Invalid UTF-8 in data: {}", e);
                    *error_out = CString::new(error_msg).unwrap().into_raw();
                }
                return ptr::null_mut();
            }
        }
    } else {
        None
    };

    // Get the cached ParsedSchema
    let parsed = match crate::PARSED_SCHEMA_CACHE.get(key_str) {
        Some(p) => p,
        None => {
            if !error_out.is_null() {
                let error_msg = format!("Schema '{}' not found in cache", key_str);
                *error_out = CString::new(error_msg).unwrap().into_raw();
            }
            return ptr::null_mut();
        }
    };

    // Create JSONEval from the cached ParsedSchema
    match crate::JSONEval::with_parsed_schema(parsed, context_str, data_str) {
        Ok(eval) => {
            if !error_out.is_null() {
                *error_out = ptr::null_mut(); // No error
            }
            let handle = Box::new(JSONEvalHandle {
                inner: Box::new(eval),
            });
            Box::into_raw(handle)
        }
        Err(e) => {
            if !error_out.is_null() {
                let error_msg = format!("Failed to create JSONEval from cache: {}", e);
                *error_out = CString::new(error_msg).unwrap().into_raw();
            }
            ptr::null_mut()
        }
    }
}

/// Reload schema from ParsedSchemaCache using a cache key
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - cache_key must be a valid UTF-8 string
/// - context and data can be NULL
#[no_mangle]
pub unsafe extern "C" fn json_eval_reload_schema_from_cache(
    handle: *mut JSONEvalHandle,
    cache_key: *const c_char,
    context: *const c_char,
    data: *const c_char,
) -> FFIResult {
    if handle.is_null() || cache_key.is_null() {
        return FFIResult::error("Invalid handle or cache_key".to_string());
    }

    let eval = &mut (*handle).inner;

    let key_str = match CStr::from_ptr(cache_key).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in cache_key".to_string()),
    };

    let context_str = if !context.is_null() {
        match CStr::from_ptr(context).to_str() {
            Ok(s) => Some(s),
            Err(_) => return FFIResult::error("Invalid UTF-8 in context".to_string()),
        }
    } else {
        None
    };

    let data_str = if !data.is_null() {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => Some(s),
            Err(_) => return FFIResult::error("Invalid UTF-8 in data".to_string()),
        }
    } else {
        None
    };

    match eval.reload_schema_from_cache(key_str, context_str, data_str) {
        Ok(_) => FFIResult::success(Vec::new()),
        Err(e) => FFIResult::error(e),
    }
}

/// Set timezone offset for datetime operations (TODAY, NOW)
///
/// # Safety
///
/// - handle must be a valid pointer from json_eval_new
/// - Pass offset_minutes as the timezone offset in minutes from UTC
///   (e.g., 420 for UTC+7, -300 for UTC-5)
/// - Pass i32::MIN to reset to UTC (no offset)
///
/// # Example
///
/// ```c
/// // Set to UTC+7 (Jakarta, Bangkok)
/// json_eval_set_timezone_offset(handle, 420);
///
/// // Set to UTC-5 (New York, EST)
/// json_eval_set_timezone_offset(handle, -300);
///
/// // Reset to UTC
/// json_eval_set_timezone_offset(handle, i32::MIN);
/// ```
#[no_mangle]
pub unsafe extern "C" fn json_eval_set_timezone_offset(
    handle: *mut JSONEvalHandle,
    offset_minutes: i32,
) {
    if handle.is_null() {
        eprintln!("[FFI ERROR] json_eval_set_timezone_offset: handle is null");
        return;
    }

    let eval = &mut (*handle).inner;

    // Use i32::MIN as sentinel value for None/reset to UTC
    let offset = if offset_minutes == i32::MIN {
        None
    } else {
        Some(offset_minutes)
    };

    eval.set_timezone_offset(offset);
}
