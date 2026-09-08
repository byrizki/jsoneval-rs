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

    eval.evaluate(&data_str, None, None, None).unwrap();
    let validation = eval.validate(&data_str, None, None, None, None).unwrap();

    assert!(validation.has_error, "Should have validation errors");
    assert_eq!(validation.errors.len(), 2, "Should have 2 errors");

    // Check pattern error has all fields
    let email_error = validation
        .errors
        .get("email")
        .expect("Should have email error");
    assert_eq!(email_error.rule_type, "pattern");
    assert_eq!(email_error.message, "Invalid email format");
    assert_eq!(email_error.code, Some("email.invalid_format".to_string()));
    assert!(
        email_error.pattern.is_some(),
        "Pattern error should have pattern field"
    );
    assert!(
        email_error.field_value.is_some(),
        "Pattern error should have field_value"
    );
    assert_eq!(email_error.field_value.as_ref().unwrap(), "invalid-email");
    assert!(
        email_error.data.is_none(),
        "Pattern error should not have data field"
    );

    // Check minValue error has code but not pattern/field_value
    let age_error = validation.errors.get("age").expect("Should have age error");
    assert_eq!(age_error.rule_type, "minValue");
    assert_eq!(age_error.message, "Age must be at least 1");
    assert_eq!(age_error.code, Some("age.too_young".to_string()));
    assert!(
        age_error.pattern.is_none(),
        "minValue error should not have pattern"
    );
    assert!(
        age_error.field_value.is_none(),
        "minValue error should not have field_value"
    );
    assert!(
        age_error.data.is_none(),
        "minValue error should not have data"
    );
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

    eval.evaluate(&data_str, None, None, None).unwrap();
    let validation = eval.validate(&data_str, None, None, None, None).unwrap();

    assert!(validation.has_error);
    let error = validation
        .errors
        .get("name")
        .expect("Should have name error");

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

    eval.evaluate(&data_str, None, None, None).unwrap();
    let validation = eval.validate(&data_str, None, None, None, None).unwrap();

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
    assert!(!error.as_object().unwrap().contains_key("fieldValue"));
    assert!(!error.as_object().unwrap().contains_key("data"));

    // Test with fieldValue present
    let schema_pattern = json!({
        "type": "object",
        "properties": {
            "code": {
                "type": "string",
                "rules": {
                    "pattern": {
                        "value": "^[0-9]+$",
                        "message": "Must be digits"
                    }
                }
            }
        }
    });

    let schema_pattern_str = serde_json::to_string(&schema_pattern).unwrap();
    let mut eval_pattern = JSONEval::new(&schema_pattern_str, None, None).unwrap();
    let data_pattern = json!({ "code": "abc" });
    let data_pattern_str = serde_json::to_string(&data_pattern).unwrap();

    eval_pattern
        .evaluate(&data_pattern_str, None, None, None)
        .unwrap();
    let validation_pattern = eval_pattern
        .validate(&data_pattern_str, None, None, None, None)
        .unwrap();

    let json_str_pattern = serde_json::to_string(&validation_pattern).unwrap();
    let parsed_pattern: serde_json::Value = serde_json::from_str(&json_str_pattern).unwrap();

    let error_pattern = &parsed_pattern["errors"]["code"];
    assert_eq!(error_pattern["type"], "pattern");
    assert_eq!(error_pattern["fieldValue"], "abc");
}

#[test]
fn test_validate_cache_identical_data() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "rules": {
                    "required": { "value": true, "message": "Name is required" }
                }
            },
            "score": {
                "type": "number",
                "rules": {
                    "minValue": { "value": 50, "message": "Minimum score is 50" }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({ "name": "Alice", "score": 40 });
    let data_str = serde_json::to_string(&data).unwrap();

    let res1 = eval.validate(&data_str, None, None, None, None).unwrap();
    assert!(res1.has_error);
    assert_eq!(res1.errors.len(), 1);
    assert!(res1.errors.contains_key("score"));

    // Second validate call with identical data should hit cache and return matching result
    let res2 = eval.validate(&data_str, None, None, None, None).unwrap();
    assert_eq!(res1.has_error, res2.has_error);
    assert_eq!(res1.errors.len(), res2.errors.len());
    assert_eq!(
        res1.errors.get("score").unwrap().message,
        res2.errors.get("score").unwrap().message
    );
}

#[test]
fn test_validate_cache_incremental_field_change() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "rules": {
                    "required": { "value": true, "message": "Name is required" }
                }
            },
            "age": {
                "type": "number",
                "rules": {
                    "minValue": { "value": 18, "message": "Must be at least 18" }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    // 1. Initial invalid data (age < 18)
    let d1 = serde_json::to_string(&json!({ "name": "Bob", "age": 16 })).unwrap();
    let r1 = eval.validate(&d1, None, None, None, None).unwrap();
    assert!(r1.has_error);
    assert_eq!(r1.errors.len(), 1);
    assert!(r1.errors.contains_key("age"));

    // 2. Incremental change: fix age, name unchanged
    let d2 = serde_json::to_string(&json!({ "name": "Bob", "age": 20 })).unwrap();
    let r2 = eval.validate(&d2, None, None, None, None).unwrap();
    assert!(!r2.has_error);
    assert_eq!(r2.errors.len(), 0);

    // 3. Incremental change: age remains valid, name emptied
    let d3 = serde_json::to_string(&json!({ "name": "", "age": 20 })).unwrap();
    let r3 = eval.validate(&d3, None, None, None, None).unwrap();
    assert!(r3.has_error);
    assert_eq!(r3.errors.len(), 1);
    assert!(r3.errors.contains_key("name"));
}

#[test]
fn test_validate_disabled_field_with_rules() {
    let schema = json!({
        "type": "object",
        "properties": {
            "fixed_id": {
                "type": "string",
                "disabled": true,
                "rules": {
                    "required": { "value": true, "message": "ID is required" },
                    "minLength": { "value": 3, "message": "ID must have 3+ characters" }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    // By default (validate_readonly = None or Some(false)):
    // Missing value on disabled field -> skipped, NO validation error
    let d1 = serde_json::to_string(&json!({})).unwrap();
    let r1_default = eval.validate(&d1, None, None, None, None).unwrap();
    assert!(!r1_default.has_error, "Disabled field must be skipped by default");
    assert!(!r1_default.errors.contains_key("fixed_id"));

    let r1_false = eval.validate(&d1, None, None, None, Some(false)).unwrap();
    assert!(!r1_false.has_error, "Disabled field must be skipped when validate_readonly=false");

    // When validate_readonly = Some(true):
    // Missing value on disabled field -> fails required rule
    let r1_true = eval.validate(&d1, None, None, None, Some(true)).unwrap();
    assert!(r1_true.has_error, "Disabled field with required rule must be validated when validate_readonly=true");
    assert!(r1_true.errors.contains_key("fixed_id"));
    assert_eq!(r1_true.errors["fixed_id"].rule_type, "required");

    // Invalid length on disabled field with validate_readonly=true -> should fail minLength rule
    let d2 = serde_json::to_string(&json!({ "fixed_id": "AB" })).unwrap();
    let r2_true = eval.validate(&d2, None, None, None, Some(true)).unwrap();
    assert!(r2_true.has_error);
    assert!(r2_true.errors.contains_key("fixed_id"));
    assert_eq!(r2_true.errors["fixed_id"].rule_type, "minLength");

    // With validate_readonly=false, invalid length on disabled field is skipped
    let r2_false = eval.validate(&d2, None, None, None, Some(false)).unwrap();
    assert!(!r2_false.has_error);

    // Valid value on disabled field -> should pass in both modes
    let d3 = serde_json::to_string(&json!({ "fixed_id": "ABC" })).unwrap();
    let r3_default = eval.validate(&d3, None, None, None, None).unwrap();
    assert!(!r3_default.has_error);
    let r3_true = eval.validate(&d3, None, None, None, Some(true)).unwrap();
    assert!(!r3_true.has_error);
}


#[test]
fn test_validate_unmapped_field_missing_from_data_and_eval_data() {
    let schema = json!({
        "type": "object",
        "properties": {
            "unmapped_required_code": {
                "type": "string",
                "rules": {
                    "required": { "value": true, "message": "Code is required" }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data_str = "{}";
    let res = eval.validate(data_str, None, None, None, None).unwrap();
    assert!(res.has_error);
    assert!(res.errors.contains_key("unmapped_required_code"));
    assert_eq!(res.errors["unmapped_required_code"].rule_type, "required");
}

#[test]
fn test_validate_cache_invalidation_on_reload() {
    let schema1 = json!({
        "type": "object",
        "properties": {
            "val": {
                "type": "number",
                "rules": {
                    "minValue": { "value": 10, "message": "Min 10" }
                }
            }
        }
    });

    let schema2 = json!({
        "type": "object",
        "properties": {
            "val": {
                "type": "number",
                "rules": {
                    "minValue": { "value": 100, "message": "Min 100" }
                }
            }
        }
    });

    let mut eval = JSONEval::new(&schema1.to_string(), None, None).unwrap();
    let d = serde_json::to_string(&json!({ "val": 50 })).unwrap();

    let r1 = eval.validate(&d, None, None, None, None).unwrap();
    assert!(!r1.has_error, "50 >= 10");

    // Reload with schema2 where min is 100
    eval.reload_schema(&schema2.to_string(), None, None).unwrap();
    let r2 = eval.validate(&d, None, None, None, None).unwrap();
    assert!(r2.has_error, "50 < 100, cache should have been invalidated on reload");
    assert_eq!(r2.errors["val"].message, "Min 100");
}

#[test]
fn test_validate_readonly_field_direct_and_conditional() {
    let schema = json!({
        "type": "object",
        "properties": {
            "ro_field": {
                "type": "string",
                "readonly": true,
                "rules": {
                    "required": { "value": true, "message": "Readonly field is required" }
                }
            },
            "cond_ro_field": {
                "type": "string",
                "condition": {
                    "readonly": true
                },
                "rules": {
                    "required": { "value": true, "message": "Conditional readonly is required" }
                }
            },
            "active_field": {
                "type": "string",
                "rules": {
                    "required": { "value": true, "message": "Active field is required" }
                }
            }
        }
    });

    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    let data_empty = "{}";

    // Default (validate_readonly = None or false): only active_field should have error
    let res_default = eval.validate(data_empty, None, None, None, None).unwrap();
    assert!(res_default.has_error);
    assert_eq!(res_default.errors.len(), 1);
    assert!(res_default.errors.contains_key("active_field"));
    assert!(!res_default.errors.contains_key("ro_field"));
    assert!(!res_default.errors.contains_key("cond_ro_field"));

    // validate_readonly = Some(true): ro_field, cond_ro_field, and active_field must all be validated
    let res_true = eval.validate(data_empty, None, None, None, Some(true)).unwrap();
    assert!(res_true.has_error);
    assert_eq!(res_true.errors.len(), 3);
    assert!(res_true.errors.contains_key("active_field"));
    assert!(res_true.errors.contains_key("ro_field"));
    assert!(res_true.errors.contains_key("cond_ro_field"));
}

#[test]
fn test_validate_layout_disabled_ref() {
    let schema = json!({
        "type": "object",
        "properties": {
            "elem": {
                "type": "string",
                "rules": {
                    "required": { "value": true, "message": "Elem is required" }
                }
            }
        },
        "$layout": {
            "elements": [
                {
                    "type": "Control",
                    "$ref": "#/properties/elem",
                    "disabled": true
                }
            ]
        }
    });

    let mut eval = JSONEval::new(&schema.to_string(), None, None).unwrap();
    let data_empty = "{}";

    // Resolving layout fills layout resolution state
    eval.resolve_layout(false).unwrap();

    // Default: layout disabled field is skipped
    let res_default = eval.validate(data_empty, None, None, None, None).unwrap();
    assert!(!res_default.has_error, "Layout disabled ref should be skipped by default");

    // validate_readonly = Some(true): layout disabled field is validated
    let res_true = eval.validate(data_empty, None, None, None, Some(true)).unwrap();
    assert!(res_true.has_error, "Layout disabled ref should be validated when validate_readonly=true");
    assert!(res_true.errors.contains_key("elem"));
}


