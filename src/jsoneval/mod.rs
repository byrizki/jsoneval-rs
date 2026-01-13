use std::sync::{Arc, Mutex};

use crate::jsoneval::eval_cache::EvalCache;
use crate::jsoneval::eval_data::EvalData;

use crate::jsoneval::table_metadata::TableMetadata;

use crate::rlogic::{
    LogicId, RLogic,
};
use crate::jsoneval::types::DependentItem;

use indexmap::{IndexMap, IndexSet};

use serde_json::Value;

pub mod cache;
pub mod cancellation;
pub mod dependents;
pub mod eval_cache;
pub mod eval_data;
pub mod evaluate;
pub mod getters;
pub mod core;
pub mod json_parser;
pub mod layout;
pub mod logic;
pub mod parsed_schema;
pub mod parsed_schema_cache;
pub mod path_utils;
pub mod subform_methods;
pub mod table_evaluate;
pub mod table_metadata;
pub mod types;
pub mod validation;

pub struct JSONEval {
    pub schema: Arc<Value>,
    pub engine: Arc<RLogic>,
    /// Zero-copy Arc-wrapped collections (shared from ParsedSchema)
    pub evaluations: Arc<IndexMap<String, LogicId>>,
    pub tables: Arc<IndexMap<String, Value>>,
    /// Pre-compiled table metadata (computed at parse time for zero-copy evaluation)
    pub table_metadata: Arc<IndexMap<String, TableMetadata>>,
    pub dependencies: Arc<IndexMap<String, IndexSet<String>>>,
    /// Evaluations grouped into parallel-executable batches
    /// Each inner Vec contains evaluations that can run concurrently
    pub sorted_evaluations: Arc<Vec<Vec<String>>>,
    /// Evaluations categorized for result handling
    /// Dependents: map from source field to list of dependent items
    pub dependents_evaluations: Arc<IndexMap<String, Vec<DependentItem>>>,
    /// Rules: evaluations with "/rules/" in path
    pub rules_evaluations: Arc<Vec<String>>,
    /// Fields with rules: dotted paths of all fields that have rules (for efficient validation)
    pub fields_with_rules: Arc<Vec<String>>,
    /// Others: all other evaluations not in sorted_evaluations (for evaluated_schema output)
    pub others_evaluations: Arc<Vec<String>>,
    /// Value: evaluations ending with ".value" in path
    pub value_evaluations: Arc<Vec<String>>,
    /// Cached layout paths (collected at parse time)
    pub layout_paths: Arc<Vec<String>>,
    /// Options URL templates (url_path, template_str, params_path) collected at parse time
    pub options_templates: Arc<Vec<(String, String, String)>>,
    /// Subforms: isolated JSONEval instances for array fields with items
    /// Key is the schema path (e.g., "#/riders"), value is the sub-JSONEval
    pub subforms: IndexMap<String, Box<JSONEval>>,

    /// Cached reference to parsed schema mappings (reffed_by)
    pub reffed_by: Arc<IndexMap<String, Vec<String>>>,

    /// Cached paths of fields that have hidden conditions
    pub conditional_hidden_fields: Arc<Vec<String>>,
    /// Cached paths of fields that have disabled conditions and value property
    pub conditional_readonly_fields: Arc<Vec<String>>,

    pub context: Value,
    pub data: Value,
    pub evaluated_schema: Value,
    pub eval_data: EvalData,
    /// Evaluation cache with content-based hashing and zero-copy storage
    pub eval_cache: EvalCache,
    /// Flag to enable/disable evaluation caching
    /// Set to false for web API usage where each request creates a new JSONEval instance
    pub cache_enabled: bool,
    /// Mutex for synchronous execution of evaluate and evaluate_dependents
    pub(crate) eval_lock: Mutex<()>,
    /// Cached MessagePack bytes for zero-copy schema retrieval
    /// Stores original MessagePack if initialized from binary, cleared on schema mutations
    pub(crate) cached_msgpack_schema: Option<Vec<u8>>,
}
