//! JSON Eval RS - High-performance JSON Logic evaluation library
//!
//! This library provides a complete implementation of JSON Logic with advanced features:
//! - Pre-compilation of logic expressions for optimal performance
//! - Automatic result caching with smart invalidation
//! - Mutation tracking via proxy-like data wrapper
//! - Zero external logic dependencies (built from scratch)

pub mod rlogic;
pub mod table_evaluate;
pub mod topo_sort;
pub mod parse_schema;

// Re-export main types for convenience
use indexmap::{IndexMap, IndexSet};
pub use rlogic::{
    CacheKey, CacheStats, CompiledLogic, CompiledLogicStore, DataVersion, EvalCache, Evaluator,
    LogicId, RLogic, RLogicConfig, TrackedData, TrackedDataBuilder,
};
use serde::de::Error as _;
use serde_json::{Value};

pub struct JSONEval {
    pub schema: Value,
    pub evaluated_schema: Value,
    pub context: Value,
    pub data: Value,
    pub engine: RLogic,
    pub evaluations: IndexMap<String, LogicId>,
    pub tables: IndexMap<String, Value>,
    pub dependencies: IndexMap<String, IndexSet<String>>,
    pub sorted_evaluations: IndexSet<String>,
}

impl JSONEval {
    pub fn new(
        schema: &str,
        context: Option<&str>,
        data: Option<&str>,
    ) -> Result<Self, serde_json::Error> {
        let schema_val: Value = serde_json::from_str(schema)?;
        let context: Value = serde_json::from_str(context.unwrap_or("{}"))?;
        let data: Value = serde_json::from_str(data.unwrap_or("{}"))?;
        let evaluated_schema = schema_val.clone();
        // Use performance config: caching enabled, tracking disabled
        // Cache helps with repeated sub-expressions even on first run
        let engine_config = RLogicConfig::performance().with_cache(true);

        let mut instance = Self {
            schema: schema_val,
            evaluated_schema,
            context,
            data,
            evaluations: IndexMap::new(),
            tables: IndexMap::new(),
            dependencies: IndexMap::new(),
            sorted_evaluations: IndexSet::new(),
            engine: RLogic::with_config(engine_config),
        };
        parse_schema::parse_schema(&mut instance).map_err(serde_json::Error::custom)?;
        Ok(instance)
    }

    pub fn reload_schema(
        &mut self,
        schema: &str,
        context: Option<&str>,
        data: Option<&str>,
    ) -> Result<(), String> {
        let schema_val: Value =
            serde_json::from_str(schema).map_err(|e| format!("failed to parse schema: {e}"))?;
        let context: Value = serde_json::from_str(context.unwrap_or("{}"))
            .map_err(|e| format!("failed to parse context: {e}"))?;
        let data: Value = serde_json::from_str(data.unwrap_or("{}"))
            .map_err(|e| format!("failed to parse data: {e}"))?;
        self.schema = schema_val;
        self.context = context;
        self.data = data;
        self.evaluated_schema = self.schema.clone();
        let config = *self.engine.config();
        self.engine = RLogic::with_config(config);
        parse_schema::parse_schema(self)?;

        Ok(())
    }

    pub fn evaluate(&mut self, data: &str, context: Option<&str>) -> Result<Value, String> {
        let data: Value =
            serde_json::from_str(data).map_err(|e| format!("failed to parse data: {e}"))?;
        let context: Value = serde_json::from_str(context.unwrap_or("{}"))
            .map_err(|e| format!("failed to parse context: {e}"))?;
        let mut scope_data = TrackedData::new(data);
        scope_data.set(
            "$params",
            self.evaluated_schema.get("$params").unwrap().clone(),
        );
        scope_data.set("$context", context);

        // Clone sorted_evaluations to avoid borrow checker issues
        // (This is acceptable as it's done once per evaluation)
        let eval_keys: Vec<String> = self.sorted_evaluations.iter().cloned().collect();

        for eval_key in eval_keys {
            // Convert path like "#/$params/constants/DEATH_SA" to "$params.constants.DEATH_SA"
            let dotted_path = eval_key.trim_start_matches("#/").replace('/', ".");
            if let Some(table) = self.tables.get(&eval_key).cloned() {
                // Evaluate table and get rows array
                if let Ok(rows) = table_evaluate::evaluate_table(self, &eval_key, &table, &mut scope_data) {
                    scope_data.set(&dotted_path, Value::Array(rows));
                }
            } else {
                let logic_id = self.evaluations.get(&eval_key).unwrap();
                // Use evaluate_raw for non-cached fast path
                if let Ok(val) = self.engine.evaluate_raw(logic_id, scope_data.data()) {
                    scope_data.set(&dotted_path, val);
                }
            }
        }

        // Cache stats only in debug mode
        #[cfg(debug_assertions)]
        println!("Cache Stats: {:?}", self.engine.cache_stats());
        
        Ok(scope_data.get("$params").unwrap().clone())
    }
}
