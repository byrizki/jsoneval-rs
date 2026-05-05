use super::JSONEval;
use crate::jsoneval::path_utils;
use crate::jsoneval::types::{LayoutOverlayEntry, ResolvedLayoutResult};
use crate::time_block;

use indexmap::IndexMap;
use serde_json::Value;

impl JSONEval {
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

        Ok(self.resolve_layout_internal())
    }

    fn resolve_layout_internal(&mut self) -> ResolvedLayoutResult {
        time_block!("  resolve_layout_internal()", {
            let layout_paths = self.layout_paths.clone();
            let mut all_entries = ResolvedLayoutResult::new();

            // Phase 1: Collect resolved elements (tree) for each layout path
            // Stores in temp Vec so we don't mutate evaluated_schema
            time_block!("    resolve_layout_elements", {
                for layout_path in layout_paths.iter() {
                    let resolved_tree = self.resolve_elements_tree(layout_path);
                    let entries = Self::tree_to_overlays(&resolved_tree, layout_path, false, false);
                    all_entries.extend(entries);
                }
            });

            // Phase 2: Hidden paths from parent cascade → schema definitions
            time_block!("    sync_layout_hidden_to_schema", {
                self.sync_layout_hidden_to_schema(&all_entries);
            });

            all_entries
        })
    }

    // ── Phase 1 helpers ─────────────────────────────────────────────

    /// Resolve $ref in elements tree, return full resolved tree (no parent cascade yet).
    /// Returns Vec<(resolved_element, schema_ref_path)> — one per element at this level.
    fn resolve_elements_tree(&self, layout_elements_path: &str) -> Vec<(Value, String)> {
        let normalized_path = path_utils::normalize_to_json_pointer(layout_elements_path);

        let elements = if let Some(Value::Array(arr)) = self.schema.pointer(&normalized_path) {
            arr.clone()
        } else {
            return Vec::new();
        };

        let mut result = Vec::with_capacity(elements.len());
        for element in elements.into_iter() {
            let (resolved, schema_ref) = self.resolve_element_ref_recursive(element, "");
            result.push((resolved, schema_ref));
        }
        result
    }

    /// Resolve an element's $ref recursively, returning (resolved_element, schema_ref_path).
    /// schema_ref_path is the outermost $ref target (empty if no $ref).
    ///
    /// The resolved element is a "full tree" — $ref expanded, metadata injected,
    /// nested elements arrays also resolved. Used for parent condition cascade.
    fn resolve_element_ref_recursive(&self, element: Value, _path_context: &str) -> (Value, String) {
        // Resolve this element's $ref
        let (mut resolved, ref_path) = self.resolve_element_ref(element);

        // Recursively resolve nested elements
        if let Value::Object(ref mut map) = resolved {
            if let Some(Value::Array(elements)) = map.get("elements").cloned() {
                let mut resolved_nested = Vec::with_capacity(elements.len());
                for nested_element in elements.into_iter() {
                    let (nested_resolved, _) =
                        self.resolve_element_ref_recursive(nested_element, "");
                    resolved_nested.push(nested_resolved);
                }
                map.insert("elements".to_string(), Value::Array(resolved_nested));
            }
        }

        (resolved, ref_path)
    }

    /// Resolve $ref in a single element. Returns (resolved_element, schema_ref_path).
    /// Does NOT recurse into nested elements.
    fn resolve_element_ref(&self, element: Value) -> (Value, String) {
        match element {
            Value::Object(mut map) => {
                let has_ref = map.get("$ref").is_some();
                let ref_path = if has_ref {
                    map.get("$ref").and_then(Value::as_str).unwrap_or("").to_string()
                } else {
                    String::new()
                };

                // Only inject metadata for elements with $ref
                if let Some(Value::String(ref_str)) = map.get("$ref").cloned() {
                    let dotted_path = path_utils::pointer_to_dot_notation(&ref_str);
                    let last_segment = dotted_path.split('.').last().unwrap_or(&dotted_path);

                    // Inject metadata
                    map.insert("$fullpath".to_string(), Value::String(dotted_path.clone()));
                    map.insert("$path".to_string(), Value::String(last_segment.to_string()));
                    map.insert("$parentHide".to_string(), Value::Bool(false));

                    let normalized_path = if ref_str.starts_with('#') || ref_str.starts_with('/')
                    {
                        path_utils::normalize_to_json_pointer(&ref_str).into_owned()
                    } else {
                        let schema_pointer =
                            path_utils::dot_notation_to_schema_pointer(&ref_str);
                        let schema_path =
                            path_utils::normalize_to_json_pointer(&schema_pointer).into_owned();

                        if self.evaluated_schema.pointer(&schema_path).is_some() {
                            schema_path
                        } else {
                            format!("/properties/{}", ref_str.replace('.', "/properties/"))
                        }
                    };

                    if let Some(referenced_value) = self.evaluated_schema.pointer(&normalized_path)
                    {
                        let resolved = referenced_value.clone();

                        if let Value::Object(mut resolved_map) = resolved {
                            map.remove("$ref");

                            if let Some(Value::Object(layout_obj)) = resolved_map.remove("$layout")
                            {
                                // Flatten $layout: start with layout_obj, merge rest from resolved_map
                                let mut result = layout_obj.clone();
                                for (key, value) in resolved_map {
                                    if key != "type" || !result.contains_key("type") {
                                        result.insert(key, value);
                                    }
                                }
                                // Element overrides on top
                                for (key, value) in map {
                                    result.insert(key, value);
                                }
                                return (Value::Object(result), dotted_path);
                            } else {
                                for (key, value) in map {
                                    resolved_map.insert(key, value);
                                }
                                return (Value::Object(resolved_map), dotted_path);
                            }
                        } else {
                            return (resolved, dotted_path);
                        }
                    }
                }

                (Value::Object(map), ref_path)
            }
            _ => (element, String::new()),
        }
    }

    /// Convert a resolved elements tree into flat overlay entries with parent condition cascade.
    ///
    /// Walks the tree recursively. At each level:
    ///   1. Determine parent state (hidden/disabled propagated from above)
    ///   2. Merge element's own condition with inherited state
    ///   3. Emit one LayoutOverlayEntry per element with full overlay delta
    ///
    /// The `base_ref_path` is the parent's `schema_ref_path` (empty for root).
    fn tree_to_overlays(
        tree: &[(Value, String)],
        layout_path: &str,
        parent_hidden: bool,
        parent_disabled: bool,
    ) -> ResolvedLayoutResult {
        let mut entries = ResolvedLayoutResult::new();

        for (idx, (element, ref_path)) in tree.iter().enumerate() {
            let element_idx = idx;
            let mut overlay = IndexMap::new();

            // ── Build thin overlay: only delta props (skip structural keys) ──
            if let Value::Object(map) = element {
                // Keys that should NOT appear in overlay:
                // - $ref: handled by compact schema
                // - elements: handled by child overlays
                // - properties/items/required: schema structure, not layout
                const EXCLUDED: &[&str] = &["$ref", "elements", "properties", "items", "required", "additionalProperties"];
                for (key, value) in map {
                    if !EXCLUDED.contains(&key.as_str()) {
                        overlay.insert(key.clone(), value.clone());
                    }
                }

                // ── Inject metadata if not present ──
                if !overlay.contains_key("$fullpath") {
                    // For direct elements without $ref, compute $fullpath and $path
                    if !ref_path.is_empty() {
                        // Has $ref, use dotted path from reference
                        let last_segment = ref_path.split('.').last().unwrap_or(&ref_path);
                        overlay.insert("$fullpath".to_string(), Value::String(ref_path.clone()));
                        overlay.insert("$path".to_string(), Value::String(last_segment.to_string()));
                    } else {
                        // Direct element without $ref, compute based on position
                        let idx = element_idx.to_string();
                        
                        // Build $fullpath from layout path + index using existing path utils
                        // layout_path is like "#/properties/form/$layout/elements"
                        // Get dotted form: "properties.form.$layout.elements"
                        let dotted_path = path_utils::pointer_to_dot_notation(layout_path);
                        
                        // Append index: "properties.form.$layout.elements.0"
                        let fullpath = format!("{}.{}", dotted_path, idx);
                        overlay.insert("$fullpath".to_string(), Value::String(fullpath));
                        overlay.insert("$path".to_string(), Value::String(idx));
                    }
                }
                
                // Override $parentHide — may exist from ref resolution as false,
                // but tree_to_overlays owns the correct parent_hidden value.
                overlay.insert("$parentHide".to_string(), Value::Bool(parent_hidden));

                // ── Parent condition cascade ──
                let mut element_hidden = parent_hidden;
                let mut element_disabled = parent_disabled;

                // Element's own condition (from overlay now)
                if let Some(Value::Object(cond)) = overlay.get("condition") {
                    if let Some(Value::Bool(h)) = cond.get("hidden") {
                        element_hidden = element_hidden || *h;
                    }
                    if let Some(Value::Bool(d)) = cond.get("disabled") {
                        element_disabled = element_disabled || *d;
                    }
                }

                // Element's own hideLayout (from overlay now)
                if let Some(Value::Object(hide)) = overlay.get("hideLayout") {
                    if let Some(Value::Bool(all)) = hide.get("all") {
                        if *all {
                            element_hidden = true;
                        }
                    }
                }
                
                // Only show condition cascade if parent has state OR element has state
                let show_condition_cascade = parent_hidden || parent_disabled || element_hidden || element_disabled;

                // $parentHide already set above

                // ── Condition cascade: if parent OR element has state, emit merged condition ──
                if show_condition_cascade {
                    let mut merged_cond = serde_json::Map::new();
                    if let Some(Value::Object(existing)) = overlay.get("condition") {
                        for (k, v) in existing.iter() {
                            merged_cond.insert(k.clone(), v.clone());
                        }
                    }
                    // Merge in hidden states from parent and element
                    if parent_hidden || element_hidden {
                        merged_cond.insert("hidden".to_string(), Value::Bool(true));
                    }
                    if parent_disabled || element_disabled {
                        merged_cond.insert("disabled".to_string(), Value::Bool(true));
                    }
                    overlay.insert("condition".to_string(), Value::Object(merged_cond));

                    // Also push hideLayout cascade for layout containers
                    if (parent_hidden || element_hidden)
                        && (element.get("hideLayout").is_some() || element.get("type").is_some())
                    {
                        let mut hide_layout = if let Some(Value::Object(h)) = element.get("hideLayout")
                        {
                            h.clone()
                        } else {
                            serde_json::Map::new()
                        };
                        hide_layout.insert("all".to_string(), Value::Bool(true));
                        overlay.insert("hideLayout".to_string(), Value::Object(hide_layout));
                    }
                }

                // ── Recurse into nested elements ──
                if let Some(Value::Array(children)) = element.get("elements") {
                    // Build nested tree from children
                    let child_tree: Vec<(Value, String)> = children
                        .iter()
                        .map(|c| {
                            if let Value::Object(m) = c {
                                (c.clone(),
                                    m.get("$fullpath")
                                        .and_then(Value::as_str)
                                        .unwrap_or("")
                                        .to_string(),
                                )
                            } else {
                                (c.clone(), String::new())
                            }
                        })
                        .collect();
                    
                    let child_layout_path = format!("{}/{}/elements", layout_path.trim_end_matches('/'), element_idx);
                    
                    // Recurse into children with element_hidden/element_disabled as parent state
                    let child_overlays = Self::tree_to_overlays(
                        &child_tree,
                        &child_layout_path,
                        element_hidden,
                        element_disabled,
                    );
                    entries.extend(child_overlays);
                }
            }

            entries.push(LayoutOverlayEntry {
                layout_path: layout_path.to_string(),
                element_idx,
                schema_ref_path: ref_path.clone(),
                overlay,
            });
        }

        entries
    }

    // ── Phase 2: Schema hidden sync ──

    /// Sync hidden state from overlay entries to evaluated_schema definitions.
    /// Collects schema paths where overlay has condition.hidden: true,
    /// writes condition.hidden = true to those definitions in evaluated_schema.
    fn sync_layout_hidden_to_schema(&mut self, entries: &[LayoutOverlayEntry]) {
        self.layout_synced_paths.clear();

        for entry in entries {
            if let Some(Value::Object(cond)) = entry.overlay.get("condition") {
                if cond
                    .get("hidden")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                    && !entry.schema_ref_path.is_empty()
                {
                    self.sync_hidden_to_schema_at(&entry.schema_ref_path);
                }
            }
            // Also track hideLayout cascade
            if let Some(Value::Object(hide_layout)) = entry.overlay.get("hideLayout") {
                if hide_layout
                    .get("all")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                    && !entry.schema_ref_path.is_empty()
                {
                    self.sync_hidden_to_schema_at(&entry.schema_ref_path);
                }
            }
        }
    }

    /// Write condition.hidden=true to schema node at dotted path.
    /// Only writes condition.hidden (not hideLayout) — hideLayout is for
    /// layout container elements, not leaf fields. Writing hideLayout would
    /// stick across re-evaluations and cause false cascade.
    fn sync_hidden_to_schema_at(&mut self, schema_path_dot: &str) {
        let schema_pointer = path_utils::dot_notation_to_schema_pointer(schema_path_dot);
        let normalized_pointer = path_utils::normalize_to_json_pointer(&schema_pointer);

        let target_pointer = if self.evaluated_schema.pointer(&normalized_pointer).is_some() {
            normalized_pointer.into_owned()
        } else {
            format!("/properties{}", normalized_pointer)
        };

        self.layout_synced_paths.push(target_pointer.clone());

        if let Some(schema_node) = self.evaluated_schema.pointer_mut(&target_pointer) {
            if let Value::Object(map) = schema_node {
                let mut condition = if let Some(Value::Object(c)) = map.get("condition") {
                    c.clone()
                } else {
                    serde_json::Map::new()
                };
                condition.insert("hidden".to_string(), Value::Bool(true));
                map.insert("condition".to_string(), Value::Object(condition));
            }
        }
    }
}
