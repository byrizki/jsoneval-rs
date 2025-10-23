use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_zlw_like_structure_metadata() {
    // Test structure similar to zlw.json with nested FlexLayout
    let schema = json!({
        "type": "object",
        "properties": {
            "illustration": {
                "type": "object",
                "properties": {
                    "insured": {
                        "type": "object",
                        "title": "Tertanggung",
                        "$layout": {
                            "type": "VerticalLayout",
                            "elements": [
                                {
                                    "$ref": "#/illustration/properties/insured/properties/ins_corrname"
                                },
                                {
                                    "type": "FlexLayout",
                                    "elements": [
                                        {
                                            "$ref": "#/illustration/properties/insured/properties/ins_dob"
                                        },
                                        {
                                            "$ref": "#/illustration/properties/insured/properties/insage"
                                        }
                                    ]
                                }
                            ]
                        },
                        "properties": {
                            "ins_corrname": {
                                "type": "string",
                                "title": "Nama Tertanggung"
                            },
                            "ins_dob": {
                                "type": "string",
                                "title": "Tanggal Lahir"
                            },
                            "insage": {
                                "type": "number",
                                "title": "Usia"
                            }
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None).unwrap();
    
    let evaluated = eval.get_evaluated_schema(false);
    
    // Test 1: First element with $ref should have populated metadata
    let first_element = evaluated
        .pointer("/properties/illustration/properties/insured/$layout/elements/0")
        .expect("First element should exist");
    
    assert_eq!(
        first_element.get("$fullpath").and_then(|v| v.as_str()),
        Some("illustration.properties.insured.properties.ins_corrname"),
        "Element with $ref should have populated $fullpath"
    );
    
    assert_eq!(
        first_element.get("$path").and_then(|v| v.as_str()),
        Some("ins_corrname"),
        "Element with $ref should have populated $path"
    );
    
    // Test 2: FlexLayout container (without $ref) should have metadata fields
    let flex_layout = evaluated
        .pointer("/properties/illustration/properties/insured/$layout/elements/1")
        .expect("FlexLayout should exist");
    
    assert!(
        flex_layout.get("$parentHide").is_some(),
        "FlexLayout should have $parentHide field"
    );
    
    assert!(
        flex_layout.get("$path").is_some(),
        "FlexLayout should have $path field"
    );
    
    assert!(
        flex_layout.get("$fullpath").is_some(),
        "FlexLayout should have $fullpath field"
    );
    
    assert_eq!(
        flex_layout.get("$parentHide").and_then(|v| v.as_bool()),
        Some(false),
        "FlexLayout $parentHide should be false"
    );
    
    // Test 3: Children inside FlexLayout should have populated metadata
    let flex_child1 = evaluated
        .pointer("/properties/illustration/properties/insured/$layout/elements/1/elements/0")
        .expect("First FlexLayout child should exist");
    
    assert_eq!(
        flex_child1.get("$fullpath").and_then(|v| v.as_str()),
        Some("illustration.properties.insured.properties.ins_dob"),
        "FlexLayout child should have populated $fullpath"
    );
    
    assert_eq!(
        flex_child1.get("$path").and_then(|v| v.as_str()),
        Some("ins_dob"),
        "FlexLayout child should have populated $path"
    );
    
    let flex_child2 = evaluated
        .pointer("/properties/illustration/properties/insured/$layout/elements/1/elements/1")
        .expect("Second FlexLayout child should exist");
    
    assert_eq!(
        flex_child2.get("$fullpath").and_then(|v| v.as_str()),
        Some("illustration.properties.insured.properties.insage"),
        "FlexLayout child should have populated $fullpath"
    );
    
    assert_eq!(
        flex_child2.get("$path").and_then(|v| v.as_str()),
        Some("insage"),
        "FlexLayout child should have populated $path"
    );
    
    println!("✅ ZLW-like structure test passed!");
}
