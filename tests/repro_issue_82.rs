use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_repro_missing_properties_with_layout() {
    let schema = json!({
        "definitions": {
            "person": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "age": { "type": "number" }
                },
                "$layout": {
                    "type": "object",
                    // Some layout stuff
                    "elements": []
                }
            }
        },
        "$layout": {
            "type": "object",
            "elements": [
                {
                    "$ref": "#/definitions/person"
                }
            ]
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    // JSONEval::new takes strings, not Arc<ParsedSchema>
    let mut je = JSONEval::new(&schema_str, None, None).unwrap();

    je.evaluate("{}", None, None).unwrap();

    // Use get_evaluated_schema(false) to ensure layout is resolved and full schema is returned
    let evaluated = je.get_evaluated_schema(false);
    
    // The resolved element should be at /$layout/elements/0
    let element = evaluated.pointer("/$layout/elements/0").expect("Element not found");
    
    let props = element.get("properties");

    // This assertion is expected to FAIL before the fix
    assert!(props.is_some(), "Properties map should be preserved but was removed by layout resolution");
    
    if let Some(props) = props {
        assert!(props.get("name").is_some(), "Property 'name' should exist");
        assert!(props.get("age").is_some(), "Property 'age' should exist");
    }
}
