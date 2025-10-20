use json_eval_rs::RLogic;
use serde_json::json;

#[test]
fn test_sum_without_threshold() {
    let mut engine = RLogic::new();
    let logic = json!({"SUM": [[1, 2, 3, 4, 5]]});
    let compiled = engine.compile(&logic).unwrap();
    let result = engine.run(&compiled, &json!({})).unwrap();
    assert_eq!(result, json!(15.0)); // Float result
}

#[test]
fn test_sum_with_threshold() {
    let mut engine = RLogic::new();
    // threshold = 2 means sum indices 0, 1, 2 (first 3 elements)
    let logic = json!({"SUM": [[1, 2, 3, 4, 5], null, 2]});
    let compiled = engine.compile(&logic).unwrap();
    let result = engine.run(&compiled, &json!({})).unwrap();
    assert_eq!(result, json!(6.0)); // 1 + 2 + 3 = 6
}

#[test]
fn test_sum_with_field_and_threshold() {
    let mut engine = RLogic::new();
    // Pass the table as data, not inline in the logic
    let data = json!({
        "table": [
            {"value": 10},
            {"value": 20},
            {"value": 30},
            {"value": 40},
            {"value": 50}
        ]
    });
    let logic = json!({"SUM": [{"var": "table"}, "value", 2]});
    let compiled = engine.compile(&logic).unwrap();
    let result = engine.run(&compiled, &data).unwrap();
    assert_eq!(result, json!(60.0)); // 10 + 20 + 30 = 60
}

#[test]
fn test_sum_with_threshold_zero() {
    let mut engine = RLogic::new();
    // threshold = 0 means sum only index 0 (first element)
    let logic = json!({"SUM": [[1, 2, 3, 4, 5], null, 0]});
    let compiled = engine.compile(&logic).unwrap();
    let result = engine.run(&compiled, &json!({})).unwrap();
    assert_eq!(result, json!(1.0));
}

#[test]
fn test_sum_with_threshold_larger_than_array() {
    let mut engine = RLogic::new();
    let logic = json!({"SUM": [[1, 2, 3], null, 10]});
    let compiled = engine.compile(&logic).unwrap();
    let result = engine.run(&compiled, &json!({})).unwrap();
    assert_eq!(result, json!(6.0)); // Sums all elements since threshold > array length
}

#[test]
fn test_sum_with_negative_threshold() {
    let mut engine = RLogic::new();
    // negative threshold means no limit (sum all)
    let logic = json!({"SUM": [[1, 2, 3, 4, 5], null, -1]});
    let compiled = engine.compile(&logic).unwrap();
    let result = engine.run(&compiled, &json!({})).unwrap();
    assert_eq!(result, json!(15.0));
}
