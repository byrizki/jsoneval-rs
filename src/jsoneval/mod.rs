//! Schema evaluation engine.
//!
//! This module contains the Rust implementation behind [`JSONEval`]: parsing,
//! dependency tracking, evaluation, layout resolution, validation, subforms, and
//! schema caches. Crate root re-exports stable public types so callers usually do
//! not need to import from submodules directly.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::jsoneval::eval_data::EvalData;

use crate::jsoneval::table_metadata::TableMetadata;

use crate::jsoneval::types::DependentItem;
use crate::rlogic::{LogicId, RLogic};

use indexmap::{IndexMap, IndexSet};

use serde_json::Value;

pub mod cancellation;
pub mod core;
pub mod dependents;
pub mod eval_cache;
pub mod eval_data;
pub mod evaluate;
pub mod getters;
pub mod json_parser;
pub mod layout;
pub mod logic;
pub mod parsed_schema;
pub mod parsed_schema_cache;
pub mod path_utils;
pub mod static_arrays;
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
    /// Field schema pointers referenced by one or more `$layout` elements.
    pub layout_field_refs: Arc<IndexSet<String>>,
    pub options_templates: Arc<Vec<(String, String, String)>>,
    pub subforms: IndexMap<String, Box<JSONEval>>,

    pub reffed_by: Arc<IndexMap<String, Vec<String>>>,

    /// Reverse map: data path → list of source field schema paths whose dependent
    /// value/clear formulas reference that path (excluding $value/$refValue context vars).
    /// When field X changes, source fields in dep_formula_triggers[X] are re-queued so
    /// their downstream dependents are re-evaluated with the new context.
    pub dep_formula_triggers: Arc<IndexMap<String, Vec<(String, usize)>>>,

    pub conditional_hidden_fields: Arc<Vec<String>>,
    pub conditional_readonly_fields: Arc<Vec<String>>,
    pub static_arrays: Arc<IndexMap<String, Arc<Value>>>,

    pub context: Value,
    pub data: Value,
    pub evaluated_schema: Value,
    pub eval_data: EvalData,
    pub eval_cache: eval_cache::EvalCache,
    pub(crate) eval_lock: Mutex<()>,
    pub(crate) cached_msgpack_schema: Option<Vec<u8>>,
    pub(crate) resolved_layout_cache: Option<Arc<Vec<crate::jsoneval::types::LayoutOverlayEntry>>>,
    /// `$ref` targets hidden in every current resolved layout occurrence.
    pub(crate) layout_hidden_refs: indexmap::IndexSet<String>,
    /// `$ref` targets visible in at least one current resolved layout occurrence.
    /// Used while resolving to subtract refs from layout_hidden_refs.
    pub(crate) layout_visible_refs: indexmap::IndexSet<String>,
    /// Subset of layout_hidden_refs hidden by a condition.hidden cascade and eligible for clearing.
    pub(crate) layout_condition_hidden_refs: indexmap::IndexSet<String>,
    pub(crate) regex_cache: std::sync::RwLock<HashMap<String, regex::Regex>>,
}
