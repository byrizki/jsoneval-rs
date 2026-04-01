use super::JSONEval;
use crate::jsoneval::eval_data::EvalData;
use crate::jsoneval::json_parser;
use crate::jsoneval::parsed_schema::ParsedSchema;
use crate::jsoneval::parsed_schema_cache::PARSED_SCHEMA_CACHE;
use crate::parse_schema;
use crate::rlogic::{RLogic, RLogicConfig};

use crate::time_block;

use indexmap::IndexMap;
use serde::de::Error as _;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

impl Clone for JSONEval {
    fn clone(&self) -> Self {
        Self {
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
            reffed_by: self.reffed_by.clone(),
            dep_formula_triggers: self.dep_formula_triggers.clone(),
            context: self.context.clone(),
            data: self.data.clone(),
            evaluated_schema: self.evaluated_schema.clone(),
            eval_data: self.eval_data.clone(),
            eval_cache: self.eval_cache.clone(),
            eval_lock: Mutex::new(()), // Create fresh mutex for the clone
            cached_msgpack_schema: self.cached_msgpack_schema.clone(),
            conditional_hidden_fields: self.conditional_hidden_fields.clone(),
            conditional_readonly_fields: self.conditional_readonly_fields.clone(),
            static_arrays: self.static_arrays.clone(),
            regex_cache: RwLock::new(HashMap::new()),
        }
    }
}

impl JSONEval {
    pub fn new(
        schema: &str,
        context: Option<&str>,
        data: Option<&str>,
    ) -> Result<Self, serde_json::Error> {
        time_block!("JSONEval::new() [total]", {
            // Use serde_json for schema (needs arbitrary_precision) and SIMD for data (needs speed)
            let mut schema_val: Value =
                time_block!("  parse schema JSON", { serde_json::from_str(schema)? });
            let context: Value = time_block!("  parse context JSON", {
                json_parser::parse_json_str(context.unwrap_or("{}"))
                    .map_err(serde_json::Error::custom)?
            });
            let data: Value = time_block!("  parse data JSON", {
                json_parser::parse_json_str(data.unwrap_or("{}"))
                    .map_err(serde_json::Error::custom)?
            });

            // Extract large static arrays
            let static_arrays = if let Some(params) = schema_val
                .get_mut("$params")
                .and_then(|v| v.as_object_mut())
            {
                crate::jsoneval::static_arrays::extract_from_params(params)
            } else {
                IndexMap::new()
            };
            let static_arrays = Arc::new(static_arrays);
            let evaluated_schema = schema_val.clone();

            // Use default config: tracking enabled
            let engine_config = RLogicConfig::default();
            let mut engine = RLogic::with_config(engine_config);
            engine.set_static_arrays(Arc::clone(&static_arrays));

            let mut instance = time_block!("  create instance struct", {
                Self {
                    schema: Arc::new(schema_val),
                    evaluations: Arc::new(IndexMap::new()),
                    tables: Arc::new(IndexMap::new()),
                    table_metadata: Arc::new(IndexMap::new()),
                    dependencies: Arc::new(IndexMap::new()),
                    sorted_evaluations: Arc::new(Vec::new()),
                    dependents_evaluations: Arc::new(IndexMap::new()),
                    rules_evaluations: Arc::new(Vec::new()),
                    fields_with_rules: Arc::new(Vec::new()),
                    others_evaluations: Arc::new(Vec::new()),
                    value_evaluations: Arc::new(Vec::new()),
                    layout_paths: Arc::new(Vec::new()),
                    options_templates: Arc::new(Vec::new()),
                    subforms: IndexMap::new(),
                    engine: Arc::new(engine),
                    reffed_by: Arc::new(IndexMap::new()),
                    dep_formula_triggers: Arc::new(IndexMap::new()),
                    context: context.clone(),
                    data: data.clone(),
                    evaluated_schema: evaluated_schema.clone(),
                    eval_data: EvalData::with_schema_data_context(
                        &evaluated_schema,
                        &data,
                        &context,
                    ),
                    eval_cache: crate::jsoneval::eval_cache::EvalCache::new(),
                    eval_lock: Mutex::new(()),
                    cached_msgpack_schema: None,
                    conditional_hidden_fields: Arc::new(Vec::new()),
                    conditional_readonly_fields: Arc::new(Vec::new()),
                    static_arrays,
                    regex_cache: RwLock::new(HashMap::new()),
                }
            });
            time_block!("  parse_schema", {
                parse_schema::legacy::parse_schema(&mut instance)
                    .map_err(serde_json::Error::custom)?
            });
            Ok(instance)
        })
    }

    /// Create a new JSONEval instance for a subform, avoiding string serialization
    pub(crate) fn new_subform(
        schema_val: Value,
        context: Value,
        static_arrays: Arc<IndexMap<String, Arc<Value>>>,
    ) -> Result<Self, serde_json::Error> {
        time_block!("JSONEval::new_subform() [total]", {
            // Data is empty for a subform initially
            let data = Value::Object(serde_json::Map::new());
            let evaluated_schema = schema_val.clone();

            // Use default config: tracking enabled
            let engine_config = RLogicConfig::default();
            let mut engine = RLogic::with_config(engine_config);
            engine.set_static_arrays(Arc::clone(&static_arrays));

            let mut instance = time_block!("  create instance struct", {
                Self {
                    schema: Arc::new(schema_val),
                    evaluations: Arc::new(IndexMap::new()),
                    tables: Arc::new(IndexMap::new()),
                    table_metadata: Arc::new(IndexMap::new()),
                    dependencies: Arc::new(IndexMap::new()),
                    sorted_evaluations: Arc::new(Vec::new()),
                    dependents_evaluations: Arc::new(IndexMap::new()),
                    rules_evaluations: Arc::new(Vec::new()),
                    fields_with_rules: Arc::new(Vec::new()),
                    others_evaluations: Arc::new(Vec::new()),
                    value_evaluations: Arc::new(Vec::new()),
                    layout_paths: Arc::new(Vec::new()),
                    options_templates: Arc::new(Vec::new()),
                    subforms: IndexMap::new(),
                    engine: Arc::new(engine),
                    reffed_by: Arc::new(IndexMap::new()),
                    dep_formula_triggers: Arc::new(IndexMap::new()),
                    context: context.clone(),
                    data: data.clone(),
                    evaluated_schema: evaluated_schema.clone(),
                    eval_data: EvalData::with_schema_data_context(
                        &evaluated_schema,
                        &data,
                        &context,
                    ),
                    eval_cache: crate::jsoneval::eval_cache::EvalCache::new(),
                    eval_lock: Mutex::new(()),
                    cached_msgpack_schema: None,
                    conditional_hidden_fields: Arc::new(Vec::new()),
                    conditional_readonly_fields: Arc::new(Vec::new()),
                    static_arrays,
                    regex_cache: RwLock::new(HashMap::new()),
                }
            });
            time_block!("  parse_schema", {
                parse_schema::legacy::parse_schema(&mut instance)
                    .map_err(serde_json::Error::custom)?
            });
            Ok(instance)
        })
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
        let mut schema_val: Value = rmp_serde::from_slice(schema_msgpack)
            .map_err(|e| format!("Failed to deserialize MessagePack schema: {}", e))?;

        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}"))
            .map_err(|e| format!("Failed to parse context: {}", e))?;
        let data: Value = json_parser::parse_json_str(data.unwrap_or("{}"))
            .map_err(|e| format!("Failed to parse data: {}", e))?;

        // Extract large static arrays
        let static_arrays = if let Some(params) = schema_val
            .get_mut("$params")
            .and_then(|v| v.as_object_mut())
        {
            crate::jsoneval::static_arrays::extract_from_params(params)
        } else {
            IndexMap::new()
        };
        let static_arrays = Arc::new(static_arrays);
        let evaluated_schema = schema_val.clone();

        let engine_config = RLogicConfig::default();
        let mut engine = RLogic::with_config(engine_config);
        engine.set_static_arrays(Arc::clone(&static_arrays));

        let mut instance = Self {
            schema: Arc::new(schema_val),
            evaluations: Arc::new(IndexMap::new()),
            tables: Arc::new(IndexMap::new()),
            table_metadata: Arc::new(IndexMap::new()),
            dependencies: Arc::new(IndexMap::new()),
            sorted_evaluations: Arc::new(Vec::new()),
            dependents_evaluations: Arc::new(IndexMap::new()),
            rules_evaluations: Arc::new(Vec::new()),
            fields_with_rules: Arc::new(Vec::new()),
            others_evaluations: Arc::new(Vec::new()),
            value_evaluations: Arc::new(Vec::new()),
            layout_paths: Arc::new(Vec::new()),
            options_templates: Arc::new(Vec::new()),
            subforms: IndexMap::new(),
            engine: Arc::new(engine),
            reffed_by: Arc::new(IndexMap::new()),
            dep_formula_triggers: Arc::new(IndexMap::new()),
            context: context.clone(),
            data: data.clone(),
            evaluated_schema: evaluated_schema.clone(),
            eval_data: EvalData::with_schema_data_context(&evaluated_schema, &data, &context),
            eval_cache: crate::jsoneval::eval_cache::EvalCache::new(),
            eval_lock: Mutex::new(()),
            cached_msgpack_schema: Some(cached_msgpack),
            conditional_hidden_fields: Arc::new(Vec::new()),
            conditional_readonly_fields: Arc::new(Vec::new()),
            static_arrays,
            regex_cache: RwLock::new(HashMap::new()),
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
            let subform_eval =
                JSONEval::with_parsed_schema(subform_parsed.clone(), Some("{}"), None)?;
            subforms.insert(path.clone(), Box::new(subform_eval));
        }

        let instance = Self {
            schema: Arc::clone(&parsed.schema),
            evaluations: Arc::clone(&parsed.evaluations),
            tables: Arc::clone(&parsed.tables),
            table_metadata: Arc::clone(&parsed.table_metadata),
            dependencies: Arc::clone(&parsed.dependencies),
            sorted_evaluations: Arc::clone(&parsed.sorted_evaluations),
            dependents_evaluations: Arc::clone(&parsed.dependents_evaluations),
            rules_evaluations: Arc::clone(&parsed.rules_evaluations),
            fields_with_rules: Arc::clone(&parsed.fields_with_rules),
            others_evaluations: Arc::clone(&parsed.others_evaluations),
            value_evaluations: Arc::clone(&parsed.value_evaluations),
            layout_paths: Arc::clone(&parsed.layout_paths),
            options_templates: Arc::clone(&parsed.options_templates),
            subforms,
            engine,
            reffed_by: Arc::clone(&parsed.reffed_by),
            dep_formula_triggers: Arc::clone(&parsed.dep_formula_triggers),
            context: context.clone(),
            data: data.clone(),
            evaluated_schema: (*evaluated_schema).clone(),
            eval_data: EvalData::with_schema_data_context(&evaluated_schema, &data, &context),
            eval_cache: crate::jsoneval::eval_cache::EvalCache::new(),
            eval_lock: Mutex::new(()),
            cached_msgpack_schema: None,
            conditional_hidden_fields: Arc::clone(&parsed.conditional_hidden_fields),
            conditional_readonly_fields: Arc::clone(&parsed.conditional_readonly_fields),
            static_arrays: Arc::clone(&parsed.static_arrays),
            regex_cache: RwLock::new(HashMap::new()),
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
        let mut schema_val: Value =
            serde_json::from_str(schema).map_err(|e| format!("failed to parse schema: {e}"))?;
        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}"))?;
        let data: Value = json_parser::parse_json_str(data.unwrap_or("{}"))?;
        self.context = context.clone();
        self.data = data.clone();

        let static_arrays = if let Some(params) = schema_val
            .get_mut("$params")
            .and_then(|v| v.as_object_mut())
        {
            crate::jsoneval::static_arrays::extract_from_params(params)
        } else {
            IndexMap::new()
        };
        let static_arrays = Arc::new(static_arrays);
        self.static_arrays = Arc::clone(&static_arrays);
        self.schema = Arc::new(schema_val);
        self.evaluated_schema = (*self.schema).clone();

        let mut engine = RLogic::new();
        engine.set_static_arrays(static_arrays);
        self.engine = Arc::new(engine);

        self.evaluations = Arc::new(IndexMap::new());
        self.tables = Arc::new(IndexMap::new());
        self.table_metadata = Arc::new(IndexMap::new());
        self.dependencies = Arc::new(IndexMap::new());
        self.sorted_evaluations = Arc::new(Vec::new());
        self.dependents_evaluations = Arc::new(IndexMap::new());
        self.rules_evaluations = Arc::new(Vec::new());
        self.fields_with_rules = Arc::new(Vec::new());
        self.others_evaluations = Arc::new(Vec::new());
        self.value_evaluations = Arc::new(Vec::new());
        self.layout_paths = Arc::new(Vec::new());
        self.options_templates = Arc::new(Vec::new());
        self.reffed_by = Arc::new(IndexMap::new());
        self.dep_formula_triggers = Arc::new(IndexMap::new());
        self.conditional_hidden_fields = Arc::new(Vec::new());
        self.conditional_readonly_fields = Arc::new(Vec::new());
        self.subforms.clear();
        parse_schema::legacy::parse_schema(self)?;

        // Re-initialize eval_data with new schema, data, and context
        self.eval_data =
            EvalData::with_schema_data_context(&self.evaluated_schema, &data, &context);
        self.eval_cache.clear();
        if let Ok(mut cache) = self.regex_cache.write() {
            cache.clear();
        }

        // Clear MessagePack cache since schema has been mutated
        self.cached_msgpack_schema = None;

        Ok(())
    }

    /// Set the timezone offset for datetime operations (TODAY, NOW)
    ///
    /// This method updates the RLogic engine configuration with a new timezone offset.
    /// The offset will be applied to all subsequent datetime evaluations.
    ///
    /// # Arguments
    ///
    /// * `offset_minutes` - Timezone offset in minutes from UTC (e.g., 420 for UTC+7, -300 for UTC-5)
    ///   Pass `None` to reset to UTC (no offset)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut eval = JSONEval::new(schema, None, None)?;
    ///
    /// // Set to UTC+7 (Jakarta, Bangkok)
    /// eval.set_timezone_offset(Some(420));
    ///
    /// // Reset to UTC
    /// eval.set_timezone_offset(None);
    /// ```
    pub fn set_timezone_offset(&mut self, offset_minutes: Option<i32>) {
        // Create new config with the timezone offset
        let mut config = RLogicConfig::default();
        if let Some(offset) = offset_minutes {
            config = config.with_timezone_offset(offset);
        }

        // Recreate the engine with the new configuration
        // This is necessary because RLogic is wrapped in Arc and config is part of the evaluator
        let mut engine = RLogic::with_config(config);
        engine.set_static_arrays(Arc::clone(&self.static_arrays));
        self.engine = Arc::new(engine);

        let _ = parse_schema::legacy::parse_schema(self);
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
        let mut schema_val: Value = rmp_serde::from_slice(schema_msgpack)
            .map_err(|e| format!("failed to deserialize MessagePack schema: {e}"))?;

        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}"))?;
        let data: Value = json_parser::parse_json_str(data.unwrap_or("{}"))?;

        self.context = context.clone();
        self.data = data.clone();

        let static_arrays = if let Some(params) = schema_val
            .get_mut("$params")
            .and_then(|v| v.as_object_mut())
        {
            crate::jsoneval::static_arrays::extract_from_params(params)
        } else {
            IndexMap::new()
        };
        let static_arrays = Arc::new(static_arrays);
        self.static_arrays = Arc::clone(&static_arrays);
        self.schema = Arc::new(schema_val);
        self.evaluated_schema = (*self.schema).clone();

        let mut engine = RLogic::new();
        engine.set_static_arrays(static_arrays);
        self.engine = Arc::new(engine);
        self.evaluations = Arc::new(IndexMap::new());
        self.tables = Arc::new(IndexMap::new());
        self.table_metadata = Arc::new(IndexMap::new());
        self.dependencies = Arc::new(IndexMap::new());
        self.sorted_evaluations = Arc::new(Vec::new());
        self.dependents_evaluations = Arc::new(IndexMap::new());
        self.rules_evaluations = Arc::new(Vec::new());
        self.fields_with_rules = Arc::new(Vec::new());
        self.others_evaluations = Arc::new(Vec::new());
        self.value_evaluations = Arc::new(Vec::new());
        self.layout_paths = Arc::new(Vec::new());
        self.options_templates = Arc::new(Vec::new());
        self.reffed_by = Arc::new(IndexMap::new());
        self.dep_formula_triggers = Arc::new(IndexMap::new());
        self.conditional_hidden_fields = Arc::new(Vec::new());
        self.conditional_readonly_fields = Arc::new(Vec::new());
        self.subforms.clear();
        parse_schema::legacy::parse_schema(self)?;

        // Re-initialize eval_data
        self.eval_data =
            EvalData::with_schema_data_context(&self.evaluated_schema, &data, &context);
        self.eval_cache.clear();
        if let Ok(mut cache) = self.regex_cache.write() {
            cache.clear();
        }

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
        self.static_arrays = parsed.static_arrays.clone();
        self.dep_formula_triggers = parsed.dep_formula_triggers.clone();

        // Share the engine Arc (cheap pointer clone, not data clone)
        self.engine = parsed.engine.clone();

        // Convert Arc<ParsedSchema> subforms to Box<JSONEval> subforms
        let mut subforms = IndexMap::new();
        for (path, subform_parsed) in &parsed.subforms {
            let subform_eval =
                JSONEval::with_parsed_schema(subform_parsed.clone(), Some("{}"), None)?;
            subforms.insert(path.clone(), Box::new(subform_eval));
        }
        self.subforms = subforms;

        self.context = context.clone();
        self.data = data.clone();
        self.evaluated_schema = (*self.schema).clone();

        // Re-initialize eval_data
        self.eval_data =
            EvalData::with_schema_data_context(&self.evaluated_schema, &data, &context);
        self.eval_cache.clear();
        self.engine.clear_indices();
        if let Ok(mut cache) = self.regex_cache.write() {
            cache.clear();
        }

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
        let parsed = PARSED_SCHEMA_CACHE
            .get(cache_key)
            .ok_or_else(|| format!("Schema '{}' not found in cache", cache_key))?;

        // Use reload_schema_parsed with the cached schema
        self.reload_schema_parsed(parsed, context, data)
    }
}
