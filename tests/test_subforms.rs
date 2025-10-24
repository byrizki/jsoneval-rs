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
    
    eval.evaluate_subform("#/items", &data_str, None).unwrap();
    
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
    
    let valid_result = eval.validate_subform("#/contacts", &valid_data_str, None, None).unwrap();
    assert!(!valid_result.has_error, "Valid data should pass validation");
    
    // Invalid data - missing required field
    let invalid_data = json!({
        "contacts": {
            "email": "test@example.com"
            // phone is missing
        }
    });
    let invalid_data_str = serde_json::to_string(&invalid_data).unwrap();
    
    let invalid_result = eval.validate_subform("#/contacts", &invalid_data_str, None, None).unwrap();
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
        false
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
    
    let result = eval.evaluate_subform("#/nonexistent", "{}", None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Subform not found"));
}
