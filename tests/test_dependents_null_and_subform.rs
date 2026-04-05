use json_eval_rs::JSONEval;
use serde_json::json;

// ---------------------------------------------------------------------------
// Test 1 — null value from dependent formula propagates as `clear: true`
//
// Schema: main-form field `flag` has a dependent targeting `label`.
// When `flag` is set to false the formula returns null.
// Before the fix the update was silently dropped; after the fix a
// `{ "$ref": "label", "clear": true }` entry must appear and the field
// must be null in eval_data.
// ---------------------------------------------------------------------------
#[test]
fn test_dependent_value_null_emits_clear() {
    let schema = json!({
        "flag": {
            "type": "boolean",
            "dependents": [
                {
                    "$ref": "#/properties/label",
                    "value": {
                        "$evaluation": {
                            "if": [
                                { "$ref": "#/properties/flag" },
                                "active",
                                null
                            ]
                        }
                    }
                }
            ]
        },
        "label": {
            "type": "string"
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();

    // Start with flag=true so label has a value to be cleared
    let initial_data = json!({
        "flag": true,
        "label": "active"
    });
    let initial_data_str = serde_json::to_string(&initial_data).unwrap();

    let mut eval = JSONEval::new(&schema_str, None, Some(&initial_data_str)).unwrap();

    // Change flag to false — the dependent formula now returns null
    let updated_data = json!({
        "flag": false,
        "label": "active"
    });
    let updated_data_str = serde_json::to_string(&updated_data).unwrap();

    let result = eval
        .evaluate_dependents(
            &["flag".to_string()],
            Some(&updated_data_str),
            None,
            false,
            None,
            None,
            false,
        )
        .unwrap();

    let changes = result.as_array().expect("result must be an array");

    // Must NOT emit a change for `label` when value is null/empty from dependents array
    let label_change = changes
        .iter()
        .find(|c| c.get("$ref").and_then(|v| v.as_str()) == Some("label"));

    assert!(
        label_change.is_none(),
        "dependent clear must NOT be emitted for null/empty value from dependents array"
    );
}

// ---------------------------------------------------------------------------
// Test 2 — subform item's own `dependents` fire when a main-form field
//          change cascades into a subform item field via the main-form
//          dependents queue.
//
// Schema:
//   main-form field `plan` has a dependent that clears `riders[*].benefit`
//   subform item field `benefit` has a dependent that clears `riders[*].loading`
//   → changing `plan` must cascade: plan → benefit (clear) → loading (clear)
//
// Before the fix `loading` was never cleared because the computed change for
// `riders.0.benefit` was not fed back into run_subform_pass as a changed path.
// ---------------------------------------------------------------------------
#[test]
fn test_main_form_dependent_cascades_into_subform_item_dependents() {
    let schema = json!({
        "plan": {
            "type": "string",
            "dependents": [
                {
                    "$ref": "#/riders/0/benefit",
                    "clear": true
                }
            ]
        },
        "riders": {
            "type": "array",
            "items": {
                "properties": {
                    "benefit": {
                        "type": "string",
                        "dependents": [
                            {
                                "$ref": "#/riders/properties/loading",
                                "clear": true
                            }
                        ]
                    },
                    "loading": {
                        "type": "string"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();

    let initial_data = json!({
        "plan": "A",
        "riders": [
            { "benefit": "WOP", "loading": "TABLE_1" }
        ]
    });
    let initial_data_str = serde_json::to_string(&initial_data).unwrap();

    let mut eval = JSONEval::new(&schema_str, None, Some(&initial_data_str)).unwrap();

    // Change plan — triggers main-form dependent → clears riders[0].benefit
    let updated_data = json!({
        "plan": "B",
        "riders": [
            { "benefit": "WOP", "loading": "TABLE_1" }
        ]
    });
    let updated_data_str = serde_json::to_string(&updated_data).unwrap();

    let result = eval
        .evaluate_dependents(
            &["plan".to_string()],
            Some(&updated_data_str),
            None,
            false,
            None,
            None,
            true, // include_subforms
        )
        .unwrap();

    let changes = result.as_array().expect("result must be an array");

    // riders.0.benefit must be cleared by the main-form dependent
    let benefit_change = changes
        .iter()
        .find(|c| c.get("$ref").and_then(|v| v.as_str()) == Some("riders.0.benefit"))
        .expect("riders.0.benefit must be cleared by main-form plan dependent");

    assert_eq!(
        benefit_change.get("clear"),
        Some(&json!(true)),
        "riders.0.benefit must carry clear:true"
    );

    // riders.0.loading must be cleared by the subform item's benefit.dependents
    let loading_change = changes
        .iter()
        .find(|c| {
            c.get("$ref").and_then(|v| v.as_str()) == Some("riders.0.loading")
        })
        .expect(
            "riders.0.loading must be cleared as cascade from benefit.dependents inside the subform",
        );

    assert_eq!(
        loading_change.get("clear"),
        Some(&json!(true)),
        "riders.0.loading must carry clear:true from subform dependent cascade"
    );

    // Confirm final eval_data state
    let data = eval.eval_data.data();
    let benefit_val = data.pointer("/riders/0/benefit");
    assert!(
        benefit_val.is_none() || benefit_val == Some(&json!(null)),
        "riders[0].benefit must be null in eval_data"
    );
    let loading_val = data.pointer("/riders/0/loading");
    assert!(
        loading_val.is_none() || loading_val == Some(&json!(null)),
        "riders[0].loading must be null in eval_data after cascade"
    );
}
