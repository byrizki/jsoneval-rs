use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_evaluation_array_format() {
    // Schema with "evaluation" as array of objects
    let schema = json!({
        "type": "object",
        "properties": {
            "form": {
                "type": "object",
                "properties": {
                    "owner": {
                        "type": "object",
                        "properties": {
                            "gender": {
                                "type": "string",
                                "title": "Owner Gender",
                                "fieldType": "options"
                            }
                        }
                    },
                    "member": {
                        "type": "object",
                        "properties": {
                            "relation": {
                                "type": "string",
                                "title": "Relation"
                            },
                            "gender": {
                                "type": "string",
                                "title": "Member Gender",
                                "rules": {
                                    "required": {
                                        "value": true,
                                        "message": "VALIDATION_REQUIRED"
                                    },
                                    "evaluation": [
                                        {
                                            "code": "relation.gender.is.same",
                                            "message": "Member gender and owner gender cannot be the same when relation is 2.",
                                            "$evaluation": {
                                                "if": [
                                                    {
                                                        "==": [
                                                            {
                                                                "$ref": "#/form/properties/member/properties/relation"
                                                            },
                                                            "2"
                                                        ]
                                                    },
                                                    {
                                                        "!=": [
                                                            {
                                                                "$ref": "#/form/properties/owner/properties/gender"
                                                            },
                                                            {
                                                                "$ref": "#/form/properties/member/properties/gender"
                                                            }
                                                        ]
                                                    },
                                                    true
                                                ]
                                            }
                                        }
                                    ]
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

    // Test 1: Validation fails when relation is "2" and genders are same
    let data_invalid = json!({
        "form": {
            "owner": {
                "gender": "M"
            },
            "member": {
                "relation": "2",
                "gender": "M"
            }
        }
    });
    let data_invalid_str = serde_json::to_string(&data_invalid).unwrap();

    eval.evaluate(&data_invalid_str, None, None, None).unwrap();
    let validation_invalid = eval
        .validate(&data_invalid_str, None, None, None, None, None)
        .unwrap();

    assert!(
        validation_invalid.has_error,
        "Should have validation error when genders are same with relation=2"
    );
    assert!(
        validation_invalid.errors.contains_key("form.member.gender"),
        "Should have error for gender"
    );

    let error = validation_invalid.errors.get("form.member.gender").unwrap();
    assert_eq!(error.rule_type, "evaluation");
    assert_eq!(error.code, Some("relation.gender.is.same".to_string()));
    assert_eq!(
        error.message,
        "Member gender and owner gender cannot be the same when relation is 2."
    );

    // Test 2: Validation passes when relation is "2" and genders are different
    let data_valid = json!({
        "form": {
            "owner": {
                "gender": "M"
            },
            "member": {
                "relation": "2",
                "gender": "F"
            }
        }
    });
    let data_valid_str = serde_json::to_string(&data_valid).unwrap();

    eval.evaluate(&data_valid_str, None, None, None).unwrap();
    let validation_valid = eval
        .validate(&data_valid_str, None, None, None, None, None)
        .unwrap();

    assert!(
        !validation_valid.has_error,
        "Should have no validation error when genders are different"
    );

    // Test 3: Validation passes when relation is not "2"
    let data_other_relation = json!({
        "form": {
            "owner": {
                "gender": "M"
            },
            "member": {
                "relation": "1",
                "gender": "M"
            }
        }
    });
    let data_other_str = serde_json::to_string(&data_other_relation).unwrap();

    eval.evaluate(&data_other_str, None, None, None).unwrap();
    let validation_other = eval
        .validate(&data_other_str, None, None, None, None, None)
        .unwrap();

    assert!(
        !validation_other.has_error,
        "Should have no validation error when relation is not '2'"
    );
}

#[test]
fn test_custom_evaluation_rule() {
    // Schema with custom evaluation rule that checks if age is valid for employment
    let schema = json!({
        "type": "object",
        "properties": {
            "person": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "title": "Name"
                    },
                    "age": {
                        "type": "number",
                        "title": "Age",
                        "rules": {
                            "minAge": {
                                "value": {
                                    "$evaluation": {
                                        ">=": [
                                            { "$ref": "#/person/properties/age" },
                                            18
                                        ]
                                    }
                                },
                                "message": "Must be at least 18 years old",
                                "code": "age.too_young",
                                "data": {
                                    "minimumAge": 18
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

    // Test 1: Invalid age (below 18)
    let data_invalid = json!({
        "person": {
            "name": "John Doe",
            "age": 16
        }
    });
    let data_invalid_str = serde_json::to_string(&data_invalid).unwrap();

    eval.evaluate(&data_invalid_str, None, None, None).unwrap();
    let validation_invalid = eval
        .validate(&data_invalid_str, None, None, None, None, None)
        .unwrap();

    assert!(
        validation_invalid.has_error,
        "Should have validation error for age < 18"
    );
    assert!(
        validation_invalid.errors.contains_key("person.age"),
        "Should have error for person.age"
    );

    let error = validation_invalid.errors.get("person.age").unwrap();
    assert_eq!(
        error.rule_type, "evaluation",
        "Error type should be 'evaluation'"
    );
    assert_eq!(error.message, "Must be at least 18 years old");
    assert_eq!(error.code, Some("age.too_young".to_string()));
    assert!(error.data.is_some(), "Should have data field");

    if let Some(data) = &error.data {
        assert_eq!(data["minimumAge"], 18);
    }

    // Test 2: Valid age (18 or above)
    let data_valid = json!({
        "person": {
            "name": "Jane Smith",
            "age": 25
        }
    });
    let data_valid_str = serde_json::to_string(&data_valid).unwrap();

    eval.evaluate(&data_valid_str, None, None, None).unwrap();
    let validation_valid = eval
        .validate(&data_valid_str, None, None, None, None, None)
        .unwrap();

    assert!(
        !validation_valid.has_error,
        "Should have no validation error for age >= 18"
    );
    assert!(
        !validation_valid.errors.contains_key("person.age"),
        "Should have no error for person.age when valid"
    );
}

#[test]
fn test_evaluation_rule_with_dynamic_message() {
    // Schema with dynamic message based on evaluation
    let schema = json!({
        "type": "object",
        "properties": {
            "score": {
                "type": "number",
                "title": "Score",
                "rules": {
                    "passingScore": {
                        "value": {
                            "$evaluation": {
                                ">=": [
                                    { "$ref": "#/score" },
                                    60
                                ]
                            }
                        },
                        "message": {
                            "$evaluation": {
                                "if": [
                                    {
                                        "<": [
                                            { "$ref": "#/score" },
                                            40
                                        ]
                                    },
                                    "Score is critically low (below 40)",
                                    "Score is below passing grade (60)"
                                ]
                            }
                        },
                        "code": "score.failing"
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

    // Test with critically low score
    let data_critical = json!({
        "score": 30
    });
    let data_critical_str = serde_json::to_string(&data_critical).unwrap();

    eval.evaluate(&data_critical_str, None, None, None).unwrap();
    let validation = eval
        .validate(&data_critical_str, None, None, None, None, None)
        .unwrap();

    assert!(validation.has_error);
    let error = validation.errors.get("score").unwrap();
    assert_eq!(error.message, "Score is critically low (below 40)");

    // Test with low but not critical score
    let data_low = json!({
        "score": 50
    });
    let data_low_str = serde_json::to_string(&data_low).unwrap();

    eval.evaluate(&data_low_str, None, None, None).unwrap();
    let validation2 = eval
        .validate(&data_low_str, None, None, None, None, None)
        .unwrap();

    assert!(validation2.has_error);
    let error2 = validation2.errors.get("score").unwrap();
    assert_eq!(error2.message, "Score is below passing grade (60)");
}

#[test]
fn test_evaluation_rule_with_evaluated_data() {
    // Schema with data field containing $evaluation
    let schema = json!({
        "type": "object",
        "properties": {
            "quantity": {
                "type": "number",
                "title": "Quantity",
                "rules": {
                    "stockCheck": {
                        "value": {
                            "$evaluation": {
                                "<=": [
                                    { "$ref": "#/quantity" },
                                    100
                                ]
                            }
                        },
                        "message": "Quantity exceeds available stock",
                        "data": {
                            "maxStock": {
                                "$evaluation": 100
                            },
                            "requested": {
                                "$evaluation": { "$ref": "#/quantity" }
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
        "quantity": 150
    });
    let data_str = serde_json::to_string(&data).unwrap();

    eval.evaluate(&data_str, None, None, None).unwrap();
    let validation = eval
        .validate(&data_str, None, None, None, None, None)
        .unwrap();

    assert!(validation.has_error);
    let error = validation.errors.get("quantity").unwrap();
    assert!(error.data.is_some());

    if let Some(data_field) = &error.data {
        assert_eq!(data_field["maxStock"], 100);
        assert_eq!(data_field["requested"], 150);
    }
}
