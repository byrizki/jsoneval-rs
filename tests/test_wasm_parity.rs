#[cfg(feature = "wasm")]
use json_eval_rs::wasm::types::JSONEvalWasm;
#[cfg(feature = "wasm")]
use serde_json::json;

#[cfg(feature = "wasm")]
#[test]
fn test_wasm_methods_parity() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "options": ["A", "B"]
            },
            "riders": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" }
                    }
                },
                "$layout": {
                    "type": "VerticalLayout",
                    "elements": [
                        { "$ref": "#/riders/properties/name" }
                    ]
                }
            },
            "form": {
                "$layout": {
                    "type": "VerticalLayout",
                    "elements": [
                        { "$ref": "name" }
                    ]
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();

    // 1. WASM constructor
    let mut wasm_eval =
        JSONEvalWasm::new(&schema_str, None, None).expect("Failed to create WASM instance");

    // 2. evaluate
    let data = json!({
        "riders": [{"name": "Rider 1"}]
    });
    let data_str = serde_json::to_string(&data).unwrap();
    wasm_eval
        .evaluate(&data_str, None, None)
        .expect("Evaluation failed");

    // 3. getEvaluatedSchema (returns String in WASM)
    let schema_json = wasm_eval.get_evaluated_schema();
    let schema_val: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
    assert!(schema_val.pointer("/properties/name").is_some());

    // 3b. get_evaluated_schema_resolved_to_value (Rust test helper)
    let schema_resolved = wasm_eval.get_evaluated_schema_resolved_to_value();
    assert!(schema_resolved
        .pointer("/properties/form/$layout/elements/0/$fullpath")
        .is_some());

    // 3c. resolve_layout_to_value (Rust test helper)
    let layout_overlays = wasm_eval.resolve_layout_to_value(false);
    assert!(layout_overlays.as_array().unwrap().len() > 0);

    // 4. getFieldOptions (returns Option<String> in WASM)
    let options_json = wasm_eval
        .get_field_options("properties.name")
        .expect("Options missing");
    let options_val: serde_json::Value = serde_json::from_str(&options_json).unwrap();
    assert_eq!(options_val, json!(["A", "B"]));

    // 5. Subform methods
    let subform_path = "#/riders/0";

    // 5a. getEvaluatedSchemaSubform (returns String in WASM)
    let subform_schema_json = wasm_eval.get_evaluated_schema_subform(subform_path);
    let subform_schema: serde_json::Value = serde_json::from_str(&subform_schema_json).unwrap();
    assert!(subform_schema.get("riders").is_some());

    // 5b. get_evaluated_schema_resolved_subform_to_value (Rust test helper)
    let subform_resolved = wasm_eval.get_evaluated_schema_resolved_subform_to_value(subform_path);
    assert!(subform_resolved
        .pointer("/riders/$layout/elements/0/$fullpath")
        .is_some());

    // 5c. hasSubform
    assert!(wasm_eval.has_subform("#/riders"));

    // 6. validate_to_value with include_subforms
    let v_schema = json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "rules": { "required": { "value": true, "message": "Title is required" } }
            },
            "contacts": {
                "type": "array",
                "items": {
                    "properties": {
                        "phone": {
                            "type": "string",
                            "rules": { "required": { "value": true, "message": "Phone is required" } }
                        }
                    }
                }
            }
        }
    });
    let mut wasm_val_eval = JSONEvalWasm::new(&v_schema.to_string(), None, None).unwrap();
    let v_data = json!({
        "title": "",
        "contacts": [{ "phone": "" }]
    }).to_string();

    let res_without = wasm_val_eval.validate_to_value(&v_data, None, None, None, Some(false)).unwrap();
    assert_eq!(res_without["has_error"], true);
    assert!(res_without["error"]["title"].is_object());
    assert!(res_without["error"]["contacts.0.phone"].is_null());

    let res_with = wasm_val_eval.validate_to_value(&v_data, None, None, None, Some(true)).unwrap();
    assert_eq!(res_with["has_error"], true);
    assert!(res_with["error"]["title"].is_object());
    assert!(res_with["error"]["contacts.0.phone"].is_object());
    assert_eq!(res_with["error"]["contacts.0.phone"]["code"], "contacts.0.phone.required");
}
