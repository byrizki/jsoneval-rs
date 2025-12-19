use json_eval_rs::rlogic::{RLogic, RLogicConfig};
use serde_json::json;

#[test]
fn test_today_no_offset() {
    let rlogic = RLogic::new();
    let result = rlogic.evaluate(&json!({"TODAY": []}), &json!({})).unwrap();
    assert!(result.is_string());
    let date_str = result.as_str().unwrap();
    assert!(date_str.ends_with("T00:00:00.000Z"));
}

#[test]
fn test_now_no_offset() {
    let rlogic = RLogic::new();
    let result = rlogic.evaluate(&json!({"NOW": []}), &json!({})).unwrap();
    assert!(result.is_string());
    let timestamp = result.as_str().unwrap();
    assert!(timestamp.contains("T") && timestamp.contains("Z"));
}

#[test]
fn test_today_with_positive_offset() {
    // UTC+7 = 420 minutes
    let config = RLogicConfig::default().with_timezone_offset(420);
    let rlogic = RLogic::with_config(config);
    
    let result = rlogic.evaluate(&json!({"TODAY": []}), &json!({})).unwrap();
    assert!(result.is_string());
    let date_str = result.as_str().unwrap();
    assert!(date_str.ends_with("T00:00:00.000Z"));
}

#[test]
fn test_today_with_negative_offset() {
    // UTC-5 = -300 minutes
    let config = RLogicConfig::default().with_timezone_offset(-300);
    let rlogic = RLogic::with_config(config);
    
    let result = rlogic.evaluate(&json!({"TODAY": []}), &json!({})).unwrap();
    assert!(result.is_string());
    let date_str = result.as_str().unwrap();
    assert!(date_str.ends_with("T00:00:00.000Z"));
}

#[test]
fn test_now_with_positive_offset() {
    // UTC+7 = 420 minutes
    let config = RLogicConfig::default().with_timezone_offset(420);
    let rlogic = RLogic::with_config(config);
    
    let result = rlogic.evaluate(&json!({"NOW": []}), &json!({})).unwrap();
    assert!(result.is_string());
    let timestamp = result.as_str().unwrap();
    assert!(timestamp.contains("T") && timestamp.contains("Z"));
}

#[test]
fn test_now_with_negative_offset() {
    // UTC-5 = -300 minutes
    let config = RLogicConfig::default().with_timezone_offset(-300);
    let rlogic = RLogic::with_config(config);
    
    let result = rlogic.evaluate(&json!({"NOW": []}), &json!({})).unwrap();
    assert!(result.is_string());
    let timestamp = result.as_str().unwrap();
    assert!(timestamp.contains("T") && timestamp.contains("Z"));
}

#[test]
fn test_offset_with_date_operations() {
    // Test that offset works with YEAR/MONTH/DAY operations
    let config = RLogicConfig::default().with_timezone_offset(420);
    let rlogic = RLogic::with_config(config);
    
    // Get TODAY with offset
    let today = rlogic.evaluate(&json!({"TODAY": []}), &json!({})).unwrap();
    
    // Extract YEAR from TODAY
    let year_result = rlogic.evaluate(
        &json!({"YEAR": [{"var": "date"}]}),
        &json!({"date": today})
    ).unwrap();
    
    // Should return a valid year number (between 1900 and 2100)
    assert!(year_result.is_number());
    let year = year_result.as_f64().unwrap();
    assert!(year >= 1900.0 && year <= 2100.0);
}

#[test]
fn test_extreme_positive_offset() {
    // UTC+14 = 840 minutes (Line Islands)
    let config = RLogicConfig::default().with_timezone_offset(840);
    let rlogic = RLogic::with_config(config);
    
    let result = rlogic.evaluate(&json!({"TODAY": []}), &json!({})).unwrap();
    assert!(result.is_string());
}

#[test]
fn test_extreme_negative_offset() {
    // UTC-12 = -720 minutes (Baker Island)
    let config = RLogicConfig::default().with_timezone_offset(-720);
    let rlogic = RLogic::with_config(config);
    
    let result = rlogic.evaluate(&json!({"TODAY": []}), &json!({})).unwrap();
    assert!(result.is_string());
}
