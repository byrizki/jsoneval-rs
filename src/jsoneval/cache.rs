use super::JSONEval;
use crate::eval_cache::{CacheKey, CacheStats};
use crate::eval_data::EvalData;
use crate::{is_timing_enabled, path_utils};


use indexmap::IndexSet;
use serde_json::Value;
use std::time::Instant;

impl JSONEval {
    /// Check if a dependency should be part of the cache key
    pub fn should_cache_dependency(&self, dep_path: &str) -> bool {
        // Cache based on:
        // 1. Data paths (starting with /)
        // 2. Context paths (starting with /) - treated same as data
        // 3. Schema paths that point to data (e.g. #/properties/foo/value)

        // Don't cache structural dependencies like loop indices or temporary variables if we can identify them
        // For now, cache everything that looks like a path
        if dep_path.starts_with('/') || dep_path.starts_with("#/") {
            return true;
        }
        false
    }

    /// Try to get a result from cache
    pub(crate) fn try_get_cached(
        &self,
        eval_key: &str,
        eval_data_snapshot: &EvalData,
    ) -> Option<Value> {
        if !self.cache_enabled {
            return None;
        }

        let deps = self.dependencies.get(eval_key)?;

        // Buffer to hold pairs of (dep_key, value_ref)
        let mut key_values = Vec::with_capacity(deps.len());

        for dep in deps {
            if self.should_cache_dependency(dep) {
                // Get value from snapshot
                // Normalize path first
                let pointer_path = path_utils::normalize_to_json_pointer(dep);
                let value = if pointer_path.starts_with("#/") {
                    // It's a schema path - check if it points to a value we track
                    // For caching purposes, we care about the DATA value at that schema path
                    // So convert to data path
                    let data_path = pointer_path.replace("/properties/", "/").replace("#", "");
                    eval_data_snapshot
                        .data()
                        .pointer(&data_path)
                        .unwrap_or(&Value::Null)
                } else {
                    // Direct data path
                    eval_data_snapshot
                        .data()
                        .pointer(&pointer_path)
                        .unwrap_or(&Value::Null)
                };

                key_values.push((dep.clone(), value));
            }
        }

        let key = CacheKey::new(eval_key.to_string(), deps, &key_values);

        self.eval_cache.get(&key).map(|v| v.as_ref().clone())
    }

    /// Cache a result
    pub(crate) fn cache_result(
        &self,
        eval_key: &str,
        _result: Value,
        eval_data_snapshot: &EvalData,
    ) {
        if !self.cache_enabled {
            return;
        }

        if let Some(deps) = self.dependencies.get(eval_key) {
             // Buffer to hold pairs of (dep_key, value_ref)
            let mut key_values = Vec::with_capacity(deps.len());

            for dep in deps {
                if self.should_cache_dependency(dep) {
                    let pointer_path = path_utils::normalize_to_json_pointer(dep);
                    let value = if pointer_path.starts_with("#/") {
                        let data_path = pointer_path.replace("/properties/", "/").replace("#", "");
                        eval_data_snapshot
                            .data()
                            .pointer(&data_path)
                            .unwrap_or(&Value::Null)
                    } else {
                        eval_data_snapshot
                            .data()
                            .pointer(&pointer_path)
                            .unwrap_or(&Value::Null)
                    };

                    key_values.push((dep.clone(), value));
                }
            }

            let key = CacheKey::new(eval_key.to_string(), deps, &key_values);

            self.eval_cache.insert(key, _result);
        }
    }

    /// Purge cache entries affected by changed data paths, comparing old and new values
    pub fn purge_cache_for_changed_data_with_comparison(
        &self,
        changed_paths: &[String],
        old_data: &Value,
        new_data: &Value,
    ) {
        // Collect actual changed paths by comparing values
        let mut actual_changes = Vec::new();

        for path in changed_paths {
            let pointer = if path.starts_with('/') {
                path.clone()
            } else {
                format!("/{}", path)
            };

            let old_val = old_data.pointer(&pointer).unwrap_or(&Value::Null);
            let new_val = new_data.pointer(&pointer).unwrap_or(&Value::Null);

            if old_val != new_val {
                actual_changes.push(path.clone());
            }
        }

        if !actual_changes.is_empty() {
            self.purge_cache_for_changed_data(&actual_changes);
        }
    }

    /// Purge cache entries affected by changed data paths
    pub fn purge_cache_for_changed_data(&self, changed_paths: &[String]) {
        if changed_paths.is_empty() {
            return;
        }

        // We need to find cache entries that depend on these paths.
        // Since we don't have a reverse mapping from dependency -> cache keys readily available for specific values,
        // we iterate the cache.
        // IMPROVEMENT: Maintain a dependency graph for cache invalidation?
        // Current implementation: Iterate all cache keys and check if they depend on changed paths.

        // Collect keys to remove to avoid borrowing issues
        // EvalCache internal structure (DashMap) allows concurrent removal, but we don't have direct access here easily w/o iterating
        // `eval_cache` in struct is `EvalCache`.

        let start = Instant::now();
        let initial_size = self.eval_cache.len();

        // Convert changed paths to a format easier to match against dependencies
        // Cache dependencies are stored as original strings from logic (e.g. "path/to/field", "#/path", "/path")
        // We need flexible matching.
        let paths_set: IndexSet<String> = changed_paths.iter().cloned().collect();

        self.eval_cache.retain(|key, _| {
            // key.dependencies is Vec<(String, Value)> (Wait, CacheKey deps logic might be different?)
            // Actually CacheKey doesn't expose dependencies easily? 
            // Ah, CacheKey struct: pub eval_key: String. It does NOT store dependencies list publically?
            // checking eval_cache.rs: struct CacheKey { pub eval_key: String, pub deps_hash: u64 }.
            // It does NOT store the dependencies themselves! 
            // So we CANNOT check dependencies from key!
            
            // This implies my purge logic is BROKEN if I can't access dependencies.
            // But strict adherence to `lib.rs`: How did `lib.rs` do it?
            // `lib.rs` view 1600+?
            // If CacheKey doesn't store dependencies, then we can't iterate dependencies.
            // But `JSONEval` has `dependencies: Arc<IndexMap<String, IndexSet<String>>>`.
            // We can look up dependencies using `key.eval_key`!
            
            if let Some(deps) = self.dependencies.get(&key.eval_key) {
                !deps.iter().any(|dep_path| {
                    self.paths_match_flexible(dep_path, &paths_set)
                })
            } else {
                // No dependencies recorded? Keep it.
                true
            }
        });

        if is_timing_enabled() {
            let _duration = start.elapsed();
            let removed = initial_size - self.eval_cache.len();
            if removed > 0 {
                // Record timing if needed, or just debug log
                // println!("Purged {} cache entries in {:?}", removed, duration);
            }
        }
    }

    /// Helper to check if a dependency path matches any of the changed paths
    pub(crate) fn paths_match_flexible(
        &self,
        dep_path: &str,
        changed_paths: &IndexSet<String>,
    ) -> bool {
        // Normalize dep_path to slash format for comparison
        let normalized_dep = path_utils::normalize_to_json_pointer(dep_path);
        let normalized_dep_slash = normalized_dep.replace("#", ""); // e.g. /properties/foo

        for changed in changed_paths {
            // changed is usually like "/foo" or "/foo/bar"
            // normalized_dep like "/properties/foo" or "/foo"

            // 1. Exact match (ignoring /properties/ noise)
            // stripped_dep: /foo
            let stripped_dep = normalized_dep_slash.replace("/properties/", "/");

            if stripped_dep == *changed {
                return true;
            }

            // 2. Ancestor/Descendant check
            // If data at "/foo" changed, then dependency on "/foo/bar" is invalid
            if stripped_dep.starts_with(changed) && stripped_dep.chars().nth(changed.len()) == Some('/') {
                return true;
            }

            // If data at "/foo/bar" changed, then dependency on "/foo" is invalid (if it evaluates object)
            // But we don't know if it evaluates object or is just a structural parent.
            // Safe bet: invalidate.
            if changed.starts_with(&stripped_dep) && changed.chars().nth(stripped_dep.len()) == Some('/') {
                // EXCEPTION: If dep is purely structural (e.g. existence check), might be fine?
                // But generally safe to invalidate.
                return true;
            }
        }

        false
    }
    
    /// Purge cache entries affected by context changes
    pub fn purge_cache_for_context_change(&self) {
        // Invalidate anything that depends on context
        // Context dependencies usually start with "/" but point to context?
        // Or they are special variables?
        // For now, invalidate all keys that have dependencies NOT starting with # (assuming # is schema/internal)
        // AND not starting with / (data).
        // Actually, context is accessed via /variables etc?
        // If we can't distinguish, we might need to clear all?
        // Or check if dependency is in context?
        
        // Safer approach: Clear everything if context changes?
        // Or iterate and check if dependency is NOT found in data?
        
        // Current implementation in lib.rs purges everything if context provided?
        // Line 1160 in lib.rs: "if context_provided { self.purge_cache_for_context_change(); }"
        // And implementation?
        
        // Let's implement based on checking dependencies for context-like paths
        // Assuming context paths start with / and data paths also start with /.
        // If we can't distinguish, we have to clear all entries with / dependencies.
        
        self.eval_cache.retain(|key, _| {
             if let Some(deps) = self.dependencies.get(&key.eval_key) {
                 !deps.iter().any(|dep_path| {
                     dep_path.starts_with('/') && !dep_path.starts_with("#")
                 })
             } else {
                 true
             }
        });
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.eval_cache.stats()
    }

    /// Clear the cache manually
    pub fn clear_cache(&self) {
        self.eval_cache.clear();
    }

    /// Enable caching
    pub fn enable_cache(&mut self) {
        self.cache_enabled = true;
        for subform in self.subforms.values_mut() {
            subform.enable_cache();
        }
    }

    /// Disable caching
    pub fn disable_cache(&mut self) {
        self.cache_enabled = false;
        self.eval_cache.clear();
        for subform in self.subforms.values_mut() {
            subform.disable_cache();
        }
    }
    
    /// Check if cache is enabled
    pub fn is_cache_enabled(&self) -> bool {
        self.cache_enabled
    }
    
    /// Get cache size
    pub fn cache_len(&self) -> usize {
        self.eval_cache.len()
    }
}
