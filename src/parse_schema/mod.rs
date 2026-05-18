//! Schema parsing entry points.
//!
//! `legacy` keeps the original parser shape used by existing callers.
//! `parsed` fills reusable [`crate::ParsedSchema`] structures. Public functions
//! are re-exported here to preserve existing Rust imports.

pub mod common;
pub mod legacy;
pub mod parsed;

pub use legacy::parse_schema;
pub use parsed::parse_schema_into;
