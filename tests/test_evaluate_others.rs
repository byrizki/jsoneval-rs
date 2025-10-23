use json_eval_rs::JSONEval;
use serde_json::json;

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
    eval.evaluate(&data_str, None).unwrap();
    
    // Get evaluated schema
    let evaluated = eval.get_evaluated_schema(false);
    
    // Check that URL template was evaluated
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
    eval.evaluate(&data_str, None).unwrap();
    
    let evaluated = eval.get_evaluated_schema(false);
    
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
    eval.evaluate(&data_str, None).unwrap();
    
    let evaluated = eval.get_evaluated_schema(false);
    
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
    eval.evaluate(&data_str, None).unwrap();
    
    let evaluated = eval.get_evaluated_schema(false);
    
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
        Some("person.name"),
        "$fullpath should match $ref"
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
    eval.evaluate(&data_str, None).unwrap();
    
    let evaluated = eval.get_evaluated_schema(false);
    
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
    eval.evaluate(&data_str, None).unwrap();
    
    let evaluated = eval.get_evaluated_schema(false);
    
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
    eval.evaluate(&data_str, None).unwrap();
    
    let evaluated = eval.get_evaluated_schema(false);
    
    // Check all URLs
    assert_eq!(
        evaluated.pointer("/properties/users/options/url").and_then(|v| v.as_str()),
        Some("/api/users/42")
    );
    
    assert_eq!(
        evaluated.pointer("/properties/posts/options/url").and_then(|v| v.as_str()),
        Some("/api/posts/100/comments/5")
    );
    
    // Static URL should remain unchanged
    assert_eq!(
        evaluated.pointer("/properties/static/options/url").and_then(|v| v.as_str()),
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
