use json_eval_rs::RLogic;
use serde_json::json;

#[test]
fn test_subtraction_with_decimal_precision() {
    // Test case: 1 - 0.003 should equal 0.997
    let logic = json!({
        "-": [1, {"*": [1, 0.003]}]
    });
    
    let data = json!({});
    
    let mut engine = RLogic::new();
    let result = engine.evaluate_direct(&logic, &data).unwrap();
    
    println!("Result: {}", serde_json::to_string_pretty(&result).unwrap());
    
    // With arbitrary_precision, this should be exactly 0.997
    let result_str = result.to_string();
    println!("Result value: {}", result_str);
    
    // Parse as f64 for comparison (should be very close to 0.997)
    let result_num = result.as_f64().unwrap();
    
    assert!((result_num - 0.997).abs() < 1e-10, 
        "Expected 0.997, got {}", result_num);
    
    // Also verify the string representation contains 0.997
    assert!(result_str.contains("0.997"), 
        "Expected string to contain '0.997', got: {}", result_str);
}

#[test]
fn test_multiplication_with_decimal_precision() {
    // Test case: 0.1 + 0.2 should equal 0.3 (classic floating point issue)
    let logic = json!({
        "+": [0.1, 0.2]
    });
    
    let data = json!({});
    
    let mut engine = RLogic::new();
    let result = engine.evaluate_direct(&logic, &data).unwrap();
    
    println!("Result: {}", serde_json::to_string_pretty(&result).unwrap());
    
    let result_num = result.as_f64().unwrap();
    
    // With Decimal precision, this should be exactly 0.3
    assert!((result_num - 0.3).abs() < 1e-10, 
        "Expected 0.3, got {}", result_num);
}

#[test]
fn test_division_with_decimal_precision() {
    // Test case: 1 / 3 should maintain precision
    let logic = json!({
        "/": [1, 3]
    });
    
    let data = json!({});
    
    let mut engine = RLogic::new();
    let result = engine.evaluate_direct(&logic, &data).unwrap();
    
    println!("Result: {}", serde_json::to_string_pretty(&result).unwrap());
    
    let result_str = result.to_string();
    
    println!("1/3 = {}", result_str);
    
    // Should have many decimal places
    assert!(result_str.len() > 10, 
        "Expected high precision result, got: {}", result_str);
}

#[test]
fn test_complex_arithmetic_with_decimal_precision() {
    // Test case: (1 - 0.003) * 100 should equal 99.7
    let logic = json!({
        "*": [
            {
                "-": [1, 0.003]
            },
            100
        ]
    });
    
    let data = json!({});
    
    let mut engine = RLogic::new();
    let result = engine.evaluate_direct(&logic, &data).unwrap();
    
    println!("Result: {}", serde_json::to_string_pretty(&result).unwrap());
    
    let result_num = result.as_f64().unwrap();
    
    assert!((result_num - 99.7).abs() < 1e-10, 
        "Expected 99.7, got {}", result_num);
}
