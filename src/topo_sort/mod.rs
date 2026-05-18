//! Dependency topological sorting entry points.
//!
//! `legacy` keeps the original sorting API for schema evaluation. `parsed` sorts
//! dependencies stored in [`crate::ParsedSchema`]. Public functions are re-exported
//! here to preserve existing Rust imports.

pub mod common;
pub mod legacy;
pub mod parsed;

pub use legacy::{topological_sort, visit_node, visit_node_with_priority};
pub use parsed::topological_sort_parsed;
