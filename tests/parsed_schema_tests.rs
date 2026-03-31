use json_eval_rs::jsoneval::parsed_schema::ParsedSchema;
use json_eval_rs::JSONEval;
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_parsed_schema_basic() {
    let schema_str = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string"
            },
            "age": {
                "type": "number",
                "$evaluation": { "logic": { "+": [20, 10] } }
            }
        }
    })
    .to_string();

    let parsed = ParsedSchema::parse(&schema_str).expect("Failed to parse schema");
    let parsed_arc = Arc::new(parsed);

    let mut eval = JSONEval::with_parsed_schema(
        Arc::clone(&parsed_arc),
        None,
        Some(&json!({}).to_string()),
    )
    .expect("Failed to initialize JSONEval");

    eval.evaluate("{}", None, None, None).expect("Failed to evaluate");

    let result = eval.get_evaluated_schema(false);

    assert_eq!(
        result.pointer("/properties/age"),
        Some(&json!(30))
    );
}

#[test]
fn test_parsed_schema_static_arrays() {
    // Generate a list of 15 elements to trigger the static arrays logic (>10 elements)
    let large_array: Vec<serde_json::Value> = (0..15).map(|i| json!({ "val": i })).collect();

    let schema_str = json!({
        "$params": {
            "references": {
                "MY_TABLE": large_array
            },
            "others": {
                "COMPUTED": {
                    "$evaluation": {
                        "logic": {
                            "VALUEAT": [
                                { "$ref": "#/$params/references/MY_TABLE" },
                                5,
                                "val"
                            ]
                        }
                    }
                }
            }
        },
        "type": "object",
        "properties": {
            "result": {
                "type": "number",
                "$evaluation": { "logic": { "$ref": "#/$params/others/COMPUTED" } }
            }
        }
    })
    .to_string();

    let parsed = ParsedSchema::parse(&schema_str).expect("Failed to parse schema");
    
    // Assert that the static array was successfully extracted
    assert!(parsed.static_arrays.contains_key("/$params/references/MY_TABLE"));
    assert_eq!(parsed.static_arrays.len(), 1);

    // Initialize evaluation with parsed schema
    let parsed_arc = Arc::new(parsed);
    let mut eval = JSONEval::with_parsed_schema(
        Arc::clone(&parsed_arc),
        None,
        Some(&json!({}).to_string()),
    )
    .expect("Failed to initialize JSONEval");

    eval.evaluate("{}", None, None, None).expect("Failed to evaluate");
    let result = eval.get_evaluated_schema(false);

    // The result should have successfully evaluated VALUEAT which looks up index 5 of large array
    assert_eq!(
        result.pointer("/$params/others/COMPUTED"),
        Some(&json!(5))
    );
    assert_eq!(
        result.pointer("/properties/result"),
        Some(&json!(5))
    );
}

#[test]
fn test_parsed_schema_reuse_multiple_evaluators() {
    let schema_str = json!({
        "type": "object",
        "properties": {
            "result": {
                "type": "number",
                "$evaluation": { "logic": { "*": [{ "var": "input_val" }, 2] } }
            }
        }
    })
    .to_string();

    let parsed = Arc::new(ParsedSchema::parse(&schema_str).expect("Failed to parse schema"));

    // Evaluator 1
    let mut eval1 = JSONEval::with_parsed_schema(
        Arc::clone(&parsed),
        None,
        Some(&json!({"input_val": 10}).to_string()),
    )
    .expect("Failed to initialize eval1");

    eval1.evaluate("{\"input_val\": 10}", None, None, None).expect("Failed to evaluate eval1");

    // Evaluator 2
    let mut eval2 = JSONEval::with_parsed_schema(
        Arc::clone(&parsed),
        None,
        Some(&json!({"input_val": 25}).to_string()),
    )
    .expect("Failed to initialize eval2");

    eval2.evaluate("{\"input_val\": 25}", None, None, None).expect("Failed to evaluate eval2");

    assert_eq!(
        eval1.get_evaluated_schema(false).pointer("/properties/result"),
        Some(&json!(20)) // 10 * 2
    );

    assert_eq!(
        eval2.get_evaluated_schema(false).pointer("/properties/result"),
        Some(&json!(50)) // 25 * 2
    );
}
