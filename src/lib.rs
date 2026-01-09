//! JSON Eval RS - High-performance JSON Logic evaluation library
//!
//! This library provides a complete implementation of JSON Logic with advanced features:
//! - Pre-compilation of logic expressions for optimal performance
//! - Mutation tracking via proxy-like data wrapper (EvalData)
//! - All data mutations gated through EvalData for thread safety
//! - Zero external logic dependencies (built from scratch)

// Use mimalloc allocator on Windows for better performance
#[cfg(windows)]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod parse_schema;
pub mod rlogic;
pub mod table_evaluate;
pub mod table_metadata;
pub mod topo_sort;

pub mod eval_cache;
pub mod eval_data;
pub mod json_parser;
pub mod parsed_schema;
pub mod parsed_schema_cache;
pub mod path_utils;
pub mod subform_methods;

// New modular structure
pub mod jsoneval;
pub mod types;
#[macro_use]
pub mod utils;

// FFI module for C# and other languages
#[cfg(feature = "ffi")]
pub mod ffi;

// WebAssembly module for JavaScript/TypeScript
#[cfg(feature = "wasm")]
pub mod wasm;

// Re-export main types for convenience
pub use eval_cache::{CacheKey, CacheStats, EvalCache};
pub use eval_data::EvalData;
pub use parsed_schema::ParsedSchema;
pub use parsed_schema_cache::{ParsedSchemaCache, ParsedSchemaCacheStats, PARSED_SCHEMA_CACHE};
pub use path_utils::ArrayMetadata;
pub use rlogic::{
    CompiledLogic, CompiledLogicId, CompiledLogicStore, CompiledLogicStoreStats, Evaluator,
    LogicId, RLogic, RLogicConfig,
};
pub use table_metadata::TableMetadata;

// Re-export from new modules
pub use jsoneval::JSONEval;
pub use types::*;
pub use utils::*;

/// Get the library version
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
