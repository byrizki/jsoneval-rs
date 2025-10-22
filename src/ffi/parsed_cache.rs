//! FFI functions for ParsedSchemaCache management

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, OnceLock};
use crate::{ParsedSchemaCache, ParsedSchema};
use super::types::FFIResult;

/// Opaque pointer type for ParsedSchemaCache instances
pub struct ParsedSchemaCacheHandle {
    inner: ParsedSchemaCache,
}

/// Create a new ParsedSchemaCache instance
/// 
/// # Safety
/// 
/// - Returns a handle that must be freed with parsed_cache_free
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_new() -> *mut ParsedSchemaCacheHandle {
    let cache = ParsedSchemaCache::new();
    let handle = Box::new(ParsedSchemaCacheHandle { inner: cache });
    Box::into_raw(handle)
}

/// Get the global ParsedSchemaCache singleton
/// 
/// # Safety
/// 
/// - Returns a pointer to the global cache (do NOT free this pointer)
/// - This is a static singleton that lives for the entire program lifetime
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_global() -> *const ParsedSchemaCacheHandle {
    static GLOBAL_CACHE_HANDLE: OnceLock<Box<ParsedSchemaCacheHandle>> = OnceLock::new();
    
    let handle = GLOBAL_CACHE_HANDLE.get_or_init(|| {
        Box::new(ParsedSchemaCacheHandle {
            inner: crate::PARSED_SCHEMA_CACHE.clone(),
        })
    });
    
    handle.as_ref() as *const ParsedSchemaCacheHandle
}

/// Parse and insert a schema into the cache
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new or parsed_cache_global
/// - key must be a valid null-terminated UTF-8 string
/// - schema_json must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_insert(
    handle: *mut ParsedSchemaCacheHandle,
    key: *const c_char,
    schema_json: *const c_char,
) -> FFIResult {
    if handle.is_null() || key.is_null() || schema_json.is_null() {
        return FFIResult::error("Invalid pointer".to_string());
    }

    let cache = &mut (*handle).inner;

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in key".to_string()),
    };

    let schema_str = match CStr::from_ptr(schema_json).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in schema".to_string()),
    };

    match ParsedSchema::parse(schema_str) {
        Ok(parsed) => {
            cache.insert(key_str.to_string(), Arc::new(parsed));
            FFIResult::success(Vec::new())
        }
        Err(e) => FFIResult::error(format!("Failed to parse schema: {}", e)),
    }
}

/// Parse and insert a schema from MessagePack into the cache
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new or parsed_cache_global
/// - key must be a valid null-terminated UTF-8 string
/// - schema_msgpack must be a valid pointer to MessagePack bytes
/// - schema_len must be the exact length of the MessagePack data
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_insert_msgpack(
    handle: *mut ParsedSchemaCacheHandle,
    key: *const c_char,
    schema_msgpack: *const u8,
    schema_len: usize,
) -> FFIResult {
    if handle.is_null() || key.is_null() || schema_msgpack.is_null() || schema_len == 0 {
        return FFIResult::error("Invalid pointer or length".to_string());
    }

    let cache = &mut (*handle).inner;

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::error("Invalid UTF-8 in key".to_string()),
    };

    let schema_bytes = std::slice::from_raw_parts(schema_msgpack, schema_len);

    match ParsedSchema::parse_msgpack(schema_bytes) {
        Ok(parsed) => {
            cache.insert(key_str.to_string(), Arc::new(parsed));
            FFIResult::success(Vec::new())
        }
        Err(e) => FFIResult::error(format!("Failed to parse schema from MessagePack: {}", e)),
    }
}

/// Get a cached schema by key
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new or parsed_cache_global
/// - key must be a valid null-terminated UTF-8 string
/// - Returns a pointer to Arc<ParsedSchema> (caller should NOT free this - it's owned by the cache)
/// - Returns NULL if key not found
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_get(
    handle: *const ParsedSchemaCacheHandle,
    key: *const c_char,
) -> *const std::ffi::c_void {
    if handle.is_null() || key.is_null() {
        return ptr::null();
    }

    let cache = &(*handle).inner;

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null(),
    };

    match cache.get(key_str) {
        Some(arc_schema) => Arc::into_raw(arc_schema) as *const std::ffi::c_void,
        None => ptr::null(),
    }
}

/// Check if a key exists in the cache
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new or parsed_cache_global
/// - key must be a valid null-terminated UTF-8 string
/// - Returns 1 if exists, 0 if not
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_contains(
    handle: *const ParsedSchemaCacheHandle,
    key: *const c_char,
) -> i32 {
    if handle.is_null() || key.is_null() {
        return 0;
    }

    let cache = &(*handle).inner;

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    if cache.contains_key(key_str) { 1 } else { 0 }
}

/// Remove a schema from the cache
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new or parsed_cache_global
/// - key must be a valid null-terminated UTF-8 string
/// - Returns 1 if removed, 0 if key not found
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_remove(
    handle: *mut ParsedSchemaCacheHandle,
    key: *const c_char,
) -> i32 {
    if handle.is_null() || key.is_null() {
        return 0;
    }

    let cache = &mut (*handle).inner;

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    if cache.remove(key_str).is_some() { 1 } else { 0 }
}

/// Clear all entries from the cache
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new or parsed_cache_global
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_clear(handle: *mut ParsedSchemaCacheHandle) {
    if handle.is_null() {
        return;
    }

    let cache = &mut (*handle).inner;
    cache.clear();
}

/// Get the number of entries in the cache
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new or parsed_cache_global
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_len(handle: *const ParsedSchemaCacheHandle) -> usize {
    if handle.is_null() {
        return 0;
    }

    let cache = &(*handle).inner;
    cache.len()
}

/// Check if the cache is empty
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new or parsed_cache_global
/// - Returns 1 if empty, 0 if not
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_is_empty(handle: *const ParsedSchemaCacheHandle) -> i32 {
    if handle.is_null() {
        return 1;
    }

    let cache = &(*handle).inner;
    if cache.is_empty() { 1 } else { 0 }
}

/// Get cache statistics (entry count and keys)
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new or parsed_cache_global
/// - Returns JSON string with stats, caller must free with json_eval_free_result
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_stats(handle: *const ParsedSchemaCacheHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let cache = &(*handle).inner;
    let stats = cache.stats();

    let stats_json = serde_json::json!({
        "entry_count": stats.entry_count,
        "keys": stats.keys,
    });

    let result_bytes = serde_json::to_vec(&stats_json).unwrap_or_default();
    FFIResult::success(result_bytes)
}

/// Get all keys in the cache as JSON array
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new or parsed_cache_global
/// - Returns JSON array of keys, caller must free with json_eval_free_result
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_keys(handle: *const ParsedSchemaCacheHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let cache = &(*handle).inner;
    let keys = cache.keys();

    let result_bytes = serde_json::to_vec(&keys).unwrap_or_default();
    FFIResult::success(result_bytes)
}

/// Free a ParsedSchemaCache instance
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from parsed_cache_new
/// - Do NOT call this on the global cache pointer!
/// - handle should not be used after calling this function
#[no_mangle]
pub unsafe extern "C" fn parsed_cache_free(handle: *mut ParsedSchemaCacheHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}
