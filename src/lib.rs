//! JSON Eval RS - High-performance JSON Logic evaluation library
//!
//! This library provides a complete implementation of JSON Logic with advanced features:
//! - Pre-compilation of logic expressions for optimal performance
//! - Mutation tracking via proxy-like data wrapper (EvalData)
//! - All data mutations gated through EvalData for thread safety
//! - Zero external logic dependencies (built from scratch)

// Use mimalloc allocator on Windows for better performance
#[cfg(windows)]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod rlogic;
pub mod table_evaluate;
pub mod table_metadata;
pub mod topo_sort;
pub mod parse_schema;

pub mod parsed_schema;
pub mod parsed_schema_cache;
pub mod json_parser;
pub mod path_utils;
pub mod eval_data;
pub mod eval_cache;
pub mod subform_methods;

// FFI module for C# and other languages
#[cfg(feature = "ffi")]
pub mod ffi;

// WebAssembly module for JavaScript/TypeScript
#[cfg(feature = "wasm")]
pub mod wasm;

// Re-export main types for convenience
use indexmap::{IndexMap, IndexSet};
pub use rlogic::{
    CompiledLogic, CompiledLogicStore, Evaluator,
    LogicId, RLogic, RLogicConfig,
    CompiledLogicId, CompiledLogicStoreStats,
};
use serde::{Deserialize, Serialize};
pub use table_metadata::TableMetadata;
pub use path_utils::ArrayMetadata;
pub use eval_data::EvalData;
pub use eval_cache::{EvalCache, CacheKey, CacheStats};
pub use parsed_schema::ParsedSchema;
pub use parsed_schema_cache::{ParsedSchemaCache, ParsedSchemaCacheStats, PARSED_SCHEMA_CACHE};
use serde::de::Error as _;
use serde_json::{Value};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use std::mem;
use std::sync::{Arc, Mutex};

/// Get the library version
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Clean floating point noise from JSON values
/// Converts values very close to zero (< 1e-10) to exactly 0
fn clean_float_noise(value: Value) -> Value {
    const EPSILON: f64 = 1e-10;
    
    match value {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.abs() < EPSILON {
                    // Clean near-zero values to exactly 0
                    Value::Number(serde_json::Number::from(0))
                } else if f.fract().abs() < EPSILON {
                    // Clean whole numbers: 33.0 → 33
                    Value::Number(serde_json::Number::from(f.round() as i64))
                } else {
                    Value::Number(n)
                }
            } else {
                Value::Number(n)
            }
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(clean_float_noise).collect())
        }
        Value::Object(obj) => {
            Value::Object(obj.into_iter().map(|(k, v)| (k, clean_float_noise(v))).collect())
        }
        _ => value,
    }
}

/// Dependent item structure for transitive dependency tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependentItem {
    pub ref_path: String,
    pub clear: Option<Value>,  // Can be $evaluation or boolean
    pub value: Option<Value>,  // Can be $evaluation or primitive value
}

pub struct JSONEval {
    pub schema: Arc<Value>,
    pub engine: Arc<RLogic>,
    pub evaluations: IndexMap<String, LogicId>,
    pub tables: IndexMap<String, Value>,
    /// Pre-compiled table metadata (computed at parse time for zero-copy evaluation)
    pub table_metadata: IndexMap<String, TableMetadata>,
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
    /// Subforms: isolated JSONEval instances for array fields with items
    /// Key is the schema path (e.g., "#/riders"), value is the sub-JSONEval
    pub subforms: IndexMap<String, Box<JSONEval>>,
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
    eval_lock: Mutex<()>,
    /// Cached MessagePack bytes for zero-copy schema retrieval
    /// Stores original MessagePack if initialized from binary, cleared on schema mutations
    cached_msgpack_schema: Option<Vec<u8>>,
}

impl Clone for JSONEval {
    fn clone(&self) -> Self {
        Self {
            cache_enabled: self.cache_enabled,
            schema: Arc::clone(&self.schema),
            engine: Arc::clone(&self.engine),
            evaluations: self.evaluations.clone(),
            tables: self.tables.clone(),
            table_metadata: self.table_metadata.clone(),
            dependencies: self.dependencies.clone(),
            sorted_evaluations: self.sorted_evaluations.clone(),
            dependents_evaluations: self.dependents_evaluations.clone(),
            rules_evaluations: self.rules_evaluations.clone(),
            fields_with_rules: self.fields_with_rules.clone(),
            others_evaluations: self.others_evaluations.clone(),
            value_evaluations: self.value_evaluations.clone(),
            layout_paths: self.layout_paths.clone(),
            options_templates: self.options_templates.clone(),
            subforms: self.subforms.clone(),
            context: self.context.clone(),
            data: self.data.clone(),
            evaluated_schema: self.evaluated_schema.clone(),
            eval_data: self.eval_data.clone(),
            eval_cache: EvalCache::new(), // Create fresh cache for the clone
            eval_lock: Mutex::new(()), // Create fresh mutex for the clone
            cached_msgpack_schema: self.cached_msgpack_schema.clone(),
        }
    }
}

impl JSONEval {
    pub fn new(
        schema: &str,
        context: Option<&str>,
        data: Option<&str>,
    ) -> Result<Self, serde_json::Error> {
        // Use serde_json for schema (needs arbitrary_precision) and SIMD for data (needs speed)
        let schema_val: Value = serde_json::from_str(schema)?;
        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}")).map_err(serde_json::Error::custom)?;
        let data: Value = json_parser::parse_json_str(data.unwrap_or("{}")).map_err(serde_json::Error::custom)?;
        let evaluated_schema = schema_val.clone();
        // Use default config: tracking enabled
        let engine_config = RLogicConfig::default();

        let mut instance = Self {
            schema: Arc::new(schema_val),
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
            engine: Arc::new(RLogic::with_config(engine_config)),
            context: context.clone(),
            data: data.clone(),
            evaluated_schema: evaluated_schema.clone(),
            eval_data: EvalData::with_schema_data_context(&evaluated_schema, &data, &context),
            eval_cache: EvalCache::new(),
            cache_enabled: true, // Caching enabled by default
            eval_lock: Mutex::new(()),
            cached_msgpack_schema: None, // JSON initialization, no MessagePack cache
        };
        parse_schema::legacy::parse_schema(&mut instance).map_err(serde_json::Error::custom)?;
        Ok(instance)
    }

    /// Create a new JSONEval instance from MessagePack-encoded schema
    /// 
    /// # Arguments
    /// 
    /// * `schema_msgpack` - MessagePack-encoded schema bytes
    /// * `context` - Optional JSON context string
    /// * `data` - Optional JSON data string
    /// 
    /// # Returns
    /// 
    /// A Result containing the JSONEval instance or an error
    pub fn new_from_msgpack(
        schema_msgpack: &[u8],
        context: Option<&str>,
        data: Option<&str>,
    ) -> Result<Self, String> {
        // Store original MessagePack bytes for zero-copy retrieval
        let cached_msgpack = schema_msgpack.to_vec();
        
        // Deserialize MessagePack schema to Value
        let schema_val: Value = rmp_serde::from_slice(schema_msgpack)
            .map_err(|e| format!("Failed to deserialize MessagePack schema: {}", e))?;
        
        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}"))
            .map_err(|e| format!("Failed to parse context: {}", e))?;
        let data: Value = json_parser::parse_json_str(data.unwrap_or("{}"))
            .map_err(|e| format!("Failed to parse data: {}", e))?;
        let evaluated_schema = schema_val.clone();
        let engine_config = RLogicConfig::default();

        let mut instance = Self {
            schema: Arc::new(schema_val),
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
            engine: Arc::new(RLogic::with_config(engine_config)),
            context: context.clone(),
            data: data.clone(),
            evaluated_schema: evaluated_schema.clone(),
            eval_data: EvalData::with_schema_data_context(&evaluated_schema, &data, &context),
            eval_cache: EvalCache::new(),
            cache_enabled: true, // Caching enabled by default
            eval_lock: Mutex::new(()),
            cached_msgpack_schema: Some(cached_msgpack), // Store for zero-copy retrieval
        };
        parse_schema::legacy::parse_schema(&mut instance)?;
        Ok(instance)
    }

    /// Create a new JSONEval instance from a pre-parsed ParsedSchema
    /// 
    /// This enables schema caching: parse once, reuse across multiple evaluations with different data/context.
    /// 
    /// # Arguments
    /// 
    /// * `parsed` - Arc-wrapped pre-parsed schema (can be cloned and cached)
    /// * `context` - Optional JSON context string
    /// * `data` - Optional JSON data string
    /// 
    /// # Returns
    /// 
    /// A Result containing the JSONEval instance or an error
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// use std::sync::Arc;
    /// 
    /// // Parse schema once and wrap in Arc for caching
    /// let parsed = Arc::new(ParsedSchema::parse(schema_str)?);
    /// cache.insert(schema_key, parsed.clone());
    /// 
    /// // Reuse across multiple evaluations (Arc::clone is cheap)
    /// let eval1 = JSONEval::with_parsed_schema(parsed.clone(), Some(context1), Some(data1))?;
    /// let eval2 = JSONEval::with_parsed_schema(parsed.clone(), Some(context2), Some(data2))?;
    /// ```
    pub fn with_parsed_schema(
        parsed: Arc<ParsedSchema>,
        context: Option<&str>,
        data: Option<&str>,
    ) -> Result<Self, String> {
        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}"))
            .map_err(|e| format!("Failed to parse context: {}", e))?;
        let data: Value = json_parser::parse_json_str(data.unwrap_or("{}"))
            .map_err(|e| format!("Failed to parse data: {}", e))?;
        
        let evaluated_schema = parsed.schema.clone();
        
        // Share the engine Arc (cheap pointer clone, not data clone)
        // Multiple JSONEval instances created from the same ParsedSchema will share the compiled RLogic
        let engine = parsed.engine.clone();
        
        // Convert Arc<ParsedSchema> subforms to Box<JSONEval> subforms
        // This is a one-time conversion when creating JSONEval from ParsedSchema
        let mut subforms = IndexMap::new();
        for (path, subform_parsed) in &parsed.subforms {
            // Create JSONEval from the cached ParsedSchema
            let subform_eval = JSONEval::with_parsed_schema(
                subform_parsed.clone(),
                Some("{}"),
                None
            )?;
            subforms.insert(path.clone(), Box::new(subform_eval));
        }
        
        let instance = Self {
            schema: Arc::clone(&parsed.schema),
            evaluations: parsed.evaluations.clone(),
            tables: parsed.tables.clone(),
            table_metadata: parsed.table_metadata.clone(),
            dependencies: parsed.dependencies.clone(),
            sorted_evaluations: parsed.sorted_evaluations.clone(),
            dependents_evaluations: parsed.dependents_evaluations.clone(),
            rules_evaluations: parsed.rules_evaluations.clone(),
            fields_with_rules: parsed.fields_with_rules.clone(),
            others_evaluations: parsed.others_evaluations.clone(),
            value_evaluations: parsed.value_evaluations.clone(),
            layout_paths: parsed.layout_paths.clone(),
            options_templates: parsed.options_templates.clone(),
            subforms,
            engine,
            context: context.clone(),
            data: data.clone(),
            evaluated_schema: (*evaluated_schema).clone(),
            eval_data: EvalData::with_schema_data_context(&evaluated_schema, &data, &context),
            eval_cache: EvalCache::new(),
            cache_enabled: true, // Caching enabled by default
            eval_lock: Mutex::new(()),
            cached_msgpack_schema: None, // No MessagePack cache for parsed schema
        };
        
        Ok(instance)
    }

    pub fn reload_schema(
        &mut self,
        schema: &str,
        context: Option<&str>,
        data: Option<&str>,
    ) -> Result<(), String> {
        // Use serde_json for schema (precision) and SIMD for data (speed)
        let schema_val: Value = serde_json::from_str(schema).map_err(|e| format!("failed to parse schema: {e}"))?;
        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}"))?;
        let data: Value = json_parser::parse_json_str(data.unwrap_or("{}"))?;
        self.schema = Arc::new(schema_val);
        self.context = context.clone();
        self.data = data.clone();
        self.evaluated_schema = (*self.schema).clone();
        self.engine = Arc::new(RLogic::new());
        self.dependents_evaluations.clear();
        self.rules_evaluations.clear();
        self.fields_with_rules.clear();
        self.others_evaluations.clear();
        self.value_evaluations.clear();
        self.layout_paths.clear();
        self.options_templates.clear();
        self.subforms.clear();
        parse_schema::legacy::parse_schema(self)?;
        
        // Re-initialize eval_data with new schema, data, and context
        self.eval_data = EvalData::with_schema_data_context(&self.evaluated_schema, &data, &context);
        
        // Clear cache when schema changes
        self.eval_cache.clear();
        
        // Clear MessagePack cache since schema has been mutated
        self.cached_msgpack_schema = None;

        Ok(())
    }

    /// Reload schema from MessagePack-encoded bytes
    /// 
    /// # Arguments
    /// 
    /// * `schema_msgpack` - MessagePack-encoded schema bytes
    /// * `context` - Optional context data JSON string
    /// * `data` - Optional initial data JSON string
    /// 
    /// # Returns
    /// 
    /// A `Result` indicating success or an error message
    pub fn reload_schema_msgpack(
        &mut self,
        schema_msgpack: &[u8],
        context: Option<&str>,
        data: Option<&str>,
    ) -> Result<(), String> {
        // Deserialize MessagePack to Value
        let schema_val: Value = rmp_serde::from_slice(schema_msgpack)
            .map_err(|e| format!("failed to deserialize MessagePack schema: {e}"))?;
        
        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}"))?;
        let data: Value = json_parser::parse_json_str(data.unwrap_or("{}"))?;
        
        self.schema = Arc::new(schema_val);
        self.context = context.clone();
        self.data = data.clone();
        self.evaluated_schema = (*self.schema).clone();
        self.engine = Arc::new(RLogic::new());
        self.dependents_evaluations.clear();
        self.rules_evaluations.clear();
        self.fields_with_rules.clear();
        self.others_evaluations.clear();
        self.value_evaluations.clear();
        self.layout_paths.clear();
        self.options_templates.clear();
        self.subforms.clear();
        parse_schema::legacy::parse_schema(self)?;
        
        // Re-initialize eval_data
        self.eval_data = EvalData::with_schema_data_context(&self.evaluated_schema, &data, &context);
        
        // Clear cache when schema changes
        self.eval_cache.clear();
        
        // Cache the MessagePack for future retrievals
        self.cached_msgpack_schema = Some(schema_msgpack.to_vec());

        Ok(())
    }

    /// Reload schema from a cached ParsedSchema
    /// 
    /// This is the most efficient way to reload as it reuses pre-parsed schema compilation.
    /// 
    /// # Arguments
    /// 
    /// * `parsed` - Arc reference to a cached ParsedSchema
    /// * `context` - Optional context data JSON string
    /// * `data` - Optional initial data JSON string
    /// 
    /// # Returns
    /// 
    /// A `Result` indicating success or an error message
    pub fn reload_schema_parsed(
        &mut self,
        parsed: Arc<ParsedSchema>,
        context: Option<&str>,
        data: Option<&str>,
    ) -> Result<(), String> {
        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}"))?;
        let data: Value = json_parser::parse_json_str(data.unwrap_or("{}"))?;
        
        // Share all the pre-compiled data from ParsedSchema
        self.schema = Arc::clone(&parsed.schema);
        self.evaluations = parsed.evaluations.clone();
        self.tables = parsed.tables.clone();
        self.table_metadata = parsed.table_metadata.clone();
        self.dependencies = parsed.dependencies.clone();
        self.sorted_evaluations = parsed.sorted_evaluations.clone();
        self.dependents_evaluations = parsed.dependents_evaluations.clone();
        self.rules_evaluations = parsed.rules_evaluations.clone();
        self.fields_with_rules = parsed.fields_with_rules.clone();
        self.others_evaluations = parsed.others_evaluations.clone();
        self.value_evaluations = parsed.value_evaluations.clone();
        self.layout_paths = parsed.layout_paths.clone();
        self.options_templates = parsed.options_templates.clone();
        
        // Share the engine Arc (cheap pointer clone, not data clone)
        self.engine = parsed.engine.clone();
        
        // Convert Arc<ParsedSchema> subforms to Box<JSONEval> subforms
        let mut subforms = IndexMap::new();
        for (path, subform_parsed) in &parsed.subforms {
            let subform_eval = JSONEval::with_parsed_schema(
                subform_parsed.clone(),
                Some("{}"),
                None
            )?;
            subforms.insert(path.clone(), Box::new(subform_eval));
        }
        self.subforms = subforms;
        
        self.context = context.clone();
        self.data = data.clone();
        self.evaluated_schema = (*self.schema).clone();
        
        // Re-initialize eval_data
        self.eval_data = EvalData::with_schema_data_context(&self.evaluated_schema, &data, &context);
        
        // Clear cache when schema changes
        self.eval_cache.clear();
        
        // Clear MessagePack cache since we're loading from ParsedSchema
        self.cached_msgpack_schema = None;

        Ok(())
    }

    /// Reload schema from ParsedSchemaCache using a cache key
    /// 
    /// This is the recommended way for cross-platform cached schema reloading.
    /// 
    /// # Arguments
    /// 
    /// * `cache_key` - Key to lookup in the global ParsedSchemaCache
    /// * `context` - Optional context data JSON string
    /// * `data` - Optional initial data JSON string
    /// 
    /// # Returns
    /// 
    /// A `Result` indicating success or an error message
    pub fn reload_schema_from_cache(
        &mut self,
        cache_key: &str,
        context: Option<&str>,
        data: Option<&str>,
    ) -> Result<(), String> {
        // Get the cached ParsedSchema from global cache
        let parsed = PARSED_SCHEMA_CACHE.get(cache_key)
            .ok_or_else(|| format!("Schema '{}' not found in cache", cache_key))?;
        
        // Use reload_schema_parsed with the cached schema
        self.reload_schema_parsed(parsed, context, data)
    }

    /// Evaluate the schema with the given data and context.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to evaluate.
    /// * `context` - The context to evaluate.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error message.
    pub fn evaluate(&mut self, data: &str, context: Option<&str>) -> Result<(), String> {
        // Use SIMD-accelerated JSON parsing
        let data: Value = json_parser::parse_json_str(data)?;
        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}"))?;
            
        self.data = data.clone();
        
        // Collect top-level data keys to selectively purge cache
        let changed_data_paths: Vec<String> = if let Some(obj) = data.as_object() {
            obj.keys().map(|k| k.clone()).collect()
        } else {
            Vec::new()
        };
        
        // Replace data and context in existing eval_data
        self.eval_data.replace_data_and_context(data, context);
        
        // Selectively purge cache entries that depend on changed top-level data keys
        // This is more efficient than clearing entire cache
        self.purge_cache_for_changed_data(&changed_data_paths);
        
        // Call internal evaluate (uses existing data if not provided)
        self.evaluate_internal()
    }
    
    /// Internal evaluate that can be called when data is already set
    /// This avoids double-locking and unnecessary data cloning for re-evaluation from evaluate_dependents
    fn evaluate_internal(&mut self) -> Result<(), String> {
        // Acquire lock for synchronous execution
        let _lock = self.eval_lock.lock().unwrap();

        // Clone sorted_evaluations (batches) to avoid borrow checker issues
        let eval_batches: Vec<Vec<String>> = self.sorted_evaluations.clone();

        // Process each batch - parallelize evaluations within each batch
        // Batches are processed sequentially to maintain dependency order
        for batch in eval_batches {
            // Skip empty batches
            if batch.is_empty() {
                continue;
            }
            
            // No pre-checking cache - we'll check inside parallel execution
            // This allows thread-safe cache access during parallel evaluation
            
            // Parallel execution within batch (no dependencies between items)
            // Use Mutex for thread-safe result collection
            // Store both eval_key and result for cache storage
            let eval_data_snapshot = self.eval_data.clone();
            
            // Parallelize only if batch has multiple items (overhead not worth it for single item)
            #[cfg(feature = "parallel")]
            if batch.len() > 1 {
                let results: Mutex<Vec<(String, String, Value)>> = Mutex::new(Vec::with_capacity(batch.len()));
                batch.par_iter().for_each(|eval_key| {
                    let pointer_path = path_utils::normalize_to_json_pointer(eval_key);
                    
                    // Try cache first (thread-safe)
                    if let Some(_) = self.try_get_cached(eval_key, &eval_data_snapshot) {
                        return;
                    }
                    
                    // Cache miss - evaluate
                    let is_table = self.table_metadata.contains_key(eval_key);
                    
                    if is_table {
                        // Evaluate table using sandboxed metadata (parallel-safe, immutable parent scope)
                        if let Ok(rows) = table_evaluate::evaluate_table(self, eval_key, &eval_data_snapshot) {
                            let value = Value::Array(rows);
                            // Cache result (thread-safe)
                            self.cache_result(eval_key, Value::Null, &eval_data_snapshot);
                            results.lock().unwrap().push((eval_key.clone(), pointer_path, value));
                        }
                    } else {
                        if let Some(logic_id) = self.evaluations.get(eval_key) {
                            // Evaluate directly with snapshot
                            if let Ok(val) = self.engine.run(logic_id, eval_data_snapshot.data()) {
                                let cleaned_val = clean_float_noise(val);
                                // Cache result (thread-safe)
                                self.cache_result(eval_key, Value::Null, &eval_data_snapshot);
                                results.lock().unwrap().push((eval_key.clone(), pointer_path, cleaned_val));
                            }
                        }
                    }
                });
                
                // Write all results back sequentially (already cached in parallel execution)
                for (_eval_key, path, value) in results.into_inner().unwrap() {
                    let cleaned_value = clean_float_noise(value);
                    
                    self.eval_data.set(&path, cleaned_value.clone());
                    // Also write to evaluated_schema
                    if let Some(schema_value) = self.evaluated_schema.pointer_mut(&path) {
                        *schema_value = cleaned_value;
                    }
                }
                continue;
            }
            
            // Sequential execution (single item or parallel feature disabled)
            #[cfg(not(feature = "parallel"))]
            let batch_items = &batch;
            
            #[cfg(feature = "parallel")]
            let batch_items = if batch.len() > 1 { &batch[0..0] } else { &batch }; // Empty slice if already processed in parallel
            
            for eval_key in batch_items {
                let pointer_path = path_utils::normalize_to_json_pointer(eval_key);
                
                // Try cache first
                if let Some(_) = self.try_get_cached(eval_key, &eval_data_snapshot) {
                    continue;
                }
                
                // Cache miss - evaluate
                let is_table = self.table_metadata.contains_key(eval_key);
                
                if is_table {
                    if let Ok(rows) = table_evaluate::evaluate_table(self, eval_key, &eval_data_snapshot) {
                        let value = Value::Array(rows);
                        // Cache result
                        self.cache_result(eval_key, Value::Null, &eval_data_snapshot);
                        
                        let cleaned_value = clean_float_noise(value);
                        self.eval_data.set(&pointer_path, cleaned_value.clone());
                        if let Some(schema_value) = self.evaluated_schema.pointer_mut(&pointer_path) {
                            *schema_value = cleaned_value;
                        }
                    }
                } else {
                    if let Some(logic_id) = self.evaluations.get(eval_key) {
                        if let Ok(val) = self.engine.run(logic_id, eval_data_snapshot.data()) {
                            let cleaned_val = clean_float_noise(val);
                            // Cache result
                            self.cache_result(eval_key, Value::Null, &eval_data_snapshot);
                            
                            self.eval_data.set(&pointer_path, cleaned_val.clone());
                            if let Some(schema_value) = self.evaluated_schema.pointer_mut(&pointer_path) {
                                *schema_value = cleaned_val;
                            }
                        }
                    }
                }
            }
        }

        // Drop lock before calling evaluate_others
        drop(_lock);
        
        self.evaluate_others();

        Ok(())
    }

    /// Get the evaluated schema with optional layout resolution.
    ///
    /// # Arguments
    ///
    /// * `skip_layout` - Whether to skip layout resolution.
    ///
    /// # Returns
    ///
    /// The evaluated schema as a JSON value.
    pub fn get_evaluated_schema(&mut self, skip_layout: bool) -> Value {
        if !skip_layout {
            self.resolve_layout_internal();
        }
        
        self.evaluated_schema.clone()
    }

    /// Get the evaluated schema as MessagePack binary format
    ///
    /// # Arguments
    ///
    /// * `skip_layout` - Whether to skip layout resolution.
    ///
    /// # Returns
    ///
    /// The evaluated schema serialized as MessagePack bytes
    ///
    /// # Zero-Copy Optimization
    ///
    /// This method serializes the evaluated schema to MessagePack. The resulting Vec<u8>
    /// is then passed to FFI/WASM boundaries via raw pointers (zero-copy at boundary).
    /// The serialization step itself (Value -> MessagePack) cannot be avoided but is
    /// highly optimized by rmp-serde.
    pub fn get_evaluated_schema_msgpack(&mut self, skip_layout: bool) -> Result<Vec<u8>, String> {
        if !skip_layout {
            self.resolve_layout_internal();
        }
        
        // Serialize evaluated schema to MessagePack
        // Note: This is the only copy required. The FFI layer then returns raw pointers
        // to this data for zero-copy transfer to calling code.
        rmp_serde::to_vec(&self.evaluated_schema)
            .map_err(|e| format!("Failed to serialize schema to MessagePack: {}", e))
    }

    /// Get all schema values (evaluations ending with .value)
    /// Mutates self.data by overriding with values from value evaluations
    /// Returns the modified data
    pub fn get_schema_value(&mut self) -> Value {
        // Ensure self.data is an object
        if !self.data.is_object() {
            self.data = Value::Object(serde_json::Map::new());
        }
        
        // Override self.data with values from value evaluations
        for eval_key in &self.value_evaluations.clone() {
            let clean_key = eval_key.replace("#", "");
            let path = clean_key.replace("/properties", "").replace("/value", "");
            
            // Get the value from evaluated_schema
            let value = match self.evaluated_schema.pointer(&clean_key) {
                Some(v) => v.clone(),
                None => continue,
            };
            
            // Parse the path and create nested structure as needed
            let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            
            if path_parts.is_empty() {
                continue;
            }
            
            // Navigate/create nested structure
            let mut current = &mut self.data;
            for (i, part) in path_parts.iter().enumerate() {
                let is_last = i == path_parts.len() - 1;
                
                if is_last {
                    // Set the value at the final key
                    if let Some(obj) = current.as_object_mut() {
                        obj.insert(part.to_string(), clean_float_noise(value.clone()));
                    }
                } else {
                    // Ensure current is an object, then navigate/create intermediate objects
                    if let Some(obj) = current.as_object_mut() {
                        current = obj.entry(part.to_string())
                            .or_insert_with(|| Value::Object(serde_json::Map::new()));
                    } else {
                        // Skip this path if current is not an object and can't be made into one
                        break;
                    }
                }
            }
        }
        
        clean_float_noise(self.data.clone())
    }

    /// Get the evaluated schema without $params field.
    /// This method filters out $params from the root level only.
    ///
    /// # Arguments
    ///
    /// * `skip_layout` - Whether to skip layout resolution.
    ///
    /// # Returns
    ///
    /// The evaluated schema with $params removed.
    pub fn get_evaluated_schema_without_params(&mut self, skip_layout: bool) -> Value {
        if !skip_layout {
            self.resolve_layout_internal();
        }
        
        // Filter $params at root level only
        if let Value::Object(mut map) = self.evaluated_schema.clone() {
            map.remove("$params");
            Value::Object(map)
        } else {
            self.evaluated_schema.clone()
        }
    }

    /// Get a value from the evaluated schema using dotted path notation.
    /// Converts dotted notation (e.g., "properties.field.value") to JSON pointer format.
    ///
    /// # Arguments
    ///
    /// * `path` - The dotted path to the value (e.g., "properties.field.value")
    /// * `skip_layout` - Whether to skip layout resolution.
    ///
    /// # Returns
    ///
    /// The value at the specified path, or None if not found.
    pub fn get_evaluated_schema_by_path(&mut self, path: &str, skip_layout: bool) -> Option<Value> {
        if !skip_layout {
            self.resolve_layout_internal();
        }
        
        // Convert dotted notation to JSON pointer
        let pointer = if path.is_empty() {
            "".to_string()
        } else {
            format!("/{}", path.replace(".", "/"))
        };
        
        self.evaluated_schema.pointer(&pointer).cloned()
    }

    /// Get a value from the schema using dotted path notation.
    /// Converts dotted notation (e.g., "properties.field.value") to JSON pointer format.
    ///
    /// # Arguments
    ///
    /// * `path` - The dotted path to the value (e.g., "properties.field.value")
    ///
    /// # Returns
    ///
    /// The value at the specified path, or None if not found.
    pub fn get_schema_by_path(&self, path: &str) -> Option<Value> {
        // Convert dotted notation to JSON pointer
        let pointer = if path.is_empty() {
            "".to_string()
        } else {
            format!("/{}", path.replace(".", "/"))
        };
        
        self.schema.pointer(&pointer).cloned()
    }

    /// Check if a dependency should be cached
    /// Caches everything except keys starting with $ (except $context)
    #[inline]
    fn should_cache_dependency(key: &str) -> bool {
        if key.starts_with("/$") || key.starts_with('$') {
            // Only cache $context, exclude other $ keys like $params
            key == "$context" || key.starts_with("$context.") || key.starts_with("/$context")
        } else {
            true
        }
    }

    /// Helper: Try to get cached result for an evaluation (thread-safe)
    /// Helper: Try to get cached result (zero-copy via Arc)
    fn try_get_cached(&self, eval_key: &str, eval_data: &EvalData) -> Option<Value> {
        // Skip cache lookup if caching is disabled
        if !self.cache_enabled {
            return None;
        }
        
        // Get dependencies for this evaluation
        let deps = self.dependencies.get(eval_key)?;
        
        // If no dependencies, use simple cache key
        let cache_key = if deps.is_empty() {
            CacheKey::simple(eval_key.to_string())
        } else {
            // Filter dependencies (exclude $ keys except $context)
            let filtered_deps: IndexSet<String> = deps
                .iter()
                .filter(|dep_key| JSONEval::should_cache_dependency(dep_key))
                .cloned()
                .collect();
            
            // Collect dependency values
            let dep_values: Vec<(String, &Value)> = filtered_deps
                .iter()
                .filter_map(|dep_key| {
                    eval_data.get(dep_key).map(|v| (dep_key.clone(), v))
                })
                .collect();
            
            CacheKey::new(eval_key.to_string(), &filtered_deps, &dep_values)
        };
        
        // Try cache lookup (zero-copy via Arc, thread-safe)
        self.eval_cache.get(&cache_key).map(|arc_val| (*arc_val).clone())
    }
    
    /// Helper: Store evaluation result in cache (thread-safe)
    fn cache_result(&self, eval_key: &str, value: Value, eval_data: &EvalData) {
        // Skip cache insertion if caching is disabled
        if !self.cache_enabled {
            return;
        }
        
        // Get dependencies for this evaluation
        let deps = match self.dependencies.get(eval_key) {
            Some(d) => d,
            None => {
                // No dependencies - use simple cache key
                let cache_key = CacheKey::simple(eval_key.to_string());
                self.eval_cache.insert(cache_key, value);
                return;
            }
        };
        
        // Filter and collect dependency values (exclude $ keys except $context)
        let filtered_deps: IndexSet<String> = deps
            .iter()
            .filter(|dep_key| JSONEval::should_cache_dependency(dep_key))
            .cloned()
            .collect();
        
        let dep_values: Vec<(String, &Value)> = filtered_deps
            .iter()
            .filter_map(|dep_key| {
                eval_data.get(dep_key).map(|v| (dep_key.clone(), v))
            })
            .collect();
        
        let cache_key = CacheKey::new(eval_key.to_string(), &filtered_deps, &dep_values);
        self.eval_cache.insert(cache_key, value);
    }

    /// Selectively purge cache entries that depend on changed data paths
    /// Only removes cache entries whose dependencies intersect with changed_paths
    /// Compares old vs new values and only purges if values actually changed
    fn purge_cache_for_changed_data_with_comparison(
        &self, 
        changed_data_paths: &[String],
        old_data: &Value,
        new_data: &Value
    ) {
        if changed_data_paths.is_empty() {
            return;
        }
        
        // Check which paths actually have different values
        let mut actually_changed_paths = Vec::new();
        for path in changed_data_paths {
            let old_val = old_data.pointer(path);
            let new_val = new_data.pointer(path);
            
            // Only add to changed list if values differ
            if old_val != new_val {
                actually_changed_paths.push(path.clone());
            }
        }
        
        // If no values actually changed, no need to purge
        if actually_changed_paths.is_empty() {
            return;
        }
        
        // Find all eval_keys that depend on the actually changed data paths
        let mut affected_eval_keys = IndexSet::new();
        
        for (eval_key, deps) in self.dependencies.iter() {
            // Check if this evaluation depends on any of the changed paths
            let is_affected = deps.iter().any(|dep| {
                // Check if the dependency matches any changed path
                actually_changed_paths.iter().any(|changed_path| {
                    // Exact match or prefix match (for nested fields)
                    dep == changed_path || 
                    dep.starts_with(&format!("{}/", changed_path)) ||
                    changed_path.starts_with(&format!("{}/", dep))
                })
            });
            
            if is_affected {
                affected_eval_keys.insert(eval_key.clone());
            }
        }
        
        // Remove all cache entries for affected eval_keys using retain
        // Keep entries whose eval_key is NOT in the affected set
        self.eval_cache.retain(|cache_key, _| {
            !affected_eval_keys.contains(&cache_key.eval_key)
        });
    }
    
    /// Selectively purge cache entries that depend on changed data paths
    /// Simpler version without value comparison for cases where we don't have old data
    fn purge_cache_for_changed_data(&self, changed_data_paths: &[String]) {
        if changed_data_paths.is_empty() {
            return;
        }
        
        // Find all eval_keys that depend on the changed data paths
        let mut affected_eval_keys = IndexSet::new();
        
        for (eval_key, deps) in self.dependencies.iter() {
            // Check if this evaluation depends on any of the changed paths
            let is_affected = deps.iter().any(|dep| {
                // Check if the dependency matches any changed path
                changed_data_paths.iter().any(|changed_path| {
                    // Exact match or prefix match (for nested fields)
                    dep == changed_path || 
                    dep.starts_with(&format!("{}/", changed_path)) ||
                    changed_path.starts_with(&format!("{}/", dep))
                })
            });
            
            if is_affected {
                affected_eval_keys.insert(eval_key.clone());
            }
        }
        
        // Remove all cache entries for affected eval_keys using retain
        // Keep entries whose eval_key is NOT in the affected set
        self.eval_cache.retain(|cache_key, _| {
            !affected_eval_keys.contains(&cache_key.eval_key)
        });
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.eval_cache.stats()
    }
    
    /// Clear evaluation cache
    pub fn clear_cache(&mut self) {
        self.eval_cache.clear();
    }
    
    /// Get number of cached entries
    pub fn cache_len(&self) -> usize {
        self.eval_cache.len()
    }
    
    /// Enable evaluation caching
    /// Useful for reusing JSONEval instances with different data
    pub fn enable_cache(&mut self) {
        self.cache_enabled = true;
    }
    
    /// Disable evaluation caching
    /// Useful for web API usage where each request creates a new JSONEval instance
    /// Improves performance by skipping cache operations that have no benefit for single-use instances
    pub fn disable_cache(&mut self) {
        self.cache_enabled = false;
        self.eval_cache.clear(); // Clear any existing cache entries
    }
    
    /// Check if caching is enabled
    pub fn is_cache_enabled(&self) -> bool {
        self.cache_enabled
    }

    fn evaluate_others(&mut self) {
        // Step 1: Evaluate options URL templates (handles {variable} patterns)
        self.evaluate_options_templates();
        
        // Step 2: Evaluate "rules" and "others" categories with caching
        // Rules are evaluated here so their values are available in evaluated_schema
        let combined_count = self.rules_evaluations.len() + self.others_evaluations.len();
        if combined_count == 0 {
            return;
        }
        
        let eval_data_snapshot = self.eval_data.clone();
        
        #[cfg(feature = "parallel")]
        {
            let combined_results: Mutex<Vec<(String, Value)>> = Mutex::new(Vec::with_capacity(combined_count));
            
            self.rules_evaluations
                .par_iter()
                .chain(self.others_evaluations.par_iter())
                .for_each(|eval_key| {
                    let pointer_path = path_utils::normalize_to_json_pointer(eval_key);

                    // Try cache first (thread-safe)
                    if let Some(_) = self.try_get_cached(eval_key, &eval_data_snapshot) {
                        return;
                    }

                    // Cache miss - evaluate
                    if let Some(logic_id) = self.evaluations.get(eval_key) {
                        if let Ok(val) = self.engine.run(logic_id, eval_data_snapshot.data()) {
                            let cleaned_val = clean_float_noise(val);
                            // Cache result (thread-safe)
                            self.cache_result(eval_key, Value::Null, &eval_data_snapshot);
                            combined_results.lock().unwrap().push((pointer_path, cleaned_val));
                        }
                    }
                });

            // Write results to evaluated_schema
            for (result_path, value) in combined_results.into_inner().unwrap() {
                if let Some(pointer_value) = self.evaluated_schema.pointer_mut(&result_path) {
                    // Special handling for rules with $evaluation
                    // This includes both direct rules and array items: /rules/evaluation/0/$evaluation
                    if !result_path.starts_with("$") && result_path.contains("/rules/") && !result_path.ends_with("/value") {
                        match pointer_value.as_object_mut() {
                            Some(pointer_obj) => {
                                pointer_obj.remove("$evaluation");
                                pointer_obj.insert("value".to_string(), value);
                            },
                            None => continue,
                        }
                    } else {
                        *pointer_value = value;
                    }
                }
            }
        }
        
        #[cfg(not(feature = "parallel"))]
        {
            // Sequential evaluation
            for eval_key in self.rules_evaluations.iter().chain(self.others_evaluations.iter()) {
                let pointer_path = path_utils::normalize_to_json_pointer(eval_key);
                
                // Try cache first
                if let Some(_) = self.try_get_cached(eval_key, &eval_data_snapshot) {
                    continue;
                }
                
                // Cache miss - evaluate
                if let Some(logic_id) = self.evaluations.get(eval_key) {
                    if let Ok(val) = self.engine.run(logic_id, eval_data_snapshot.data()) {
                        let cleaned_val = clean_float_noise(val);
                        // Cache result
                        self.cache_result(eval_key, Value::Null, &eval_data_snapshot);
                        
                        if let Some(pointer_value) = self.evaluated_schema.pointer_mut(&pointer_path) {
                            if !pointer_path.starts_with("$") && pointer_path.contains("/rules/") && !pointer_path.ends_with("/value") {
                                match pointer_value.as_object_mut() {
                                    Some(pointer_obj) => {
                                        pointer_obj.remove("$evaluation");
                                        pointer_obj.insert("value".to_string(), cleaned_val);
                                    },
                                    None => continue,
                                }
                            } else {
                                *pointer_value = cleaned_val;
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Evaluate options URL templates (handles {variable} patterns)
    fn evaluate_options_templates(&mut self) {
        // Use pre-collected options templates from parsing (no clone, no recursion)
        let templates_to_eval = self.options_templates.clone();
        
        // Evaluate each template
        for (path, template_str, params_path) in templates_to_eval {
            if let Some(params) = self.evaluated_schema.pointer(&params_path) {
                if let Ok(evaluated) = self.evaluate_template(&template_str, params) {
                    if let Some(target) = self.evaluated_schema.pointer_mut(&path) {
                        *target = Value::String(evaluated);
                    }
                }
            }
        }
    }
    
    /// Evaluate a template string like "api/users/{id}" with params
    fn evaluate_template(&self, template: &str, params: &Value) -> Result<String, String> {
        let mut result = template.to_string();
        
        // Simple template evaluation: replace {key} with params.key
        if let Value::Object(params_map) = params {
            for (key, value) in params_map {
                let placeholder = format!("{{{}}}", key);
                if let Some(str_val) = value.as_str() {
                    result = result.replace(&placeholder, str_val);
                } else {
                    // Convert non-string values to strings
                    result = result.replace(&placeholder, &value.to_string());
                }
            }
        }
        
        Ok(result)
    }

    /// Compile a logic expression from a JSON string and store it globally
    /// 
    /// Returns a CompiledLogicId that can be used with run_logic for zero-clone evaluation.
    /// The compiled logic is stored in a global thread-safe cache and can be shared across
    /// different JSONEval instances. If the same logic was compiled before, returns the existing ID.
    /// 
    /// For repeated evaluations with different data, compile once and run multiple times.
    ///
    /// # Arguments
    ///
    /// * `logic_str` - JSON logic expression as a string
    ///
    /// # Returns
    ///
    /// A CompiledLogicId that can be reused for multiple evaluations across instances
    pub fn compile_logic(&self, logic_str: &str) -> Result<CompiledLogicId, String> {
        rlogic::compiled_logic_store::compile_logic(logic_str)
    }
    
    /// Compile a logic expression from a Value and store it globally
    /// 
    /// This is more efficient than compile_logic when you already have a parsed Value,
    /// as it avoids the JSON string serialization/parsing overhead.
    /// 
    /// Returns a CompiledLogicId that can be used with run_logic for zero-clone evaluation.
    /// The compiled logic is stored in a global thread-safe cache and can be shared across
    /// different JSONEval instances. If the same logic was compiled before, returns the existing ID.
    ///
    /// # Arguments
    ///
    /// * `logic` - JSON logic expression as a Value
    ///
    /// # Returns
    ///
    /// A CompiledLogicId that can be reused for multiple evaluations across instances
    pub fn compile_logic_value(&self, logic: &Value) -> Result<CompiledLogicId, String> {
        rlogic::compiled_logic_store::compile_logic_value(logic)
    }
    
    /// Run pre-compiled logic with zero-clone pattern
    /// 
    /// Uses references to avoid data cloning - similar to evaluate method.
    /// This is the most efficient way to evaluate logic multiple times with different data.
    /// The CompiledLogicId is retrieved from global storage, allowing the same compiled logic
    /// to be used across different JSONEval instances.
    ///
    /// # Arguments
    ///
    /// * `logic_id` - Pre-compiled logic ID from compile_logic
    /// * `data` - Optional data to evaluate against (uses existing data if None)
    /// * `context` - Optional context to use (uses existing context if None)
    ///
    /// # Returns
    ///
    /// The result of the evaluation as a Value
    pub fn run_logic(&mut self, logic_id: CompiledLogicId, data: Option<&Value>, context: Option<&Value>) -> Result<Value, String> {
        // Get compiled logic from global store
        let compiled_logic = rlogic::compiled_logic_store::get_compiled_logic(logic_id)
            .ok_or_else(|| format!("Compiled logic ID {:?} not found in store", logic_id))?;
        
        // Get the data to evaluate against
        // If custom data is provided, merge it with context and $params
        // Otherwise, use the existing eval_data which already has everything merged
        let eval_data_value = if let Some(input_data) = data {
            let context_value = context.unwrap_or(&self.context);
            
            self.eval_data.replace_data_and_context(input_data.clone(), context_value.clone());
            self.eval_data.data()
        } else {
            self.eval_data.data()
        };
        
        // Create an evaluator and run the pre-compiled logic with zero-clone pattern
        let evaluator = Evaluator::new();
        let result = evaluator.evaluate(&compiled_logic, &eval_data_value)?;
        
        Ok(clean_float_noise(result))
    }
    
    /// Compile and run JSON logic in one step (convenience method)
    /// 
    /// This is a convenience wrapper that combines compile_logic and run_logic.
    /// For repeated evaluations with different data, use compile_logic once 
    /// and run_logic multiple times for better performance.
    ///
    /// # Arguments
    ///
    /// * `logic_str` - JSON logic expression as a string
    /// * `data` - Optional data JSON string to evaluate against (uses existing data if None)
    /// * `context` - Optional context JSON string to use (uses existing context if None)
    ///
    /// # Returns
    ///
    /// The result of the evaluation as a Value
    pub fn compile_and_run_logic(&mut self, logic_str: &str, data: Option<&str>, context: Option<&str>) -> Result<Value, String> {
        // Parse the logic string and compile
        let compiled_logic = self.compile_logic(logic_str)?;
        
        // Parse data and context if provided
        let data_value = if let Some(data_str) = data {
            Some(json_parser::parse_json_str(data_str)?)
        } else {
            None
        };
        
        let context_value = if let Some(ctx_str) = context {
            Some(json_parser::parse_json_str(ctx_str)?)
        } else {
            None
        };
        
        // Run the compiled logic
        self.run_logic(compiled_logic, data_value.as_ref(), context_value.as_ref())
    }

    /// Resolve layout references with optional evaluation
    ///
    /// # Arguments
    ///
    /// * `evaluate` - If true, runs evaluation before resolving layout. If false, only resolves layout.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error message.
    pub fn resolve_layout(&mut self, evaluate: bool) -> Result<(), String> {
        if evaluate {
            // Use existing data
            let data_str = serde_json::to_string(&self.data)
                .map_err(|e| format!("Failed to serialize data: {}", e))?;
            self.evaluate(&data_str, None)?;
        }
        
        self.resolve_layout_internal();
        Ok(())
    }
    
    fn resolve_layout_internal(&mut self) {
        // Use cached layout paths (collected at parse time)
        // Clone to avoid borrow checker issues
        let layout_paths = self.layout_paths.clone();
        
        for layout_path in &layout_paths {
            self.resolve_layout_elements(layout_path);
        }
        
        // After resolving all references, propagate parent hidden/disabled to children
        for layout_path in &layout_paths {
            self.propagate_parent_conditions(layout_path);
        }
    }
    
    /// Propagate parent hidden/disabled conditions to children recursively
    fn propagate_parent_conditions(&mut self, layout_elements_path: &str) {
        // Normalize path from schema format (#/) to JSON pointer format (/)
        let normalized_path = path_utils::normalize_to_json_pointer(layout_elements_path);
        
        // Extract elements array to avoid borrow checker issues
        let elements = if let Some(Value::Array(arr)) = self.evaluated_schema.pointer_mut(&normalized_path) {
            mem::take(arr)
        } else {
            return;
        };
        
        // Process elements (now we can borrow self immutably)
        let mut updated_elements = Vec::with_capacity(elements.len());
        for element in elements {
            updated_elements.push(self.apply_parent_conditions(element, false, false));
        }
        
        // Write back the updated elements
        if let Some(target) = self.evaluated_schema.pointer_mut(&normalized_path) {
            *target = Value::Array(updated_elements);
        }
    }
    
    /// Recursively apply parent hidden/disabled conditions to an element and its children
    fn apply_parent_conditions(&self, element: Value, parent_hidden: bool, parent_disabled: bool) -> Value {
        if let Value::Object(mut map) = element {
            // Get current element's condition
            let mut element_hidden = parent_hidden;
            let mut element_disabled = parent_disabled;
            
            // Check condition field (used by field elements with $ref)
            if let Some(Value::Object(condition)) = map.get("condition") {
                if let Some(Value::Bool(hidden)) = condition.get("hidden") {
                    element_hidden = element_hidden || *hidden;
                }
                if let Some(Value::Bool(disabled)) = condition.get("disabled") {
                    element_disabled = element_disabled || *disabled;
                }
            }
            
            // Check hideLayout field (used by direct layout elements without $ref)
            if let Some(Value::Object(hide_layout)) = map.get("hideLayout") {
                // Check hideLayout.all
                if let Some(Value::Bool(all_hidden)) = hide_layout.get("all") {
                    if *all_hidden {
                        element_hidden = true;
                    }
                }
            }
            
            // Update condition to include parent state (for field elements)
            if parent_hidden || parent_disabled {
                // Update condition field if it exists or if this is a field element
                if map.contains_key("condition") || map.contains_key("$ref") || map.contains_key("$fullpath") {
                    let mut condition = if let Some(Value::Object(c)) = map.get("condition") {
                        c.clone()
                    } else {
                        serde_json::Map::new()
                    };
                    
                    if parent_hidden {
                        condition.insert("hidden".to_string(), Value::Bool(true));
                    }
                    if parent_disabled {
                        condition.insert("disabled".to_string(), Value::Bool(true));
                    }
                    
                    map.insert("condition".to_string(), Value::Object(condition));
                }
                
                // Update hideLayout for direct layout elements
                if parent_hidden && (map.contains_key("hideLayout") || map.contains_key("type")) {
                    let mut hide_layout = if let Some(Value::Object(h)) = map.get("hideLayout") {
                        h.clone()
                    } else {
                        serde_json::Map::new()
                    };
                    
                    // Set hideLayout.all to true when parent is hidden
                    hide_layout.insert("all".to_string(), Value::Bool(true));
                    map.insert("hideLayout".to_string(), Value::Object(hide_layout));
                }
            }
            
            // Update $parentHide flag if element has it (came from $ref resolution)
            // Only update if the element already has the field (to avoid adding it to non-ref elements)
            if map.contains_key("$parentHide") {
                map.insert("$parentHide".to_string(), Value::Bool(parent_hidden));
            }
            
            // Recursively process children if elements array exists
            if let Some(Value::Array(elements)) = map.get("elements") {
                let mut updated_children = Vec::with_capacity(elements.len());
                for child in elements {
                    updated_children.push(self.apply_parent_conditions(
                        child.clone(),
                        element_hidden,
                        element_disabled,
                    ));
                }
                map.insert("elements".to_string(), Value::Array(updated_children));
            }
            
            return Value::Object(map);
        }
        
        element
    }
    
    /// Resolve $ref references in layout elements (recursively)
    fn resolve_layout_elements(&mut self, layout_elements_path: &str) {
        // Normalize path from schema format (#/) to JSON pointer format (/)
        let normalized_path = path_utils::normalize_to_json_pointer(layout_elements_path);
        
        // Always read elements from original schema (not evaluated_schema)
        // This ensures we get fresh $ref entries on re-evaluation
        // since evaluated_schema elements get mutated to objects after first resolution
        let elements = if let Some(Value::Array(arr)) = self.schema.pointer(&normalized_path) {
            arr.clone()
        } else {
            return;
        };
        
        // Extract the parent path from normalized_path (e.g., "/properties/form/$layout/elements" -> "form.$layout")
        let parent_path = normalized_path
            .trim_start_matches('/')
            .replace("/elements", "")
            .replace('/', ".");
        
        // Process elements (now we can borrow self immutably)
        let mut resolved_elements = Vec::with_capacity(elements.len());
        for (index, element) in elements.iter().enumerate() {
            let element_path = if parent_path.is_empty() {
                format!("elements.{}", index)
            } else {
                format!("{}.elements.{}", parent_path, index)
            };
            let resolved = self.resolve_element_ref_recursive(element.clone(), &element_path);
            resolved_elements.push(resolved);
        }
        
        // Write back the resolved elements
        if let Some(target) = self.evaluated_schema.pointer_mut(&normalized_path) {
            *target = Value::Array(resolved_elements);
        }
    }
    
    /// Recursively resolve $ref in an element and its nested elements
    /// path_context: The dotted path to the current element (e.g., "form.$layout.elements.0")
    fn resolve_element_ref_recursive(&self, element: Value, path_context: &str) -> Value {
        // First resolve the current element's $ref
        let resolved = self.resolve_element_ref(element);
        
        // Then recursively resolve any nested elements arrays
        if let Value::Object(mut map) = resolved {
            // Ensure all layout elements have metadata fields
            // For elements with $ref, these were already set by resolve_element_ref
            // For direct layout elements without $ref, set them based on path_context
            if !map.contains_key("$parentHide") {
                map.insert("$parentHide".to_string(), Value::Bool(false));
            }
            
            // Set path metadata for direct layout elements (without $ref)
            if !map.contains_key("$fullpath") {
                map.insert("$fullpath".to_string(), Value::String(path_context.to_string()));
            }
            
            if !map.contains_key("$path") {
                // Extract last segment from path_context
                let last_segment = path_context.split('.').last().unwrap_or(path_context);
                map.insert("$path".to_string(), Value::String(last_segment.to_string()));
            }
            
            // Check if this object has an "elements" array
            if let Some(Value::Array(elements)) = map.get("elements") {
                let mut resolved_nested = Vec::with_capacity(elements.len());
                for (index, nested_element) in elements.iter().enumerate() {
                    let nested_path = format!("{}.elements.{}", path_context, index);
                    resolved_nested.push(self.resolve_element_ref_recursive(nested_element.clone(), &nested_path));
                }
                map.insert("elements".to_string(), Value::Array(resolved_nested));
            }
            
            return Value::Object(map);
        }
        
        resolved
    }
    
    /// Resolve $ref in a single element
    fn resolve_element_ref(&self, element: Value) -> Value {
        match element {
            Value::Object(mut map) => {
                // Check if element has $ref
                if let Some(Value::String(ref_path)) = map.get("$ref").cloned() {
                    // Convert ref_path to dotted notation for metadata storage
                    let dotted_path = path_utils::pointer_to_dot_notation(&ref_path);
                    
                    // Extract last segment for $path and path fields
                    let last_segment = dotted_path.split('.').last().unwrap_or(&dotted_path);
                    
                    // Inject metadata fields with dotted notation
                    map.insert("$fullpath".to_string(), Value::String(dotted_path.clone()));
                    map.insert("$path".to_string(), Value::String(last_segment.to_string()));
                    map.insert("$parentHide".to_string(), Value::Bool(false));
                    
                    // Normalize to JSON pointer for actual lookup
                    // Try different path formats to find the referenced value
                    let normalized_path = if ref_path.starts_with('#') || ref_path.starts_with('/') {
                        // Already a pointer, normalize it
                        path_utils::normalize_to_json_pointer(&ref_path)
                    } else {
                        // Try as schema path first (for paths like "illustration.insured.name")
                        let schema_pointer = path_utils::dot_notation_to_schema_pointer(&ref_path);
                        let schema_path = path_utils::normalize_to_json_pointer(&schema_pointer);
                        
                        // Check if it exists
                        if self.evaluated_schema.pointer(&schema_path).is_some() {
                            schema_path
                        } else {
                            // Try with /properties/ prefix (for simple refs like "parent_container")
                            let with_properties = format!("/properties/{}", ref_path.replace('.', "/properties/"));
                            with_properties
                        }
                    };
                    
                    // Get the referenced value
                    if let Some(referenced_value) = self.evaluated_schema.pointer(&normalized_path) {
                        // Clone the referenced value
                        let resolved = referenced_value.clone();
                        
                        // If resolved is an object, check for special handling
                        if let Value::Object(mut resolved_map) = resolved {
                            // Remove $ref from original map
                            map.remove("$ref");
                            
                            // Special case: if resolved has $layout, flatten it
                            // Extract $layout contents and merge at root level
                            if let Some(Value::Object(layout_obj)) = resolved_map.remove("$layout") {
                                // Start with layout properties (they become root properties)
                                let mut result = layout_obj.clone();
                                
                                // Remove properties from resolved (we don't want it)
                                resolved_map.remove("properties");
                                
                                // Merge remaining resolved_map properties (except type if layout has it)
                                for (key, value) in resolved_map {
                                    if key != "type" || !result.contains_key("type") {
                                        result.insert(key, value);
                                    }
                                }
                                
                                // Finally, merge element override properties
                                for (key, value) in map {
                                    result.insert(key, value);
                                }
                                
                                return Value::Object(result);
                            } else {
                                // Normal merge: element properties override referenced properties
                                for (key, value) in map {
                                    resolved_map.insert(key, value);
                                }
                                
                                return Value::Object(resolved_map);
                            }
                        } else {
                            // If referenced value is not an object, just return it
                            return resolved;
                        }
                    }
                }
                
                // No $ref or couldn't resolve - return element as-is
                Value::Object(map)
            }
            _ => element,
        }
    }

    /// Evaluate fields that depend on a changed path
    /// This processes all dependent fields transitively when a source field changes
    /// 
    /// # Arguments
    /// * `changed_paths` - Array of field paths that changed (supports dot notation or schema pointers)
    /// * `data` - Optional JSON data to update before processing
    /// * `context` - Optional context data
    /// * `re_evaluate` - If true, performs full evaluation after processing dependents
    pub fn evaluate_dependents(
        &mut self,
        changed_paths: &[String],
        data: Option<&str>,
        context: Option<&str>,
        re_evaluate: bool,
    ) -> Result<Value, String> {
        // Acquire lock for synchronous execution
        let _lock = self.eval_lock.lock().unwrap();
        
        // Update data if provided
        if let Some(data_str) = data {
            // Save old data for comparison
            let old_data = self.eval_data.clone_data_without(&["$params"]);
            
            let data_value = json_parser::parse_json_str(data_str)?;
            let context_value = if let Some(ctx) = context {
                json_parser::parse_json_str(ctx)?
            } else {
                Value::Object(serde_json::Map::new())
            };
            self.eval_data.replace_data_and_context(data_value.clone(), context_value);
            
            // Selectively purge cache entries that depend on changed data
            // Only purge if values actually changed
            // Convert changed_paths to data pointer format for cache purging
            let data_paths: Vec<String> = changed_paths
                .iter()
                .map(|path| {
                    // Convert "illustration.insured.ins_dob" to "/illustration/insured/ins_dob"
                    format!("/{}", path.replace('.', "/"))
                })
                .collect();
            self.purge_cache_for_changed_data_with_comparison(&data_paths, &old_data, &data_value);
        }
        
        let mut result = Vec::new();
        let mut processed = IndexSet::new();
        
        // Normalize all changed paths and add to processing queue
        // Converts: "illustration.insured.name" -> "#/illustration/properties/insured/properties/name"
        let mut to_process: Vec<(String, bool)> = changed_paths
            .iter()
            .map(|path| (path_utils::dot_notation_to_schema_pointer(path), false))
            .collect(); // (path, is_transitive)
        
        // Process dependents recursively (always nested/transitive)
        while let Some((current_path, is_transitive)) = to_process.pop() {
            if processed.contains(&current_path) {
                continue;
            }
            processed.insert(current_path.clone());
            
            // Get the value of the changed field for $value context
            let current_data_path = path_utils::normalize_to_json_pointer(&current_path)
                .replace("/properties/", "/")
                .trim_start_matches('#')
                .to_string();
            let mut current_value = self.eval_data.data().pointer(&current_data_path)
                .cloned()
                .unwrap_or(Value::Null);
            
            // Find dependents for this path
            if let Some(dependent_items) = self.dependents_evaluations.get(&current_path) {
                for dep_item in dependent_items {
                    let ref_path = &dep_item.ref_path;
                    let pointer_path = path_utils::normalize_to_json_pointer(ref_path);
                    // Data paths don't include /properties/, strip it for data access
                    let data_path = pointer_path.replace("/properties/", "/");

                    let current_ref_value = self.eval_data.data().pointer(&data_path)
                        .cloned()
                        .unwrap_or(Value::Null);
                    
                    // Get field and parent field from schema
                    let field = self.evaluated_schema.pointer(&pointer_path).cloned();
                    
                    // Get parent field - skip /properties/ to get actual parent object
                    let parent_path = if let Some(last_slash) = pointer_path.rfind("/properties") {
                        &pointer_path[..last_slash]
                    } else {
                        "/"
                    };
                    let mut parent_field = if parent_path.is_empty() || parent_path == "/" {
                        self.evaluated_schema.clone()
                    } else {
                        self.evaluated_schema.pointer(parent_path).cloned()
                            .unwrap_or_else(|| Value::Object(serde_json::Map::new()))
                    };

                    // omit properties to minimize size of parent field
                    if let Value::Object(ref mut map) = parent_field {
                        map.remove("properties");
                        map.remove("$layout");
                    }
                    
                    let mut change_obj = serde_json::Map::new();
                    change_obj.insert("$ref".to_string(), Value::String(path_utils::pointer_to_dot_notation(&data_path)));
                    if let Some(f) = field {
                        change_obj.insert("$field".to_string(), f);
                    }
                    change_obj.insert("$parentField".to_string(), parent_field);
                    change_obj.insert("transitive".to_string(), Value::Bool(is_transitive));
                    
                    let mut add_transitive = false;
                    let mut add_deps = false;
                    // Process clear
                    if let Some(clear_val) = &dep_item.clear {
                        let clear_val_clone = clear_val.clone();
                        let should_clear = Self::evaluate_dependent_value_static(&self.engine, &self.evaluations, &self.eval_data, &clear_val_clone, &current_value, &current_ref_value)?;
                        let clear_bool = match should_clear {
                            Value::Bool(b) => b,
                            _ => false,
                        };
                        
                        if clear_bool {
                            // Clear the field
                            if data_path == current_data_path {
                                current_value = Value::Null;
                            }
                            self.eval_data.set(&data_path, Value::Null);
                            change_obj.insert("clear".to_string(), Value::Bool(true));
                            add_transitive = true;
                            add_deps = true;
                        }
                    }
                    
                    // Process value
                    if let Some(value_val) = &dep_item.value {
                        let value_val_clone = value_val.clone();
                        let computed_value = Self::evaluate_dependent_value_static(&self.engine, &self.evaluations, &self.eval_data, &value_val_clone, &current_value, &current_ref_value)?;
                        let cleaned_val = clean_float_noise(computed_value.clone());
                        
                        if cleaned_val != current_ref_value && cleaned_val != Value::Null {   
                            // Set the value
                            if data_path == current_data_path {
                                current_value = cleaned_val.clone();
                            }
                            self.eval_data.set(&data_path, cleaned_val.clone());
                            change_obj.insert("value".to_string(), cleaned_val);
                            add_transitive = true;
                            add_deps = true;
                        }
                    }
                    
                    // add only when has clear / value
                    if add_deps {
                        result.push(Value::Object(change_obj));
                    }
                    
                    // Add this dependent to queue for transitive processing
                    if add_transitive {
                        to_process.push((ref_path.clone(), true));
                    }
                }
            }
        }
        
        // If re_evaluate is true, perform full evaluation with the mutated eval_data
        // Use evaluate_internal to avoid serialization overhead
        // We need to drop the lock first since evaluate_internal acquires its own lock
        if re_evaluate {
            drop(_lock);  // Release the evaluate_dependents lock
            self.evaluate_internal()?;
        }
        
        Ok(Value::Array(result))
    }
    
    /// Helper to evaluate a dependent value - uses pre-compiled eval keys for fast lookup
    fn evaluate_dependent_value_static(
        engine: &RLogic,
        evaluations: &IndexMap<String, LogicId>,
        eval_data: &EvalData,
        value: &Value,
        changed_field_value: &Value,
        changed_field_ref_value: &Value
    ) -> Result<Value, String> {
        match value {
            // If it's a String, check if it's an eval key reference
            Value::String(eval_key) => {
                if let Some(logic_id) = evaluations.get(eval_key) {
                    // It's a pre-compiled evaluation - run it with scoped context
                    // Create internal context with $value and $refValue
                    let mut internal_context = serde_json::Map::new();
                    internal_context.insert("$value".to_string(), changed_field_value.clone());
                    internal_context.insert("$refValue".to_string(), changed_field_ref_value.clone());
                    let context_value = Value::Object(internal_context);
                    
                    let result = engine.run_with_context(logic_id, eval_data.data(), &context_value)
                        .map_err(|e| format!("Failed to evaluate dependent logic '{}': {}", eval_key, e))?;
                    Ok(result)
                } else {
                    // It's a regular string value
                    Ok(value.clone())
                }
            }
            // For backwards compatibility: compile $evaluation on-the-fly
            // This shouldn't happen with properly parsed schemas
            Value::Object(map) if map.contains_key("$evaluation") => {
                Err("Dependent evaluation contains unparsed $evaluation - schema was not properly parsed".to_string())
            }
            // Primitive value - return as-is
            _ => Ok(value.clone()),
        }
    }

    /// Validate form data against schema rules
    /// Returns validation errors for fields that don't meet their rules
    pub fn validate(
        &mut self,
        data: &str,
        context: Option<&str>,
        paths: Option<&[String]>
    ) -> Result<ValidationResult, String> {
        // Acquire lock for synchronous execution
        let _lock = self.eval_lock.lock().unwrap();
        
        // Save old data for comparison
        let old_data = self.eval_data.clone_data_without(&["$params"]);
        
        // Parse data and context
        let data_value = json_parser::parse_json_str(data)?;
        let context_value = if let Some(ctx) = context {
            json_parser::parse_json_str(ctx)?
        } else {
            Value::Object(serde_json::Map::new())
        };
        
        // Update eval_data with new data/context
        self.eval_data.replace_data_and_context(data_value.clone(), context_value);
        
        // Selectively purge cache for rule evaluations that depend on changed data
        // Collect all top-level data keys as potentially changed paths
        let changed_data_paths: Vec<String> = if let Some(obj) = data_value.as_object() {
            obj.keys().map(|k| format!("/{}", k)).collect()
        } else {
            Vec::new()
        };
        self.purge_cache_for_changed_data_with_comparison(&changed_data_paths, &old_data, &data_value);
        
        // Drop lock before calling evaluate_others which needs mutable access
        drop(_lock);
        
        // Re-evaluate rule evaluations to ensure fresh values
        // This ensures all rule.$evaluation expressions are re-computed
        self.evaluate_others();
        
        // Update evaluated_schema with fresh evaluations
        self.evaluated_schema = self.get_evaluated_schema(false);
        
        let mut errors: IndexMap<String, ValidationError> = IndexMap::new();
        
        // Use pre-parsed fields_with_rules from schema parsing (no runtime collection needed)
        // This list was collected during schema parse and contains all fields with rules
        for field_path in &self.fields_with_rules {
            // Check if we should validate this path (path filtering)
            if let Some(filter_paths) = paths {
                if !filter_paths.is_empty() && !filter_paths.iter().any(|p| field_path.starts_with(p.as_str()) || p.starts_with(field_path.as_str())) {
                    continue;
                }
            }
            
            self.validate_field(field_path, &data_value, &mut errors);
        }
        
        let has_error = !errors.is_empty();
        
        Ok(ValidationResult {
            has_error,
            errors,
        })
    }
    
    /// Validate a single field that has rules
    fn validate_field(
        &self,
        field_path: &str,
        data: &Value,
        errors: &mut IndexMap<String, ValidationError>
    ) {
        // Skip if already has error
        if errors.contains_key(field_path) {
            return;
        }
        
        // Get schema for this field
        let schema_path = path_utils::dot_notation_to_schema_pointer(field_path);
        
        // Remove leading "#" from path for pointer lookup
        let pointer_path = schema_path.trim_start_matches('#');
        
        // Try to get schema, if not found, try with /properties/ prefix for standard JSON Schema
        let field_schema = match self.evaluated_schema.pointer(pointer_path) {
            Some(s) => s,
            None => {
                // Try with /properties/ prefix (for standard JSON Schema format)
                let alt_path = format!("/properties{}", pointer_path);
                match self.evaluated_schema.pointer(&alt_path) {
                    Some(s) => s,
                    None => return,
                }
            }
        };
        
        // Check if field is hidden (skip validation)
        if let Value::Object(schema_map) = field_schema {
            if let Some(Value::Object(condition)) = schema_map.get("condition") {
                if let Some(Value::Bool(true)) = condition.get("hidden") {
                    return;
                }
            }
            
            // Get rules object
            let rules = match schema_map.get("rules") {
                Some(Value::Object(r)) => r,
                _ => return,
            };
            
            // Get field data
            let field_data = self.get_field_data(field_path, data);
            
            // Validate each rule
            for (rule_name, rule_value) in rules {
                self.validate_rule(
                    field_path,
                    rule_name,
                    rule_value,
                    &field_data,
                    schema_map,
                    field_schema,
                    errors
                );
            }
        }
    }
    
    /// Get data value for a field path
    fn get_field_data(&self, field_path: &str, data: &Value) -> Value {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = data;
        
        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part).unwrap_or(&Value::Null);
                }
                _ => return Value::Null,
            }
        }
        
        current.clone()
    }
    
    /// Validate a single rule
    fn validate_rule(
        &self,
        field_path: &str,
        rule_name: &str,
        rule_value: &Value,
        field_data: &Value,
        schema_map: &serde_json::Map<String, Value>,
        _schema: &Value,
        errors: &mut IndexMap<String, ValidationError>
    ) {
        // Skip if already has error
        if errors.contains_key(field_path) {
            return;
        }
        
        // Check if disabled
        if let Some(Value::Object(condition)) = schema_map.get("condition") {
            if let Some(Value::Bool(true)) = condition.get("disabled") {
                return;
            }
        }
        
        // Get the evaluated rule from evaluated_schema (which has $evaluation already processed)
        // Convert field_path to schema path
        let schema_path = path_utils::dot_notation_to_schema_pointer(field_path);
        let rule_path = format!("{}/rules/{}", schema_path.trim_start_matches('#'), rule_name);
        
        // Look up the evaluated rule from evaluated_schema
        let evaluated_rule = if let Some(eval_rule) = self.evaluated_schema.pointer(&rule_path) {
            eval_rule.clone()
        } else {
            rule_value.clone()
        };
        
        // Extract rule object (after evaluation)
        let (rule_active, rule_message, rule_code, rule_data) = match &evaluated_rule {
            Value::Object(rule_obj) => {
                let active = rule_obj.get("value").unwrap_or(&Value::Bool(false));
                
                // Handle message - could be string or object with "value"
                let message = match rule_obj.get("message") {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Object(msg_obj)) if msg_obj.contains_key("value") => {
                        msg_obj.get("value")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Validation failed")
                            .to_string()
                    }
                    Some(msg_val) => msg_val.as_str().unwrap_or("Validation failed").to_string(),
                    None => "Validation failed".to_string()
                };
                
                let code = rule_obj.get("code")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());
                
                // Handle data - extract "value" from objects with $evaluation
                let data = rule_obj.get("data").map(|d| {
                    if let Value::Object(data_obj) = d {
                        let mut cleaned_data = serde_json::Map::new();
                        for (key, value) in data_obj {
                            // If value is an object with only "value" key, extract it
                            if let Value::Object(val_obj) = value {
                                if val_obj.len() == 1 && val_obj.contains_key("value") {
                                    cleaned_data.insert(key.clone(), val_obj["value"].clone());
                                } else {
                                    cleaned_data.insert(key.clone(), value.clone());
                                }
                            } else {
                                cleaned_data.insert(key.clone(), value.clone());
                            }
                        }
                        Value::Object(cleaned_data)
                    } else {
                        d.clone()
                    }
                });
                
                (active.clone(), message, code, data)
            }
            _ => (evaluated_rule.clone(), "Validation failed".to_string(), None, None)
        };
        
        // Generate default code if not provided
        let error_code = rule_code.or_else(|| Some(format!("{}.{}", field_path, rule_name)));
        
        let is_empty = matches!(field_data, Value::Null) || 
                       (field_data.is_string() && field_data.as_str().unwrap_or("").is_empty()) ||
                       (field_data.is_array() && field_data.as_array().unwrap().is_empty());
        
        match rule_name {
            "required" => {
                if let Value::Bool(true) = rule_active {
                    if is_empty {
                        errors.insert(field_path.to_string(), ValidationError {
                            rule_type: "required".to_string(),
                            message: rule_message,
                            code: error_code.clone(),
                            pattern: None,
                            field_value: None,
                            data: None,
                        });
                    }
                }
            }
            "minLength" => {
                if !is_empty {
                    if let Some(min) = rule_active.as_u64() {
                        let len = match field_data {
                            Value::String(s) => s.len(),
                            Value::Array(a) => a.len(),
                            _ => 0
                        };
                        if len < min as usize {
                            errors.insert(field_path.to_string(), ValidationError {
                                rule_type: "minLength".to_string(),
                                message: rule_message,
                                code: error_code.clone(),
                                pattern: None,
                                field_value: None,
                                data: None,
                            });
                        }
                    }
                }
            }
            "maxLength" => {
                if !is_empty {
                    if let Some(max) = rule_active.as_u64() {
                        let len = match field_data {
                            Value::String(s) => s.len(),
                            Value::Array(a) => a.len(),
                            _ => 0
                        };
                        if len > max as usize {
                            errors.insert(field_path.to_string(), ValidationError {
                                rule_type: "maxLength".to_string(),
                                message: rule_message,
                                code: error_code.clone(),
                                pattern: None,
                                field_value: None,
                                data: None,
                            });
                        }
                    }
                }
            }
            "minValue" => {
                if !is_empty {
                    if let Some(min) = rule_active.as_f64() {
                        if let Some(val) = field_data.as_f64() {
                            if val < min {
                                errors.insert(field_path.to_string(), ValidationError {
                                    rule_type: "minValue".to_string(),
                                    message: rule_message,
                                    code: error_code.clone(),
                                    pattern: None,
                                    field_value: None,
                                    data: None,
                                });
                            }
                        }
                    }
                }
            }
            "maxValue" => {
                if !is_empty {
                    if let Some(max) = rule_active.as_f64() {
                        if let Some(val) = field_data.as_f64() {
                            if val > max {
                                errors.insert(field_path.to_string(), ValidationError {
                                    rule_type: "maxValue".to_string(),
                                    message: rule_message,
                                    code: error_code.clone(),
                                    pattern: None,
                                    field_value: None,
                                    data: None,
                                });
                            }
                        }
                    }
                }
            }
            "pattern" => {
                if !is_empty {
                    if let Some(pattern) = rule_active.as_str() {
                        if let Some(text) = field_data.as_str() {
                            if let Ok(regex) = regex::Regex::new(pattern) {
                                if !regex.is_match(text) {
                                    errors.insert(field_path.to_string(), ValidationError {
                                        rule_type: "pattern".to_string(),
                                        message: rule_message,
                                        code: error_code.clone(),
                                        pattern: Some(pattern.to_string()),
                                        field_value: Some(text.to_string()),
                                        data: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            "evaluation" => {
                // Handle array of evaluation rules
                // Format: "evaluation": [{ "code": "...", "message": "...", "$evaluation": {...} }]
                if let Value::Array(eval_array) = &evaluated_rule {
                    for (idx, eval_item) in eval_array.iter().enumerate() {
                        if let Value::Object(eval_obj) = eval_item {
                            // Get the evaluated value (should be in "value" key after evaluation)
                            let eval_result = eval_obj.get("value").unwrap_or(&Value::Bool(true));
                            
                            // Check if result is falsy
                            let is_falsy = match eval_result {
                                Value::Bool(false) => true,
                                Value::Null => true,
                                Value::Number(n) => n.as_f64() == Some(0.0),
                                Value::String(s) => s.is_empty(),
                                Value::Array(a) => a.is_empty(),
                                _ => false,
                            };
                            
                            if is_falsy {
                                let eval_code = eval_obj.get("code")
                                    .and_then(|c| c.as_str())
                                    .map(|s| s.to_string())
                                    .or_else(|| Some(format!("{}.evaluation.{}", field_path, idx)));
                                
                                let eval_message = eval_obj.get("message")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Validation failed")
                                    .to_string();
                                
                                let eval_data = eval_obj.get("data").cloned();
                                
                                errors.insert(field_path.to_string(), ValidationError {
                                    rule_type: "evaluation".to_string(),
                                    message: eval_message,
                                    code: eval_code,
                                    pattern: None,
                                    field_value: None,
                                    data: eval_data,
                                });
                                
                                // Stop at first failure
                                break;
                            }
                        }
                    }
                }
            }
            _ => {
                // Custom evaluation rules
                // In JS: if (!opt.rule.value) then error
                // This handles rules with $evaluation that return false/falsy values
                if !is_empty {
                    // Check if rule_active is falsy (false, 0, null, empty string, empty array)
                    let is_falsy = match &rule_active {
                        Value::Bool(false) => true,
                        Value::Null => true,
                        Value::Number(n) => n.as_f64() == Some(0.0),
                        Value::String(s) => s.is_empty(),
                        Value::Array(a) => a.is_empty(),
                        _ => false,
                    };
                    
                    if is_falsy {
                        errors.insert(field_path.to_string(), ValidationError {
                            rule_type: "evaluation".to_string(),
                            message: rule_message,
                            code: error_code.clone(),
                            pattern: None,
                            field_value: None,
                            data: rule_data,
                        });
                    }
                }
            }
        }
    }
}

/// Validation error for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub has_error: bool,
    pub errors: IndexMap<String, ValidationError>,
}

