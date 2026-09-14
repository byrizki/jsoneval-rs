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
    let validation = eval.validate(&data_str, None, None, None, None, None).unwrap();

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
        email_error.data.is_some(),
        "Pattern error should have data field with field title"
    );
    assert_eq!(
        email_error.data.as_ref().unwrap()["title"],
        "Email"
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
        age_error.data.is_some(),
        "minValue error should have data"
    );
    let age_data = age_error.data.as_ref().unwrap();
    assert_eq!(age_data["title"], "Age");
    assert_eq!(age_data["minValue"], 1);
    assert!(age_data.get("min").is_none(), "Should not have redundant min key");
    assert!(age_data.get("max").is_none(), "Should not have redundant max key");
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
    let validation = eval.validate(&data_str, None, None, None, None, None).unwrap();

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
    let validation = eval.validate(&data_str, None, None, None, None, None).unwrap();

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
    assert_eq!(error["data"]["required"], true);

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
        .validate(&data_pattern_str, None, None, None, None, None)
        .unwrap();

    let json_str_pattern = serde_json::to_string(&validation_pattern).unwrap();
    let parsed_pattern: serde_json::Value = serde_json::from_str(&json_str_pattern).unwrap();

    let error_pattern = &parsed_pattern["errors"]["code"];
    assert_eq!(error_pattern["type"], "pattern");
    assert_eq!(error_pattern["fieldValue"], "abc");
    assert!(!error_pattern.as_object().unwrap().contains_key("data"));
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

    let res1 = eval.validate(&data_str, None, None, None, None, None).unwrap();
    assert!(res1.has_error);
    assert_eq!(res1.errors.len(), 1);
    assert!(res1.errors.contains_key("score"));

    // Second validate call with identical data should hit cache and return matching result
    let res2 = eval.validate(&data_str, None, None, None, None, None).unwrap();
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
    let r1 = eval.validate(&d1, None, None, None, None, None).unwrap();
    assert!(r1.has_error);
    assert_eq!(r1.errors.len(), 1);
    assert!(r1.errors.contains_key("age"));

    // 2. Incremental change: fix age, name unchanged
    let d2 = serde_json::to_string(&json!({ "name": "Bob", "age": 20 })).unwrap();
    let r2 = eval.validate(&d2, None, None, None, None, None).unwrap();
    assert!(!r2.has_error);
    assert_eq!(r2.errors.len(), 0);

    // 3. Incremental change: age remains valid, name emptied
    let d3 = serde_json::to_string(&json!({ "name": "", "age": 20 })).unwrap();
    let r3 = eval.validate(&d3, None, None, None, None, None).unwrap();
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
    let r1_default = eval.validate(&d1, None, None, None, None, None).unwrap();
    assert!(!r1_default.has_error, "Disabled field must be skipped by default");
    assert!(!r1_default.errors.contains_key("fixed_id"));

    let r1_false = eval.validate(&d1, None, None, None, Some(false), None).unwrap();
    assert!(!r1_false.has_error, "Disabled field must be skipped when validate_readonly=false");

    // When validate_readonly = Some(true):
    // Missing value on disabled field -> fails required rule
    let r1_true = eval.validate(&d1, None, None, None, Some(true), None).unwrap();
    assert!(r1_true.has_error, "Disabled field with required rule must be validated when validate_readonly=true");
    assert!(r1_true.errors.contains_key("fixed_id"));
    assert_eq!(r1_true.errors["fixed_id"].rule_type, "required");

    // Invalid length on disabled field with validate_readonly=true -> should fail minLength rule
    let d2 = serde_json::to_string(&json!({ "fixed_id": "AB" })).unwrap();
    let r2_true = eval.validate(&d2, None, None, None, Some(true), None).unwrap();
    assert!(r2_true.has_error);
    assert!(r2_true.errors.contains_key("fixed_id"));
    assert_eq!(r2_true.errors["fixed_id"].rule_type, "minLength");

    // With validate_readonly=false, invalid length on disabled field is skipped
    let r2_false = eval.validate(&d2, None, None, None, Some(false), None).unwrap();
    assert!(!r2_false.has_error);

    // Valid value on disabled field -> should pass in both modes
    let d3 = serde_json::to_string(&json!({ "fixed_id": "ABC" })).unwrap();
    let r3_default = eval.validate(&d3, None, None, None, None, None).unwrap();
    assert!(!r3_default.has_error);
    let r3_true = eval.validate(&d3, None, None, None, Some(true), None).unwrap();
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
    let res = eval.validate(data_str, None, None, None, None, None).unwrap();
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

    let r1 = eval.validate(&d, None, None, None, None, None).unwrap();
    assert!(!r1.has_error, "50 >= 10");

    // Reload with schema2 where min is 100
    eval.reload_schema(&schema2.to_string(), None, None).unwrap();
    let r2 = eval.validate(&d, None, None, None, None, None).unwrap();
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
    let res_default = eval.validate(data_empty, None, None, None, None, None).unwrap();
    assert!(res_default.has_error);
    assert_eq!(res_default.errors.len(), 1);
    assert!(res_default.errors.contains_key("active_field"));
    assert!(!res_default.errors.contains_key("ro_field"));
    assert!(!res_default.errors.contains_key("cond_ro_field"));

    // validate_readonly = Some(true): ro_field, cond_ro_field, and active_field must all be validated
    let res_true = eval.validate(data_empty, None, None, None, Some(true), None).unwrap();
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
    let res_default = eval.validate(data_empty, None, None, None, None, None).unwrap();
    assert!(!res_default.has_error, "Layout disabled ref should be skipped by default");

    // validate_readonly = Some(true): layout disabled field is validated
    let res_true = eval.validate(data_empty, None, None, None, Some(true), None).unwrap();
    assert!(res_true.has_error, "Layout disabled ref should be validated when validate_readonly=true");
    assert!(res_true.errors.contains_key("elem"));
}

#[test]
fn test_validate_with_include_subforms_flag() {
    let schema = json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "rules": {
                    "required": { "value": true, "message": "Title is required" }
                }
            },
            "contacts": {
                "type": "array",
                "items": {
                    "properties": {
                        "name": {
                            "type": "string",
                            "rules": {
                                "required": { "value": true, "message": "Name is required" }
                            }
                        },
                        "phone": {
                            "type": "string",
                            "rules": {
                                "required": { "value": true, "message": "Phone is required" }
                            }
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({
        "title": "",
        "contacts": [
            { "name": "Alice", "phone": "" },
            { "name": "", "phone": "12345" }
        ]
    });
    let data_str = serde_json::to_string(&data).unwrap();

    // 1. include_subforms = None (default): only root fields validated
    let res_default = eval.validate(&data_str, None, None, None, None, None).unwrap();
    assert!(res_default.has_error);
    assert_eq!(res_default.errors.len(), 1);
    assert!(res_default.errors.contains_key("title"));
    assert!(!res_default.errors.contains_key("contacts.0.phone"));
    assert!(!res_default.errors.contains_key("contacts.1.name"));

    // 2. include_subforms = Some(false): only root fields validated
    let res_false = eval.validate(&data_str, None, None, None, None, Some(false)).unwrap();
    assert!(res_false.has_error);
    assert_eq!(res_false.errors.len(), 1);
    assert!(res_false.errors.contains_key("title"));

    // 3. include_subforms = Some(true): root + all array subform items validated
    let res_true = eval.validate(&data_str, None, None, None, None, Some(true)).unwrap();
    assert!(res_true.has_error);
    assert_eq!(res_true.errors.len(), 3);
    assert!(res_true.errors.contains_key("title"));
    assert!(res_true.errors.contains_key("contacts.0.phone"));
    assert!(res_true.errors.contains_key("contacts.1.name"));
    assert!(!res_true.errors.contains_key("contacts.0.name"));
    assert!(!res_true.errors.contains_key("contacts.1.phone"));

    let err_c0_phone = &res_true.errors["contacts.0.phone"];
    assert_eq!(err_c0_phone.message, "Phone is required");
    assert_eq!(err_c0_phone.code, Some("contacts.0.phone.required".to_string()));

    let err_c1_name = &res_true.errors["contacts.1.name"];
    assert_eq!(err_c1_name.message, "Name is required");
    assert_eq!(err_c1_name.code, Some("contacts.1.name.required".to_string()));
}

#[test]
fn test_validate_with_include_subforms_paths_filter_and_cache() {
    let schema = json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "rules": {
                    "required": { "value": true, "message": "Title is required" }
                }
            },
            "contacts": {
                "type": "array",
                "items": {
                    "properties": {
                        "name": {
                            "type": "string",
                            "rules": {
                                "required": { "value": true, "message": "Name is required" }
                            }
                        },
                        "phone": {
                            "type": "string",
                            "rules": {
                                "required": { "value": true, "message": "Phone is required" }
                            }
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({
        "title": "",
        "contacts": [
            { "name": "Alice", "phone": "" },
            { "name": "", "phone": "12345" }
        ]
    });
    let data_str = serde_json::to_string(&data).unwrap();

    // 1. Selective path: only contacts.0.phone
    let paths_item0 = vec!["contacts.0.phone".to_string()];
    let res_item0 = eval.validate(&data_str, None, Some(&paths_item0), None, None, Some(true)).unwrap();
    assert!(res_item0.has_error);
    assert_eq!(res_item0.errors.len(), 1);
    assert!(res_item0.errors.contains_key("contacts.0.phone"));

    // 2. Selective path: whole item 0 -> should validate item 0 fields only
    let paths_whole_item0 = vec!["contacts.0".to_string()];
    let res_whole0 = eval.validate(&data_str, None, Some(&paths_whole_item0), None, None, Some(true)).unwrap();
    assert!(res_whole0.has_error);
    assert_eq!(res_whole0.errors.len(), 1);
    assert!(res_whole0.errors.contains_key("contacts.0.phone"));
    assert!(!res_whole0.errors.contains_key("contacts.1.name"));

    // 3. Selective path: title only
    let paths_title = vec!["title".to_string()];
    let res_title = eval.validate(&data_str, None, Some(&paths_title), None, None, Some(true)).unwrap();
    assert!(res_title.has_error);
    assert_eq!(res_title.errors.len(), 1);
    assert!(res_title.errors.contains_key("title"));

    // 4. Cache isolation: full validate with include_subforms=true then include_subforms=false
    let r_full_true = eval.validate(&data_str, None, None, None, None, Some(true)).unwrap();
    assert_eq!(r_full_true.errors.len(), 3);

    let r_full_false = eval.validate(&data_str, None, None, None, None, Some(false)).unwrap();
    assert_eq!(r_full_false.errors.len(), 1, "Cache hit must not leak subform errors when include_subforms=false");

    let r_full_true_again = eval.validate(&data_str, None, None, None, None, Some(true)).unwrap();
    assert_eq!(r_full_true_again.errors.len(), 3, "Cached subforms result returned when include_subforms=true");
}

#[test]
fn test_validate_with_include_subforms_nested_and_items_root_key() {
    let schema = json!({
        "type": "object",
        "properties": {
            "form": {
                "type": "object",
                "properties": {
                    "items": {
                        "type": "array",
                        "itemsRootKey": "items",
                        "items": {
                            "properties": {
                                "amount": {
                                    "type": "number",
                                    "rules": {
                                        "minValue": { "value": 1000, "message": "Min amount is 1000" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({
        "form": {
            "items": [
                { "amount": 500 },
                { "amount": 2000 }
            ]
        }
    });
    let data_str = serde_json::to_string(&data).unwrap();

    let res = eval.validate(&data_str, None, None, None, None, Some(true)).unwrap();
    assert!(res.has_error);
    assert_eq!(res.errors.len(), 1);
    let err = res.errors.get("form.items.0.amount").expect("Should map to form.items.0.amount");
    assert_eq!(err.message, "Min amount is 1000");
    assert_eq!(err.code, Some("form.items.0.amount.minValue".to_string()));
    assert!(err.data.is_some());
    let err_data = err.data.as_ref().unwrap();
    assert_eq!(err_data["minValue"], 1000);
    assert!(err_data.get("min").is_none(), "Should not have redundant min key");
}

#[test]
fn test_validation_data_numeric_range_and_companion_rules() {
    let schema = json!({
        "type": "object",
        "properties": {
            "score": {
                "type": "number",
                "rules": {
                    "minValue": { "value": 10, "message": "Min score is 10" },
                    "maxValue": { "value": 100, "message": "Max score is 100" }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    // Test failing minValue
    let data_low = json!({ "score": 5 });
    let res_low = eval.validate(&serde_json::to_string(&data_low).unwrap(), None, None, None, None, None).unwrap();
    assert!(res_low.has_error);
    let err_low = res_low.errors.get("score").unwrap();
    assert_eq!(err_low.rule_type, "minValue");
    assert!(err_low.data.is_some());
    let data_low_obj = err_low.data.as_ref().unwrap();
    assert_eq!(data_low_obj["minValue"], 10);
    assert_eq!(data_low_obj["maxValue"], 100);
    assert!(data_low_obj.get("min").is_none(), "Should not have redundant min key");
    assert!(data_low_obj.get("max").is_none(), "Should not have redundant max key");

    // Test failing maxValue
    let data_high = json!({ "score": 150 });
    let res_high = eval.validate(&serde_json::to_string(&data_high).unwrap(), None, None, None, None, None).unwrap();
    assert!(res_high.has_error);
    let err_high = res_high.errors.get("score").unwrap();
    assert_eq!(err_high.rule_type, "maxValue");
    assert!(err_high.data.is_some());
    let data_high_obj = err_high.data.as_ref().unwrap();
    assert_eq!(data_high_obj["minValue"], 10);
    assert_eq!(data_high_obj["maxValue"], 100);
    assert!(data_high_obj.get("min").is_none(), "Should not have redundant min key");
    assert!(data_high_obj.get("max").is_none(), "Should not have redundant max key");
}

#[test]
fn test_validation_data_length_range_and_companion_rules() {
    let schema = json!({
        "type": "object",
        "properties": {
            "code": {
                "type": "string",
                "rules": {
                    "minLength": { "value": 3, "message": "Min length 3" },
                    "maxLength": { "value": 8, "message": "Max length 8" }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    // Test failing minLength
    let data_short = json!({ "code": "ab" });
    let res_short = eval.validate(&serde_json::to_string(&data_short).unwrap(), None, None, None, None, None).unwrap();
    assert!(res_short.has_error);
    let err_short = res_short.errors.get("code").unwrap();
    assert_eq!(err_short.rule_type, "minLength");
    assert!(err_short.data.is_some());
    let data_short_obj = err_short.data.as_ref().unwrap();
    assert_eq!(data_short_obj["minLength"], 3);
    assert_eq!(data_short_obj["maxLength"], 8);
    assert!(data_short_obj.get("min").is_none(), "Should not have redundant min key");
    assert!(data_short_obj.get("max").is_none(), "Should not have redundant max key");

    // Test failing maxLength
    let data_long = json!({ "code": "toolongcodehere" });
    let res_long = eval.validate(&serde_json::to_string(&data_long).unwrap(), None, None, None, None, None).unwrap();
    assert!(res_long.has_error);
    let err_long = res_long.errors.get("code").unwrap();
    assert_eq!(err_long.rule_type, "maxLength");
    assert!(err_long.data.is_some());
    let data_long_obj = err_long.data.as_ref().unwrap();
    assert_eq!(data_long_obj["minLength"], 3);
    assert_eq!(data_long_obj["maxLength"], 8);
    assert!(data_long_obj.get("min").is_none(), "Should not have redundant min key");
    assert!(data_long_obj.get("max").is_none(), "Should not have redundant max key");
}

#[test]
fn test_validation_data_merge_with_schema_evaluation_data() {
    let schema = json!({
        "type": "object",
        "properties": {
            "premium": {
                "type": "number",
                "rules": {
                    "minValue": {
                        "value": 50000,
                        "message": "Minimum premium is 50000",
                        "data": {
                            "currency": "IDR",
                            "multiplier": 1000
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({ "premium": 25000 });
    let res = eval.validate(&serde_json::to_string(&data).unwrap(), None, None, None, None, None).unwrap();
    assert!(res.has_error);
    let err = res.errors.get("premium").unwrap();
    assert_eq!(err.rule_type, "minValue");
    assert!(err.data.is_some());
    let data_obj = err.data.as_ref().unwrap();
    // Rule constraints
    assert_eq!(data_obj["minValue"], 50000);
    assert!(data_obj.get("min").is_none(), "Should not have redundant min key");
    // Schema evaluation data merged
    assert_eq!(data_obj["currency"], "IDR");
    assert_eq!(data_obj["multiplier"], 1000);
}

#[test]
fn test_validation_data_dynamic_evaluation_in_rule_data() {
    let schema = json!({
        "type": "object",
        "properties": {
            "threshold": {
                "type": "number",
                "value": 100
            },
            "amount": {
                "type": "number",
                "rules": {
                    "minValue": {
                        "value": {
                            "$evaluation": { "$ref": "#/properties/threshold" }
                        },
                        "message": "Amount is below threshold",
                        "data": {
                            "customLimit": {
                                "$evaluation": { "$ref": "#/properties/threshold" }
                            },
                            "unit": "USD"
                        }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({ "threshold": 200, "amount": 50 });
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let res = eval.validate(&data_str, None, None, None, None, None).unwrap();
    assert!(res.has_error);
    let err = res.errors.get("amount").unwrap();
    assert_eq!(err.rule_type, "minValue");
    assert!(err.data.is_some());
    let data_obj = err.data.as_ref().unwrap();
    assert_eq!(data_obj["minValue"], 200);
    assert!(data_obj.get("min").is_none(), "Should not have redundant min key");
    assert_eq!(data_obj["customLimit"], 200);
    assert_eq!(data_obj["unit"], "USD");
}

#[test]
fn test_validation_data_required_and_pattern_with_schema_data() {
    let schema = json!({
        "type": "object",
        "properties": {
            "username": {
                "type": "string",
                "rules": {
                    "required": {
                        "value": true,
                        "message": "Username required",
                        "data": { "fieldGroup": "auth" }
                    }
                }
            },
            "phone": {
                "type": "string",
                "rules": {
                    "pattern": {
                        "value": "^[0-9]+$",
                        "message": "Digits only",
                        "data": { "expectedFormat": "numeric" }
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({ "username": "", "phone": "abc" });
    let res = eval.validate(&serde_json::to_string(&data).unwrap(), None, None, None, None, None).unwrap();
    assert!(res.has_error);

    let user_err = res.errors.get("username").unwrap();
    assert_eq!(user_err.rule_type, "required");
    assert!(user_err.data.is_some());
    assert_eq!(user_err.data.as_ref().unwrap()["fieldGroup"], "auth");

    let phone_err = res.errors.get("phone").unwrap();
    assert_eq!(phone_err.rule_type, "pattern");
    assert!(phone_err.data.is_some());
    assert_eq!(phone_err.data.as_ref().unwrap()["expectedFormat"], "numeric");
}

#[test]
fn test_validation_data_field_title_and_description() {
    let schema = json!({
        "type": "object",
        "properties": {
            "age": {
                "type": "number",
                "title": "Age of Applicant",
                "description": "Must be between 18 and 65 years old",
                "rules": {
                    "minValue": { "value": 18, "message": "Too young" },
                    "maxValue": { "value": 65, "message": "Too old" }
                }
            },
            "fullName": {
                "type": "string",
                "title": "Full Legal Name",
                "description": "As written in national ID card",
                "rules": {
                    "required": { "value": true, "message": "Full name is required" }
                }
            },
            "bio": {
                "type": "string",
                "title": "User Biography",
                "description": "Short description of yourself",
                "rules": {
                    "maxLength": { "value": 200, "message": "Bio too long" }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({
        "age": 16,
        "fullName": "",
        "bio": "a".repeat(205)
    });
    let data_str = serde_json::to_string(&data).unwrap();
    let res = eval.validate(&data_str, None, None, None, None, None).unwrap();
    assert!(res.has_error);
    assert_eq!(res.errors.len(), 3);

    // 1. age: has title, description, and boundary rules (minValue, companion maxValue)
    let age_err = res.errors.get("age").unwrap();
    let age_data = age_err.data.as_ref().expect("age should have data");
    assert_eq!(age_data["title"], "Age of Applicant");
    assert_eq!(age_data["description"], "Must be between 18 and 65 years old");
    assert_eq!(age_data["minValue"], 18);
    assert_eq!(age_data["maxValue"], 65);
    assert!(age_data.get("min").is_none(), "no redundant min");
    assert!(age_data.get("max").is_none(), "no redundant max");
    assert!(age_data.get("label").is_none(), "no label key");
    assert!(age_data.get("desc").is_none(), "no desc key");

    // 2. fullName: uses title and description, and has required: true
    let name_err = res.errors.get("fullName").unwrap();
    let name_data = name_err.data.as_ref().expect("fullName should have data");
    assert_eq!(name_data["title"], "Full Legal Name");
    assert_eq!(name_data["description"], "As written in national ID card");
    assert_eq!(name_data["required"], true);
    assert!(name_data.get("label").is_none(), "no label key");
    assert!(name_data.get("desc").is_none(), "no desc key");

    // 3. bio: title and description with maxLength
    let bio_err = res.errors.get("bio").unwrap();
    let bio_data = bio_err.data.as_ref().expect("bio should have data");
    assert_eq!(bio_data["title"], "User Biography");
    assert_eq!(bio_data["description"], "Short description of yourself");
    assert_eq!(bio_data["maxLength"], 200);
    assert!(bio_data.get("max").is_none(), "no redundant max");
    assert!(bio_data.get("label").is_none(), "no label key");
    assert!(bio_data.get("desc").is_none(), "no desc key");
}

#[test]
fn test_validation_data_required_rule_populates_required_true() {
    let schema = json!({
        "type": "object",
        "properties": {
            "email": {
                "type": "string",
                "rules": {
                    "required": {
                        "value": true,
                        "message": "Email is required"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    let data = json!({ "email": "" });
    let res = eval.validate(&serde_json::to_string(&data).unwrap(), None, None, None, None, None).unwrap();
    assert!(res.has_error);

    let err = res.errors.get("email").unwrap();
    assert_eq!(err.rule_type, "required");
    assert!(err.data.is_some());
    let data_obj = err.data.as_ref().unwrap();
    assert_eq!(data_obj["required"], true);
}



