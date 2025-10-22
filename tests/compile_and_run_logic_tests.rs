use json_eval_rs::{JSONEval, ParsedSchema, ParsedSchemaCache};
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_compile_and_run_logic_with_object() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string"
            },
            "age": {
                "type": "number"
            }
        }
    }).to_string();

    let data = json!({"name": "Alice", "age": 30}).to_string();
    
    let mut eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    // Test with a logic object (not a string)
    let logic = json!({"==": [{"var": "age"}, 30]}).to_string();
    
    let result = eval.compile_and_run_logic(&logic, None)
        .expect("Failed to compile and run logic");

    assert_eq!(result, json!(true));
}

#[test]
fn test_compile_and_run_logic_with_parsed_schema_cache() {
    let schema = json!({
        "type": "object",
        "properties": {
            "score": {
                "type": "number"
            },
            "passed": {
                "type": "boolean"
            }
        }
    }).to_string();

    // Parse schema and add to cache
    let parsed = ParsedSchema::parse(&schema)
        .expect("Failed to parse schema");
    
    let cache = ParsedSchemaCache::new();
    cache.insert("test-schema".to_string(), Arc::new(parsed));

    // Create JSONEval from cached schema
    let cached = cache.get("test-schema")
        .expect("Schema not found in cache");
    
    let data = json!({"score": 85, "passed": false}).to_string();
    let mut eval = JSONEval::with_parsed_schema(cached, None, Some(&data))
        .expect("Failed to create JSONEval from cached schema");

    // Test compile_and_run_logic with a logic object
    let logic = json!({">=": [{"var": "score"}, 80]}).to_string();
    
    let result = eval.compile_and_run_logic(&logic, None)
        .expect("Failed to compile and run logic with cached schema");

    assert_eq!(result, json!(true));
}

#[test]
fn test_compile_and_run_logic_with_complex_object() {
    let schema = json!({
        "type": "object",
        "properties": {
            "values": {
                "type": "array"
            }
        }
    }).to_string();

    let data = json!({"values": [1, 2, 3, 4, 5]}).to_string();
    
    let mut eval = JSONEval::new(&schema, None, Some(&data))
        .expect("Failed to create JSONEval");

    // Test with nested logic object
    let logic = json!({
        "reduce": [
            {"var": "values"},
            {"+": [{"var": "current"}, {"var": "accumulator"}]},
            0
        ]
    }).to_string();
    
    let result = eval.compile_and_run_logic(&logic, None)
        .expect("Failed to compile and run complex logic");

    assert_eq!(result, json!(15));
}

#[test]
fn test_compile_and_run_logic_with_custom_data() {
    let schema = json!({
        "type": "object",
        "properties": {
            "x": {"type": "number"}
        }
    }).to_string();

    let initial_data = json!({"x": 10}).to_string();
    
    let mut eval = JSONEval::new(&schema, None, Some(&initial_data))
        .expect("Failed to create JSONEval");

    // Use different data than what was initialized
    let custom_data = json!({"y": 20}).to_string();
    let logic = json!({"*": [{"var": "y"}, 2]}).to_string();
    
    let result = eval.compile_and_run_logic(&logic, Some(&custom_data))
        .expect("Failed to compile and run logic with custom data");

    assert_eq!(result, json!(40));
}

#[test]
fn test_compile_and_run_logic_from_global_cache() {
    use json_eval_rs::PARSED_SCHEMA_CACHE;

    let schema = json!({
        "type": "object",
        "properties": {
            "status": {"type": "string"}
        }
    }).to_string();

    // Parse schema and add to global cache
    let parsed = ParsedSchema::parse(&schema)
        .expect("Failed to parse schema");
    
    PARSED_SCHEMA_CACHE.insert("global-test-schema".to_string(), Arc::new(parsed));

    // Create JSONEval from global cache
    let cached = PARSED_SCHEMA_CACHE.get("global-test-schema")
        .expect("Schema not found in global cache");
    
    let data = json!({"status": "active"}).to_string();
    let mut eval = JSONEval::with_parsed_schema(cached, None, Some(&data))
        .expect("Failed to create JSONEval from global cache");

    // Test compile_and_run_logic
    let logic = json!({"==": [{"var": "status"}, "active"]}).to_string();
    
    let result = eval.compile_and_run_logic(&logic, None)
        .expect("Failed to compile and run logic from global cache");

    assert_eq!(result, json!(true));

    // Cleanup
    PARSED_SCHEMA_CACHE.remove("global-test-schema");
}
