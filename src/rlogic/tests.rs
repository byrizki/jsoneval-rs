use super::*;
use serde_json::json;

#[test]
fn test_dependency_based_caching() {
    let mut engine = RLogic::new();
    
    // Logic that depends only on "name"
    let logic_id = engine.compile(&json!({"var": "name"})).unwrap();
    
    // Initial evaluation
    let mut data = TrackedData::new(json!({"name": "John", "age": 30, "city": "NYC"}));
    let result1 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result1, json!("John"));
    
    let stats1 = engine.cache_stats();
    assert_eq!(stats1.misses, 1);
    assert_eq!(stats1.hits, 0);
    
    // Change unrelated field (age) - should hit cache because "name" hasn't changed
    data.set("age", json!(31));
    let result2 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result2, json!("John"));
    
    let stats2 = engine.cache_stats();
    assert_eq!(stats2.hits, 1); // Cache hit!
    assert_eq!(stats2.misses, 1);
    
    // Change another unrelated field (city) - should still hit cache
    data.set("city", json!("LA"));
    let result3 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result3, json!("John"));
    
    let stats3 = engine.cache_stats();
    assert_eq!(stats3.hits, 2); // Another cache hit!
    assert_eq!(stats3.misses, 1);
    
    // Change the dependent field (name) - should miss cache
    data.set("name", json!("Jane"));
    let result4 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result4, json!("Jane"));
    
    let stats4 = engine.cache_stats();
    assert_eq!(stats4.hits, 2);
    assert_eq!(stats4.misses, 2); // Cache miss because dependency changed
}

#[test]
fn test_multiple_dependencies_caching() {
    let mut engine = RLogic::new();
    
    // Logic that depends on both "x" and "y"
    let logic_id = engine.compile(&json!({"+": [{"var": "x"}, {"var": "y"}]})).unwrap();
    
    let mut data = TrackedData::new(json!({"x": 10, "y": 20, "z": 30}));
    
    // Initial evaluation
    let result1 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result1, json!(30.0));
    assert_eq!(engine.cache_stats().misses, 1);
    
    // Change unrelated field - cache hit
    data.set("z", json!(40));
    let result2 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result2, json!(30.0));
    assert_eq!(engine.cache_stats().hits, 1);
    
    // Change one dependency - cache miss
    data.set("x", json!(15));
    let result3 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result3, json!(35.0));
    assert_eq!(engine.cache_stats().misses, 2);
    
    // Change other dependency - cache miss
    data.set("y", json!(25));
    let result4 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result4, json!(40.0));
    assert_eq!(engine.cache_stats().misses, 3);
}

#[test]
fn test_compile_basic_types() {
    let mut engine = RLogic::new();
    
    // Test null
    let logic_id = engine.compile(&json!(null)).unwrap();
    assert!(engine.store.get(&logic_id).is_some());
    
    // Test boolean
    let logic_id = engine.compile(&json!(true)).unwrap();
    assert!(engine.store.get(&logic_id).is_some());
    
    // Test number
    let logic_id = engine.compile(&json!(42)).unwrap();
    assert!(engine.store.get(&logic_id).is_some());
    
    // Test string
    let logic_id = engine.compile(&json!("hello")).unwrap();
    assert!(engine.store.get(&logic_id).is_some());
}

#[test]
fn test_variable_access() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"name": "John", "age": 30}));
    
    let logic_id = engine.compile(&json!({"var": "name"})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!("John"));
    
    let logic_id = engine.compile(&json!({"var": "age"})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(30));
}

#[test]
fn test_variable_with_default() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"name": "John"}));
    
    let logic_id = engine.compile(&json!({"var": ["age", 25]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(25.0));
}

#[test]
fn test_nested_variable_access() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"user": {"name": "John", "address": {"city": "NYC"}}}));
    
    let logic_id = engine.compile(&json!({"var": "user.name"})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!("John"));
    
    let logic_id = engine.compile(&json!({"var": "user.address.city"})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!("NYC"));
}

#[test]
fn test_logical_and() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({}));
    
    let logic_id = engine.compile(&json!({"and": [true, true]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
    
    let logic_id = engine.compile(&json!({"and": [true, false]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(false));
}

#[test]
fn test_logical_or() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({}));
    
    let logic_id = engine.compile(&json!({"or": [false, true]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
    
    let logic_id = engine.compile(&json!({"or": [false, false]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(false));
}

#[test]
fn test_logical_not() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({}));
    
    let logic_id = engine.compile(&json!({"!": true})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(false));
    
    let logic_id = engine.compile(&json!({"!": false})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
}

#[test]
fn test_if_condition() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"age": 20}));
    
    let logic_id = engine.compile(&json!({
        "if": [
            {">": [{"var": "age"}, 18]},
            "adult",
            "minor"
        ]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!("adult"));
}

#[test]
fn test_comparison_operators() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({}));
    
    // Equal
    let logic_id = engine.compile(&json!({"==": [5, 5]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
    
    // Not equal
    let logic_id = engine.compile(&json!({"!=": [5, 3]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
    
    // Less than
    let logic_id = engine.compile(&json!({"<": [3, 5]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
    
    // Greater than
    let logic_id = engine.compile(&json!({">": [5, 3]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
}

#[test]
fn test_arithmetic_operators() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({}));
    
    // Addition
    let logic_id = engine.compile(&json!({"+": [2, 3, 5]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(10.0));
    
    // Subtraction
    let logic_id = engine.compile(&json!({"-": [10, 3]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(7.0));
    
    // Multiplication
    let logic_id = engine.compile(&json!({"*": [2, 3, 4]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(24.0));
    
    // Division
    let logic_id = engine.compile(&json!({"/": [20, 4]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(5.0));
    
    // Modulo
    let logic_id = engine.compile(&json!({"%": [10, 3]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(1.0));
    
    // Power
    let logic_id = engine.compile(&json!({"^": [2, 3]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(8.0));
    
    // Power with decimal
    let logic_id = engine.compile(&json!({"^": [4, 0.5]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(2.0));
}

#[test]
fn test_for_loop() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"base": 10}));
    
    // FOR loop that multiplies base by iteration
    let logic_id = engine.compile(&json!({
        "FOR": [
            1,
            3,
            {"*": [{"var": "base"}, {"var": "$iteration"}]}
        ]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    // Last iteration: 10 * 3 = 30
    assert_eq!(*result, json!(30.0));
    
    // FOR loop with addition
    let logic_id = engine.compile(&json!({
        "FOR": [
            0,
            5,
            {"+": [{"var": "base"}, {"var": "$iteration"}]}
        ]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    // Last iteration: 10 + 5 = 15
    assert_eq!(*result, json!(15.0));
}

#[test]
fn test_array_map() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"numbers": [1, 2, 3]}));
    
    let logic_id = engine.compile(&json!({
        "map": [
            {"var": "numbers"},
            {"*": [{"var": ""}, 2]}
        ]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!([2.0, 4.0, 6.0]));
}

#[test]
fn test_array_filter() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"numbers": [1, 2, 3, 4, 5]}));
    
    let logic_id = engine.compile(&json!({
        "filter": [
            {"var": "numbers"},
            {">": [{"var": ""}, 2]}
        ]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!([3, 4, 5]));
}

#[test]
fn test_array_all() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"numbers": [2, 4, 6]}));
    
    let logic_id = engine.compile(&json!({
        "all": [
            {"var": "numbers"},
            {"==": [{"%": [{"var": ""}, 2]}, 0]}
        ]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
}

#[test]
fn test_array_some() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"numbers": [1, 2, 3]}));
    
    let logic_id = engine.compile(&json!({
        "some": [
            {"var": "numbers"},
            {">": [{"var": ""}, 2]}
        ]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
}

#[test]
fn test_array_in() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"fruits": ["apple", "banana", "orange"]}));
    
    let logic_id = engine.compile(&json!({
        "in": ["banana", {"var": "fruits"}]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
    
    let logic_id = engine.compile(&json!({
        "in": ["grape", {"var": "fruits"}]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(false));
}

#[test]
fn test_string_cat() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"first": "Hello", "last": "World"}));
    
    let logic_id = engine.compile(&json!({
        "cat": [{"var": "first"}, " ", {"var": "last"}]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!("Hello World"));
}

#[test]
fn test_string_substr() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"text": "Hello World"}));
    
    let logic_id = engine.compile(&json!({
        "substr": [{"var": "text"}, 0, 5]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!("Hello"));
}

#[test]
fn test_missing() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"name": "John"}));
    
    let logic_id = engine.compile(&json!({
        "missing": ["name", "age"]
    })).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(["age"]));
}

#[test]
fn test_cache_hit() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"x": 10}));
    
    let logic_id = engine.compile(&json!({"+": [{"var": "x"}, 5]})).unwrap();
    
    // First evaluation - cache miss
    let result1 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result1, json!(15.0));
    
    // Second evaluation - cache hit
    let result2 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result2, json!(15.0));
    
    let stats = engine.cache_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
}

#[test]
fn test_cache_invalidation_on_mutation() {
    let mut engine = RLogic::new();
    let mut data = TrackedData::new(json!({"x": 10}));
    
    let logic_id = engine.compile(&json!({"+": [{"var": "x"}, 5]})).unwrap();
    
    // First evaluation
    let result1 = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result1, json!(15.0));
    
    // Mutate data - version changes
    let _new_version = data.set("x", json!(20));
    
    // Create new tracked data with updated version
    let data2 = data.clone();
    
    // Evaluation with new version - cache miss
    let result2 = engine.evaluate(&logic_id, &data2).unwrap();
    assert_eq!(*result2, json!(25.0));
    
    let stats = engine.cache_stats();
    assert_eq!(stats.misses, 2); // Both were cache misses due to different versions
}

#[test]
fn test_tracked_data_mutation() {
    let mut data = TrackedData::new(json!({"name": "John", "age": 30}));
    let v1 = data.version();
    
    data.set("age", json!(31));
    let v2 = data.version();
    
    assert!(v2 > v1);
    assert_eq!(data.get("age"), Some(&json!(31)));
}

#[test]
fn test_complex_logic() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({
        "user": {
            "age": 25,
            "premium": true
        },
        "cart": {
            "total": 150
        }
    }));
    
    // If user is premium and age > 18 and cart total > 100, apply 20% discount
    let logic_id = engine.compile(&json!({
        "if": [
            {"and": [
                {"var": "user.premium"},
                {">": [{"var": "user.age"}, 18]},
                {">": [{"var": "cart.total"}, 100]}
            ]},
            {"*": [{"var": "cart.total"}, 0.8]},
            {"var": "cart.total"}
        ]
    })).unwrap();
    
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(120.0)); // 150 * 0.8 = 120
}

#[test]
fn test_referenced_vars() {
    let mut engine = RLogic::new();
    
    let logic_id = engine.compile(&json!({
        "if": [
            {"and": [
                {"var": "user.premium"},
                {">": [{"var": "user.age"}, 18]}
            ]},
            {"var": "discount"},
            0
        ]
    })).unwrap();
    
    let vars = engine.get_referenced_vars(&logic_id).unwrap();
    assert!(vars.contains(&"user.premium".to_string()));
    assert!(vars.contains(&"user.age".to_string()));
    assert!(vars.contains(&"discount".to_string()));
}

#[test]
fn test_evaluate_direct() {
    let mut engine = RLogic::new();
    
    let result = engine.evaluate_direct(
        &json!({"+": [2, 3]}),
        &json!({})
    ).unwrap();
    
    assert_eq!(result, json!(5.0));
}

#[test]
fn test_builder_pattern() {
    let engine = RLogic::with_config(RLogicConfig::new().with_recursion_limit(500));
    assert!(engine.cache_stats().size == 0);
}

#[test]
fn test_tracked_data_builder() {
    let data = TrackedDataBuilder::new()
        .set("name", json!("John"))
        .set("age", json!(30))
        .build();
    
    assert_eq!(data.get("name"), Some(&json!("John")));
    assert_eq!(data.get("age"), Some(&json!(30)));
}

#[test]
fn test_array_merge() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({
        "arr1": [1, 2],
        "arr2": [3, 4]
    }));
    
    let logic_id = engine.compile(&json!({
        "merge": [{"var": "arr1"}, {"var": "arr2"}]
    })).unwrap();
    
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!([1, 2, 3, 4]));
}

#[test]
fn test_strict_equality() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({}));
    
    // Loose equality: "5" == 5
    let logic_id = engine.compile(&json!({"==": ["5", 5]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(true));
    
    // Strict equality: "5" === 5
    let logic_id = engine.compile(&json!({"===": ["5", 5]})).unwrap();
    let result = engine.evaluate(&logic_id, &data).unwrap();
    assert_eq!(*result, json!(false));
}

#[test]
fn test_performance_multiple_evaluations() {
    let mut engine = RLogic::new();
    let data = TrackedData::new(json!({"x": 10, "y": 20}));
    
    let logic_id = engine.compile(&json!({
        "+": [
            {"*": [{"var": "x"}, 2]},
            {"*": [{"var": "y"}, 3]}
        ]
    })).unwrap();
    
    // Multiple evaluations should benefit from cache
    for _ in 0..100 {
        let result = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(80.0)); // (10*2) + (20*3) = 80
    }
    
    let stats = engine.cache_stats();
    assert_eq!(stats.hits, 99); // First is miss, rest are hits
    assert_eq!(stats.misses, 1);
    assert!(stats.hit_rate > 0.98);
}
