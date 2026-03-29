# Smart Evaluation Cache Refinement

## The Core Scenario to Solve

**Run 1 → Main form eval:**
- Table `WOP_RIDERS` evaluates with `$datas` injecting `riders[1]` data
- Result cached into `self.entries["WOP_RIDERS"]` with dep version snapshot from main `data_versions`

**Run 2 → Subform eval (riders.1):**
- The subform has `riders` data already merged in externally  
- Table `WOP_RIDERS` evaluates with the identical effective data
- **Expected: cache hit from Run 1** — no re-evaluation

## Root Cause of Current Plan's Gap

In the current `check_cache` implementation, when `active_item_index = Some(1)`:
```rust
let cache = self.subform_caches.get(&idx)?;  // returns None early if empty!
```

This uses `?` to return early when the subform sub-cache doesn't exist yet — it **never falls through to check `self.entries`**. Run 2 always misses even though Run 1 stored the result globally.

## Fixed Design

### Two-Tier Lookup in `check_cache`

```rust
pub fn check_cache(&self, eval_key: &str, deps: &IndexSet<String>) -> Option<Value> {
    // Tier 1: check subform-specific entries (item-scoped)
    if let Some(idx) = self.active_item_index {
        if let Some(cache) = self.subform_caches.get(&idx) {
            if let Some(hit) = self.validate_entry(eval_key, deps, &cache.entries, &cache.data_versions) {
                return Some(hit);
            }
        }
        // Tier 2: fall back to global entries, validate against item data_versions
        // This enables Run 1's main-form result to be a cache hit in Run 2!
        let item_data_versions = self.subform_caches
            .get(&self.active_item_index.unwrap())
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
        let data_dep_path = normalize_path(dep);
        let current_ver = if data_dep_path.starts_with("/$params") {
            self.params_versions.get(&data_dep_path)
        } else {
            data_versions.get(&data_dep_path)  // item-scoped versions
        };
        let &cached_ver = entry.dep_versions.get(&data_dep_path)?;
        if current_ver != cached_ver {
            return None;  // dep changed → miss
        }
    }
    Some(entry.result.clone())
}
```

### Why This Works for the Scenario

**Run 1:** Table stored in `self.entries["WOP_RIDERS"]` with dep snapshot `{"/riders/1": v1, "/$params/...": v2}`.

**Run 2 (subform, riders.1):**
- Tier 1: `subform_caches.get(&1)` → empty → skip
- Tier 2: checks `self.entries["WOP_RIDERS"]`
- Validates deps against `subform_caches[1].data_versions` (which was diffed against the injected riders.1 data)
- Since the riders.1 data is the same and versions match → **Cache HIT** ✓

### `store_cache` Behaviour (Unchanged)

When a table **misses** both tiers and is freshly evaluated in a subform context, the result is stored into `subform_caches[idx].entries`. This isolates different riders from each other:
- `riders.0` and `riders.1` with different data store separate entries — no stale cross-contamination.
- On repeat evaluations for the same `idx` with the same data → subform cache hits directly via Tier 1.

### Cache Key Strategy (EvalCache fields)
```rust
pub struct EvalCache {
    pub data_versions: VersionTracker,    // global (main form) versions
    pub params_versions: VersionTracker,  // shared across all subforms

    pub entries: HashMap<String, CacheEntry>,  // main form + shared table results

    pub active_item_index: Option<usize>,                  // current iteration index
    pub subform_caches: HashMap<usize, SubformItemCache>,  // per-idx isolated state
    // subform key remains `usize` — auto-upserted, path tracking is in the caller
    ...
}
```

### Cache Swap Strategy in `dependents.rs`
```rust
let main_cache = std::mem::take(&mut self.eval_cache);
std::mem::swap(&mut subform.eval_cache, &mut main_cache);
subform.eval_cache.set_active_item(idx); // sets active_item_index = Some(idx)

subform.evaluate_dependents(...);

std::mem::swap(&mut subform.eval_cache, &mut main_cache);
self.eval_cache = main_cache;
self.eval_cache.clear_active_item();
```

## Verification Plan
1. `cargo test` — all existing tests must pass.
2. Enable `JSONEVAL_DEBUG_CACHE=1` and trace `test_zpp_multi_eval` — Run 2/5 table evaluations must show `Cache HIT #/$params/references/WOP_RIDERS`.
3. Verify correctness: different rider indices must NOT share cache entries that depend on item-specific data.
