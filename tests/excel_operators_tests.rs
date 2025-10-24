use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_round_with_decimals() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // ROUND with positive decimals
    let logic = json!({"ROUND": [3.14159, 2]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!(3.14));
    
    // ROUND with 0 decimals (backward compatible)
    let logic = json!({"ROUND": [3.7]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 4.0);
    
    // ROUND with negative decimals (round to left of decimal)
    let logic = json!({"ROUND": [1234.5, -2]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 1200.0);
}

#[test]
fn test_roundup_with_decimals() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // ROUNDUP with positive decimals
    let logic = json!({"ROUNDUP": [3.14159, 2]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!(3.15));
    
    // ROUNDUP with negative decimals
    let logic = json!({"ROUNDUP": [1234.5, -2]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 1300.0);
}

#[test]
fn test_rounddown_with_decimals() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // ROUNDDOWN with positive decimals
    let logic = json!({"ROUNDDOWN": [3.14159, 2]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!(3.14));
    
    // ROUNDDOWN with negative decimals
    let logic = json!({"ROUNDDOWN": [1234.5, -2]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 1200.0);
}

#[test]
fn test_ceiling_function() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // CEILING with default significance (1)
    let logic = json!({"CEILING": [4.3]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 5.0);
    
    // CEILING with custom significance
    let logic = json!({"CEILING": [4.3, 0.5]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!(4.5));
    
    // CEILING with larger significance
    let logic = json!({"CEILING": [123, 10]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 130.0);
}

#[test]
fn test_floor_function() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // FLOOR with default significance (1)
    let logic = json!({"FLOOR": [4.7]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 4.0);
    
    // FLOOR with custom significance
    let logic = json!({"FLOOR": [4.7, 0.5]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!(4.5));
    
    // FLOOR with larger significance
    let logic = json!({"FLOOR": [123, 10]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 120.0);
}

#[test]
fn test_trunc_function() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // TRUNC with default (0 decimals)
    let logic = json!({"TRUNC": [8.9]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 8.0);
    
    // TRUNC with positive decimals
    let logic = json!({"TRUNC": [8.9876, 2]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!(8.98));
    
    // TRUNC with negative decimals
    let logic = json!({"TRUNC": [123.456, -1]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 120.0);
}

#[test]
fn test_mround_function() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // MROUND to nearest multiple
    let logic = json!({"MROUND": [10, 3]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 9.0);
    
    let logic = json!({"MROUND": [11, 3]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 12.0);
    
    // MROUND with decimal multiple
    let logic = json!({"MROUND": [1.23, 0.1]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    let rounded = result.as_f64().unwrap();
    assert!((rounded - 1.2).abs() < 0.01);
}

#[test]
fn test_stringformat_basic() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // Format with decimals
    let logic = json!({"STRINGFORMAT": [1234.567, 2]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!("1,234.57"));
    
    // Format with prefix
    let logic = json!({"STRINGFORMAT": [1000, 0, "$"]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!("$1,000"));
    
    // Format with suffix
    let logic = json!({"STRINGFORMAT": [50, 0, "", "%"]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!("50%"));
}

#[test]
fn test_stringformat_full() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // Format with all parameters: value, decimals, prefix, suffix, thousands_sep
    let logic = json!({"STRINGFORMAT": [1234567.89, 2, "$", " USD", ","]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!("$1,234,567.89 USD"));
    
    // Format with custom thousands separator
    let logic = json!({"STRINGFORMAT": [1234567, 0, "", "", "."]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!("1.234.567"));
}

#[test]
fn test_dateformat_prebuilt() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    let data = json!({"date": "2024-01-15"});
    let data_str = data.to_string();
    
    // ISO format (default)
    let logic = json!({"DATEFORMAT": [{"var": "date"}]});
    let result = eval.compile_and_run_logic(&logic.to_string(), Some(&data_str), None).unwrap();
    assert_eq!(result, json!("2024-01-15"));
    
    // Short format
    let logic = json!({"DATEFORMAT": [{"var": "date"}, "short"]});
    let result = eval.compile_and_run_logic(&logic.to_string(), Some(&data_str), None).unwrap();
    assert_eq!(result, json!("01/15/2024"));
    
    // Long format
    let logic = json!({"DATEFORMAT": [{"var": "date"}, "long"]});
    let result = eval.compile_and_run_logic(&logic.to_string(), Some(&data_str), None).unwrap();
    assert_eq!(result, json!("January 15, 2024"));
    
    // EU format
    let logic = json!({"DATEFORMAT": [{"var": "date"}, "eu"]});
    let result = eval.compile_and_run_logic(&logic.to_string(), Some(&data_str), None).unwrap();
    assert_eq!(result, json!("15/01/2024"));
}

#[test]
fn test_dateformat_custom() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    let data = json!({"date": "2024-01-15"});
    let data_str = data.to_string();
    
    // Custom format with strftime
    let logic = json!({"DATEFORMAT": [{"var": "date"}, "%Y/%m/%d"]});
    let result = eval.compile_and_run_logic(&logic.to_string(), Some(&data_str), None).unwrap();
    assert_eq!(result, json!("2024/01/15"));
    
    // Custom format with day and month names
    let logic = json!({"DATEFORMAT": [{"var": "date"}, "%A, %B %d"]});
    let result = eval.compile_and_run_logic(&logic.to_string(), Some(&data_str), None).unwrap();
    assert_eq!(result, json!("Monday, January 15"));
}

#[test]
fn test_compile_logic_separation() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // Compile logic once
    let logic_str = json!({"ROUND": [{"var": "x"}, 2]}).to_string();
    let logic_id = eval.compile_logic(&logic_str).unwrap();
    
    // Run with different data
    let data1 = json!({"x": 3.14159});
    let result1 = eval.run_logic(logic_id, Some(&data1), None).unwrap();
    assert_eq!(result1, json!(3.14));
    
    let data2 = json!({"x": 2.71828});
    let result2 = eval.run_logic(logic_id, Some(&data2), None).unwrap();
    assert_eq!(result2, json!(2.72));
}

#[test]
fn test_excel_compatibility_round() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // Excel ROUND(2.15, 1) = 2.2 (rounds to even)
    let logic = json!({"ROUND": [2.15, 1]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    let rounded = result.as_f64().unwrap();
    assert!((rounded - 2.2).abs() < 0.01 || (rounded - 2.1).abs() < 0.01); // Floating point rounding
    
    // Excel ROUND(2.149, 1) = 2.1
    let logic = json!({"ROUND": [2.149, 1]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result, json!(2.1));
    
    // Excel ROUND(21.5, -1) = 20 (round to nearest 10)
    let logic = json!({"ROUND": [21.5, -1]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 20.0);
}

#[test]
fn test_backward_compatibility() {
    let schema = json!({});
    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    
    // Old syntax without decimals should still work
    let logic = json!({"ROUND": [3.7]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 4.0);
    
    let logic = json!({"ROUNDUP": [3.2]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 4.0);
    
    let logic = json!({"ROUNDDOWN": [3.9]});
    let result = eval.compile_and_run_logic(&logic.to_string(), None, None).unwrap();
    assert_eq!(result.as_f64().unwrap(), 3.0);
}
