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
