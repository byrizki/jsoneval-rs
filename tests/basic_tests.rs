use json_eval_rs::*;
use serde_json::json;

/// Basic functionality tests - fundamental operations
#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_compile_basic_types() {
        let mut engine = RLogic::new();

        // Test null - just verify compilation succeeds
        assert!(engine.compile(&json!(null)).is_ok());

        // Test boolean
        assert!(engine.compile(&json!(true)).is_ok());

        // Test number
        assert!(engine.compile(&json!(42)).is_ok());

        // Test string
        assert!(engine.compile(&json!("hello")).is_ok());
    }

    #[test]
    fn test_evaluate_direct() {
        let mut engine = RLogic::new();
        let data = json!({"name": "John", "age": 30});

        // Test null evaluation
        let logic_id = engine.compile(&json!(null)).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Test boolean evaluation
        let logic_id = engine.compile(&json!(true)).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        // Test number evaluation
        let logic_id = engine.compile(&json!(42)).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(42));

        // Test string evaluation
        let logic_id = engine.compile(&json!("hello")).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("hello"));
    }

    #[test]
    fn test_variable_access() {
        let mut engine = RLogic::new();
        let data = json!({"name": "John", "age": 30.0});

        let logic_id = engine.compile(&json!({"var": "name"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("John"));

        let logic_id = engine.compile(&json!({"var": "age"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(30.0));
    }

    #[test]
    fn test_variable_with_default() {
        let mut engine = RLogic::new();
        let data = json!({"name": "John"});

        let logic_id = engine.compile(&json!({"var": ["age", 25]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(25));
    }

    #[test]
    fn test_nested_variable_access() {
        let mut engine = RLogic::new();
        let data = json!({
            "user": {
                "name": "John",
                "profile": {
                    "age": 30,
                    "city": "NYC"
                }
            },
            "items": [1, 2, 3]
        });

        // Nested object access (dotted notation internally converted to JSON pointer)
        let logic_id = engine.compile(&json!({"var": "user.name"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("John"));

        // Deep nesting (dotted notation internally converted to JSON pointer)
        let logic_id = engine.compile(&json!({"var": "user.profile.city"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("NYC"));

        // Array access (dotted notation internally converted to JSON pointer)
        let logic_id = engine.compile(&json!({"var": "items.0"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1));
    }

    #[test]
    fn test_reference_access() {
        let mut engine = RLogic::new();
        let data = json!({
            "user": {"name": "John"},
            "$params": {"rate": 0.05}
        });

        // $ref access (dotted notation internally converted to JSON pointer)
        let logic_id = engine.compile(&json!({"$ref": "user.name"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("John"));

        // $ref with default (dotted notation internally converted to JSON pointer)
        let logic_id = engine.compile(&json!({"$ref": ["missing.field", "default"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("default"));
    }

    #[test]
    fn test_missing_variables() {
        let mut engine = RLogic::new();
        let data = json!({"name": "John"});

        // Missing variable without default
        let logic_id = engine.compile(&json!({"var": "missing"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_array_literals() {
        let mut engine = RLogic::new();
        let data = json!({});

        let logic_id = engine.compile(&json!([1, 2, 3, "hello", true, null])).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([1, 2, 3, "hello", true, null]));
    }

    #[test]
    fn test_empty_data() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Variable access on empty data
        let logic_id = engine.compile(&json!({"var": "anything"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Literal on empty data
        let logic_id = engine.compile(&json!("test")).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("test"));
    }

    #[test]
    fn test_compilation_errors() {
        let mut engine = RLogic::new();

        // Invalid operator
        let result = engine.compile(&json!({"invalid_op": 123}));
        assert!(result.is_err());

        // Malformed structure
        let result = engine.compile(&json!({"+": "not_an_array"}));
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluation_errors() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Division by zero
        let logic_id = engine.compile(&json!({"/": [10, 0]})).unwrap();
        let result = engine.run(&logic_id, &data);
        assert!(result.is_ok()); // Should return null, not error
        assert_eq!(result.unwrap(), json!(null));
    }

    #[test]
    fn test_json_pointer_paths() {
        let mut engine = RLogic::new();
        let data = json!({
            "user": {
                "profile": {
                    "address": {
                        "city": "NYC",
                        "zip": "10001"
                    },
                    "contacts": ["email@test.com", "555-1234"]
                },
                "settings": {"theme": "dark"}
            }
        });

        // JSON pointer style access
        let logic_id = engine.compile(&json!({"var": "user.profile.address.city"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("NYC"));

        // Array access
        let logic_id = engine.compile(&json!({"var": "user.profile.contacts.0"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("email@test.com"));

        // Multiple levels with arrays
        let logic_id = engine.compile(&json!({"var": "user.profile.contacts.1"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("555-1234"));
    }

    #[test]
    fn test_complex_nested_structures() {
        let mut engine = RLogic::new();
        let data = json!({
            "policy": {
                "insured": {
                    "name": "John Doe",
                    "age": 35,
                    "dependents": [
                        {"name": "Jane", "age": 8, "relation": "child"},
                        {"name": "Bob", "age": 5, "relation": "child"}
                    ]
                },
                "coverage": {
                    "amount": 500000,
                    "riders": [
                        {"type": "critical_illness", "premium": 150},
                        {"type": "disability", "premium": 200}
                    ]
                }
            }
        });

        // Deep nested access
        let logic_id = engine.compile(&json!({"var": "policy.insured.dependents.0.name"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("Jane"));

        // Access coverage details
        let logic_id = engine.compile(&json!({"var": "policy.coverage.riders.1.type"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("disability"));
    }

    #[test]
    fn test_variable_with_complex_defaults() {
        let mut engine = RLogic::new();
        let data = json!({"existing": "value"});

        // Default with string
        let logic_id = engine.compile(&json!({
            "var": ["missing", "default_value"]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("default_value"));

        // Default with number
        let logic_id = engine.compile(&json!({
            "var": ["missing", 42]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(42));

        // Default with boolean
        let logic_id = engine.compile(&json!({
            "var": ["missing", true]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        // Default with null
        let logic_id = engine.compile(&json!({
            "var": ["missing", null]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Existing value should override default
        let logic_id = engine.compile(&json!({
            "var": ["existing", "default"]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("value"));
    }

    #[test]
    fn test_mixed_data_types() {
        let mut engine = RLogic::new();
        let data = json!({
            "string": "test",
            "number": 42,
            "float": 3.14,
            "boolean": true,
            "null": null,
            "array": [1, 2, 3],
            "object": {"key": "value"}
        });

        let test_cases = vec![
            ("string", json!("test")),
            ("number", json!(42)),
            ("float", json!(3.14)),
            ("boolean", json!(true)),
            ("null", json!(null)),
            ("array", json!([1, 2, 3])),
            ("object", json!({"key": "value"})),
        ];

        for (key, expected) in test_cases {
            let logic_id = engine.compile(&json!({"var": key})).unwrap();
            let result = engine.run(&logic_id, &data).unwrap();
            assert_eq!(result, expected, "Failed for key: {}", key);
        }
    }

    #[test]
    fn test_real_world_data_access() {
        let mut engine = RLogic::new();
        
        // Simulate real insurance product data
        let data = json!({
            "$params": {
                "constants": {
                    "MAX_POL_AGE": 100,
                    "MORT_MULTIPLIER": 0.97
                },
                "references": {
                    "PRODUCT_PACKAGE": [
                        {"PROD_PACKAGE": "Essential", "COMP_CODE": "ESS"},
                        {"PROD_PACKAGE": "Premium", "COMP_CODE": "PREM"}
                    ]
                }
            },
            "illustration": {
                "product_benefit": {
                    "benefit_type": {
                        "prem_freq": 12,
                        "prem_pay_period": 10
                    }
                },
                "insured": {
                    "insage": 30,
                    "ins_dob": "1995-01-01"
                }
            }
        });

        // Access constants
        let logic_id = engine.compile(&json!({"$ref": "$params.constants.MAX_POL_AGE"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(100));

        // Access array element in references
        let logic_id = engine.compile(&json!({
            "$ref": "$params.references.PRODUCT_PACKAGE.0.COMP_CODE"
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("ESS"));

        // Access illustration data
        let logic_id = engine.compile(&json!({
            "var": "illustration.product_benefit.benefit_type.prem_freq"
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(12));
    }

    #[test]
    fn test_null_vs_undefined_behavior() {
        let mut engine = RLogic::new();
        let data = json!({
            "explicit_null": null,
            "empty_string": "",
            "zero": 0,
            "false_value": false
        });

        // Explicit null
        let logic_id = engine.compile(&json!({"var": "explicit_null"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Missing key (undefined)
        let logic_id = engine.compile(&json!({"var": "undefined_key"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Empty string (falsy but defined)
        let logic_id = engine.compile(&json!({"var": "empty_string"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(""));

        // Zero (falsy but defined)
        let logic_id = engine.compile(&json!({"var": "zero"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0));

        // False (falsy but defined)
        let logic_id = engine.compile(&json!({"var": "false_value"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_deeply_nested_access() {
        let mut engine = RLogic::new();
        let data = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "value": "deep"
                            }
                        }
                    }
                }
            }
        });

        // Very deep nesting
        let logic_id = engine.compile(&json!({
            "var": "level1.level2.level3.level4.level5.value"
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("deep"));

        // Partial path
        let logic_id = engine.compile(&json!({
            "var": "level1.level2.level3"
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!({
            "level4": {
                "level5": {
                    "value": "deep"
                }
            }
        }));
    }
}
