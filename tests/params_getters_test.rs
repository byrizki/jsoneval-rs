use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_get_plain_and_evaluated_params_with_and_without_static_arrays() {
    let schema = json!({
        "$params": {
            "metadata": "form_v1",
            "constants": {
                "TAX_RATE": 0.11
            },
            "references": {
                "SMALL_LIST": ["a", "b", "c"],
                "LARGE_TABLE": [
                    {"id": 1, "val": 10},
                    {"id": 2, "val": 20},
                    {"id": 3, "val": 30},
                    {"id": 4, "val": 40},
                    {"id": 5, "val": 50},
                    {"id": 6, "val": 60},
                    {"id": 7, "val": 70},
                    {"id": 8, "val": 80},
                    {"id": 9, "val": 90},
                    {"id": 10, "val": 100},
                    {"id": 11, "val": 110},
                    {"id": 12, "val": 120}
                ]
            }
        },
        "properties": {
            "rate": {
                "type": "number",
                "value": {
                    "$evaluation": {
                        "var": "$params.constants.TAX_RATE"
                    }
                }
            }
        }
    })
    .to_string();

    let mut eval = JSONEval::new(&schema, None, None).expect("Should parse schema successfully");

    // 1. Check plain params (always clean, no static array data)
    let plain = eval.get_plain_params().expect("Should return plain params");

    assert_eq!(
        plain.pointer("/metadata").and_then(|v| v.as_str()),
        Some("form_v1")
    );
    assert_eq!(
        plain
            .pointer("/constants/TAX_RATE")
            .and_then(|v| v.as_f64()),
        Some(0.11)
    );
    assert_eq!(
        plain
            .pointer("/references/SMALL_LIST")
            .and_then(|v| v.as_array())
            .map(|a| a.len()),
        Some(3),
        "Small arrays (<= 10 items) must remain intact"
    );
    assert!(
        plain.pointer("/references/LARGE_TABLE").is_none(),
        "Static array (> 10 items) must be stripped in plain params"
    );

    // 2. Evaluate the form
    eval.evaluate("{}", None, None, None)
        .expect("Evaluation must succeed");

    // 3. Check evaluated params WITHOUT static arrays
    let eval_without = eval
        .get_evaluated_params(false)
        .expect("Should return evaluated params");

    assert_eq!(
        eval_without.pointer("/metadata").and_then(|v| v.as_str()),
        Some("form_v1")
    );
    assert!(
        eval_without.pointer("/references/LARGE_TABLE").is_none(),
        "Static array must be stripped in evaluated params when with_static_array = false"
    );

    // 4. Check evaluated params WITH static arrays
    let eval_with = eval
        .get_evaluated_params(true)
        .expect("Should return evaluated params");

    assert_eq!(
        eval_with
            .pointer("/references/LARGE_TABLE")
            .and_then(|v| v.as_array())
            .map(|a| a.len()),
        Some(12),
        "Static array must be resolved in evaluated params when with_static_array = true"
    );
}

#[test]
fn test_get_params_on_schema_without_params() {
    let schema = json!({
        "properties": {
            "name": { "type": "string" }
        }
    })
    .to_string();

    let mut eval = JSONEval::new(&schema, None, None).expect("Should parse schema successfully");

    assert!(
        eval.get_plain_params().is_none(),
        "get_plain_params must return None when $params does not exist"
    );
    assert!(
        eval.get_evaluated_params(false).is_none(),
        "get_evaluated_params must return None when $params does not exist"
    );
    assert!(
        eval.get_evaluated_params(true).is_none(),
        "get_evaluated_params must return None when $params does not exist"
    );
}

#[test]
fn test_subform_params() {
    let schema = json!({
        "$params": {
            "sub_meta": "sub_v1",
            "list": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
        },
        "benefits": {
            "type": "array",
            "items": {
                "properties": {
                    "field": { "type": "string", "value": "test" }
                }
            }
        }
    })
    .to_string();

    let mut eval = JSONEval::new(&schema, None, None).expect("Should parse schema successfully");

    // Subform plain params
    let plain_sub = eval
        .get_plain_params_subform("#/benefits")
        .expect("Should get subform plain params");
    assert_eq!(
        plain_sub.pointer("/sub_meta").and_then(|v| v.as_str()),
        Some("sub_v1")
    );
    assert!(plain_sub.pointer("/list").is_none());

    // Subform evaluated params
    let eval_sub_without = eval
        .get_evaluated_params_subform("#/benefits", false)
        .expect("Should get subform evaluated params without static arrays");
    assert!(eval_sub_without.pointer("/list").is_none());

    let eval_sub_with = eval
        .get_evaluated_params_subform("#/benefits", true)
        .expect("Should get subform evaluated params with static arrays");
    assert_eq!(
        eval_sub_with
            .pointer("/list")
            .and_then(|v| v.as_array())
            .map(|a| a.len()),
        Some(11)
    );
}
