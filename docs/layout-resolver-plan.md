# Layout Resolver: Struct-Based Overlay Output

## Problem

`resolve_layout()` expands `$ref` inline into `evaluated_schema`. Every `$ref` copies the full referenced object. Schema grows 2-5x. Bigger FFI transfer, more GC pressure.

```json
// Before resolve: $layout.elements = [{ "$ref": "form.name" }]
// After resolve:  { "$ref": "form.name", "type": "string", "title": "Name",
//                    "condition": {...}, "$fullpath": "form.name", "$parentHide": false, ... }
//                 → full $ref target copied inline
```

## Solution: `LayoutOverlay` — Minimal Delta Output

Instead of mutating `evaluated_schema`, the resolver produces a list of **overlay entries**. Each entry says: "at this element in this layout array, here are the extra properties to apply." Consumer (JS UI) applies these overlays onto the compact schema.

### Output Struct

```rust
/// One resolved element in a layout tree.
/// Consumer takes this + the original $ref target from compact schema → merges overlay props.
#[derive(Serialize, Clone)]
struct LayoutOverlayEntry {
    /// Which $layout.elements array owns this entry
    /// e.g. "#/form/$layout/elements"
    pub layout_path: String,

    /// Index within layout_path's elements array
    pub element_idx: usize,

    /// Dotted path to the $ref target in schema (empty if no $ref)
    /// Consumer uses this to fetch field definition from compact schema
    pub schema_ref_path: String,

    /// OVERLAY properties — these are the DELTA:
    /// Properties that CANNOT be derived from compact schema alone.
    /// Consumer applies these on top of the $ref target's properties.
    pub overlay: IndexMap<String, Value>,
}
```

### What's in `overlay`

| Property                      | Source                          | Why overlay (not in compact schema)                |
| ----------------------------- | ------------------------------- | -------------------------------------------------- |
| `$fullpath`                   | `resolve_element_ref_recursive` | Derived from layout tree position, not schema      |
| `$path`                       | `resolve_element_ref_recursive` | Derived from layout tree position                  |
| `$parentHide`                 | `apply_parent_conditions`       | Computed at runtime via tree walk                  |
| `condition` (hidden/disabled) | `apply_parent_conditions`       | Merges element's own + inherited from parents      |
| `hideLayout` (`all:true`)     | `apply_parent_conditions`       | Injected by parent condition cascade               |
| Inline element overrides      | `resolve_element_ref`           | Properties from `{$ref: "...", "label": "Custom"}` |

### What is NOT in overlay (consumer gets from compact schema)

- `type`, `title`, `description`, `format` — from `$ref` target definition
- `properties`, `items` — schema structure
- `value`, `$evaluation`, `rules` — evaluation logic
- `condition.hidden` from the **original $ref element** (consumer can read from schema directly)

### Size Comparison

```
Current (inline expansion):
  $layout.elements[0] → { "type": "string", "title": "Name", "condition": {...},
                           "$fullpath": "form.name", "$path": "name", "$parentHide": false,
                           "label": "Custom Label" }
  ~200 bytes per element

Overlay (delta only):
  { "layout_path": "#/form/$layout/elements", "element_idx": 0,
    "schema_ref_path": "form.name",
    "overlay": { "$fullpath": "form.name", "$path": "name", "$parentHide": false,
                 "label": "Custom Label" } }
  ~100 bytes per element (50% smaller)
  Schema stays compact: form.name definition not duplicated
```

Bigger savings on repeated `$ref`:

```
10 elements all $ref → "form.name":
  Current: 10 × 200 = 2000 bytes (same object copied 10 times)
  Overlay: 1 × form.name definition (compact) + 10 × 100 = 1000 bytes
```

## Architecture

```
BEFORE:
                  resolve_layout()
  evaluated_schema ──────────────► evaluated_schema (mutated, bloated)
                                    ↓
                                  getEvaluatedSchema() → returns bloated JSON

AFTER:
                  resolve_layout()
  evaluated_schema ──────────────► Vec<LayoutOverlayEntry>  (new, no mutation)
                                    ↓
                                  getResolvedLayout() → returns overlay array
                                  evaluated_schema (unchanged, compact)
                                    ↓
                                  getEvaluatedSchema() → returns compact JSON
                                  getEvaluatedSchemaResolved() → compact + overlay[] pair
```

### Consumer Pattern (JS)

```ts
// Compact schema + overlay = fully resolved layout
const compact = await eval.getEvaluatedSchema(); // $ref intact, small
const overlays = await eval.getResolvedLayout(); // delta array

// Apply overlays to get what current resolve_layout produces:
function applyOverlays(schema: any, overlays: LayoutOverlayEntry[]) {
  for (const entry of overlays) {
    const layoutArr = getPointer(schema, entry.layout_path);
    const element = layoutArr[entry.element_idx];

    // Read target from compact schema via $ref
    const target = getByDottedPath(schema, entry.schema_ref_path);

    // Merge: element.$ref overrides > target properties > overlay.props
    const resolved = { ...target, ...element, ...entry.overlay };
    delete resolved.$ref;
    layoutArr[entry.element_idx] = resolved;
  }
}

// Or get pre-merged from binding:
const resolvedSchema = await eval.getEvaluatedSchemaResolved();
```

## Changes by File

### Rust — Core (`src/jsoneval/`)

| File                      | Change                                                                                                                                                                                                                                                        |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `layout.rs`               | Rewrite `resolve_layout()` to return `Vec<LayoutOverlayEntry>` instead of mutating `evaluated_schema`. All 4 sub-functions same logic, but output to overlay list. Remove `&mut self` → `&self`.                                                              |
| `mod.rs`                  | Remove `layout_paths` field from `JSONEval` struct (only needed for resolver internal use). Add `resolved_layout_cache: Option<Arc<Vec<LayoutOverlayEntry>>>` cleared on re-evaluate.                                                                         |
| `getters.rs`              | Remove `skip_layout` param from all `get_evaluated_schema*()` methods. Add `get_resolved_layout()` method that calls `resolve_layout()` once, caches. Add `get_evaluated_schema_resolved()` → returns `{ schema: Value, overlays: Vec<LayoutOverlayEntry> }`. |
| `subform_methods.rs`      | Same treatment for subform variants. `resolve_layout_subform` returns overlay list. `get_evaluated_schema_subform` no longer takes `resolve_layout` param. Add `get_resolved_layout_subform`.                                                                 |
| `core.rs` / `evaluate.rs` | On re-evaluate + new schema walk → clear `resolved_layout_cache`.                                                                                                                                                                                             |

### Rust — New type (`src/jsoneval/types.rs`)

```rust
#[derive(Serialize, Clone)]
pub struct LayoutOverlayEntry {
    pub layout_path: String,
    pub element_idx: usize,
    pub schema_ref_path: String,
    pub overlay: IndexMap<String, Value>,
}

// Return type
pub type ResolvedLayoutResult = Vec<LayoutOverlayEntry>;
```

### Rust — FFI (`src/ffi/`)

| File        | Change                                                                                                                                                                                                                                                             |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `layout.rs` | Remove `json_eval_resolve_layout` (mutates nothing). Keep `json_eval_validate_paths`.                                                                                                                                                                              |
| `schema.rs` | Remove `skip_layout` param from all `get_evaluated_schema*()` FFI functions. Add `json_eval_get_resolved_layout()` → returns serialized `Vec<LayoutOverlayEntry>`. Add `json_eval_get_evaluated_schema_resolved()` → returns `{ schema: bytes, overlays: bytes }`. |

### Rust — WASM (`src/wasm/`)

| File          | Change                                                                                                                                                                             |
| ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `layout.rs`   | Remove `resolveLayout()`.                                                                                                                                                          |
| `schema.rs`   | Remove `skipLayout` from all `getEvaluatedSchema*()` JS methods. Add `getResolvedLayoutJS()` returning `JsValue` (array of overlay objects). Add `getEvaluatedSchemaResolvedJS()`. |
| `subforms.rs` | Same changes for subform methods.                                                                                                                                                  |

### Bindings — `@json-eval-rs/common`

| File                        | Change                                                                                                                                                                             |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/types.ts`              | Add `LayoutOverlayEntry` interface. Remove `GetEvaluatedSchemaOptions.skipLayout`, `ResolveLayoutSubformOptions`. Add `ResolvedLayoutResult`, `GetEvaluatedSchemaResolvedOptions`. |
| `src/index.ts`              | Re-export new types + `applyLayoutOverlays`, `getByDottedPath`, `setPointer`.                                                                                                      |
| **`src/layout-applier.ts`** | NEW — Pure TS helper functions for merging overlay deltas. See below.                                                                                                               |

### Bindings — Web (`bindings/npm/packages/webcore/src/index.ts`)

| File       | Change                                                                                                                                                                                               |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `index.ts` | Remove `resolveLayout()`. Remove `skipLayout` from all `getEvaluatedSchema*()`. Add `getResolvedLayout()` returning overlay array. Add `getEvaluatedSchemaResolved()` → merges overlays and returns. |

### Bindings — React Native

| File            | Change                                                                                                                             |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `jsi-bridge.ts` | Remove `resolveLayout()`. Remove `skipLayout` from schema getter types. Add `getResolvedLayout()`, `getEvaluatedSchemaResolved()`. |
| `index.tsx`     | Same API surface changes as Web.                                                                                                   |

### Bindings — C#

| File                 | Change                                                                                                                                                        |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `JsonEvalRs.Main.cs` | Remove `ResolveLayout()`. Remove `skipLayout` from `GetEvaluatedSchema*()`. Add `GetResolvedLayout()` returning `JArray`. Add `GetEvaluatedSchemaResolved()`. |

## Edge Cases

### 1. `sync_layout_hidden_to_schema`

Currently writes `condition.hidden: true` back into `evaluated_schema` fields. This is needed for validation to skip hidden fields.

**New approach**: Remove schema mutation. `is_effective_hidden()` checks overlay entries instead. Or validation reads from resolved layout overlay entries to determine hidden state.

Option: Validation iterates overlay entries, collects `schema_ref_path` values where overlay has `condition.hidden: true`, skips those fields.

### 2. `propagate_parent_conditions`

Parent cascade currently mutates child elements in `evaluated_schema`. With overlays, each level's condition merge is captured as overlay properties:

```
Layout: group → nested group → field
Each level adds to overlay:
  element[0] (group):     { overlay: { condition: {hidden: false} } }
  element[0].elements[0](field): { overlay: { $parentHide: false, condition: {hidden: false} } }
```

The overlay is computed per-element, not per-level. `propagate_parent_conditions` is still done in Rust, but writes to overlay entries instead of mutating schema.

### 3. Nested `elements` arrays

Current resolver recursively resolves `elements` arrays inside `$layout` elements. Overlay entries flatten this hierarchy:

```
Input:  #/form/$layout/elements → [{ "$ref": "group", "elements": [{"$ref": "name"}] }]
Output: [
  { layout_path: "#/form/$layout/elements", idx: 0, schema_ref_path: "form.group", overlay: {...} },
  { layout_path: "#/form/$layout/elements", idx: 0, elements: [
    { layout_path: "#/form/$layout/elements/0/elements", idx: 0, schema_ref_path: "form.name", overlay: {...} }
  ]}
]
```

Consumer applies overlays bottom-up (inner elements first).

## Migration

Backward compatible at the consumer level:

```ts
// Old API — still works, just returns compact + overlays merged internally
// eval.getEvaluatedSchema({ skipLayout: false }) → new eval.getEvaluatedSchemaResolved()

// New API — explicit
const compact = eval.getEvaluatedSchema();
const overlays = eval.getResolvedLayout();
const full = eval.getEvaluatedSchemaResolved(); // convenience: compact + merged overlays
```

For consumers that don't use layout at all: **nothing changes**. `getEvaluatedSchema()` just returns compact schema instead of bloated one (major perf win for free).

---

## `@json-eval-rs/common` — `layout-applier.ts` Detail

New file in `bindings/npm/packages/common/src/layout-applier.ts`. Pure TS, zero dependencies. Exports 3 functions:

### 1. `applyLayoutOverlays(schema, overlays) → schema`

Main consumer API. Takes compact schema + overlay array → returns schema with resolved layout.
Deep-clones schema first (no mutation of input).

```ts
import { LayoutOverlayEntry } from './types.js';

/**
 * Apply layout overlay entries to a compact evaluated schema.
 * Returns a new schema object (does not mutate input).
 *
 * For each overlay entry:
 *  1. Walk to the layout elements array at `layout_path`
 *  2. Get the element at `element_idx`
 *  3. Resolve $ref: fetch target definition from schema via `schema_ref_path`
 *  4. Merge: target definition + element overrides + overlay properties
 *  5. Remove $ref from result
 *  6. Write merged element back into the cloned schema
 *
 * Nested elements (elements inside elements) are handled by
 * overlay entries whose layout_path includes the parent index path.
 */
export function applyLayoutOverlays(
  schema: Record<string, any>,
  overlays: LayoutOverlayEntry[]
): Record<string, any> {
  if (!overlays.length) return schema;

  const result = structuredClone(schema);

  // Sort overlays bottom-up (deeper paths first) so inner elements resolve before outer
  const sorted = [...overlays].sort((a, b) =>
    b.layout_path.split('/').length - a.layout_path.split('/').length
  );

  for (const entry of sorted) {
    const arr = getPointer(result, entry.layout_path);
    if (!Array.isArray(arr)) continue;

    const element = arr[entry.element_idx];
    if (!element) continue;

    // Resolve $ref target from compact schema
    let resolved: Record<string, any>;
    if (entry.schema_ref_path && element.$ref) {
      const target = getByDottedPath(result, entry.schema_ref_path);
      if (target && typeof target === 'object') {
        // If target has $layout, flatten it (same as Rust resolve_element_ref)
        const { $layout, ...rest } = target;
        if ($layout && typeof $layout === 'object') {
          resolved = { ...$layout };
          // Merge non-type from rest, then overlay
          for (const [k, v] of Object.entries(rest)) {
            if (k !== 'type' || !('type' in resolved)) resolved[k] = v;
          }
        } else {
          resolved = { ...rest };
        }
      } else {
        resolved = { ...element };
      }
    } else {
      resolved = { ...element };
    }

    // Apply overlay delta properties
    for (const [key, value] of Object.entries(entry.overlay)) {
      resolved[key] = value;
    }

    // Remove $ref from final output
    delete resolved.$ref;

    arr[entry.element_idx] = resolved;
  }

  return result;
}
```

### 2. `getByDottedPath(schema, dottedPath) → value`

Utility: walk schema using dotted notation (e.g. `"form.name"` → `schema.properties.form.properties.name`).
Mirrors Rust `path_utils::pointer_to_dot_notation` in reverse.

```ts
/**
 * Get a value from schema by dotted path notation.
 * "form.name" → schema.properties.form.properties.name
 * Returns undefined if path doesn't exist.
 */
export function getByDottedPath(
  schema: Record<string, any>,
  dottedPath: string
): any {
  const parts = dottedPath.split('.');
  let current: any = schema;
  for (const part of parts) {
    if (current == null || typeof current !== 'object') return undefined;
    current = current.properties?.[part] ?? current[part];
  }
  return current;
}
```

### 3. `getPointer(obj, jsonPointer) → value`

Utility: walk object using JSON pointer (e.g. `"#/form/$layout/elements"` → `obj.form.$layout.elements`).
Mirrors Rust `path_utils::normalize_to_json_pointer`.

```ts
/**
 * Get a value from an object by JSON pointer path.
 * "#/form/$layout/elements" → obj["form"]["$layout"]["elements"]
 * Returns undefined if path doesn't exist.
 */
export function getPointer(
  obj: Record<string, any>,
  pointer: string
): any {
  const parts = pointer
    .replace(/^#\/?/, '')  // strip #/ prefix
    .split('/');
  let current: any = obj;
  for (const part of parts) {
    if (current == null) return undefined;
    current = current[part];
  }
  return current;
}
```

### `types.ts` additions

```ts
/**
 * One resolved element overlay produced by the Rust layout resolver.
 * Each entry describes properties to apply on top of the compact schema.
 */
export interface LayoutOverlayEntry {
  /** Which $layout.elements array (e.g. "#/form/$layout/elements") */
  layout_path: string;
  /** Index within that elements array */
  element_idx: number;
  /** Dotted path to $ref target in schema (empty string if no $ref) */
  schema_ref_path: string;
  /** Delta properties to overlay onto the element */
  overlay: Record<string, any>;
}

/** Result of getResolvedLayout() */
export type ResolvedLayoutResult = LayoutOverlayEntry[];
```

### `index.ts` exports update

```ts
// Add to re-exports:
export type { LayoutOverlayEntry, ResolvedLayoutResult } from './types.js';
export { applyLayoutOverlays, getByDottedPath, getPointer } from './layout-applier.js';
```

---

## Implementation Order

1. Define `LayoutOverlayEntry` + `ResolvedLayoutResult` in `src/jsoneval/types.rs`
2. Rewrite `src/jsoneval/layout.rs` — output `Vec<LayoutOverlayEntry>` instead of mutating schema
3. Add `resolved_layout_cache` to `JSONEval` struct in `mod.rs`
4. Update `getters.rs` — remove `skip_layout`, add `get_resolved_layout()`, add `get_evaluated_schema_resolved()`
5. Update `subform_methods.rs` — same
6. Update WASM bindings
7. Update FFI bindings
8. Update C# bindings
9. Update `@json-eval-rs/common` types
10. Add `layout-applier.ts` to common (optional consumer helper)
11. Update Web and RN bindings
12. Tests
