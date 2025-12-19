use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_validation_error_has_all_fields() {
    // Schema with pattern rule to test all error fields
    let schema = json!({
        "type": "object",
        "properties": {
            "email": {
                "type": "string",
                "title": "Email",
                "rules": {
                    "pattern": {
                        "value": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$",
                        "message": "Invalid email format",
                        "code": "email.invalid_format"
                    }
                }
            },
            "age": {
                "type": "number",
                "title": "Age",
                "rules": {
                    "minValue": {
                        "value": 1,
                        "message": "Age must be at least 1",
                        "code": "age.too_young"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    // Invalid data
    let data = json!({
        "email": "invalid-email",
        "age": 0
    });
    let data_str = serde_json::to_string(&data).unwrap();
    
    eval.evaluate(&data_str, None, None).unwrap();
    let validation = eval.validate(&data_str, None, None).unwrap();
    
    assert!(validation.has_error, "Should have validation errors");
    assert_eq!(validation.errors.len(), 2, "Should have 2 errors");
    
    // Check pattern error has all fields
    let email_error = validation.errors.get("email").expect("Should have email error");
    assert_eq!(email_error.rule_type, "pattern");
    assert_eq!(email_error.message, "Invalid email format");
    assert_eq!(email_error.code, Some("email.invalid_format".to_string()));
    assert!(email_error.pattern.is_some(), "Pattern error should have pattern field");
    assert!(email_error.field_value.is_some(), "Pattern error should have field_value");
    assert_eq!(email_error.field_value.as_ref().unwrap(), "invalid-email");
    assert!(email_error.data.is_none(), "Pattern error should not have data field");
    
    // Check minValue error has code but not pattern/field_value
    let age_error = validation.errors.get("age").expect("Should have age error");
    assert_eq!(age_error.rule_type, "minValue");
    assert_eq!(age_error.message, "Age must be at least 1");
    assert_eq!(age_error.code, Some("age.too_young".to_string()));
    assert!(age_error.pattern.is_none(), "minValue error should not have pattern");
    assert!(age_error.field_value.is_none(), "minValue error should not have field_value");
    assert!(age_error.data.is_none(), "minValue error should not have data");
}

#[test]
fn test_validation_error_default_code() {
    // Schema without custom code - should generate default
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "rules": {
                    "required": {
                        "value": true,
                        "message": "Name is required"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    
    eval.evaluate(&data_str, None, None).unwrap();
    let validation = eval.validate(&data_str, None, None).unwrap();
    
    assert!(validation.has_error);
    let error = validation.errors.get("name").expect("Should have name error");
    
    // Default code should be "{path}.{ruleName}"
    assert_eq!(error.code, Some("name.required".to_string()));
}

#[test]
fn test_validation_error_serialization() {
    // Test that errors serialize correctly with optional fields
    let schema = json!({
        "type": "object",
        "properties": {
            "test": {
                "type": "string",
                "rules": {
                    "required": {
                        "value": true,
                        "message": "Required"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();
    
    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    
    eval.evaluate(&data_str, None, None).unwrap();
    let validation = eval.validate(&data_str, None, None).unwrap();
    
    // Serialize the validation result
    let json_str = serde_json::to_string(&validation).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    
    // Check structure
    assert_eq!(parsed["has_error"], true);
    assert!(parsed["errors"].is_object());
    
    let error = &parsed["errors"]["test"];
    assert_eq!(error["type"], "required");
    assert_eq!(error["message"], "Required");
    assert_eq!(error["code"], "test.required");
    
    // Optional fields should not be present when None
    assert!(!error.as_object().unwrap().contains_key("pattern"));
    assert!(!error.as_object().unwrap().contains_key("field_value"));
    assert!(!error.as_object().unwrap().contains_key("data"));
}
