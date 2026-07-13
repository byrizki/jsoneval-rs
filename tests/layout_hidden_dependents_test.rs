use json_eval_rs::jsoneval::JSONEval;
use serde_json::json;

fn condition_hidden_layout_schema(keep_hidden_value: bool) -> serde_json::Value {
    json!({
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
                        {
                            "type": "VerticalLayout",
                            "elements": [
                                { "$ref": "#/properties/target" }
                            ]
                        }
                    ]
                }
            },
            "target": {
                "type": "string",
                "config": {
                    "all": { "keepHiddenValue": keep_hidden_value }
                }
            }
        },
        "$layout": {
            "elements": [
                { "$ref": "#/properties/section" }
            ]
        }
    })
}

#[test]
fn condition_hidden_layout_ref_clears_non_empty_dependent_data() {
    let schema = condition_hidden_layout_schema(false).to_string();
    let initial_data = r#"{"toggle":false,"target":"keep me"}"#;
    let changed_data = r#"{"toggle":true,"target":"keep me"}"#;
    let mut eval = JSONEval::new(&schema, None, Some(initial_data)).unwrap();

    eval.evaluate(initial_data, None, None, None).unwrap();
    let changes = eval
        .evaluate_dependents(
            &["toggle".to_string()],
            Some(changed_data),
            None,
            true,
            None,
            None,
            true,
        )
        .unwrap();

    assert!(
        changes.as_array().unwrap().iter().any(|change| {
            change.get("$ref").and_then(|value| value.as_str()) == Some("target")
                && change.get("$hidden") == Some(&json!(true))
                && change.get("clear") == Some(&json!(true))
        }),
        "condition-hidden layout ref must emit hidden clear event"
    );
    assert_eq!(eval.eval_data.data().pointer("/target"), Some(&json!(null)));
}

#[test]
fn inherited_condition_hidden_respects_keep_hidden_value() {
    let schema = condition_hidden_layout_schema(true).to_string();
    let initial_data = r#"{"toggle":false,"target":"keep me"}"#;
    let changed_data = r#"{"toggle":true,"target":"keep me"}"#;
    let mut eval = JSONEval::new(&schema, None, Some(initial_data)).unwrap();

    eval.evaluate(initial_data, None, None, None).unwrap();
    let changes = eval
        .evaluate_dependents(
            &["toggle".to_string()],
            Some(changed_data),
            None,
            true,
            None,
            None,
            true,
        )
        .unwrap();

    assert!(
        !changes.as_array().unwrap().iter().any(|change| {
            change.get("$ref").and_then(|value| value.as_str()) == Some("target")
                && change.get("clear") == Some(&json!(true))
        }),
        "keepHiddenValue must suppress inherited hidden clear"
    );
    assert_eq!(
        eval.eval_data.data().pointer("/target"),
        Some(&json!("keep me"))
    );
}

#[test]
fn layout_only_hidden_ref_does_not_clear_dependent_data() {
    let schema = json!({
        "type": "object",
        "properties": {
            "trigger": { "type": "string" },
            "target": { "type": "string" },
            "section": {
                "$layout": {
                    "hideLayout": { "all": true },
                    "elements": [
                        { "$ref": "#/properties/target" }
                    ]
                }
            }
        },
        "$layout": {
            "elements": [
                { "$ref": "#/properties/section" }
            ]
        }
    })
    .to_string();
    let data = r#"{"trigger":"changed","target":"keep me"}"#;
    let mut eval = JSONEval::new(&schema, None, Some(data)).unwrap();

    eval.evaluate(data, None, None, None).unwrap();
    let changes = eval
        .evaluate_dependents(
            &["trigger".to_string()],
            Some(data),
            None,
            true,
            None,
            None,
            true,
        )
        .unwrap();

    assert!(!changes.as_array().unwrap().iter().any(|change| {
        change.get("$ref").and_then(|value| value.as_str()) == Some("target")
            && change.get("clear") == Some(&json!(true))
    }));
    assert_eq!(
        eval.eval_data.data().pointer("/target"),
        Some(&json!("keep me"))
    );
}
