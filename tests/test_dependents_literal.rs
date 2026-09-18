use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_dependents_literal_clear_and_value() {
    let schema = json!({
        "form": {
            "type": "object",
            "properties": {
                "trigger_field": {
                    "type": "string",
                    "dependents": [
                        {
                            "$ref": "#/form/properties/target_clear",
                            "clear": true
                        },
                        {
                            "$ref": "#/form/properties/target_value_str",
                            "value": "custom_literal_string"
                        },
                        {
                            "$ref": "#/form/properties/target_value_num",
                            "value": 42
                        }
                    ]
                },
                "target_clear": {
                    "type": "string",
                    "dependents": [
                        {
                            "$ref": "#/form/properties/cascaded_clear",
                            "clear": true
                        }
                    ]
                },
                "cascaded_clear": {
                    "type": "string"
                },
                "target_value_str": {
                    "type": "string"
                },
                "target_value_num": {
                    "type": "number"
                }
            }
        }
    });

    let mut data = json!({
        "form": {
            "trigger_field": "initial",
            "target_clear": "has_value",
            "cascaded_clear": "also_has_value",
            "target_value_str": "old_val",
            "target_value_num": 0
        }
    });

    let mut eval = JSONEval::new(&schema.to_string(), None, Some(&data.to_string()))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data.to_string(), None, None, None)
        .expect("evaluate failed");

    // Change trigger_field
    data["form"]["trigger_field"] = "updated".into();

    let deps_result = eval
        .evaluate_dependents(
            &["form.trigger_field".to_string()],
            Some(&data.to_string()),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    let deps_array = deps_result.as_array().expect("deps should be array");

    // 1. target_clear should be cleared
    let clear_item = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("form.target_clear"));
    assert!(clear_item.is_some(), "form.target_clear should be present");
    assert_eq!(clear_item.unwrap().get("clear"), Some(&json!(true)));

    // 2. cascaded_clear should also be cleared transitively
    let cascaded_item = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("form.cascaded_clear"));
    assert!(cascaded_item.is_some(), "form.cascaded_clear should be cleared transitively");
    assert_eq!(cascaded_item.unwrap().get("clear"), Some(&json!(true)));

    // 3. target_value_str should have literal string "custom_literal_string"
    let val_str_item = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("form.target_value_str"));
    assert!(val_str_item.is_some(), "form.target_value_str should be present");
    assert_eq!(val_str_item.unwrap().get("value"), Some(&json!("custom_literal_string")));

    // 4. target_value_num should have literal number 42
    let val_num_item = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("form.target_value_num"));
    assert!(val_num_item.is_some(), "form.target_value_num should be present");
    assert_eq!(val_num_item.unwrap().get("value"), Some(&json!(42)));
}

#[test]
fn test_dependents_literal_clear_truthy_and_falsy() {
    let schema = json!({
        "form": {
            "type": "object",
            "properties": {
                "trigger": {
                    "type": "string",
                    "dependents": [
                        {
                            "$ref": "#/form/properties/field_false",
                            "clear": false
                        },
                        {
                            "$ref": "#/form/properties/field_str_true",
                            "clear": "true"
                        },
                        {
                            "$ref": "#/form/properties/field_num_one",
                            "clear": 1
                        }
                    ]
                },
                "field_false": { "type": "string" },
                "field_str_true": { "type": "string" },
                "field_num_one": { "type": "string" }
            }
        }
    });

    let mut data = json!({
        "form": {
            "trigger": "A",
            "field_false": "keep",
            "field_str_true": "remove1",
            "field_num_one": "remove2"
        }
    });

    let mut eval = JSONEval::new(&schema.to_string(), None, Some(&data.to_string()))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data.to_string(), None, None, None)
        .expect("evaluate failed");

    data["form"]["trigger"] = "B".into();

    let deps_result = eval
        .evaluate_dependents(
            &["form.trigger".to_string()],
            Some(&data.to_string()),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    let deps_array = deps_result.as_array().expect("deps should be array");

    // field_false should NOT be cleared
    assert!(
        deps_array
            .iter()
            .all(|item| item.get("$ref").and_then(|r| r.as_str()) != Some("form.field_false")),
        "field_false with clear: false must not be cleared"
    );

    // field_str_true should be cleared
    let str_true_item = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("form.field_str_true"));
    assert!(str_true_item.is_some(), "field_str_true should be cleared");
    assert_eq!(str_true_item.unwrap().get("clear"), Some(&json!(true)));

    // field_num_one should be cleared
    let num_one_item = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("form.field_num_one"));
    assert!(num_one_item.is_some(), "field_num_one should be cleared");
    assert_eq!(num_one_item.unwrap().get("clear"), Some(&json!(true)));
}
