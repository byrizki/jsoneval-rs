# Resolved Schema Parity Plan

## Design Summary
Make `getEvaluatedSchemaResolved()` produce same result as merging `getEvaluatedSchemaWithoutParams()` with `getResolvedLayout()`. Resolved output excludes root `$params`. Metadata is added to every schema property and every layout element.

## Requirements

- `getEvaluatedSchemaResolved()` omits root `$params`.
- Its result equals compact schema plus `getResolvedLayout()` overlays.
- Every property receives `$fullpath`, `$path`, and `$parentHide`.
- Property `$fullpath` uses raw schema-pointer-style dotted segments, e.g. `illustration.properties.name`.
- Inline custom layout item `$fullpath` keeps literal layout structure, e.g. `illustration.$layout.elements.1`; `$path` is `1`.
- `$ref` layout item metadata continues to use resolved target path.

## Non-goals

- No public API redesign.
- No layout resolver/cascade semantic change.
- No metadata beyond requested fields.

## Approach / Architecture

Add shared metadata stamping to Rust resolved-schema output after overlay application. It walks schema property maps, retaining raw structural segments in `$fullpath`; it sets `$path` to final segment and inherited `$parentHide`. Update inline layout metadata generation to retain `$layout.elements` in its structural path. Mirror same behavior in TypeScript overlay utility so separate compact-plus-overlay path remains deep-equal.

## Implementation Plan

1. Add Rust regression test for resolved vs compact+overlay parity, `$params` omission, all-property metadata, and literal inline layout path. Run red.
2. Add TypeScript regression test or executable utility harness for same merge metadata behavior. Run red.
3. Change Rust resolved getter to start compact schema, stamp properties, and preserve literal inline layout path. Run focused Rust tests green.
4. Change TypeScript utility to stamp property metadata and literal inline layout path. Build common package / run test harness.
5. Run focused parity/layout tests, Cargo suite scoped to changed behavior, TypeScript build, and `git diff --check`.

## Validation

```bash
cargo test --test test_layout_sync --test test_evaluate_others --test test_ffi_parity --test test_wasm_parity --quiet
yarn --cwd bindings/npm workspace @json-eval-rs/common build
git diff --check
```
