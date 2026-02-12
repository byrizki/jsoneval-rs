use json_eval_rs::jsoneval::JSONEval;
use serde_json::json;


#[test]
fn test_hidden_field_filtering() {
    let schema = json!({
        "type": "object",
        "properties": {
            "visible_field": {
                "type": "string",
                "value": "visible"
            },
            "simple_hidden": {
                "type": "string",
                "condition": {
                    "hidden": true
                },
                "value": "hidden"
            },
            "parent_hidden": {
                "type": "object",
                "condition": {
                    "hidden": true
                },
                "properties": {
                    "child_hidden": {
                        "type": "string",
                        "value": "child_hidden"
                    }
                }
            },
            "layout_hidden": {
                "type": "object",
                "$layout": {
                    "hideLayout": {
                        "all": true
                    }
                },
                "properties": {
                    "child_layout_hidden": {
                        "type": "string",
                        "value": "child_layout_hidden"
                    }
                }
            },
            "nested_visible": {
                "type": "object",
                "properties": {
                    "child_visible": {
                        "type": "string",
                        "value": "child_visible"
                    }
                }
            }
        }
    });

    let data = json!({
        "visible_field": "user_visible",
        "simple_hidden": "user_hidden", // Should be pruned
        "parent_hidden": {
            "child_hidden": "user_child_hidden" // Should be pruned
        },
        "layout_hidden": {
            "child_layout_hidden": "user_child_layout_hidden" // Should be pruned
        },
        "nested_visible": {
            "child_visible": "user_child_visible"
        }
    });

    // Initialize JSONEval
    let schema_str = schema.to_string();
    let data_str = data.to_string();
    // Use JSONEval::new which takes &str pointers
    let mut eval = JSONEval::new(&schema_str, Some("{}"), Some(&data_str))
        .expect("Failed to create JSONEval");
    
    // Evaluate to populate evaluations and evaluated_schema
    eval.evaluate(&data_str, None, None, None)
        .expect("Evaluation failed");

    // Test get_schema_value (Data View)
    let result_value = eval.get_schema_value();
    
    // 1. Check visible fields exist and have values
    assert_eq!(result_value.pointer("/visible_field"), Some(&json!("visible")));
    assert_eq!(result_value.pointer("/nested_visible/child_visible"), Some(&json!("child_visible")));
    
    // 2. Check simple hidden is gone
    assert_eq!(result_value.pointer("/simple_hidden"), None, "Simple hidden field should be removed");
    
    // 3. Check parent hidden child is gone
    // 3. Check parent hidden child is gone
    assert_eq!(result_value.pointer("/parent_hidden/child_hidden"), None, "Child of hidden parent should be removed");
    assert_eq!(result_value.pointer("/parent_hidden"), None, "Hidden parent object should be removed");

    // 4. Check layout hidden child is gone
    assert_eq!(result_value.pointer("/layout_hidden/child_layout_hidden"), None, "Child of layout hidden parent should be removed");
    assert_eq!(result_value.pointer("/layout_hidden"), None, "Layout hidden parent object should be removed");
    
    
    // Test get_schema_value_object (Flat View)
    let result_obj = eval.get_schema_value_object();
    let obj_map = result_obj.as_object().unwrap();
    
    assert!(obj_map.contains_key("visible_field"));
    assert!(obj_map.contains_key("nested_visible.child_visible"));
    assert!(!obj_map.contains_key("simple_hidden"));
    assert!(!obj_map.contains_key("parent_hidden.child_hidden"));
    assert!(!obj_map.contains_key("layout_hidden.child_layout_hidden"));
    
    // Test get_schema_value_array (Array View)
    let result_arr = eval.get_schema_value_array();
    let arr = result_arr.as_array().unwrap();
    
    let paths: Vec<&str> = arr.iter()
        .map(|item| item["path"].as_str().unwrap())
        .collect();
        
    assert!(paths.contains(&"visible_field"));
    assert!(paths.contains(&"nested_visible.child_visible"));
    assert!(!paths.contains(&"simple_hidden"));
    assert!(!paths.contains(&"parent_hidden.child_hidden"));
    assert!(!paths.contains(&"layout_hidden.child_layout_hidden"));
}

#[test]
fn test_hidden_field_validation() {
    let schema = json!({
        "type": "object",
        "properties": {
            "visible_required": {
                "type": "string",
                "rules": {
                    "required": true
                }
            },
            "hidden_required": {
                "type": "string",
                "condition": {
                    "hidden": true
                },
                "rules": {
                    "required": true
                }
            },
            "parent_hidden_required": {
                "type": "object",
                "condition": {
                    "hidden": true
                },
                "properties": {
                    "child_required": {
                        "type": "string",
                        "rules": {
                            "required": true
                        }
                    }
                }
            },
            "layout_hidden_required": {
                "type": "object",
                "$layout": {
                    "hideLayout": {
                        "all": true
                    }
                },
                "properties": {
                    "child_layout_required": {
                        "type": "string",
                        "rules": {
                            "required": true
                        }
                    }
                }
            }
        }
    });

    // Data missing all required fields
    let data = json!({});

    // Initialize JSONEval
    let schema_str = schema.to_string();
    let data_str = data.to_string();
    let mut eval = JSONEval::new(&schema_str, Some("{}"), Some(&data_str))
        .expect("Failed to create JSONEval");
    
    // Evaluate first to ensure schema is processed
    eval.evaluate(&data_str, None, None, None)
        .expect("Evaluation failed");

    // Validate
    let result = eval.validate(&data_str, None, None, None)
        .expect("Validation failed");

    // Check errors
    // Should have error for visible_required
    assert!(result.errors.contains_key("visible_required"), "Should have error for visible_required");
    
    // Should NOT have error for hidden_required
    assert!(!result.errors.contains_key("hidden_required"), "Should NOT have error for hidden_required");
    
    // Should NOT have error for parent_hidden_required.child_required
    assert!(!result.errors.contains_key("parent_hidden_required.child_required"), "Should NOT have error for hidden parent child");
    
    // Should NOT have error for layout_hidden_required.child_layout_required
    assert!(!result.errors.contains_key("layout_hidden_required.child_layout_required"), "Should NOT have error for hidden layout parent child");
}

#[test]
fn test_layout_structure_hiding() {
    // Scenario: "container" is hidden. "field" is a sibling of "container" in schema,
    // but "field" is placed inside "container" in the layout.
    // Therefore "field" should be effectively hidden.

    // Note: JSONEval uses layout_paths to find root layouts. 
    // For this test, we construct a schema that includes a root layout referencing the container.
    
    // Better schema structure that JSONEval parses correctly as layout:
    let schema_with_root_layout = json!({
        "type": "object",
        "properties": {
             "section": {
                 "type": "object",
                 "condition": { "hidden": true },
                 "$layout": {
                     "elements": [
                         { "$ref": "#/properties/target_field" }
                     ]
                 }
             },
             "target_field": {
                 "type": "string", 
                 "rules": { "required": true }
             }
        },
        "$layout": {
            "elements": [
                { "$ref": "#/properties/section" }
            ]
        }
    });

    let data = json!({});
    let schema_str = schema_with_root_layout.to_string();
    let data_str = data.to_string();

    let mut eval = JSONEval::new(&schema_str, Some("{}"), Some(&data_str))
        .expect("Failed to create JSONEval");
    
    // Ensure layout paths are found and processed.
    let result = eval.validate(&data_str, None, None, None)
        .expect("Validation failed");

    // "target_field" should be hidden because it is inside "section" (hidden).
    assert!(!result.errors.contains_key("target_field"), "Should NOT have error for target_field hidden via layout nesting");
}
