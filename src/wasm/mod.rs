//! WebAssembly bindings for browser and Node.js
//! 
//! This module provides JavaScript/TypeScript compatible bindings

pub mod types;
pub mod core;
pub mod evaluation;
pub mod validation;
pub mod schema;
pub mod cache;
pub mod layout;
pub mod subforms;

use wasm_bindgen::prelude::*;

// Re-export types for external use
pub use types::{ValidationError, ValidationResult, JSONEvalWasm};

// Re-export all functions for backward compatibility
pub use core::{get_version, version, init};

/// Helper to serialize Rust values to JS objects (not Maps)
/// converting serde_json::Value::Object/HashMap to plain JS objects
pub(crate) fn to_value(value: &impl serde::Serialize) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    value.serialize(&serializer)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
