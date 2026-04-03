use indexmap::IndexSet;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

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

    /// Merge version counters from `other`, taking the **maximum** for each path.
    /// Using max (not insert) ensures that if this tracker already saw a higher version
    /// for a path (e.g., from a previous subform evaluation round), it is never downgraded.
    pub fn merge_from(&mut self, other: &VersionTracker) {
        for (k, v) in &other.versions {
            let current = self.versions.get(k).copied().unwrap_or(0);
            self.versions.insert(k.clone(), current.max(*v));
        }
    }

    /// Merge only `/$params`-prefixed version counters from `other` (max strategy).
    /// Used when giving a per-item tracker the latest schema-level param versions
    /// without absorbing data-path bumps that belong to other items.
    pub fn merge_from_params(&mut self, other: &VersionTracker) {
        for (k, v) in &other.versions {
            if k.starts_with("/$params") {
                let current = self.versions.get(k).copied().unwrap_or(0);
                self.versions.insert(k.clone(), current.max(*v));
            }
        }
    }

    /// Returns true if any tracked path with the given prefix has been bumped (version > 0).
    /// Used to gate table re-evaluation when item fields change without the item being new.
    pub fn any_bumped_with_prefix(&self, prefix: &str) -> bool {
        self.versions
            .iter()
            .any(|(k, &v)| k.starts_with(prefix) && v > 0)
    }

    /// Returns true if any path with the given prefix has a **higher** version than in `baseline`.
    /// Unlike `any_bumped_with_prefix`, this detects only brand-new bumps from a specific diff
    /// pass, ignoring historical bumps that were already present in the baseline.
    pub fn any_newly_bumped_with_prefix(&self, prefix: &str, baseline: &VersionTracker) -> bool {
        self.versions
            .iter()
            .any(|(k, &v)| k.starts_with(prefix) && v > baseline.get(k))
    }

    /// Returns an iterator over all (path, version) pairs, for targeted bump enumeration.
    pub fn versions(&self) -> impl Iterator<Item = (&str, &u64)> {
        self.versions.iter().map(|(k, v)| (k.as_str(), v))
    }
}

/// A cached evaluation result with the specific dependency versions it was evaluated against
#[derive(Clone)]
pub struct CacheEntry {
    pub dep_versions: HashMap<String, u64>,
    pub result: Value,
    /// The `active_item_index` this entry was computed under.
    /// `None` = computed during main-form evaluation (safe to reuse across all items
    /// provided the dep versions match). `Some(idx)` = computed for a specific item;
    /// Tier-2 reuse is restricted to entries whose deps are entirely `$params`-scoped.
    pub computed_for_item: Option<usize>,
}

/// Independent cache state for a single item in a subform array
#[derive(Default, Clone)]
pub struct SubformItemCache {
    pub data_versions: VersionTracker,
    pub entries: HashMap<String, CacheEntry>,
    pub item_snapshot: Value,
    /// Per-item snapshot of the evaluated schema captured after each evaluate_subform_item.
    /// Allows get_evaluated_schema_subform to return the correct per-item values without
    /// re-running the full evaluation pipeline in a shared subform context.
    pub evaluated_schema: Option<Value>,
}

impl SubformItemCache {
    pub fn new() -> Self {
        Self {
            data_versions: VersionTracker::new(),
            entries: HashMap::new(),
            item_snapshot: Value::Null,
            evaluated_schema: None,
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

    /// Snapshot of the last fully-diffed main-form data payload.
    /// Stored after each successful `evaluate_internal_with_new_data` call so the next
    /// invocation can avoid an extra `snapshot_data_clone()` when computing the diff.
    pub main_form_snapshot: Option<Value>,
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
            main_form_snapshot: None,
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
        self.main_form_snapshot = None;
    }

    /// Remove item caches for indices >= `current_count`.
    /// Call this whenever the subform array length is known to have shrunk so that
    /// stale per-item version trackers and cached entries do not linger in memory.
    pub fn prune_subform_caches(&mut self, current_count: usize) {
        self.subform_caches.retain(|&idx, _| idx < current_count);
    }

    /// Invalidate all `$params`-scoped table cache entries for a specific item.
    ///
    /// Called when a brand-new subform item is introduced so that `$params` tables
    /// that aggregate array data (e.g. WOP_RIDERS) are forced to recompute instead
    /// of returning stale results cached from a prior main-form evaluation that ran
    /// when the item was absent (and thus saw zero/null for that item's values).
    pub fn invalidate_params_tables_for_item(&mut self, idx: usize, table_keys: &[String]) {
        // Bump params_versions so T2 global entries for these tables are stale.
        for key in table_keys {
            let data_path = crate::jsoneval::path_utils::normalize_to_json_pointer(key)
                .replace("/properties/", "/");
            let data_path = data_path.trim_start_matches('#');
            let data_path = if data_path.starts_with('/') {
                data_path.to_string()
            } else {
                format!("/{}", data_path)
            };
            self.params_versions.bump(&data_path);
            self.eval_generation += 1;
        }

        // Evict matching T1 (item-level) entries so they are not reused.
        if let Some(item_cache) = self.subform_caches.get_mut(&idx) {
            for key in table_keys {
                item_cache.entries.remove(key);
            }
        }
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
        self.subform_caches
            .entry(idx)
            .or_insert_with(SubformItemCache::new);
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
            self.subform_caches
                .get(&idx)
                .map(|c| c.item_snapshot.clone())
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        }
    }

    pub fn diff_active_item(
        &mut self,
        field_key: &str,
        old_sub_data: &Value,
        new_sub_data: &Value,
    ) {
        if let Some(idx) = self.active_item_index {
            self.ensure_active_item_cache(idx);
            let sub_cache = self.subform_caches.get_mut(&idx).unwrap();

            // Diff ONLY the localized item part, skipping the massive parent tree
            let empty = Value::Null;
            let old_item = old_sub_data.get(field_key).unwrap_or(&empty);
            let new_item = new_sub_data.get(field_key).unwrap_or(&empty);

            diff_and_update_versions(
                &mut sub_cache.data_versions,
                &format!("/{}", field_key),
                old_item,
                new_item,
            );
            sub_cache.item_snapshot = new_sub_data.clone();
        }
    }

    pub fn bump_data_version(&mut self, data_path: &str) {
        // Always signal that something changed so the parent's needs_full_evaluation()
        // returns true even when the bump was item-scoped.
        self.eval_generation += 1;
        if let Some(idx) = self.active_item_index {
            if let Some(cache) = self.subform_caches.get_mut(&idx) {
                cache.data_versions.bump(data_path);
            }
        } else {
            self.data_versions.bump(data_path);
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
            // Tier 1: item-specific entries (always safe to reuse for the same index)
            if let Some(cache) = self.subform_caches.get(&idx) {
                if let Some(hit) =
                    self.validate_entry(eval_key, deps, &cache.entries, &cache.data_versions)
                {
                    if crate::utils::is_debug_cache_enabled() {
                        println!("Cache HIT [T1 idx={}] {}", idx, eval_key);
                    }
                    return Some(hit);
                }
            }

            // Tier 2: global entries (may have been stored by main-form Run 1).
            // Only reuse if the entry is index-safe:
            //   (a) computed with no active item (main-form result), OR
            //   (b) computed for the same item index, OR
            //   (c) all deps are $params-scoped (truly index-independent)
            let item_data_versions = self
                .subform_caches
                .get(&idx)
                .map(|c| &c.data_versions)
                .unwrap_or(&self.data_versions);

            if let Some(entry) = self.entries.get(eval_key) {
                let index_safe = match entry.computed_for_item {
                    // Main-form entry (no active item when stored): only safe if ALL its deps
                    // are $params-scoped. Non-$params deps (like /riders/prem_pay_period) mean
                    // the formula result is rider-specific — using it for a different rider via
                    // the batch fast path would corrupt eval_data and poison subsequent formulas.
                    None => entry.dep_versions.keys().all(|p| p.starts_with("/$params")),
                    Some(stored_idx) if stored_idx == idx => true,
                    _ => entry.dep_versions.keys().all(|p| p.starts_with("/$params")),
                };
                if index_safe {
                    let result =
                        self.validate_entry(eval_key, deps, &self.entries, item_data_versions);
                    if result.is_some() {
                        if crate::utils::is_debug_cache_enabled() {
                            println!(
                                "Cache HIT [T2 idx={} for={:?}] {}",
                                idx, entry.computed_for_item, eval_key
                            );
                        }
                    }
                    return result;
                }
            }

            None
        } else {
            self.validate_entry(eval_key, deps, &self.entries, &self.data_versions)
        }
    }

    /// Specialized cache check for `$params`-scoped table evaluations.
    ///
    /// Tables in `$params/references/` aggregate cross-item data and produce a single result
    /// that is independent of which subform item is currently active. The standard `check_cache`
    /// blocks T2 reuse for entries whose deps include non-`$params` paths (e.g. `/riders/...`),
    /// because scalar formula results are item-specific. But table results are global: the same
    /// 734-row array is correct for rider 0, rider 1, and rider 2 alike.
    ///
    /// This method validates the global entry directly — using `item_data_versions` for
    /// non-`$params` deps — without the `index_safe` gate, allowing the expensive table forward/
    /// backward pass to be skipped when inputs have not changed.
    pub fn check_table_cache(&self, eval_key: &str, deps: &IndexSet<String>) -> Option<Value> {
        if let Some(idx) = self.active_item_index {
            // Tier 1: item-scoped entries first (unlikely for $params tables but check anyway)
            if let Some(cache) = self.subform_caches.get(&idx) {
                if let Some(hit) =
                    self.validate_entry(eval_key, deps, &cache.entries, &cache.data_versions)
                {
                    if crate::utils::is_debug_cache_enabled() {
                        println!("Cache HIT [T1 table idx={}] {}", idx, eval_key);
                    }
                    return Some(hit);
                }
            }

            // Tier 2: validate the global entry against the parent main-form tracker.
            //
            // Global $params table entries are stored using `self.data_versions` (store_cache
            // with no active item). When a rider field (e.g. `riders.sa`) changes via
            // `with_item_cache_swap`, the newly-bumped paths are propagated into
            // `parent_cache.data_versions` before the swap. This ensures the T2 check
            // here correctly sees the change without needing MaxVersionTracker, which
            // would pick up historical per-rider bumps and cause false misses.
            let result = self.validate_entry(eval_key, deps, &self.entries, &self.data_versions);
            if result.is_some() {
                if crate::utils::is_debug_cache_enabled() {
                    println!("Cache HIT [T2 table idx={}] {}", idx, eval_key);
                }
            }
            result
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
            let data_dep_path = crate::jsoneval::path_utils::normalize_to_json_pointer(dep)
                .replace("/properties/", "/");

            let current_ver = if data_dep_path.starts_with("/$params") {
                self.params_versions.get(&data_dep_path)
            } else {
                data_versions.get(&data_dep_path)
            };

            if let Some(&cached_ver) = entry.dep_versions.get(&data_dep_path) {
                if current_ver != cached_ver {
                    if crate::utils::is_debug_cache_enabled() {
                        println!(
                            "Cache MISS {}: dep {} changed ({} -> {})",
                            eval_key, data_dep_path, cached_ver, current_ver
                        );
                    }
                    return None;
                }
            } else {
                if crate::utils::is_debug_cache_enabled() {
                    println!(
                        "Cache MISS {}: dep {} missing from cache entry",
                        eval_key, data_dep_path
                    );
                }
                return None;
            }
        }
        if crate::utils::is_debug_cache_enabled() {
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
                let data_dep_path = crate::jsoneval::path_utils::normalize_to_json_pointer(dep)
                    .replace("/properties/", "/");
                let ver = if data_dep_path.starts_with("/$params") {
                    self.params_versions.get(&data_dep_path)
                } else {
                    data_versions.get(&data_dep_path)
                };
                dep_versions.insert(data_dep_path, ver);
            }
        }

        // Phase 2: insert into the correct tier, tagging with the current item index.
        let computed_for_item = self.active_item_index;

        // For $params-scoped entries, only bump params_versions when the result value
        // actually changed relative to the canonical cached entry.
        //
        // For T1 stores (active_item set), we compare against T2 (global) first.
        // T2 is the authoritative reference: if T2 already holds the same value,
        // params_versions was already bumped for it — bumping again per-rider causes
        // an O(riders × $params_formulas) version explosion that makes every downstream
        // formula (TOTAL_WOP_SA, WOP_MULTIPLIER, COMMISSION_FACTOR…) miss on each rider.
        if eval_key.starts_with("#/$params") {
            let existing_result: Option<&Value> = if let Some(idx) = self.active_item_index {
                // Check T2 (global) first — if T2 has same value, no need to bump again.
                self.entries.get(eval_key).map(|e| &e.result).or_else(|| {
                    self.subform_caches
                        .get(&idx)
                        .and_then(|c| c.entries.get(eval_key))
                        .map(|e| &e.result)
                })
            } else {
                self.entries.get(eval_key).map(|e| &e.result)
            };

            let value_changed = existing_result.map_or(true, |r| r != &result);

            if value_changed {
                let data_path = crate::jsoneval::path_utils::normalize_to_json_pointer(eval_key)
                    .replace("/properties/", "/");
                let data_path = data_path.trim_start_matches('#').to_string();
                let data_path = if data_path.starts_with('/') {
                    data_path
                } else {
                    format!("/{}", data_path)
                };

                // Bump the explicit path and its table-level parent.
                // Stop at slash_count < 3 — never bump /$params/others or /$params itself.
                let mut current_path = data_path.as_str();
                let mut slash_count = current_path.matches('/').count();

                while slash_count >= 3 {
                    self.params_versions.bump(current_path);
                    if let Some(last_slash) = current_path.rfind('/') {
                        current_path = &current_path[..last_slash];
                        slash_count -= 1;
                    } else {
                        break;
                    }
                }

                self.eval_generation += 1;
            }
        }

        let entry = CacheEntry {
            dep_versions,
            result,
            computed_for_item,
        };

        if let Some(idx) = self.active_item_index {
            // Store item-scoped: isolates per-rider entries so riders with different data don't collide
            self.subform_caches
                .get_mut(&idx)
                .unwrap()
                .entries
                .insert(eval_key.to_string(), entry.clone());

            // For $params-scoped formulas, also promote to T2 (global entries).
            // $params formulas are index-independent: all riders produce the same result.
            // Without T2 promotion, each rider's first store compares against stale/missing T2
            // and sees the value as "new" → bumps params_versions → O(riders) cascade.
            // With T2 promotion, rider 1 finds rider 0's result in T2 → no bump → no cascade.
            if eval_key.starts_with("#/$params") {
                self.entries.insert(eval_key.to_string(), entry);
            }
        } else {
            self.entries.insert(eval_key.to_string(), entry);
        }
    }
}




/// Recursive helper to walk JSON structures and bump specific leaf versions where they differ
pub(crate) fn diff_and_update_versions(
    tracker: &mut VersionTracker,
    pointer: &str,
    old: &Value,
    new: &Value,
) {
    if pointer.is_empty() {
        diff_and_update_versions_internal(tracker, "", old, new);
    } else {
        diff_and_update_versions_internal(tracker, pointer, old, new);
    }
}

fn diff_and_update_versions_internal(
    tracker: &mut VersionTracker,
    pointer: &str,
    old: &Value,
    new: &Value,
) {
    if old == new {
        return;
    }

    match (old, new) {
        (Value::Object(a), Value::Object(b)) => {
            let mut keys = HashSet::new();
            for k in a.keys() {
                keys.insert(k.as_str());
            }
            for k in b.keys() {
                keys.insert(k.as_str());
            }

            for key in keys {
                // Do not deep-diff $params at any nesting level — it is manually tracked
                // via bump_params_version on evaluations. Skipping at root-only was insufficient
                // when item data is diffed via a non-empty pointer prefix.
                if key == "$params" {
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
        (old_val, new_val) => {
            if old_val != new_val {
                tracker.bump(pointer);

                // If either side contains nested structures (e.g. Object replaced by Null, or vice versa)
                // we must recursively bump all paths inside them so targeted cache entries invalidate.
                if old_val.is_object() || old_val.is_array() {
                    traverse_and_bump(tracker, pointer, old_val);
                }
                if new_val.is_object() || new_val.is_array() {
                    traverse_and_bump(tracker, pointer, new_val);
                }
            }
        }
    }
}

/// Recursively traverses a value and bumps the version for every nested path.
/// Used when a structural type mismatch occurs (e.g., Object -> Null) so that
/// cache entries depending on nested fields are correctly invalidated.
fn traverse_and_bump(tracker: &mut VersionTracker, pointer: &str, val: &Value) {
    match val {
        Value::Object(map) => {
            for (key, v) in map {
                if key == "$params" {
                    continue; // Skip the special top-level params branch if it leaked here
                }
                let escaped_key = key.replace('~', "~0").replace('/', "~1");
                let next_path = format!("{}/{}", pointer, escaped_key);
                tracker.bump(&next_path);
                traverse_and_bump(tracker, &next_path, v);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let next_path = format!("{}/{}", pointer, i);
                tracker.bump(&next_path);
                traverse_and_bump(tracker, &next_path, v);
            }
        }
        _ => {}
    }
}
