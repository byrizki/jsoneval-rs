use std::collections::{HashMap, HashSet};
use indexmap::IndexSet;
use serde_json::Value;

/// Token-version tracker for json paths
#[derive(Default, Clone)]
pub struct VersionTracker {
    versions: HashMap<String, u64>,
}

impl VersionTracker {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    #[inline]
    pub fn get(&self, path: &str) -> u64 {
        self.versions.get(path).copied().unwrap_or(0)
    }

    #[inline]
    pub fn bump(&mut self, path: &str) {
        let current = self.get(path);
        // We use actual data pointers here
        self.versions.insert(path.to_string(), current + 1);
    }

    pub fn merge_from(&mut self, other: &VersionTracker) {
        for (k, v) in &other.versions {
            self.versions.insert(k.clone(), *v);
        }
    }

    /// Returns true if any of the given paths has a version > 0 (i.e. was bumped at least once).
    /// Used to decide whether a subform needs a `re_evaluate` pass.
    pub fn has_any_version_for(&self, paths: &[String]) -> bool {
        paths.iter().any(|p| self.versions.get(p).copied().unwrap_or(0) > 0)
    }
}

/// A cached evaluation result with the specific dependency versions it was evaluated against
#[derive(Clone)]
pub struct CacheEntry {
    pub dep_versions: HashMap<String, u64>,
    pub result: Value,
}

/// Independent cache state for a single item in a subform array
#[derive(Default, Clone)]
pub struct SubformItemCache {
    pub data_versions: VersionTracker,
    pub entries: HashMap<String, CacheEntry>,
    pub item_snapshot: Value,
}

impl SubformItemCache {
    pub fn new() -> Self {
        Self {
            data_versions: VersionTracker::new(),
            entries: HashMap::new(),
            item_snapshot: Value::Null,
        }
    }
}

/// Primary cache structure for a JSON evaluation instance
#[derive(Clone)]
pub struct EvalCache {
    pub data_versions: VersionTracker,
    pub params_versions: VersionTracker,
    pub entries: HashMap<String, CacheEntry>,

    pub active_item_index: Option<usize>,
    pub subform_caches: HashMap<usize, SubformItemCache>,

    /// Monotonically increasing counter bumped whenever data_versions or params_versions change.
    /// When `eval_generation == last_evaluated_generation`, all cache entries are guaranteed valid
    /// and `evaluate_internal` can skip the full tree traversal.
    pub eval_generation: u64,
    pub last_evaluated_generation: u64,
}

impl Default for EvalCache {
    fn default() -> Self {
        Self::new()
    }
}

impl EvalCache {
    pub fn new() -> Self {
        Self {
            data_versions: VersionTracker::new(),
            params_versions: VersionTracker::new(),
            entries: HashMap::new(),
            active_item_index: None,
            subform_caches: HashMap::new(),
            eval_generation: 0,
            last_evaluated_generation: u64::MAX, // force first evaluate_internal to run
        }
    }

    pub fn clear(&mut self) {
        self.data_versions = VersionTracker::new();
        self.params_versions = VersionTracker::new();
        self.entries.clear();
        self.active_item_index = None;
        self.subform_caches.clear();
        self.eval_generation = 0;
        self.last_evaluated_generation = u64::MAX;
    }

    /// Returns true if evaluate_internal must run (versions changed since last full evaluation)
    pub fn needs_full_evaluation(&self) -> bool {
        self.eval_generation != self.last_evaluated_generation
    }

    /// Call after evaluate_internal completes successfully to mark the generation stable
    pub fn mark_evaluated(&mut self) {
        self.last_evaluated_generation = self.eval_generation;
    }

    pub(crate) fn ensure_active_item_cache(&mut self, idx: usize) {
        self.subform_caches.entry(idx).or_insert_with(SubformItemCache::new);
    }

    pub fn set_active_item(&mut self, idx: usize) {
        self.active_item_index = Some(idx);
        self.ensure_active_item_cache(idx);
    }

    pub fn clear_active_item(&mut self) {
        self.active_item_index = None;
    }

    /// Recursively diffs `old` against `new` and bumps version for every changed data path scalar.
    pub fn store_snapshot_and_diff_versions(&mut self, old: &Value, new: &Value) {
        if let Some(idx) = self.active_item_index {
            self.ensure_active_item_cache(idx);
            let sub_cache = self.subform_caches.get_mut(&idx).unwrap();
            diff_and_update_versions(&mut sub_cache.data_versions, "", old, new);
            sub_cache.item_snapshot = new.clone();
        } else {
            diff_and_update_versions(&mut self.data_versions, "", old, new);
        }
    }

    pub fn get_active_snapshot(&self) -> Value {
        if let Some(idx) = self.active_item_index {
            self.subform_caches.get(&idx).map(|c| c.item_snapshot.clone()).unwrap_or(Value::Null)
        } else {
            Value::Null
        }
    }

    pub fn diff_active_item(&mut self, field_key: &str, old_sub_data: &Value, new_sub_data: &Value) {
        if let Some(idx) = self.active_item_index {
            self.ensure_active_item_cache(idx);
            let sub_cache = self.subform_caches.get_mut(&idx).unwrap();
            
            // Diff ONLY the localized item part, skipping the massive parent tree
            let empty = Value::Null;
            let old_item = old_sub_data.get(field_key).unwrap_or(&empty);
            let new_item = new_sub_data.get(field_key).unwrap_or(&empty);
            
            diff_and_update_versions(&mut sub_cache.data_versions, &format!("/{}", field_key), old_item, new_item);
            sub_cache.item_snapshot = new_sub_data.clone();
        }
    }

    pub fn bump_data_version(&mut self, data_path: &str) {
        if let Some(idx) = self.active_item_index {
            if let Some(cache) = self.subform_caches.get_mut(&idx) {
                cache.data_versions.bump(data_path);
            }
        } else {
            self.data_versions.bump(data_path);
            self.eval_generation += 1;
        }
    }

    pub fn bump_params_version(&mut self, data_path: &str) {
        self.params_versions.bump(data_path);
        self.eval_generation += 1;
    }

    /// Check if the `eval_key` result can be safely bypassed because dependencies are unchanged.
    ///
    /// Two-tier lookup:
    /// - Tier 1: item-scoped entries in `subform_caches[idx]` — checked first when an active item is set
    /// - Tier 2: global `self.entries` — allows Run 1 (main form) results to be reused in Run 2 (subform)
    pub fn check_cache(&self, eval_key: &str, deps: &IndexSet<String>) -> Option<Value> {
        if let Some(idx) = self.active_item_index {
            // Tier 1: item-specific entries
            if let Some(cache) = self.subform_caches.get(&idx) {
                if let Some(hit) = self.validate_entry(eval_key, deps, &cache.entries, &cache.data_versions) {
                    return Some(hit);
                }
            }
            // Tier 2: global entries (may have been stored by main-form Run 1)
            // Validate against item-scoped data_versions so stale per-item deps aren't silently reused
            let item_data_versions = self.subform_caches
                .get(&idx)
                .map(|c| &c.data_versions)
                .unwrap_or(&self.data_versions);
            self.validate_entry(eval_key, deps, &self.entries, item_data_versions)
        } else {
            self.validate_entry(eval_key, deps, &self.entries, &self.data_versions)
        }
    }

    fn validate_entry(
        &self,
        eval_key: &str,
        deps: &IndexSet<String>,
        entries: &HashMap<String, CacheEntry>,
        data_versions: &VersionTracker,
    ) -> Option<Value> {
        let entry = entries.get(eval_key)?;
        for dep in deps {
            let data_dep_path = crate::jsoneval::path_utils::normalize_to_json_pointer(dep).replace("/properties/", "/");

            let current_ver = if data_dep_path.starts_with("/$params") {
                self.params_versions.get(&data_dep_path)
            } else {
                data_versions.get(&data_dep_path)
            };

            if let Some(&cached_ver) = entry.dep_versions.get(&data_dep_path) {
                if current_ver != cached_ver {
                    if std::env::var("JSONEVAL_DEBUG_CACHE").is_ok() {
                        println!("Cache MISS {}: dep {} changed ({} -> {})", eval_key, data_dep_path, cached_ver, current_ver);
                    }
                    return None;
                }
            } else {
                if std::env::var("JSONEVAL_DEBUG_CACHE").is_ok() {
                    println!("Cache MISS {}: dep {} missing from cache entry", eval_key, data_dep_path);
                }
                return None;
            }
        }
        if std::env::var("JSONEVAL_DEBUG_CACHE").is_ok() {
            println!("Cache HIT {}", eval_key);
        }
        Some(entry.result.clone())
    }

    /// Store the newly evaluated value and snapshot the dependency versions.
    ///
    /// Storage strategy:
    /// - When an active item is set, store into `subform_caches[idx].entries` (item-scoped).
    ///   This isolates per-rider results so different items with different data don't collide.
    /// - The global `self.entries` is written only from the main form (no active item).
    ///   Subforms can reuse these via the Tier 2 fallback in `check_cache`.
    pub fn store_cache(&mut self, eval_key: &str, deps: &IndexSet<String>, result: Value) {
        // Phase 1: snapshot dep versions using the correct data_versions tracker
        let mut dep_versions = HashMap::with_capacity(deps.len());
        {
            let data_versions = if let Some(idx) = self.active_item_index {
                self.ensure_active_item_cache(idx);
                &self.subform_caches[&idx].data_versions
            } else {
                &self.data_versions
            };

            for dep in deps {
                let data_dep_path = crate::jsoneval::path_utils::normalize_to_json_pointer(dep).replace("/properties/", "/");
                let ver = if data_dep_path.starts_with("/$params") {
                    self.params_versions.get(&data_dep_path)
                } else {
                    data_versions.get(&data_dep_path)
                };
                dep_versions.insert(data_dep_path, ver);
            }
        }

        // Phase 2: insert into the correct tier
        let entry = CacheEntry { dep_versions, result };
        if let Some(idx) = self.active_item_index {
            // Store item-scoped: isolates per-rider entries so riders with different data don't collide
            self.subform_caches.get_mut(&idx).unwrap().entries.insert(eval_key.to_string(), entry);
        } else {
            self.entries.insert(eval_key.to_string(), entry);
        }
    }
}

/// Recursive helper to walk JSON structures and bump specific leaf versions where they differ
pub(crate) fn diff_and_update_versions(tracker: &mut VersionTracker, pointer: &str, old: &Value, new: &Value) {
    if pointer.is_empty() {
        diff_and_update_versions_internal(tracker, "", old, new);
    } else {
        diff_and_update_versions_internal(tracker, pointer, old, new);
    }
}

fn diff_and_update_versions_internal(tracker: &mut VersionTracker, pointer: &str, old: &Value, new: &Value) {
    if old == new {
        return;
    }

    match (old, new) {
        (Value::Object(a), Value::Object(b)) => {
            let mut keys = HashSet::new();
            for k in a.keys() { keys.insert(k.as_str()); }
            for k in b.keys() { keys.insert(k.as_str()); }

            for key in keys {
                // Do not deep-diff $params, it is manually tracked via bumps on evaluations.
                if pointer.is_empty() && key == "$params" {
                    continue;
                }
                
                let a_val = a.get(key).unwrap_or(&Value::Null);
                let b_val = b.get(key).unwrap_or(&Value::Null);
                
                let escaped_key = key.replace('~', "~0").replace('/', "~1");
                let next_path = format!("{}/{}", pointer, escaped_key);
                diff_and_update_versions_internal(tracker, &next_path, a_val, b_val);
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            let max_len = a.len().max(b.len());
            for i in 0..max_len {
                let a_val = a.get(i).unwrap_or(&Value::Null);
                let b_val = b.get(i).unwrap_or(&Value::Null);
                let next_path = format!("{}/{}", pointer, i);
                diff_and_update_versions_internal(tracker, &next_path, a_val, b_val);
            }
        }
        (old_leaf, new_leaf) => {
            if old_leaf != new_leaf {
                tracker.bump(pointer);
            }
        }
    }
}
