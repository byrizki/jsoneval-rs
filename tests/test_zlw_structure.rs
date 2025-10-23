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

#[test]
fn test_direct_layout_path_values() {
    // Test to verify actual path values for direct layout elements
    let schema = json!({
        "type": "object",
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
    
    // Test FlexLayout container paths
    let flex_layout = evaluated
        .pointer("/illustration/properties/insured/$layout/elements/1")
        .expect("FlexLayout should exist");
    
    let fullpath = flex_layout.get("$fullpath").and_then(|v| v.as_str()).unwrap();
    let path = flex_layout.get("$path").and_then(|v| v.as_str()).unwrap();
    
    println!("FlexLayout $fullpath: {}", fullpath);
    println!("FlexLayout $path: {}", path);
    
    // Verify FlexLayout has proper path
    assert_eq!(
        fullpath,
        "illustration.properties.insured.$layout.elements.1",
        "FlexLayout should have full hierarchical path"
    );
    
    assert_eq!(
        path,
        "1",
        "FlexLayout $path should be its index"
    );
    
    println!("✅ Direct layout path values test passed!");
}

#[test]
fn test_evaluate_dependents_ins_dob_to_insage() {
    // Test ins_dob dependent evaluation to calculate insage using DATEDIF
    // This mimics the actual zlw.json structure where ins_dob updates insage
    let schema = json!({
        "illustration": {
            "type": "object",
            "properties": {
                "insured": {
                    "type": "object",
                    "title": "Tertanggung",
                    "properties": {
                        "ins_dob": {
                            "type": "string",
                            "title": "Tanggal Lahir Tertanggung",
                            "fieldType": "datepicker",
                            "rules": {
                                "required": {
                                    "value": true,
                                    "message": "VALIDATION_REQUIRED"
                                }
                            },
                            "dependents": [
                                {
                                    "$ref": "#/illustration/properties/insured/properties/insage",
                                    "value": {
                                        "$evaluation": {
                                            "DATEDIF": [
                                                {
                                                    "$ref": "$value"
                                                },
                                                {
                                                    "NOW": []
                                                },
                                                "Y"
                                            ]
                                        }
                                    }
                                },
                                {
                                    "$ref": "#/illustration/properties/product_benefit/properties/benefit_type/properties/prem_pay_period",
                                    "clear": {
                                        "$evaluation": true
                                    }
                                },
                                {
                                    "$ref": "#/illustration/properties/signature",
                                    "clear": {
                                        "$evaluation": true
                                    }
                                }
                            ]
                        },
                        "insage": {
                            "type": "number",
                            "title": "Usia Tertanggung"
                        }
                    }
                },
                "product_benefit": {
                    "type": "object",
                    "properties": {
                        "benefit_type": {
                            "type": "object",
                            "properties": {
                                "prem_pay_period": {
                                    "type": "string",
                                    "title": "Premium Payment Period"
                                }
                            }
                        }
                    }
                },
                "signature": {
                    "type": "string",
                    "title": "Signature"
                }
            }
        }
    });

    // Initial data with a date of birth
    let initial_data = json!({
        "illustration": {
            "insured": {
                "ins_dob": "1990-01-15",
                "insage": null
            },
            "product_benefit": {
                "benefit_type": {
                    "prem_pay_period": "10"
                }
            },
            "signature": "Initial Signature"
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let initial_data_str = serde_json::to_string(&initial_data).unwrap();

    let mut eval = JSONEval::new(&schema_str, None, Some(&initial_data_str))
        .expect("Failed to create JSONEval");

    // Perform initial evaluation
    eval.evaluate(&initial_data_str, None)
        .expect("Initial evaluation failed");

    // Update ins_dob to trigger dependent evaluation
    let mut updated_data = initial_data.clone();
    updated_data["illustration"]["insured"]["ins_dob"] = json!("2003-10-23T00:00:00.000Z");
    let updated_data_str = serde_json::to_string(&updated_data).unwrap();

    // Call evaluate_dependents for ins_dob field
    let result = eval.evaluate_dependents(
        &[String::from("#/illustration/properties/insured/properties/ins_dob")],
        Some(&updated_data_str),
        None,
        false, // Don't re-evaluate
    ).expect("evaluate_dependents failed");

    // Verify result structure
    assert!(result.is_array(), "Result should be an array");
    let changes = result.as_array().unwrap();
    
    // Should have at least 3 dependents (insage, prem_pay_period, signature)
    assert!(changes.len() >= 3, "Should have at least 3 dependent changes");

    // Find the insage change
    let insage_change = changes.iter()
        .find(|c| {
            c["$ref"].as_str()
                .map(|s| s.contains("insage"))
                .unwrap_or(false)
        })
        .expect("Should have insage in dependents");

    println!("insage_change: {:#?}", insage_change);
    // Verify insage change structure
    assert!(
        insage_change.get("value").is_some(),
        "insage change should have a value field"
    );

    // Verify the value is a number (age calculation result)
    let age_value = &insage_change["value"];
    assert!(
        age_value.is_number(),
        "insage value should be a number, got: {:?}",
        age_value
    );

    // If the value is a number, verify it's reasonable (between 0 and 150)
    if let Some(age) = age_value.as_i64() {
        assert!(
            age >= 0 && age <= 150,
            "Age should be reasonable (0-150), got: {}",
            age
        );
        println!("Calculated age from DATEDIF: {}", age);
    }

    // Verify transitive flag
    assert_eq!(
        insage_change["transitive"].as_bool(),
        Some(false),
        "insage should be a direct dependent (not transitive)"
    );

    // Verify $ref points to the correct field
    let ref_path = insage_change["$ref"].as_str().unwrap();
    assert!(
        ref_path.contains("insage") || ref_path == "illustration.insured.insage",
        "Reference should point to insage field, got: {}",
        ref_path
    );

    println!("✅ ins_dob -> insage dependent evaluation test passed!");
    println!("   Dependents returned: {}", changes.len());
}
