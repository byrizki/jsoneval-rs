//! FFI (Foreign Function Interface) bindings for C#, C++, and other languages
//!
//! This module provides a C-compatible API for the JSON evaluation library.

pub mod cache;
pub mod compiled_logic;
pub mod core;
pub mod evaluation;
pub mod layout;
pub mod parsed_cache;
pub mod schema;
pub mod subforms;
pub mod types;

// Re-export types for external use
pub use parsed_cache::ParsedSchemaCacheHandle;
pub use types::{FFIResult, JSONEvalHandle};

// Re-export all functions for backward compatibility
pub use cache::*;
pub use compiled_logic::*;
pub use core::*;
pub use evaluation::*;
pub use layout::*;
pub use parsed_cache::*;
pub use schema::*;
pub use subforms::*;
