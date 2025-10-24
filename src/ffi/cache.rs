//! FFI cache management functions

use super::types::{FFIResult, JSONEvalHandle};

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

/// Enable evaluation caching
/// Useful for reusing JSONEval instances with different data
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
#[no_mangle]
pub unsafe extern "C" fn json_eval_enable_cache(
    handle: *mut JSONEvalHandle,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &mut (*handle).inner;
    eval.enable_cache();
    
    FFIResult::success(Vec::new())
}

/// Disable evaluation caching
/// Useful for web API usage where each request creates a new JSONEval instance
/// Improves performance by skipping cache operations that have no benefit for single-use instances
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
#[no_mangle]
pub unsafe extern "C" fn json_eval_disable_cache(
    handle: *mut JSONEvalHandle,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::error("Invalid handle pointer".to_string());
    }

    let eval = &mut (*handle).inner;
    eval.disable_cache();
    
    FFIResult::success(Vec::new())
}

/// Check if evaluation caching is enabled
/// 
/// # Safety
/// 
/// - handle must be a valid pointer from json_eval_new
/// - Returns 1 if enabled, 0 if disabled
#[no_mangle]
pub unsafe extern "C" fn json_eval_is_cache_enabled(
    handle: *mut JSONEvalHandle,
) -> i32 {
    if handle.is_null() {
        return 0;
    }

    let eval = &(*handle).inner;
    if eval.is_cache_enabled() { 1 } else { 0 }
}
