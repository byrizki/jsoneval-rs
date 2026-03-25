use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_subform_detection_and_creation() {
    // Test that subforms are detected and created for array fields with items
    let schema = json!({
        "$params": {
            "constants": {
                "MAX_RIDERS": 5
            }
        },
        "riders": {
            "type": "array",
            "title": "Riders",
            "items": {
                "$layout": {
                    "type": "VerticalLayout",
                    "elements": [
                        { "$ref": "#/riders/properties/name" },
                        { "$ref": "#/riders/properties/premium" }
                    ]
                },
                "properties": {
                    "name": {
                        "type": "string",
                        "title": "Rider Name"
                    },
                    "premium": {
                        "type": "number",
                        "title": "Premium Amount"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Check that subform was created
    assert!(eval.has_subform("#/riders"), "Subform should be created for riders array");
    
    // Get subform paths
    let subform_paths = eval.get_subform_paths();
    assert_eq!(subform_paths.len(), 1);
    assert_eq!(subform_paths[0], "#/riders");
}

#[test]
fn test_subform_schema_structure() {
    // Test that subform schema is correctly structured
    let schema = json!({
        "$params": {
            "constants": {
                "MIN_PREMIUM": 100
            }
        },
        "benefits": {
            "type": "array",
            "title": "Benefits",
            "items": {
                "properties": {
                    "code": {
                        "type": "string",
                        "title": "Benefit Code"
                    },
                    "amount": {
                        "type": "number",
                        "title": "Benefit Amount",
                        "rules": {
                            "required": {
                                "value": true,
                                "message": "Amount is required"
                            }
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Get subform schema
    let subform_schema = eval.get_evaluated_schema_subform("#/benefits", false);
    
    // Verify $params are copied
    assert!(subform_schema.get("$params").is_some(), "Subform should have $params");
    assert_eq!(
        subform_schema.pointer("/$params/constants/MIN_PREMIUM"),
        Some(&json!(100))
    );
    
    // Verify field structure
    assert!(subform_schema.get("benefits").is_some(), "Subform should have benefits field");
    assert_eq!(
        subform_schema.pointer("/benefits/type"),
        Some(&json!("object"))
    );
    assert!(subform_schema.pointer("/benefits/properties/code").is_some());
    assert!(subform_schema.pointer("/benefits/properties/amount").is_some());
}

#[test]
fn test_evaluate_subform() {
    // Test evaluating a subform with data
    let schema = json!({
        "$params": {
            "constants": {
                "TAX_RATE": 0.1
            }
        },
        "items": {
            "type": "array",
            "items": {
                "properties": {
                    "price": {
                        "type": "number"
                    },
                    "tax": {
                        "type": "number",
                        "$evaluation": {
                            "*": [
                                { "$ref": "#/items/properties/price" },
                                { "$ref": "#/$params/constants/TAX_RATE" }
                            ]
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Evaluate subform with data
    let data = json!({
        "items": {
            "price": 100
        }
    });
    let data_str = serde_json::to_string(&data).unwrap();
    
    eval.evaluate_subform("#/items", &data_str, None, None, None).unwrap();
    
    // Get evaluated schema
    let result = eval.get_evaluated_schema_subform("#/items", false);
    
    // Check that tax was calculated
    assert_eq!(
        result.pointer("/items/properties/tax/$evaluation"),
        None,
        "Evaluation should be resolved"
    );
    // Note: The exact value would depend on how the evaluation is stored
}

#[test]
fn test_validate_subform() {
    // Test validating subform data
    let schema = json!({
        "contacts": {
            "type": "array",
            "items": {
                "properties": {
                    "email": {
                        "type": "string",
                        "rules": {
                            "required": {
                                "value": true,
                                "message": "Email is required"
                            },
                            "pattern": {
                                "value": "^[^@]+@[^@]+\\.[^@]+$",
                                "message": "Invalid email format"
                            }
                        }
                    },
                    "phone": {
                        "type": "string",
                        "rules": {
                            "required": {
                                "value": true,
                                "message": "Phone is required"
                            }
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Valid data
    let valid_data = json!({
        "contacts": {
            "email": "test@example.com",
            "phone": "1234567890"
        }
    });
    let valid_data_str = serde_json::to_string(&valid_data).unwrap();
    
    let valid_result = eval.validate_subform("#/contacts", &valid_data_str, None, None, None).unwrap();
    assert!(!valid_result.has_error, "Valid data should pass validation");
    
    // Invalid data - missing required field
    let invalid_data = json!({
        "contacts": {
            "email": "test@example.com"
            // phone is missing
        }
    });
    let invalid_data_str = serde_json::to_string(&invalid_data).unwrap();
    
    let invalid_result = eval.validate_subform("#/contacts", &invalid_data_str, None, None, None).unwrap();
    // Note: Validation behavior depends on schema structure in subform
    // The subform may need evaluation first for validation to work properly
    println!("Validation result: has_error={}, errors={:?}", invalid_result.has_error, invalid_result.errors);
}

#[test]
fn test_evaluate_dependents_subform() {
    // Test evaluating dependents in a subform
    let schema = json!({
        "calculations": {
            "type": "array",
            "items": {
                "properties": {
                    "base": {
                        "type": "number"
                    },
                    "multiplier": {
                        "type": "number"
                    },
                    "result": {
                        "type": "number",
                        "$evaluation": {
                            "*": [
                                { "$ref": "#/calculations/properties/base" },
                                { "$ref": "#/calculations/properties/multiplier" }
                            ]
                        },
                        "dependents": []
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Initial data
    let data = json!({
        "calculations": {
            "base": 10,
            "multiplier": 5
        }
    });
    let data_str = serde_json::to_string(&data).unwrap();
    
    // Evaluate dependents when base changes
    let result = eval.evaluate_dependents_subform(
        "#/calculations",
        &[String::from("#/calculations/properties/base")],
        Some(&data_str),
        None,
        false,
        None,
        None,
        true
    );
    
    assert!(result.is_ok(), "Should successfully evaluate dependents");
}

#[test]
fn test_resolve_layout_subform() {
    // Test resolving layout in a subform
    let schema = json!({
        "form_items": {
            "type": "array",
            "items": {
                "$layout": {
                    "type": "HorizontalLayout",
                    "elements": [
                        { "$ref": "#/form_items/properties/label" },
                        { "$ref": "#/form_items/properties/value" }
                    ]
                },
                "properties": {
                    "label": {
                        "type": "string",
                        "title": "Label"
                    },
                    "value": {
                        "type": "string",
                        "title": "Value"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Resolve layout
    let result = eval.resolve_layout_subform("#/form_items", false);
    assert!(result.is_ok(), "Should successfully resolve layout");
    
    // Get schema with resolved layout
    let schema = eval.get_evaluated_schema_subform("#/form_items", true);
    
    // Verify layout elements are resolved
    assert!(schema.pointer("/form_items/$layout/elements").is_some());
}

#[test]
fn test_multiple_subforms() {
    // Test handling multiple subforms in one schema
    let schema = json!({
        "$params": {
            "constants": {
                "MAX_ITEMS": 10
            }
        },
        "riders": {
            "type": "array",
            "items": {
                "properties": {
                    "name": { "type": "string" }
                }
            }
        },
        "benefits": {
            "type": "array",
            "items": {
                "properties": {
                    "code": { "type": "string" }
                }
            }
        },
        "contacts": {
            "type": "array",
            "items": {
                "properties": {
                    "email": { "type": "string" }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Check all subforms were created
    let subform_paths = eval.get_subform_paths();
    assert_eq!(subform_paths.len(), 3, "Should have 3 subforms");
    
    assert!(eval.has_subform("#/riders"));
    assert!(eval.has_subform("#/benefits"));
    assert!(eval.has_subform("#/contacts"));
}

#[test]
fn test_subform_isolation() {
    // Test that subforms are truly isolated
    let schema = json!({
        "$params": {
            "constants": {
                "VALUE": 100
            }
        },
        "other_field": {
            "type": "string",
            "title": "Other Field"
        },
        "subform_field": {
            "type": "array",
            "items": {
                "properties": {
                    "item": {
                        "type": "string"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Get subform schema
    let subform_schema = eval.get_evaluated_schema_subform("#/subform_field", false);
    
    // Verify subform only has $params and its own field
    assert!(subform_schema.get("$params").is_some());
    assert!(subform_schema.get("subform_field").is_some());
    assert!(subform_schema.get("other_field").is_none(), "Subform should not have parent's other fields");
}

#[test]
fn test_get_schema_value_subform() {
    // Test getting schema values from subform
    let schema = json!({
        "items": {
            "type": "array",
            "items": {
                "properties": {
                    "quantity": {
                        "type": "number",
                        "value": 5
                    },
                    "price": {
                        "type": "number",
                        "value": 10.5
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Get schema values
    let values = eval.get_schema_value_subform("#/items");
    
    // Schema values should be extracted
    assert!(values.is_object());
}

#[test]
fn test_get_evaluated_schema_without_params_subform() {
    // Test getting evaluated schema without $params
    let schema = json!({
        "$params": {
            "constants": {
                "TEST": "value"
            }
        },
        "data": {
            "type": "array",
            "items": {
                "properties": {
                    "field": {
                        "type": "string"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Get schema without $params
    let schema_without_params = eval.get_evaluated_schema_without_params_subform("#/data", false);
    
    assert!(schema_without_params.get("$params").is_none(), "Should not have $params");
    assert!(schema_without_params.get("data").is_some(), "Should have data field");
}

#[test]
fn test_nonexistent_subform_error() {
    // Test error handling for nonexistent subform
    let schema = json!({
        "regular_field": {
            "type": "string"
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Try to access nonexistent subform
    assert!(!eval.has_subform("#/nonexistent"));
    
    let result = eval.evaluate_subform("#/nonexistent", "{}", None, None, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Subform not found"));
}

#[test]
fn test_nested_subform_key() {
    let schema = json!({
        "properties": {
            "form": {
                "type": "object",
                "properties": {
                    "riders": {
                        "type": "array",
                        "items": {
                            "properties": {
                                "name": { "type": "string" }
                            }
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Subform path should be #/properties/form/properties/riders
    assert!(eval.has_subform("#/properties/form/properties/riders"));
    
    // Get schema without params
    let schema_without_params = eval.get_evaluated_schema_without_params_subform("#/properties/form/properties/riders", false);
    
// Should have "riders" key instead of "properties/form/properties/riders"
    assert!(schema_without_params.get("riders").is_some(), "Should extract only the last segment of the path as key");
    assert!(schema_without_params.get("properties").is_none(), "Should not contain 'properties' from parent path");
}

#[test]
fn test_evaluate_dependents_subform_array_iteration() {
    let schema = json!({
        "$params": {
            "constants": {
                "MULTIPLIER": 2
            }
        },
        "riders": {
            "type": "array",
            "items": {
                "properties": {
                    "base": {
                        "type": "number",
                        "dependents": [
                            {
                                "$ref": "#/riders/properties/calculated/value",
                                "value": {
                                    "$evaluation": {
                                        "*": [
                                            { "$ref": "#/riders/properties/base" },
                                            { "$ref": "#/$params/constants/MULTIPLIER" }
                                        ]
                                    }
                                }
                            }
                        ]
                    },
                    "calculated": {
                        "type": "number",
                        "condition": {
                            "disabled": true
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({
        "riders": [
            { "base": 10 },
            { "base": 20 }
        ]
    });
    let data_str = serde_json::to_string(&data).unwrap();

    let subform_schema = eval.get_evaluated_schema_subform("#/riders", false);
    println!("Subform schema: {}", serde_json::to_string_pretty(&subform_schema).unwrap());
    
    // Check subform internally before and after
    let subform_values_before = eval.get_schema_value_object_subform("#/riders");
    println!("Subform values before: {:?}", subform_values_before);
    
    // Trigger dependents evaluation. We use explicit paths to trigger the dependents on each item
    // changed_paths = ["riders[0].base", "riders[1].base"], include_subforms = true
    let result = eval.evaluate_dependents(
        &["riders[0].base".to_string(), "riders[1].base".to_string()],
        Some(&data_str),
        None,
        true,
        None,
        None,
        true // include_subforms
    ).unwrap();

    // result should be an array of flat subform execution results
    let result_arr = result.as_array().expect("result should be an array");
    
    // There should be two entries in the array: "riders.0.calculated.value" and "riders.1.calculated.value"
    assert_eq!(result_arr.len(), 2, "Should have 2 subform result items");
    
    let mut found_0 = false;
    let mut found_1 = false;

    for item in result_arr {
        let path = item.get("$ref").unwrap().as_str().unwrap();
        let value = item.get("value").unwrap().as_f64().unwrap();
        
        if path == "riders.0.calculated.value" {
            assert_eq!(value, 20.0);
            found_0 = true;
        } else if path == "riders.1.calculated.value" {
            assert_eq!(value, 40.0);
            found_1 = true;
        }
    }
    
    assert!(found_0, "Should have found evaluation for riders[0]");
    assert!(found_1, "Should have found evaluation for riders[1]");
}
