
#[cfg(test)]
mod integration_tests {
    use serde_json::json;
    use json_eval_rs::rlogic::{RLogic, TrackedData};
    
    #[test]
    fn test_full_workflow() {
        let mut engine = RLogic::new();
        
        // Compile logic
        let logic_id = engine.compile(&json!({
            "if": [
                {">": [{"var": "score"}, 90]},
                "A",
                {
                    "if": [
                        {">": [{"var": "score"}, 80]},
                        "B",
                        "C"
                    ]
                }
            ]
        })).unwrap();
        
        // Test with different data
        let data1 = TrackedData::new(json!({"score": 95}));
        let result1 = engine.evaluate(&logic_id, &data1).unwrap();
        assert_eq!(*result1, json!("A"));
        
        let data2 = TrackedData::new(json!({"score": 85}));
        let result2 = engine.evaluate(&logic_id, &data2).unwrap();
        assert_eq!(*result2, json!("B"));
        
        let data3 = TrackedData::new(json!({"score": 75}));
        let result3 = engine.evaluate(&logic_id, &data3).unwrap();
        assert_eq!(*result3, json!("C"));
    }
}
