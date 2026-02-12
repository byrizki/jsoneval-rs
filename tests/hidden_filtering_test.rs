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
    
    // 1. Check visible field exists and has evaluated value (from schema value here as no logic)
    // Actually evaluate() overwrites data with evaluated values if expressions exist, 
    // or keeps data if no matching evaluations. 
    // Wait, get_schema_value() uses `value_evaluations` which are only fields with "value" property?
    // Let's check the schema again.
    // Yes, all fields in my schema have "value" property. So they should be in value_evaluations.
    // And get_schema_value() overrides data with these values.
    
    assert_eq!(result_value.pointer("/visible_field"), Some(&json!("visible")));
    assert_eq!(result_value.pointer("/nested_visible/child_visible"), Some(&json!("child_visible")));
    
    // 2. Check simple hidden is gone
    assert_eq!(result_value.pointer("/simple_hidden"), None, "Simple hidden field should be removed");
    
    // 3. Check parent hidden child is gone
    assert_eq!(result_value.pointer("/parent_hidden/child_hidden"), None, "Child of hidden parent should be removed");
    // The parent object itself might remain if it has other visible properties, but here it's empty so it might be empty object or removed?
    // prune logic iterates keys. If parent_hidden is in data, we check /properties/parent_hidden.
    // parent_hidden has condition.hidden=true. So it should be removed entirely from data.
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
