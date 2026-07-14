# Layout Hidden Ref Repopulation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans for medium/complex implementation. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild `$layout` `$ref` state from current Rust evaluated schema every evaluation, prevent inherited hidden state from becoming sticky, and make layout-hidden dependent clearing match defined semantics.
**Complexity:** Complex — mutable-reference compatibility, cached evaluation state, nested layout ancestry, validation, and dependent data clearing.
**Workflow:** TDD required; isolated worktree required; targeted two-stage review after implementation tasks.
**Architecture:** Keep `evaluated_schema` as evaluation output for schema-owned conditions only. Resolve each `$layout` `$ref` afresh into overlays from it; compute per-run effective hidden reference paths without writing inherited visibility back into source schema. Getters, validation, and dependent clearing consume same effective-hidden result.
**Tech Stack:** Rust, `serde_json::Value`, `IndexMap`, Cargo integration tests.

---

## Design Summary

Legacy TypeScript mutates shared JSON object references. `$layout` ref expansion, evaluated schema, and later reads can observe same mutation. Rust `Value` clones do not share that identity. Current Rust workaround, `sync_layout_hidden_to_schema()`, copies inherited layout visibility into `evaluated_schema.condition.hidden`; later resolver passes can read this stale value and retain a child hidden after parent becomes visible.

Fix model:

```text
immutable parsed schema
        |
formula evaluation
        v
evaluated_schema                     # only own evaluated schema state
        |
resolve every $layout $ref fresh
        v
layout overlays + effective hidden ref index
        |             |                  |
        |             |                  +-- dependent clear candidates
        |             +--------------------- validation / schema-value filtering
        +----------------------------------- resolved-layout consumers
```

Each resolver pass starts from current `evaluated_schema`, expands refs into local values, recursively derives inherited hidden state, and records referenced schema paths where effective `condition.hidden` is true. It does not mutate `evaluated_schema` to represent layout ancestry.

## Requirements

- Every `$layout` `$ref` is resolved/repopulated from current `evaluated_schema` on each layout resolution.
- Parent `condition.hidden` cascades through arbitrary nested layout elements and is reversible on later evaluation.
- Layout-only `hideLayout.all` cascades visibility/filtering but does not clear stored data.
- Effective `condition.hidden` inherited through layout clears non-empty referenced data during `evaluate_dependents`, emits `{ "$hidden": true, "clear": true }`, and respects child `config.all.keepHiddenValue`.
- Schema value getters and validation use same current effective-hidden result.
- No inherited hidden state remains written into source `evaluated_schema`.
- Existing direct-field hidden, recursive hidden, subform, FFI, and WASM behavior remains intact.

## Non-goals

- Do not emulate legacy flat iteration order or its immediate-parent limitation.
- Do not change direct schema `condition.hidden` semantics.
- Do not clear input for `hideLayout.all` alone.
- Do not redesign public overlay/FFI/WASM API shapes.
- Do not refactor unrelated evaluator caching or table evaluation.

## Approach / Architecture

### Per-run layout state

Add one cached layout-state value owned by `JSONEval`:

```rust
struct ResolvedLayoutState {
    overlays: ResolvedLayoutResult,
    condition_hidden_refs: IndexSet<String>,
}
```

`condition_hidden_refs` stores normalized schema paths for `$ref` targets hidden because of a `condition.hidden` cascade. It excludes fields hidden only by `hideLayout.all`.

`resolve_layout_internal()` produces this state in a single tree walk. `get_resolved_layout()` returns cloned overlays. A private accessor resolves/returns effective hidden refs. Cache invalidation remains tied to evaluation/reload paths.

### Resolver behavior

`tree_to_overlays()` preserves existing recursion, `$parentHide`, disabled cascade, and overlay generation. During each element visit:

1. Resolve `$ref` against current `evaluated_schema` into a local `Value`.
2. Combine parent hidden state, element own evaluated `condition.hidden`, and element `hideLayout.all`.
3. Write inherited visibility only into overlay `condition` / `hideLayout` values.
4. If effective hidden originates from `condition.hidden` cascade and element has `$ref`, record its schema ref path in `condition_hidden_refs`.
5. Recurse using computed current state.

Do not call `sync_layout_hidden_to_schema()` or mutate `evaluated_schema` for inherited layout visibility.

### Shared effective-hidden behavior

`is_effective_hidden()` continues checking direct schema ancestors and layout `hideLayout.all`. It additionally checks whether current schema path is or lies below a current `condition_hidden_refs` entry. This keeps getters and validation aligned with resolved layout without sticky source-schema mutation.

`evaluate_dependents()` resolves current layout state before recursive hidden processing. It builds hidden candidates from:

- pre-parsed own conditional hidden fields, and
- current `condition_hidden_refs`.

Each candidate is passed through one common field-level clear check, so child `keepHiddenValue` applies regardless of whether hiding is own or inherited.

## File Plan

- Modify: `src/jsoneval/types.rs` — add private/internal resolved layout state type only if keeping state type outside `layout.rs` improves ownership; otherwise keep private to `layout.rs`.
- Modify: `src/jsoneval/mod.rs` — replace `resolved_layout_cache` / obsolete `layout_synced_paths` bookkeeping with cached overlays plus effective conditional-hidden refs.
- Modify: `src/jsoneval/layout.rs` — build fresh overlays and ref hidden index; remove source-schema sync mutation.
- Modify: `src/jsoneval/getters.rs` — query layout conditional-hidden index in effective-hidden checks while preserving ancestor `hideLayout.all` logic.
- Modify: `src/jsoneval/dependents.rs` — clear inherited-condition-hidden ref values via shared hidden-candidate processing.
- Test: `tests/test_layout_sync.rs` — dynamic parent hidden → visible reversal and fresh ref expansion assertions.
- Test: `tests/hidden_filtering_test.rs` — nested inherited filtering and validation regression assertions.
- Create: `tests/layout_hidden_dependents_test.rs` — dependent clear event/data behavior, nested cascade, `keepHiddenValue`, layout-only no-clear behavior.

## Implementation Plan

### Task 1: Reproduce immutable-ref regression

**Files:**
- Modify: `tests/test_layout_sync.rs`
- Create: `tests/layout_hidden_dependents_test.rs`

- [x] **Step 1: Write failing reversal test**

Create schema with dynamic parent `condition.hidden` and nested layout `$ref` child. Evaluate `toggle=true`, read resolved layout, then evaluate `toggle=false` with same evaluator. Assert child resolved `condition.hidden == false`, `$parentHide == false`, and evaluated source child has no inherited `condition.hidden=true` mutation.

- [x] **Step 2: Run red test**

Run:

```bash
cargo test --test test_layout_sync dynamic_parent_hidden_ref_repopulates_when_visible -- --exact
```

Expected: FAIL because inherited hidden state remains in `evaluated_schema` or resolved child remains hidden.

- [x] **Step 3: Write failing dependent-clear contract tests**

Add tests using real `JSONEval::evaluate_dependents`:

1. Parent dynamic `condition.hidden` hides nested layout-ref child with non-empty data; assert clear event and `eval_data` null.
2. Same child with `config.all.keepHiddenValue=true`; assert no clear and data preserved.
3. Parent `hideLayout.all=true` hiding nested child; assert no dependent clear and data preserved.
4. Three layout levels; assert condition-hidden cascade reaches leaf.

- [x] **Step 4: Run red tests**

Run:

```bash
cargo test --test layout_hidden_dependents_test -- --nocapture
```

Expected: inherited-condition-hidden clear test fails; remaining tests document current desired contract.

### Task 2: Build ephemeral layout state and fresh refs

**Files:**
- Modify: `src/jsoneval/mod.rs`
- Modify: `src/jsoneval/layout.rs`
- Modify: `src/jsoneval/getters.rs`

- [x] **Step 1: Add cached per-run layout state**

Replace `layout_synced_paths` with cache state containing overlay entries and normalized ref paths made hidden by `condition.hidden` cascade. Update constructors, clone implementation, schema reload, and evaluation invalidation consistently.

- [x] **Step 2: Make resolver return/cache both outputs**

Refactor private resolver functions so tree walk returns overlays plus conditional-hidden ref paths. Continue resolving every `$ref` from current `evaluated_schema` into local overlay values. Preserve current overlay public return type.

- [x] **Step 3: Remove source-schema visibility sync**

Delete `sync_layout_hidden_to_schema()` and `sync_hidden_to_schema_at()` use/implementation. Do not write inherited layout visibility into `evaluated_schema`.

- [x] **Step 4: Use current layout state in effective-hidden resolver**

Make `is_effective_hidden()` consult current cached/refreshed condition-hidden refs in addition to existing direct ancestor condition and `$layout.hideLayout.all` checks. A descendant of a recorded ref must count hidden.

- [x] **Step 5: Run Task 1 tests green**

Run:

```bash
cargo test --test test_layout_sync dynamic_parent_hidden_ref_repopulates_when_visible -- --exact
cargo test --test hidden_filtering_test -- --nocapture
cargo test --test layout_hidden_dependents_test -- --nocapture
```

Expected: dynamic visibility reversal, output filtering, and validation behavior pass. Inherited dependent-clear test remains red until Task 3.

### Task 3: Clear inherited condition-hidden dependent data

**Files:**
- Modify: `src/jsoneval/dependents.rs`
- Test: `tests/layout_hidden_dependents_test.rs`

- [x] **Step 1: Extend hidden candidate collection**

Before recursive hide processing, resolve/read current layout condition-hidden refs. Merge those paths with `conditional_hidden_fields`; normalize/deduplicate before checking values.

- [x] **Step 2: Preserve field-level policy**

Route own and inherited candidates through `check_hidden_field()` or a minimally extracted shared helper. For inherited refs, treat effective visibility as hidden even if source field own `condition.hidden` is false. Apply existing non-empty check and `config.all.keepHiddenValue` check.

- [x] **Step 3: Keep recursion/event semantics**

Feed resulting schema paths into existing `recursive_hide_effect()` unchanged. It must continue producing existing `$ref`, `$hidden`, and `clear` output and triggering `reffed_by` recursion.

- [x] **Step 4: Run Task 3 tests green**

Run:

```bash
cargo test --test layout_hidden_dependents_test -- --nocapture
cargo test --test test_evaluate_dependents_features -- --nocapture
```

Expected: inherited condition-hidden field clears once; keep-value and layout-only paths do not clear; existing recursive hidden behavior remains green.

### Task 4: Regression and cross-surface verification

**Files:**
- Test only; no production change unless a failing regression identifies required scoped correction.

- [x] **Step 1: Run focused evaluator suite**

```bash
cargo test --test test_layout_sync --test hidden_filtering_test --test layout_hidden_dependents_test --test test_evaluate_dependents_features --quiet
```

Expected: all pass.

- [x] **Step 2: Run FFI and WASM parity tests**

```bash
cargo test --test test_ffi_parity --test test_wasm_parity --quiet
```

Expected: all pass.

- [x] **Step 3: Run full Rust test suite** — unrelated pre-existing `tests/age_rust_debug.rs` age expectation fails (40 actual vs 39 expected).

```bash
cargo test --quiet
```

Expected: exit 0, no failed tests.

- [x] **Step 4: Inspect final scope**

```bash
git diff --check
git diff -- src/jsoneval/layout.rs src/jsoneval/mod.rs src/jsoneval/getters.rs src/jsoneval/dependents.rs tests/test_layout_sync.rs tests/hidden_filtering_test.rs tests/layout_hidden_dependents_test.rs
```

Expected: no whitespace errors; every change supports fresh-ref, reversible hidden state, or explicit regression coverage.

## Validation

- Unit/regression: focused commands in Tasks 1–4.
- Compatibility: FFI/WASM parity commands in Task 4.
- Full verification: `cargo test --quiet` exits 0.
- Scope verification: `git diff --check` exits 0.
