use json_eval_rs::jsoneval::parsed_schema::ParsedSchema;
use json_eval_rs::JSONEval;
use serde_json::{json, Value};
use std::sync::Arc;

#[test]
fn cached_parsed_subform_resolves_parent_static_array() {
    // `rates` has >10 entries, so ParsedSchema extracts it and leaves a
    // `$static_array` marker in nested rider schemas.
    let schema = json!({
        "$params": { "rates": [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110] },
        "riders": {
            "type": "array",
            "items": {
                "properties": {
                    "rate_index": { "type": "number" },
                    "first_prem": {
                        "type": "number",
                        "value": {
                            "$evaluation": {
                                "VALUEAT": [
                                    { "$ref": "#/$params/rates" },
                                    { "$ref": "#/riders/properties/rate_index" }
                                ]
                            }
                        }
                    }
                }
            }
        }
    })
    .to_string();
    let data = json!({ "riders": [{ "rate_index": 3 }] }).to_string();
    let parsed = Arc::new(ParsedSchema::parse(&schema).expect("schema must parse"));
    let mut eval = JSONEval::with_parsed_schema(parsed, None, Some(&data))
        .expect("cached evaluator must initialize");

    eval.evaluate_subform("riders.0", &data, None, None, None)
        .expect("cached rider subform must evaluate");

    assert_eq!(
        eval.get_evaluated_schema_subform("riders.0")
            .pointer("/riders/properties/first_prem/value")
            .and_then(Value::as_i64),
        Some(40),
        "cached subform must resolve root static-array entries"
    );
}
