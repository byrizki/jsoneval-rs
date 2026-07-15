# Rider Shared Cache Plan

> **For agentic workers:** Inline execution. User explicitly requested no worktree.

**Goal:** Restore rider dependent-result updates after parent changes while preserving correct rider table values and avoiding repeated heavy table evaluation.
**Complexity:** Complex — cache ownership and evaluation scope cross parent/subform boundaries; regression affects ZIP and ZPP schemas.
**Workflow:** TDD; shared item-context architecture approved by user.
**Architecture:** Existing `with_item_cache_swap` becomes sole owner of parent/subform data binding and Tier 1/Tier 2 cache lifecycle. Parent-only changes use projection evaluation: exact affected non-table rider formulas run in active-item context; `$params` tables never run. Explicit rider edits retain full item evaluation and dependency-driven table refresh.
**Tech Stack:** Rust, `serde_json`, existing `JSONEval` Tier 1/Tier 2 evaluation cache, Cargo integration tests.

---

## Design Summary

A rider formula needs two simultaneous views:

```text
#/riders/properties/**                  $datas
active rider at root /riders            full parent /illustration/.../riders[]
```

Current duplicate `run_subform_pass` setup builds a reduced payload and reconstructs cache state. It cannot safely satisfy both views and duplicates `with_item_cache_swap` logic. The shared item-swap path must own full-parent data, root active-item binding, version tracking, T1 cache selection, and T2 cache swap.

`ParentProjection` receives only formula keys whose declared dependencies exactly include a changed parent path. It filters out table keys and evaluates no `$params` table. This restores rider output events while preventing transient per-rider table evaluations from overwriting ZIP global rows.

## Requirements

- Parent-only ZPP changes emit necessary rider updates/clears.
- ZIP rider table results remain correct after parent transition; no empty/stale `$datas` table result.
- Projection keeps full parent array for `$datas` and active rider binding for `#/riders/...`.
- Cache promotion never stores a projection result as a table result.
- Unrelated parent changes perform zero rider work.
- Explicit rider edits retain current dependency-correct table behavior.

## Non-goals

- Redesign schema dependency extraction.
- Rewrite table engine or table formula semantics.
- Add timing thresholds dependent on host hardware.
- Change frontend worker protocol.

## File Plan

- Modify: `tests/rider_cascade_test.rs` — focused ZPP projection regression and ZIP table regression.
- Modify: `src/jsoneval/subform_methods.rs` — shared projection entrypoint built on `with_item_cache_swap`.
- Modify: `src/jsoneval/dependents.rs` — select projection formulas and delegate; remove duplicate context/cache setup.
- Validate: `tests/zip_scenario.rs`, `tests/rider_cascade_test.rs`.

## Implementation Plan

### Task 1: Confirm red regressions

**Files:** `tests/rider_cascade_test.rs`

- [x] Add a focused ZPP parent `wop_flag=false` regression proving rider clear events are required.
- [x] Run it and confirm old behavior fails because parent-only rider evaluation is skipped.
- [x] Preserve ZIP TF/SA regression coverage for table correctness.

### Task 2: Implement shared projection entrypoint

**Files:** `src/jsoneval/subform_methods.rs`

- [ ] Reuse `with_item_cache_swap`; do not duplicate cache swap, item diff, T1 merge, or table invalidation.
- [ ] Bind full parent payload and active item in the shared context.
- [ ] Add internal projection runner accepting selected formula keys.
- [ ] Reject/ignore table keys in projection.
- [ ] Return changed rider values, read-only values, and hide clears.

### Task 3: Delegate parent rider refresh to projection

**Files:** `src/jsoneval/dependents.rs`

- [ ] Find non-table subform formulas whose dependency metadata includes each changed parent path.
- [ ] Direct rider edits use existing full behavior.
- [ ] Parent-only changes call projection once per rider only when selection is non-empty.
- [ ] Delete manual subform context/cache lifecycle from `run_subform_pass`.
- [ ] Preserve existing output remapping and whole-array result patching.

### Task 4: Validate behavior and performance boundaries

**Files:** `tests/rider_cascade_test.rs`, `tests/zip_scenario.rs`

- [ ] ZPP WOP parent change emits rider clear events.
- [ ] ZIP TF transition preserves non-empty rider table and numeric non-zero premium.
- [ ] Explicit rider edit regression passes.
- [ ] Verify projection selection contains no `#/$params/**` key.
- [ ] Run formatting and complete Rust test suite.

## Validation

```bash
cargo fmt --check
cargo test --test rider_cascade_test
cargo test --test zip_scenario
cargo test --tests
```

Expected: ZPP rider clear output returns. ZIP table/premium behavior remains correct. Parent-only projection never evaluates `$params` tables.
