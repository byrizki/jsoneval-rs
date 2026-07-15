# Indexed Subform Transaction Plan

> **For agentic workers:** Use `subagent-driven-development` (recommended) or `executing-plans`. Steps use checkbox (`- [ ]`) tracking.

**Goal:** Make indexed subform evaluation commit one stable generation of runtime data, tables, formulas, resolved schema, and response patches.

**Complexity:** Complex — evaluation ordering, two-tier cache ownership, parent/subform projection, public binding behavior, and existing stale WOP regression cross several lifecycle boundaries.

**Workflow:** Brainstormed design approved: **indexed subforms first**. TDD before behavior code; cache-impact review before edits; test plus binding parity verification before completion.

**Architecture:** Replace current interleaving of evaluation, dependent writes, cache invalidation, patch collection, and item-schema persistence with an internal transaction. Transaction has a mutation/stabilization phase that produces no consumer-visible output, followed by a materialization/commit phase that alone produces patches and stores `SubformItemCache.evaluated_schema`. Reuse current `with_item_cache_swap` as exclusive owner of parent data binding and Tier 1/Tier 2 cache swap.

**Tech Stack:** Rust, `serde_json`, `JSONEval`, two-tier `EvalCache`, Cargo integration tests.

---

## Design Summary

### Problem

Current active indexed subform flow can expose two generations:

```text
G0 evaluate table/formula/schema      rider wop_rider_benefit is null
G0 emit patch + save item schema      wop_rider_premi = 0
G1 dependent queue writes WOP02       eval_data changes
G1 invalidate/rebuild WOP table       table now reflects WOP02
G0 consumer snapshot remains          returned/cached premium still 0
```

`eval_data` is mutable runtime state. Tables and formulas consume it. `evaluated_schema` and response patches are consumer state. `get_evaluated_schema_subform()` returns cached item schema, not raw `eval_data` or a table cache entry. Therefore cache invalidation alone cannot fix a schema/patch created before dependent writes settle.

### Target lifecycle

```text
Bind parent + active item; acquire item cache
→ evaluate source state
→ write defaults / readonly / dependent changes into runtime data
→ record dirty runtime paths
→ invalidate only tables/formulas dependent on dirty paths
→ re-evaluate affected state
→ repeat until dirty set empty or iteration limit reached
→ resolve final schema, readonly, visibility, and layout
→ collect patches from final generation
→ atomically persist final item schema/cache and return final patches
```

### Required invariants

1. Returned `$ref` patches, `eval_data`, cached formula/table values, and `SubformItemCache.evaluated_schema` refer to same finalized item generation.
2. Provisional patches cannot escape mutation/stabilization phase.
3. `with_item_cache_swap` remains sole owner of parent snapshot binding, active-index selection, T1/T2 cache swap, version propagation, and final item-cache persistence.
4. Invalidations are dependency-targeted. No global table/formula eviction.
5. Parent projection and explicit item edit retain separate scope semantics, but both use same transaction finalization contract.
6. Non-convergent evaluation returns deterministic diagnostic error with changed paths; no unbounded loop.

## Requirements

- Existing indexed subform API and bindings remain unchanged.
- WOP self-relation enable derives rider `WOP02` and returns/stores nonzero WOP premium without unrelated SA edit.
- Relation transitions recompute or clear WOP outputs correctly.
- Explicit rider edits retain current table freshness behavior.
- Parent-driven projection does not commit transient table results into durable parent cache.
- Same item cache stays isolated per index; no item-to-item table/schema bleed.
- Existing readonly/default/hidden semantics preserved after stabilization.
- Stale provisional patch must never be observable, even if final value equals another valid falsy value such as `0`, `false`, empty string, or `null`.

## Non-goals

- Rewrite generic full-form evaluator.
- Change schema dependency extraction grammar.
- Implement a whole-engine reactive graph or background incremental evaluator.
- Change public Rust, C#, WASM, React Native, or Node API signatures.
- Optimize unrelated tables or alter frontend worker protocol.

## Approach / Architecture

### Transaction boundary

Create an internal transaction module, preferably `src/jsoneval/subform_transaction.rs`. It owns **only** indexed-subform evaluation phases while borrowing existing `JSONEval` facilities. Keep raw cache storage in `eval_cache.rs`; do not duplicate cache rules in transaction.

Suggested types:

```rust
pub(crate) struct IndexedSubformTransaction<'a> {
    subform: &'a mut JSONEval,
    item_index: usize,
    dirty_paths: BTreeSet<String>,
    provisional_changes: Vec<Value>,
    final_changes: Vec<Value>,
    max_rounds: usize,
}

pub(crate) struct StabilizationReport {
    pub rounds: usize,
    pub changed_paths: BTreeSet<String>,
}
```

Exact names may vary; responsibilities cannot.

### Phase A — mutation and stabilization

1. Perform initial evaluation using current `evaluate_internal_pre_diffed` behavior.
2. Apply visible static defaults, readonly writes, dependent queue effects, recursive hide effects, and parent-driven projected writes to runtime `eval_data`.
3. Every write returns/records normalized data path in `dirty_paths` and bumps correct active-item data version.
4. Resolve affected `$params` table keys from existing dependency metadata. Invalidate T1 and T2 with `invalidate_params_tables_for_item` only for affected keys.
5. Resolve calculated schema leaves/formula entries downstream of changed tables/paths. Re-evaluate using existing evaluator; do not manually calculate premium or copy table rows.
6. Continue while writes or targeted invalidations produce new dirty paths.
7. Bound rounds. Recommended initial limit: 8. If limit reached with new paths, return `Indexed subform evaluation did not stabilize after 8 rounds: <paths>`.
8. `provisional_changes` exists only for existing internal helpers that require a result vector; discard it before materialization.

### Phase B — materialization and atomic commit

1. Snapshot active-item version tracker after stabilization.
2. Run final schema resolution pass with no runtime writes permitted. Assert/debug-check no dirty path appears; if one appears, return it to Phase A rather than emitting output.
3. Build response patches by comparing finalized schema/versions to transaction's final-generation baseline.
4. Evaluate readonly/hidden/layout output from finalized schema.
5. Let `with_item_cache_swap` persist item snapshot, T1 entries, and clone of finalized `evaluated_schema` only after Phase B succeeds.
6. Deduplicate final patches by `$ref` with existing last-writer semantics, but first assert no provisional patch was merged.

### Dirty-path and dependency APIs

Add small, reusable helpers rather than reproducing dependency matching in `dependents.rs` and `subform_methods.rs`:

```rust
fn normalized_dirty_data_path(schema_or_data_path: &str) -> String;
fn params_tables_depending_on(&self, paths: &BTreeSet<String>) -> Vec<String>;
fn formula_outputs_depending_on(&self, paths: &BTreeSet<String>) -> Vec<String>;
```

Rules:

- Normalize `#`, schema `/properties/`, data paths, and `/value` explicitly at helper boundary.
- Match exact dependency and child path dependency; no prefix that can confuse `foo` with `foobar`.
- Use one representation in all transaction code. Preserve current graph keys where they omit `#`.
- Formula invalidation must cover a schema leaf dependent on an invalidated table even when there is no direct input-field dependent edge.

### Integration points

- `evaluate_subform_item` becomes transaction entrypoint for direct indexed edits.
- Parent-to-subform dependent cascade in `dependents.rs` invokes transaction in projection mode. Projection mode selects parent-affected formulas/writes but uses same Phase B final materialization.
- `run_re_evaluate_pass` becomes a Phase A helper or splits into explicit mutation and materialization functions. It must no longer emit patches before dependents settle.
- `with_item_cache_swap` executes transaction callback and commits item state only after `Result::Ok`; error path restores cache ownership without saving partial `evaluated_schema`.

## File Plan

- Create: `src/jsoneval/subform_transaction.rs` — indexed transaction state, stabilization loop, final materialization contract.
- Modify: `src/jsoneval/mod.rs` or module declaration owner — expose internal module only.
- Modify: `src/jsoneval/dependents.rs` — split mutation-only dependent/readonly/hide helpers from patch generation; parent cascade delegates to transaction projection mode.
- Modify: `src/jsoneval/subform_methods.rs` — route `evaluate_subform_item` through transaction; make `with_item_cache_swap` commit only finalized schema/item cache.
- Modify: `src/jsoneval/eval_cache.rs` — targeted dirty-path invalidation helpers; cache entry/formula invalidation needed for final schema recalculation; no public API changes.
- Modify: `src/jsoneval/evaluate.rs` — expose narrow internal evaluation phase primitive only if existing visibility/layout ordering cannot be reused.
- Test: `tests/zcc_wop_edge_cases_test.rs` — WOP setup, self/non-self relation transitions, response/schema final-generation agreement.
- Test: `tests/rider_cascade_test.rs` — parent projection behavior and output clearing.
- Test: `tests/zip_scenario.rs` — existing table freshness regression.
- Test: create `tests/indexed_subform_transaction_test.rs` — generic transaction invariants with compact schema fixture.

## Implementation Plan

### Task 1: Lock contract with red tests

**Files:**
- Modify: `tests/zcc_wop_edge_cases_test.rs`
- Create: `tests/indexed_subform_transaction_test.rs`
- Modify: `tests/rider_cascade_test.rs`

- [ ] **Step 1: Add WOP final-generation regression.**

Assert after `evaluate_dependents` on self-relation WOP enable:

```rust
let changes = evaluate_change(&mut eval, &mut data, WOP_FLAG);
let returned_premium = patch_value(&changes, "illustration.product_benefit.riders.0.wop_rider_premi");
let cached_premium = eval
    .get_evaluated_schema_subform("illustration.product_benefit.riders.0")
    .pointer("/riders/properties/wop_rider_premi/value")
    .cloned();
assert_nonzero(&returned_premium, "final returned rider premium must be nonzero");
assert_eq!(returned_premium, cached_premium.unwrap());
```

Add equivalent checks for relation `3 → 1` and `1 → 3`.

- [ ] **Step 2: Add compact generic fixture.**

Schema: source formula writes selector; `$params` table reads selector; calculated leaf reads table. Assert one public indexed-subform call emits only final leaf patch and saves same final schema leaf. Avoid ZCC-specific field names.

- [ ] **Step 3: Add isolation and convergence tests.**

Two different item indexes must retain distinct tables/schema. Fixture with oscillating dependent writes must return bounded non-convergence error rather than hang.

- [ ] **Step 4: Run red tests.**

Run:

```bash
cargo test --test zcc_wop_edge_cases_test self_relation_rider_wop_enable_calculates_premium_without_sa_change -- --nocapture
cargo test --test indexed_subform_transaction_test -- --nocapture
```

Expected: existing WOP target fails; new generic contract fails because provisional consumer materialization still exists.

### Task 2: Extract path/dependency selection helpers

**Files:**
- Modify: `src/jsoneval/eval_cache.rs`
- Modify: `src/jsoneval/dependents.rs`
- Test: `tests/indexed_subform_transaction_test.rs`

- [ ] **Step 1: Add normalized path conversion helper tests.**

Cover schema `#/riders/properties/wop_rider_benefit/value`, graph `/riders/properties/wop_rider_benefit`, and runtime `/riders/wop_rider_benefit` representations.

- [ ] **Step 2: Implement one targeted dependency selector.**

Selector returns affected `$params` tables and calculated schema outputs. It must require exact segment match or child segment match.

- [ ] **Step 3: Add cache invalidation method accepting normalized dirty paths.**

Use current `invalidate_params_tables_for_item` internals for table keys. Add formula/cache eviction only where cache entry is proven downstream by dependency metadata. Never evict unrelated table keys.

- [ ] **Step 4: Run selection tests.**

Run:

```bash
cargo test --test indexed_subform_transaction_test dirty_path -- --nocapture
cargo test --test indexed_subform_transaction_test dependency -- --nocapture
```

Expected: only declared table/formula entries selected; segment-boundary near misses excluded.

### Task 3: Add transaction skeleton with no behavior change

**Files:**
- Create: `src/jsoneval/subform_transaction.rs`
- Modify: module declaration owner
- Modify: `src/jsoneval/subform_methods.rs`
- Test: `tests/indexed_subform_transaction_test.rs`

- [ ] **Step 1: Add `IndexedSubformTransaction` state and Phase enum.**

Transaction accepts active `JSONEval`, index, token, evaluation-path selector, and projection/direct mode. It owns provisional/final patch vectors and dirty set.

- [ ] **Step 2: Route `evaluate_subform_item` through transaction while preserving existing single-pass behavior.**

Keep `with_item_cache_swap` outermost. Transaction must not move/swap cache state itself.

- [ ] **Step 3: Add commit guard.**

`with_item_cache_swap` persists `item_cache.evaluated_schema` only when transaction returns a finalization report. On `Err`, preserve restored parent cache but do not overwrite prior committed item snapshot/schema.

- [ ] **Step 4: Run direct indexed regressions.**

Run:

```bash
cargo test --test zcc_wop_edge_cases_test relation_change_refreshes_indexed_rider_wop_disabled_condition
cargo test --test zip_scenario
```

Expected: existing passing behavior unchanged; skeleton adds no material semantic change.

### Task 4: Split mutation from materialization

**Files:**
- Modify: `src/jsoneval/dependents.rs`
- Modify: `src/jsoneval/subform_transaction.rs`
- Modify: `src/jsoneval/evaluate.rs` only if narrow internal hook required
- Test: `tests/indexed_subform_transaction_test.rs`

- [ ] **Step 1: Extract mutation-only helpers.**

Move default, readonly write, dependent queue write, and recursive hide runtime mutation out of code that emits result patches. Each helper returns normalized paths it changed.

- [ ] **Step 2: Preserve read-only semantics.**

Readonly generated patch metadata (`$readonly`, `clear`) must be built only in materialization. Runtime readonly write remains Phase A.

- [ ] **Step 3: Preserve hide semantics.**

Recursive hide can mutate runtime values in Phase A. Its response clear patches must be built from final state in Phase B.

- [ ] **Step 4: Add regression: provisional value absent.**

Generic fixture must assert response array has no early `$ref` with zero/null when final calculated leaf is nonzero.

- [ ] **Step 5: Run focused suite.**

```bash
cargo test --test indexed_subform_transaction_test
cargo test --test zcc_wop_edge_cases_test
cargo test --test rider_cascade_test
```

Expected: no changed user-visible behavior except intended final-generation semantics.

### Task 5: Implement bounded stabilization

**Files:**
- Modify: `src/jsoneval/subform_transaction.rs`
- Modify: `src/jsoneval/eval_cache.rs`
- Test: `tests/indexed_subform_transaction_test.rs`

- [ ] **Step 1: Stabilization round.**

For a nonempty dirty set: select affected tables/formulas; invalidate active-item entries; force re-evaluation; run mutation helpers; add new paths. Clear paths consumed in current round.

- [ ] **Step 2: Loop safety.**

Track `(path, version)` or round-specific fresh writes. Stop on no new versions/paths. Cap at 8 rounds. Return deterministic error including sorted paths after cap.

- [ ] **Step 3: Avoid false iterations.**

Repeated writes of equal JSON value must not bump version or create dirty path. Historical item versions must not look newly dirty.

- [ ] **Step 4: Run stabilization tests.**

```bash
cargo test --test indexed_subform_transaction_test stabilization -- --nocapture
cargo test --test zcc_wop_edge_cases_test -- --nocapture
```

Expected: WOP target passes; intentional cycle errors after bounded rounds; no panic/hang.

### Task 6: Implement final materialization and atomic item commit

**Files:**
- Modify: `src/jsoneval/subform_transaction.rs`
- Modify: `src/jsoneval/dependents.rs`
- Modify: `src/jsoneval/subform_methods.rs`
- Test: `tests/zcc_wop_edge_cases_test.rs`

- [ ] **Step 1: Capture baseline after Phase A stabilization.**

Use active item version tracker. Final pass determines changed calculated leaves from this stable baseline.

- [ ] **Step 2: Build final patches.**

Generate formula values, readonly metadata, and hide clears after final evaluation only. Deduplicate within final patch vector only.

- [ ] **Step 3: Atomically persist.**

When transaction finalizes successfully, write `item_snapshot`, T1 entries, and `evaluated_schema` in existing item cache. A partial transaction must not replace previous schema snapshot.

- [ ] **Step 4: Test response/schema equality.**

For WOP and generic fixture, each computed field emitted in final patches must equal final cached schema value. Confirm direct subsequent `get_evaluated_schema_subform` call returns same value.

- [ ] **Step 5: Run cache isolation suite.**

```bash
cargo test --test indexed_subform_transaction_test commit -- --nocapture
cargo test --test zcc_wop_edge_cases_test
cargo test --test zip_scenario
```

Expected: per-item snapshots remain isolated; WOP premium final value is nonzero without SA edit.

### Task 7: Migrate parent-to-subform cascade to projection transaction mode

**Files:**
- Modify: `src/jsoneval/dependents.rs`
- Modify: `src/jsoneval/subform_transaction.rs`
- Modify: `src/jsoneval/subform_methods.rs`
- Test: `tests/rider_cascade_test.rs`
- Test: `tests/zcc_wop_edge_cases_test.rs`

- [ ] **Step 1: Define `Projection` mode.**

Mode accepts selected parent-dependent formulas. It may mutate active rider runtime data but must not publish parent-only global tables from disposable projection state.

- [ ] **Step 2: Replace direct overlay result handling.**

Current parent cascade must request transaction final result, then merge only final item patches. Remove direct double-pass/overlay cache manipulation once equivalent tests pass.

- [ ] **Step 3: Preserve durable-cache boundary.**

Projection cannot leave transient item entries or table results in wrong parent cache tier. Explicit item path can commit normally after finalization.

- [ ] **Step 4: Test parent-triggered WOP.**

Relation/WOP parent changes must update indexed rider premium and cached schema in same call. Verify an unrelated parent change produces no rider transaction.

- [ ] **Step 5: Run parent cascade suite.**

```bash
cargo test --test rider_cascade_test -- --nocapture
cargo test --test zcc_wop_edge_cases_test -- --nocapture
```

Expected: parent cascade behavior correct without stale patch or table-cache pollution.

### Task 8: Remove legacy split lifecycle and validate compatibility

**Files:**
- Modify: `src/jsoneval/dependents.rs`
- Modify: `src/jsoneval/subform_methods.rs`
- Modify: `src/jsoneval/eval_cache.rs`
- Test: relevant existing suites

- [ ] **Step 1: Delete superseded overlay re-evaluation branches.**

Remove only paths fully replaced by transaction. Preserve unrelated main-form behavior.

- [ ] **Step 2: Audit all item schema writes.**

Search `evaluated_schema =`, `subform_caches`, and response `$ref` construction. Confirm indexed subform uses transaction commit/materialization path only.

- [ ] **Step 3: Ensure no temporary diagnostics remain.**

No `eprintln!`, debug branches, or test-only production conditionals.

- [ ] **Step 4: Run full validation.**

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test indexed_subform_transaction_test
cargo test --test zcc_wop_edge_cases_test
cargo test --test rider_cascade_test
cargo test --test zip_scenario
cargo test --tests
```

Expected: all pass; no formatting/lint errors; WOP edge test proves output/cache final-generation invariant.

### Task 9: Binding and regression review

**Files:**
- Inspect: `bindings/csharp/JsonEvalRs.Subforms.cs`
- Inspect: `bindings/web/**` subform call sites
- Inspect: `bindings/react-native/**` subform call sites
- Test: existing parity/binding test commands documented by workspace

- [ ] **Step 1: Verify no binding observes provisional Rust state.**

Public method signatures unchanged. Verify `evaluate_subform`, dependent evaluation, and `get_evaluated_schema_subform` only observe post-commit state.

- [ ] **Step 2: Run repository binding smoke tests where available.**

Use documented commands from each workspace; do not introduce binding changes absent a compile/test failure.

- [ ] **Step 3: Reviewer pass.**

Review transaction ownership, cache-tier transitions, failure atomicity, dirty path normalization, and no-output-before-stable invariant.

## Validation

### Mandatory Rust validation

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test indexed_subform_transaction_test
cargo test --test zcc_wop_edge_cases_test
cargo test --test rider_cascade_test
cargo test --test zip_scenario
cargo test --tests
```

### Required acceptance evidence

- Self-relation WOP enable returns nonzero `wop_rider_premi` with no SA edit.
- Returned WOP premium equals `get_evaluated_schema_subform(...).pointer(.../value)` immediately after same call.
- Relation transitions and readonly/hidden handling stay correct.
- Two item indexes retain independent cached schema/table output.
- Cycle test terminates with bounded diagnostic error.
- Parent projection cannot leak transient cache state.

## Future Plan and Scope — Not Included in This Implementation

### Phase 2: all subform flows

**Scope:** Unindexed/standalone `evaluate_subform`, `validate_subform`, layout-resolution subform paths, and every parent-to-subform projection route use one shared transaction facade.

**Why defer:** Current accepted scope fixes indexed subforms, where active item cache, parent array visibility, and stale WOP output interact. Broad migration increases risk and blocks urgent repair. Phase 1 transaction should be designed reusable, but Phase 2 begins only after Phase 1 demonstrates stable API and cache behavior.

**Phase 2 work:**

1. Map all public/internal subform flows and their output/cache commit points.
2. Extend transaction context to optional index; retain a lightweight standalone mode.
3. Migrate validation/layout operations only where they mutate/resolved-schema cache; read-only operations may remain direct.
4. Add cross-mode parity matrix: standalone vs indexed vs parent projection outputs for equivalent fixture.
5. Remove duplicate lifecycle code only after parity suite passes.

**Phase 2 acceptance:** Same input/schema has consistent resolved schema and patches regardless of subform entrypoint; no duplicate cache swap/binding implementation remains.

### Phase 3: evaluator-wide dependency scheduler

**Scope:** Replace hand-sequenced full evaluator stages with explicit dependency work graph for full form and subforms.

**Why defer:** This is not required for WOP, carries high semantic compatibility risk, and existing dependency metadata may need normalization first.

**Phase 3 work:**

1. Define node types: input, formula, table, condition, readonly effect, hide effect, schema materialization.
2. Compile normalized dependency graph from schema metadata.
3. Use deterministic dirty-work queue with topological processing; retain bounded strongly connected component handling for intentional cycles.
4. Add generation snapshots and trace diagnostics per node.
5. Benchmark against existing schemas before enabling; gate behind internal feature flag during migration.

**Phase 3 acceptance:** Equivalent full-form/subform output across golden schemas; measurable no-regression performance; dependency traces explain each recalculation; old lifecycle removed only after full parity confidence.

### Explicitly rejected alternative: cache-only patch

Adding further table invalidation or rerunning `evaluate_internal` inside current overlay flow cannot guarantee output/schema atomicity. It rebuilds table cache but may preserve a pre-write patch or snapshot. This plan instead moves output ownership after stabilization.
