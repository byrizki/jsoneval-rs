use super::JSONEval;
use crate::jsoneval::eval_cache::{CacheKey, CacheStats};
use crate::jsoneval::eval_data::EvalData;
use crate::jsoneval::path_utils;


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
            key.starts_with("$context.") || key.starts_with("/$context") || key.starts_with("$context/")
                || key.starts_with("$params.") || key.starts_with("/$params") || key.starts_with("$params/")
        } else {
            true
        }
    }

    /// Try to get a value from cache if it exists and dependencies match
    pub(crate) fn try_get_cached(
        &self,
        eval_key: &str,
        eval_data_snapshot: &EvalData,
    ) -> Option<Value> {
        if !self.cache_enabled {
            return None;
        }

        let empty_deps = IndexSet::new();
        let deps = self.dependencies.get(eval_key).unwrap_or(&empty_deps);
 
        let cache_key = if deps.is_empty() {
            CacheKey::simple(eval_key.to_string())
        } else {
            let normalized_eval_key = path_utils::normalize_to_json_pointer(eval_key);
            if let Some(_) = deps.iter().find(|dep| {
                // Skip self-dependency: a key should not invalidate itself
                dep.as_str() != normalized_eval_key.as_ref()
                    && self.missed_keys.contains(dep.as_str())
            }) {
                return None;
            }

            let filtered_deps: IndexSet<String> = deps
                .iter()
                .filter(|dep_key| self.should_cache_dependency(dep_key) && dep_key.as_str() != path_utils::normalize_to_json_pointer(eval_key))
                .cloned()
                .collect();

            // Collect dep values; for $params deps use the version counter as the value.
            let dep_values: Vec<(String, Value)> = filtered_deps
                .iter()
                .filter_map(|dep_key| {
                    if self.is_params_dep(dep_key) {
                        let ver = self.params_version_for(dep_key);
                        return Some((dep_key.clone(), Value::Number(ver.into())));
                    }
                    eval_data_snapshot.get(dep_key).and_then(|v| {
                        if let Value::Array(arr) = v {
                            if arr.len() > 10 && !crate::parse_schema::common::has_actionable_keys(v) {
                                return None;
                            }
                        }
                        Some((dep_key.clone(), v.clone()))
                    })
                })
                .collect();

            let dep_refs: Vec<(String, &Value)> = dep_values.iter().map(|(k, v)| (k.clone(), v)).collect();
 
            CacheKey::new(eval_key.to_string(), &filtered_deps, &dep_refs)
        };

        let result = self.eval_cache
            .get(&cache_key)
            .map(|arc_val| (*arc_val).clone());
            
            
        result
    }

    /// Cache a result
    pub(crate) fn cache_result(
        &self,
        eval_key: &str,
        value: Value,
        eval_data_snapshot: &EvalData,
    ) {
        if !self.cache_enabled {
            return;
        }
 
        let cache_key = if let Some(deps) = self.dependencies.get(eval_key) {
            if deps.is_empty() {
                CacheKey::simple(eval_key.to_string())
            } else {
                let filtered_deps: IndexSet<String> = deps
                    .iter()
                    .filter(|dep_key| self.should_cache_dependency(dep_key) && dep_key.as_str() != path_utils::normalize_to_json_pointer(eval_key))
                    .cloned()
                    .collect();
    
                // Collect dep values; for $params deps use the version counter as the value.
                let dep_values: Vec<(String, Value)> = filtered_deps
                    .iter()
                    .filter_map(|dep_key| {
                        if self.is_params_dep(dep_key) {
                            let ver = self.params_version_for(dep_key);
                            return Some((dep_key.clone(), Value::Number(ver.into())));
                        }
                        eval_data_snapshot.get(dep_key).and_then(|v| {
                            if let Value::Array(arr) = v {
                                if arr.len() > 10 && !crate::parse_schema::common::has_actionable_keys(v) {
                                    return None;
                                }
                            }
                            Some((dep_key.clone(), v.clone()))
                        })
                    })
                    .collect();
    
                let dep_refs: Vec<(String, &Value)> = dep_values.iter().map(|(k, v)| (k.clone(), v)).collect();
                CacheKey::new(eval_key.to_string(), &filtered_deps, &dep_refs)
            }
        } else {
            CacheKey::simple(eval_key.to_string())
        };
 
        self.eval_cache.insert(cache_key, value);
    }

    /// Returns true if `dep_key` refers to a `$params` evaluation.
    #[inline]
    fn is_params_dep(&self, dep_key: &str) -> bool {
        (dep_key.starts_with("/$params") || dep_key.starts_with("$params.") || dep_key.starts_with("$params/"))
            && self.params_versions.contains_key(dep_key)
    }

    /// Returns the current version counter for a `$params` dep key.
    /// Falls back to 0 if the key is not tracked (shouldn't happen in practice).
    #[inline]
    fn params_version_for(&self, dep_key: &str) -> u64 {
        self.params_versions
            .get(dep_key)
            .map(|v| *v)
            .unwrap_or(0)
    }

    /// Bump the version counter for a `$params` evaluation key.
    /// Called immediately after every uncached `$params` evaluation run.
    /// `eval_key` is the raw schema key (e.g. `#/$params/accessList`).
    pub(crate) fn bump_params_version(&self, eval_key: &str) {
        let norm = eval_key.trim_start_matches('#');
        if let Some(mut ver) = self.params_versions.get_mut(norm) {
            *ver += 1;
        }
    }

    /// Populate params_versions from the current evaluations map.
    /// Should be called once after schema is parsed (or reloaded).
    pub(crate) fn init_params_versions(&mut self) {
        self.params_versions.clear();
        for eval_key in self.evaluations.keys() {
            let norm = eval_key.trim_start_matches('#');
            if norm.starts_with("/$params/") || norm == "/$params" {
                self.params_versions.insert(norm.to_string(), 0);
            }
        }
        for table_key in self.tables.keys() {
            let norm = table_key.trim_start_matches('#');
            if norm.starts_with("/$params/") || norm == "/$params" {
                self.params_versions.insert(norm.to_string(), 0);
            }
        }
    }

    /// Purge cache entries affected by changed data paths, comparing old and new values
    /// Selectively purge cache entries that depend on changed data paths
    /// Only removes cache entries whose dependencies intersect with changed_paths
    /// Recursively find all paths where the values differ
    fn compute_changed_paths(old: &Value, new: &Value, current_path: String, changed: &mut Vec<String>) {
        if old == new {
            return;
        }

        match (old, new) {
            (Value::Object(o1), Value::Object(o2)) => {
                for (k, v2) in o2 {
                    if let Some(v1) = o1.get(k) {
                        if v1 != v2 {
                            let path = format!("{}/{}", current_path, k);
                            Self::compute_changed_paths(v1, v2, path, changed);
                        }
                    } else {
                        let path = format!("{}/{}", current_path, k);
                        changed.push(path);
                    }
                }
                for k in o1.keys() {
                    if !o2.contains_key(k) {
                        let path = format!("{}/{}", current_path, k);
                        changed.push(path);
                    }
                }
            }
            (Value::Array(a1), Value::Array(a2)) if a1.len() == a2.len() => {
                for (i, (v1, v2)) in a1.iter().zip(a2.iter()).enumerate() {
                    if v1 != v2 {
                        let path = format!("{}/{}", current_path, i);
                        Self::compute_changed_paths(v1, v2, path, changed);
                    }
                }
            }
            _ => {
                if current_path.is_empty() {
                    changed.push("/".to_string());
                } else {
                    changed.push(current_path);
                }
            }
        }
    }

    /// Purge cache entries whose dependencies intersect a known set of written data paths.
    /// Used after `process_dependents_queue` / `recursive_hide_effect` to avoid a full cache clear.
    /// `written_paths` are plain JSON pointer data paths (e.g. `/illustration/insured/name`),
    /// i.e. the `/properties/` segments are already stripped out.
    pub(crate) fn purge_cache_for_affected_data_paths(&self, written_paths: &IndexSet<String>) {
        if written_paths.is_empty() {
            return;
        }

        let mut affected_eval_keys = IndexSet::new();

        for (eval_key, deps) in self.dependencies.iter() {
            let is_affected = deps.iter().any(|dep| {
                // Skip $params deps — they are handled via version tokens
                if dep.starts_with("/$params") || dep.starts_with("$params.") || dep.starts_with("$params/") {
                    return false;
                }
                // Normalize dep paths (strip /properties/) to match written data paths
                let dep_data = dep.replace("/properties/", "/");
                written_paths.iter().any(|written| {
                    dep_data == *written
                        || dep_data.starts_with(&format!("{}/", written))
                        || written.starts_with(&format!("{}/", dep_data))
                })
            });

            if is_affected {
                affected_eval_keys.insert(eval_key.clone());
            }
        }

        self.eval_cache
            .retain(|cache_key, _| !affected_eval_keys.contains(&cache_key.eval_key));
    }


    /// Shared helper: deep-diff `old` vs `new`, then purge all cache entries whose
    /// dependencies are matched by `dep_matches(dep_key, &changed_paths)`.
    fn purge_cache_for_changed_values<F>(&self, old: &Value, new: &Value, dep_matches: F)
    where
        F: Fn(&str, &[String]) -> bool,
    {
        let mut changed_paths = Vec::new();
        Self::compute_changed_paths(old, new, String::new(), &mut changed_paths);

        if changed_paths.is_empty() {
            return;
        }

        let mut affected_eval_keys = IndexSet::new();

        for (eval_key, deps) in self.dependencies.iter() {
            if deps.iter().any(|dep| dep_matches(dep, &changed_paths)) {
                affected_eval_keys.insert(eval_key.clone());
            }
        }

        self.eval_cache
            .retain(|cache_key, _| !affected_eval_keys.contains(&cache_key.eval_key));
    }

    /// Deep-diff old vs new form data and purge affected cache entries.
    pub fn purge_cache_for_changed_data_with_comparison(&self, old_data: &Value, new_data: &Value) {
        self.purge_cache_for_changed_values(old_data, new_data, |dep, changed_paths| {
            // TODO: $params dependencies need separate handling — their values are
            // derived from schema evaluations, not raw form data. Skip for now to
            // avoid over-invalidating cache entries that depend on computed params.
            if dep.starts_with("/$params") || dep.starts_with("$params.") || dep.starts_with("$params/") {
                return false;
            }

            changed_paths.iter().any(|p| {
                dep == p || dep.starts_with(&format!("{}/", p)) || p.starts_with(&format!("{}/", dep))
            })
        });
    }

    /// Deep-diff old vs new context and purge only entries that depend on
    /// the specific context fields that actually changed.
    pub fn purge_cache_for_changed_context_with_comparison(
        &self,
        old_context: &Value,
        new_context: &Value,
    ) {
        self.purge_cache_for_changed_values(old_context, new_context, |dep, changed_paths| {
            // Only consider $context dependencies
            if !(dep == "$context"
                || dep.starts_with("$context.")
                || dep.starts_with("/$context")
                || dep.starts_with("$context/"))
            {
                return false;
            }

            // Root-context dependency — always affected when any context field changed
            if dep == "$context" {
                return true;
            }

            // Normalise each changed relative path to both dep-key formats and match:
            //   /profile/sob  ->  "/$context/profile/sob"  (slash form)
            //                     "$context.profile.sob"   (dot form)
            changed_paths.iter().any(|p| {
                if p == "/" || p.is_empty() {
                    return true;
                }
                let slash = format!("/$context{}", p);
                let dot = format!("$context{}", p.replace('/', "."));
                dep == slash
                    || dep == dot
                    || dep.starts_with(&format!("{}/", slash))
                    || dep.starts_with(&format!("{}.", dot))
                    || slash.starts_with(&format!("{}/", dep))
                    || dot.starts_with(&format!("{}.", dep))
            })
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
