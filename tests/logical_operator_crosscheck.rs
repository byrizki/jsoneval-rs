/// Cross-check test to verify Rust implementation matches JS behavior
/// for all logical operators from js-version/internal/subscript/feature/logic.js
///
/// JS Reference:
/// - unary('!', PREC_PREFIX), operator('!', (a, b) => !b && (a = compile(a), ctx => !a(ctx)))
/// - binary('||', PREC_LOR), operator('||', (a, b) => (a = compile(a), b = compile(b), ctx => a(ctx) || b(ctx)))
/// - binary('&&', PREC_LAND), operator('&&', (a, b) => (a = compile(a), b = compile(b), ctx => a(ctx) && b(ctx)))

use json_eval_rs::RLogic;
use serde_json::json;

#[test]
fn crosscheck_not_operator() {
    let mut engine = RLogic::new();
    let data = json!({});

    println!("\n=== NOT (!) Operator Cross-Check ===");
    
    // JS: !true => false
    let logic_id = engine.compile(&json!({"!": true})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!true = {:?}", result);
    assert_eq!(result, json!(false), "JS: !true should return false");

    // JS: !false => true
    let logic_id = engine.compile(&json!({"!": false})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!false = {:?}", result);
    assert_eq!(result, json!(true), "JS: !false should return true");

    // JS: !0 => true (0 is falsy)
    let logic_id = engine.compile(&json!({"!": 0})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!0 = {:?}", result);
    assert_eq!(result, json!(true), "JS: !0 should return true");

    // JS: !1 => false (1 is truthy)
    let logic_id = engine.compile(&json!({"!": 1})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!1 = {:?}", result);
    assert_eq!(result, json!(false), "JS: !1 should return false");

    // JS: !"" => true (empty string is falsy)
    let logic_id = engine.compile(&json!({"!": ""})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!\"\" = {:?}", result);
    assert_eq!(result, json!(true), "JS: !\"\" should return true");

    // JS: !"hello" => false (non-empty string is truthy)
    let logic_id = engine.compile(&json!({"!": "hello"})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!\"hello\" = {:?}", result);
    assert_eq!(result, json!(false), "JS: !\"hello\" should return false");

    // JS: !null => true (null is falsy)
    let logic_id = engine.compile(&json!({"!": null})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!null = {:?}", result);
    assert_eq!(result, json!(true), "JS: !null should return true");

    // JS: ![] => true (empty array is falsy)
    // Note: In JSON logic, empty array as argument needs to be wrapped in array
    let logic_id = engine.compile(&json!({"!": [[]]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("![] = {:?}", result);
    assert_eq!(result, json!(true), "JS: ![] should return true");

    // JS: ![1,2,3] => false (non-empty array is truthy)
    let logic_id = engine.compile(&json!({"!": [[1, 2, 3]]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("![1,2,3] = {:?}", result);
    assert_eq!(result, json!(false), "JS: ![1,2,3] should return false");

    println!("✓ NOT operator matches JS behavior");
}

#[test]
fn crosscheck_or_operator() {
    let mut engine = RLogic::new();
    let data = json!({});

    println!("\n=== OR (||) Operator Cross-Check ===");
    
    // JS: true || false => true (returns first truthy)
    let logic_id = engine.compile(&json!({"or": [true, false]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("true || false = {:?}", result);
    assert_eq!(result, json!(true), "JS: true || false should return true");

    // JS: false || true => true (returns first truthy)
    let logic_id = engine.compile(&json!({"or": [false, true]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("false || true = {:?}", result);
    assert_eq!(result, json!(true), "JS: false || true should return true");

    // JS: false || false => false (returns last value when all falsy)
    let logic_id = engine.compile(&json!({"or": [false, false]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("false || false = {:?}", result);
    assert_eq!(result, json!(false), "JS: false || false should return false");

    // JS: 0 || 5 => 5 (returns first truthy)
    let logic_id = engine.compile(&json!({"or": [0, 5]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("0 || 5 = {:?}", result);
    assert_eq!(result, json!(5), "JS: 0 || 5 should return 5");

    // JS: 5 || 10 => 5 (short-circuits, returns first truthy)
    let logic_id = engine.compile(&json!({"or": [5, 10]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("5 || 10 = {:?}", result);
    assert_eq!(result, json!(5), "JS: 5 || 10 should return 5");

    // JS: "" || "hello" => "hello" (returns first truthy)
    let logic_id = engine.compile(&json!({"or": ["", "hello"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("\"\" || \"hello\" = {:?}", result);
    assert_eq!(result, json!("hello"), "JS: \"\" || \"hello\" should return \"hello\"");

    // JS: "hello" || "world" => "hello" (short-circuits)
    let logic_id = engine.compile(&json!({"or": ["hello", "world"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("\"hello\" || \"world\" = {:?}", result);
    assert_eq!(result, json!("hello"), "JS: \"hello\" || \"world\" should return \"hello\"");

    // JS: null || 42 => 42
    let logic_id = engine.compile(&json!({"or": [null, 42]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("null || 42 = {:?}", result);
    assert_eq!(result, json!(42), "JS: null || 42 should return 42");

    // JS: false || 0 || null => null (all falsy, returns last)
    let logic_id = engine.compile(&json!({"or": [false, 0, null]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("false || 0 || null = {:?}", result);
    assert_eq!(result, json!(null), "JS: false || 0 || null should return null");

    // JS: false || 0 || "" || 42 || "never" => 42 (returns first truthy)
    let logic_id = engine.compile(&json!({"or": [false, 0, "", 42, "never"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("false || 0 || \"\" || 42 || \"never\" = {:?}", result);
    assert_eq!(result, json!(42), "JS: should return 42");

    println!("✓ OR operator matches JS behavior");
}

#[test]
fn crosscheck_and_operator() {
    let mut engine = RLogic::new();
    let data = json!({});

    println!("\n=== AND (&&) Operator Cross-Check ===");
    
    // JS: true && false => false (returns first falsy)
    let logic_id = engine.compile(&json!({"and": [true, false]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("true && false = {:?}", result);
    assert_eq!(result, json!(false), "JS: true && false should return false");

    // JS: false && true => false (short-circuits, returns first falsy)
    let logic_id = engine.compile(&json!({"and": [false, true]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("false && true = {:?}", result);
    assert_eq!(result, json!(false), "JS: false && true should return false");

    // JS: true && true => true (all truthy, returns last)
    let logic_id = engine.compile(&json!({"and": [true, true]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("true && true = {:?}", result);
    assert_eq!(result, json!(true), "JS: true && true should return true");

    // JS: 5 && 10 => 10 (all truthy, returns last)
    let logic_id = engine.compile(&json!({"and": [5, 10]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("5 && 10 = {:?}", result);
    assert_eq!(result, json!(10), "JS: 5 && 10 should return 10");

    // JS: 0 && 10 => 0 (returns first falsy)
    let logic_id = engine.compile(&json!({"and": [0, 10]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("0 && 10 = {:?}", result);
    assert_eq!(result, json!(0), "JS: 0 && 10 should return 0");

    // JS: 5 && 0 => 0 (returns first falsy)
    let logic_id = engine.compile(&json!({"and": [5, 0]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("5 && 0 = {:?}", result);
    assert_eq!(result, json!(0), "JS: 5 && 0 should return 0");

    // JS: "hello" && "world" => "world" (all truthy, returns last)
    let logic_id = engine.compile(&json!({"and": ["hello", "world"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("\"hello\" && \"world\" = {:?}", result);
    assert_eq!(result, json!("world"), "JS: \"hello\" && \"world\" should return \"world\"");

    // JS: "" && "world" => "" (returns first falsy)
    let logic_id = engine.compile(&json!({"and": ["", "world"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("\"\" && \"world\" = {:?}", result);
    assert_eq!(result, json!(""), "JS: \"\" && \"world\" should return \"\"");

    // JS: "hello" && "" => "" (returns first falsy)
    let logic_id = engine.compile(&json!({"and": ["hello", ""]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("\"hello\" && \"\" = {:?}", result);
    assert_eq!(result, json!(""), "JS: \"hello\" && \"\" should return \"\"");

    // JS: 1 && "hello" && 0 && "never" => 0 (returns first falsy)
    let logic_id = engine.compile(&json!({"and": [1, "hello", 0, "never"]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("1 && \"hello\" && 0 && \"never\" = {:?}", result);
    assert_eq!(result, json!(0), "JS: should return 0");

    // JS: 1 && "hello" && 42 => 42 (all truthy, returns last)
    let logic_id = engine.compile(&json!({"and": [1, "hello", 42]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("1 && \"hello\" && 42 = {:?}", result);
    assert_eq!(result, json!(42), "JS: should return 42");

    println!("✓ AND operator matches JS behavior");
}

#[test]
fn crosscheck_short_circuit_behavior() {
    let mut engine = RLogic::new();
    let data = json!({});

    println!("\n=== Short-Circuit Behavior Cross-Check ===");
    
    // JS: false && (this would error) => false (short-circuits before error)
    let logic_id = engine.compile(&json!({"and": [false, {"+": [1, "error"]}]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("false && (error) = {:?}", result);
    assert_eq!(result, json!(false), "JS: AND should short-circuit on false");

    // JS: true || (this would error) => true (short-circuits before error)
    let logic_id = engine.compile(&json!({"or": [true, {"+": [1, "error"]}]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("true || (error) = {:?}", result);
    assert_eq!(result, json!(true), "JS: OR should short-circuit on true");

    // JS: 0 && anything => 0 (short-circuits)
    let logic_id = engine.compile(&json!({"and": [0, 100, 200]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("0 && 100 && 200 = {:?}", result);
    assert_eq!(result, json!(0), "JS: should short-circuit at 0");

    // JS: 5 || anything => 5 (short-circuits)
    let logic_id = engine.compile(&json!({"or": [5, 100, 200]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("5 || 100 || 200 = {:?}", result);
    assert_eq!(result, json!(5), "JS: should short-circuit at 5");

    println!("✓ Short-circuit behavior matches JS");
}

#[test]
fn crosscheck_combined_logical_operations() {
    let mut engine = RLogic::new();
    let data = json!({"a": 5, "b": 0, "c": 10});

    println!("\n=== Combined Logical Operations Cross-Check ===");
    
    // JS: (5 && 10) || 0 => 10
    let logic_id = engine.compile(&json!({"or": [{"and": [{"var": "a"}, {"var": "c"}]}, {"var": "b"}]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("(5 && 10) || 0 = {:?}", result);
    assert_eq!(result.as_f64(), Some(10.0), "JS: (5 && 10) || 0 should return 10");

    // JS: (5 && 0) || 10 => 10
    let logic_id = engine.compile(&json!({"or": [{"and": [{"var": "a"}, {"var": "b"}]}, {"var": "c"}]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("(5 && 0) || 10 = {:?}", result);
    assert_eq!(result.as_f64(), Some(10.0), "JS: (5 && 0) || 10 should return 10");

    // JS: 5 && (0 || 10) => 10
    let logic_id = engine.compile(&json!({"and": [{"var": "a"}, {"or": [{"var": "b"}, {"var": "c"}]}]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("5 && (0 || 10) = {:?}", result);
    assert_eq!(result.as_f64(), Some(10.0), "JS: 5 && (0 || 10) should return 10");

    // JS: !(5 && 10) => false
    let logic_id = engine.compile(&json!({"!": {"and": [{"var": "a"}, {"var": "c"}]}})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!(5 && 10) = {:?}", result);
    assert_eq!(result, json!(false), "JS: !(5 && 10) should return false");

    // JS: !(0 || false) => true
    let logic_id = engine.compile(&json!({"!": {"or": [{"var": "b"}, false]}})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!(0 || false) = {:?}", result);
    assert_eq!(result, json!(true), "JS: !(0 || false) should return true");

    println!("✓ Combined operations match JS behavior");
}

#[test]
fn crosscheck_with_arrays_and_special_values() {
    let mut engine = RLogic::new();
    let data = json!({});

    println!("\n=== Arrays and Special Values Cross-Check ===");
    
    // JS: [] || [1,2,3] => [1,2,3] (empty array is falsy)
    let logic_id = engine.compile(&json!({"or": [[], [1, 2, 3]]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("[] || [1,2,3] = {:?}", result);
    assert_eq!(result, json!([1, 2, 3]), "JS: [] || [1,2,3] should return [1,2,3]");

    // JS: [1,2] && [3,4] => [3,4] (both truthy, returns last)
    let logic_id = engine.compile(&json!({"and": [[1, 2], [3, 4]]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("[1,2] && [3,4] = {:?}", result);
    assert_eq!(result, json!([3, 4]), "JS: [1,2] && [3,4] should return [3,4]");

    // JS: !([]) => true (empty array is falsy)
    let logic_id = engine.compile(&json!({"!": [[]]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!([]) = {:?}", result);
    assert_eq!(result, json!(true), "JS: !([]) should return true");

    // JS: !([1,2,3]) => false (non-empty array is truthy)
    let logic_id = engine.compile(&json!({"!": [[1, 2, 3]]})).unwrap();
    let result = engine.run(&logic_id, &data).unwrap();
    println!("!([1,2,3]) = {:?}", result);
    assert_eq!(result, json!(false), "JS: !([1,2,3]) should return false");

    println!("✓ Arrays and special values match JS behavior");
}
