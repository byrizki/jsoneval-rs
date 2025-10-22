/// Schema parsing module - coordinates between legacy and parsed implementations

pub mod common;
pub mod legacy;
pub mod parsed;

// Re-export public APIs for backward compatibility
pub use legacy::parse_schema;
pub use parsed::parse_schema_into;
