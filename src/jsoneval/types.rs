use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    #[serde(rename = "fieldValue")]
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

/// One resolved element overlay produced by the layout resolver.
/// Each entry describes properties to apply on top of the compact schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutOverlayEntry {
    /// Which $layout.elements array (e.g. "#/form/$layout/elements")
    pub layout_path: String,
    /// Index within that elements array
    pub element_idx: usize,
    /// Dotted path to $ref target in schema (empty if no $ref)
    pub schema_ref_path: String,
    /// Delta properties to overlay onto the element
    pub overlay: IndexMap<String, Value>,
}

/// Result of get_resolved_layout()
pub type ResolvedLayoutResult = Vec<LayoutOverlayEntry>;
