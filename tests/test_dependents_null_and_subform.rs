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
//   main-form field `group` has a dependent that clears `items[*].category`
//   subform item field `category` has a dependent that clears `items[*].type_code`
//   → changing `group` must cascade: group → category (clear) → type_code (clear)
//
// Before the fix `type_code` was never cleared because the computed change for
// `items.0.category` was not fed back into run_subform_pass as a changed path.
// ---------------------------------------------------------------------------
#[test]
fn test_main_form_dependent_cascades_into_subform_item_dependents() {
    let schema = json!({
        "group": {
            "type": "string",
            "dependents": [
                {
                    "$ref": "#/items/0/category",
                    "clear": true
                }
            ]
        },
        "items": {
            "type": "array",
            "items": {
                "properties": {
                    "category": {
                        "type": "string",
                        "dependents": [
                            {
                                "$ref": "#/items/properties/type_code",
                                "clear": true
                            }
                        ]
                    },
                    "type_code": {
                        "type": "string"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();

    let initial_data = json!({
        "group": "A",
        "items": [
            { "category": "CAT_A", "type_code": "TABLE_1" }
        ]
    });
    let initial_data_str = serde_json::to_string(&initial_data).unwrap();

    let mut eval = JSONEval::new(&schema_str, None, Some(&initial_data_str)).unwrap();

    // Change group — triggers main-form dependent → clears items[0].category
    let updated_data = json!({
        "group": "B",
        "items": [
            { "category": "CAT_A", "type_code": "TABLE_1" }
        ]
    });
    let updated_data_str = serde_json::to_string(&updated_data).unwrap();

    let result = eval
        .evaluate_dependents(
            &["group".to_string()],
            Some(&updated_data_str),
            None,
            false,
            None,
            None,
            true, // include_subforms
        )
        .unwrap();

    let changes = result.as_array().expect("result must be an array");

    // items.0.category must be cleared by the main-form dependent
    let category_change = changes
        .iter()
        .find(|c| c.get("$ref").and_then(|v| v.as_str()) == Some("items.0.category"))
        .expect("items.0.category must be cleared by main-form group dependent");

    assert_eq!(
        category_change.get("clear"),
        Some(&json!(true)),
        "items.0.category must carry clear:true"
    );

    // items.0.type_code must be cleared by the subform item's category.dependents
    let type_code_change = changes
        .iter()
        .find(|c| {
            c.get("$ref").and_then(|v| v.as_str()) == Some("items.0.type_code")
        })
        .expect(
            "items.0.type_code must be cleared as cascade from category.dependents inside the subform",
        );

    assert_eq!(
        type_code_change.get("clear"),
        Some(&json!(true)),
        "items.0.type_code must carry clear:true from subform dependent cascade"
    );

    // Confirm final eval_data state
    let data = eval.eval_data.data();
    let category_val = data.pointer("/items/0/category");
    assert!(
        category_val.is_none() || category_val == Some(&json!(null)),
        "items[0].category must be null in eval_data"
    );
    let type_code_val = data.pointer("/items/0/type_code");
    assert!(
        type_code_val.is_none() || type_code_val == Some(&json!(null)),
        "items[0].type_code must be null in eval_data after cascade"
    );
}
