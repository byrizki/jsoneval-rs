/**
 * @json-eval-rs/common
 * Shared utility functions for Web and React Native bindings.
 */

/**
 * Convert a value to JSON string.
 * If already a string, returns as-is.
 * Otherwise serializes with JSON.stringify.
 */
export function stringifyValue(value: string | object): string {
  return typeof value === 'string' ? value : JSON.stringify(value);
}

/**
 * Parse a JSON string into a value.
 * If the input is not a string (already an object/array/primitive), returns it as-is.
 */
export function parseValue(value: unknown): any {
  return typeof value === 'string' ? JSON.parse(value) : value;
}

/**
 * Serialize a value to JSON string, or return null if null/undefined.
 */
export function stringifyOrNull(value: any): string | null {
  if (value == null) return null;
  return typeof value === 'string' ? value : JSON.stringify(value);
}

/**
 * Extract error message from unknown error.
 */
export function extractErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

// ============================================================================
// Layout overlay helpers
// ============================================================================

import type { LayoutOverlayEntry } from './types';

// ─── Private pointer helpers ─────────────────────────────────────────────────

/**
 * Get a nested value from an object using a slash-separated pointer path.
 * Handles `#/foo/bar`, `/foo/bar`, and bare `foo/bar` forms.
 */
function getByPointer(obj: any, pointer: string): any {
  if (pointer === '' || pointer === '#') return obj;
  const path = pointer.startsWith('#/') ? pointer.slice(2) : pointer.startsWith('/') ? pointer.slice(1) : pointer;
  const parts = path.split('/');
  let current: any = obj;
  for (const part of parts) {
    if (current == null) return undefined;
    current = current[part];
  }
  return current;
}

/**
 * Set a nested value in an object using a slash-separated pointer path (mutates).
 * Creates intermediate objects as needed.
 */
function setByPointer(obj: any, pointer: string, value: any): void {
  if (pointer === '' || pointer === '#') {
    return; // Cannot set root; caller should replace the reference directly
  }
  const path = pointer.startsWith('#/') ? pointer.slice(2) : pointer.startsWith('/') ? pointer.slice(1) : pointer;
  const parts = path.split('/');
  let current: any = obj;
  for (let i = 0; i < parts.length - 1; i++) {
    if (current[parts[i]] == null) current[parts[i]] = {};
    current = current[parts[i]];
  }
  current[parts[parts.length - 1]] = value;
}

/**
 * Set a nested value using dot notation (mutates).
 * Creates intermediate objects as needed.
 */
function setByDottedPath(obj: any, dottedPath: string, value: any): void {
  const parts = dottedPath.split('.');
  let current: any = obj;
  for (let i = 0; i < parts.length - 1; i++) {
    if (current[parts[i]] == null) current[parts[i]] = {};
    current = current[parts[i]];
  }
  current[parts[parts.length - 1]] = value;
}

// ─── Port of path_utils::normalize_to_json_pointer ───────────────────────────

/**
 * Normalise any path variant to a JSON-pointer starting with `/`.
 * Mirrors `path_utils::normalize_to_json_pointer` in Rust exactly.
 *
 * - `#/foo/bar`  → `/foo/bar`
 * - `foo.bar`    → `/foo/bar`   (dots become slashes for non-`#`/`/` paths)
 * - `/foo/bar`   → `/foo/bar`   (already normalised)
 * - `#` / ``     → ``           (root)
 */
function normalizeToJsonPointer(path: string): string {
  if (path === '') return '';

  if (path.startsWith('#/') && !path.includes('//')) {
    return path.slice(1);
  }

  if (path.startsWith('/') && !path.includes('//')) {
    return path === '/' ? '' : path;
  }

  const source = path.startsWith('#/') ? path.slice(1) : path;
  const shouldConvertDots = !path.startsWith('/') && !path.startsWith('#');

  let normalized = path.startsWith('/') || path.startsWith('#/') ? '' : '/';
  let prevSlash = normalized === '/';

  for (const ch of source) {
    const c = shouldConvertDots && ch === '.' ? '/' : ch;
    if (c === '/') {
      if (!prevSlash) normalized += '/';
      prevSlash = true;
    } else {
      normalized += c;
      prevSlash = false;
    }
  }

  return normalized === '/' ? '' : normalized;
}

// ─── Port of path_utils::dot_notation_to_schema_pointer ──────────────────────

/**
 * Convert a dotted schema path to a JSON Schema pointer.
 * Mirrors `path_utils::dot_notation_to_schema_pointer` in Rust exactly.
 *
 * - `"illustration.insured.name"` → `"#/illustration/properties/insured/properties/name"`
 * - `"#/already/formatted"`       → `"#/already/formatted"` (no change)
 * - `"properties.foo.bar"`        → `"#/properties/foo/bar"` (explicit properties prefix)
 */
function dotNotationToSchemaPointer(path: string): string {
  if (path.startsWith('#') || path.startsWith('/')) return path;

  if (path.startsWith('properties.') || path.includes('.properties.')) {
    return '#/' + path.replace(/\./g, '/');
  }

  const parts = path.split('.');
  if (parts.length === 0) return '#/';

  let result = '#';
  for (let i = 0; i < parts.length; i++) {
    if (parts[i] === 'properties') continue;
    if (i > 0 && !path.startsWith('$')) result += '/properties';
    result += '/' + parts[i];
  }
  return result;
}

// ─── $ref pointer resolution (mirrors Rust Phase 2 inline logic) ─────────────

/**
 * Resolve a `$ref` string to a normalised JSON pointer and verify it exists
 * in `schema`. Returns the pointer string, or `null` if unresolvable.
 *
 * Mirrors the `$ref`-resolution block inside `get_evaluated_schema_resolved` Phase 2.
 */
function resolveRefPointer(schema: any, refStr: string): string | null {
  let pointer: string;

  if (refStr.startsWith('#') || refStr.startsWith('/')) {
    pointer = normalizeToJsonPointer(refStr);
  } else {
    const normalized = normalizeToJsonPointer(dotNotationToSchemaPointer(refStr));
    if (getByPointer(schema, normalized) !== undefined) {
      pointer = normalized;
    } else {
      pointer = '/properties/' + refStr.replace(/\./g, '/properties/');
    }
  }

  return getByPointer(schema, pointer) !== undefined ? pointer : null;
}

// ─── Public API ──────────────────────────────────────────────────────────────

/**
 * Merge layout overlay entries into an evaluated schema, exactly mirroring
 * the Rust `get_evaluated_schema_resolved` Phase 2 logic.
 *
 * **Algorithm** (matches Rust step-for-step):
 * 1. Sort entries shallowest `layout_path` first so parents are expanded
 *    before child entries that navigate through the nested `elements` array.
 * 2. For each entry, operating on the **already-mutated** schema:
 *    a. Read `$ref` from the live element at `layout_path[element_idx]`.
 *    b. Resolve the `$ref` against the current schema state.
 *    c. [Dropped] Clone the resolved node; if it has a `$layout` key, flatten it:
 *       `$layout` becomes the base object, non-`type` fields from the resolved
 *       node are merged on top (matching Rust priority).
 *    d. Strip `$ref` from the original element and merge its remaining keys
 *       into the resolved object (element keys win over resolved).
 *    e. Replace the element in-place with the merged object.
 *    f. Apply `entry.overlay` properties on top of the replaced element.
 *
 * @param schema - Evaluated schema with unresolved `$layout` (mutated in-place)
 * @param overlayEntries - Entries returned by `getResolvedLayout()`
 * @returns The schema with all layout overlays applied
 */
export function mergeLayoutOverlay(
  schema: any,
  overlayEntries: LayoutOverlayEntry[],
): any {
  if (!schema || !Array.isArray(overlayEntries) || overlayEntries.length === 0) {
    return schema;
  }

  // Step 1: Sort parent-first by slash-depth of layout_path, then element_idx
  const sorted = [...overlayEntries].sort((a, b) => {
    const da = (a.layout_path.match(/\//g) ?? []).length;
    const db = (b.layout_path.match(/\//g) ?? []).length;
    return da !== db ? da - db : a.element_idx - b.element_idx;
  });

  // Step 2: Process each entry against the live schema
  for (const entry of sorted) {
    const layoutPointer = normalizeToJsonPointer(entry.layout_path);
    const elements: any[] | undefined = getByPointer(schema, layoutPointer);
    if (!Array.isArray(elements) || entry.element_idx >= elements.length) continue;

    const element = elements[entry.element_idx];
    if (element == null || typeof element !== 'object') continue;

    // 2a-e: Resolve $ref and replace element
    const refStr: string | undefined = element.$ref;
    if (refStr) {
      const refPointer = resolveRefPointer(schema, refStr);
      if (refPointer !== null) {
        let resolved: any = getByPointer(schema, refPointer);

        // 2c: Flatten $layout — $layout becomes base, resolved non-type fields merge in
        if (resolved != null && typeof resolved === 'object' && resolved.$layout != null && typeof resolved.$layout === 'object') {
          const base: Record<string, any> = { ...resolved.$layout };
          for (const [k, v] of Object.entries(resolved)) {
            if (k === '$layout') continue;
            if (k === 'type' && base.type !== undefined) continue;
            base[k] = v;
          }
          resolved = base;
        }

        // 2d: Merge element's own keys (except $ref) into resolved
        if (resolved != null && typeof resolved === 'object') {
          for (const [k, v] of Object.entries(element)) {
            if (k === '$ref') continue;
            resolved[k] = v;
          }
          elements[entry.element_idx] = resolved;
        } else {
          elements[entry.element_idx] = resolved;
        }
      }
    }

    // 2f: Apply overlay on top of the (now-resolved) element
    const target = elements[entry.element_idx];
    if (target != null && typeof target === 'object' && entry.overlay != null) {
      for (const [k, v] of Object.entries(entry.overlay)) {
        target[k] = v;
      }
    }
  }

  return schema;
}

/**
 * Convenience: get evaluated schema with layout fully resolved in one call.
 *
 * @param getSchema - Async function returning the compact evaluated schema
 * @param getOverlay - Async function returning the layout overlay entries
 * @returns Schema with layout overlays applied
 */
export async function resolveEvaluatedLayout(
  getSchema: () => Promise<any>,
  getOverlay: () => Promise<LayoutOverlayEntry[]>,
): Promise<any> {
  const [schema, overlayEntries] = await Promise.all([getSchema(), getOverlay()]);
  return mergeLayoutOverlay(schema, overlayEntries);
}
