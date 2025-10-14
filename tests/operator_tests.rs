use json_eval_rs::*;
use serde_json::json;

/// Operator tests - arithmetic, comparison, and logical operations
#[cfg(test)]
mod operator_tests {
    use super::*;

    #[test]
    fn test_arithmetic_operators() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Addition
        let logic_id = engine.compile(&json!({"+": [2, 3, 4]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(9.0));

        // Subtraction
        let logic_id = engine.compile(&json!({"-": [10, 3, 2]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(5.0));

        // Multiplication
        let logic_id = engine.compile(&json!({"*": [2, 3, 4]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(24.0));

        // Division
        let logic_id = engine.compile(&json!({"/": [24, 3, 2]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(4.0));
    }

    #[test]
    fn test_arithmetic_with_variables() {
        let mut engine = RLogic::new();
        let data = json!({"a": 10, "b": 5, "c": 2});

        let logic_id = engine.compile(&json!({"+": [{"var": "a"}, {"var": "b"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(15.0));

        let logic_id = engine.compile(&json!({"*": [{"var": "a"}, {"var": "b"}, {"var": "c"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(100.0));
    }

    #[test]
    fn test_arithmetic_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Single operand
        let logic_id = engine.compile(&json!({"+": [42]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(42.0));

        // Empty array
        let logic_id = engine.compile(&json!({"+": []})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0));

        // Division by zero
        let logic_id = engine.compile(&json!({"/": [10, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_comparison_operators() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Equality
        let logic_id = engine.compile(&json!({"==": [5, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        let logic_id = engine.compile(&json!({"==": [5, 6]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));

        // Inequality
        let logic_id = engine.compile(&json!({"!=": [5, 6]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        // Less than
        let logic_id = engine.compile(&json!({"<": [3, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        // Greater than
        let logic_id = engine.compile(&json!({">": [7, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        // Less than or equal
        let logic_id = engine.compile(&json!({"<=": [5, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        // Greater than or equal
        let logic_id = engine.compile(&json!({">=": [5, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_loose_equality() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Number and string
        let logic_id = engine.compile(&json!({"==": [5, "5"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        // Boolean and number
        let logic_id = engine.compile(&json!({"==": [true, 1]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        // Null comparisons
        let logic_id = engine.compile(&json!({"==": [null, null]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        let logic_id = engine.compile(&json!({"==": [null, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_logical_operators() {
        let mut engine = RLogic::new();
        let data = json!({});

        // And
        let logic_id = engine.compile(&json!({"and": [true, true, true]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        let logic_id = engine.compile(&json!({"and": [true, false, true]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));

        // Or
        let logic_id = engine.compile(&json!({"or": [false, false, true]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));

        let logic_id = engine.compile(&json!({"or": [false, false, false]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));

        // Not
        let logic_id = engine.compile(&json!({"!": true})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));

        let logic_id = engine.compile(&json!({"!": false})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_logical_short_circuit() {
        let mut engine = RLogic::new();
        let data = json!({});

        // And short-circuits on false
        let logic_id = engine.compile(&json!({"and": [false, {"+": [1, "error"]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(false));

        // Or short-circuits on true
        let logic_id = engine.compile(&json!({"or": [true, {"+": [1, "error"]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_if_condition() {
        let mut engine = RLogic::new();
        let data = json!({"value": 10});

        // True condition
        let logic_id = engine.compile(&json!({"if": [{"var": "value"}, "yes", "no"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("yes"));

        // False condition
        let data = json!({"value": 0});
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("no"));

        // Truthy values
        let data = json!({"value": "hello"});
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("yes"));
    }

    #[test]
    fn test_truthy_falsy_values() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Test various truthy/falsy values
        let test_cases = vec![
            (json!(true), true),
            (json!(false), false),
            (json!(1), true),
            (json!(0), false),
            (json!(-1), true),
            (json!("hello"), true),
            (json!(""), false),
            (json!([]), false),
            (json!([1,2,3]), true),
            (json!({}), false),
            (json!(null), false),
        ];

        for (value, expected_truthy) in test_cases {
            let logic_id = engine.compile(&json!({"if": [value, true, false]})).unwrap();
            let result = engine.run(&logic_id, &data).unwrap();
            assert_eq!(result, json!(expected_truthy), "Value {:?} should be truthy: {}", value, expected_truthy);
        }
    }

    #[test]
    fn test_nested_expressions() {
        let mut engine = RLogic::new();
        let data = json!({"a": 2, "b": 3, "c": 4});

        // Nested arithmetic
        let logic_id = engine.compile(&json!({"+": [{"*": [{"var": "a"}, {"var": "b"}]}, {"var": "c"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(10.0)); // (2*3) + 4 = 10

        // Nested logical
        let logic_id = engine.compile(&json!({"and": [{"<": [{"var": "a"}, {"var": "c"}]}, {">": [{"var": "b"}, {"var": "a"}]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(true)); // (2 < 4) AND (3 > 2)
    }

    #[test]
    fn test_operator_precedence() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Test that operators work as expected (no built-in precedence, explicit nesting required)
        let logic_id = engine.compile(&json!({"+": [2, {"*": [3, 4]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(14.0)); // 2 + (3*4) = 14
    }
}
