# Subform Default Hydration Plan

> **For agentic workers:** Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans for medium/complex implementation. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make indexed subform evaluation hydrate missing visible static rider defaults, process their dependents, and re-evaluate cheaply.
**Complexity:** Medium — changes shared subform behavior and cache/version flow.
**Workflow:** Brainstorming approved; TDD and verification required.
**Architecture:** Reuse existing `JSONEval::apply_visible_static_defaults` and `run_schema_default_value_pass` after first indexed-subform evaluation. Hydration writes only absent/null/empty visible primitive values. Dependent queue then overrides defaults; second evaluation happens only after writes.
**Tech Stack:** Rust, serde_json, cargo tests.

---

## Design Summary

Indexed `evaluate_subform` already swaps parent cache/data into a subform, but calls only one evaluation pass. Main form performs evaluate → static-default hydration → re-evaluate. Add equivalent subform flow inside cache swap:

1. First evaluate resolves conditions and formulas.
2. Hydrate visible static defaults into missing active rider input.
3. Run dependent graph for hydrated fields so dependent mappings override defaults.
4. Re-evaluate only when hydration/dependents changed data.

## Requirements

- Empty ZCC rider with complete parent WOP context hydrates `code=ZLOB` and evaluates `wop_flag=true`.
- Hydration never overwrites supplied nonempty input.
- Schema defaults run before dependent mappings; dependent mappings win.
- No extra evaluation when no default applies.

## Non-goals

- Hydrating parent fields absent from a subform payload.
- Persisting arbitrary formula output as input data.
- Altering non-indexed `evaluate_subform` behavior.

## Approach / Architecture

- Modify `src/jsoneval/subform_methods.rs` cache-swap execution path to invoke a dedicated indexed-subform default/dependent pass after initial evaluation.
- Reuse `run_schema_default_value_pass` rather than duplicate default filtering or dependent queue behavior.
- Ensure active item cache remains selected for table invalidation/version updates during both passes.

## File Plan

- Modify: `src/jsoneval/subform_methods.rs` — add indexed subform evaluate/default/dependent/re-evaluate lifecycle.
- Modify: `src/jsoneval/dependents.rs` — expose existing default/dependent pass at crate scope if required.
- Test: `tests/wop_zcc_subform_test.rs` — prove empty rider hydration and parent WOP evaluation.
- Test: existing ZIP rider/cache regressions — ensure performance behavior remains valid.

## Implementation Plan

### Task 1: Establish failing ZCC subform hydration contract

**Files:**
- Modify: `tests/wop_zcc_subform_test.rs`

- [ ] Test full parent ZCC WOP payload with root `riders: {}`.
- [ ] Assert indexed subform evaluates `riders.code` to static `ZLOB` and `riders.wop_flag` to `true`.
- [ ] Run `cargo test --test wop_zcc_subform_test`; expect failure before library change.

### Task 2: Add indexed-subform hydration lifecycle

**Files:**
- Modify: `src/jsoneval/subform_methods.rs`
- Modify: `src/jsoneval/dependents.rs` only if visibility prevents reuse.

- [ ] Run first indexed subform evaluation.
- [ ] Hydrate missing visible static defaults and process their dependents.
- [ ] Run second evaluation only if a default/dependent write occurred.
- [ ] Preserve cache swap restoration and per-item evaluated-schema snapshot behavior.

### Task 3: Verify behavior and cache regressions

**Files:**
- Test: `tests/wop_zcc_subform_test.rs`
- Test: `tests/zip_scenario.rs`

- [ ] Run targeted ZCC test.
- [ ] Run ZCC WOP dependent response test.
- [ ] Run ZIP scenario serially and `cargo fmt --check` / `git diff --check`.

## Validation

```bash
cargo fmt --check
cargo test --test wop_zcc_subform_test
cargo test --test wop_zcc_enable_dependents_test
cargo test --test zip_scenario -- --test-threads=1
git diff --check
```
