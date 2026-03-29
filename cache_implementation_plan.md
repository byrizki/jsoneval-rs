# Evaluation Cache with Token-Version Mechanism

Add a versioned evaluation cache to `jsoneval-rs` that avoids redundant re-evaluations when data/context hasn't changed. Cache uses pointer-path version tokens for form data changes, eval-result version bumps for `$params` fields, and handles subform item isolation correctly.

---

## Resolved Design Decisions

| # | Question | Answer |
|---|---|---|
| 1 | `$params` versioning | **Not deep-diffed.** `$params` is never mutated by user input. Track version by bumping whenever a `$params`-rooted eval key's result changes. |
| 2 | Table cell eval caching | **Excluded.** Cache operates at the per-eval-run level. Table rows/cells run inside `evaluate_table` and their final aggregated result is whatever sits in `evaluated_schema` after the run — no sub-cell cache needed. |
| 3 | Subform cache ownership | **Item-scoped data versions, value-synced `$params` versions.** Evaluation is fully sequential (gated by `eval_lock`), so no Arc or Mutex is needed. Parent copies `params_versions` into each subform by value before the subform loop. Non-`$params` data versions are keyed by array-item index, not by subform instance. |

---

## Core Design

### Version Tracker

Tracks a `u64` version counter per pointer path. Only two operations:

```
get(path) → u64        // current version (0 = never set)
bump(path)             // increment version at path
```

**For form data** (rule 1): When `replace_data_and_context` is called with new data, walk both old and new values recursively. For each **leaf** that changes, `bump(pointer_path)`. This gives fine-grained tracking at each nested key.

**For `$params`** (rule 1 clarification): Do NOT deep-diff `$params` internals. Instead, whenever a `$params`-rooted eval result is written (e.g. `#/$params/constants/DEATH_SA/value`), `bump("/$params/constants/DEATH_SA/value")`. This version only increments when the eval actually produces a new value — a pure output-driven signal.

### Cache Entry

```rust
pub struct CacheEntry {
    /// Snapshot of dep-path → version at cache-store time
    pub dep_versions: HashMap<String, u64>,
    /// The cached evaluation result
    pub result: Value,
}
```

### EvalCache

```rust
pub struct EvalCache {
    /// Form data versions — per-path, bumped on leaf changes
    pub data_versions: VersionTracker,
    /// $params eval result versions.
    /// Plain value (no Arc/Mutex) — evaluation is sequential, no concurrent access.
    /// Parent syncs this into subforms by clone before the subform loop.
    pub params_versions: VersionTracker,
    /// eval_key → cached result (for main form and $params evals)
    pub entries: HashMap<String, CacheEntry>,
    /// Subform item-scoped data version: item_index → VersionTracker
    /// Each array item has independent form-data version tracking
    pub subform_item_versions: HashMap<usize, VersionTracker>,
    /// Subform item-scoped cache: (eval_key, item_index) → CacheEntry
    pub subform_entries: HashMap<(String, usize), CacheEntry>,
}
```

---

## Cache Hit / Miss Logic (rule 2)

For each `eval_key` before running `engine.run(...)`:

```
deps = self.dependencies.get(eval_key)  // Set<pointer_path>

if deps is empty or None:
  // No $ref/var dependencies
  current = self.evaluated_schema.pointer(eval_key)
  if current is a primitive (not $evaluation object):
    → CACHE HIT: return current schema value as-is
  else:
    → CACHE MISS: must evaluate

else:
  if let Some(entry) = eval_cache.entries.get(eval_key):
    for (dep_path, cached_ver) in &entry.dep_versions:
      current_ver = if dep_path starts with "/$params":
        params_versions.get(dep_path)
      else:
        data_versions.get(dep_path)   // or subform_item_versions[idx].get(dep_path)
      if current_ver != cached_ver:
        → CACHE MISS: dep changed, re-evaluate
    → CACHE HIT: all deps unchanged, return entry.result

  else:
    → CACHE MISS: no entry yet
```

**Inline table column ref/var**: Ignore for cache check. Table evals (`is_table = true`) bypass the cache check entirely — they always run through `evaluate_table`.

> [!WARNING]
> **Stale-cache hazard — tables with per-item data dependencies in subforms:**
> A subform's `$table` may depend on per-item form fields (e.g. `riders.base`). When iterating over items,
> the table always re-runs (bypasses cache check), but any **downstream** eval key that depends on the table's
> output will read the version for that table path and find it unchanged — triggering a false CACHE HIT.
>
> **Fix (version-bump-on-write principle):** After every `eval_data.set(pointer_path, val)` call — whether
> from a table result, a regular eval, or a dependents write — compare old vs new value and if different,
> `bump(pointer_path)` in the appropriate version tracker (`data_versions` for main form,
> `subform_item_versions[idx]` for subform items). This ensures downstream evals that read the table output
> path will always see a version change and correctly re-evaluate.

### Storing a cache entry (on miss)

After a successful evaluation:

```
dep_versions = {}
for dep_path in deps:
  dep_versions[dep_path] = current_version(dep_path)

eval_cache.entries.insert(eval_key, CacheEntry { dep_versions, result: cleaned_val })

// If result changed AND eval_key starts with "/$params":
  eval_cache.params_versions.bump(eval_key)
```

---

---

## Deep-Diff for Form Data (rule 1 detail)

Called once at the start of `evaluate_internal` after `replace_data_and_context`:

```rust
fn diff_and_update_versions(tracker: &mut VersionTracker, pointer: &str, old: &Value, new: &Value) {
    match (old, new) {
        (Object(a), Object(b)) => {
            for key in union(a.keys(), b.keys()) {
                diff_and_update_versions(tracker, &format!("{pointer}/{key}"), 
                    a.get(key).unwrap_or(&Null), b.get(key).unwrap_or(&Null));
            }
        }
        (Array(a), Array(b)) => {
            for i in 0..max(a.len(), b.len()) {
                diff_and_update_versions(tracker, &format!("{pointer}/{i}"),
                    a.get(i).unwrap_or(&Null), b.get(i).unwrap_or(&Null));
            }
        }
        (old_leaf, new_leaf) => {
            if old_leaf != new_leaf {
                tracker.bump(pointer);
            }
        }
    }
}
```

`$params` fields at the root of `eval_data` are **skipped** during this diff — they are handled separately via `params_versions` bump-on-write.

`$context` IS walked recursively by this diff (context can change per-evaluate call and has real dependencies).

---

## Subform Cache Isolation (rule 3)

### Setup

When `JSONEval` is created (or subforms are built in `parse_schema::legacy`):
- Parent creates `EvalCache` with a plain `params_versions: VersionTracker::new()`
- Each subform's `EvalCache` also holds a plain `params_versions`

No Arc or Mutex needed — evaluation is sequential. The parent **copies** its `params_versions` into each subform **once before the subform loop begins** on every evaluation pass.

### Per-item data version space

Subform loop in `evaluate_dependents` / `evaluate_internal`:

```
// Sync parent's current params_versions into all subforms ONCE before loop
for subform in self.subforms.values_mut():
  subform.eval_cache.params_versions = self.eval_cache.params_versions.clone();

for (idx, item) in items.iter().enumerate():
  // Ensure item-scoped version tracker exists
  subform.eval_cache.ensure_item_versions(idx);
  
  // Diff new item data against stored item snapshot
  let old_item = subform.eval_cache.get_item_snapshot(idx).unwrap_or(&Null);
  diff_and_update_versions(
    subform.eval_cache.subform_item_versions.entry(idx).or_default(),
    "", old_item, item
  );
  subform.eval_cache.set_item_snapshot(idx, item.clone());
  
  // When checking cache inside subform evaluation, pass current idx
  // so check_cache uses subform_item_versions[idx] for non-$params deps
```

### Cache lookup in subform

```
fn check_cache_for_item(eval_cache, eval_key, deps, item_idx):
  if let Some(entry) = eval_cache.subform_entries.get(&(eval_key, item_idx)):
    for (dep_path, cached_ver) in &entry.dep_versions:
      current_ver = if starts_with("/$params"):
        eval_cache.params_versions.get(dep_path)  // plain field, no lock
      else:
        eval_cache.subform_item_versions[item_idx].get(dep_path)
      if current_ver != cached_ver: → CACHE MISS
    → CACHE HIT
  → CACHE MISS
```

This means:
- Two different items (`idx=0`, `idx=1`) never share non-`$params` cache entries
- The same `idx=0` item on a second `evaluate` call reuses its cached result if data hasn't changed

---

## Files to Create / Modify

### [NEW] `src/jsoneval/eval_cache.rs`

Contains:
- `VersionTracker` struct + `new()`, `get()`, `bump()`
- `diff_and_update_versions()` free fn (recursive leaf differ)
- `CacheEntry` struct
- `EvalCache` struct with all fields above
- `EvalCache::check_cache()` — returns `Option<Value>` (None = miss)
- `EvalCache::store_cache()` — stores result + dep snapshot
- `EvalCache::ensure_item_versions(idx)` — lazily init per-item tracker
- `EvalCache::bump_params_eval(eval_key)` — bumps params_versions for a key

---

### [MODIFY] `src/jsoneval/mod.rs`

```rust
pub mod eval_cache;
```

Add to `JSONEval` struct:
```rust
pub eval_cache: EvalCache,
```

Update `Clone` impl:
- Clone `eval_cache` — `data_versions` and `subform_*` fresh-cloned, `params_versions` Arc-cloned (shared)

---

### [MODIFY] `src/jsoneval/core.rs`

In `new()`, `new_from_msgpack()`, `with_parsed_schema()`:
```rust
eval_cache: EvalCache::new(),
```

Subforms are created with their own `EvalCache::new()` — no Arc sharing at construction time.
The parent syncs `params_versions` into subforms at evaluation time (before each subform loop), not at creation.

In `reload_schema*` methods: reset `eval_cache` to `EvalCache::new()` entirely.

---

### [MODIFY] `src/jsoneval/evaluate.rs`

**`evaluate_internal_with_new_data`**: After `replace_data_and_context`, diff old vs new to update `data_versions`:
```rust
let old_data = self.eval_data.snapshot_data();
self.eval_data.replace_data_and_context(data.clone(), context.clone());
// Diff form data (skip $params, skip $context handled separately)
diff_and_update_versions(&mut self.eval_cache.data_versions, "", &old_data, &data);
diff_and_update_versions(&mut self.eval_cache.data_versions, "/$context", &old_context, &context);
```

**`evaluate_internal` — value_evaluations loop**:
```rust
// Before engine.run:
if let Some(cached) = self.eval_cache.check_cache(eval_key, &self.dependencies, None) {
    // Apply cached value to schema
    if let Some(v) = self.evaluated_schema.pointer_mut(&pointer_path) { *v = cached; }
    continue;
}
// ... run engine ...
self.eval_cache.store_cache(eval_key, &self.dependencies, cleaned_val.clone(), None);
// bump $params versions if applicable
if eval_key.contains("/$params/") {
    self.eval_cache.bump_params_eval(&pointer_path);
}
```

**`evaluate_internal` — batch loop**: Same pattern wrapping the `engine.run` call.

**`evaluate_others` — rules+others loop**: Same pattern.

**Table eval keys**: Skip cache check — always run `evaluate_table`. After writing the result, bump version if value changed:
```rust
if is_table {
    let old_val = self.eval_data.get(&pointer_path).cloned();
    if let Ok(rows) = table_evaluate::evaluate_table(self, eval_key, &eval_data_snapshot, token) {
        let value = Value::Array(rows);
        // Version-bump-on-write: bump version if table result changed
        if Some(&value) != old_val.as_ref() {
            self.eval_cache.data_versions.bump(&pointer_path); // or subform_item_versions[idx]
        }
        self.eval_data.set(&pointer_path, value.clone());
        // ... update evaluated_schema ...
    }
    continue; // skip normal eval path
}
```

**General version-bump-on-write rule**: Every `eval_data.set(path, val)` during `evaluate_internal` (table result, regular eval result, `$params` eval result) must check if the value changed and `bump` the path. This is the single invariant that keeps the entire cache coherent — including across subform iterations where tables share the same schema but process different item data.

---

### [MODIFY] `src/jsoneval/eval_data.rs`

Expose old data before replacement:
```rust
/// Returns a clone of the current data (for version diffing before replacement)
pub fn snapshot_data_clone(&self) -> Value {
    (*self.data).clone()
}
```

(We already have `snapshot_data() -> Arc<Value>` but need the actual old value for diff.)

---

### [MODIFY] `src/jsoneval/dependents.rs`

After writing a new value via `eval_data.set(path, val)` in `process_dependents_queue`:
```rust
// Bump version for the written path so subsequent cache checks see the change
eval_cache.data_versions.bump(path);
```

The `process_dependents_queue` signature will need `eval_cache: &mut EvalCache` added.

For subform iteration loop (`evaluate_dependents` subform section):
```rust
let idx = ...;
subform.eval_cache.ensure_item_versions(idx);
// diff item data against last-seen snapshot for this idx
let old_item = subform.eval_cache.get_item_snapshot(idx).cloned().unwrap_or(Value::Null);
diff_and_update_versions(&mut subform.eval_cache.subform_item_versions.get_mut(&idx).unwrap(), "", &old_item, item);
subform.eval_cache.set_item_snapshot(idx, item.clone());
```

---

## Data Flow Summary

```
User calls evaluate(new_data)
  → diff old_data vs new_data → bump data_versions for changed leaf paths
  → for each eval_key in sorted batches:
      if is_table:
        run evaluate_table
        if result changed → bump data_versions[pointer_path]    ← stale-cache fix
        write to eval_data + evaluated_schema
      else if check_cache(eval_key, deps) == HIT:
        apply cached value to evaluated_schema
      else:
        engine.run() → if result changed → bump data_versions[pointer_path]
        store_cache()
        if eval_key is $params-rooted → bump params_versions[pointer_path]

User calls evaluate_dependents(changed_paths, new_data)
  → same data version diff as above
  → process_dependents_queue:
      on each eval_data.set(path, val): if val changed → bump data_versions[path]
  → if re_evaluate → evaluate_internal (cached paths now hit on second pass)

Subform loop (idx=0,1,...N):
  → sync parent.params_versions → subform.eval_cache.params_versions (clone)
  → diff item[idx] vs last snapshot[idx] → bump subform_item_versions[idx]
  → for each eval_key in subform batches:
      if is_table:
        run evaluate_table
        if result changed → bump subform_item_versions[idx][pointer_path]   ← stale-cache fix
      else if check_cache(eval_key, deps, idx) == HIT:
        apply cached value
      else:
        engine.run() → if result changed → bump subform_item_versions[idx][pointer_path]
        store_cache(eval_key, idx)
        $params eval → bump params_versions[pointer_path]
```

---

## Verification Plan

### Automated Tests
```
cargo test  -- all existing tests must pass
```

New tests to add in `tests/`:
- `test_cache_hit_same_data` — call `evaluate` twice with identical data; verify engine `run` count is 0 on second call
- `test_cache_miss_on_nested_change` — change a single nested leaf; verify only evals with that dep re-run
- `test_params_version_bumped_on_eval_change` — verify `params_versions` increments only when a `$params` eval result changes
- `test_subform_item_cache_isolation` — item[0] change doesn't invalidate item[1] cache entries
- `test_subform_params_shared` — all subform instances receive the same `params_versions` snapshot before their loop
- `test_table_dep_on_item_field_no_stale_cache` — subform `$table` that depends on per-item data (e.g. `riders.base`) must NOT return stale results across iterations

### Performance Validation
- Benchmark `evaluate_dependents` on ZPP/ZCC schema before and after
- Expect significant cache hit ratio on repeat evaluations for fields with unchanged `$params` dependencies
