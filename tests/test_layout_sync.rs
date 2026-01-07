use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_get_evaluated_schema_layout_sync() {
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
    eval.evaluate(r#"{"hide_flag": true}"#, None, None).unwrap();
    let result_true = eval.get_evaluated_schema(false);
    
    let layout_elem_true = result_true
        .pointer("/properties/container/properties/$layout/elements/0")
        .expect("Should have layout element");
    
    assert_field_matches(layout_elem_true, "Target Field", "string", true, "hide_flag=true");

    // EVALUATE: hide_flag = false  
    // Expected: condition.hidden should be false (layout should re-sync from updated evaluation)
    eval.evaluate(r#"{"hide_flag": false}"#, None, None).unwrap();
    let result_false = eval.get_evaluated_schema(false);
    
    let layout_elem_false = result_false
        .pointer("/properties/container/properties/$layout/elements/0")
        .expect("Should have layout element");
    
    assert_field_matches(layout_elem_false, "Target Field", "string", false, "hide_flag=false");

    println!("✓ Layout sync test passed: layout elements correctly sync with evaluation changes");
}

fn assert_field_matches(
    element: &serde_json::Value, 
    expected_title: &str,
    expected_type: &str,
    expected_hidden: bool,
    case: &str
) {
    assert_eq!(
        element.get("title").and_then(|v| v.as_str()),
        Some(expected_title),
        "Case {}: Title mismatch", case
    );
    
    assert_eq!(
        element.get("type").and_then(|v| v.as_str()),
        Some(expected_type),
        "Case {}: Type mismatch", case
    );
    
    assert_eq!(
        element.pointer("/condition/hidden").and_then(|v| v.as_bool()),
        Some(expected_hidden),
        "Case {}: condition.hidden should be {} (layout should sync with evaluation)",
        case, expected_hidden
    );
}
