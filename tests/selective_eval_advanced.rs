use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_selective_eval_params_root() {
    // Test selective evaluation of $params fields
    let schema = json!({
        "type": "object",
        "$params": {
            "constants": {
                "RATE": 0.05
            },
            "othersr": {
                "HAS_BEEN_PAID": {
                    "value": {
                        "$evaluation": {
                            "*": [
                                {
                                    "$ref": "$params.constants.RATE"
                                },
                                100
                            ]
                        }
                    }
                }
            }
        }
    });
    
    let data = json!({});
    
    let schema_str = serde_json::to_string(&schema).unwrap();
    let data_str = serde_json::to_string(&data).unwrap();
    
    let mut eval = JSONEval::new(&schema_str, None, Some(&data_str)).unwrap();
    
    // Initial full evaluation
    eval.evaluate(&data_str, None, None, None).unwrap();
    
    let result = eval.get_evaluated_schema(false);
    println!("value_evaluations count: {}", eval.value_evaluations.len());
    println!("value_evaluations: {:?}", eval.value_evaluations);
    println!("Full schema after initial eval: {}", serde_json::to_string_pretty(&result).unwrap());
    let has_been_paid_value = result.pointer("/$params/othersr/HAS_BEEN_PAID/value");
    println!("HAS_BEEN_PAID value pointer result: {:?}", has_been_paid_value);
    assert_eq!(*result.pointer("/$params/othersr/HAS_BEEN_PAID/value").unwrap(), json!(5));
    
    // Update $params and selectively re-evaluate
    let updated_schema = json!({
        "type": "object",
        "$params": {
            "constants": {
                "RATE": 0.10  // Changed rate
            },
            "othersr": {
                "HAS_BEEN_PAID": {
                    "value": {
                        "$evaluation": {
                            "*": [
                                {
                                    "$ref":"$params.constants.RATE"
                                },
                                100
                            ]
                        }
                    }
                }
            }
        }
    });
    
    let updated_schema_str = serde_json::to_string(&updated_schema).unwrap();
    let mut eval2 = JSONEval::new(&updated_schema_str, None, Some(&data_str)).unwrap();
    
    // First do a full evaluation to process the $evaluation objects
   eval2.evaluate(&data_str, None, None, None).unwrap();
    
    let result2 = eval2.get_evaluated_schema(false);
    assert_eq!(*result2.pointer("/$params/othersr/HAS_BEEN_PAID/value").unwrap(), json!(10));
}

#[test]
fn test_selective_eval_explicit_properties() {
    // Test with explicit "properties" in dotted notation
    let schema = json!({
        "type": "object",
        "properties": {
            "illustration": {
                "type": "object",
                "properties": {
                    "user": {
                        "type": "object",
                        "properties": {
                            "firstName": {
                                "value": {
                                    "$evaluation": {
                                        "$ref": "first"
                                    }
                                }
                            },
                            "fullname": {
                                "value": {
                                    "$evaluation": {
                                        "CONCAT": [
                                            {
                                                "$ref": "first"
                                            },
                                            " ",
                                            {
                                                "$ref": "last"
                                            }
                                        ]
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    
    let data = json!({
        "first": "John",
        "last": "Doe"
    });
    
    let schema_str = serde_json::to_string(&schema).unwrap();
    let data_str = serde_json::to_string(&data).unwrap();
    
    let mut eval = JSONEval::new(&schema_str, None, Some(&data_str)).unwrap();
    
    // Initial evaluation
    eval.evaluate(&data_str, None, None, None).unwrap();
    
    let result = eval.get_evaluated_schema(false);
    assert_eq!(*result.pointer("/properties/illustration/properties/user/properties/fullname/value").unwrap(), json!("John Doe"));
    
    // Update data and selectively re-evaluate only fullname
    let updated_data = json!({
        "first": "Jane",
        "last": "Smith"
    });
    let updated_data_str = serde_json::to_string(&updated_data).unwrap();
    
    // Test both path formats
    let paths_dot = vec!["illustration.user.fullname".to_string()];
    eval.evaluate(&updated_data_str, None, Some(&paths_dot), None).unwrap();
    
    let result2 = eval.get_evaluated_schema(false);
    let fullname = result2.pointer("/properties/illustration/properties/user/properties/fullname/value").unwrap();
    assert_eq!(*fullname, json!("Jane Smith"), "fullname should be updated");
    
    // firstName should NOT be updated (kept old value from schema)
    let firstname = result2.pointer("/properties/illustration/properties/user/properties/firstName/value").unwrap();
    assert_eq!(*firstname, json!("John"), "firstName should NOT be updated");
}

#[test]
fn test_selective_eval_explicit_properties_path() {
    // Test with explicit "properties" keywords in the path
    let schema = json!({
        "type": "object",
        "properties": {
            "illustration": {
                "type": "object",
                "properties": {
                    "user": {
                        "type": "object",
                        "properties": {
                            "fullname": {
                                "value": {
                                    "$evaluation": {
                                        "CONCAT": [
                                            {
                                                "$ref": "first"
                                            },
                                            " ",
                                            {
                                                "$ref": "last"
                                            }
                                        ]
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    
    let data = json!({
        "first": "John",
        "last": "Doe"
    });
    
    let schema_str = serde_json::to_string(&schema).unwrap();
    let data_str = serde_json::to_string(&data).unwrap();
    
    let mut eval = JSONEval::new(&schema_str, None, Some(&data_str)).unwrap();
    
    // Initial evaluation
    eval.evaluate(&data_str, None, None, None).unwrap();
    
    // Update data and use explicit properties path (as user specified)
    let updated_data = json!({
        "first": "Jane",
        "last": "Smith"
    });
    let updated_data_str = serde_json::to_string(&updated_data).unwrap();
    
    // User's exact path format
    let paths = vec!["illustration.properties.user.properties.fullname".to_string()];
    eval.evaluate(&updated_data_str, None, Some(&paths), None).unwrap();
    
    let result = eval.get_evaluated_schema(false);
    let fullname = result.pointer("/properties/illustration/properties/user/properties/fullname/value").unwrap();
    assert_eq!(*fullname, json!("Jane Smith"));
}
