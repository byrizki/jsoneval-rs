use super::*;
use serde_json::json;

#[cfg(test)]
mod ref_operator_tests {
    use super::*;
    
    #[test]
    fn test_ref_simple_field() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"name": "John", "age": 30}));
        
        // Simple field access
        let logic_id = engine.compile(&json!({"$ref": "name"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("John"));
    }
    
    #[test]
    fn test_ref_with_hash_slash() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"user": {"name": "Alice"}}));
        
        // #/user/name should resolve to user.name
        let logic_id = engine.compile(&json!({"$ref": "#/user/name"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("Alice"));
    }
    
    #[test]
    fn test_ref_with_properties() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "user": {
                "email": "test@example.com"
            }
        }));
        
        // #/properties/user/properties/email should resolve to user.email
        let logic_id = engine.compile(&json!({"$ref": "#/properties/user/properties/email"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("test@example.com"));
    }
    
    #[test]
    fn test_ref_slash_properties() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"firstName": "Bob"}));
        
        // /properties/firstName should resolve to firstName
        let logic_id = engine.compile(&json!({"$ref": "/properties/firstName"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("Bob"));
    }
    
    #[test]
    fn test_ref_dot_properties() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "address": {
                "city": "NYC"
            }
        }));
        
        // properties.address.properties.city should resolve to address.city
        let logic_id = engine.compile(&json!({"$ref": "properties.address.properties.city"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("NYC"));
    }
    
    #[test]
    fn test_ref_nested_path() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "user": {
                "profile": {
                    "settings": {
                        "theme": "dark"
                    }
                }
            }
        }));
        
        // #/user/profile/settings/theme
        let logic_id = engine.compile(&json!({"$ref": "#/user/profile/settings/theme"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("dark"));
    }
    
    #[test]
    fn test_ref_with_default() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"name": "John"}));
        
        // Missing field with default
        let logic_id = engine.compile(&json!({"$ref": ["age", 25]})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(25.0));
    }
    
    #[test]
    fn test_ref_missing_field() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"name": "John"}));
        
        // Missing field without default returns null
        let logic_id = engine.compile(&json!({"$ref": "missing"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(null));
    }
    
    #[test]
    fn test_ref_array_index() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "items": ["apple", "banana", "orange"]
        }));
        
        // Access array element
        let logic_id = engine.compile(&json!({"$ref": "items.1"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("banana"));
    }
    
    #[test]
    fn test_ref_complex_schema_path() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "order": {
                "customer": {
                    "billing": {
                        "zipCode": "10001"
                    }
                }
            }
        }));
        
        // Complex JSON Schema path
        let logic_id = engine.compile(&json!({
            "$ref": "#/properties/order/properties/customer/properties/billing/properties/zipCode"
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("10001"));
    }
    
    #[test]
    fn test_ref_in_condition() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "user": {
                "age": 25,
                "premium": true
            }
        }));
        
        // Use $ref in conditional logic
        let logic_id = engine.compile(&json!({
            "if": [
                {"and": [
                    {"$ref": "#/user/premium"},
                    {">": [{"$ref": "#/user/age"}, 18]}
                ]},
                "eligible",
                "not eligible"
            ]
        })).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("eligible"));
    }
    
    #[test]
    fn test_ref_vs_var() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"field": "value"}));
        
        // Both should work the same for simple paths
        let ref_logic = engine.compile(&json!({"$ref": "field"})).unwrap();
        let var_logic = engine.compile(&json!({"var": "field"})).unwrap();
        
        let ref_result = engine.evaluate(&ref_logic, &data).unwrap();
        let var_result = engine.evaluate(&var_logic, &data).unwrap();
        
        assert_eq!(ref_result, var_result);
    }
    
    #[test]
    fn test_ref_referenced_vars() {
        let mut engine = RLogic::new();
        
        let logic_id = engine.compile(&json!({
            "if": [
                {"$ref": "#/properties/user/properties/active"},
                {"$ref": "/properties/user/properties/name"},
                "inactive"
            ]
        })).unwrap();
        
        let vars = engine.get_referenced_vars(&logic_id).unwrap();
        assert!(vars.contains(&"user.active".to_string()));
        assert!(vars.contains(&"user.name".to_string()));
    }
    
    #[test]
    fn test_ref_with_leading_slash() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"status": "active"}));
        
        // /status should work
        let logic_id = engine.compile(&json!({"$ref": "/status"})).unwrap();
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!("active"));
    }
    
    #[test]
    fn test_ref_operator_alias() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({"value": 42}));
        
        // Both "$ref" and "ref" should work
        let logic_id1 = engine.compile(&json!({"$ref": "value"})).unwrap();
        let logic_id2 = engine.compile(&json!({"ref": "value"})).unwrap();
        
        let result1 = engine.evaluate(&logic_id1, &data).unwrap();
        let result2 = engine.evaluate(&logic_id2, &data).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(*result1, json!(42));
    }
    
    #[test]
    fn test_ref_normalization_edge_cases() {
        let mut engine = RLogic::new();
        let data = TrackedData::new(json!({
            "field": "test"
        }));
        
        // All these should resolve to "field"
        let paths = vec![
            "field",
            "/field",
            "#/field",
            "properties.field",
            "/properties/field",
            "#/properties/field",
        ];
        
        for path in paths {
            let logic_id = engine.compile(&json!({"$ref": path})).unwrap();
            let result = engine.evaluate(&logic_id, &data).unwrap();
            assert_eq!(*result, json!("test"), "Failed for path: {}", path);
        }
    }
}
