//! FFI type definitions

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use crate::JSONEval;

/// Opaque pointer type for JSONEval instances
pub struct JSONEvalHandle {
    pub(super) inner: Box<JSONEval>,
}

/// Zero-copy result type for FFI operations
/// 
/// # Zero-Copy Architecture
/// 
/// This structure implements true zero-copy data transfer across the FFI boundary:
/// 
/// 1. **Rust Side**: Serialized data (JSON/MessagePack) is allocated in a Vec<u8>
/// 2. **Boxing**: Vec is boxed and converted to raw pointer via Box::into_raw()
/// 3. **Transfer**: Raw pointer and length are passed to caller (NO COPY)
/// 4. **Caller Side**: Reads data directly from Rust-owned memory (NO COPY)
/// 5. **Cleanup**: Caller must call json_eval_free_result() to drop the Box
/// 
/// The data remains valid and owned by Rust until explicitly freed. The caller
/// accesses Rust's memory directly without any intermediate copies.
/// 
/// # Memory Layout
/// 
/// ```text
/// Rust Memory:        FFI Boundary:      Caller Memory:
/// +-----------+       data_ptr ------>  Direct read (zero-copy)
/// | Vec<u8>   |       data_len          No allocation needed
/// | [1,2,3..] |                         Marshal.Copy if needed
/// +-----------+
/// ```
#[repr(C)]
pub struct FFIResult {
    pub success: bool,
    pub data_ptr: *const u8,
    pub data_len: usize,
    pub error: *mut c_char,
    // Internal pointer to owned data for cleanup
    pub(super) _owned_data: *mut Vec<u8>,
}

impl Default for FFIResult {
    fn default() -> Self {
        Self {
            success: false,
            data_ptr: ptr::null(),
            data_len: 0,
            error: ptr::null_mut(),
            _owned_data: ptr::null_mut(),
        }
    }
}

impl FFIResult {
    pub fn success(data: Vec<u8>) -> Self {
        let boxed_data = Box::new(data);
        let data_ptr = boxed_data.as_ptr();
        let data_len = boxed_data.len();
        Self {
            success: true,
            data_ptr,
            data_len,
            error: ptr::null_mut(),
            _owned_data: Box::into_raw(boxed_data),
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            success: false,
            data_ptr: ptr::null(),
            data_len: 0,
            error: CString::new(msg).unwrap_or_else(|_| CString::new("Error message contains null byte").unwrap()).into_raw(),
            _owned_data: ptr::null_mut(),
        }
    }
}
