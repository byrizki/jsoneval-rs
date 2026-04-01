use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_cascade_dependents_via_formula_reference() {
    let schema = json!({
        "type": "object",
        "properties": {
            "form": {
                "type": "object",
                "properties": {
                    "trigger_field": { "type": "string" },
                    "source_field": {
                        "type": "string",
                        "dependents": [
                            {
                                "$ref": "#/properties/form/properties/target_field",
                                "value": {
                                    "$evaluation": {
                                        "if": [
                                            { "==": [{ "$ref": "#/properties/form/properties/trigger_field" }, "ACTIVE"] },
                                            { "$ref": "$value" },
                                            null
                                        ]
                                    }
                                }
                            }
                        ]
                    },
                    "target_field": { "type": "string" }
                }
            }
        }
    });

    let mut data = json!({
        "form": {
            "trigger_field": "INACTIVE",
            "source_field": "COPY_ME"
        }
    });

    let mut eval = JSONEval::new(&schema.to_string(), None, Some(&data.to_string()))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data.to_string(), None, None, None)
        .expect("evaluate failed");

    // Change trigger_field to ACTIVE
    data["form"]["trigger_field"] = "ACTIVE".into();

    // Trigger evaluate_dependents on trigger_field
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

    // We expect target_field to be updated to "COPY_ME"
    let matched_target = deps_array.iter().find(|item| {
        item.get("$ref").and_then(|r| r.as_str()) == Some("form.target_field")
    });

    assert!(
        matched_target.is_some(),
        "target_field should have been updated transitively"
    );

    let target_val = matched_target.unwrap().get("value").unwrap();
    assert_eq!(
        target_val,
        &json!("COPY_ME"),
        "target_field should receive the value from source_field"
    );
}

#[test]
fn test_self_cycle_dependents() {
    let schema = json!({
        "type": "object",
        "properties": {
            "form": {
                "type": "object",
                "properties": {
                    "trigger_field": { "type": "string" },
                    "source_field": {
                        "type": "string",
                        "dependents": [
                            {
                                "$ref": "#/properties/form/properties/trigger_field",
                                "value": {
                                    "$evaluation": {
                                        "if": [
                                            { "==": [{ "$ref": "#/properties/form/properties/trigger_field" }, "RESET"] },
                                            "RESET_DONE",
                                            null
                                        ]
                                    }
                                }
                            }
                        ]
                    }
                }
            }
        }
    });

    let mut data = json!({
        "form": {
            "trigger_field": "A",
            "source_field": "SRC"
        }
    });

    let mut eval = JSONEval::new(&schema.to_string(), None, Some(&data.to_string()))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data.to_string(), None, None, None)
        .expect("evaluate failed");

    // Trigger trigger_field change to RESET
    data["form"]["trigger_field"] = "RESET".into();

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

    // We expect trigger_field NOT to be in the deps_array, as it's the original trigger field
    // It should be skipped due to the processed set.
    let self_reference = deps_array.iter().find(|item| {
        item.get("$ref").and_then(|r| r.as_str()) == Some("form.trigger_field")
    });

    assert!(
        self_reference.is_none(),
        "trigger_field must not appear as a transitive dependent of itself"
    );
}
