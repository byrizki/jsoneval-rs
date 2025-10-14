//! JSON Eval RS - High-performance JSON Logic evaluation library
//!
//! This library provides a complete implementation of JSON Logic with advanced features:
//! - Pre-compilation of logic expressions for optimal performance
//! - Mutation tracking via proxy-like data wrapper (EvalData)
//! - All data mutations gated through EvalData for thread safety
//! - Zero external logic dependencies (built from scratch)

pub mod rlogic;
pub mod table_evaluate;
pub mod table_metadata;
pub mod topo_sort;
pub mod parse_schema;
pub mod json_parser;
pub mod path_utils;
pub mod eval_data;
pub mod eval_cache;

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
};
pub use table_metadata::TableMetadata;
pub use path_utils::ArrayMetadata;
pub use eval_data::EvalData;
pub use eval_cache::{EvalCache, CacheKey, CacheStats};
use serde::de::Error as _;
use serde_json::{Value};
use rayon::prelude::*;
use std::mem;
use std::sync::Mutex;

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

pub struct JSONEval {
    pub schema: Value,
    pub engine: RLogic,
    pub evaluations: IndexMap<String, LogicId>,
    pub tables: IndexMap<String, Value>,
    /// Pre-compiled table metadata (computed at parse time for zero-copy evaluation)
    pub table_metadata: IndexMap<String, TableMetadata>,
    pub dependencies: IndexMap<String, IndexSet<String>>,
    /// Evaluations grouped into parallel-executable batches
    /// Each inner Vec contains evaluations that can run concurrently
    pub sorted_evaluations: Vec<Vec<String>>,
    /// Evaluations categorized for result handling
    /// Dependents: evaluations with "dependents" in path
    pub dependents_evaluations: Vec<String>,
    /// Rules: evaluations with "/rules/" in path
    pub rules_evaluations: Vec<String>,
    /// Others: all other evaluations not in sorted_evaluations (for evaluated_schema output)
    pub others_evaluations: Vec<String>,
    /// Value: evaluations ending with ".value" in path
    pub value_evaluations: Vec<String>,
    /// Cached layout paths (collected at parse time)
    pub layout_paths: Vec<String>,
    pub context: Value,
    pub data: Value,
    pub evaluated_schema: Value,
    pub eval_data: EvalData,
    /// Evaluation cache with content-based hashing and zero-copy storage
    pub eval_cache: EvalCache,
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
            schema: schema_val,
            evaluations: IndexMap::new(),
            tables: IndexMap::new(),
            table_metadata: IndexMap::new(),
            dependencies: IndexMap::new(),
            sorted_evaluations: Vec::new(),
            dependents_evaluations: Vec::new(),
            rules_evaluations: Vec::new(),
            others_evaluations: Vec::new(),
            value_evaluations: Vec::new(),
            layout_paths: Vec::new(),
            engine: RLogic::with_config(engine_config),
            context: context.clone(),
            data: data.clone(),
            evaluated_schema: evaluated_schema.clone(),
            eval_data: EvalData::with_schema_data_context(&evaluated_schema, &data, &context),
            eval_cache: EvalCache::new(),
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
        // Use serde_json for schema (precision) and SIMD for data (speed)
        let schema_val: Value = serde_json::from_str(schema).map_err(|e| format!("failed to parse schema: {e}"))?;
        let context: Value = json_parser::parse_json_str(context.unwrap_or("{}"))?;
        let data: Value = json_parser::parse_json_str(data.unwrap_or("{}"))?;
        self.schema = schema_val;
        self.context = context.clone();
        self.data = data.clone();
        self.evaluated_schema = self.schema.clone();
        self.engine = RLogic::new();
        self.dependents_evaluations.clear();
        self.rules_evaluations.clear();
        self.others_evaluations.clear();
        self.value_evaluations.clear();
        self.layout_paths.clear();
        parse_schema::parse_schema(self)?;
        
        // Re-initialize eval_data with new schema, data, and context
        self.eval_data = EvalData::with_schema_data_context(&self.evaluated_schema, &data, &context);
        
        // Clear cache when schema changes
        self.eval_cache.clear();

        Ok(())
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
        // Replace data and context in existing eval_data
        self.eval_data.replace_data_and_context(data, context);

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
            let results: Mutex<Vec<(String, String, Value)>> = Mutex::new(Vec::with_capacity(batch.len()));
            let eval_data_snapshot = self.eval_data.clone();
            
            // Parallelize only if batch has multiple items (overhead not worth it for single item)
            if batch.len() > 1 {
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
            } else {
                // Single item - no parallelization overhead
                let eval_key = &batch[0];
                let pointer_path = path_utils::normalize_to_json_pointer(eval_key);
                
                // Try cache first
                if let Some(_) = self.try_get_cached(eval_key, &eval_data_snapshot) {
                    continue;
                } else {
                    // Cache miss - evaluate
                    let is_table = self.table_metadata.contains_key(eval_key);
                    
                    if is_table {
                        if let Ok(rows) = table_evaluate::evaluate_table(self, eval_key, &eval_data_snapshot) {
                            let value = Value::Array(rows);
                            // Cache result
                            self.cache_result(eval_key, Value::Null, &eval_data_snapshot);
                            results.lock().unwrap().push((eval_key.clone(), pointer_path, value));
                        }
                    } else {
                        if let Some(logic_id) = self.evaluations.get(eval_key) {
                            if let Ok(val) = self.engine.run(logic_id, eval_data_snapshot.data()) {
                                let cleaned_val = clean_float_noise(val);
                                // Cache result
                                self.cache_result(eval_key, Value::Null, &eval_data_snapshot);
                                results.lock().unwrap().push((eval_key.clone(), pointer_path, cleaned_val));
                            }
                        }
                    }
                }
            }
            
            // Write all results back sequentially (already cached in parallel execution)
            for (_eval_key, path, value) in results.into_inner().unwrap() {
                let cleaned_value = clean_float_noise(value);
                
                self.eval_data.set(&path, cleaned_value.clone());
                // Also write to evaluated_schema
                if let Some(schema_value) = self.evaluated_schema.pointer_mut(&path) {
                    *schema_value = cleaned_value;
                }
            }
        }

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
            self.resolve_layout();
        }
        
        self.evaluated_schema.clone()
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
    /// Returns Some(value) if cache hit, None if cache miss
    fn try_get_cached(&self, eval_key: &str, eval_data: &EvalData) -> Option<Value> {
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

    fn evaluate_others(&mut self) {
        // Evaluate "rules" and "others" categories in parallel with caching
        let combined_count = self.rules_evaluations.len() + self.others_evaluations.len();
        if combined_count > 0 {
            let combined_results: Mutex<Vec<(String, Value)>> = Mutex::new(Vec::with_capacity(combined_count));
            let eval_data_snapshot = self.eval_data.clone();

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
    }

    fn resolve_layout(&mut self) {
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
        // Extract elements array to avoid borrow checker issues
        let elements = if let Some(Value::Array(arr)) = self.evaluated_schema.pointer_mut(layout_elements_path) {
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
        if let Some(target) = self.evaluated_schema.pointer_mut(layout_elements_path) {
            *target = Value::Array(updated_elements);
        }
    }
    
    /// Recursively apply parent hidden/disabled conditions to an element and its children
    fn apply_parent_conditions(&self, element: Value, parent_hidden: bool, parent_disabled: bool) -> Value {
        if let Value::Object(mut map) = element {
            // Get current element's condition
            let mut element_hidden = parent_hidden;
            let mut element_disabled = parent_disabled;
            
            if let Some(Value::Object(condition)) = map.get("condition") {
                if let Some(Value::Bool(hidden)) = condition.get("hidden") {
                    element_hidden = element_hidden || *hidden;
                }
                if let Some(Value::Bool(disabled)) = condition.get("disabled") {
                    element_disabled = element_disabled || *disabled;
                }
            }
            
            // Update condition to include parent state
            if parent_hidden || parent_disabled {
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
        // Extract elements array to avoid borrow checker issues
        let elements = if let Some(Value::Array(arr)) = self.evaluated_schema.pointer_mut(layout_elements_path) {
            mem::take(arr)
        } else {
            return;
        };
        
        // Process elements (now we can borrow self immutably)
        let mut resolved_elements = Vec::with_capacity(elements.len());
        for element in elements {
            let resolved = self.resolve_element_ref_recursive(element);
            resolved_elements.push(resolved);
        }
        
        // Write back the resolved elements
        if let Some(target) = self.evaluated_schema.pointer_mut(layout_elements_path) {
            *target = Value::Array(resolved_elements);
        }
    }
    
    /// Recursively resolve $ref in an element and its nested elements
    fn resolve_element_ref_recursive(&self, element: Value) -> Value {
        // First resolve the current element's $ref
        let resolved = self.resolve_element_ref(element);
        
        // Then recursively resolve any nested elements arrays
        if let Value::Object(mut map) = resolved {
            // Check if this object has an "elements" array
            if let Some(Value::Array(elements)) = map.get("elements") {
                let mut resolved_nested = Vec::with_capacity(elements.len());
                for nested_element in elements {
                    resolved_nested.push(self.resolve_element_ref_recursive(nested_element.clone()));
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
                if let Some(Value::String(ref_path)) = map.get("$ref") {
                    let normalized_path = path_utils::normalize_to_json_pointer(ref_path);
                    
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

    /// Evaluate fields that depend on changed paths
    /// This re-evaluates dependent fields when source fields change
    pub fn evaluate_dependents(
        &mut self, 
        changed_paths: &[String],
        data: &str,
        context: Option<&str>,
        nested: bool
    ) -> Result<Value, String> {
        // Parse data and context
        let data_value = json_parser::parse_json_str(data)?;
        let context_value = if let Some(ctx) = context {
            json_parser::parse_json_str(ctx)?
        } else {
            Value::Object(serde_json::Map::new())
        };
        
        // Update eval_data with new values
        self.eval_data.replace_data_and_context(data_value.clone(), context_value.clone());
        
        // Collect all paths to re-evaluate
        let mut paths_to_eval: IndexSet<String> = changed_paths.iter().cloned().collect();
        
        if nested {
            // Recursively collect dependent paths
            let mut to_process: Vec<String> = changed_paths.to_vec();
            let mut processed: IndexSet<String> = IndexSet::new();
            
            while let Some(path) = to_process.pop() {
                if processed.contains(&path) {
                    continue;
                }
                processed.insert(path.clone());
                
                // Find all fields that depend on this path
                for (eval_key, deps) in &self.dependencies {
                    if deps.contains(&path) || deps.iter().any(|d| d.starts_with(&path)) {
                        if !paths_to_eval.contains(eval_key) && !eval_key.starts_with("$params") {
                            paths_to_eval.insert(eval_key.clone());
                            to_process.push(eval_key.clone());
                        }
                    }
                }
            }
        }
        
        // Re-evaluate collected paths
        for eval_key in &paths_to_eval {
            let pointer_path = path_utils::normalize_to_json_pointer(eval_key);
            
            // Check if this is a table
            if self.table_metadata.contains_key(eval_key) {
                if let Ok(rows) = table_evaluate::evaluate_table(self, eval_key, &self.eval_data) {
                    let value = Value::Array(rows);
                    self.eval_data.set(&pointer_path, value.clone());
                    if let Some(schema_value) = self.evaluated_schema.pointer_mut(&pointer_path) {
                        *schema_value = value;
                    }
                }
            } else if let Some(logic_id) = self.evaluations.get(eval_key) {
                if let Ok(val) = self.engine.run(logic_id, self.eval_data.data()) {
                    let cleaned_val = clean_float_noise(val);
                    self.eval_data.set(&pointer_path, cleaned_val.clone());
                    if let Some(schema_value) = self.evaluated_schema.pointer_mut(&pointer_path) {
                        *schema_value = cleaned_val;
                    }
                }
            }
        }
        
        Ok(self.evaluated_schema.clone())
    }

    /// Validate form data against schema rules
    /// Returns validation errors for fields that don't meet their rules
    pub fn validate(
        &self,
        data: &str,
        context: Option<&str>,
        paths: Option<&[String]>
    ) -> Result<ValidationResult, String> {
        // Parse data and context
        let data_value = json_parser::parse_json_str(data)?;
        let _context_value = if let Some(ctx) = context {
            json_parser::parse_json_str(ctx)?
        } else {
            Value::Object(serde_json::Map::new())
        };
        
        let mut errors: IndexMap<String, ValidationError> = IndexMap::new();
        
        // Walk through schema and validate rules
        self.validate_object(&self.evaluated_schema, &data_value, "", &mut errors, paths);
        
        let has_error = !errors.is_empty();
        
        Ok(ValidationResult {
            has_error,
            errors,
        })
    }
    
    /// Recursively validate an object against its schema
    fn validate_object(
        &self,
        schema: &Value,
        data: &Value,
        current_path: &str,
        errors: &mut IndexMap<String, ValidationError>,
        filter_paths: Option<&[String]>
    ) {
        if let Value::Object(schema_map) = schema {
            // Check if this field has rules
            if let Some(Value::Object(rules)) = schema_map.get("rules") {
                // Check if we should validate this path
                if let Some(paths) = filter_paths {
                    if !paths.is_empty() && !paths.iter().any(|p| current_path.starts_with(p) || p.starts_with(current_path)) {
                        return;
                    }
                }
                
                // Check if field is hidden (skip validation if hidden)
                if let Some(Value::Object(condition)) = schema_map.get("condition") {
                    if let Some(Value::Bool(true)) = condition.get("hidden") {
                        return;
                    }
                }
                
                // Validate each rule
                for (rule_name, rule_value) in rules {
                    self.validate_rule(
                        current_path,
                        rule_name,
                        rule_value,
                        data,
                        schema_map,
                        errors
                    );
                }
            }
            
            // Recurse into properties
            if let Some(Value::Object(properties)) = schema_map.get("properties") {
                for (prop_name, prop_schema) in properties {
                    let next_path = if current_path.is_empty() {
                        prop_name.clone()
                    } else {
                        format!("{}.{}", current_path, prop_name)
                    };
                    
                    let prop_data = if let Value::Object(data_map) = data {
                        data_map.get(prop_name).unwrap_or(&Value::Null)
                    } else {
                        &Value::Null
                    };
                    
                    self.validate_object(prop_schema, prop_data, &next_path, errors, filter_paths);
                }
            }
        }
    }
    
    /// Validate a single rule
    fn validate_rule(
        &self,
        field_path: &str,
        rule_name: &str,
        rule_value: &Value,
        field_data: &Value,
        schema: &serde_json::Map<String, Value>,
        errors: &mut IndexMap<String, ValidationError>
    ) {
        // Skip if already has error
        if errors.contains_key(field_path) {
            return;
        }
        
        // Check if disabled
        if let Some(Value::Object(condition)) = schema.get("condition") {
            if let Some(Value::Bool(true)) = condition.get("disabled") {
                return;
            }
        }
        
        // Extract rule object
        let (rule_active, rule_message) = match rule_value {
            Value::Object(rule_obj) => {
                let active = rule_obj.get("value").unwrap_or(&Value::Bool(false));
                let message = rule_obj.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Validation failed");
                (active.clone(), message.to_string())
            }
            _ => (rule_value.clone(), "Validation failed".to_string())
        };
        
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
                                    });
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Validation error for a field
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub rule_type: String,
    pub message: String,
}

/// Result of validation
#[derive(Debug)]
pub struct ValidationResult {
    pub has_error: bool,
    pub errors: IndexMap<String, ValidationError>,
}

