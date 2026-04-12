# JSONEval Evaluation Cache — System Reference

This document describes the complete caching architecture of `JSONEval`, including
how versions are tracked, how cache entries are validated, the two-tier lookup
strategy for subforms, and every cache invalidation pathway.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Core Data Structures](#2-core-data-structures)
   - 2.1 [VersionTracker](#21-versiontracker)
   - 2.2 [CacheEntry](#22-cacheentry)
   - 2.3 [SubformItemCache](#23-subformitemcache)
   - 2.4 [EvalCache](#24-evalcache)
3. [Version Tracking Mechanics](#3-version-tracking-mechanics)
   - 3.1 [data_versions vs params_versions](#31-data_versions-vs-params_versions)
   - 3.2 [eval_generation and the generation-based skip](#32-eval_generation-and-the-generation-based-skip)
   - 3.3 [diff_and_update_versions — the diff engine](#33-diff_and_update_versions--the-diff-engine)
4. [Two-Tier Cache Lookup (check_cache)](#4-two-tier-cache-lookup-check_cache)
   - 4.1 [Tier 1 — Item-scoped entries](#41-tier-1--item-scoped-entries)
   - 4.2 [Tier 2 — Global entries](#42-tier-2--global-entries)
   - 4.3 [index_safe guard](#43-index_safe-guard)
5. [Table Cache (check_table_cache)](#5-table-cache-check_table_cache)
6. [Cache Storage (store_cache)](#6-cache-storage-store_cache)
   - 6.1 [Dependency version snapshot](#61-dependency-version-snapshot)
   - 6.2 [params_versions bumping on value change](#62-params_versions-bumping-on-value-change)
   - 6.3 [T2 promotion for $params-scoped tables](#63-t2-promotion-for-params-scoped-tables)
7. [evaluate() — Main Form Evaluation](#7-evaluate--main-form-evaluation)
   - 7.1 [Snapshot and diff](#71-snapshot-and-diff)
   - 7.2 [Generation-based early exit](#72-generation-based-early-exit)
   - 7.3 [Subform baseline pre-seed](#73-subform-baseline-pre-seed)
8. [evaluate_dependents() — Dependency Cascade](#8-evaluate_dependents--dependency-cascade)
   - 8.1 [process_dependents_queue](#81-process_dependents_queue)
   - 8.2 [run_re_evaluate_pass](#82-run_re_evaluate_pass)
   - 8.3 [Readonly pass — subform array scalar merge](#83-readonly-pass--subform-array-scalar-merge)
   - 8.4 [run_subform_pass — T2 invalidation (pass 1)](#84-run_subform_pass--t2-invalidation-pass-1)
   - 8.5 [Second evaluate_internal + subform pass 2](#85-second-evaluate_internal--subform-pass-2)
   - 8.6 [Result deduplication](#86-result-deduplication)
9. [evaluate_subform() — Per-Item Isolated Evaluation](#9-evaluate_subform--per-item-isolated-evaluation)
   - 9.1 [with_item_cache_swap lifecycle](#91-with_item_cache_swap-lifecycle)
   - 9.2 [Item diff and newly-bumped propagation](#92-item-diff-and-newly-bumped-propagation)
   - 9.3 [T1 entry restoration and stale-entry eviction](#93-t1-entry-restoration-and-stale-entry-eviction)
   - 9.4 [Selective $params table invalidation and re-evaluation](#94-selective-params-table-invalidation-and-re-evaluation)
10. [Structural Change Invalidation](#10-structural-change-invalidation)
11. [Cache Invalidation Pathways — Reference Table](#11-cache-invalidation-pathways--reference-table)
12. [Path Formats](#12-path-formats)
13. [Debug Mode](#13-debug-mode)

---

## 1. Overview

The JSONEval caching system eliminates redundant formula re-evaluations by recording,
for every evaluated formula, the exact dependency versions at the time of evaluation.
On the next evaluation pass, the cache entry is only reused if all dependency versions
are still the same.

There are two orthogonal dimensions the cache must manage:

| Dimension | Problem |
|---|---|
| **Data scope** | Subform items have their own localized field values (e.g. `riders[0].sa ≠ riders[1].sa`) — per-item cache isolation is required. |
| **Evaluation scope** | `$params` tables (e.g. `RIDER_ZLOS_TABLE`) are global — the same 734-row array is correct for every rider. Cross-context reuse must be safe. |

The two-tier architecture resolves this tension:
- **Tier 1 (T1)** — item-specific entries in `subform_caches[idx].entries`
- **Tier 2 (T2)** — global entries in `EvalCache.entries`, shareable across items

---

## 2. Core Data Structures

### 2.1 `VersionTracker`

```rust
pub struct VersionTracker {
    versions: HashMap<String, u64>,
}
```

A monotonically increasing counter per JSON data path. Paths that have never been
touched have an implicit version of `0` (returned by `.get()` without a map lookup).

**Key methods:**

| Method | Purpose |
|---|---|
| `bump(path)` | Increment the counter for `path` by 1 |
| `get(path)` | Return the current version (0 if absent) |
| `merge_from(other)` | Take `max(self[k], other[k])` for all paths — never downgrades |
| `merge_from_params(other)` | Same as `merge_from` but only for `/$params`-prefixed keys |
| `any_bumped_with_prefix(prefix)` | True if any path under `prefix` has version > 0 |
| `any_newly_bumped_with_prefix(prefix, baseline)` | True if any path under `prefix` has a version **higher than** `baseline` — detects only the current diff pass |

`merge_from_params` is used when the item cache receives global `$params` version
updates, while data-path bumps from other riders are deliberately NOT inherited to
avoid cross-item contamination.

### 2.2 `CacheEntry`

```rust
pub struct CacheEntry {
    pub dep_versions: HashMap<String, u64>,
    pub result: Value,
    pub computed_for_item: Option<usize>,
}
```

- **`dep_versions`** — snapshot of every dependency's version at evaluation time. The cache is valid only when every dep still matches this snapshot.
- **`result`** — the computed JSON value.
- **`computed_for_item`** — `None` means the entry was computed during main-form evaluation (globally safe for `$params`-only deps). `Some(idx)` means it was computed for a specific subform item.

### 2.3 `SubformItemCache`

```rust
pub struct SubformItemCache {
    pub data_versions: VersionTracker, // Per-item version tracker
    pub entries: HashMap<String, CacheEntry>, // T1 entries
    pub item_snapshot: Value,          // Last known data for this item slot
    pub evaluated_schema: Option<Value>, // Per-item evaluated schema snapshot
}
```

One `SubformItemCache` exists per active subform array index. It is created lazily
on first access and pruned when the array shrinks (`prune_subform_caches`).

`item_snapshot` stores the raw item data from the last evaluation. It is the
baseline used by the next diff — only fields that changed between the snapshot and
the new incoming data cause a version bump.

`evaluated_schema` is written after each `evaluate_subform_item` call, letting
`get_evaluated_schema_subform` return the correct per-item evaluated schema
without re-running the full evaluation pipeline.

### 2.4 `EvalCache`

```rust
pub struct EvalCache {
    pub data_versions: VersionTracker,          // Main-form data versions
    pub params_versions: VersionTracker,        // $params formula output versions
    pub entries: HashMap<String, CacheEntry>,   // T2 global entries

    pub active_item_index: Option<usize>,       // Which item is being evaluated (None = main form)
    pub subform_caches: HashMap<usize, SubformItemCache>, // T1 per-item stores

    pub eval_generation: u64,                   // Bumped by store_cache on $params change
    pub last_evaluated_generation: u64,         // Set by mark_evaluated after full traversal
    pub main_form_snapshot: Option<Value>,      // Previous evaluate() payload for diff reuse
}
```

`EvalCache` is the single cache object owned by a `JSONEval` instance. When a
subform item is being evaluated via `with_item_cache_swap`, the **parent's** entire
`EvalCache` is `mem::take`n and moved into the subform's `eval_cache` field, giving
the subform full access to both T1 and T2 entries. After the subform operation
completes, the cache is swapped back.

---

## 3. Version Tracking Mechanics

### 3.1 `data_versions` vs `params_versions`

Two separate version trackers exist because their invalidation rules differ:

| Tracker | What it tracks | Bumped by |
|---|---|---|
| `data_versions` | Raw user input data fields (e.g. `/illustration/insured/insage`) | `diff_and_update_versions` on data change, `bump_data_version` on computed field write |
| `params_versions` | `$params` formula outputs (e.g. `/$params/references/RIDER_ZLOS_TABLE`) | `store_cache` when a `$params` formula produces a new value, `invalidate_params_tables_for_item`, structural change handlers |

During `validate_entry`, dependencies are dispatched to the correct tracker:

```rust
let current_ver = if data_dep_path.starts_with("/$params") {
    self.params_versions.get(&data_dep_path)
} else {
    data_versions.get(&data_dep_path)
};
```

This means formula A that depends on `$params/references/RIDER_ZLOS_TABLE` will miss
its cache when `RIDER_ZLOS_TABLE`'s version in `params_versions` changes — even if the
raw data that feeds `RIDER_ZLOS_TABLE` was the rider subform's field, not the main form.

### 3.2 `eval_generation` and the generation-based skip

`eval_generation` is a monotonically increasing counter, bumped whenever:
- `store_cache` stores a `$params`-scoped entry whose value changed.
- `invalidate_params_tables_for_item` fires.
- A structural change evicts T2 entries.

`last_evaluated_generation` is set to `eval_generation` at the end of each
successful `evaluate_internal`.

The generation-based skip in `evaluate_internal_with_new_data`:

```rust
// If eval_generation == last_evaluated_generation, no formula is stale.
if paths.is_none() && !self.eval_cache.needs_full_evaluation() {
    self.evaluate_others(paths, token, false);
    return Ok(());
}
```

This saves the full batch traversal when an `evaluate()` call receives unchanged data
and no `$params` outputs changed since the last run — common after `evaluate_subform`
on an unchanged rider.

> **Important**: `diff_and_update_versions` (called during the diff phase of `evaluate()`)
> does NOT bump `eval_generation`. Only writes via `store_cache` do. This is intentional:
> the diff finds which raw fields changed, but until a formula actually reads those fields
> and produces a different output, no `eval_generation` increment is needed.

### 3.3 `diff_and_update_versions` — the diff engine

```rust
pub(crate) fn diff_and_update_versions(
    tracker: &mut VersionTracker,
    pointer: &str,
    old: &Value,
    new: &Value,
)
```

The diff engine walks the old and new JSON trees in parallel and bumps the version
for every path where a scalar value changed. Key rules:

1. **Equal subtrees are skipped entirely** — `if old == new { return; }` prunes huge
   unchanged branches in O(1).
2. **Objects**: union of all keys is diffed recursively, with `$params` always skipped
   (it is managed separately via `bump_params_version`).
3. **Arrays**: elements are compared by index. Missing indices are treated as `Null`.
4. **Structural type change** (e.g. `Object → Null`): the leaf path itself is bumped,
   and `traverse_and_bump` is called on whichever side is structured — bumping every
   nested path so cache entries depending on those fields correctly miss.
5. **Scalars**: if `old_val != new_val`, the path is bumped.

The output of the diff is a set of bumped paths in a `VersionTracker`. These paths
are later consulted by `validate_entry` to determine whether a cache entry is stale.

---

## 4. Two-Tier Cache Lookup (`check_cache`)

Standard formula cache lookup — used for all non-table formulas.

```
check_cache(eval_key, deps)
├── active_item_index set? (subform context)
│   ├── T1: subform_caches[idx].entries + item data_versions → HIT? return
│   ├── T2: self.entries, validated with item_data_versions
│   │      → only if index_safe (see §4.3)
│   └── miss → return None
└── No active item (main form)
    └── self.entries + self.data_versions → HIT? return
```

### 4.1 Tier 1 — Item-scoped entries

When `active_item_index = Some(idx)`, T1 is checked first against the item's own
`data_versions`. This ensures rider-specific formula results (e.g. a formula that
reads `riders.sa`) are only reused for the same rider at the same version.

If a field on rider 0 changed (bumping `/riders/sa` in `subform_caches[0].data_versions`),
T1 for rider 0 correctly misses. T1 for rider 1 is entirely separate and is not affected.

### 4.2 Tier 2 — Global entries

T2 is checked when T1 misses. The T2 entry may have been written by:
- The main-form `evaluate()` pass (no active item)
- A previous `evaluate_subform` for the same or a different item

T2 validation uses the **item's** `data_versions` (not the parent's) for non-`$params`
deps, and `params_versions` (always global) for `$params` deps. This is correct for
most field-level formulas: if rider 1's `sa` bumped but rider 0's `sa` did not, rider
0 can still reuse a T2 entry that depends on `/riders/sa` if the version check passes
against rider 0's own tracker.

### 4.3 `index_safe` guard

T2 reuse is only allowed when the entry is _index-safe_:

```rust
let index_safe = match entry.computed_for_item {
    None => entry.dep_versions.keys().all(|p| p.starts_with("/$params")),
    Some(stored_idx) if stored_idx == idx => true,
    _ => entry.dep_versions.keys().all(|p| p.starts_with("/$params")),
};
```

**Why**: A T2 entry computed for rider 0 that depends on `/riders/sa` embeds the
version of `/riders/sa` from rider 0's tracker. If rider 1 reads this entry,
the version check against rider 1's tracker might pass (both trackers happen to
agree), but the **result** would be wrong because it was computed using rider 0's
`sa` value. The `index_safe` guard prevents this cross-rider contamination by
only allowing T2 entries whose deps are entirely `$params`-scoped, which are
truly index-independent.

---

## 5. Table Cache (`check_table_cache`)

`$params` tables (in `$params/references/`, `$params/others/`) are evaluated
differently from scalar formulas. A RIDER_ZLOS_TABLE result is a 734-row array
that is the **same** regardless of which rider item is currently active — it is a
global aggregate. The `index_safe` guard in `check_cache` would block reuse of any
T2 entry that has non-`$params` deps (like `/riders/sa`), causing expensive table
re-evaluation on every rider.

`check_table_cache` bypasses the `index_safe` gate:

```
check_table_cache(eval_key, deps)
├── active_item_index set? (subform context)
│   ├── T1: item entries + item data_versions → HIT? return (rare for $params tables)
│   └── T2: self.entries validated with PARENT self.data_versions (not item versions)
│          (no index_safe check)
└── No active item (main form)
    └── self.entries + self.data_versions
```

T2 for table entries is validated against `self.data_versions` (the parent main-form
tracker), NOT the item tracker. This is deliberate:

- T2 table entries store dep_versions using the **parent** `data_versions` at storage time
  (see §6.3 on T2 promotion). This ensures consistency at validation time.
- The parent `data_versions` is bumped whenever a field that affects the table changes
  (via the propagation step in `with_item_cache_swap` §9.2 and `run_subform_pass` §8.3).

---

## 6. Cache Storage (`store_cache`)

### 6.1 Dependency version snapshot

When a formula result is stored, all dependency paths are resolved to their current
versions:

```rust
for dep in deps {
    let data_dep_path = normalize_to_json_pointer(dep).replace("/properties/", "/");
    let ver = if data_dep_path.starts_with("/$params") {
        self.params_versions.get(&data_dep_path)
    } else {
        data_versions.get(&data_dep_path)   // item versions when in subform context
    };
    dep_versions.insert(data_dep_path, ver);
}
```

This snapshot is what `validate_entry` compares against on future reads.

### 6.2 `params_versions` bumping on value change

When a `$params`-scoped formula produces a value **different** from what is already
stored in T2, `params_versions` is bumped for the formula's path (and its table-level
parent, stopping at depth < 3):

```
eval_key = "#/$params/references/RIDER_ZLOS_TABLE/850/BENPAY_DEATHSA"
bumps: /$params/references/RIDER_ZLOS_TABLE/850/BENPAY_DEATHSA
       /$params/references/RIDER_ZLOS_TABLE/850
       /$params/references/RIDER_ZLOS_TABLE
```

This cascade ensures formulas that depend on `RIDER_ZLOS_TABLE` (at any depth) will
see a version change and miss their cache.

**Per-rider dedup**: When in subform context (`active_item_index = Some(idx)`), the new
result is compared against T2 (global) first before comparing T1. If T2 already holds
the same value, `params_versions` is NOT bumped again — avoiding an O(riders ×
formulas) version explosion that would cause every downstream formula to miss on each
rider.

### 6.3 T2 promotion for `$params`-scoped tables

When a `$params` table is evaluated in a subform context, the result is stored in
BOTH T1 and T2:

- **T1** uses the item's `data_versions` for non-`$params` dep paths — correct for
  per-item scoping.
- **T2** is promoted with a **rebuilt** dep snapshot using the PARENT `data_versions`
  for non-`$params` paths:

```rust
let t2_dep_versions: HashMap<String, u64> = entry.dep_versions.iter()
    .map(|(path, &item_ver)| {
        let parent_ver = if path.starts_with("/$params") {
            item_ver  // global — same in both trackers
        } else {
            self.data_versions.get(path)  // use PARENT tracker
        };
        (path.clone(), parent_ver)
    })
    .collect();
```

**Why this matters**: If T2 stored the item tracker version for `/riders/sa` (e.g. `1`
for rider 0), but `check_table_cache` validates against `self.data_versions["/riders/sa"]`
(which is `0` at the parent level), you'd get a guaranteed miss for every other rider
(`1 ≠ 0`). By rebuilding with the parent version, T2 validation is consistent with
what `check_table_cache` reads.

---

## 7. `evaluate()` — Main Form Evaluation

### 7.1 Snapshot and diff

`evaluate(data)` → `evaluate_internal_with_new_data(data)`:

1. Retrieves `old_data` from `eval_cache.main_form_snapshot` (avoids an extra clone).
2. Calls `replace_data_and_context` to update `eval_data`.
3. Runs `diff_and_update_versions(&old_data, &new_data)` → bumps `data_versions` for
   every changed field.
4. Saves `new_data` as the new `main_form_snapshot` for the next evaluation.

### 7.2 Generation-based early exit

After the diff, if `eval_generation == last_evaluated_generation` (no formula produced
a new `$params` output since the last full run), `evaluate_internal` is skipped:

```rust
if paths.is_none() && !self.eval_cache.needs_full_evaluation() {
    self.evaluate_others(paths, token, false);
    return Ok(());
}
```

This saves the full batch traversal when the data is identical or only non-formula
fields changed.

An additional early exit exists when **both** `old_data == new_data` and `old_context == new_context`:

```rust
if has_previous_eval && old_data == new_data && old_context == new_context && paths.is_none() {
    self.eval_cache.main_form_snapshot = Some(new_data);
    return Ok(());
}
```

### 7.3 Subform baseline pre-seed

Before the diff, `evaluate()` seeds `item_snapshot` in each subform's `SubformItemCache`
for every existing array element. Without this, the first `evaluate_subform` call after
a fresh `evaluate()` would diff against `Null` and treat every field as new, causing false
T2 table misses:

```rust
for (subform_path, subform) in &mut self.subforms {
    if let Some(items) = new_data.pointer(&subform_ptr).and_then(|v| v.as_array()) {
        for (idx, item_val) in items.iter().enumerate() {
            // Seed parent cache and subform cache
            self.eval_cache.subform_caches[idx].item_snapshot = item_val.clone();
            subform.eval_cache.subform_caches[idx].item_snapshot = item_val.clone();
        }
    }
}
```

---

## 8. `evaluate_dependents()` — Dependency Cascade

`evaluate_dependents(changed_paths, data, ...)` runs the following sequential steps
whenever `re_evaluate=true` and `include_subforms=true`:

```
1. process_dependents_queue      — walk the dep graph; compute new values for direct dependents
2. run_re_evaluate_pass          — full evaluate_internal to pick up $params changes
   └── readonly pass             — sync readonly fields; subform arrays get scalar-merge treatment
3. run_subform_pass (pass 1)     — evaluate dependents per subform item + T2 invalidation
4. evaluate_internal (2nd)       — refresh parent schema after T2 tables updated         ┐ only when
5. run_subform_pass (pass 2)     — re-evaluate subform items against fresh T2 tables     ┘ tables invalidated
6. patch whole-array result      — update riders array entry with post-pass eval_data
7. deduplication                 — keep last-emitted value per $ref path
```

### 8.1 `process_dependents_queue`

Walks the dependency graph starting from `changed_paths`. For each dependent formula:
- Computes the new value using the current `eval_data`.
- If the value actually changed, writes it back to `eval_data` (bumps `data_versions`
  via `bump_data_version`) and appends to the `result` array.
- Enqueues transitive dependents.

### 8.2 `run_re_evaluate_pass`

Calls `evaluate_internal` on the parent form. After `process_dependents_queue` wrote
new values, the full formula tree is re-run so `$params` tables and other aggregates
pick up the changes.

At this point, tables like `RIDER_ZLOS_TABLE` that depend on rider fields may still
return a stale **Cache HIT** — because the change happened to `riders.0.sa` (indexed
path in `data_versions`) but the T2 entry is cached against `/riders/sa` (the
subform-local path, never bumped in parent `data_versions`). This is corrected by
`run_subform_pass`.

### 8.3 Readonly pass — subform array scalar merge

Inside `run_re_evaluate_pass`, after `evaluate_internal`, each `conditional_readonly_field`
is checked. Fields whose `evaluated_schema` value differs from `eval_data` are added to
`readonly_changes` and written back to `eval_data`.

**Subform root arrays require special handling.** Writing the full schema array back to
`eval_data` would overwrite computed nested objects (e.g. `loading_benefit.first_prem`)
with the snapshot from `evaluated_schema`, which was captured **before** the T2 tables
were refreshed by `run_subform_pass`. To prevent this staleness:

- Subform root paths (e.g. `/riders`) are identified from `self.subforms`.
- For those paths only, a **per-item scalar merge** is performed: each item's top-level
  primitive fields (e.g. `sa`, `code`, `prem_pay_period`) are copied from the schema
  snapshot into the existing `eval_data` item. Nested objects (e.g. `loading_benefit`)
  are left untouched.
- All other (non-subform) readonly paths are written in full as before.

This ensures that:
1. Input scalar fields (`sa`) propagate correctly → `run_subform_pass` can detect the
   diff and invalidate the right T2 tables.
2. Computed nested output fields (`first_prem`) in `eval_data` are not poisoned with
   stale schema values.

**Path normalization**: Schema pointers for readonly fields may include the `/value/`
wrapper used internally for subform array entries (e.g.
`#/riders/value/0/sa`). This wrapper is stripped before writing to `eval_data`:
```
#/riders/properties/value/0/sa
  → normalize_to_json_pointer → /riders/properties/value/0/sa
  → .replace("/properties/", "/") → /riders/value/0/sa
  → .replace("/value/", "/")     → /riders/0/sa  ✓
```

### 8.4 `run_subform_pass` — T2 invalidation (pass 1)

For each subform (e.g. riders), for each item:

1. **Pre-diff snapshot** — records `subform_caches[idx].data_versions` BEFORE the diff.
2. **`evaluate_dependents` on the subform** — runs the subform's dependency cascade.
3. **Post-diff detection** — compares current item `data_versions` against the
   pre-diff snapshot to find paths `v > pre.get(k)`.
4. **Converts bumped paths to schema dep format**:
   - `/riders/sa` → `/riders/properties/sa` (matches `self.dependencies` key format)
5. **Finds affected `$params` tables** whose `self.dependencies[table_key]` overlap the bumped paths.
6. **Calls `invalidate_params_tables_for_item(idx, &params_table_keys)`**:
   - Bumps `params_versions` for each table path → forces T2 to miss on next read.
   - Evicts matching T1 entries for this item.
   - Sets `any_table_invalidated = true`.

The subform's own `evaluate_dependents` then re-evaluates the invalidated tables in
the correct subform context, storing fresh rows back into T2.

After the subform's `evaluate_dependents` returns, computed output values (e.g.
`first_prem`, `wop_rider_premi`) are written back to `parent eval_data` via
`eval_data.set`. The per-item `item_snapshot` is updated so subsequent
`evaluate_subform` calls see the correct post-computation baseline.

### 8.5 Second `evaluate_internal` + subform pass 2

This step only runs when `any_table_invalidated = true` from pass 1.

**Second `evaluate_internal`**: After `run_subform_pass` (pass 1), T2 has fresh
`RIDER_ZLOS_TABLE` rows, but the parent's `evaluated_schema` still contains the stale
values written during `run_re_evaluate_pass` (§8.2). Downstream tables like
`RIDER_FIRST_PREM_PER_PAY_TABLE` (which `VALUEAT`s from `RIDER_ZLOS_TABLE`) also
contain stale premium values. Running `evaluate_internal` a second time resolves this:

```rust
if subform_invalidated_tables {
    self.evaluate_internal(None, token)?;   // hits fresh T2 → re-evaluates downstream tables
}
```

**Second `run_subform_pass` (re_evaluate=true)**: Even after the second `evaluate_internal`,
the subform items still hold their pass-1 results for computed readonly fields like
`first_prem` (774200 from stale tables). Pass 2 forces each subform item to re-run its
`evaluate_internal` with the now-correct T2 entries:

```rust
self.run_subform_pass(&[], true, token, &mut result)?;
```

Because `re_evaluate=true`, every item runs its full `evaluate_dependents` → its own
`run_re_evaluate_pass` → `evaluate_internal` → T2 cache HIT with the fresh rows →
correct `first_prem` emitted into `result`.

**Whole-array patch**: After pass 2 writes fresh item values back to `eval_data`, any
`$ref: riders` entry already in `result` (from an earlier pass) is patched in-place
with the latest eval_data array so the client receives a consistent snapshot.

### 8.6 Result deduplication

All passes may independently emit entries for the same `$ref` path. A final
deduplication step keeps the **last** entry for each path (last-writer-wins):

```rust
// Pass ordering guarantees: pass 2 subform results are appended last → win.
let last_indices: IndexSet<usize> = seen.values().copied().collect();
```

This is what makes the two-pass strategy safe: pass 1's stale `riders.0.loading_benefit.first_prem`
is overwritten by pass 2's correct value without any special-case removal logic.

---

## 9. `evaluate_subform()` — Per-Item Isolated Evaluation

`evaluate_subform(path, idx, data)` runs a single rider/item evaluation through the
`with_item_cache_swap` mechanism.

### 9.1 `with_item_cache_swap` lifecycle

```
1. Replace subform eval_data with incoming merged data
2. Compute item diff → bump subform_caches[idx].data_versions
3. mem::take(parent eval_cache) → set active_item_index = Some(idx)
4. Restore T1 entries from subform's own per-item cache (stale entries evicted)
5. Propagate newly-bumped item paths to parent_cache.data_versions
6. Invalidate / re-evaluate dependent $params tables
7. mem::swap(parent cache into subform.eval_cache)
8. Run f(subform) — the actual evaluate/validate/evaluate_dependents
9. mem::swap back → restore self.eval_cache = parent_cache
10. Persist updated T1 cache back to subform's per-item cache
```

The key invariant: at step 7, the subform's `eval_cache` IS the parent cache with
`active_item_index = Some(idx)`. Every cache read inside the subform operation
goes through the two-tier lookup described in §4.

### 9.2 Item diff and newly-bumped propagation

The item diff uses `diff_and_update_versions` with the `/{field_key}` prefix, comparing
`old_item_snapshot` (from prior evaluation) against `new_item_val` (incoming data).

After the diff, **newly-bumped** item paths are also propagated to `parent_cache.data_versions`:

```rust
for k in newly_bumped {  // e.g. /riders/sa
    parent_cache.data_versions.bump(&k);
}
```

This allows `check_table_cache` (which validates T2 against `self.data_versions`) to
detect that `/riders/sa` changed, causing a T2 miss for tables with that dependency.

**Only newly-bumped paths** (v > pre) are propagated — historical bumps from prior calls
are excluded via the `pre_diff_item_versions` baseline. This prevents spurious T2
misses on every subsequent `evaluate_subform` call for an unchanged rider.

### 9.3 T1 entry restoration and stale-entry eviction

At step 4, T1 entries that lived in the subform's own `SubformItemCache` are merged
back into the active parent cache. Before merging:

1. The historical `data_versions` from the prior item cache are merged in (so the
   current tracker reflects the full accumulated state, not just the current diff).
2. Each entry is validated: `dep_versions` must match the current `data_versions`.
   Entries depending on fields that changed (e.g. `/riders/sa` bumped) are dropped.

This prevents stale T1 entries from surviving across evaluations where the rider data changed.

### 9.4 Selective `$params` table invalidation and re-evaluation

When `is_new_item` OR `item_paths_bumped`:

1. Identify affected `$params` tables whose `self.dependencies` overlap the newly-bumped paths.
2. Call `invalidate_params_tables_for_item(idx, &params_table_keys)` — bumps
   `params_versions`, evicts T1 entries.
3. For each affected table that does **NOT** depend on subform-item paths (e.g. a
   parent-scope aggregate like `WOP_RIDERS`), evaluate the table immediately using
   the parent's `eval_data` and store the result in T2.
   - Tables that DO depend on subform-item paths (e.g. `RIDER_ZLOS_TABLE` which uses
     `#/riders/properties/sa`) are **skipped** here — they must be evaluated by the
     subform engine with subform-scoped data.

The distinction: `with_item_cache_swap` handles the `evaluate_subform` API path.
`run_subform_pass` handles the same invalidation for the `evaluate_dependents` path.
Both use the same `invalidate_params_tables_for_item` + selective re-evaluation pattern.

---

## 10. Structural Change Invalidation

When the subform array length changes or items shift positions (detected by
`items_same_input_identity` comparing only raw input keys), the following cleanup runs:

1. **T2 entries with subform-local deps are evicted** (`retail` on `self.eval_cache.entries`).
   Any entry whose `dep_versions` contains a key prefixed with `/{field_key}/` is removed.
2. **`params_versions` is bumped** for each evicted T2 entry path — so downstream
   formulas correctly miss.
3. **T1 caches for index-shifted items are cleared** (entries + data_versions reset).
4. **T1 caches for removed items are pruned** (`prune_subform_caches(new_len)`).
5. **`eval_generation` is incremented**.

This runs from two call sites:
- `evaluate_internal_with_new_data` (every `evaluate()` call)
- `evaluate_dependents` when the `changed_paths` include a subform array path

---

## 11. Cache Invalidation Pathways — Reference Table

| Trigger | Mechanism | What gets invalidated |
|---|---|---|
| `evaluate(data)` with changed field | `diff_and_update_versions` → `data_versions.bump(path)` | Any T1/T2 entry with that dep path |
| `evaluate(data)` — subform array added/removed/reordered | `invalidate_subform_caches_on_structural_change` | T2 entries with subform-local deps; T1 for shifted indices |
| `evaluate_subform` — item field changed | Item diff → `subform_caches[idx].data_versions.bump` + propagate to parent `data_versions` | T1 entries for that item with that dep; T2 table entries via `check_table_cache` |
| `evaluate_subform` — brand-new item | `invalidate_params_tables_for_item` | All `$params` table T2 entries + T1 for that item |
| `evaluate_dependents` → `process_dependents_queue` writes a field | `bump_data_version` | `data_versions` bumped; T1/T2 entries with that dep |
| `$params` formula produces new value | `store_cache` → `params_versions.bump(path)` cascade | Any T1/T2 entry that depends on that `$params` path |
| `run_subform_pass` (pass 1) detects newly-bumped item paths | `invalidate_params_tables_for_item` | Specific `$params` T2 table entries + T1 for that item |
| `run_subform_pass` (pass 2, re_evaluate=true) | Subform `evaluate_internal` hits fresh T2 tables | Stale per-item computed fields (e.g. `first_prem`) in `result` overwritten by deduplication |

---

## 12. Path Formats

The caching system uses multiple path formats. It is critical to distinguish them:

| Format | Example | Used in |
|---|---|---|
| **Schema pointer** (with `#`) | `#/riders/properties/sa` | `self.dependencies` keys, `eval_key` in `check_cache` |
| **Schema pointer with `/value/` wrapper** | `#/riders/value/0/sa` | Internal `evaluated_schema` paths for subform array items; stripped before any `eval_data` write |
| **Data pointer** (no `#`) | `/riders/sa`, `/illustration/product_benefit/riders/0/sa` | `data_versions`, `params_versions`, `dep_versions` in `CacheEntry` |
| **Indexed data pointer** | `/illustration/product_benefit/riders/0/sa` | `diff_and_update_versions` output during `evaluate()` |
| **Subform-local data pointer** | `/riders/sa` | `SubformItemCache.data_versions`, T2 dep_versions for subform-item deps |
| **Dot notation** | `riders.0.sa` | External API (`changed_paths` to `evaluate_dependents`) |

The normalization from schema pointer to data pointer strips both `/properties/` segments
and the internal `/value/` wrapper:
```
#/riders/properties/value/0/sa
  → normalize_to_json_pointer()      → /riders/properties/value/0/sa
  → .replace("/properties/", "/")   → /riders/value/0/sa
  → .replace("/value/", "/")        → /riders/0/sa  ✓
```

Mismatches between these formats are the root cause of many cache staleness bugs.
Always verify which format is expected when writing invalidation or dep-matching code.

---

## 13. Debug Mode

Set the environment variable `JSONEVAL_DEBUG_CACHE=1` to enable cache HIT/MISS logging:

```sh
JSONEVAL_DEBUG_CACHE=1 cargo test --test zip_scenario test_zip_sa_tf_cascade -- --no-capture
```

Example output:
```
Cache HIT #/$params/references/RIDER_ZLOS_TABLE
Cache HIT [T2 table idx=0] #/$params/references/ZLOS_RATE
Cache MISS #/$params/references/RIDER_ZLOS_TABLE: dep /$params/references/ZLOS_RATE changed (1 -> 2)
Cache MISS #/$params/others/RIDER_FIRST_PREM_PER_PAY_TABLE/2/premi: dep /$params/references/RIDER_ZLOS_TABLE changed (1 -> 3)
```

HIT/MISS labels include the tier:
- `Cache HIT` — plain T2 hit in main-form context
- `Cache HIT [T1 idx=N]` — item-scoped T1 hit
- `Cache HIT [T2 table idx=N]` — global table T2 hit in subform context
- `Cache MISS … dep X changed (A -> B)` — version mismatch, states old and new version
- `Cache MISS … dep X missing from cache entry` — dep not in stored snapshot (new dep)
