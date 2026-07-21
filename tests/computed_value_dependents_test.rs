use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn computed_value_refreshes_earlier_hidden_condition() {
    let schema = json!({
        "type": "object",
        "properties": {
            "dependent": {
                "type": "string",
                "condition": {
                    "hidden": {
                        "$evaluation": {
                            "!=": [{ "$ref": "#/properties/source" }, true]
                        }
                    }
                }
            },
            "source": {
                "type": "boolean",
                "value": { "$evaluation": true }
            }
        }
    })
    .to_string();

    let mut eval = JSONEval::new(&schema, None, Some("{}")).unwrap();
    eval.evaluate("{}", None, None, None).unwrap();

    assert_eq!(
        eval.get_evaluated_schema()
            .pointer("/properties/dependent/condition/hidden"),
        Some(&json!(false)),
        "computed source must refresh earlier condition depending on it"
    );
}
