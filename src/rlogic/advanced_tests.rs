use super::*;
use serde_json::json;

#[cfg(test)]
mod advanced_operator_tests {
    use super::*;
    
    // Complex table operations
    #[test]
    fn test_valueat() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "table": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25},
                {"name": "Charlie", "age": 35}
            ]
        }));
        
        // Get entire row
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, 1]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!({"name": "Bob", "age": 25}));
        
        // Get specific column
        let logic_id = engine.compile(&json!({"VALUEAT": [{"var": "table"}, 1, "name"]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("Bob"));
    }
    
    #[test]
    fn test_maxat() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "table": [
                {"name": "Alice", "score": 85},
                {"name": "Bob", "score": 92},
                {"name": "Charlie", "score": 78}
            ]
        }));
        
        let logic_id = engine.compile(&json!({"MAXAT": [{"var": "table"}, "score"]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(78)); // Last row's score
    }
    
    #[test]
    fn test_indexat() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "table": [
                {"name": "Alice", "score": 85},
                {"name": "Bob", "score": 92},
                {"name": "Charlie", "score": 78}
            ]
        }));
        
        // Exact match
        let logic_id = engine.compile(&json!({"INDEXAT": [92, {"var": "table"}, "score"]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(1.0)); // Bob is at index 1
        
        // Range match (find last index where score <= 90)
        let logic_id = engine.compile(&json!({"INDEXAT": [90, {"var": "table"}, "score", true]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(2.0)); // Charlie at index 2 (score 78 <= 90)
    }
    
    #[test]
    fn test_match() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "table": [
                {"name": "Alice", "age": 30, "city": "NYC"},
                {"name": "Bob", "age": 25, "city": "LA"},
                {"name": "Charlie", "age": 30, "city": "NYC"}
            ]
        }));
        
        // Match age=30 and city=NYC
        let logic_id = engine.compile(&json!({
            "MATCH": [{"var": "table"}, 30, "age", "NYC", "city"]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(0.0)); // Alice at index 0
    }
    
    #[test]
    fn test_matchrange() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "table": [
                {"name": "Item A", "min_price": 10, "max_price": 20},
                {"name": "Item B", "min_price": 15, "max_price": 25},
                {"name": "Item C", "min_price": 20, "max_price": 30}
            ]
        }));
        
        // Find item where 18 is between min_price and max_price
        let logic_id = engine.compile(&json!({
            "MATCHRANGE": [{"var": "table"}, "min_price", "max_price", 18]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(0.0)); // Item A (10-20 contains 18)
    }
    
    #[test]
    fn test_choose() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "table": [
                {"name": "Alice", "age": 30, "city": "NYC"},
                {"name": "Bob", "age": 25, "city": "LA"},
                {"name": "Charlie", "age": 30, "city": "SF"}
            ]
        }));
        
        // Find first row where age=30 OR city=LA
        let logic_id = engine.compile(&json!({
            "CHOOSE": [{"var": "table"}, 30, "age", "LA", "city"]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(0.0)); // Alice matches age=30
    }
    
    #[test]
    fn test_findindex() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "table": [
                {"name": "Alice", "age": 30, "active": true},
                {"name": "Bob", "age": 25, "active": false},
                {"name": "Charlie", "age": 35, "active": true}
            ]
        }));
        
        // Find first row where age > 28 AND active = true
        let logic_id = engine.compile(&json!({
            "FINDINDEX": [
                {"var": "table"},
                {">": [{"var": "age"}, 28]},
                {"==": [{"var": "active"}, true]}
            ]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(0.0)); // Alice at index 0
    }
    
    // Array operations
    #[test]
    fn test_multiplies() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        // Simple multiplication
        let logic_id = engine.compile(&json!({"MULTIPLIES": [2, 3, 4]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(24.0));
        
        // With array flattening
        let logic_id = engine.compile(&json!({"MULTIPLIES": [[2, 3], 4]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(24.0));
    }
    
    #[test]
    fn test_divides() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        // Simple division
        let logic_id = engine.compile(&json!({"DIVIDES": [100, 2, 5]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(10.0));
        
        // With array flattening
        let logic_id = engine.compile(&json!({"DIVIDES": [[100], 2, 5]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(10.0));
    }
    
    // Advanced date functions
    #[test]
    fn test_yearfrac() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        // Default basis (30/360)
        let logic_id = engine.compile(&json!({
            "YEARFRAC": ["2024-01-01", "2024-12-31"]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert!(result.as_f64().unwrap() > 0.9); // Close to 1 year
        
        // Actual/365 basis
        let logic_id = engine.compile(&json!({
            "YEARFRAC": ["2024-01-01", "2024-12-31", 3]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert!(result.as_f64().unwrap() > 0.9);
    }
    
    #[test]
    fn test_datedif() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        // Days
        let logic_id = engine.compile(&json!({
            "DATEDIF": ["2024-01-01", "2024-01-10", "D"]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(9.0));
        
        // Months
        let logic_id = engine.compile(&json!({
            "DATEDIF": ["2024-01-01", "2024-04-01", "M"]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(3.0));
        
        // Years
        let logic_id = engine.compile(&json!({
            "DATEDIF": ["2022-01-01", "2024-01-01", "Y"]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(2.0));
    }
    
    // UI helpers
    #[test]
    fn test_rangeoptions() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"RANGEOPTIONS": [1, 5]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        
        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], json!({"label": "1", "value": "1"}));
        assert_eq!(arr[4], json!({"label": "5", "value": "5"}));
    }
    
    #[test]
    fn test_mapoptions() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"},
                {"id": 3, "name": "Charlie"}
            ]
        }));
        
        let logic_id = engine.compile(&json!({
            "MAPOPTIONS": [{"var": "users"}, "name", "id"]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        
        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], json!({"label": "Alice", "value": 1}));
        assert_eq!(arr[1], json!({"label": "Bob", "value": 2}));
    }
    
    #[test]
    fn test_mapoptionsif() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "users": [
                {"id": 1, "name": "Alice", "age": 30},
                {"id": 2, "name": "Bob", "age": 25},
                {"id": 3, "name": "Charlie", "age": 35}
            ]
        }));
        
        // Filter users where age >= 30
        let logic_id = engine.compile(&json!({
            "MAPOPTIONSIF": [{"var": "users"}, "name", "id", 30, "<=", "age"]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        
        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2); // Alice and Charlie
        assert_eq!(arr[0], json!({"label": "Alice", "value": 1}));
        assert_eq!(arr[1], json!({"label": "Charlie", "value": 3}));
    }
    
    #[test]
    fn test_return() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"value": 42}));
        
        let logic_id = engine.compile(&json!({"return": {"var": "value"}})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(42));
    }
    
    // Integration tests
    #[test]
    fn test_complex_table_query() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "products": [
                {"name": "Widget A", "price": 10, "stock": 50},
                {"name": "Widget B", "price": 15, "stock": 30},
                {"name": "Widget C", "price": 20, "stock": 0}
            ]
        }));
        
        // Find index of first product with stock > 0 and price < 20
        let logic_id = engine.compile(&json!({
            "FINDINDEX": [
                {"var": "products"},
                {">": [{"var": "stock"}, 0]},
                {"<": [{"var": "price"}, 20]}
            ]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(0.0)); // Widget A
        
        // Get that product's name
        let logic_id = engine.compile(&json!({
            "VALUEAT": [
                {"var": "products"},
                {"FINDINDEX": [
                    {"var": "products"},
                    {">": [{"var": "stock"}, 0]},
                    {"<": [{"var": "price"}, 20]}
                ]},
                "name"
            ]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("Widget A"));
    }
    
    #[test]
    fn test_date_calculations() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "start_date": "2024-01-01",
            "end_date": "2024-12-31"
        }));
        
        // Calculate days and convert to years
        let logic_id = engine.compile(&json!({
            "/": [
                {"DATEDIF": [{"var": "start_date"}, {"var": "end_date"}, "D"]},
                365
            ]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        let years = result.as_f64().unwrap();
        assert!(years > 0.9 && years < 1.1); // Close to 1 year
    }
    
    #[test]
    fn test_array_operations_with_sum() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "values": [2, 3, 4]
        }));
        
        // Multiply all values (result is 24)
        let logic_id = engine.compile(&json!({
            "MULTIPLIES": [{"var": "values"}]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(24.0)); // 2*3*4 = 24
    }
}
