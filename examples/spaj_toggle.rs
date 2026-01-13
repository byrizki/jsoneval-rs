use std::fs;
use std::path::Path;
use json_eval_rs::JSONEval;
use serde_json::json;

fn main() {
    println!("\n🚀 JSON Evaluation - SPAJ Toggle Example\n");

    let schema_path = Path::new("samples/spaj.json");
    let schema_str = fs::read_to_string(schema_path).expect("Failed to read schema");

    // Initial data with minimal context required
    let context_str = json!({
        "agentProfile": { "sob": "AG" } 
    }).to_string();

    let initial_data = json!({
        "illustration": {
            "basicinformation": {
                "print_polflag": false
            }
        }
    }).to_string();

    // Initialize logic
    let mut eval = JSONEval::new(&schema_str, Some(&context_str), Some(&initial_data))
        .expect("Failed to create JSONEval");

    // Helper to check visibility
    let check_visibility = |eval: &mut JSONEval, expected_hidden: bool, step: &str| {
        let result = eval.get_evaluated_schema(false);
        let hidden = result.pointer("/illustration/properties/basicinformation/properties/print_poladdress/condition/hidden")
            .and_then(|v| v.as_bool());
        
        match hidden {
            Some(val) => {
                if val == expected_hidden {
                    println!("✅ {}: Hidden = {} (Expected: {})", step, val, expected_hidden);
                } else {
                    println!("❌ {}: Hidden = {} (Expected: {})", step, val, expected_hidden);
                }
            },
            None => println!("❌ {}: 'hidden' property not found", step),
        }
    };

    // Step 1: Initial state (false)
    println!("Step 1: Initial State (print_polflag: false)");
    eval.evaluate(&initial_data, Some(&context_str), None, None).expect("Evaluation failed");
    check_visibility(&mut eval, true, "Initial check");

    // Step 2: Toggle to true
    println!("\nStep 2: Toggle True (print_polflag: true)");
    let data_true = json!({
        "illustration": {
            "basicinformation": {
                "print_polflag": true
            }
        }
    }).to_string();
    eval.evaluate(&data_true, Some(&context_str), None, None).expect("Evaluation failed");
    check_visibility(&mut eval, false, "Toggle ON check");

    // Step 3: Toggle back to false
    println!("\nStep 3: Toggle False (print_polflag: false)");
    let data_false = json!({
        "illustration": {
            "basicinformation": {
                "print_polflag": false
            }
        }
    }).to_string();
    eval.evaluate(&data_false, Some(&context_str), None, None).expect("Evaluation failed");
    
    let hidden_path = "#/illustration/properties/basicinformation/properties/print_poladdress/condition/hidden";
    if let Some(deps) = eval.dependencies.get(hidden_path) {
        println!("Debug: Dependencies for hidden: {:?}", deps);
    } else {
        println!("Debug: No dependencies found for hidden path");
    }

    // Debug: Print current flag value
    if let Some(val) = eval.get_evaluated_schema(false).pointer("/illustration/properties/basicinformation/properties/print_polflag/value") {
         println!("Debug: print_polflag value is: {}", val);
    }

    check_visibility(&mut eval, true, "Toggle OFF check");
}
