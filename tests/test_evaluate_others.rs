use json_eval_rs::{jsoneval::types::LayoutOverlayEntry, JSONEval};
use serde_json::json;

/// Get a nested value from a JSON value using a JSON pointer path.
/// Supports paths like "#/properties/foo" or "/properties/foo".
fn get_by_pointer<'a>(
    value: &'a serde_json::Value,
    pointer: &str,
) -> Option<&'a serde_json::Value> {
    // Strip leading # only — keep leading / for serde_json::Value::pointer()
    let path = pointer.strip_prefix('#').unwrap_or(pointer);
    value.pointer(path)
}

/// Set a nested value in a JSON value using a JSON pointer path.
/// Uses serde_json's pointer_mut for correct array/object navigation.
fn set_by_pointer(value: &mut serde_json::Value, pointer: &str, new_value: serde_json::Value) {
    // Strip leading # only — keep leading /
    let path = pointer.strip_prefix('#').unwrap_or(pointer);
    if let Some(target) = value.pointer_mut(path) {
        *target = new_value;
    }
}

/// Merge layout overlay entries into an evaluated schema.
/// This is the Rust equivalent of the TypeScript mergeLayoutOverlay helper.
fn merge_layout_overlay(schema: &mut serde_json::Value, overlay_entries: &[LayoutOverlayEntry]) {
    for entry in overlay_entries {
        let layout_path = &entry.layout_path;
        let element_idx = entry.element_idx;

        // Phase 1: Read element from compact schema (immutable borrow)
        let (element_data, ref_path) = {
            let elements_array = get_by_pointer(schema, layout_path);
            match elements_array.and_then(|v| v.as_array()) {
                Some(elements) if element_idx < elements.len() => {
                    let el = &elements[element_idx];
                    let ref_path = el
                        .get("$ref")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    (Some(el.clone()), ref_path)
                }
                _ => (None, String::new()),
            }
        };

        let mut element = match element_data {
            Some(e) => e,
            None => continue,
        };

        // Phase 2: Apply overlay onto element
        if let Some(obj) = element.as_object_mut() {
            for (key, value) in &entry.overlay {
                obj.insert(key.clone(), value.clone());
            }
        }

        // Phase 3: Write element back into schema (mutable borrow)
        let element_path = format!("{}/{}", layout_path, element_idx);
        set_by_pointer(schema, &element_path, element);

        // Phase 4: Also merge overlay onto the schema definition (the ref target)
        if !ref_path.is_empty() && !entry.schema_ref_path.is_empty() {
            let schema_node = get_by_pointer(schema, &ref_path).cloned();
            if let Some(mut merged_value) = schema_node {
                if let Some(obj) = merged_value.as_object_mut() {
                    for (key, value) in &entry.overlay {
                        obj.insert(key.clone(), value.clone());
                    }
                }
                let schema_path = format!("#{}", ref_path);
                set_by_pointer(schema, &schema_path, merged_value);
            }
        }
    }
}

#[test]
fn test_options_url_dynamic_template_evaluation() {
    // Test that options.url templates are evaluated with params
    let schema = json!({
        "type": "object",
        "properties": {
            "users": {
                "type": "object",
                "properties": {
                    "list": {
                        "type": "array",
                        "options": {
                            "url": "https://api.example.com/users/{id}/posts/{category}",
                            "params": {
                                "id": {
                                    "$evaluation": {
                                        "+": [1, 2]
                                    }
                                },
                                "category": {
                                    "$evaluation": {
                                        "+": [4, 2]
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    // Run evaluation
    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let mut evaluated = eval.get_evaluated_schema();
    let overlay_entries = eval.get_resolved_layout();
    merge_layout_overlay(&mut evaluated, &overlay_entries);

    let url = evaluated
        .pointer("/properties/users/properties/list/options/url")
        .and_then(|v| v.as_str());

    assert_eq!(
        url,
        Some("https://api.example.com/users/3/posts/6"),
        "URL template should be evaluated with params"
    );
}

#[test]
fn test_options_url_template_evaluation() {
    // Test that options.url templates are evaluated with params
    let schema = json!({
        "type": "object",
        "properties": {
            "users": {
                "type": "object",
                "properties": {
                    "list": {
                        "type": "array",
                        "options": {
                            "url": "https://api.example.com/users/{id}/posts/{category}",
                            "params": {
                                "id": "123",
                                "category": "tech"
                            }
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    // Run evaluation
    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let mut evaluated = eval.get_evaluated_schema();
    let overlay_entries = eval.get_resolved_layout();
    merge_layout_overlay(&mut evaluated, &overlay_entries);

    let url = evaluated
        .pointer("/properties/users/properties/list/options/url")
        .and_then(|v| v.as_str());

    assert_eq!(
        url,
        Some("https://api.example.com/users/123/posts/tech"),
        "URL template should be evaluated with params"
    );
}

#[test]
fn test_options_url_template_with_number_params() {
    // Test that numeric params are converted to strings in templates
    let schema = json!({
        "type": "object",
        "properties": {
            "items": {
                "type": "array",
                "options": {
                    "url": "https://api.example.com/items?page={page}&limit={limit}",
                    "params": {
                        "page": 2,
                        "limit": 50
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let mut evaluated = eval.get_evaluated_schema();
    let overlay_entries = eval.get_resolved_layout();
    merge_layout_overlay(&mut evaluated, &overlay_entries);

    let url = evaluated
        .pointer("/properties/items/options/url")
        .and_then(|v| v.as_str());

    assert_eq!(
        url,
        Some("https://api.example.com/items?page=2&limit=50"),
        "Numeric params should be converted to strings"
    );
}

#[test]
fn test_options_url_without_template_unchanged() {
    // Test that URLs without templates remain unchanged
    let schema = json!({
        "type": "object",
        "properties": {
            "data": {
                "type": "array",
                "options": {
                    "url": "https://api.example.com/static/endpoint",
                    "params": {
                        "ignored": "value"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let mut evaluated = eval.get_evaluated_schema();
    let overlay_entries = eval.get_resolved_layout();
    merge_layout_overlay(&mut evaluated, &overlay_entries);

    let url = evaluated
        .pointer("/properties/data/options/url")
        .and_then(|v| v.as_str());

    // Should remain unchanged since no {template} pattern
    assert_eq!(
        url,
        Some("https://api.example.com/static/endpoint"),
        "URL without templates should remain unchanged"
    );
}

#[test]
fn test_layout_metadata_injection() {
    // Test that layout elements get metadata fields injected
    let schema = json!({
        "type": "object",
        "properties": {
            "person": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string"
                    },
                    "age": {
                        "type": "number"
                    }
                }
            },
            "form": {
                "$layout": {
                    "type": "VerticalLayout",
                    "elements": [
                        {
                            "$ref": "person.name"
                        },
                        {
                            "$ref": "person.age"
                        }
                    ]
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let mut evaluated = eval.get_evaluated_schema();
    let overlay_entries = eval.get_resolved_layout();
    merge_layout_overlay(&mut evaluated, &overlay_entries);

    // Check first element has metadata
    let first_element = evaluated
        .pointer("/properties/form/$layout/elements/0")
        .expect("First layout element should exist");

    assert!(
        first_element.get("$fullpath").is_some(),
        "Element should have $fullpath"
    );
    assert_eq!(
        first_element.get("$fullpath").and_then(|v| v.as_str()),
        Some("properties.person.properties.name"),
        "$fullpath should match resolved dotted path"
    );

    assert!(
        first_element.get("$path").is_some(),
        "Element should have $path"
    );
    assert_eq!(
        first_element.get("$path").and_then(|v| v.as_str()),
        Some("name"),
        "$path should be last segment"
    );

    assert!(
        first_element.get("$parentHide").is_some(),
        "Element should have $parentHide"
    );
    assert_eq!(
        first_element.get("$parentHide").and_then(|v| v.as_bool()),
        Some(false),
        "$parentHide should be false initially"
    );
}

#[test]
fn test_layout_metadata_parent_hidden() {
    // Test that $parentHide flag is properly propagated to children
    let schema = json!({
        "type": "object",
        "properties": {
            "child_field": {
                "type": "string",
                "title": "Child Field"
            },
            "parent_container": {
                "type": "object",
                "condition": {
                    "hidden": true  // Parent is hidden
                },
                "properties": {
                    "nested_field": {
                        "type": "string",
                        "title": "Nested Field"
                    }
                }
            },
            "form": {
                "$layout": {
                    "type": "VerticalLayout",
                    "elements": [
                        {
                            "$ref": "child_field"
                        },
                        {
                            "$ref": "parent_container",
                            "elements": [
                                {
                                    "$ref": "parent_container.nested_field"
                                }
                            ]
                        }
                    ]
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let mut evaluated = eval.get_evaluated_schema();
    let overlay_entries = eval.get_resolved_layout();
    merge_layout_overlay(&mut evaluated, &overlay_entries);

    // Test 1: Child at root level should have $parentHide = false
    let root_element = evaluated
        .pointer("/properties/form/$layout/elements/0")
        .expect("Root level element should exist");

    assert_eq!(
        root_element.get("$parentHide").and_then(|v| v.as_bool()),
        Some(false),
        "$parentHide should be false at root level"
    );

    // Test 2: Parent container should have $parentHide = false (no parent above it is hidden)
    let parent_element = evaluated
        .pointer("/properties/form/$layout/elements/1")
        .expect("Parent element should exist");

    assert_eq!(
        parent_element.get("$parentHide").and_then(|v| v.as_bool()),
        Some(false),
        "$parentHide should be false for parent element at root"
    );

    // Test 3: Child of hidden parent should have $parentHide = true
    let child_of_hidden = evaluated
        .pointer("/properties/form/$layout/elements/1/elements/0")
        .expect("Child of hidden parent should exist");

    assert_eq!(
        child_of_hidden.get("$parentHide").and_then(|v| v.as_bool()),
        Some(true),
        "$parentHide should be true when parent is hidden"
    );
}

#[test]
fn test_hide_layout_propagation() {
    // Test that hideLayout.all is propagated to children of direct layout elements
    let schema = json!({
        "type": "object",
        "properties": {
            "field1": {
                "type": "string",
                "title": "Field 1"
            },
            "field2": {
                "type": "string",
                "title": "Field 2"
            },
            "form": {
                "$layout": {
                    "type": "VerticalLayout",
                    "elements": [
                        {
                            "type": "HorizontalLayout",
                            "hideLayout": {
                                "all": true
                            },
                            "elements": [
                                {
                                    "$ref": "field1"
                                },
                                {
                                    "type": "FlexLayout",
                                    "elements": [
                                        {
                                            "$ref": "field2"
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let mut evaluated = eval.get_evaluated_schema();
    let overlay_entries = eval.get_resolved_layout();
    merge_layout_overlay(&mut evaluated, &overlay_entries);

    // Test 1: Parent HorizontalLayout has hideLayout.all = true
    let parent_layout = evaluated
        .pointer("/properties/form/$layout/elements/0")
        .expect("Parent layout should exist");

    assert_eq!(
        parent_layout
            .get("hideLayout")
            .and_then(|h| h.get("all"))
            .and_then(|v| v.as_bool()),
        Some(true),
        "Parent should have hideLayout.all = true"
    );

    // Test 2: Field child should have condition.hidden = true
    let field_child = evaluated
        .pointer("/properties/form/$layout/elements/0/elements/0")
        .expect("Field child should exist");

    assert_eq!(
        field_child
            .get("condition")
            .and_then(|c| c.get("hidden"))
            .and_then(|v| v.as_bool()),
        Some(true),
        "Field child should have condition.hidden = true (inherited from parent)"
    );

    assert_eq!(
        field_child.get("$parentHide").and_then(|v| v.as_bool()),
        Some(true),
        "Field child should have $parentHide = true"
    );

    // Test 3: Nested FlexLayout should have hideLayout.all = true
    let nested_layout = evaluated
        .pointer("/properties/form/$layout/elements/0/elements/1")
        .expect("Nested layout should exist");

    assert_eq!(
        nested_layout
            .get("hideLayout")
            .and_then(|h| h.get("all"))
            .and_then(|v| v.as_bool()),
        Some(true),
        "Nested layout should have hideLayout.all = true (inherited from parent)"
    );

    // Test 4: Deeply nested field should have condition.hidden = true
    let nested_field = evaluated
        .pointer("/properties/form/$layout/elements/0/elements/1/elements/0")
        .expect("Nested field should exist");

    assert_eq!(
        nested_field
            .get("condition")
            .and_then(|c| c.get("hidden"))
            .and_then(|v| v.as_bool()),
        Some(true),
        "Nested field should have condition.hidden = true (inherited from parent)"
    );

    assert_eq!(
        nested_field.get("$parentHide").and_then(|v| v.as_bool()),
        Some(true),
        "Nested field should have $parentHide = true"
    );
}

#[test]
fn test_direct_layout_elements_have_metadata() {
    // Test that direct layout elements (without $ref) also get metadata fields
    let schema = json!({
        "type": "object",
        "properties": {
            "field1": {
                "type": "string",
                "title": "Field 1"
            },
            "field2": {
                "type": "string",
                "title": "Field 2"
            },
            "form": {
                "$layout": {
                    "type": "VerticalLayout",
                    "elements": [
                        {
                            "type": "FlexLayout",
                            "elements": [
                                {
                                    "$ref": "field1"
                                },
                                {
                                    "$ref": "field2"
                                }
                            ]
                        }
                    ]
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let mut evaluated = eval.get_evaluated_schema();
    let overlay_entries = eval.get_resolved_layout();
    merge_layout_overlay(&mut evaluated, &overlay_entries);

    // Check the FlexLayout container (which has no $ref)
    let flex_container = evaluated
        .pointer("/properties/form/$layout/elements/0")
        .expect("FlexLayout container should exist");

    // Direct layout elements should have metadata fields
    assert!(
        flex_container.get("$parentHide").is_some(),
        "Direct layout elements should have $parentHide"
    );

    assert!(
        flex_container.get("$path").is_some(),
        "Direct layout elements should have $path"
    );

    assert!(
        flex_container.get("$fullpath").is_some(),
        "Direct layout elements should have $fullpath"
    );

    // Values should reflect position in layout hierarchy for non-$ref elements
    assert_eq!(
        flex_container.get("$parentHide").and_then(|v| v.as_bool()),
        Some(false),
        "$parentHide should default to false"
    );

    // Direct layout elements should have path based on their position
    assert_eq!(
        flex_container.get("$path").and_then(|v| v.as_str()),
        Some("0"),
        "$path should be element index for direct layout elements"
    );

    assert_eq!(
        flex_container.get("$fullpath").and_then(|v| v.as_str()),
        Some("form.0"),
        "$fullpath for non-$ref container should be clean positional path (layout segments stripped)"
    );
}

#[test]
fn test_json_pointer_ref_conversion() {
    // Test that JSON pointer format $ref is converted to dotted notation in metadata
    let schema = json!({
        "type": "object",
        "properties": {
            "illustration": {
                "type": "object",
                "properties": {
                    "insured": {
                        "type": "object",
                        "properties": {
                            "ins_corrname": {
                                "type": "string",
                                "title": "Insured Name"
                            }
                        }
                    }
                }
            },
            "form": {
                "$layout": {
                    "type": "VerticalLayout",
                    "elements": [
                        {
                            "$ref": "#/illustration/properties/insured/properties/ins_corrname"
                        }
                    ]
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let mut evaluated = eval.get_evaluated_schema();
    let overlay_entries = eval.get_resolved_layout();
    merge_layout_overlay(&mut evaluated, &overlay_entries);

    let element = evaluated
        .pointer("/properties/form/$layout/elements/0")
        .expect("Layout element should exist");

    // $fullpath should be converted to dotted notation
    assert_eq!(
        element.get("$fullpath").and_then(|v| v.as_str()),
        Some("illustration.properties.insured.properties.ins_corrname"),
        "$fullpath should be in dotted notation"
    );

    // $path should be the last segment only
    assert_eq!(
        element.get("$path").and_then(|v| v.as_str()),
        Some("ins_corrname"),
        "$path should be the last segment"
    );
}

#[test]
fn test_multiple_options_templates_in_schema() {
    // Test multiple options with templates are all evaluated
    let schema = json!({
        "type": "object",
        "properties": {
            "users": {
                "options": {
                    "url": "/api/users/{userId}",
                    "params": { "userId": "42" }
                }
            },
            "posts": {
                "options": {
                    "url": "/api/posts/{postId}/comments/{commentId}",
                    "params": { "postId": "100", "commentId": "5" }
                }
            },
            "static": {
                "options": {
                    "url": "/api/static"
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let mut evaluated = eval.get_evaluated_schema();
    let overlay_entries = eval.get_resolved_layout();
    merge_layout_overlay(&mut evaluated, &overlay_entries);

    assert_eq!(
        evaluated
            .pointer("/properties/users/options/url")
            .and_then(|v| v.as_str()),
        Some("/api/users/42")
    );
    assert_eq!(
        evaluated
            .pointer("/properties/posts/options/url")
            .and_then(|v| v.as_str()),
        Some("/api/posts/100/comments/5")
    );
    assert_eq!(
        evaluated
            .pointer("/properties/static/options/url")
            .and_then(|v| v.as_str()),
        Some("/api/static")
    );
}

#[test]
fn test_options_template_collected_at_parse_time() {
    // Verify that options templates are collected during parsing, not evaluation
    let schema = json!({
        "properties": {
            "api": {
                "options": {
                    "url": "/users/{id}",
                    "params": { "id": "test" }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let eval = JSONEval::new(&schema_str, None, None).unwrap();

    // Check that options_templates were collected
    assert_eq!(
        eval.options_templates.len(),
        1,
        "Should have collected 1 options template at parse time"
    );

    let (url_path, template_str, params_path) = &eval.options_templates[0];
    // Note: paths are normalized to JSON pointer format during parsing
    assert_eq!(url_path, "/properties/api/options/url");
    assert_eq!(template_str, "/users/{id}");
    assert_eq!(params_path, "/properties/api/options/params");
}
