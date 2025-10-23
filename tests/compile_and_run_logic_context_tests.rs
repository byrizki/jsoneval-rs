use json_eval_rs::{JSONEval, ParsedSchema, ParsedSchemaCache};
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_compile_and_run_logic_with_context_ref() {
    let schema = json!({
        "type": "object",
        "$params": {
            "accessList": ["AP", "BP", "CP"]
        },
        "properties": {
            "name": {
                "type": "string"
            }
        }
    }).to_string();

    let data = json!({}).to_string();
    let context = json!({
        "agentProfile": {
            "sob": "AP"
        }
    }).to_string();
    
    let mut eval = JSONEval::new(&schema, Some(&context), Some(&data))
        .expect("Failed to create JSONEval");

    // Evaluate first to setup eval_data properly
    eval.evaluate(&data, Some(&context))
        .expect("Failed to evaluate");

    // Test logic with $ref to $context
    let logic = json!({
        "in": [
            {"$ref": "$context.agentProfile.sob"},
            {"$ref": "#/$params/accessList"}
        ]
    }).to_string();
    
    let result = eval.compile_and_run_logic(&logic, None, None)
        .expect("Failed to compile and run logic with context ref");

    assert_eq!(result, json!(true), "Should find 'AP' in accessList");
}

#[test]
fn test_compile_and_run_logic_with_context_ref_from_cache() {
    let schema = json!({
        "type": "object",
        "$params": {
            "allowedRoles": ["admin", "editor", "viewer"]
        },
        "properties": {
            "userId": {
                "type": "string"
            }
        }
    }).to_string();

    // Parse schema and add to cache
    let parsed = ParsedSchema::parse(&schema)
        .expect("Failed to parse schema");
    
    let cache = ParsedSchemaCache::new();
    cache.insert("test-context-schema".to_string(), Arc::new(parsed));

    // Create JSONEval from cached schema
    let cached = cache.get("test-context-schema")
        .expect("Schema not found in cache");
    
    let data = json!({"userId": "user123"}).to_string();
    let context = json!({
        "userProfile": {
            "role": "admin"
        }
    }).to_string();
    
    let mut eval = JSONEval::with_parsed_schema(cached, Some(&context), Some(&data))
        .expect("Failed to create JSONEval from cached schema");

    // Evaluate first
    eval.evaluate(&data, Some(&context))
        .expect("Failed to evaluate");

    // Test logic with $ref to both $context and $params
    let logic = json!({
        "in": [
            {"$ref": "$context.userProfile.role"},
            {"$ref": "#/$params/allowedRoles"}
        ]
    }).to_string();
    
    let result = eval.compile_and_run_logic(&logic, None, None)
        .expect("Failed to compile and run logic from cached schema");

    assert_eq!(result, json!(true), "Should find 'admin' in allowedRoles");
}

#[test]
fn test_compile_and_run_logic_with_custom_data_and_context() {
    let schema = json!({
        "type": "object",
        "$params": {
            "threshold": 100
        },
        "properties": {
            "score": {"type": "number"}
        }
    }).to_string();

    let initial_data = json!({"score": 50}).to_string();
    let context = json!({
        "multiplier": 2
    }).to_string();
    
    let mut eval = JSONEval::new(&schema, Some(&context), Some(&initial_data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&initial_data, Some(&context))
        .expect("Failed to evaluate");

    // Use different data but context should still be available
    let custom_data = json!({"score": 75}).to_string();
    let logic = json!({
        ">=": [
            {"*": [{"$ref": "score"}, {"$ref": "$context.multiplier"}]},
            {"$ref": "#/$params/threshold"}
        ]
    }).to_string();
    
    let result = eval.compile_and_run_logic(&logic, Some(&custom_data), None)
        .expect("Failed to compile and run logic with custom data");

    // 75 * 2 = 150, which is >= 100
    assert_eq!(result, json!(true));
}

#[test]
fn test_compile_and_run_logic_nested_context_refs() {
    let schema = json!({
        "type": "object",
        "$params": {
            "config": {
                "maxAttempts": 3,
                "timeout": 30
            },
            "table_data": ["AP","AG"]
        },
        "properties": {}
    }).to_string();

    let data = json!({}).to_string();
    let context = json!({
        "agentProfile": {
            "sob": "AP"
        },
        "session": {
            "attempts": 2,
            "settings": {
                "autoRetry": true
            }
        }
    }).to_string();
    
    let mut eval = JSONEval::new(&schema, Some(&context), Some(&data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data, Some(&context))
        .expect("Failed to evaluate");


    let logic = json!({
        "in": [
            {"$ref": "$context.agentProfile.sob"},
            {"$ref": "#/$params/table_data"}
        ]
    }).to_string();

    let result = eval.compile_and_run_logic(&logic, None, None)
        .expect("Failed to compile and run logic with nested refs");

    assert_eq!(result, json!(true));

    // Test nested $ref access
    let logic = json!({
        "and": [
            {"<": [
                {"$ref": "$context.session.attempts"},
                {"$ref": "#/$params/config/maxAttempts"}
            ]},
            {"==": [
                {"$ref": "$context.session.settings.autoRetry"},
                true
            ]}
        ]
    }).to_string();
    
    let result = eval.compile_and_run_logic(&logic, None, None)
        .expect("Failed to compile and run logic with nested refs");

    assert_eq!(result, json!(true));
}

#[test]
fn test_compile_and_run_logic_missing_context_ref_returns_null() {
    let schema = json!({
        "type": "object",
        "properties": {}
    }).to_string();

    let data = json!({}).to_string();
    let context = json!({
        "existing": "value"
    }).to_string();
    
    let mut eval = JSONEval::new(&schema, Some(&context), Some(&data))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data, Some(&context))
        .expect("Failed to evaluate");

    // Reference non-existent path in context
    let logic = json!({"$ref": "$context.nonExistent.path"}).to_string();
    
    let result = eval.compile_and_run_logic(&logic, None, None)
        .expect("Failed to compile and run logic");

    assert_eq!(result, json!(null), "Non-existent ref should return null");
}
