//! JSON Eval RS - high-performance JSON Logic and schema evaluation.
//!
//! Main public entry points are re-exported from this crate root:
//! [`JSONEval`] for schema evaluation, [`ParsedSchema`] for reusable parsed schemas,
//! [`ParsedSchemaCache`] for cache-backed reuse, and [`RLogic`] for lower-level JSON
//! Logic compilation/evaluation.
//!
//! Module map:
//! - [`jsoneval`] owns schema parsing, evaluation, layout resolution, validation, and caches.
//! - [`rlogic`] owns the JSON Logic compiler/evaluator used by schema evaluation.
//! - [`parse_schema`] and [`topo_sort`] preserve legacy parsing/sorting entry points.
//! - [`utils`] contains crate-wide timing/debug helpers and numeric cleanup utilities.
//! - [`ffi`] and [`wasm`] expose binding layers behind feature flags.

// Use mimalloc allocator on Windows for better performance
#[cfg(windows)]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod parse_schema;
pub mod rlogic;
pub mod topo_sort;

pub mod jsoneval;
#[macro_use]
pub mod utils;

// FFI module for C# and other languages
#[cfg(feature = "ffi")]
pub mod ffi;

// WebAssembly module for JavaScript/TypeScript
#[cfg(feature = "wasm")]
pub mod wasm;

// Re-export stable public Rust API from focused internal modules.
pub use jsoneval::eval_data::EvalData;
pub use jsoneval::parsed_schema::ParsedSchema;
pub use jsoneval::parsed_schema_cache::{
    ParsedSchemaCache, ParsedSchemaCacheStats, PARSED_SCHEMA_CACHE,
};
pub use jsoneval::path_utils::ArrayMetadata;
pub use jsoneval::table_metadata::TableMetadata;
pub use rlogic::{
    CompiledLogic, CompiledLogicId, CompiledLogicStore, CompiledLogicStoreStats, Evaluator,
    LogicId, RLogic, RLogicConfig,
};

pub use jsoneval::types::*;
pub use jsoneval::JSONEval;
pub use utils::*;

/// Get the library version
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
