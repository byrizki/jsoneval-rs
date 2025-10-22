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

// Re-export types for external use
pub use types::{ValidationError, ValidationResult, JSONEvalWasm};

// Re-export all functions for backward compatibility
pub use core::{get_version, version, init};
