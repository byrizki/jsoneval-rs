use super::JSONEval;
use crate::jsoneval::path_utils;
use crate::jsoneval::types::{LayoutOverlayEntry, ResolvedLayoutResult};
use crate::time_block;

use indexmap::IndexMap;
use serde_json::Value;

use std::sync::Arc;

#[derive(Default, Clone)]
pub(crate) struct LayoutResolutionState {
    pub(crate) resolved: bool,
    pub(crate) cache: Option<Arc<Vec<LayoutOverlayEntry>>>,
    pub(crate) layout_hidden_refs: indexmap::IndexSet<String>,
    pub(crate) layout_visible_refs: indexmap::IndexSet<String>,
    pub(crate) layout_condition_hidden_refs: indexmap::IndexSet<String>,
    pub(crate) layout_disabled_refs: indexmap::IndexSet<String>,
}

impl JSONEval {
    /// Ensure layout references and visibility state are resolved and cached.
    pub(crate) fn ensure_layout_resolved(&self) {
        if self.layout_paths.is_empty() {
            return;
        }

        if let Ok(state) = self.layout_state.read() {
            if state.resolved {
                return;
            }
        }

        let mut state = match self.layout_state.write() {
            Ok(s) => s,
            Err(poisoned) => poisoned.into_inner(),
        };

        if state.resolved {
            return;
        }

        let entries = self.compute_layout_resolution(&mut state);
        state.cache = Some(Arc::new(entries));
        state.resolved = true;
    }

    /// Invalidate the layout resolution cache and state.
    pub(crate) fn invalidate_layout_cache(&self) {
        let mut state = match self.layout_state.write() {
            Ok(s) => s,
            Err(poisoned) => poisoned.into_inner(),
        };
        state.resolved = false;
        state.cache = None;
        state.layout_hidden_refs.clear();
        state.layout_visible_refs.clear();
        state.layout_condition_hidden_refs.clear();
        state.layout_disabled_refs.clear();
    }

    /// Resolve layout references, return overlay entries.
    ///
    /// Unlike old version: does NOT mutate evaluated_schema.
    /// Returns list of overlay entries describing delta properties per element.
    ///
    /// # Arguments
    ///
    /// * `evaluate` - If true, runs evaluation before resolving layout.
    pub fn resolve_layout(&mut self, evaluate: bool) -> Result<ResolvedLayoutResult, String> {
        if evaluate {
            let data_str = serde_json::to_string(&self.data)
                .map_err(|e| format!("Failed to serialize data: {}", e))?;
            self.evaluate(&data_str, None, None, None)?;
        }

        self.ensure_layout_resolved();
        let state = self.layout_state.read().unwrap();
        Ok(state
            .cache
            .as_ref()
            .map(|c| (**c).clone())
            .unwrap_or_default())
    }

    fn compute_layout_resolution(&self, state: &mut LayoutResolutionState) -> ResolvedLayoutResult {
        time_block!("  resolve_layout_internal()", {
            let mut all_entries = ResolvedLayoutResult::new();

            state.layout_hidden_refs.clear();
            state.layout_visible_refs.clear();
            state.layout_condition_hidden_refs.clear();
            state.layout_disabled_refs.clear();

            if self.root_layout_paths.is_empty() {
                return all_entries;
            }

            let mut ref_cache = std::collections::HashMap::new();
            time_block!("    resolve_layout_elements", {
                for layout_path in self.root_layout_paths.iter() {
                    let normalized_path = path_utils::normalize_to_json_pointer(layout_path);
                    if let Some(Value::Array(elements)) = self.schema.pointer(&normalized_path) {
                        self.resolve_and_collect_overlays(
                            elements,
                            layout_path,
                            false,
                            false,
                            false,
                            state,
                            &mut ref_cache,
                            &mut all_entries,
                        );
                    }
                }
            });

            for visible_ref in &state.layout_visible_refs {
                state.layout_hidden_refs.shift_remove(visible_ref);
                state.layout_condition_hidden_refs.shift_remove(visible_ref);
            }

            all_entries
        })
    }

    // ── Phase 1 helpers ─────────────────────────────────────────────

    /// Return schema pointer owning `.../$layout/elements`; root layouts have no owner.
    pub(crate) fn layout_owner_pointer(layout_path: &str) -> String {
        let owner = layout_path
            .trim_end_matches("/$layout/elements")
            .trim_start_matches('#');
        owner.to_string()
    }

    /// Compute root layout paths (layout paths not attached to another element).
    pub(crate) fn compute_root_layout_paths(
        layout_paths: &[String],
        schema: &Value,
    ) -> Vec<String> {
        let attached_layout_refs = Self::collect_layout_ref_targets(schema);
        layout_paths
            .iter()
            .filter(|path| {
                let owner = Self::layout_owner_pointer(path);
                owner.is_empty() || !attached_layout_refs.contains(&owner)
            })
            .cloned()
            .collect()
    }

    /// Collect schema targets referenced from layout elements only. Formula `$ref`s are
    /// intentionally ignored: they do not attach a field to a visual layout parent.
    pub(crate) fn collect_layout_ref_targets(schema: &Value) -> indexmap::IndexSet<String> {
        fn collect_elements(elements: &Value, refs: &mut indexmap::IndexSet<String>) {
            let Some(elements) = elements.as_array() else {
                return;
            };
            for element in elements {
                let Some(map) = element.as_object() else {
                    continue;
                };
                if let Some(reference) = map.get("$ref").and_then(Value::as_str) {
                    let pointer = path_utils::normalize_to_json_pointer(
                        &path_utils::dot_notation_to_schema_pointer(reference),
                    )
                    .trim_start_matches('#')
                    .to_string();
                    refs.insert(pointer);
                }
                if let Some(children) = map.get("elements") {
                    collect_elements(children, refs);
                }
            }
        }

        fn walk(value: &Value, refs: &mut indexmap::IndexSet<String>) {
            let Some(map) = value.as_object() else {
                return;
            };
            if let Some(elements) = map
                .get("$layout")
                .and_then(Value::as_object)
                .and_then(|layout| layout.get("elements"))
            {
                collect_elements(elements, refs);
            }
            for child in map.values() {
                walk(child, refs);
            }
        }

        let mut refs = indexmap::IndexSet::new();
        walk(schema, &mut refs);
        refs
    }

    /// Single-pass layout resolution and overlay collection.
    ///
    /// Resolves `$ref` references, cascades parent visibility and disabled state,
    /// emits flat `LayoutOverlayEntry` objects, and records hidden/visible references.
    fn resolve_and_collect_overlays(
        &self,
        elements: &[Value],
        layout_path: &str,
        parent_hidden: bool,
        parent_condition_hidden: bool,
        parent_disabled: bool,
        state: &mut LayoutResolutionState,
        ref_cache: &mut std::collections::HashMap<String, (String, String, String)>,
        all_entries: &mut Vec<LayoutOverlayEntry>,
    ) {
        for (idx, element) in elements.iter().enumerate() {
            let element_idx = idx;
            let (resolved, ref_path) = self.resolve_element_ref(element, ref_cache);
            let Value::Object(map) = resolved else {
                continue;
            };

            const EXCLUDED: &[&str] = &[
                "$ref",
                "elements",
                "properties",
                "items",
                "required",
                "additionalProperties",
            ];
            let mut overlay = IndexMap::new();
            for (key, value) in &map {
                if !EXCLUDED.contains(&key.as_str()) {
                    overlay.insert(key.clone(), value.clone());
                }
            }

            // Inject $fullpath for ALL elements (ref and non-ref)
            if !overlay.contains_key("$fullpath") {
                if !ref_path.is_empty() {
                    let last_segment = ref_path.split('.').last().unwrap_or(&ref_path);
                    overlay.insert("$fullpath".to_string(), Value::String(ref_path.clone()));
                    overlay.insert("$path".to_string(), Value::String(last_segment.to_string()));
                } else {
                    let base = Self::layout_path_to_structural_path(layout_path);
                    let fullpath = if base.is_empty() {
                        format!("{}", element_idx)
                    } else {
                        format!("{}.{}", base, element_idx)
                    };
                    let last_segment = fullpath.split('.').last().unwrap_or(&fullpath).to_string();
                    overlay.insert("$fullpath".to_string(), Value::String(fullpath));
                    overlay.insert("$path".to_string(), Value::String(last_segment));
                }
            }

            overlay.insert("$parentHide".to_string(), Value::Bool(parent_hidden));

            // Parent condition cascade
            let mut element_hidden = parent_hidden;
            let mut element_condition_hidden = parent_condition_hidden;
            let mut element_disabled = parent_disabled;

            if let Some(Value::Bool(d)) = overlay.get("disabled") {
                element_disabled = element_disabled || *d;
            }
            if let Some(Value::Bool(r)) = overlay.get("readonly") {
                element_disabled = element_disabled || *r;
            }
            if let Some(Value::Bool(r)) = overlay.get("readOnly") {
                element_disabled = element_disabled || *r;
            }

            if let Some(Value::Object(cond)) = overlay.get("condition") {
                if let Some(Value::Bool(true)) = cond.get("hidden") {
                    element_hidden = true;
                    element_condition_hidden = true;
                }
                if let Some(Value::Bool(d)) = cond.get("disabled") {
                    element_disabled = element_disabled || *d;
                }
                if let Some(Value::Bool(r)) = cond.get("readonly") {
                    element_disabled = element_disabled || *r;
                }
                if let Some(Value::Bool(r)) = cond.get("readOnly") {
                    element_disabled = element_disabled || *r;
                }
            }

            if let Some(Value::Object(hide)) = overlay.get("hideLayout") {
                if let Some(Value::Bool(true)) = hide.get("all") {
                    element_hidden = true;
                }
            }

            if !ref_path.is_empty() {
                let pointer = path_utils::normalize_to_json_pointer(
                    &path_utils::dot_notation_to_schema_pointer(&ref_path),
                )
                .trim_start_matches('#')
                .to_string();
                if element_hidden {
                    state.layout_hidden_refs.insert(pointer.clone());
                    if element_condition_hidden {
                        state.layout_condition_hidden_refs.insert(pointer.clone());
                    }
                } else {
                    state.layout_visible_refs.insert(pointer.clone());
                }
                if element_disabled {
                    state.layout_disabled_refs.insert(pointer);
                }
            }

            let show_condition_cascade =
                parent_hidden || parent_disabled || element_hidden || element_disabled;

            if show_condition_cascade {
                let mut merged_cond = serde_json::Map::new();
                if let Some(Value::Object(existing)) = overlay.get("condition") {
                    for (k, v) in existing.iter() {
                        merged_cond.insert(k.clone(), v.clone());
                    }
                }
                if parent_hidden || element_hidden {
                    merged_cond.insert("hidden".to_string(), Value::Bool(true));
                }
                if parent_disabled || element_disabled {
                    merged_cond.insert("disabled".to_string(), Value::Bool(true));
                }
                overlay.insert("condition".to_string(), Value::Object(merged_cond));

                if (parent_hidden || element_hidden)
                    && (map.get("hideLayout").is_some() || map.get("type").is_some())
                {
                    let mut hide_layout = if let Some(Value::Object(h)) = map.get("hideLayout") {
                        h.clone()
                    } else {
                        serde_json::Map::new()
                    };
                    hide_layout.insert("all".to_string(), Value::Bool(true));
                    overlay.insert("hideLayout".to_string(), Value::Object(hide_layout));
                }
            }

            // Recurse into nested elements (if any)
            if let Some(Value::Array(children)) = map.get("elements") {
                let child_layout_path = format!(
                    "{}/{}/elements",
                    layout_path.trim_end_matches('/'),
                    element_idx
                );
                self.resolve_and_collect_overlays(
                    children,
                    &child_layout_path,
                    element_hidden,
                    element_condition_hidden,
                    element_disabled,
                    state,
                    ref_cache,
                    all_entries,
                );
            }

            all_entries.push(LayoutOverlayEntry {
                layout_path: layout_path.to_string(),
                element_idx,
                schema_ref_path: ref_path,
                overlay,
            });
        }
    }

    /// Resolve $ref in a single element. Returns (resolved_element, schema_ref_path).
    /// Does NOT recurse into nested elements.
    fn resolve_element_ref(
        &self,
        element: &Value,
        ref_cache: &mut std::collections::HashMap<String, (String, String, String)>,
    ) -> (Value, String) {
        let Some(map) = element.as_object() else {
            return (element.clone(), String::new());
        };
        let mut map = map.clone();
        let has_ref = map.get("$ref").is_some();
        let ref_path = if has_ref {
            map.get("$ref")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        } else {
            String::new()
        };

        if let Some(Value::String(ref_str)) = map.get("$ref").cloned() {
            let (normalized_path, dotted_path, last_segment) =
                if let Some(cached) = ref_cache.get(&ref_str) {
                    cached.clone()
                } else {
                    let normalized_path = if ref_str.starts_with('#') || ref_str.starts_with('/') {
                        path_utils::normalize_to_json_pointer(&ref_str).into_owned()
                    } else {
                        let schema_pointer = path_utils::dot_notation_to_schema_pointer(&ref_str);
                        let schema_path =
                            path_utils::normalize_to_json_pointer(&schema_pointer).into_owned();

                        if self.evaluated_schema.pointer(&schema_path).is_some() {
                            schema_path
                        } else {
                            format!("/properties/{}", ref_str.replace('.', "/properties/"))
                        }
                    };

                    let dotted_path = path_utils::pointer_to_dot_notation(&normalized_path);
                    let last_segment =
                        dotted_path.split('.').last().unwrap_or(&dotted_path).to_string();
                    let entry = (normalized_path, dotted_path, last_segment);
                    ref_cache.insert(ref_str, entry.clone());
                    entry
                };

            map.insert("$fullpath".to_string(), Value::String(dotted_path.clone()));
            map.insert("$path".to_string(), Value::String(last_segment));
            map.insert("$parentHide".to_string(), Value::Bool(false));

            if let Some(referenced_value) = self.evaluated_schema.pointer(&normalized_path) {
                if let Value::Object(ref_map) = referenced_value {
                    map.remove("$ref");

                    let mut result =
                        if let Some(Value::Object(layout_obj)) = ref_map.get("$layout") {
                            layout_obj.clone()
                        } else {
                            serde_json::Map::new()
                        };

                    for (key, value) in ref_map {
                        if key == "$layout"
                            || key == "properties"
                            || key == "items"
                            || key == "required"
                            || key == "additionalProperties"
                        {
                            continue;
                        }
                        if key != "type" || !result.contains_key("type") {
                            result.insert(key.clone(), value.clone());
                        }
                    }

                    for (key, value) in map {
                        result.insert(key, value);
                    }
                    return (Value::Object(result), dotted_path);
                } else {
                    return (referenced_value.clone(), dotted_path);
                }
            }
        }

        (Value::Object(map), ref_path)
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Convert a layout elements pointer to its literal dotted structural path.
    ///
    /// ## Examples
    ///
    /// ```text
    /// "#/illustration/$layout/elements" → "illustration.$layout.elements"
    /// "#/properties/form/$layout/elements" → "properties.form.$layout.elements"
    /// ```
    fn layout_path_to_structural_path(layout_path: &str) -> String {
        layout_path
            .trim_start_matches('#')
            .trim_start_matches('/')
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join(".")
    }
}
