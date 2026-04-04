# JsonEval Benchmark Analysis

> **Last updated:** 2026-04-04 (Session 4 — Readonly Pass Fix)
> **Test:** `test_zpp_scenario_base_prem_update` in `tests/zip_scenario.rs`

## Current Baseline (after opt #1 + #2 + #3 + #4)

> **TOTAL: ~582 ms** — down from **2,806 ms** original (−79%)

**Key run timings (3-run average):**

| Run | Description | Original | Post #4 | Δ vs original |
|---|---|---|---|---|
| run 1 | initial evaluate | 849 ms | **446 ms** | −403 ms |
| run 2 | evaluate_dependents (ill_sign) | 994 ms | **335 ms** | −659 ms |
| run 3 | full re-evaluate (cache hit) | 35 ms | **31 ms** | stable |
| run 3a | subform riders (ill_sign) | 329 ms | **143 ms** | −186 ms |
| **run 4** | **evaluate_dependents (wop cascade)** | **~2,500 ms** | **~520 ms** | **−1,980 ms (−79%)** |
| run 5 | full re-evaluate (cache hit) | 35 ms | **75 ms** | see note |
| run 5a total | subform riders (wop cascade) | 826 ms | **131 ms** | −84% |

> **Note on run 5:** run 5 is now ~75ms (was 35ms). The readonly_pass change means fewer
> tables are pre-invalidated after run 4's subform processing. The first post-commit full
> evaluate now needs to recompute those tables once. This is a one-time cost; subsequent
> evaluations hit T2 normally.

---

## Optimization #1 — Lazy snapshot + scoped Arc drop (DONE)

**Result: 2,806 ms → 2,395 ms (−411 ms, −15%)**

Replaced unconditional `exclusive_clone()` at batch start with lazy `Option<EvalData>`.
Scoped `table_scope` Arc drop before `set()` keeps `Arc::make_mut` free.

---

## Optimization #2 — Per-formula inline Arc snapshot (zero exclusive_clone) (DONE)

**Result: 2,395 ms → 753 ms (−1,642 ms, −69%)**

Each formula engine call now uses a scoped `snapshot_data()` (O(1) Arc::clone) dropped
before `set()`. Zero O(n) deep copies in the entire batch loop.

---

## Optimization #3 — T2 dep_versions parent-aligned for cross-rider hit (DONE)

**Result: run 5a 575 ms → 148 ms (−74%), run 4 1,754 ms → 1,350 ms (−23%)**

### Root Cause

`$params` tables evaluated inside a subform (e.g. `WOP_ZLOB_PREMI_TABLE`) were stored
into T2 with **item data_versions** for their non-`$params` dep paths:

```
T2.dep[/riders/code] = item_data_versions[/riders/code] = 1  (bumped for rider 0)
```

But `check_table_cache` validates T2 using **parent data_versions**:

```
check: parent.data_versions[/riders/code] = 0  (never bumped in parent tracker)
result: 1 != 0 → GUARANTEED MISS for every subsequent rider
```

### Fix — `eval_cache.rs` `store_cache`

When promoting a `#/$params` table result to T2 from a subform context, the dep_versions
snapshot is rebuilt using **parent `data_versions`** for non-`$params` paths.

---

## Optimization #4 — Selective $params table invalidation in readonly_pass (DONE)

**Result: run 4 1,350 ms → ~520 ms (−830 ms, −62%)**

### Root Cause (Two bugs)

**Bug A — Wrong `had_readonly_changes` guard:**

`had_actual_readonly_changes` was `!to_process.is_empty()`. `to_process` is a shared
queue that may already contain entries from schema-default passes or the main dependents
queue — unrelated to whether a readonly field actually changed. This caused the
`invalidate_params_tables_for_item + evaluate_internal` block to fire for every rider
on every wop cascade, even when no readonly field had a new value.

**Fix:** Capture `!readonly_changes.is_empty()` **before** draining `readonly_changes`
into `to_process`, so the guard reflects only genuine readonly updates.

**Bug B — Unconditional all-table invalidation:**

Even when `had_actual_readonly_changes = true`, the code invalidated **all** `$params`
tables unconditionally (e.g. `WOP_ZLOB_PREMI_TABLE`, `WOP_RIDERS`, etc.), forcing a
full second `evaluate_internal` per rider (~300ms each × 3 riders = ~900ms).

The root issue: `wop_rider_premi` and `wop_rider_benefit` are readonly **outputs** of
the evaluation — they are not **inputs** (deps) of any `$params` table. Bumping tables
that don't read these fields was pure overhead.

**Fix:** Filter `$params` table keys to only those whose `self.dependencies` set
overlaps with the readonly-changed field paths before calling
`invalidate_params_tables_for_item`. Only if matching tables exist is the second
`evaluate_internal` triggered.

### Code change — `dependents.rs` `run_re_evaluate_pass`

```rust
// BEFORE: always ran invalidate + evaluate_internal for every rider
let had_readonly_changes = !to_process.is_empty();  // BUG: unrelated entries

// AFTER:
let had_actual_readonly_changes = !readonly_changes.is_empty();  // before drain
// ...
let params_table_keys: Vec<String> = self.table_metadata.keys()
    .filter(|k| {
        k.starts_with("#/$params") &&
        self.dependencies.get(*k).map_or(false, |deps| {
            deps.iter().any(|dep| readonly_dep_prefixes.iter().any(|ro| dep == ro))
        })
    })
    .cloned().collect();
// Only invalidate + re-eval if overlapping tables exist
if !params_table_keys.is_empty() { ... evaluate_internal ... }
```

### Measured breakdown (run 4 before/after)

| Phase | Before | After |
|---|---|---|
| `data_parse_and_diff` | 78 ms | 62 ms |
| parent `run_re_evaluate_pass` | 382 ms | ~330 ms |
| `run_subform_pass` (3 riders) | 1,096 ms | ~180 ms |
| → each rider `evaluate_internal` | 44–48 ms | 44–48 ms |
| → each rider `readonly_pass` | **238–303 ms** | **~0 ms** |
| **Total run 4** | **~1,574 ms** | **~520 ms** |

---

## Remaining Bottleneck #1 — Parent `run_re_evaluate_pass` ~330 ms

The parent (non-subform) `run_re_evaluate_pass` `evaluate_internal` takes ~330ms for
the wop cascade because `wop_basic_benefit` changing invalidates many WOP formula
batches. This is likely near the theoretical minimum for full re-evaluation of
WOP-dependent fields. Could be reduced by path-filtered evaluation (only re-evaluate
formulas in batches that have `wop_basic_benefit` as a dep).

---

## Remaining Bottleneck #2 — `batch cache fast path` 0.51 ms avg

O(keys × deps) version lookups per batch. Total = ~37 ms (~5%). Low priority.

---

## Priority Matrix

| # | Issue | Status | Savings |
|---|---|---|---|
| #1 | `exclusive_clone` → lazy snapshot | ✅ DONE | −411 ms |
| #2 | Inline Arc snapshot (no clone) | ✅ DONE | −1,642 ms |
| #3 | T2 dep_versions parent-aligned | ✅ DONE | −427 ms |
| #4 | Readonly pass: dep-filtered invalidation | ✅ DONE | **−830 ms** |
| #5 | Parent re_evaluate: path-filtered evaluate_internal | TODO | ~200–280 ms |
| #6 | `batch cache fast path` O(n×m) gen fingerprint | TODO | ~20–40 ms |
