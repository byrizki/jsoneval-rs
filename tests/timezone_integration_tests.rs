use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_jsoneval_with_timezone_offset() {
    let schema = json!({
        "type": "object",
        "properties": {
            "current_date": {
                "type": "string",
                "$evaluation": {"TODAY": []}
            },
            "current_time": {
                "type": "string",
                "$evaluation": {"NOW": []}
            }
        }
    }).to_string();

    // Create JSONEval and set timezone offset
    let mut eval = JSONEval::new(&schema, None, None)
        .expect("Failed to create JSONEval");
    
    eval.set_timezone_offset(Some(420)); // UTC+7

    // Evaluate
    eval.evaluate(&json!({}).to_string(), None, None, None)
        .expect("Evaluation failed");

    // Get results
    let result = eval.get_evaluated_schema(false);
    
    // Verify that current_date and current_time are strings
    assert!(result["properties"]["current_date"].is_string(), "current_date should be a string");
    assert!(result["properties"]["current_time"].is_string(), "current_time should be a string");
    
    let date_str = result["properties"]["current_date"].as_str().unwrap();
    let time_str = result["properties"]["current_time"].as_str().unwrap();
    
    // Verify format
    assert!(date_str.ends_with("T00:00:00.000Z"), "Date should end with T00:00:00.000Z");
    assert!(time_str.contains("T") && time_str.contains("Z"), "Time should be in RFC3339 format");
}

#[test]
fn test_jsoneval_default_utc() {
    let schema = json!({
        "type": "object",
        "properties": {
            "current_date": {
                "type": "string",
                "$evaluation": {"TODAY": []}
            }
        }
    }).to_string();

    // Create JSONEval without timezone offset (defaults to UTC)
    let mut eval = JSONEval::new(&schema, None, None)
        .expect("Failed to create JSONEval");

    // Evaluate
    eval.evaluate(&json!({}).to_string(), None, None, None)
        .expect("Evaluation failed");

    // Get results
    let result = eval.get_evaluated_schema(false);
    
    // Verify that current_date is a string
    assert!(result["properties"]["current_date"].is_string());
}

#[test]
fn test_set_timezone_offset() {
    let schema = json!({
        "type": "object",
        "properties": {
            "current_date": {
                "type": "string",
                "$evaluation": {"TODAY": []}
            },
            "current_time": {
                "type": "string",
                "$evaluation": {"NOW": []}
            }
        }
    }).to_string();

    // Create JSONEval with default UTC
    let mut eval = JSONEval::new(&schema, None, None)
        .expect("Failed to create JSONEval");

    // Evaluate with UTC
    eval.evaluate(&json!({}).to_string(), None, None, None)
        .expect("Evaluation failed");
    
    let result_utc = eval.get_evaluated_schema(false);
    let date_utc = result_utc["properties"]["current_date"].as_str().unwrap();
    let time_utc = result_utc["properties"]["current_time"].as_str().unwrap();
    
    // Verify format
    assert!(date_utc.ends_with("T00:00:00.000Z"));
    assert!(time_utc.contains("T") && time_utc.contains("Z"));
    
    // Change to UTC+7
    eval.set_timezone_offset(Some(420));
    
    // Evaluate again with new timezone
    eval.evaluate(&json!({}).to_string(), None, None, None)
        .expect("Evaluation failed");
    
    let result_utc7 = eval.get_evaluated_schema(false);
    let date_utc7 = result_utc7["properties"]["current_date"].as_str().unwrap();
    let time_utc7 = result_utc7["properties"]["current_time"].as_str().unwrap();
    
    // Verify new results are still valid
    assert!(date_utc7.ends_with("T00:00:00.000Z"));
    assert!(time_utc7.contains("T") && time_utc7.contains("Z"));
    
    // The dates might be different due to timezone offset
    // (depending on when the test runs relative to day boundaries)
    
    // Reset to UTC
    eval.set_timezone_offset(None);
    
    // Evaluate one more time
    eval.evaluate(&json!({}).to_string(), None, None, None)
        .expect("Evaluation failed");
    
    let result_reset = eval.get_evaluated_schema(false);
    let date_reset = result_reset["properties"]["current_date"].as_str().unwrap();
    
    // Should match original UTC date (assuming test runs quickly)
    assert_eq!(date_reset, date_utc, "Reset to UTC should match original UTC date");
}
