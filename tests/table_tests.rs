use json_eval_rs::*;
use serde_json::json;

/// Table operation tests - VALUEAT, INDEXAT, MATCH, etc.
#[cfg(test)]
mod table_tests {
    use super::*;

    #[test]
    fn test_valueat_basic() {
        let mut engine = RLogic::new();
        let data = json!({
            "table": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25},
                {"name": "Charlie", "age": 35}
            ]
        });

        // Get entire row
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, 1]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!({"name": "Bob", "age": 25}));

        // Get specific column
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, 1, "name"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("Bob"));

        // Get age column
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, 2, "age"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(35));
    }

    #[test]
    fn test_valueat_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({
            "table": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        });

        // Out of bounds (negative)
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, -1]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Out of bounds (too large)
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Empty table
        let data = json!({"table": []});
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Non-existent column
        let data = json!({"table": [{"name": "Alice"}]});
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, 0, "missing"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_maxat() {
        let mut engine = RLogic::new();
        let data = json!({
            "table": [
                {"name": "Alice", "score": 85},
                {"name": "Bob", "score": 92},
                {"name": "Charlie", "score": 78}
            ]
        });

        // MAXAT returns column value from last row (assumes sorted data)
        // JS: tableData[tableData.length - 1][colName]
        let logic_id = engine.compile(&json!({"MAXAT": [{"var": "table"}, "score"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(78)); // Charlie's score (last row)
    }

    #[test]
    fn test_indexat() {
        let mut engine = RLogic::new();
        let data = json!({
            "table": [
                {"id": 100, "name": "Alice"},
                {"id": 200, "name": "Bob"},
                {"id": 300, "name": "Charlie"}
            ]
        });

        // Find index by exact match
        let logic_id = engine.compile(&json!({"INDEXAT": [200, {"var": "table"}, "id"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1));

        // Find index by range (first where field <= value)
        // JS: table?.findIndex((x) => x[field] <= look)
        let logic_id = engine.compile(&json!({"INDEXAT": [250, {"var": "table"}, "id", true]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0)); // First item with id <= 250 is Alice (100)

        // Range with value smaller than all items
        let logic_id = engine.compile(&json!({"INDEXAT": [50, {"var": "table"}, "id", true]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(-1)); // No item with id <= 50

        // Not found (exact match)
        let logic_id = engine.compile(&json!({"INDEXAT": [999, {"var": "table"}, "id"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(-1));
    }

    #[test]
    fn test_match() {
        let mut engine = RLogic::new();
        let data = json!({
            "table": [
                {"name": "Alice", "age": 30, "city": "NYC"},
                {"name": "Bob", "age": 25, "city": "LA"},
                {"name": "Charlie", "age": 35, "city": "NYC"}
            ]
        });

        // Match single condition
        let logic_id = engine.compile(&json!({"MATCH": [{"var": "table"}, "Alice", "name"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0));

        // Match multiple conditions
        let logic_id = engine.compile(&json!({"MATCH": [{"var": "table"}, "Alice", "name", "NYC", "city"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0));

        // Match not found
        let logic_id = engine.compile(&json!({"MATCH": [{"var": "table"}, "David", "name"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(-1));
    }

    #[test]
    fn test_match_range() {
        let mut engine = RLogic::new();
        let data = json!({
            "rates": [
                {"min_age": 0, "max_age": 25, "rate": 0.05},
                {"min_age": 26, "max_age": 40, "rate": 0.07},
                {"min_age": 41, "max_age": 60, "rate": 0.09}
            ]
        });

        // Find rate for age 30 (should match 26-40 range)
        let logic_id = engine.compile(&json!({"MATCHRANGE": [{"var": "rates"}, "min_age", "max_age", 30]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1)); // Index 1 has the matching range

        // Find rate for age 50 (should match 41-60 range)
        let logic_id = engine.compile(&json!({"MATCHRANGE": [{"var": "rates"}, "min_age", "max_age", 50]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(2));
    }

    #[test]
    fn test_choose() {
        let mut engine = RLogic::new();
        let data = json!({
            "products": [
                {"name": "Widget", "category": "A", "price": 10},
                {"name": "Gadget", "category": "B", "price": 20},
                {"name": "Tool", "category": "A", "price": 15}
            ]
        });

        // Choose any item in category A
        let logic_id = engine.compile(&json!({"CHOOSE": [{"var": "products"}, "A", "category"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        // Should return index of first matching item (could be 0 or 2)
        let index = result.as_f64().unwrap();
        assert!(index == 0.0 || index == 2.0);

        // Get the actual product at that index
        let products = data["products"].as_array().unwrap();
        let product = &products[index as usize];
        assert_eq!(product["category"], "A");
    }

    #[test]
    fn test_find_index() {
        let mut engine = RLogic::new();
        let data = json!({
            "items": [
                {"value": 10, "active": true},
                {"value": 20, "active": false},
                {"value": 30, "active": true}
            ]
        });

        // Find index of first active item with value > 15
        // Note: Multiple conditions are ANDed together automatically
        // Format: [op, compare_value, col_name] means row[col_name] op compare_value
        let logic_id = engine.compile(&json!({"FINDINDEX": [{"var": "items"},
            "active", [">", 15, "value"]
        ]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        println!("Result with active AND value>15: {}", result);
        println!("Expected: 2 (item 2 has active=true AND value=30 > 15)");
        println!("Item 0: active={}, value={}", data["items"][0]["active"], data["items"][0]["value"]);
        println!("Item 1: active={}, value={}", data["items"][1]["active"], data["items"][1]["value"]);
        println!("Item 2: active={}, value={}", data["items"][2]["active"], data["items"][2]["value"]);
        assert_eq!(result, json!(2)); // Index 2 has value 30 and is active

        // No match found
        let logic_id = engine.compile(&json!({"FINDINDEX": [{"var": "items"},
            {">": [{"var": "value"}, 50]}
        ]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(-1));
    }

    #[test]
    fn test_table_operations_with_references() {
        let mut engine = RLogic::new();
        let data = json!({
            "$params": {
                "mortality_table": [
                    {"age": 30, "qx": 0.001},
                    {"age": 35, "qx": 0.002},
                    {"age": 40, "qx": 0.003}
                ]
            },
            "current_age": 35
        });

        // Use $ref to access table and find by current_age
        let logic_id = engine.compile(&json!({"VALUEAT": [
            {"$ref": "$params.mortality_table"},
            {"INDEXAT": [{"var": "current_age"}, {"$ref": "$params.mortality_table"}, "age"]},
            "qx"
        ]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.002));
    }

    #[test]
    fn test_table_operations_performance() {
        let mut engine = RLogic::new();

        // Create a larger table for performance testing
        let mut table = Vec::new();
        for i in 0..100 {
            table.push(json!({"id": i, "value": i * 2, "active": i % 2 == 0}));
        }
        let data = json!({"table": table});

        // Test VALUEAT on large table
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, 50, "value"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(100));

        // Test INDEXAT on large table
        let logic_id = engine.compile(&json!({"INDEXAT": [75, {"var": "table"}, "id"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(75));

        // Test MATCH on large table
        let logic_id = engine.compile(&json!({"MATCH": [{"var": "table"}, true, "active"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0)); // First even index is active
    }

    #[test]
    fn test_table_operations_errors() {
        let mut engine = RLogic::new();
        let data = json!({"not_array": "string"});

        // VALUEAT with non-array table
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "not_array"}, 0]})).unwrap();
        let result = engine.run(&logic_id, &data);
        // Should return error or null for non-array
        assert!(result.is_err() || result.unwrap().is_null());

        // INDEXAT with non-array table
        let logic_id = engine.compile(&json!({"INDEXAT": [1, {"var": "not_array"}, "field"]})).unwrap();
        let result = engine.run(&logic_id, &data);
        // Should return error or -1.0 for non-array
        assert!(result.is_err() || result.unwrap() == json!(-1.0));

        // MATCH with non-array table
        let logic_id = engine.compile(&json!({"MATCH": [{"var": "not_array"}, "value", "field"]})).unwrap();
        let result = engine.run(&logic_id, &data);
        // Should return error or -1.0 for non-array
        assert!(result.is_err() || result.unwrap() == json!(-1.0));
    }

    #[test]
    fn test_findindex_preprocessing() {
        // Test that the preprocessing converts array shorthand correctly
        let mut engine = RLogic::new();
        let data = json!({"items": [{"x": 10}, {"x": 20}]});
        
        // Simple comparison with number: ["==", value, col]
        let logic_id = engine.compile(&json!({
            "FINDINDEX": [{"var": "items"}, ["==", 20, "x"]]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        println!("Test 1 (number) result: {}", result);
        assert_eq!(result, json!(1)); // Second item has x=20
        
        // Test with string values
        let data2 = json!({"items": [{"name": "Alice"}, {"name": "Bob"}]});
        let logic_id = engine.compile(&json!({
            "FINDINDEX": [{"var": "items"}, ["==", "Bob", "name"]]
        })).unwrap();
        let result = engine.run(&logic_id, &data2).unwrap();
        println!("Test 2 (string) result: {}", result);
        assert_eq!(result, json!(1)); // Bob is at index 1
    }

    #[test]
    fn test_findindex_with_and() {
        let mut engine = RLogic::new();
        let data = json!({
            "employees": [
                {"id": 1, "name": "Alice", "salary": 50000, "department": "Engineering"},
                {"id": 2, "name": "Bob", "salary": 55000, "department": "Sales"},
                {"id": 3, "name": "Charlie", "salary": 60000, "department": "Engineering"},
                {"id": 4, "name": "Diana", "salary": 52000, "department": "Sales"}
            ]
        });

        // Test FINDINDEX with single condition using array shorthand
        // Format: [op, value, col] where col is auto-converted to {"var": col}
        let logic_id = engine.compile(&json!({
            "FINDINDEX": [
                {"var": "employees"},
                ["==", "Engineering", "department"]
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0)); // Alice is first engineer at index 0
        
        // Test FINDINDEX with multiple conditions using array shorthand with &&
        let logic_id = engine.compile(&json!({
            "FINDINDEX": [
                {"var": "employees"},
                ["&&", 
                    ["==", "Engineering", "department"],
                    [">", 55000, "salary"]
                ]
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(2)); // Charlie is at index 2
    }

    #[test]
    fn test_combined_table_operations() {
        let mut engine = RLogic::new();
        let data = json!({
            "employees": [
                {"id": 1, "name": "Alice", "salary": 50000, "department": "Engineering"},
                {"id": 2, "name": "Bob", "salary": 55000, "department": "Sales"},
                {"id": 3, "name": "Charlie", "salary": 60000, "department": "Engineering"},
                {"id": 4, "name": "Diana", "salary": 52000, "department": "Sales"}
            ]
        });

        // Find first engineering employee with salary > 55000 using array shorthand
        let logic_id = engine.compile(&json!({
            "VALUEAT": [
                {"var": "employees"},
                {"FINDINDEX": [
                    {"var": "employees"},
                    ["&&",
                        ["==", "Engineering", "department"],
                        [">", 55000, "salary"]
                    ]
                ]},
                "name"
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("Charlie")); // Charlie is in Engineering with salary > 55000

        // Test filter + map + sum combination
        let logic_id = engine.compile(&json!({
            "sum": [{"map": [
                {"filter": [
                    {"var": "employees"},
                    {"==": [{"var": "department"}, "Engineering"]}
                ]},
                {"var": "salary"}
            ]}]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(110000)); // 50000 + 60000
    }
}
