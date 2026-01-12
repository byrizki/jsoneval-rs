use super::JSONEval;
use crate::jsoneval::eval_cache::{CacheKey, CacheStats};
use crate::jsoneval::eval_data::EvalData;


use indexmap::IndexSet;
use serde_json::Value;


impl JSONEval {
    /// Check if a dependency should be part of the cache key
    /// Check if a dependency should be cached
    /// Caches everything except keys starting with $ (except $context)
    #[inline]
    pub fn should_cache_dependency(&self, key: &str) -> bool {
        if key.starts_with("/$") || key.starts_with('$') {
            // Only cache $context, exclude other $ keys like $params
            key == "$context" || key.starts_with("$context.") || key.starts_with("/$context")
        } else {
            true
        }
    }

    /// Try to get a result from cache
    /// Helper: Try to get cached result for an evaluation (thread-safe)
    /// Helper: Try to get cached result (zero-copy via Arc)
    pub(crate) fn try_get_cached(
        &self,
        eval_key: &str,
        eval_data_snapshot: &EvalData,
    ) -> Option<Value> {
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
                .filter(|dep_key| self.should_cache_dependency(dep_key))
                .cloned()
                .collect();

            // Collect dependency values
            let dep_values: Vec<(String, &Value)> = filtered_deps
                .iter()
                .filter_map(|dep_key| eval_data_snapshot.get(dep_key).map(|v| (dep_key.clone(), v)))
                .collect();

            CacheKey::new(eval_key.to_string(), &filtered_deps, &dep_values)
        };

        // Try cache lookup (zero-copy via Arc, thread-safe)
        self.eval_cache
            .get(&cache_key)
            .map(|arc_val| (*arc_val).clone())
    }

    /// Cache a result
    /// Helper: Store evaluation result in cache (thread-safe)
    pub(crate) fn cache_result(
        &self,
        eval_key: &str,
        value: Value,
        eval_data_snapshot: &EvalData,
    ) {
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
            .filter(|dep_key| self.should_cache_dependency(dep_key))
            .cloned()
            .collect();

        let dep_values: Vec<(String, &Value)> = filtered_deps
            .iter()
            .filter_map(|dep_key| eval_data_snapshot.get(dep_key).map(|v| (dep_key.clone(), v)))
            .collect();

        let cache_key = CacheKey::new(eval_key.to_string(), &filtered_deps, &dep_values);
        self.eval_cache.insert(cache_key, value);
    }

    /// Purge cache entries affected by changed data paths, comparing old and new values
    /// Selectively purge cache entries that depend on changed data paths
    /// Only removes cache entries whose dependencies intersect with changed_paths
    /// Compares old vs new values and only purges if values actually changed
    pub fn purge_cache_for_changed_data_with_comparison(
        &self,
        changed_data_paths: &[String],
        old_data: &Value,
        new_data: &Value,
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
                    dep == changed_path
                        || dep.starts_with(&format!("{}/", changed_path))
                        || changed_path.starts_with(&format!("{}/", dep))
                })
            });

            if is_affected {
                affected_eval_keys.insert(eval_key.clone());
            }
        }

        // Remove all cache entries for affected eval_keys using retain
        // Keep entries whose eval_key is NOT in the affected set
        self.eval_cache
            .retain(|cache_key, _| !affected_eval_keys.contains(&cache_key.eval_key));
    }

    /// Selectively purge cache entries that depend on changed data paths
    /// Finds all eval_keys that depend on the changed paths and removes them
    /// Selectively purge cache entries that depend on changed data paths
    /// Simpler version without value comparison for cases where we don't have old data
    pub fn purge_cache_for_changed_data(&self, changed_data_paths: &[String]) {
        if changed_data_paths.is_empty() {
            return;
        }

        // Find all eval_keys that depend on the changed paths
        let mut affected_eval_keys = IndexSet::new();

        for (eval_key, deps) in self.dependencies.iter() {
            // Check if this evaluation depends on any of the changed paths
            let is_affected = deps.iter().any(|dep| {
                // Check if dependency path matches any changed data path using flexible matching
                changed_data_paths.iter().any(|changed_for_purge| {
                    // Check both directions:
                    // 1. Dependency matches changed data (dependency is child of change)
                    // 2. Changed data matches dependency (change is child of dependency)
                    Self::paths_match_flexible(dep, changed_for_purge)
                        || Self::paths_match_flexible(changed_for_purge, dep)
                })
            });

            if is_affected {
                affected_eval_keys.insert(eval_key.clone());
            }
        }

        // Remove all cache entries for affected eval_keys using retain
        // Keep entries whose eval_key is NOT in the affected set
        self.eval_cache
            .retain(|cache_key, _| !affected_eval_keys.contains(&cache_key.eval_key));
    }

    /// Flexible path matching that handles structural schema keywords (e.g. properties, oneOf)
    /// Returns true if schema_path structurally matches data_path
    fn paths_match_flexible(schema_path: &str, data_path: &str) -> bool {
        let s_segs: Vec<&str> = schema_path
            .trim_start_matches('#')
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        let d_segs: Vec<&str> = data_path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let mut d_idx = 0;

        for s_seg in s_segs {
            // If we matched all data segments, we are good (schema is deeper/parent)
            if d_idx >= d_segs.len() {
                return true;
            }

            let d_seg = d_segs[d_idx];

            if s_seg == d_seg {
                // Exact match, advance data pointer
                d_idx += 1;
            } else if s_seg == "items"
                || s_seg == "additionalProperties"
                || s_seg == "patternProperties"
            {
                // Wildcard match for arrays/maps - consume data segment if it looks valid
                // Note: items matches array index (numeric). additionalProperties matches any key.
                if s_seg == "items" {
                    // Only match if data segment is numeric (array index)
                    if d_seg.chars().all(|c| c.is_ascii_digit()) {
                        d_idx += 1;
                    }
                } else {
                    // additionalProperties/patternProperties matches any string key
                    d_idx += 1;
                }
            } else if Self::is_structural_keyword(s_seg)
                || s_seg.chars().all(|c| c.is_ascii_digit())
            {
                // Skip structural keywords (properties, oneOf, etc) and numeric indices in schema (e.g. oneOf/0)
                continue;
            } else {
                // Mismatch: schema has a named segment that data doesn't have
                return false;
            }
        }

        // Return true if we consumed all data segments
        true
    }
    
    /// Purge cache entries affected by context changes
    /// Purge cache entries that depend on context
    pub fn purge_cache_for_context_change(&self) {
        // Find all eval_keys that depend on $context
        let mut affected_eval_keys = IndexSet::new();

        for (eval_key, deps) in self.dependencies.iter() {
            let is_affected = deps.iter().any(|dep| {
                dep == "$context" || dep.starts_with("$context.") || dep.starts_with("/$context")
            });

            if is_affected {
                affected_eval_keys.insert(eval_key.clone());
            }
        }

        self.eval_cache
            .retain(|cache_key, _| !affected_eval_keys.contains(&cache_key.eval_key));
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

    /// Helper to check if a key is a structural JSON Schema keyword
    /// Helper to check if a key is a structural JSON Schema keyword
    fn is_structural_keyword(key: &str) -> bool {
        matches!(
            key,
            "properties"
                | "definitions"
                | "$defs"
                | "allOf"
                | "anyOf"
                | "oneOf"
                | "not"
                | "if"
                | "then"
                | "else"
                | "dependentSchemas"
                | "$params"
                | "dependencies"
        )
    }
    
    /// Get cache size
    pub fn cache_len(&self) -> usize {
        self.eval_cache.len()
    }
}
