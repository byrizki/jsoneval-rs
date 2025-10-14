use json_eval_rs::RLogic;
use serde_json::json;

/// Test that logical operators return actual values, not just booleans
/// This matches JavaScript behavior where:
/// - `true || "error"` returns `true` (not true converted to boolean)
/// - `false || 42` returns `42` (not 42 converted to boolean)
/// - `1 && 2` returns `2` (the last truthy value)
/// - `0 && 2` returns `0` (the first falsy value)
#[test]
fn test_logical_operators_return_actual_values() {
    let mut engine = RLogic::new();
    let data = json!({});

    // OR operator should return the first truthy value
    let logic_id = engine.compile(&json!({"or": [false, 0, "", 42, "never"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(42.0), "OR should return first truthy value (42)");

    // OR with all falsy should return the last value
    let logic_id = engine.compile(&json!({"or": [false, 0, null]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(null), "OR with all falsy should return last value (null)");

    // AND operator should return the first falsy value
    let logic_id = engine.compile(&json!({"and": [1, "hello", 0, "never"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(0.0), "AND should return first falsy value (0)");

    // AND with all truthy should return the last value
    let logic_id = engine.compile(&json!({"and": [1, "hello", 42]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(42.0), "AND with all truthy should return last value (42)");

    // OR with strings
    let logic_id = engine.compile(&json!({"or": ["", "first", "second"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!("first"), "OR should return first non-empty string");

    // AND with strings
    let logic_id = engine.compile(&json!({"and": ["first", "second", "third"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!("third"), "AND with all truthy strings should return last");

    // AND stops at first falsy
    let logic_id = engine.compile(&json!({"and": ["first", false, "third"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(false), "AND should stop and return false");

    // OR stops at first truthy
    let logic_id = engine.compile(&json!({"or": [false, "found", false]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!("found"), "OR should stop and return 'found'");

    // Numbers with OR
    let logic_id = engine.compile(&json!({"or": [0, 5]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(5.0), "OR: 0 || 5 should return 5");

    // Numbers with AND
    let logic_id = engine.compile(&json!({"and": [5, 10]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(10.0), "AND: 5 && 10 should return 10");

    let logic_id = engine.compile(&json!({"and": [0, 10]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(0.0), "AND: 0 && 10 should return 0");
}

#[test]
fn test_logical_operators_with_arrays_and_objects() {
    let mut engine = RLogic::new();
    let data = json!({});

    // Arrays are truthy if non-empty, falsy if empty
    let logic_id = engine.compile(&json!({"or": [[], [1, 2, 3]]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!([1.0, 2.0, 3.0]), "OR should return non-empty array");

    let logic_id = engine.compile(&json!({"and": [[1, 2], [3, 4]]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!([3.0, 4.0]), "AND should return last array");

    // Empty arrays are falsy, test with strings instead
    let logic_id = engine.compile(&json!({"or": ["", "value"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!("value"), "OR should return non-empty string");
}

#[test]
fn test_js_compatibility_examples() {
    let mut engine = RLogic::new();
    let data = json!({});

    // Examples from JS that should work identically:
    // true || false => true
    let logic_id = engine.compile(&json!({"or": [true, false]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(true));

    // false || true => true
    let logic_id = engine.compile(&json!({"or": [false, true]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(true));

    // true && false => false
    let logic_id = engine.compile(&json!({"and": [true, false]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(false));

    // false && true => false
    let logic_id = engine.compile(&json!({"and": [false, true]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(false));

    // "hello" || "world" => "hello"
    let logic_id = engine.compile(&json!({"or": ["hello", "world"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!("hello"));

    // "" || "world" => "world"
    let logic_id = engine.compile(&json!({"or": ["", "world"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!("world"));

    // "hello" && "world" => "world"
    let logic_id = engine.compile(&json!({"and": ["hello", "world"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!("world"));

    // "" && "world" => ""
    let logic_id = engine.compile(&json!({"and": ["", "world"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    assert_eq!(result, json!(""));
}
