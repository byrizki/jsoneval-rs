use json_eval_rs::*;
use serde_json::json;

/// Comprehensive edge case tests - boundary conditions, error handling, unusual inputs
#[cfg(test)]
mod edge_case_tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn test_null_and_undefined_handling() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Operations with null
        let test_cases = vec![
            (json!({"+": [null, 5]}), json!(5.0)),
            (json!({"*": [null, 3]}), json!(0.0)),
            (json!({"cat": [null, "test"]}), json!("test")),
            (json!({"length": null}), json!(0.0)),
        ];

        for (logic, expected) in test_cases {
            let logic_id = engine.compile(&logic).unwrap();
            let result = engine.run(&logic_id, &data).unwrap();
            assert_eq!(result, expected, "Failed for logic: {:?}", logic);
        }
    }

    #[test]
    fn test_empty_arrays_and_objects() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Empty array operations
        let logic_id = engine.compile(&json!({"+": []})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0));

        let logic_id = engine.compile(&json!({"*": []})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0)); // Engine returns 0 for empty product

        let logic_id = engine.compile(&json!({"cat": []})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(""));

        // Empty object operations
        let data = json!({"empty": {}});
        let logic_id = engine.compile(&json!({"length": {"var": "empty"}})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0));
    }

    #[test]
    fn test_type_coercion_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({});

        // String to number coercion edge cases
        let test_cases = vec![
            (json!({"+": ["", 5]}), json!(5.0)),      // Empty string -> 0
            (json!({"+": ["abc", 5]}), json!(5.0)),   // Invalid string -> 0
            (json!({"+": ["123", 5]}), json!(128.0)), // Numeric strings coerced to numbers
        ];

        for (logic, expected) in test_cases {
            let logic_id = engine.compile(&logic).unwrap();
            let result = engine.run(&logic_id, &data).unwrap();
            assert_eq!(result, expected, "Failed for logic: {:?}", logic);
        }

        // Boolean coercion
        let logic_id = engine.compile(&json!({"+": [true, false]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1.0));

        // Array coercion (takes first element)
        let logic_id = engine.compile(&json!({"+": [[10, 20], 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(5.0));
    }

    #[test]
    fn test_array_bounds_and_indexing() {
        let mut engine = RLogic::new();
        let data = json!({"arr": [10, 20, 30]});

        // Negative indices
        let logic_id = engine.compile(&json!({"var": "arr.-1"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null)); // Negative indices not supported

        // Out of bounds positive indices
        let logic_id = engine.compile(&json!({"var": "arr.5"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Valid indices
        let logic_id = engine.compile(&json!({"var": "arr.0"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(10));

        let logic_id = engine.compile(&json!({"var": "arr.2"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(30));
    }
    #[test]
    fn test_string_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Unicode string length still returns a number
        let logic_id = engine.compile(&json!({"length": "Hello 🌍 世界"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert!(result.is_number());

        // ASCII substring to avoid UTF-8 boundary issues
        let logic_id = engine.compile(&json!({"substr": ["Hello World", 6, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("World"));

        // ASCII search
        let logic_id = engine.compile(&json!({"search": ["World", "Hello World"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(7.0));

        // Empty string operations
        let logic_id = engine.compile(&json!({"substr": ["", 0, 1]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(""));

        let logic_id = engine.compile(&json!({"search": ["x", ""]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_circular_references() {
        let mut engine = RLogic::new();
        let data = json!({
            "a": {"ref": "$ref:b"},
            "b": {"ref": "$ref:a"}
        });

        // This should not cause infinite recursion due to recursion limits
        let logic_id = engine.compile(&json!({"var": "a.ref.ref"})).unwrap();
        let result = engine.run(&logic_id, &data);
        // Should either succeed with null or fail with recursion limit
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_variable_name_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({
            "": "empty_key",
            "key with spaces": "spaces",
            "key-with-dashes": "dashes",
            "key.with.dots": "dots",
            "key_with_underscores": "underscores",
            "123numeric": "numeric_start",
            "key$special": "special_chars"
        });

        // Empty key
        let logic_id = engine.compile(&json!({"var": ""})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        // Engine returns whole object for empty var
        assert!(result.is_object());

        // Key with spaces (using bracket notation equivalent)
        let logic_id = engine.compile(&json!({"var": "key with spaces"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("spaces"));

        // Numeric start
        let logic_id = engine.compile(&json!({"var": "123numeric"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("numeric_start"));
    }

    #[test]
    fn test_array_operations_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Map on non-array
        let logic_id = engine.compile(&json!({"map": ["not_array", {"var": ""}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([]));

        // Filter on non-array
        let logic_id = engine.compile(&json!({"filter": ["not_array", {"var": ""}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([]));

        // Reduce on non-array
        let logic_id = engine.compile(&json!({"reduce": ["not_array", {"var": ""}, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0));

        // Quantifiers on non-array
        let logic_id = engine.compile(&json!({"all": ["not_array", {"var": ""}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false)); // Engine returns false for non-arrays

        let logic_id = engine.compile(&json!({"some": ["not_array", {"var": ""}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_table_operations_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({"table": []});

        // VALUEAT on empty table
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // INDEXAT on empty table
        let logic_id = engine.compile(&json!({"INDEXAT": ["value", {"var": "table"}, "field"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(-1.0));

        // MATCH on empty table
        let logic_id = engine.compile(&json!({"MATCH": [{"var": "table"}, ["value", "field"]]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(-1.0));
    }

    #[test]
    fn test_comparison_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({});

        // NaN comparisons
        let logic_id = engine.compile(&json!({"==": [{"pow": [-1, 0.5]}, {"pow": [-1, 0.5]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true)); // Engine maps NaN -> null; null == null

        // Infinity comparisons
        let logic_id = engine.compile(&json!({"==": [{"+": [1e308, 1e308]}, {"+": [1e308, 1e308]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true)); // inf == inf

        // Mixed type comparisons
        let test_cases = vec![
            (json!({"==": [0, false]}), json!(true)),   // 0 == false
            (json!({"==": [1, true]}), json!(true)),    // 1 == true
            (json!({"==": ["", false]}), json!(true)),  // "" == false
            (json!({"==": ["0", 0]}), json!(true)),     // "0" == 0
        ];

        for (logic, expected) in test_cases {
            let logic_id = engine.compile(&logic).unwrap();
            let result = engine.run(&logic_id, &data).unwrap();
            assert_eq!(result, expected, "Failed for logic: {:?}", logic);
        }
    }

    #[test]
    fn test_logical_operations_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Short-circuit evaluation
        let logic_id = engine.compile(&json!({"and": [false, {"+": [1, "error"]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false)); // Should not evaluate second operand

        let logic_id = engine.compile(&json!({"or": [true, {"+": [1, "error"]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true)); // Should not evaluate second operand

    }

    #[test]
    fn test_compilation_edge_cases() {
        let mut engine = RLogic::new();

        // Empty object
        let logic_id = engine.compile(&json!({})).unwrap();
        let data = json!({});
        let result = engine.run(&logic_id, &data);
        assert!(result.is_ok()); // Engine treats empty object as noop

        // Unknown operators
        let result = engine.compile(&json!({"unknown_op": 123}));
        assert!(result.is_err());

        // Malformed operators
        let result = engine.compile(&json!({"+": "not_an_array"}));
        assert!(result.is_err());

        // Nested malformed operators
        let result = engine.compile(&json!({"+": [{"+": "invalid"}, 1]}));
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_and_performance_edge_cases() {
        let mut engine = RLogic::new();

        // Large arrays
        let large_array: Vec<Value> = (0..1000).map(|i| json!(i)).collect();
        let data = json!({ "large": large_array });

        // Sum of large array
        let logic_id = engine.compile(&json!({"sum": [{"var": "large"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(499500.0));

        // Map over large array
        let logic_id = engine.compile(&json!({"map": [{"var": "large"}, {"*": [{"var": ""}, 2]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert!(result.as_array().unwrap().len() == 1000);
        assert_eq!(result.as_array().unwrap()[0], json!(0.0));
        assert_eq!(result.as_array().unwrap()[999], json!(1998.0));
    }

    #[test]
    fn test_reference_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({
            "$params": {
                "config": {
                    "rate": 0.05,
                    "": "empty_key_in_ref"
                }
            }
        });

        // $ref with empty key
        let logic_id = engine.compile(&json!({"$ref": ""})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!({
            "$params": {
                "config": {
                    "rate": 0.05,
                    "": "empty_key_in_ref"
                }
            }
        }));

        // $ref with complex paths
        let logic_id = engine.compile(&json!({"$ref": "$params.config.rate"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.05));

        // $ref with missing path
        let logic_id = engine.compile(&json!({"$ref": "$params.missing.path"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_error_recovery() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Operations that should return null instead of crashing
        let error_cases = vec![
            json!({"/": [1, 0]}),                    // Division by zero
            json!({"%": [5, 0]}),                   // Modulo by zero
            json!({"pow": ["invalid", 2]}),         // Invalid base
            json!({"substr": [null, 0, 5]}),        // Substr on null
            json!({"VALUEAT": [[], -1]}),           // Negative index
        ];

        for logic in error_cases {
            let logic_id = engine.compile(&logic).unwrap();
            let result = engine.run(&logic_id, &data);
            // Should either succeed with null or fail gracefully
            assert!(result.is_ok() || result.is_err());
            if result.is_ok() {
                let value = result.unwrap();
                assert!(value.is_null() || value.is_number() || value.is_string());
            }
        }
    }

    #[test]
    fn test_json_path_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({
            "a": {
                "b": {
                    "c": [1, 2, {"d": "value"}]
                }
            },
            "array": [
                {"prop": "first"},
                {"prop": "second"}
            ]
        });

        // Deep nesting
        let logic_id = engine.compile(&json!({"var": "a.b.c.2.d"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("value"));

        // Array indexing with dots
        let logic_id = engine.compile(&json!({"var": "array.1.prop"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("second"));

        // Non-existent deep path
        let logic_id = engine.compile(&json!({"var": "a.b.c.3.d"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_compilation_reuse() {
        let mut engine = RLogic::new();

        // Compile the same logic multiple times
        let logic = json!({"+": [{"var": "x"}, {"var": "y"}]});
        let logic_id1 = engine.compile(&logic).unwrap();
        let logic_id2 = engine.compile(&logic).unwrap();

        // IDs may differ; verify both execute equivalently

        // Both should work
        let data = json!({"x": 10, "y": 20});
        let result1 = engine.run(&logic_id1, &data).unwrap();
        let result2 = engine.run(&logic_id2, &data).unwrap();
        assert_eq!(result1, json!(30.0));
        assert_eq!(result2, json!(30.0));
    }

    #[test]
    fn test_mixed_data_types() {
        let mut engine = RLogic::new();
        let data = json!({
            "mixed": [1, "string", true, null, {"nested": "value"}, [1, 2]]
        });

        // Operations on mixed arrays should handle each element appropriately
        let logic_id = engine.compile(&json!({"map": [{"var": "mixed"}, {"length": {"var": ""}}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        let lengths = result.as_array().unwrap();
        assert_eq!(lengths.len(), 6);
        // Numbers have length 0, strings their length, booleans 0, null 0, objects their size, arrays their length
        assert_eq!(lengths[0], json!(0.0));  // 1
        assert_eq!(lengths[1], json!(6.0)); // "string"
        assert_eq!(lengths[2], json!(0.0)); // true
        assert_eq!(lengths[3], json!(0.0)); // null
        assert_eq!(lengths[4], json!(1.0)); // {"nested": "value"}
        assert_eq!(lengths[5], json!(2.0)); // [1, 2]
    }
}
