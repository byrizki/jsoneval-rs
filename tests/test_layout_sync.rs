use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_get_evaluated_schema_layout_sync() {
    // UNEVALUATED: Schema with layout containing a reference
    let schema = json!({
        "$params": {},
        "illustration": {
            "type": "object",
            "properties": {
                "hide_flag": {
                    "type": "boolean",
                    "title": "Hide Flag",
                    "default": false
                },
                "target_field": {
                    "type": "string",
                    "title": "Target Field",
                    "condition": {
                        "hidden": {
                            "$evaluation": {
                                "$ref": "#/illustration/properties/hide_flag"
                            }
                        }
                    }
                },
                "container": {
                    "type": "object",
                    "properties": {
                        "$layout": {
                            "elements": [
                                {
                                    "$ref": "#/illustration/properties/target_field"
                                }
                            ]
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    // EVALUATE: hide_flag = true
    // Expected: condition.hidden should be true
    eval.evaluate(r#"{"illustration": {"hide_flag": true}}"#, None, None, None)
        .unwrap();
    let result_true = eval.get_evaluated_schema_resolved();

    let layout_elem_true = result_true
        .pointer("/illustration/properties/container/properties/$layout/elements/0")
        .expect("Should have layout element");

    assert_field_matches(
        layout_elem_true,
        "Target Field",
        "string",
        true,
        "hide_flag=true",
    );

    // EVALUATE: hide_flag = false
    // Expected: condition.hidden should be false (layout should re-sync from updated evaluation)
    eval.evaluate(
        r#"{"illustration": {"hide_flag": false}}"#,
        None,
        None,
        None,
    )
    .unwrap();
    let result_false = eval.get_evaluated_schema_resolved();

    let layout_elem_false = result_false
        .pointer("/illustration/properties/container/properties/$layout/elements/0")
        .expect("Should have layout element");

    assert_field_matches(
        layout_elem_false,
        "Target Field",
        "string",
        false,
        "hide_flag=false",
    );

    println!("✓ Layout sync test passed: layout elements correctly sync with evaluation changes");
}

#[test]
fn test_get_evaluated_schema_root_layout_sync() {
    // UNEVALUATED: Schema with layout containing a reference
    let schema = json!({
        "type": "object",
        "properties": {
            "hide_flag": {
                "type": "boolean",
                "title": "Hide Flag",
                "default": false
            },
            "target_field": {
                "type": "string",
                "title": "Target Field",
                "condition": {
                    "hidden": {
                        "$evaluation": {
                            "$ref": "#/properties/hide_flag"
                        }
                    }
                }
            },
            "container": {
                "type": "object",
                "properties": {
                    "$layout": {
                        "elements": [
                            {
                                "$ref": "#/properties/target_field"
                            }
                        ]
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    // EVALUATE: hide_flag = true
    // Expected: condition.hidden should be true
    eval.evaluate(r#"{"hide_flag": true}"#, None, None, None)
        .unwrap();
    let result_true = eval.get_evaluated_schema_resolved();

    let layout_elem_true = result_true
        .pointer("/properties/container/properties/$layout/elements/0")
        .expect("Should have layout element");

    assert_field_matches(
        layout_elem_true,
        "Target Field",
        "string",
        true,
        "hide_flag=true",
    );

    // EVALUATE: hide_flag = false
    // Expected: condition.hidden should be false (layout should re-sync from updated evaluation)
    eval.evaluate(r#"{"hide_flag": false}"#, None, None, None)
        .unwrap();
    let result_false = eval.get_evaluated_schema_resolved();

    let layout_elem_false = result_false
        .pointer("/properties/container/properties/$layout/elements/0")
        .expect("Should have layout element");

    assert_field_matches(
        layout_elem_false,
        "Target Field",
        "string",
        false,
        "hide_flag=false",
    );

    println!("✓ Layout sync test passed: layout elements correctly sync with evaluation changes");
}

#[test]
fn dynamic_parent_hidden_ref_repopulates_when_visible() {
    let schema = json!({
        "type": "object",
        "properties": {
            "toggle": { "type": "boolean" },
            "section": {
                "type": "object",
                "condition": {
                    "hidden": {
                        "$evaluation": { "$ref": "#/properties/toggle" }
                    }
                },
                "$layout": {
                    "elements": [
                        { "$ref": "#/properties/target" }
                    ]
                }
            },
            "target": { "type": "string", "title": "Target", "value": "fallback", "rules": { "required": true } }
        },
        "$layout": {
            "elements": [
                { "$ref": "#/properties/section" }
            ]
        }
    });
    let schema = schema.to_string();
    let mut eval = JSONEval::new(&schema, None, None).unwrap();

    eval.evaluate(r#"{"toggle":true,"target":"value"}"#, None, None, None)
        .unwrap();
    assert!(
        !eval
            .get_schema_value_object()
            .as_object()
            .unwrap()
            .contains_key("target"),
        "getter must filter inherited-hidden ref"
    );
    assert!(
        !eval
            .validate(r#"{"toggle":true}"#, None, None, None)
            .unwrap()
            .errors
            .contains_key("target"),
        "validation must skip inherited-hidden ref"
    );
    let hidden = eval.get_evaluated_schema_resolved();
    assert_eq!(
        hidden.pointer("/$layout/elements/0/elements/0/condition/hidden"),
        Some(&json!(true)),
        "layout child should inherit hidden parent state"
    );

    eval.evaluate(r#"{"toggle":false,"target":"value"}"#, None, None, None)
        .unwrap();
    assert!(
        eval.get_schema_value_object()
            .as_object()
            .unwrap()
            .contains_key("target"),
        "getter must restore ref visibility after parent becomes visible"
    );
    assert!(
        eval.validate(r#"{"toggle":false}"#, None, None, None)
            .unwrap()
            .errors
            .contains_key("target"),
        "validation must restore ref rules after parent becomes visible"
    );
    let visible = eval.get_evaluated_schema_resolved();
    assert_eq!(
        visible.pointer("/$layout/elements/0/elements/0/condition/hidden"),
        None,
        "layout child must be repopulated from current ref when parent becomes visible"
    );
    assert_eq!(
        visible.pointer("/$layout/elements/0/elements/0/$parentHide"),
        Some(&json!(false))
    );
    assert_eq!(
        eval.get_evaluated_schema()
            .pointer("/properties/target/condition/hidden"),
        None,
        "inherited layout visibility must not mutate source field state"
    );
}

#[test]
fn shared_layout_ref_uses_each_attached_parent_visibility() {
    let schema = json!({
        "type": "object",
        "properties": {
            "hidden_section": {
                "type": "object",
                "condition": { "hidden": true },
                "$layout": { "elements": [{ "$ref": "#/properties/target" }] }
            },
            "visible_section": {
                "type": "object",
                "$layout": { "elements": [{ "$ref": "#/properties/target" }] }
            },
            "target": {
                "type": "string",
                "value": "fallback",
                "rules": { "required": true }
            }
        },
        "$layout": {
            "elements": [
                { "$ref": "#/properties/hidden_section" },
                { "$ref": "#/properties/visible_section" }
            ]
        }
    })
    .to_string();
    let mut eval = JSONEval::new(&schema, None, None).unwrap();

    eval.evaluate(r#"{"target":"value"}"#, None, None, None)
        .unwrap();

    let resolved = eval.get_evaluated_schema_resolved();
    assert_eq!(
        resolved.pointer("/$layout/elements/0/elements/0/$parentHide"),
        Some(&json!(true)),
        "shared ref under hidden parent must stay hidden in that layout occurrence"
    );
    assert_eq!(
        resolved.pointer("/$layout/elements/0/elements/0/condition/hidden"),
        Some(&json!(true))
    );
    assert_eq!(
        resolved.pointer("/$layout/elements/1/elements/0/$parentHide"),
        Some(&json!(false)),
        "shared ref under visible parent must remain visible in that layout occurrence"
    );
    assert_eq!(
        resolved.pointer("/$layout/elements/1/elements/0/condition/hidden"),
        None
    );
    assert!(
        eval.get_schema_value_object()
            .as_object()
            .unwrap()
            .contains_key("target"),
        "one visible attachment must keep schema getter value"
    );
    assert!(
        eval.validate("{}", None, None, None)
            .unwrap()
            .errors
            .contains_key("target"),
        "one visible attachment must keep required validation"
    );
}

#[test]
fn hidden_super_parent_cascades_through_deep_child_layouts() {
    let schema = json!({
        "type": "object",
        "properties": {
            "hide_root": { "type": "boolean" },
            "root": {
                "type": "object",
                "condition": {
                    "hidden": { "$evaluation": { "$ref": "#/properties/hide_root" } }
                },
                "$layout": { "elements": [{ "$ref": "#/properties/middle" }] }
            },
            "middle": {
                "type": "object",
                "$layout": {
                    "elements": [{
                        "type": "VerticalLayout",
                        "elements": [{ "$ref": "#/properties/leaf" }]
                    }]
                }
            },
            "leaf": {
                "type": "string",
                "value": "fallback",
                "rules": { "required": true }
            }
        },
        "$layout": { "elements": [{ "$ref": "#/properties/root" }] }
    })
    .to_string();
    let mut eval = JSONEval::new(&schema, None, None).unwrap();

    eval.evaluate(r#"{"hide_root":true,"leaf":"value"}"#, None, None, None)
        .unwrap();
    assert!(
        !eval
            .get_schema_value_object()
            .as_object()
            .unwrap()
            .contains_key("leaf"),
        "hidden super-parent must filter deeply attached layout ref"
    );
    assert!(
        !eval
            .validate(r#"{"hide_root":true}"#, None, None, None)
            .unwrap()
            .errors
            .contains_key("leaf"),
        "hidden super-parent must skip deeply attached ref validation"
    );
    let hidden = eval.get_evaluated_schema_resolved();
    assert_eq!(
        hidden.pointer("/$layout/elements/0/elements/0/elements/0/elements/0/$parentHide"),
        Some(&json!(true)),
        "super-parent hidden state must reach deepest layout ref"
    );

    eval.evaluate(r#"{"hide_root":false,"leaf":"value"}"#, None, None, None)
        .unwrap();
    assert!(
        eval.get_schema_value_object()
            .as_object()
            .unwrap()
            .contains_key("leaf"),
        "deep ref must become visible when super-parent does"
    );
    assert!(
        eval.validate(r#"{"hide_root":false}"#, None, None, None)
            .unwrap()
            .errors
            .contains_key("leaf"),
        "deep ref rules must resume when super-parent does"
    );
}

fn assert_field_matches(
    element: &serde_json::Value,
    expected_title: &str,
    expected_type: &str,
    expected_hidden: bool,
    case: &str,
) {
    assert_eq!(
        element.get("title").and_then(|v| v.as_str()),
        Some(expected_title),
        "Case {}: Title mismatch",
        case
    );

    assert_eq!(
        element.get("type").and_then(|v| v.as_str()),
        Some(expected_type),
        "Case {}: Type mismatch",
        case
    );

    assert_eq!(
        element
            .pointer("/condition/hidden")
            .and_then(|v| v.as_bool()),
        Some(expected_hidden),
        "Case {}: condition.hidden should be {} (layout should sync with evaluation)",
        case,
        expected_hidden
    );
}
