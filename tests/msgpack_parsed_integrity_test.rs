use json_eval_rs::jsoneval::parsed_schema::ParsedSchema;
use json_eval_rs::JSONEval;
use serde_json::{json, Value};
use std::sync::Arc;

fn schema() -> Value {
    json!({
        "$params": { "internal": true },
        "form": {
            "type": "object",
            "properties": {
                "input": { "type": "number" },
                "computed": {
                    "type": "number",
                    "$evaluation": { "logic": { "+": [{ "var": "input" }, 2] } }
                }
            },
            "$layout": {
                "elements": [
                    { "$ref": "#/form/properties/computed" }
                ]
            }
        }
    })
}

fn evaluate_outputs(mut eval: JSONEval, data: &str) -> (Value, Value, Value) {
    eval.evaluate(data, None, None, None).unwrap();

    let compact = eval.get_evaluated_schema();
    let resolved = eval.get_evaluated_schema_resolved();
    let msgpack_output: Value =
        rmp_serde::from_slice(&eval.get_evaluated_schema_msgpack().unwrap())
            .expect("evaluated schema MessagePack must decode into JSON value");
    let resolved_msgpack_output: Value =
        rmp_serde::from_slice(&eval.get_evaluated_schema_resolved_msgpack().unwrap())
            .expect("resolved evaluated schema MessagePack must decode into JSON value");

    assert_eq!(
        resolved_msgpack_output, resolved,
        "resolved MessagePack must serialize existing resolved output"
    );

    (compact, resolved, msgpack_output)
}

#[test]
fn json_msgpack_and_parsed_paths_preserve_compact_and_resolved_schema_contracts() {
    let schema = schema();
    let schema_json = schema.to_string();
    let schema_msgpack = rmp_serde::to_vec(&schema).unwrap();
    let data = json!({ "input": 40 }).to_string();

    let direct = evaluate_outputs(
        JSONEval::new(&schema_json, None, Some(&data)).unwrap(),
        &data,
    );
    let msgpack = evaluate_outputs(
        JSONEval::new_from_msgpack(&schema_msgpack, None, Some(&data)).unwrap(),
        &data,
    );

    let parsed_json = Arc::new(ParsedSchema::parse(&schema_json).unwrap());
    let parsed = evaluate_outputs(
        JSONEval::with_parsed_schema(parsed_json, None, Some(&data)).unwrap(),
        &data,
    );

    let parsed_msgpack = Arc::new(ParsedSchema::parse_msgpack(&schema_msgpack).unwrap());
    let parsed_from_msgpack = evaluate_outputs(
        JSONEval::with_parsed_schema(parsed_msgpack, None, Some(&data)).unwrap(),
        &data,
    );

    for actual in [&msgpack, &parsed, &parsed_from_msgpack] {
        assert_eq!(actual.0, direct.0, "compact output must match JSON path");
        assert_eq!(actual.1, direct.1, "resolved output must match JSON path");
        assert_eq!(
            actual.2, direct.2,
            "MessagePack output must match JSON path"
        );
    }

    let (compact, resolved, compact_msgpack) = direct;
    assert_eq!(
        compact_msgpack, compact,
        "MessagePack getter serializes compact schema"
    );
    assert_eq!(compact.pointer("/$params/internal"), Some(&json!(true)));
    assert_eq!(
        compact.pointer("/form/$layout/elements/0/$ref"),
        Some(&json!("#/form/properties/computed"))
    );
    assert_eq!(
        compact.pointer("/form/properties/computed"),
        Some(&json!(42))
    );

    assert!(resolved.get("$params").is_none());
    assert!(resolved.pointer("/form/$layout/elements/0/$ref").is_none());
    assert_eq!(
        resolved.pointer("/form/properties/computed"),
        Some(&json!(42))
    );
}

#[test]
fn reload_parsed_schema_preserves_layout_metadata_for_schema_value_results() {
    let schema = json!({
        "type": "object",
        "properties": {
            "input": { "type": "number" },
            "computed": {
                "type": "number",
                "value": {
                    "$evaluation": { "logic": { "+": [{ "var": "input" }, 2] } }
                }
            }
        },
        "$layout": {
            "elements": [
                { "$ref": "#/properties/computed" }
            ]
        }
    });
    let schema_json = schema.to_string();
    let data = json!({ "input": 40, "computed": 999 }).to_string();

    let mut direct = JSONEval::new(&schema_json, None, Some(&data)).unwrap();
    direct.evaluate(&data, None, None, None).unwrap();
    let direct_values = direct.get_schema_value();

    let parsed = Arc::new(ParsedSchema::parse(&schema_json).unwrap());
    let mut reloaded = JSONEval::new(&"{}", None, None).unwrap();
    reloaded
        .reload_schema_parsed(parsed, None, Some(&data))
        .unwrap();
    reloaded.evaluate(&data, None, None, None).unwrap();
    let reloaded_values = reloaded.get_schema_value();

    assert_eq!(direct_values.pointer("/computed"), Some(&json!(999)));
    assert_eq!(reloaded_values, direct_values);
}

#[test]
fn malformed_msgpack_reports_errors_from_direct_and_parsed_constructors() {
    let invalid = [0xc1];

    let direct_error = match JSONEval::new_from_msgpack(&invalid, None, None) {
        Ok(_) => panic!("invalid MessagePack must fail direct construction"),
        Err(error) => error,
    };
    assert!(direct_error.contains("Failed to deserialize MessagePack schema"));

    let parsed_error = match ParsedSchema::parse_msgpack(&invalid) {
        Ok(_) => panic!("invalid MessagePack must fail ParsedSchema parsing"),
        Err(error) => error,
    };
    assert!(parsed_error.contains("Failed to deserialize MessagePack schema"));
}
