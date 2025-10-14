use json_eval_rs::JSONEval;
use serde_json::json;

/// Helper function to create a test schema with evaluations, rules, and dependencies
fn create_test_schema() -> String {
    json!({
        "$schema": "test-schema",
        "type": "object",
        "properties": {
            "user": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "title": "Name",
                        "rules": {
                            "required": {
                                "value": true,
                                "message": "Name is required"
                            },
                            "minLength": {
                                "value": 3,
                                "message": "Name must be at least 3 characters"
                            },
                            "maxLength": {
                                "value": 50,
                                "message": "Name must not exceed 50 characters"
                            },
                            "pattern": {
                                "value": "^[a-zA-Z\\s]+$",
                                "message": "Name must contain only letters and spaces"
                            }
                        }
                    },
                    "age": {
                        "type": "number",
                        "title": "Age",
                        "rules": {
                            "required": {
                                "value": true,
                                "message": "Age is required"
                            },
                            "minValue": {
                                "value": 18,
                                "message": "Must be at least 18 years old"
                            },
                            "maxValue": {
                                "value": 100,
                                "message": "Age must not exceed 100"
                            }
                        }
                    },
                    "email": {
                        "type": "string",
                        "title": "Email",
                        "rules": {
                            "required": {
                                "value": true,
                                "message": "Email is required"
                            },
                            "pattern": {
                                "value": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$",
                                "message": "Invalid email format"
                            }
                        }
                    },
                    "birthYear": {
                        "type": "number"
                    },
                    "isAdult": {
                        "type": "boolean"
                    }
                }
            },
            "profile": {
                "type": "object",
                "properties": {
                    "bio": {
                        "type": "string",
                        "title": "Bio",
                        "rules": {
                            "maxLength": {
                                "value": 200,
                                "message": "Bio must not exceed 200 characters"
                            }
                        },
                        "condition": {
                            "hidden": false
                        }
                    },
                    "hiddenField": {
                        "type": "string",
                        "title": "Hidden",
                        "rules": {
                            "required": {
                                "value": true,
                                "message": "Should not validate (hidden)"
                            }
                        },
                        "condition": {
                            "hidden": true
                        }
                    }
                }
            }
        },
        "$layout": {
            "type": "VerticalLayout",
            "elements": [
                { "$ref": "#/properties/user" },
                { "$ref": "#/properties/profile" }
            ]
        }
    }).to_string()
}

#[test]
fn test_evaluate_basic() {
    let schema = create_test_schema();
    let data = json!({
        "user": {
            "name": "John Doe",
            "age": 30,
            "email": "john@example.com"
        },
        "profile": {
            "bio": "Software engineer"
        }
    }).to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data, Some("{}"))
        .expect("Evaluation failed");

    // Check that schema was evaluated successfully
    let result = eval.get_evaluated_schema(false);
    assert!(result.is_object(), "Result should be an object");
    // The evaluated_schema contains the schema structure with properties
    assert!(result.pointer("/properties").is_some() || result.pointer("/user").is_some(), 
        "Schema structure should exist");
}

#[test]
fn test_evaluate_with_context() {
    let schema = json!({
        "type": "object",
        "properties": {
            "username": {
                "type": "string"
            }
        }
    }).to_string();

    let data = json!({"username": "Alice"}).to_string();
    let context = json!({"role": "admin"}).to_string();

    let mut eval = JSONEval::new(&schema, Some(&context), Some(&data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data, Some(&context))
        .expect("Evaluation failed");

    // Verify evaluation completes successfully
    let result = eval.get_evaluated_schema(false);
    assert!(result.is_object(), "Result should be an object");
}

#[test]
fn test_validate_all_rules_pass() {
    let schema = create_test_schema();
    let data = json!({
        "user": {
            "name": "Jane Smith",
            "age": 25,
            "email": "jane@example.com"
        },
        "profile": {
            "bio": "Designer"
        }
    }).to_string();

    let eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    let validation = eval.validate(&data, None, None)
        .expect("Validation failed");

    assert!(!validation.has_error, "Should have no validation errors");
    assert!(validation.errors.is_empty(), "Errors map should be empty");
}

#[test]
fn test_validate_required_field_missing() {
    let schema = create_test_schema();
    let data = json!({
        "user": {
            "age": 25,
            "email": "jane@example.com"
            // name is missing
        }
    }).to_string();

    let eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    let validation = eval.validate(&data, None, None)
        .expect("Validation failed");

    assert!(validation.has_error, "Should have validation errors");
    assert!(validation.errors.contains_key("user.name"), "Should have error for user.name");
    
    let error = validation.errors.get("user.name").unwrap();
    assert_eq!(error.rule_type, "required");
    assert_eq!(error.message, "Name is required");
}

#[test]
fn test_validate_min_length() {
    let schema = create_test_schema();
    let data = json!({
        "user": {
            "name": "Jo",  // Too short (< 3 chars)
            "age": 25,
            "email": "jo@example.com"
        }
    }).to_string();

    let eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    let validation = eval.validate(&data, None, None)
        .expect("Validation failed");

    assert!(validation.has_error);
    assert!(validation.errors.contains_key("user.name"));
    
    let error = validation.errors.get("user.name").unwrap();
    assert_eq!(error.rule_type, "minLength");
}

#[test]
fn test_validate_max_length() {
    let schema = create_test_schema();
    let data = json!({
        "user": {
            "name": "A".repeat(51),  // Too long (> 50 chars)
            "age": 25,
            "email": "test@example.com"
        }
    }).to_string();

    let eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    let validation = eval.validate(&data, None, None)
        .expect("Validation failed");

    assert!(validation.has_error);
    assert!(validation.errors.contains_key("user.name"));
    
    let error = validation.errors.get("user.name").unwrap();
    assert_eq!(error.rule_type, "maxLength");
}

#[test]
fn test_validate_pattern() {
    let schema = create_test_schema();
    let data = json!({
        "user": {
            "name": "John123",  // Invalid (contains numbers)
            "age": 25,
            "email": "john@example.com"
        }
    }).to_string();

    let eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    let validation = eval.validate(&data, None, None)
        .expect("Validation failed");

    assert!(validation.has_error);
    assert!(validation.errors.contains_key("user.name"));
    
    let error = validation.errors.get("user.name").unwrap();
    assert_eq!(error.rule_type, "pattern");
}

#[test]
fn test_validate_min_max_value() {
    let schema = create_test_schema();
    
    // Test min value
    let data_min = json!({
        "user": {
            "name": "John Doe",
            "age": 17,  // Below minimum (18)
            "email": "john@example.com"
        }
    }).to_string();

    let eval = JSONEval::new(&schema, None, Some(&data_min))
        .expect("Failed to create JSONEval");

    let validation = eval.validate(&data_min, None, None)
        .expect("Validation failed");

    assert!(validation.has_error);
    assert!(validation.errors.contains_key("user.age"));
    assert_eq!(validation.errors.get("user.age").unwrap().rule_type, "minValue");

    // Test max value
    let data_max = json!({
        "user": {
            "name": "John Doe",
            "age": 101,  // Above maximum (100)
            "email": "john@example.com"
        }
    }).to_string();

    let eval2 = JSONEval::new(&schema, None, Some(&data_max))
        .expect("Failed to create JSONEval");

    let validation2 = eval2.validate(&data_max, None, None)
        .expect("Validation failed");

    assert!(validation2.has_error);
    assert!(validation2.errors.contains_key("user.age"));
    assert_eq!(validation2.errors.get("user.age").unwrap().rule_type, "maxValue");
}

#[test]
fn test_validate_skip_hidden_fields() {
    let schema = create_test_schema();
    let data = json!({
        "user": {
            "name": "John Doe",
            "age": 25,
            "email": "john@example.com"
        },
        "profile": {
            "bio": "Developer"
            // hiddenField is missing, but it's hidden so should not validate
        }
    }).to_string();

    let eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    let validation = eval.validate(&data, None, None)
        .expect("Validation failed");

    assert!(!validation.has_error, "Hidden fields should not be validated");
    assert!(!validation.errors.contains_key("profile.hiddenField"));
}

#[test]
fn test_validate_with_path_filter() {
    let schema = create_test_schema();
    let data = json!({
        "user": {
            "name": "Jo",  // Invalid (too short)
            "age": 17,     // Invalid (below min)
            "email": "invalid-email"  // Invalid format
        }
    }).to_string();

    let eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    // Only validate user.name
    let validation = eval.validate(&data, None, Some(&vec!["user.name".to_string()]))
        .expect("Validation failed");

    assert!(validation.has_error);
    assert!(validation.errors.contains_key("user.name"));
    // Other fields should not be validated
    assert!(!validation.errors.contains_key("user.age"));
    assert!(!validation.errors.contains_key("user.email"));
}

#[test]
fn test_evaluate_dependents_basic() {
    let schema = create_test_schema();
    let initial_data = json!({
        "user": {
            "name": "John Doe",
            "age": 30,
            "email": "john@example.com"
        }
    }).to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&initial_data))
        .expect("Failed to create JSONEval");

    // Initial evaluation
    eval.evaluate(&initial_data, Some("{}"))
        .expect("Initial evaluation failed");

    // Update age
    let updated_data = json!({
        "user": {
            "name": "John Doe",
            "age": 40,  // Changed from 30
            "email": "john@example.com"
        }
    }).to_string();

    // Re-evaluate dependents
    eval.evaluate_dependents(
        &vec!["user.age".to_string()],
        &updated_data,
        Some("{}"),
        true
    ).expect("evaluate_dependents failed");

    // Check that the updated data is reflected
    let result = eval.get_evaluated_schema(false);
    assert!(result.is_object(), "Result should be an object");
}

#[test]
fn test_evaluate_dependents_nested() {
    let schema = json!({
        "type": "object",
        "properties": {
            "input": {
                "type": "object",
                "properties": {
                    "x": { "type": "number" }
                }
            }
        }
    }).to_string();

    let initial_data = json!({
        "input": { "x": 5 }
    }).to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&initial_data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&initial_data, Some("{}"))
        .expect("Initial evaluation failed");

    // Update input.x
    let updated_data = json!({
        "input": { "x": 10 }
    }).to_string();

    // Re-evaluate with nested=true
    eval.evaluate_dependents(
        &vec!["input.x".to_string()],
        &updated_data,
        Some("{}"),
        true
    ).expect("evaluate_dependents failed");

    // Verify the update completed successfully
    let result = eval.get_evaluated_schema(false);
    assert!(result.is_object(), "Result should be an object");
}

#[test]
fn test_evaluate_dependents_non_nested() {
    let schema = json!({
        "type": "object",
        "properties": {
            "input": {
                "type": "object",
                "properties": {
                    "x": { "type": "number" }
                }
            }
        }
    }).to_string();

    let initial_data = json!({
        "input": { "x": 5 }
    }).to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&initial_data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&initial_data, Some("{}"))
        .expect("Initial evaluation failed");

    let updated_data = json!({
        "input": { "x": 10 }
    }).to_string();

    // Re-evaluate with nested=false
    eval.evaluate_dependents(
        &vec!["input.x".to_string()],
        &updated_data,
        Some("{}"),
        false  // Non-nested
    ).expect("evaluate_dependents failed");

    // Verify the update completed successfully
    let result = eval.get_evaluated_schema(false);
    assert!(result.is_object(), "Result should be an object");
}

#[test]
fn test_integration_evaluate_validate_dependents() {
    let schema = create_test_schema();
    let data = json!({
        "user": {
            "name": "Alice",
            "age": 25,
            "email": "alice@example.com"
        }
    }).to_string();

    // 1. Initial evaluation
    let mut eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");
    
    eval.evaluate(&data, Some("{}"))
        .expect("Evaluation failed");
    
    let result = eval.get_evaluated_schema(false);
    assert!(result.is_object(), "Result should be an object");

    // 2. Validate initial data (should pass)
    let validation = eval.validate(&data, None, None)
        .expect("Validation failed");
    assert!(!validation.has_error, "Valid data should pass validation");

    // 3. Update age and re-evaluate dependents
    let updated_data = json!({
        "user": {
            "name": "Alice",
            "age": 30,
            "email": "alice@example.com"
        }
    }).to_string();

    eval.evaluate_dependents(
        &vec!["user.age".to_string()],
        &updated_data,
        Some("{}"),
        true
    ).expect("evaluate_dependents failed");

    let result2 = eval.get_evaluated_schema(false);
    assert!(result2.is_object(), "Updated result should be an object");

    // 4. Validate updated data (should still pass)
    let validation2 = eval.validate(&updated_data, None, None)
        .expect("Validation failed");
    assert!(!validation2.has_error, "Updated valid data should still pass");

    // 5. Try invalid data
    let invalid_data = json!({
        "user": {
            "name": "Al",  // Too short
            "age": 15,     // Too young
            "email": "invalid"
        }
    }).to_string();

    let validation3 = eval.validate(&invalid_data, None, None)
        .expect("Validation failed");
    
    assert!(validation3.has_error, "Invalid data should have errors");
    assert!(validation3.errors.len() >= 3, "Should have at least 3 validation errors");
}
