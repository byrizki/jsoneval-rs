use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_selective_eval_subforms() {
    // Use Array schema which definitely creates a subform
    let schema = json!({
        "items": {
            "type": "array",
            "items": {
                "properties": {
                    "price": { 
                        "type": "number", 
                        "value": 100 
                    },
                    "unrelated": { "type": "string" },
                    "tax": {
                        "type": "number",
                        "$evaluation": { 
                            "*": [{ "$ref": "#/items/properties/price" }, 0.1]
                        }
                    }
                }
            }
        }
    });

    let data_initial = json!({
        "items": {
            "price": 100,
            "unrelated": "A"
        }
    }).to_string();

    let mut eval = JSONEval::new(&schema.to_string(), None, Some(&data_initial))
        .expect("Failed to create JSONEval");

    // Initial evaluation (full)
    eval.evaluate_subform("#/items", &data_initial, None, None, None).expect("Initial evaluation failed");

    let schema_v1 = eval.get_evaluated_schema_subform("#/items", true);
    println!("Schema V1: {}", schema_v1);

    // Assert tax value directly from schema
    // Based on debug output: {"items":{"properties":{..., "tax":10}}}
    let get_tax_from_schema = |val: &serde_json::Value| -> f64 {
        val.pointer("/items/properties/tax").and_then(|v| v.as_f64()).unwrap_or(0.0)
    };

    let v1 = get_tax_from_schema(&schema_v1);
    assert_eq!(v1, 10.0, "Initial tax should be 10");

    // 2. Update with selective eval on unrelated path
    let data_updated = json!({
        "items": {
            "price": 200,
            "unrelated": "B"
        }
    }).to_string();
    
    let selective_paths = vec!["items.properties.unrelated".to_string()];
    
    eval.evaluate_subform("#/items", &data_updated, None, Some(&selective_paths), None).expect("Selective evaluation failed");
        
    let schema_v2 = eval.get_evaluated_schema_subform("#/items", true);
    println!("Schema V2: {}", schema_v2);
    let v2 = get_tax_from_schema(&schema_v2);
    
    assert_eq!(v2, 10.0, "Tax should NOT update when unrelated field is selected");
    
    // 3. Update with selective eval on price
    // Note: evaluate(paths) evaluates EXACTLY the paths provided. It does not auto-trigger dependents.
    // To update tax, we must include it in the paths.
    let target_paths = vec![
        "items.properties.price".to_string(),
        "items.properties.tax".to_string()
    ]; 
    eval.evaluate_subform("#/items", &data_updated, None, Some(&target_paths), None).expect("Selective evaluation failed 2");
        
    let schema_v3 = eval.get_evaluated_schema_subform("#/items", true);
    println!("Schema V3: {}", schema_v3);
    let v3 = get_tax_from_schema(&schema_v3);
    
    assert_eq!(v3, 20.0, "Tax SHOULD update when price is selected");
}
