use json_eval_rs::JSONEval;
use serde_json::json;

fn main() {
    let schema = json!({
        "type": "object",
        "properties": {
            "price": {
                "type": "number"
            },
            "tax": {
                "type": "number",
                "value": {
                    "$evaluation": {
                        "*": [
                            { "$ref": "#/properties/price" },
                            0.1
                        ]
                    }
                }
            },
            "total": {
                "type": "number",
                "value": {
                    "$evaluation": {
                        "+": [
                            { "$ref": "#/properties/price" },
                            { "$ref": "#/properties/tax" }
                        ]
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    
    println!("=== Example 1: With Caching (Default) ===");
    {
        let data = json!({ "price": 100 });
        let data_str = serde_json::to_string(&data).unwrap();
        
        let mut eval = JSONEval::new(&schema_str, None, Some(&data_str)).unwrap();
        
        println!("Cache enabled: {}", eval.is_cache_enabled());
        println!("Initial cache size: {}", eval.cache_len());
        
        eval.evaluate(&data_str, None, None).unwrap();
        
        println!("After evaluation cache size: {}", eval.cache_len());
        let stats = eval.cache_stats();
        println!("Cache stats: {}", stats);
    }
    
    println!("\n=== Example 2: Without Caching (Web API Mode) ===");
    {
        let data = json!({ "price": 200 });
        let data_str = serde_json::to_string(&data).unwrap();
        
        let mut eval = JSONEval::new(&schema_str, None, Some(&data_str)).unwrap();
        
        // Disable caching for single-use web API scenario
        eval.disable_cache();
        
        println!("Cache enabled: {}", eval.is_cache_enabled());
        println!("Initial cache size: {}", eval.cache_len());
        
        eval.evaluate(&data_str, None, None).unwrap();
        
        println!("After evaluation cache size: {}", eval.cache_len());
        let stats = eval.cache_stats();
        println!("Cache stats: {}", stats);
        
        println!("\n✅ No cache overhead - perfect for web APIs!");
    }
    
    println!("\n=== Example 3: Re-enabling Cache ===");
    {
        let data = json!({ "price": 300 });
        let data_str = serde_json::to_string(&data).unwrap();
        
        let mut eval = JSONEval::new(&schema_str, None, Some(&data_str)).unwrap();
        
        // Disable then re-enable
        eval.disable_cache();
        eval.enable_cache();
        
        println!("Cache enabled: {}", eval.is_cache_enabled());
        eval.evaluate(&data_str, None, None).unwrap();
        
        println!("Cache size after evaluation: {}", eval.cache_len());
        println!("\n✅ Cache can be toggled as needed!");
    }
}
