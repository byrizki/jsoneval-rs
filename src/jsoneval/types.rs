use serde::{Deserialize, Serialize};
use serde_json::Value;
use indexmap::IndexMap;

/// Return format for path-based methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReturnFormat {
    /// Nested object preserving the path hierarchy (default)
    /// Example: { "user": { "profile": { "name": "John" } } }
    #[default]
    Nested,
    /// Flat object with dotted keys
    /// Example: { "user.profile.name": "John" }
    Flat,
    /// Array of values in the order of requested paths
    /// Example: ["John"]
    Array,
}

/// Dependent item structure for transitive dependency tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependentItem {
    pub ref_path: String,
    pub clear: Option<Value>, // Can be $evaluation or boolean
    pub value: Option<Value>, // Can be $evaluation or primitive value
}

/// Validation error for a field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationError {
    #[serde(rename = "type")]
    pub rule_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Result of validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationResult {
    pub has_error: bool,
    pub errors: IndexMap<String, ValidationError>,
}
