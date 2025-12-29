use json_eval_rs::*;
use serde_json::json;

/// Array operation tests - map, filter, reduce, quantifiers, etc.
#[cfg(test)]
mod array_tests {
    use super::*;

    #[test]
    fn test_array_map() {
        let mut engine = RLogic::new();
        let data = json!({"numbers": [1, 2, 3, 4, 5]});

        // Double each number
        let logic_id = engine.compile(&json!({"map": [{"var": "numbers"}, {"*": [{"var": ""}, 2]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([2, 4, 6, 8, 10]));

        // Convert to strings
        let logic_id = engine.compile(&json!({"map": [{"var": "numbers"}, {"cat": [{"var": ""}, "x"]} ]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(["1x", "2x", "3x", "4x", "5x"]));
    }

    #[test]
    fn test_array_filter() {
        let mut engine = RLogic::new();
        let data = json!({"numbers": [1, 2, 3, 4, 5, 6]});

        // Filter even numbers
        let logic_id = engine.compile(&json!({"filter": [{"var": "numbers"}, {"==": [{"%": [{"var": ""}, 2]}, 0]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([2, 4, 6]));

        // Filter numbers greater than 3
        let logic_id = engine.compile(&json!({"filter": [{"var": "numbers"}, {">": [{"var": ""}, 3]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([4, 5, 6]));
    }

    #[test]
    fn test_array_reduce() {
        let mut engine = RLogic::new();
        let data = json!({"numbers": [1, 2, 3, 4, 5]});

        // Sum all numbers
        let logic_id = engine.compile(&json!({"reduce": [{"var": "numbers"}, {"+": [{"var": "accumulator"}, {"var": "current"}]}, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(15));

        // Find maximum
        let logic_id = engine.compile(&json!({"reduce": [{"var": "numbers"}, {"if": [{">": [{"var": "current"}, {"var": "accumulator"}]}, {"var": "current"}, {"var": "accumulator"}]}, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(5));
    }

    #[test]
    fn test_array_quantifiers() {
        let mut engine = RLogic::new();
        let data = json!({"numbers": [1, 2, 3, 4, 5]});

        // All even numbers? (should be false)
        let logic_id = engine.compile(&json!({"all": [{"var": "numbers"}, {"==": [{"%": [{"var": ""}, 2]}, 0]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));

        // Some even numbers? (should be true)
        let logic_id = engine.compile(&json!({"some": [{"var": "numbers"}, {"==": [{"%": [{"var": ""}, 2]}, 0]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        // None are negative? (should be true)
        let logic_id = engine.compile(&json!({"none": [{"var": "numbers"}, {"<": [{"var": ""}, 0]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_array_merge() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Merge arrays
        let logic_id = engine.compile(&json!({"merge": [[1, 2], [3, 4], [5]]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([1, 2, 3, 4, 5]));

        // Merge with mixed types
        let logic_id = engine.compile(&json!({"merge": [["a", "b"], [1, 2], [true, null]]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(["a", "b", 1, 2, true, null]));
    }

    #[test]
    fn test_array_in() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Value in array
        let logic_id = engine.compile(&json!({"in": [3, [1, 2, 3, 4, 5]]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        let logic_id = engine.compile(&json!({"in": [6, [1, 2, 3, 4, 5]]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));

        // Substring in string
        let logic_id = engine.compile(&json!({"in": ["world", "hello world"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        let logic_id = engine.compile(&json!({"in": ["foo", "hello world"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_array_operations_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Empty array map
        let logic_id = engine.compile(&json!({"map": [[], {"*": [{"var": ""}, 2]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([]));

        // Filter on empty array
        let logic_id = engine.compile(&json!({"filter": [[], {">": [{"var": ""}, 0]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([]));

        // Reduce on empty array
        let logic_id = engine.compile(&json!({"reduce": [[], {"+": [{"var": "accumulator"}, {"var": "current"}]}, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0));

        // Quantifiers on empty array
        let logic_id = engine.compile(&json!({"all": [[], {">": [{"var": ""}, 0]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true)); // vacuously true

        let logic_id = engine.compile(&json!({"some": [[], {">": [{"var": ""}, 0]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));

        let logic_id = engine.compile(&json!({"none": [[], {">": [{"var": ""}, 0]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_array_operations_with_objects() {
        let mut engine = RLogic::new();
        let data = json!({"users": [
            {"name": "Alice", "age": 30, "active": true},
            {"name": "Bob", "age": 25, "active": false},
            {"name": "Charlie", "age": 35, "active": true}
        ]});

        // Map to get names
        let logic_id = engine.compile(&json!({"map": [{"var": "users"}, {"var": "name"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(["Alice", "Bob", "Charlie"]));

        // Filter active users
        let logic_id = engine.compile(&json!({"filter": [{"var": "users"}, {"var": "active"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([
            {"name": "Alice", "age": 30, "active": true},
            {"name": "Charlie", "age": 35, "active": true}
        ]));

        // Map ages and sum them
        let logic_id = engine.compile(&json!({"reduce": [
            {"map": [{"var": "users"}, {"var": "age"}]},
            {"+": [{"var": "accumulator"}, {"var": "current"}]},
            0
        ]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(90)); // 30 + 25 + 35
    }

    #[test]
    fn test_array_operations_errors() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Map with non-array
        let logic_id = engine.compile(&json!({"map": ["not_array", {"var": ""}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([]));

        // Filter with non-array
        let logic_id = engine.compile(&json!({"filter": ["not_array", {"var": ""}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([]));

        // Reduce with non-array
        let logic_id = engine.compile(&json!({"reduce": ["not_array", {"var": ""}, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0));
    }

    #[test]
    fn test_array_sum() {
        let mut engine = RLogic::new();
        let data = json!({"values": [1, 2, 3, 4, 5]});

        // Sum array values
        let logic_id = engine.compile(&json!({"sum": [{"var": "values"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(15));

        // Sum object field values
        let data = json!({"items": [
            {"value": 10},
            {"value": 20},
            {"value": 30}
        ]});
        let logic_id = engine.compile(&json!({"sum": [{"var": "items"}, "value"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(60));
    }

    #[test]
    fn test_array_for_loop() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Simple for loop
        let logic_id = engine.compile(&json!({"FOR": [0, 3, {"+": [{"var": "$loopIteration"}, 1]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([1, 2, 3]));

        // For loop with complex logic
        let logic_id = engine.compile(&json!({"FOR": [1, 4, {"*": [{"var": "$loopIteration"}, 2]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([2, 4, 6]));
    }

    #[test]
    fn test_real_world_table_processing() {
        let mut engine = RLogic::new();
        
        // Real-world scenario: Process insurance rider premium table
        let data = json!({
            "riders": [
                {"code": "RIDER_A", "premium": 100000, "loading": 1.05},
                {"code": "RIDER_B", "premium": 200000, "loading": 1.10},
                {"code": "RIDER_C", "premium": 150000, "loading": 1.08}
            ],
            "frequency": 12,
            "discount_rate": 0.95
        });

        // Calculate total annual premium with loading and discount
        let logic_id = engine.compile(&json!({
            "*": [
                {"sum": [
                    {"map": [
                        {"var": "riders"},
                        {"*": [{"var": "premium"}, {"var": "loading"}]}
                    ]}
                ]},
                {"var": "frequency"},
                {"var": "discount_rate"}
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        
        // (100000*1.05 + 200000*1.10 + 150000*1.08) * 12 * 0.95
        let expected = (105000.0 + 220000.0 + 162000.0) * 12.0 * 0.95;
        assert!((result.as_f64().unwrap() - expected).abs() < 0.01);
    }

    #[test]
    fn test_nested_array_operations() {
        let mut engine = RLogic::new();
        let data = json!({
            "products": [
                {
                    "name": "Product A",
                    "variants": [{"price": 100}, {"price": 150}]
                },
                {
                    "name": "Product B",
                    "variants": [{"price": 200}, {"price": 250}, {"price": 300}]
                }
            ]
        });

        // Get all prices from all products
        // Note: merge flattens one level, so nested maps create nested arrays
        let logic_id = engine.compile(&json!({
            "map": [
                {"var": "products"},
                {"map": [{"var": "variants"}, {"var": "price"}]}
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        // Result is array of arrays: [[100, 150], [200, 250, 300]]
        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], json!([100, 150]));
        assert_eq!(arr[1], json!([200, 250, 300]));
    }

    #[test]
    fn test_array_operations_with_nulls() {
        let mut engine = RLogic::new();
        let data = json!({
            "values": [10, null, 20, null, 30]
        });

        // Sum should skip nulls
        let logic_id = engine.compile(&json!({"sum": [{"var": "values"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(60));

        // Filter out nulls
        let logic_id = engine.compile(&json!({
            "filter": [
                {"var": "values"},
                {"!=": [{"var": ""}, null]}
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([10, 20, 30]));

        // Map preserves nulls
        let logic_id = engine.compile(&json!({
            "map": [
                {"var": "values"},
                {"*": [{"var": ""}, 2]}
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!([20, 0, 40, 0, 60]));
    }

    #[test]
    fn test_array_conditional_operations() {
        let mut engine = RLogic::new();
        let data = json!({
            "transactions": [
                {"amount": 1000, "type": "credit"},
                {"amount": 500, "type": "debit"},
                {"amount": 2000, "type": "credit"},
                {"amount": 300, "type": "debit"}
            ]
        });

        // Calculate net balance (credits positive, debits negative)
        let logic_id = engine.compile(&json!({
            "reduce": [
                {"map": [
                    {"var": "transactions"},
                    {"if": [
                        {"==": [{"var": "type"}, "credit"]},
                        {"var": "amount"},
                        {"-": [0, {"var": "amount"}]}
                    ]}
                ]},
                {"+": [{"var": "accumulator"}, {"var": "current"}]},
                0
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(2200)); // 1000 + 2000 - 500 - 300
    }

    #[test]
    fn test_array_with_large_dataset() {
        let mut engine = RLogic::new();
        
        // Simulate large dataset processing
        let mut items = Vec::new();
        for i in 0..1000 {
            items.push(json!({
                "id": i,
                "value": i * 10,
                "category": if i % 2 == 0 { "even" } else { "odd" }
            }));
        }
        let data = json!({"items": items});

        // Filter even category and sum values
        let logic_id = engine.compile(&json!({
            "sum": [
                {"map": [
                    {"filter": [
                        {"var": "items"},
                        {"==": [{"var": "category"}, "even"]}
                    ]},
                    {"var": "value"}
                ]}
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        
        // Sum of even indices: 0*10 + 2*10 + 4*10 + ... + 998*10
        // = 10 * (0 + 2 + 4 + ... + 998) = 10 * (2 * (0+1+...+499)) = 10 * 2 * (499*500/2)
        let expected = 10.0 * (0..500).sum::<i32>() as f64 * 2.0;
        assert_eq!(result.as_f64().unwrap(), expected);
    }

    #[test]
    fn test_array_grouping_simulation() {
        let mut engine = RLogic::new();
        let data = json!({
            "sales": [
                {"product": "A", "amount": 100},
                {"product": "B", "amount": 200},
                {"product": "A", "amount": 150},
                {"product": "B", "amount": 300}
            ]
        });

        // Sum sales for product A
        let logic_id = engine.compile(&json!({
            "sum": [
                {"map": [
                    {"filter": [
                        {"var": "sales"},
                        {"==": [{"var": "product"}, "A"]}
                    ]},
                    {"var": "amount"}
                ]}
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(250));

        // Count product B entries
        let logic_id = engine.compile(&json!({
            "length": {"filter": [
                {"var": "sales"},
                {"==": [{"var": "product"}, "B"]}
            ]}
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(2));
    }

    #[test]
    fn test_array_edge_case_empty_operations() {
        let mut engine = RLogic::new();
        let data = json!({
            "empty": [],
            "single": [42],
            "mixed": [1, null, "", 0, false]
        });

        // Operations on empty arrays
        let test_cases = vec![
            (json!({"sum": [{"var": "empty"}]}), json!(0)),
            (json!({"length": {"var": "empty"}}), json!(0)),
            (json!({"merge": [{"var": "empty"}, {"var": "empty"}]}), json!([])),
        ];

        for (logic, expected) in test_cases {
            let logic_id = engine.compile(&logic).unwrap();
            let result = engine.run(&logic_id, &data).unwrap();
            assert_eq!(result, expected, "Failed for logic: {:?}", logic);
        }

        // Single element operations
        let logic_id = engine.compile(&json!({"sum": [{"var": "single"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(42));
    }
}
