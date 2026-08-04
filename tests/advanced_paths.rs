use json_eval_rs::{JSONEval, ReturnFormat};
use serde_json::json;

fn evaluator_with_static_array() -> JSONEval {
    let schema = json!({
        "$params": {
            "references": {
                "RIDER_TABLE": {
                    "$table": [
                        { "$repeat": [0, 2, { "PREM_RIDER_PER_PAY": { "$evaluation": { "$ref": "$iteration" } } }] }
                    ]
                }
            }
        }
    });
    let schema = schema.to_string();
    let mut evaluator = JSONEval::new(&schema, None, None).expect("create evaluator");
    evaluator
        .evaluate("{}", None, None, None)
        .expect("evaluate");
    evaluator
}

#[test]
fn evaluated_schema_path_resolves_static_array_row() {
    let mut evaluator = evaluator_with_static_array();

    assert_eq!(
        evaluator.get_evaluated_schema_by_path("$params.references.RIDER_TABLE.1"),
        Some(json!({ "PREM_RIDER_PER_PAY": 1 }))
    );
}

#[test]
fn evaluated_schema_path_resolves_static_array_cell() {
    let mut evaluator = evaluator_with_static_array();

    assert_eq!(
        evaluator
            .get_evaluated_schema_by_path("$params.references.RIDER_TABLE.1.PREM_RIDER_PER_PAY"),
        Some(json!(1))
    );
}

#[test]
fn evaluated_schema_paths_resolve_static_array_cells() {
    let mut evaluator = evaluator_with_static_array();
    let paths = vec![
        "$params.references.RIDER_TABLE.1.PREM_RIDER_PER_PAY".to_string(),
        "$params.references.RIDER_TABLE.2.PREM_RIDER_PER_PAY".to_string(),
    ];

    assert_eq!(
        evaluator.get_evaluated_schema_by_paths(&paths, Some(ReturnFormat::Array)),
        json!([1, 2])
    );
}
