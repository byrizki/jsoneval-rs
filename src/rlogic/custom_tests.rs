use super::*;
use serde_json::json;

#[cfg(test)]
mod custom_operator_tests {
    use super::*;
    
    // Math operators
    #[test]
    fn test_abs() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"val": -5}));
        
        let logic_id = engine.compile(&json!({"abs": {"var": "val"}})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(5.0));
    }
    
    #[test]
    fn test_max() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"max": [1, 5, 3, 9, 2]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(9.0));
    }
    
    #[test]
    fn test_min() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"min": [1, 5, 3, 9, 2]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(1.0));
    }
    
    #[test]
    fn test_pow() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"pow": [2, 3]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(8.0));
    }
    
    #[test]
    fn test_round() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"round": 3.7})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(4.0));
    }
    
    #[test]
    fn test_roundup() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"ROUNDUP": 3.2})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(4.0));
    }
    
    #[test]
    fn test_rounddown() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"ROUNDDOWN": 3.9})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(3.0));
    }
    
    // String operators
    #[test]
    fn test_length() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"arr": [1, 2, 3]}));
        
        let logic_id = engine.compile(&json!({"length": {"var": "arr"}})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(3.0));
    }
    
    #[test]
    fn test_len() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"text": "Hello"}));
        
        let logic_id = engine.compile(&json!({"LEN": {"var": "text"}})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(5.0));
    }
    
    #[test]
    fn test_search() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"SEARCH": ["world", "Hello World"]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(7.0)); // 1-indexed position
    }
    
    #[test]
    fn test_left() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"LEFT": ["Hello World", 5]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("Hello"));
    }
    
    #[test]
    fn test_right() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"RIGHT": ["Hello World", 5]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("World"));
    }
    
    #[test]
    fn test_splittext() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"SPLITTEXT": ["a,b,c", ",", 1]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("b"));
    }
    
    #[test]
    fn test_concat() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"first": "John", "last": "Doe"}));
        
        let logic_id = engine.compile(&json!({"CONCAT": [{"var": "first"}, " ", {"var": "last"}]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("John Doe"));
    }
    
    #[test]
    fn test_splitvalue() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"SPLITVALUE": ["a,b,c", ","]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(["a", "b", "c"]));
    }
    
    // Logical operators
    #[test]
    fn test_xor() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"xor": [true, false]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(true));
        
        let logic_id = engine.compile(&json!({"xor": [true, true]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(false));
    }
    
    #[test]
    fn test_ifnull() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"val": null}));
        
        let logic_id = engine.compile(&json!({"IFNULL": [{"var": "val"}, "default"]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("default"));
    }
    
    #[test]
    fn test_isempty() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"empty": "", "full": "text"}));
        
        let logic_id = engine.compile(&json!({"ISEMPTY": {"var": "empty"}})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(true));
        
        let logic_id = engine.compile(&json!({"ISEMPTY": {"var": "full"}})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(false));
    }
    
    #[test]
    fn test_empty() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"EMPTY": []})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(""));
    }
    
    // Date operators
    #[test]
    fn test_today() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"TODAY": []})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        // Just verify it returns a string
        assert!(result.is_string());
    }
    
    #[test]
    fn test_now() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"NOW": []})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        // Just verify it returns a string
        assert!(result.is_string());
    }
    
    #[test]
    fn test_days() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"DAYS": ["2024-01-10", "2024-01-01"]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(9.0));
    }
    
    #[test]
    fn test_year() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"YEAR": "2024-03-15"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(2024.0));
    }
    
    #[test]
    fn test_month() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"MONTH": "2024-03-15"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(3.0));
    }
    
    #[test]
    fn test_day() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"DAY": "2024-03-15"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(15.0));
    }
    
    #[test]
    fn test_date() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({}));
        
        let logic_id = engine.compile(&json!({"DATE": [2024, 3, 15]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert!(result.as_str().unwrap().contains("2024-03-15"));
    }
    
    // Array/Table operators
    #[test]
    fn test_sum_array() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"numbers": [1, 2, 3, 4, 5]}));
        
        let logic_id = engine.compile(&json!({"SUM": {"var": "numbers"}})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(15.0));
    }
    
    #[test]
    fn test_sum_with_field() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "items": [
                {"price": 10},
                {"price": 20},
                {"price": 30}
            ]
        }));
        
        let logic_id = engine.compile(&json!({"SUM": [{"var": "items"}, "price"]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(60.0));
    }
}
