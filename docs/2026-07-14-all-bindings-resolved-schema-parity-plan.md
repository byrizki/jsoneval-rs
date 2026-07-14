# All Bindings Resolved Schema Parity Plan

> **For agentic workers:** Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` for medium/complex implementation. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every public resolved-schema API compose compact schema plus layout overlays, for root schemas and subforms, while preserving Rust output parity.
**Complexity:** Complex — public behavior changes across TypeScript, React Native, C#, Rust tests, and generated-artifact boundaries.
**Workflow:** TDD, focused build/test verification, read-only review before completion.
**Architecture:** Rust core remains canonical implementation. WebCore and React Native compose their existing compact and overlay APIs through common TypeScript `resolveEvaluatedLayout`. C# composes existing compact and overlay FFI calls through an internal Newtonsoft merger mirroring common/Rust semantics. Node, Bundler, and Vanilla inherit WebCore behavior.
**Tech Stack:** Rust/serde_json, TypeScript, React Native/Jest, C#/.NET/Newtonsoft.Json.

---

## Design Summary

Resolved schema output must have one invariant across all public bindings:

```text
resolved schema = evaluated schema without $params + resolved-layout overlays
```

This applies to root and subform APIs. Inline non-`$ref` layout items preserve literal structural paths. Example:

```text
#/illustration/$layout/elements/1
→ illustration.$layout.elements.1
```

`$fullpath`, `$path`, and `$parentHide` must exist on schema properties even if a schema/subform has no layout overlay entries.

## Requirements

- Root and subform public resolved-schema APIs compose their respective compact-schema and resolved-layout APIs.
- WebCore subform composition flows to Node, Bundler, and Vanilla through inheritance.
- React Native root and subform composition uses `@json-eval-rs/common` merger and never invokes native `getEvaluatedSchemaResolved*` APIs.
- C# root and subform composition uses existing `GetEvaluatedSchemaWithoutParams*` and `GetResolvedLayout*` public methods; public resolved methods do not invoke resolved-schema FFI exports.
- Overlay merge behavior matches Rust common behavior: shallowest layout first, live `$ref` resolution, `$layout` flattening, inline metadata stamping, recursive layout traversal, property metadata stamping.
- Tests prove direct native resolved calls are bypassed where wrappers exist.
- Do not hand-edit generated WASM packages or React Native `lib/` build outputs.

## Non-goals

- Remove Rust/WASM/FFI native resolved getter exports.
- Change low-level native ABI APIs.
- Change output schema metadata semantics beyond existing Rust parity.
- Regenerate or commit generated package outputs unless package build tooling produces required tracked files.

## Approach / Architecture

### TypeScript shared merger

`bindings/npm/packages/common/src/utils.ts` stays sole TypeScript merger. It fetches compact schema and overlays concurrently, merges overlays in parent-first order, then stamps layout and property metadata. It already handles no-overlay property metadata.

### Web bindings

`JSONEvalCore` root method already delegates. Extend its subform method with callbacks to `getEvaluatedSchemaWithoutParamsSubform` and `getResolvedLayoutSubform`. Node, Bundler, and Vanilla need no direct source changes because they subclass `JSONEvalCore`.

### React Native

Import `resolveEvaluatedLayout`. Root and subform methods delegate through existing public compact and overlay methods after disposal checks. Bridge/JSI native `getEvaluatedSchemaResolved*` methods remain for ABI compatibility but receive no public-wrapper calls.

### C#

Add an internal static merger in a new source file, using `JObject`/`JArray` and `SelectToken`. It parses overlay entries from existing `JArray`, sorts by pointer depth and index, resolves/merges live `$ref` elements, flattens `$layout`, applies overlay data, recursively stamps layout items, and stamps properties. Public root and subform methods call it after validation/disposal checks.

## File Plan

- Modify: `bindings/npm/packages/webcore/src/index.ts` — compose subform resolved schema via common merger.
- Modify: `bindings/npm/packages/webcore/test/resolved-schema.test.mjs` — cover root and subform composition, forbidding native resolved calls.
- Modify: `bindings/npm/packages/react-native/src/index.tsx` — compose root/subform resolved schema via common merger.
- Create: `bindings/npm/packages/react-native/src/__tests__/resolved-schema.test.ts` — mocked bridge/JSI-free root/subform regressions.
- Create: `bindings/csharp/JsonEvalRs.LayoutOverlayMerger.cs` — internal Newtonsoft parity merger.
- Modify: `bindings/csharp/JsonEvalRs.Main.cs` — compose root resolved schema via merger.
- Modify: `bindings/csharp/JsonEvalRs.Subforms.cs` — compose subform resolved schema via merger.
- Create: `bindings/csharp/tests/JsonEvalRs.LayoutOverlayMergerTests.cs` or existing C# test-project equivalent — merger behavior tests; if no C# test project exists, add only testable internal coverage in project-approved location after confirming build configuration.
- Modify: `tests/resolved_schema_parity_test.rs` — root and subform compact-plus-overlays equality, metadata/path assertions.
- Modify only if needed: `bindings/npm/packages/common/test/utils.test.mjs` — direct common-merger regression additions.

## Implementation Plan

### Task 1: Prove WebCore subform delegation

**Files:**
- Modify: `bindings/npm/packages/webcore/test/resolved-schema.test.mjs`
- Modify: `bindings/npm/packages/webcore/src/index.ts`

- [ ] **Step 1: Write failing subform test**

Add mock methods for compact and overlay subform data. Make `getEvaluatedSchemaResolvedSubform()` throw. Assert returned inline path:

```js
assert.equal(
  resolved.subform.$layout.elements[1].$fullpath,
  'subform.$layout.elements.1',
);
```

- [ ] **Step 2: Run RED**

Run: `yarn --cwd bindings/npm workspace @json-eval-rs/webcore build && node bindings/npm/packages/webcore/test/resolved-schema.test.mjs`

Expected: failure from mock native `getEvaluatedSchemaResolvedSubform` call.

- [ ] **Step 3: Implement minimal delegation**

```ts
return resolveEvaluatedLayout(
  () => this.getEvaluatedSchemaWithoutParamsSubform({ subformPath }),
  () => this.getResolvedLayoutSubform({ subformPath }),
);
```

- [ ] **Step 4: Run GREEN**

Run same command. Expected: pass.

### Task 2: Prove React Native root and subform delegation

**Files:**
- Create: `bindings/npm/packages/react-native/src/__tests__/resolved-schema.test.ts`
- Modify: `bindings/npm/packages/react-native/src/index.tsx`

- [ ] **Step 1: Write failing mocked native-module tests**

Mock `react-native` native module methods. Supply compact and overlay methods. Make native `getEvaluatedSchemaResolved` and `getEvaluatedSchemaResolvedSubform` throw. Assert root and subform inline structural paths plus property metadata.

- [ ] **Step 2: Run RED**

Run: `yarn --cwd bindings/npm workspace @json-eval-rs/react-native test --runInBand resolved-schema.test.ts`

Expected: failure from native resolved method calls.

- [ ] **Step 3: Implement minimal delegation**

Import `resolveEvaluatedLayout`; replace both public resolved getters with callback composition over existing public compact/overlay methods.

- [ ] **Step 4: Run GREEN**

Run focused Jest command. Expected: pass.

- [ ] **Step 5: Type-check**

Run: `yarn --cwd bindings/npm workspace @json-eval-rs/react-native typescript`

Expected: no TypeScript errors.

### Task 3: Add C# internal layout-overlay merger and use it

**Files:**
- Create: `bindings/csharp/JsonEvalRs.LayoutOverlayMerger.cs`
- Modify: `bindings/csharp/JsonEvalRs.Main.cs`
- Modify: `bindings/csharp/JsonEvalRs.Subforms.cs`
- Create: C# test project/file only after confirming existing .NET test infrastructure.

- [ ] **Step 1: Write failing merger test(s)**

Cover:
- `$ref` layout element merges referenced field plus overlay;
- inline TabLayout at `#/illustration/$layout/elements/1` gets `illustration.$layout.elements.1`;
- properties receive metadata with empty overlays;
- nested/subform-relative layout pointer behavior.

- [ ] **Step 2: Run RED**

Run detected C# test command, or if no test project exists, run compilation containing deliberately referenced missing merger API and record failure. Do not modify production methods before a failing assertion/test exists.

- [ ] **Step 3: Implement `LayoutOverlayMerger.Merge(JObject schema, JArray overlays)`**

Implement same ordering and metadata semantics as `mergeLayoutOverlay`; operate only on cloned/returned compact `JObject` and never mutate native results.

- [ ] **Step 4: Route public methods**

```csharp
return LayoutOverlayMerger.Merge(
    GetEvaluatedSchemaWithoutParams(),
    GetResolvedLayout());
```

For subforms pass `subformPath` to both existing compact/overlay methods.

- [ ] **Step 5: Run GREEN**

Run C# test/build command. Expected: tests/build pass.

### Task 4: Extend Rust parity coverage to subforms

**Files:**
- Modify: `tests/resolved_schema_parity_test.rs`

- [ ] **Step 1: Write failing subform compact-plus-overlay equality assertion**

Create array-items subform containing `$params`, property reference, and inline TabLayout. Assert direct subform resolved output equals its compact-without-params plus `get_resolved_layout_subform` merger, and metadata path remains structural.

- [ ] **Step 2: Run RED**

Run: `cargo test --test resolved_schema_parity_test -- --nocapture`

Expected: current test helper lacks recursive/subform parity behavior if needed, or test exposes actual mismatch.

- [ ] **Step 3: Make minimal production or helper correction**

Only change Rust production code if direct core subform parity differs. Otherwise update test helper only enough to model canonical existing behavior.

- [ ] **Step 4: Run GREEN**

Run same command. Expected: pass.

### Task 5: Cross-binding validation and review

**Files:** all changed files above.

- [ ] **Step 1: Run focused checks**

```bash
cargo fmt --check
cargo test --test resolved_schema_parity_test --quiet
cargo test --test test_subforms --quiet
yarn --cwd bindings/npm workspace @json-eval-rs/common build
node bindings/npm/packages/common/test/utils.test.mjs
yarn --cwd bindings/npm workspace @json-eval-rs/webcore build
node bindings/npm/packages/webcore/test/resolved-schema.test.mjs
yarn --cwd bindings/npm workspace @json-eval-rs/react-native typescript
yarn --cwd bindings/npm workspace @json-eval-rs/react-native test --runInBand resolved-schema.test.ts
dotnet build bindings/csharp/JsonEvalRs.csproj
git diff --check
```

- [ ] **Step 2: Read-only review**

Review all public root and subform resolved methods. Confirm each uses compact + overlay composition and no public binding calls native resolved methods.

## Validation

- Root resolution parity: compact-without-params + overlay equals direct Rust resolved schema.
- Subform resolution parity: same invariant for each subform API.
- Inline path exactness: `illustration.$layout.elements.1`.
- Empty overlays: properties still get all three metadata values.
- WebCore builds/tests pass; Node/Bundler/Vanilla inherit wrapper behavior.
- React Native type-check and Jest pass.
- C# test/build passes under supported target frameworks.
- `cargo fmt --check`, Rust targeted tests, and `git diff --check` pass.
