//! FFI (Foreign Function Interface) bindings for C#, C++, and other languages
//! 
//! This module provides a C-compatible API for the JSON evaluation library.

pub mod types;
pub mod core;
pub mod evaluation;
pub mod schema;
pub mod cache;
pub mod layout;
pub mod subforms;
pub mod parsed_cache;

// Re-export types for external use
pub use types::{FFIResult, JSONEvalHandle};
pub use parsed_cache::ParsedSchemaCacheHandle;

// Re-export all functions for backward compatibility
pub use core::*;
pub use evaluation::*;
pub use schema::*;
pub use cache::*;
pub use layout::*;
pub use subforms::*;
pub use parsed_cache::*;
