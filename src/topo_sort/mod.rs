/// Topological sorting module - coordinates between legacy and parsed implementations
pub mod common;
pub mod legacy;
pub mod parsed;

// Re-export public APIs for backward compatibility
pub use legacy::{topological_sort, visit_node, visit_node_with_priority};
pub use parsed::topological_sort_parsed;
