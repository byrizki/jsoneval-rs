use super::JSONEval;
use crate::jsoneval::path_utils;
use crate::jsoneval::types::{ResolvedLayoutResult, ReturnFormat};
use crate::time_block;
use crate::utils::clean_float_noise_scalar;
use serde_json::Value;
use std::sync::Arc;

impl JSONEval {
    /// Check if a field is effectively hidden by checking its condition and all parents
    /// Also checks for $layout.hideLayout.all on parents
    pub(crate) fn is_effective_hidden(&self, schema_pointer: &str) -> bool {
        let schema_pointer = schema_pointer.trim_start_matches('#');
        if self.layout_hidden_refs.iter().any(|hidden_ref| {
            schema_pointer == hidden_ref
                || schema_pointer
                    .strip_prefix(hidden_ref)
                    .is_some_and(|suffix| {
                        suffix.starts_with("/properties/") || suffix.starts_with("/items/")
                    })
        }) {
            return true;
        }

        let mut end = schema_pointer.len();

        loop {
            let current_path = &schema_pointer[..end];

            if let Some(schema_node) = self.evaluated_schema.pointer(current_path) {
                if let Value::Object(map) = schema_node {
                    if let Some(Value::Object(condition)) = map.get("condition") {
                        if let Some(Value::Bool(true)) = condition.get("hidden") {
                            return true;
                        }
                    }

                    if let Some(Value::Object(layout)) = map.get("$layout") {
                        if let Some(Value::Object(hide_layout)) = layout.get("hideLayout") {
                            if let Some(Value::Bool(true)) = hide_layout.get("all") {
                                return true;
                            }
                        }
                    }
                }
            }

            if end == 0 {
                break;
            }

            // Move to parent: find last '/' and strip /properties or /items suffixes
            match schema_pointer[..end].rfind('/') {
                Some(0) | None => {
                    end = 0;
                }
                Some(last_slash) => {
                    end = last_slash;
                    let parent = &schema_pointer[..end];
                    if parent.ends_with("/properties") {
                        end -= "/properties".len();
                    } else if parent.ends_with("/items") {
                        end -= "/items".len();
                    }
                }
            }
        }

        false
    }

    /// Prune hidden values from data object recursively
    fn prune_hidden_values(&self, data: &mut Value, current_path: &str) {
        if let Value::Object(map) = data {
            // Collect keys to remove to avoid borrow checker issues
            let mut keys_to_remove = Vec::new();

            for (key, value) in map.iter_mut() {
                // Skip special keys
                if key == "$params" || key == "$context" {
                    continue;
                }

                // Construct schema path for this key
                // For root fields: /properties/key
                // For nested fields: current_path/properties/key
                let schema_path = if current_path.is_empty() {
                    format!("/properties/{}", key)
                } else {
                    format!("{}/properties/{}", current_path, key)
                };

                // Check if hidden
                if self.is_effective_hidden(&schema_path) {
                    keys_to_remove.push(key.clone());
                } else {
                    // Recurse if object
                    if value.is_object() {
                        self.prune_hidden_values(value, &schema_path);
                    }
                }
            }

            // Remove hidden keys
            for key in keys_to_remove {
                map.remove(&key);
            }
        }
    }

    /// Replace any `{"$static_array": "/$table/..."}` and `{"$static_array": "/$params/..."}` markers in `schema_output`
    /// with the actual evaluated array data from `eval_data`.
    ///
    /// By iterating only over tracked `static_arrays`, we replace markers in O(markers) time
    /// instead of requiring an expensive O(schema_nodes) recursive tree walk.
    fn resolve_static_markers_in_value(&self, schema_output: &mut Value) {
        for (static_key, array_arc) in self.static_arrays.iter() {
            // Determine the schema pointer path where this marker was placed
            let schema_path = if static_key.starts_with("/$table") {
                &static_key["/$table".len()..] // e.g. /properties/product_benefit/...
            } else {
                static_key.as_str() // e.g. /$params/references/...
            };

            // Only attempt replacement if the exact path exists in the cloned schema output
            if let Some(target_val) = schema_output.pointer_mut(schema_path) {
                // The actual evaluated array is seamlessly stored right in the map's value
                *target_val = (**array_arc).clone();
            }
        }
    }

    /// Get the evaluated schema (compact — $ref intact, no layout expansion).
    ///
    /// # Returns
    ///
    /// The evaluated schema as a JSON value, with all `$static_array` markers resolved
    /// to their actual evaluated data.
    pub fn get_evaluated_schema(&mut self) -> Value {
        time_block!("get_evaluated_schema()", {
            let mut schema = self.evaluated_schema.clone();
            self.resolve_static_markers_in_value(&mut schema);
            schema
        })
    }

    /// Get layout overlay entries — the delta properties per layout element.
    /// Consumer merges these into compact schema to get fully resolved layout.
    pub fn get_resolved_layout(&mut self) -> ResolvedLayoutResult {
        time_block!("get_resolved_layout()", {
            // Check cache
            if let Some(ref cached) = self.resolved_layout_cache {
                return cached.as_ref().clone();
            }
            // Resolve and cache
            let result = match self.resolve_layout(false) {
                Ok(entries) => entries,
                Err(e) => {
                    eprintln!("Warning: Layout resolution failed: {}", e);
                    Vec::new()
                }
            };
            self.resolved_layout_cache = Some(Arc::new(result.clone()));
            result
        })
    }

    /// Get evaluated schema with layout overlays already applied.
    /// Convenience: returns compact schema + overlays merged.
    ///
    /// Two-pass approach to handle nested elements:
    /// 1. First pass: resolve $ref and apply overlay for entries whose target
    ///    path exists in the compact schema (top-level elements).
    /// 2. Second pass: apply overlay-only for entries whose path appears
    ///    after parent $ref resolution (nested elements).
    pub fn get_evaluated_schema_resolved(&mut self) -> Value {
        time_block!("get_evaluated_schema_resolved()", {
            let mut schema = self.get_evaluated_schema_without_params();
            let overlays = self.get_resolved_layout();

            struct ResolveEntry {
                layout_path: String,
                element_idx: usize,
                overlay: indexmap::IndexMap<String, Value>,
            }

            let mut entries: Vec<ResolveEntry> = overlays
                .iter()
                .map(|entry| {
                    let layout_path =
                        path_utils::normalize_to_json_pointer(&entry.layout_path).into_owned();
                    ResolveEntry {
                        layout_path,
                        element_idx: entry.element_idx,
                        overlay: entry.overlay.clone(),
                    }
                })
                .collect();
            drop(overlays);

            // Sort entries shallow-first so parent elements are expanded before their children.
            // Child entries (e.g. layout_path = ".../elements/1/elements") depend on the parent
            // ("…/elements") being resolved first so the nested `elements` array exists in `schema`.
            entries.sort_by(|a, b| {
                let depth_a = a.layout_path.matches('/').count();
                let depth_b = b.layout_path.matches('/').count();
                depth_a
                    .cmp(&depth_b)
                    .then_with(|| a.element_idx.cmp(&b.element_idx))
            });

            // ── Phase 2 (mutable): resolve $ref + apply overlays (parent-first order) ──
            // Entries are sorted shallowest layout_path first, so parent elements are
            // expanded before any child entries that path through them.
            for entry in entries {
                // Resolve $ref from the current (already partially mutated) schema so that
                // parent expansions are visible when we process child entries.
                let resolved_value: Option<Value> = (|| -> Option<Value> {
                    let arr = schema.pointer(&entry.layout_path)?.as_array()?;
                    let element = arr.get(entry.element_idx)?;
                    let ref_str = element.get("$ref")?.as_str()?;

                    let ref_pointer = if ref_str.starts_with('#') || ref_str.starts_with('/') {
                        path_utils::normalize_to_json_pointer(ref_str).into_owned()
                    } else {
                        let schema_pointer = path_utils::dot_notation_to_schema_pointer(ref_str);
                        let normalized =
                            path_utils::normalize_to_json_pointer(&schema_pointer).into_owned();
                        if schema.pointer(&normalized).is_some() {
                            normalized
                        } else {
                            format!("/properties/{}", ref_str.replace('.', "/properties/"))
                        }
                    };

                    let mut resolved = schema.pointer(&ref_pointer)?.clone();

                    // Flatten $layout into top level
                    if let Value::Object(ref mut resolved_map) = resolved {
                        if let Some(Value::Object(layout_obj)) = resolved_map.remove("$layout") {
                            let mut result = layout_obj;
                            for (key, value) in resolved_map.clone().into_iter() {
                                if key != "type" || !result.contains_key("type") {
                                    result.insert(key, value);
                                }
                            }
                            resolved = Value::Object(result);
                        }
                    }

                    Some(resolved)
                })();

                if let Some(Value::Array(arr)) = schema.pointer_mut(&entry.layout_path) {
                    if entry.element_idx < arr.len() {
                        let element = &mut arr[entry.element_idx];

                        // Apply $ref resolution
                        if let Some(resolved) = resolved_value {
                            if let Value::Object(mut resolved_map) = resolved {
                                if let Value::Object(mut map) = element.take() {
                                    map.remove("$ref");
                                    for (key, value) in map {
                                        resolved_map.insert(key, value);
                                    }
                                }
                                *element = Value::Object(resolved_map);
                            } else {
                                *element = resolved;
                            }
                        }

                        // Apply overlay on top
                        if let Value::Object(ref mut map) = element {
                            for (k, v) in &entry.overlay {
                                map.insert(k.clone(), v.clone());
                            }
                        }
                    }
                }
            }

            Self::stamp_property_metadata(&mut schema);
            schema
        })
    }

    /// Stamp every schema property with raw pointer-style dotted metadata.
    fn stamp_property_metadata(schema: &mut Value) {
        fn walk(value: &mut Value, path: &str, parent_hidden: bool) {
            let Some(map) = value.as_object_mut() else {
                return;
            };

            let hidden = parent_hidden
                || map
                    .get("condition")
                    .and_then(Value::as_object)
                    .and_then(|condition| condition.get("hidden"))
                    .is_some_and(|hidden| hidden == &Value::Bool(true));

            if let Some(Value::Object(properties)) = map.get_mut("properties") {
                for (name, property) in properties {
                    let property_path = if path.is_empty() {
                        format!("properties.{}", name)
                    } else {
                        format!("{}.properties.{}", path, name)
                    };
                    if let Value::Object(property_map) = property {
                        property_map.insert(
                            "$fullpath".to_string(),
                            Value::String(property_path.clone()),
                        );
                        property_map.insert("$path".to_string(), Value::String(name.clone()));
                        property_map.insert("$parentHide".to_string(), Value::Bool(hidden));
                    }
                    walk(property, &property_path, hidden);
                }
            }

            for (name, child) in map {
                if name != "properties" && !name.starts_with('$') && child.is_object() {
                    let child_path = if path.is_empty() {
                        name.clone()
                    } else {
                        format!("{}.{}", path, name)
                    };
                    walk(child, &child_path, hidden);
                }
            }
        }

        walk(schema, "", false);
    }

    /// Resolve `$static_array` markers within the subtree rooted at `schema_prefix`.
    ///
    /// Clones only the node at `schema_prefix` from `evaluated_schema`, then iterates
    /// the tracked `static_arrays` list filtering to entries whose schema path is at or
    /// under `schema_prefix`. Only those markers are replaced inside the cloned subtree;
    /// unrelated entries are skipped entirely.
    ///
    /// # Examples
    /// - `schema_prefix = "/$params/references"` → resolves only arrays nested under that key
    /// - `schema_prefix = "/properties/foo/value"` → resolves a single marker if the field itself is one
    fn resolve_static_markers_at_path(&self, schema_prefix: &str) -> Option<Value> {
        let mut subtree = self.evaluated_schema.pointer(schema_prefix)?.clone();

        // Pre-build "prefix/" once for the starts_with check in the loop
        let prefix_slash = format!("{}/", schema_prefix);

        for (static_key, array_arc) in self.static_arrays.iter() {
            // Derive the absolute schema path the same way resolve_static_markers_in_value does
            let schema_path: &str = if static_key.starts_with("/$table") {
                &static_key["/$table".len()..]
            } else {
                static_key.as_str()
            };

            // Compute the path relative to the subtree root
            let relative: &str = if schema_path == schema_prefix {
                // The subtree root itself is the marker — replace the whole subtree
                ""
            } else if schema_path.starts_with(&prefix_slash) {
                // Strip the prefix: remainder is the sub-path within the cloned subtree
                &schema_path[schema_prefix.len()..]
            } else {
                continue; // Not under the requested path — skip
            };

            if relative.is_empty() {
                subtree = (**array_arc).clone();
            } else if let Some(target) = subtree.pointer_mut(relative) {
                *target = (**array_arc).clone();
            }
        }

        Some(subtree)
    }

    /// Get specific schema value by path, resolving any `$static_array` markers at or
    /// under that path.
    pub fn get_schema_value_by_path(&self, path: &str) -> Option<Value> {
        let pointer_path = path_utils::dot_notation_to_schema_pointer(path);
        self.resolve_static_markers_at_path(pointer_path.trim_start_matches('#'))
    }

    /// Get all schema values (data view)
    /// Mutates internal data state by overriding with values from value evaluations
    /// This corresponds to subform.get_schema_value() usage
    pub fn get_schema_value(&mut self) -> Value {
        // Start with current authoritative data from eval_data
        let mut current_data = self.eval_data.data().clone();

        // Ensure it's an object
        if !current_data.is_object() {
            current_data = Value::Object(serde_json::Map::new());
        }

        // Strip $params and $context from data
        if let Some(obj) = current_data.as_object_mut() {
            obj.remove("$params");
            obj.remove("$context");
        }

        // Prune hidden values from current_data (to remove user input in hidden fields)
        self.prune_hidden_values(&mut current_data, "");

        // Override data with values from value evaluations
        // We use value_evaluations which stores the paths of fields with .value
        for eval_key in self.value_evaluations.iter() {
            let clean_key = eval_key.strip_prefix('#').unwrap_or(eval_key);

            // Exclude rules.*.value, options.*.value, and $params
            if clean_key.starts_with("/$params")
                || (clean_key.ends_with("/value")
                    && (clean_key.contains("/rules/") || clean_key.contains("/options/")))
            {
                continue;
            }

            let path = clean_key.replace("/properties", "").replace("/value", "");

            // Check if field is effectively hidden
            // Schema path is clean_key without /value
            let schema_path = clean_key.strip_suffix("/value").unwrap_or(&clean_key);
            if self.is_effective_hidden(schema_path) {
                continue;
            }

            // Resolve static markers at this specific pointer (handles markers at or under this path)
            let value = match self.resolve_static_markers_at_path(clean_key) {
                Some(v) => v,
                None => continue,
            };

            // Parse the path and create nested structure as needed
            let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

            if path_parts.is_empty() {
                continue;
            }

            // Navigate/create nested structure
            let mut current = &mut current_data;
            for (i, part) in path_parts.iter().enumerate() {
                let is_last = i == path_parts.len() - 1;

                if is_last {
                    // Set the value at the final key
                    if let Some(obj) = current.as_object_mut() {
                        let should_update = match obj.get(*part) {
                            Some(v) => v.is_null(),
                            None => true,
                        };

                        if should_update {
                            obj.insert(
                                (*part).to_string(),
                                crate::utils::clean_float_noise(value.clone()),
                            );
                        }
                    }
                } else {
                    // Ensure current is an object, then navigate/create intermediate objects
                    if let Some(obj) = current.as_object_mut() {
                        if !obj.contains_key(*part) {
                            obj.insert((*part).to_string(), Value::Object(serde_json::Map::new()));
                        }

                        current = obj.get_mut(*part).unwrap();
                    } else {
                        // Skip this path if current is not an object and can't be made into one
                        break;
                    }
                }
            }
        }

        // Update self.data to persist the view changes (matching backup behavior)
        self.data = current_data.clone();

        crate::utils::clean_float_noise(current_data)
    }

    /// Get all schema values as array of path-value pairs
    /// Returns [{path: "", value: ""}, ...]
    ///
    /// # Returns
    ///
    /// Array of objects containing path (dotted notation) and value pairs from value evaluations
    pub fn get_schema_value_array(&self) -> Value {
        let mut result = Vec::new();

        for eval_key in self.value_evaluations.iter() {
            let clean_key = eval_key.strip_prefix('#').unwrap_or(eval_key);

            // Exclude rules.*.value, options.*.value, and $params
            if clean_key.starts_with("/$params")
                || (clean_key.ends_with("/value")
                    && (clean_key.contains("/rules/") || clean_key.contains("/options/")))
            {
                continue;
            }

            // Check if field is effectively hidden
            let schema_path = clean_key.strip_suffix("/value").unwrap_or(&clean_key);
            if self.is_effective_hidden(schema_path) {
                continue;
            }

            // Convert JSON pointer to dotted notation
            let dotted_path = clean_key
                .replace("/properties", "")
                .replace("/value", "")
                .trim_start_matches('/')
                .replace('/', ".");

            if dotted_path.is_empty() {
                continue;
            }

            // Resolve static markers at this specific pointer (handles markers at or under this path)
            let value = match self.resolve_static_markers_at_path(clean_key) {
                Some(v) => crate::utils::clean_float_noise(v),
                None => continue,
            };

            // Create {path, value} object
            let mut item = serde_json::Map::new();
            item.insert("path".to_string(), Value::String(dotted_path));
            item.insert("value".to_string(), value);
            result.push(Value::Object(item));
        }

        Value::Array(result)
    }

    /// Get all schema values as object with dotted path keys
    /// Returns {path: value, ...}
    ///
    /// # Returns
    ///
    /// Flat object with dotted notation paths as keys and evaluated values
    pub fn get_schema_value_object(&self) -> Value {
        let mut result = serde_json::Map::new();

        for eval_key in self.value_evaluations.iter() {
            let clean_key = eval_key.strip_prefix('#').unwrap_or(eval_key);

            // Exclude rules.*.value, options.*.value, and $params
            if clean_key.starts_with("/$params")
                || (clean_key.ends_with("/value")
                    && (clean_key.contains("/rules/") || clean_key.contains("/options/")))
            {
                continue;
            }

            // Check if field is effectively hidden
            let schema_path = clean_key.strip_suffix("/value").unwrap_or(&clean_key);
            if self.is_effective_hidden(schema_path) {
                continue;
            }

            // Convert JSON pointer to dotted notation
            let dotted_path = clean_key
                .replace("/properties", "")
                .replace("/value", "")
                .trim_start_matches('/')
                .replace('/', ".");

            if dotted_path.is_empty() {
                continue;
            }

            // Resolve static markers at this specific pointer (handles markers at or under this path)
            let value = match self.resolve_static_markers_at_path(clean_key) {
                Some(v) => crate::utils::clean_float_noise(v),
                None => continue,
            };

            result.insert(dotted_path, value);
        }

        Value::Object(result)
    }

    /// Get evaluated schema without $params
    pub fn get_evaluated_schema_without_params(&mut self) -> Value {
        let mut schema = self.get_evaluated_schema();
        if let Value::Object(ref mut map) = schema {
            map.remove("$params");
        }
        schema
    }

    /// Get evaluated schema as MessagePack bytes
    pub fn get_evaluated_schema_msgpack(&mut self) -> Result<Vec<u8>, String> {
        let schema = self.get_evaluated_schema();
        rmp_serde::to_vec(&schema).map_err(|e| format!("MessagePack serialization failed: {}", e))
    }

    /// Get value from evaluated schema by path
    pub fn get_evaluated_schema_by_path(&mut self, path: &str) -> Option<Value> {
        self.get_schema_value_by_path(path)
    }

    /// Get evaluated schema parts by multiple paths
    pub fn get_evaluated_schema_by_paths(
        &mut self,
        paths: &[String],
        format: Option<ReturnFormat>,
    ) -> Value {
        match format.unwrap_or(ReturnFormat::Nested) {
            ReturnFormat::Nested => {
                let mut result = Value::Object(serde_json::Map::new());
                for path in paths {
                    if let Some(val) = self.get_schema_value_by_path(path) {
                        // Insert into result object at proper path nesting
                        Self::insert_at_path(&mut result, path, val);
                    }
                }
                result
            }
            ReturnFormat::Flat => {
                let mut result = serde_json::Map::new();
                for path in paths {
                    if let Some(val) = self.get_schema_value_by_path(path) {
                        result.insert(path.clone(), val);
                    }
                }
                Value::Object(result)
            }
            ReturnFormat::Array => {
                let mut result = Vec::new();
                for path in paths {
                    if let Some(val) = self.get_schema_value_by_path(path) {
                        result.push(val);
                    } else {
                        result.push(Value::Null);
                    }
                }
                Value::Array(result)
            }
        }
    }

    /// Get original (unevaluated) schema by path
    pub fn get_schema_by_path(&self, path: &str) -> Option<Value> {
        let pointer_path = path_utils::dot_notation_to_schema_pointer(path);
        self.schema
            .pointer(&pointer_path.trim_start_matches('#'))
            .cloned()
    }

    /// Get original schema by multiple paths
    pub fn get_schema_by_paths(&self, paths: &[String], format: Option<ReturnFormat>) -> Value {
        match format.unwrap_or(ReturnFormat::Nested) {
            ReturnFormat::Nested => {
                let mut result = Value::Object(serde_json::Map::new());
                for path in paths {
                    if let Some(val) = self.get_schema_by_path(path) {
                        Self::insert_at_path(&mut result, path, val);
                    }
                }
                result
            }
            ReturnFormat::Flat => {
                let mut result = serde_json::Map::new();
                for path in paths {
                    if let Some(val) = self.get_schema_by_path(path) {
                        result.insert(path.clone(), val);
                    }
                }
                Value::Object(result)
            }
            ReturnFormat::Array => {
                let mut result = Vec::new();
                for path in paths {
                    if let Some(val) = self.get_schema_by_path(path) {
                        result.push(val);
                    } else {
                        result.push(Value::Null);
                    }
                }
                Value::Array(result)
            }
        }
    }

    /// Helper to insert value into nested object at dotted path
    pub(crate) fn insert_at_path(root: &mut Value, path: &str, value: Value) {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = root;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - set value
                if let Value::Object(map) = current {
                    map.insert(part.to_string(), value);
                    return; // Done
                }
            } else {
                // Intermediate part - traverse or create
                // We need to temporarily take the value or use raw pointer manipulation?
                // serde_json pointer is read-only or requires mutable reference

                if !current.is_object() {
                    *current = Value::Object(serde_json::Map::new());
                }

                if let Value::Object(map) = current {
                    if !map.contains_key(*part) {
                        map.insert(part.to_string(), Value::Object(serde_json::Map::new()));
                    }
                    current = map.get_mut(*part).unwrap();
                }
            }
        }
    }

    /// Flatten a nested object key-value pair to dotted keys
    pub fn flatten_object(
        prefix: &str,
        value: &Value,
        result: &mut serde_json::Map<String, Value>,
    ) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    let new_key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    Self::flatten_object(&new_key, v, result);
                }
            }
            _ => {
                result.insert(prefix.to_string(), value.clone());
            }
        }
    }

    pub fn convert_to_format(value: Value, format: ReturnFormat) -> Value {
        match format {
            ReturnFormat::Nested => value,
            ReturnFormat::Flat => {
                let mut result = serde_json::Map::new();
                Self::flatten_object("", &value, &mut result);
                Value::Object(result)
            }
            ReturnFormat::Array => {
                if let Value::Object(map) = value {
                    Value::Array(map.values().cloned().collect())
                } else if let Value::Array(arr) = value {
                    Value::Array(arr)
                } else {
                    Value::Array(vec![value])
                }
            }
        }
    }

    /// Evaluate and return the options for a specific field on demand.
    ///
    /// Accepts dotted notation (`form.occupation`), JSON pointer
    /// (`/properties/form/properties/occupation`), or schema ref
    /// (`#/properties/form/properties/occupation`).
    ///
    /// Returns `None` when the field does not have an `options` key.
    /// Returns the resolved options value (array, URL string, or null) otherwise.
    pub fn get_field_options(&mut self, field_path: &str) -> Option<Value> {
        // Normalize the input to a schema pointer (e.g. #/properties/form/properties/occupation)
        let schema_ptr = if field_path.starts_with('#') || field_path.starts_with('/') {
            path_utils::normalize_to_json_pointer(field_path).into_owned()
        } else {
            path_utils::dot_notation_to_schema_pointer(field_path)
        };

        // Build the JSON pointer path to the /options node (strip leading # for serde pointer())
        let options_schema_key = format!("{}/options", schema_ptr);
        let options_pointer =
            path_utils::normalize_to_json_pointer(&options_schema_key).into_owned();

        // Check if the options node exists in the evaluated schema
        let options_node = self.evaluated_schema.pointer(&options_pointer)?.clone();

        // If the options node is an object with $evaluation, evaluate it now (deferred)
        if let Value::Object(ref map) = options_node {
            if map.contains_key("$evaluation") {
                let eval_key = options_schema_key.clone();

                if let Some(logic_id) = self.evaluations.get(&eval_key).copied() {
                    let snap = self.eval_data.snapshot_data();
                    if let Ok(result) = self.engine.run(&logic_id, &*snap) {
                        let cleaned = clean_float_noise_scalar(result);
                        if let Some(node) = self.evaluated_schema.pointer_mut(&options_pointer) {
                            *node = cleaned.clone();
                        }
                        return Some(cleaned);
                    }
                }
                // No compiled logic found — options cannot be resolved
                return None;
            }
        }

        // Check options_templates for a URL template at this field's options/url path
        let url_pointer =
            path_utils::normalize_to_json_pointer(&format!("{}/options/url", schema_ptr))
                .into_owned();

        let templates = self.options_templates.clone();
        for (tmpl_url_path, tmpl_str, tmpl_params_path) in templates.iter() {
            if *tmpl_url_path == url_pointer {
                if let Some(params) = self.evaluated_schema.pointer(tmpl_params_path) {
                    let params = params.clone();
                    if let Ok(resolved_url) = self.evaluate_template(tmpl_str, &params) {
                        if let Some(target) = self.evaluated_schema.pointer_mut(&url_pointer) {
                            *target = Value::String(resolved_url);
                        }
                        return self.evaluated_schema.pointer(&options_pointer).cloned();
                    }
                }
                break;
            }
        }

        // Static options (already-evaluated array or plain value)
        Some(options_node)
    }
}
