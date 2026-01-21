use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_get_schema_value_params_product_name() {
    // Test getting a value from $params.productName path
    let schema = json!({
        "type": "object",
        "$params": {
            "productName": "Widget Pro",
            "productVersion": "2.0",
            "constants": {
                "MAX_QUANTITY": 100
            }
        },
        "properties": {
            "order": {
                "type": "object",
                "properties": {
                    "item": {
                        "value": {
                            "$evaluation": {
                                "$ref": "$params.productName"
                            }
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
    
    // Evaluate
    eval.evaluate(&data_str, None, None, None).unwrap();
    
    let result = eval.get_evaluated_schema(false);
    
    // Verify that $params.productName was correctly referenced
    let item_value = result.pointer("/properties/order/properties/item/value");
    assert!(item_value.is_some(), "Should find item value");
    assert_eq!(*item_value.unwrap(), json!("Widget Pro"), "Item should reference productName from $params");
    
    // Also verify we can directly access $params
    let product_name = result.pointer("/$params/productName");
    assert!(product_name.is_some(), "Should find $params.productName");
    assert_eq!(*product_name.unwrap(), json!("Widget Pro"));
}

#[test]
fn test_get_schema_value_params_nested() {
    // Test getting nested values from $params paths
    let schema = json!({
        "type": "object",
        "$params": {
            "constants": {
                "RATE": 0.05,
                "TAX_PERCENTAGE": 15
            },
            "config": {
                "settings": {
                    "timeout": 30,
                    "retries": 3
                }
            }
        },
        "properties": {
            "calculation": {
                "type": "object",
                "properties": {
                    "rate": {
                        "value": {
                            "$evaluation": {
                                "$ref": "$params.constants.RATE"
                            }
                        }
                    },
                    "timeout": {
                        "value": {
                            "$evaluation": {
                                "$ref": "$params.config.settings.timeout"
                            }
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
    
    // Evaluate
    eval.evaluate(&data_str, None, None, None).unwrap();
    
    let result = eval.get_evaluated_schema(false);
    
    // Verify nested $params references
    let rate_value = result.pointer("/properties/calculation/properties/rate/value");
    assert!(rate_value.is_some(), "Should find rate value");
    assert_eq!(*rate_value.unwrap(), json!(0.05), "Rate should reference from $params");
    
    let timeout_value = result.pointer("/properties/calculation/properties/timeout/value");
    assert!(timeout_value.is_some(), "Should find timeout value");
    assert_eq!(*timeout_value.unwrap(), json!(30), "Timeout should reference from $params");
    
    // Verify direct access to nested $params
    let rate = result.pointer("/$params/constants/RATE");
    assert_eq!(*rate.unwrap(), json!(0.05));
    
    let timeout = result.pointer("/$params/config/settings/timeout");
    assert_eq!(*timeout.unwrap(), json!(30));
}

#[test]
fn test_selective_eval_params_product_name() {
    // Test selective evaluation with $params paths
    let schema = json!({
        "type": "object",
        "$params": {
            "productName": "Initial Product",
            "version": "1.0"
        },
        "properties": {
            "details": {
                "type": "object",
                "properties": {
                    "product": {
                        "value": {
                            "$evaluation": {
                                "CONCAT": [
                                    {
                                        "$ref": "$params.productName"
                                    },
                                    " v",
                                    {
                                        "$ref": "$params.version"
                                    }
                                ]
                            }
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
    
    // Initial evaluation
    eval.evaluate(&data_str, None, None, None).unwrap();
    
    let result = eval.get_evaluated_schema(false);
    let product_value = result.pointer("/properties/details/properties/product/value").unwrap();
    assert_eq!(*product_value, json!("Initial Product v1.0"));
    
    // Update schema with new $params values
    let updated_schema = json!({
        "type": "object",
        "$params": {
            "productName": "Updated Product",
            "version": "2.0"
        },
        "properties": {
            "details": {
                "type": "object",
                "properties": {
                    "product": {
                        "value": {
                            "$evaluation": {
                                "CONCAT": [
                                    {
                                        "$ref": "$params.productName"
                                    },
                                    " v",
                                    {
                                        "$ref": "$params.version"
                                    }
                                ]
                            }
                        }
                    }
                }
            }
        }
    });
    
    let updated_schema_str = serde_json::to_string(&updated_schema).unwrap();
    let mut eval2 = JSONEval::new(&updated_schema_str, None, Some(&data_str)).unwrap();
    
    // Re-evaluate with updated schema
    eval2.evaluate(&data_str, None, None, None).unwrap();
    
    let result2 = eval2.get_evaluated_schema(false);
    let updated_product_value = result2.pointer("/properties/details/properties/product/value").unwrap();
    assert_eq!(*updated_product_value, json!("Updated Product v2.0"));
}
