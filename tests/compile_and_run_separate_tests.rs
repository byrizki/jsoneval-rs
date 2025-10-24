use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_separate_compile_and_run_zero_clone() {
    let schema = json!({
        "type": "object",
        "properties": {
            "x": {"type": "number"}
        }
    }).to_string();
    
    let data = json!({"x": 10}).to_string();
    // Compile once - stored in global cache
    let logic = json!({"*": [{"var": "x"}, 2]}).to_string();
    let eval1 = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");
    let compiled_id = eval1.compile_logic(&logic)
        .expect("Failed to compile logic");

    // Run multiple times with different data (zero-clone pattern)
    // Can even use different JSONEval instances - compiled logic is global
    let data1 = json!({"x": 10});
    let mut eval2 = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");
    let result1 = eval2.run_logic(compiled_id, Some(&data1), None)
        .expect("Failed to run logic with data1");
    assert_eq!(result1, json!(20));

    let data2 = json!({"x": 25});
    let mut eval3 = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");
    let result2 = eval3.run_logic(compiled_id, Some(&data2), None)
        .expect("Failed to run logic with data2");
    assert_eq!(result2, json!(50));

    let data3 = json!({"x": 100});
    let mut eval4 = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");
    let result3 = eval4.run_logic(compiled_id, Some(&data3), None)
        .expect("Failed to run logic with data3");
    assert_eq!(result3, json!(200));
}

#[test]
fn test_separate_compile_and_run_with_context() {
    let schema = json!({
        "type": "object",
        "$params": {
            "multiplier": 3
        },
        "properties": {
            "value": {"type": "number"}
        }
    }).to_string();
    
    let data = json!({"value": 5}).to_string();
    let context = json!({"multiplier": 3}).to_string();
    let mut eval = JSONEval::new(&schema, Some(&context), Some(&data))
        .expect("Failed to create JSONEval");

    // Compile once - logic uses both data and context
    let logic = json!({
        "*": [
            {"var": "value"},
            {"$ref": "$context.multiplier"}
        ]
    }).to_string();
    
    let compiled_id = eval.compile_logic(&logic)
        .expect("Failed to compile logic");

    // Run with different data values
    let data1 = json!({"value": 5});
    let result1 = eval.run_logic(compiled_id, Some(&data1), None)
        .expect("Failed to run logic");
    assert_eq!(result1, json!(15));

    let data2 = json!({"value": 10});
    let result2 = eval.run_logic(compiled_id, Some(&data2), None)
        .expect("Failed to run logic");
    assert_eq!(result2, json!(30));

    // Run with different context
    let context2 = json!({"multiplier": 5});
    let result3 = eval.run_logic(compiled_id, Some(&data1), Some(&context2))
        .expect("Failed to run logic with different context");
    assert_eq!(result3, json!(25));
}

#[test]
fn test_separate_compile_and_run_no_data() {
    let schema = json!({
        "type": "object",
        "properties": {
            "status": {"type": "string"}
        }
    }).to_string();
    
    let data = json!({"status": "active"}).to_string();
    let mut eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    // Compile logic
    let logic = json!({"==": [{"var": "status"}, "active"]}).to_string();
    let compiled_id = eval.compile_logic(&logic)
        .expect("Failed to compile logic");

    // Run without providing data (uses existing eval_data)
    let result = eval.run_logic(compiled_id, None, None)
        .expect("Failed to run logic");
    assert_eq!(result, json!(true));
}

#[test]
fn test_separate_compile_complex_logic() {
    let schema = json!({
        "type": "object",
        "properties": {
            "a": {"type": "number"},
            "b": {"type": "number"},
            "c": {"type": "number"}
        }
    }).to_string();
    
    let data = json!({"a": 10, "b": 20, "c": 30}).to_string();
    let mut eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    // Compile complex nested logic
    let logic = json!({
        "if": [
            {">": [{"var": "a"}, 5]},
            {"+": [{"var": "b"}, {"var": "c"}]},
            {"*": [{"var": "a"}, 2]}
        ]
    }).to_string();
    
    let compiled_id = eval.compile_logic(&logic)
        .expect("Failed to compile complex logic");

    // Test with condition true
    let data1 = json!({"a": 10, "b": 20, "c": 30});
    let result1 = eval.run_logic(compiled_id, Some(&data1), None)
        .expect("Failed to run logic");
    assert_eq!(result1, json!(50)); // b + c = 50

    // Test with condition false
    let data2 = json!({"a": 3, "b": 20, "c": 30});
    let result2 = eval.run_logic(compiled_id, Some(&data2), None)
        .expect("Failed to run logic");
    assert_eq!(result2, json!(6)); // a * 2 = 6
}
