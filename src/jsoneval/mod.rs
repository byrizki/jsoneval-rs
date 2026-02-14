
use std::collections::HashMap;
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
    pub evaluations: Arc<IndexMap<String, LogicId>>,
    pub tables: Arc<IndexMap<String, Value>>,
    pub table_metadata: Arc<IndexMap<String, TableMetadata>>,
    pub dependencies: Arc<IndexMap<String, IndexSet<String>>>,
    pub sorted_evaluations: Arc<Vec<Vec<String>>>,
    pub dependents_evaluations: Arc<IndexMap<String, Vec<DependentItem>>>,
    pub rules_evaluations: Arc<Vec<String>>,
    pub fields_with_rules: Arc<Vec<String>>,
    pub others_evaluations: Arc<Vec<String>>,
    pub value_evaluations: Arc<Vec<String>>,
    pub layout_paths: Arc<Vec<String>>,
    pub options_templates: Arc<Vec<(String, String, String)>>,
    pub subforms: IndexMap<String, Box<JSONEval>>,

    pub reffed_by: Arc<IndexMap<String, Vec<String>>>,

    pub conditional_hidden_fields: Arc<Vec<String>>,
    pub conditional_readonly_fields: Arc<Vec<String>>,

    pub context: Value,
    pub data: Value,
    pub evaluated_schema: Value,
    pub eval_data: EvalData,
    pub eval_cache: EvalCache,
    pub cache_enabled: bool,
    pub(crate) eval_lock: Mutex<()>,
    pub(crate) cached_msgpack_schema: Option<Vec<u8>>,
    pub(crate) regex_cache: std::sync::RwLock<HashMap<String, regex::Regex>>,
}
