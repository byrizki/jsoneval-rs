//! Parsed Schema - Reusable parsing results for caching across multiple JSONEval instances
//!
//! This module separates the parsing results from the evaluation state, allowing
//! schemas to be parsed once and reused across multiple evaluations with different data/context.

use indexmap::{IndexMap, IndexSet};
use serde_json::Value;
use std::sync::Arc;
use crate::{LogicId, RLogic, RLogicConfig, TableMetadata, DependentItem};

/// Parsed schema containing all pre-compiled evaluation metadata.
/// This structure is separate from JSONEval to enable caching and reuse.
/// 
/// # Caching Strategy
/// 
/// Wrap ParsedSchema in Arc for sharing across threads and caching:
/// 
/// ```ignore
/// use std::sync::Arc;
/// 
/// // Parse once and wrap in Arc for caching
/// let parsed = Arc::new(ParsedSchema::parse(schema_str)?);
/// cache.insert(schema_key, parsed.clone());
/// 
/// // Reuse across multiple evaluations (Arc::clone is cheap)
/// let eval1 = JSONEval::with_parsed_schema(parsed.clone(), Some(context1), Some(data1))?;
/// let eval2 = JSONEval::with_parsed_schema(parsed.clone(), Some(context2), Some(data2))?;
/// ```
pub struct ParsedSchema {
    /// The original schema Value (wrapped in Arc for efficient sharing)
    pub schema: Arc<Value>,
    
    /// RLogic engine with all compiled logic expressions (wrapped in Arc for sharing)
    /// Multiple JSONEval instances created from the same ParsedSchema will share this engine
    pub engine: Arc<RLogic>,
    
    /// Map of evaluation keys to compiled logic IDs
    pub evaluations: IndexMap<String, LogicId>,
    
    /// Table definitions (rows, datas, skip, clear)
    pub tables: IndexMap<String, Value>,
    
    /// Pre-compiled table metadata (computed at parse time for zero-copy evaluation)
    pub table_metadata: IndexMap<String, TableMetadata>,
    
    /// Dependencies map (evaluation key -> set of dependency paths)
    pub dependencies: IndexMap<String, IndexSet<String>>,
    
    /// Evaluations grouped into parallel-executable batches
    /// Each inner Vec contains evaluations that can run concurrently
    pub sorted_evaluations: Vec<Vec<String>>,
    
    /// Evaluations categorized for result handling
    /// Dependents: map from source field to list of dependent items
    pub dependents_evaluations: IndexMap<String, Vec<DependentItem>>,
    
    /// Rules: evaluations with "/rules/" in path
    pub rules_evaluations: Vec<String>,
    
    /// Fields with rules: dotted paths of all fields that have rules (for efficient validation)
    pub fields_with_rules: Vec<String>,
    
    /// Others: all other evaluations not in sorted_evaluations (for evaluated_schema output)
    pub others_evaluations: Vec<String>,
    
    /// Value: evaluations ending with ".value" in path
    pub value_evaluations: Vec<String>,
    
    /// Cached layout paths (collected at parse time)
    pub layout_paths: Vec<String>,
    
    /// Options URL templates (url_path, template_str, params_path) collected at parse time
    pub options_templates: Vec<(String, String, String)>,
    
    /// Subforms: cached ParsedSchema instances for array fields with items
    /// Key is the schema path (e.g., "#/riders"), value is Arc<ParsedSchema> for cheap cloning
    /// This allows subforms to be shared across multiple JSONEval instances efficiently
    pub subforms: IndexMap<String, Arc<ParsedSchema>>,
}

impl ParsedSchema {
    /// Parse a schema string into a ParsedSchema structure
    /// 
    /// # Arguments
    /// 
    /// * `schema` - JSON schema string
    /// 
    /// # Returns
    /// 
    /// A Result containing the ParsedSchema or an error
    pub fn parse(schema: &str) -> Result<Self, String> {
        let schema_val: Value = serde_json::from_str(schema)
            .map_err(|e| format!("Failed to parse schema JSON: {}", e))?;
        Self::parse_value(schema_val)
    }
    
    /// Parse a schema Value into a ParsedSchema structure
    /// 
    /// # Arguments
    /// 
    /// * `schema_val` - JSON schema Value
    /// 
    /// # Returns
    /// 
    /// A Result containing the ParsedSchema or an error
    pub fn parse_value(schema_val: Value) -> Result<Self, String> {
        let engine_config = RLogicConfig::default();
        
        let mut parsed = Self {
            schema: Arc::new(schema_val),
            engine: Arc::new(RLogic::with_config(engine_config)),
            evaluations: IndexMap::new(),
            tables: IndexMap::new(),
            table_metadata: IndexMap::new(),
            dependencies: IndexMap::new(),
            sorted_evaluations: Vec::new(),
            dependents_evaluations: IndexMap::new(),
            rules_evaluations: Vec::new(),
            fields_with_rules: Vec::new(),
            others_evaluations: Vec::new(),
            value_evaluations: Vec::new(),
            layout_paths: Vec::new(),
            options_templates: Vec::new(),
            subforms: IndexMap::new(),
        };
        
        // Parse the schema to populate all fields
        crate::parse_schema::parsed::parse_schema_into(&mut parsed)?;
        
        Ok(parsed)
    }
    
    /// Parse a MessagePack-encoded schema into a ParsedSchema structure
    /// 
    /// # Arguments
    /// 
    /// * `schema_msgpack` - MessagePack-encoded schema bytes
    /// 
    /// # Returns
    /// 
    /// A Result containing the ParsedSchema or an error
    pub fn parse_msgpack(schema_msgpack: &[u8]) -> Result<Self, String> {
        let schema_val: Value = rmp_serde::from_slice(schema_msgpack)
            .map_err(|e| format!("Failed to deserialize MessagePack schema: {}", e))?;
        
        Self::parse_value(schema_val)
            .map_err(|e| format!("Failed to parse schema: {}", e))
    }
    
    /// Get a reference to the original schema
    pub fn schema(&self) -> &Value {
        &*self.schema
    }
}
