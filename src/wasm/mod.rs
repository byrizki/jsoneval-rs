//! WebAssembly bindings for browser and Node.js
//!
//! This module provides JavaScript/TypeScript compatible bindings

pub mod cache;
pub mod core;
pub mod evaluation;
pub mod layout;
pub mod schema;
pub mod subforms;
pub mod types;
pub mod validation;

use wasm_bindgen::prelude::*;

// Re-export types for external use
pub use types::{JSONEvalWasm, ValidationError, ValidationResult};

// Re-export all functions for backward compatibility
pub use core::{get_version, init, version};

/// Helper to serialize Rust values to JS objects (not Maps)
/// converting serde_json::Value::Object/HashMap to plain JS objects
pub(crate) fn to_value(
    value: &impl serde::Serialize,
) -> Result<JsValue, serde_wasm_bindgen::Error> {
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true)
        .serialize_large_number_types_as_bigints(true);
    value.serialize(&serializer)
}
