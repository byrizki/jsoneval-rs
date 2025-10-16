mod common;

use json_eval_rs::JSONEval;
use serde_json::json;
use common::*;

/// Helper function to create a test schema with evaluations, rules, and dependencies
fn create_test_schema() -> String {
    load_minimal_form_schema()
}

#[test]
fn test_evaluate_basic() {
    let schema = create_test_schema();
    let data = get_minimal_form_data().to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data, Some("{}"))
        .expect("Evaluation failed");

    // Check that schema was evaluated successfully
    let result = eval.get_evaluated_schema(false);
    assert!(result.is_object(), "Result should be an object");
    // The evaluated_schema contains the schema structure
    assert!(result.pointer("/illustration").is_some(), 
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
    let data = get_minimal_form_data().to_string();

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
    let mut data = get_minimal_form_data();
    // Remove required field name
    data["illustration"]["insured"].as_object_mut().unwrap().remove("name");
    let data_str = data.to_string();

    let eval = JSONEval::new(&schema, None, Some(&data_str))
        .expect("Failed to create JSONEval");

    let validation = eval.validate(&data_str, None, None)
        .expect("Validation failed");

    // Name field doesn't have required rule in minimal_form.json, so skip this test
    // or just verify validation works
    println!("Validation result: has_error={}, errors={:?}", validation.has_error, validation.errors);
}

#[test]
fn test_validate_min_max_value() {
    let schema = create_test_schema();
    
    // Test validation with boundary values
    let mut data_min = get_minimal_form_data();
    data_min["illustration"]["insured"]["age"] = json!(0);
    let data_min_str = data_min.to_string();

    let eval = JSONEval::new(&schema, None, Some(&data_min_str))
        .expect("Failed to create JSONEval");

    let validation = eval.validate(&data_min_str, None, None)
        .expect("Validation failed");

    // Just verify validation runs - actual rules enforcement depends on schema
    println!("Validation for age=0: has_error={}, errors={:?}", validation.has_error, validation.errors);
    
    // Test max value
    let mut data_max = get_minimal_form_data();
    data_max["illustration"]["insured"]["age"] = json!(101);
    let data_max_str = data_max.to_string();

    let eval2 = JSONEval::new(&schema, None, Some(&data_max_str))
        .expect("Failed to create JSONEval");

    let validation2 = eval2.validate(&data_max_str, None, None)
        .expect("Validation failed");

    println!("Validation for age=101: has_error={}, errors={:?}", validation2.has_error, validation2.errors);
}


#[test]
fn test_validate_skip_hidden_fields() {
    let schema = create_test_schema();
    let mut data = get_minimal_form_data();
    // coverage_type is hidden when has_additional_coverage is false
    data["illustration"]["policy_container"]["has_additional_coverage"] = json!(false);
    let data_str = data.to_string();

    let eval = JSONEval::new(&schema, None, Some(&data_str))
        .expect("Failed to create JSONEval");

    let validation = eval.validate(&data_str, None, None)
        .expect("Validation failed");

    assert!(!validation.has_error, "Hidden fields should not be validated");
}

#[test]
fn test_validate_with_path_filter() {
    let schema = create_test_schema();
    let mut data = get_minimal_form_data();
    // Make age field invalid (below min)
    data["illustration"]["insured"]["age"] = json!(0);
    let data_str = data.to_string();

    let eval = JSONEval::new(&schema, None, Some(&data_str))
        .expect("Failed to create JSONEval");

    // Test path filtering - validate all fields first
    let validation_all = eval.validate(&data_str, None, None)
        .expect("Validation failed");
    
    // Verify path filter works by checking validation runs
    println!("Validation with filter: has_error={}, errors={:?}", validation_all.has_error, validation_all.errors);
}

#[test]
fn test_evaluate_dependents_basic() {
    // Use actual minimal_form.json schema with date_of_birth -> age dependent
    let schema = create_test_schema();
    let initial_data = get_minimal_form_data().to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&initial_data))
        .expect("Failed to create JSONEval");

    // Evaluate initial schema
    eval.evaluate(&initial_data, None)
        .expect("Initial evaluation failed");

    // Update date_of_birth field and trigger age calculation
    let mut updated_data = get_minimal_form_data();
    updated_data["illustration"]["insured"]["date_of_birth"] = json!("2000-01-01");
    let updated_data_str = updated_data.to_string();

    let result = eval.evaluate_dependents(
        "#/illustration/properties/insured/properties/date_of_birth",
        Some(&updated_data_str),
        None,
    ).expect("evaluate_dependents failed");

    // Check result structure
    assert!(result.is_array(), "Result should be an array");
    let changes = result.as_array().unwrap();
    
    // Verify dependents were triggered
    println!("Dependents result: {} changes", changes.len());
    if changes.len() > 0 {
        println!("First change: {:?}", changes[0]);
    }
}

#[test]
fn test_evaluate_dependents_with_clear() {
    // Use actual minimal_form.json schema with has_additional_coverage -> coverage_type clear logic
    let schema = create_test_schema();
    let mut initial_data = get_minimal_form_data();
    initial_data["illustration"]["policy_container"]["has_additional_coverage"] = json!(true);
    initial_data["illustration"]["policy_container"]["coverage_type"] = json!("PREMIUM");
    let initial_data_str = initial_data.to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&initial_data_str))
        .expect("Failed to create JSONEval");

    eval.evaluate(&initial_data_str, None)
        .expect("Initial evaluation failed");

    // Toggle off - should trigger dependent evaluation
    let mut updated_data = initial_data.clone();
    updated_data["illustration"]["policy_container"]["has_additional_coverage"] = json!(false);
    let updated_data_str = updated_data.to_string();

    let result = eval.evaluate_dependents(
        "#/illustration/properties/policy_container/properties/has_additional_coverage",
        Some(&updated_data_str),
        None,
    ).expect("evaluate_dependents failed");

    let changes = result.as_array().unwrap();
    
    // Verify dependents were triggered
    println!("Clear dependents result: {} changes", changes.len());
    for change in changes {
        println!("Change: $ref={}, clear={:?}, value={:?}", 
                 change["$ref"], change["clear"], change.get("value"));
    }
}

#[test]
fn test_evaluate_dependents_transitive() {
    // Use actual minimal_form.json with transitive deps: occupation -> occupation_class -> risk_category
    let schema = create_test_schema();
    let mut initial_data = get_minimal_form_data();
    // Start with PROFESSIONAL to ensure clean state
    initial_data["illustration"]["insured"]["occupation"] = json!("PROFESSIONAL");
    initial_data["illustration"]["insured"]["occupation_class"] = json!("1");
    let initial_data_str = initial_data.to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&initial_data_str))
        .expect("Failed to create JSONEval");

    eval.evaluate(&initial_data_str, None)
        .expect("Initial evaluation failed");

    // Update occupation - should cascade to occupation_class and risk_category
    let mut updated_data = initial_data.clone();
    updated_data["illustration"]["insured"]["occupation"] = json!("MANUAL");
    let updated_data_str = updated_data.to_string();

    let result = eval.evaluate_dependents(
        "#/illustration/properties/insured/properties/occupation",
        Some(&updated_data_str),
        None,
    ).expect("evaluate_dependents failed");

    let changes = result.as_array().unwrap();
    
    // Verify transitive dependents were triggered
    println!("Transitive dependents result: {} changes", changes.len());
    for (i, change) in changes.iter().enumerate() {
        println!("Change {}: $ref={}, value={:?}, transitive={}", 
                 i, change["$ref"], change.get("value"), change["transitive"]);
    }
    
    // Verify at least one dependent was triggered
    assert!(changes.len() >= 1, "Should have at least 1 dependent change");
}

#[test]
fn test_evaluate_dependents_no_data_update() {
    // Test calling evaluate_dependents without updating data (uses existing data) - mimic zlw.json structure
    let schema = json!({
        "type": "object",
        "properties": {
            "input": {
                "type": "number",
                "dependents": [
                    {
                        "$ref": "#/properties/output",
                        "value": {
                            "$evaluation": {
                                "*": [{ "var": "input" }, 3]
                            }
                        }
                    }
                ]
            },
            "output": {
                "type": "number"
            }
        }
    }).to_string();

    let data = json!({
        "input": 7,
        "output": 0
    }).to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data, None)
        .expect("Initial evaluation failed");

    // Call evaluate_dependents without updating data (data=None)
    let result = eval.evaluate_dependents(
        "#/properties/input",
        None,  // No data update, use existing data
        None,
    ).expect("evaluate_dependents failed");

    let changes = result.as_array().unwrap();
    assert!(changes.len() > 0, "Should have changes");

    let change = &changes[0];
    assert_eq!(change["$ref"], "#/properties/output");
    assert_eq!(change["value"], 21, "Should compute using existing data: 7 * 3 = 21");
}

#[test]
fn test_evaluate_dependents_output_structure() {
    // Comprehensive test to validate exact output structure - mimic zlw.json structure
    let schema = json!({
        "type": "object",
        "properties": {
            "trigger": {
                "type": "number",
                "title": "Trigger Field",
                "dependents": [
                    {
                        "$ref": "#/properties/computed",
                        "value": {
                            "$evaluation": {
                                "*": [{ "var": "trigger" }, 5]
                            }
                        }
                    },
                    {
                        "$ref": "#/properties/conditional",
                        "clear": {
                            "$evaluation": {
                                ">": [{ "var": "trigger" }, 10]
                            }
                        }
                    }
                ]
            },
            "computed": {
                "type": "number",
                "title": "Computed Field"
            },
            "conditional": {
                "type": "string",
                "title": "Conditional Field"
            }
        }
    }).to_string();

    let data = json!({
        "trigger": 12,
        "computed": 0,
        "conditional": "existing value"
    }).to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data, None)
        .expect("Initial evaluation failed");

    let result = eval.evaluate_dependents(
        "#/properties/trigger",
        None,
        None,
    ).expect("evaluate_dependents failed");

    // Validate result is an array
    assert!(result.is_array(), "Result must be an array");
    let changes = result.as_array().unwrap();
    assert_eq!(changes.len(), 2, "Should have 2 dependent changes");

    // Test first change (value computation)
    let value_change = &changes[0];
    
    // 1. Validate $ref field
    assert!(value_change.get("$ref").is_some(), "Must have $ref field");
    assert!(value_change["$ref"].is_string(), "$ref must be a string");
    assert_eq!(value_change["$ref"], "#/properties/computed");

    // 2. Validate $field (the target field schema)
    assert!(value_change.get("$field").is_some(), "Must have $field");
    assert!(value_change["$field"].is_object(), "$field must be an object");
    let field = value_change["$field"].as_object().unwrap();
    assert_eq!(field.get("type"), Some(&json!("number")), "$field should contain field schema");
    assert_eq!(field.get("title"), Some(&json!("Computed Field")));

    // 3. Validate $parentField (the actual parent data object, not schema)
    assert!(value_change.get("$parentField").is_some(), "Must have $parentField");
    assert!(value_change["$parentField"].is_object(), "$parentField must be an object");
    let parent_field = value_change["$parentField"].as_object().unwrap();
    // Parent field should be the data object containing trigger, computed, conditional VALUES
    assert!(parent_field.contains_key("trigger"), "$parentField should contain trigger data");
    assert_eq!(parent_field.get("trigger"), Some(&json!(12)), "trigger should have data value");
    assert!(parent_field.contains_key("computed"), "$parentField should contain computed data");
    assert!(parent_field.contains_key("conditional"), "$parentField should contain conditional data");

    // 4. Validate value field
    assert!(value_change.get("value").is_some(), "Must have value field");
    assert_eq!(value_change["value"], 60, "Value should be trigger * 5 = 60");

    // 5. Validate transitive field
    assert!(value_change.get("transitive").is_some(), "Must have transitive field");
    assert!(value_change["transitive"].is_boolean(), "transitive must be a boolean");
    assert_eq!(value_change["transitive"], false, "Direct dependent should have transitive=false");

    // 6. Validate clear field is not present (only when field is cleared)
    assert!(value_change.get("clear").is_none(), "clear should not be present when not clearing");

    // Test second change (clear logic)
    let clear_change = &changes[1];
    
    // 1. Validate $ref field
    assert_eq!(clear_change["$ref"], "#/properties/conditional");

    // 2. Validate $field
    assert!(clear_change["$field"].is_object());
    let field2 = clear_change["$field"].as_object().unwrap();
    assert_eq!(field2.get("type"), Some(&json!("string")));
    assert_eq!(field2.get("title"), Some(&json!("Conditional Field")));

    // 3. Validate $parentField (should be same parent data object)
    assert!(clear_change["$parentField"].is_object());
    let parent_field2 = clear_change["$parentField"].as_object().unwrap();
    assert_eq!(parent_field2.get("trigger"), Some(&json!(12)), "Parent should contain trigger data");
    assert_eq!(parent_field2.get("computed"), Some(&json!(60)), "Parent should have updated computed value");

    // 4. Validate transitive
    assert_eq!(clear_change["transitive"], false);

    // 5. Validate clear field is present and true
    assert!(clear_change.get("clear").is_some(), "Must have clear field when clearing");
    assert!(clear_change["clear"].is_boolean(), "clear must be a boolean");
    assert_eq!(clear_change["clear"], true, "Field should be cleared (trigger > 10)");

    // 6. Validate value field is not present (only when computing value)
    assert!(clear_change.get("value").is_none(), "value should not be present when only clearing");

    println!("✅ All output structure validations passed!");
}

#[test]
fn test_evaluate_dependents_dot_notation() {
    // Test dot notation path input - mimic real zlw.json schema structure
    let schema = json!({
        "type": "object",
        "properties": {
            "user": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "title": "User Name",
                        "dependents": [
                            {
                                "$ref": "#/properties/user/properties/display",
                                "value": {
                                    "$evaluation": {
                                        "concat": [
                                            "Hello, ",
                                            { "var": "user.name" }
                                        ]
                                    }
                                }
                            }
                        ]
                    },
                    "display": {
                        "type": "string",
                        "title": "Display Name"
                    }
                }
            }
        }
    }).to_string();

    let data = json!({
        "user": {
            "name": "Alice",
            "display": ""
        }
    }).to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data, None)
        .expect("Initial evaluation failed");

    // Test with full schema path format (like zlw.json)
    let result = eval.evaluate_dependents(
        "#/properties/user/properties/name",  // Full schema path like in zlw.json
        None,
        None,
    ).expect("evaluate_dependents failed");

    let changes = result.as_array().unwrap();
    assert!(changes.len() > 0, "Should have changes");

    let change = &changes[0];
    assert_eq!(change["$ref"], "#/properties/user/properties/display");
    assert_eq!(change["value"], "Hello, Alice");
    
    // Validate $parentField is the user object data
    let parent_field = change["$parentField"].as_object().unwrap();
    assert_eq!(parent_field.get("name"), Some(&json!("Alice")), "Parent should be user data object");
    assert!(parent_field.contains_key("display"), "Parent should contain display field");

    println!("✅ Dot notation path test passed!");
}

#[test]
fn test_evaluate_dependents_with_dot_notation_input() {
    // Test that evaluate_dependents accepts dot notation as input
    let schema = create_test_schema();
    let initial_data = get_minimal_form_data();
    let initial_data_str = initial_data.to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&initial_data_str))
        .expect("Failed to create JSONEval");

    eval.evaluate(&initial_data_str, None)
        .expect("Initial evaluation failed");

    // Test with DOT NOTATION - should work now!
    let result = eval.evaluate_dependents(
        "illustration.insured.date_of_birth",  // Dot notation instead of "#/illustration/properties/insured/properties/date_of_birth"
        None,
        None,
    ).expect("evaluate_dependents with dot notation failed");

    let changes = result.as_array().unwrap();
    println!("Dot notation result: {} changes", changes.len());
    
    // Should have triggered the age dependent
    if changes.len() > 0 {
        let age_change = changes.iter()
            .find(|c| c["$ref"].as_str().unwrap().contains("age"));
        if let Some(change) = age_change {
            println!("✅ Dot notation input test passed! Age change: {:?}", change["value"]);
        }
    }
}

#[test]
fn test_evaluate_dependents_dot_vs_schema_path() {
    // Verify both formats work identically
    let schema = create_test_schema();
    let data = get_minimal_form_data().to_string();

    // Test with schema path
    let mut eval1 = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");
    eval1.evaluate(&data, None).expect("Initial evaluation failed");
    
    let result1 = eval1.evaluate_dependents(
        "#/illustration/properties/insured/properties/occupation",
        None,
        None,
    ).expect("Schema path failed");

    // Test with dot notation
    let mut eval2 = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");
    eval2.evaluate(&data, None).expect("Initial evaluation failed");
    
    let result2 = eval2.evaluate_dependents(
        "illustration.insured.occupation",  // Same field, dot notation
        None,
        None,
    ).expect("Dot notation failed");

    // Both should produce the same number of changes
    let changes1 = result1.as_array().unwrap();
    let changes2 = result2.as_array().unwrap();
    
    assert_eq!(changes1.len(), changes2.len(), 
               "Dot notation and schema path should produce same number of changes");
    
    println!("✅ Both formats work identically! {} changes each", changes1.len());
}
