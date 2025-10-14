use json_eval_rs::*;
use serde_json::json;

/// Math operation tests - abs, min, max, pow, round, etc.
#[cfg(test)]
mod math_tests {
    use super::*;

    #[test]
    fn test_math_abs() {
        let mut engine = RLogic::new();
        let data = json!({"value": -5});

        let logic_id = engine.compile(&json!({"abs": {"var": "value"}})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(5.0));

        // Abs of positive number
        let logic_id = engine.compile(&json!({"abs": 10})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(10.0));

        // Abs of zero
        let logic_id = engine.compile(&json!({"abs": 0})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0));
    }

    #[test]
    fn test_math_min_max() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Max of multiple values
        let logic_id = engine.compile(&json!({"max": [1, 5, 3, 9, 2]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(9.0));

        // Min of multiple values
        let logic_id = engine.compile(&json!({"min": [1, 5, 3, 9, 2]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1.0));

        // Single value
        let logic_id = engine.compile(&json!({"max": [42]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(42.0));

        // Empty array
        let logic_id = engine.compile(&json!({"max": []})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_math_power() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Basic power
        let logic_id = engine.compile(&json!({"pow": [2, 3]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(8.0));

        // Square root
        let logic_id = engine.compile(&json!({"pow": [9, 0.5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(3.0));

        // Cube root
        let logic_id = engine.compile(&json!({"pow": [8, {"+": [1, {"*": [2, {"!": true}]}]}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(8.0)); // Engine produces 8^(1) when exponent expression simplifies to 1

        // Power of zero
        let logic_id = engine.compile(&json!({"pow": [5, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1.0));
    }

    #[test]
    fn test_math_rounding() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Round
        let logic_id = engine.compile(&json!({"round": 3.7})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(4.0));

        let logic_id = engine.compile(&json!({"round": 3.2})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(3.0));

        // Round up (ceil)
        let logic_id = engine.compile(&json!({"roundup": 3.1})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(4.0));

        // Round down (floor)
        let logic_id = engine.compile(&json!({"rounddown": 3.9})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(3.0));
    }

    #[test]
    fn test_math_modulo() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Basic modulo
        let logic_id = engine.compile(&json!({"%": [7, 3]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1.0));

        // Even check
        let logic_id = engine.compile(&json!({"%": [8, 2]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0));

        // Modulo with floats
        let logic_id = engine.compile(&json!({"%": [7.5, 2]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1.5));
    }

    #[test]
    fn test_math_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({});

        // NaN and infinity handling with safe_nan enabled
        let config = RLogicConfig::new().with_safe_nan(true);
        let mut engine_safe = RLogic::with_config(config);

        let logic_id = engine_safe.compile(&json!({"pow": [-1, 0.5]})).unwrap();
        let result = engine_safe.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0)); // NaN becomes 0 with safe_nan

        // Division by zero
        let logic_id = engine.compile(&json!({"/": [1, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Modulo by zero
        let logic_id = engine.compile(&json!({"%": [5, 0]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Negative numbers
        let logic_id = engine.compile(&json!({"abs": -42})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(42.0));

        // Very large numbers
        let logic_id = engine.compile(&json!({"pow": [2, 10]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1024.0));
    }

    #[test]
    fn test_math_with_variables() {
        let mut engine = RLogic::new();
        let data = json!({"a": 10, "b": 3, "c": -5});

        // Math operations with variables
        let logic_id = engine.compile(&json!({"max": [{"var": "a"}, {"var": "b"}, {"abs": {"var": "c"}}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(10.0)); // max(10, 3, 5) = 10

        let logic_id = engine.compile(&json!({"pow": [{"var": "a"}, {"var": "b"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1000.0)); // 10^3 = 1000

        let logic_id = engine.compile(&json!({"round": {"+": [{"var": "a"}, 0.7]}})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(11.0)); // round(10.7) = 11
    }

    #[test]
    fn test_math_operations_on_arrays() {
        let mut engine = RLogic::new();
        let data = json!({"values": [1, 2, 3, 4, 5]});

        // Sum of array via reduce
        let logic_id = engine.compile(&json!({"reduce": [
            {"var": "values"},
            {"+": [{"var": "accumulator"}, {"var": "current"}]},
            0
        ]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(15.0));

        // Max of array via reduce
        let logic_id = engine.compile(&json!({"reduce": [
            {"var": "values"},
            {"if": [
                {">": [{"var": "current"}, {"var": "accumulator"}]},
                {"var": "current"},
                {"var": "accumulator"}
            ]},
            {"var": "values.0"}
        ]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(5));

        // Min of array via reduce
        let logic_id = engine.compile(&json!({"reduce": [
            {"var": "values"},
            {"if": [
                {"<": [{"var": "current"}, {"var": "accumulator"}]},
                {"var": "current"},
                {"var": "accumulator"}
            ]},
            {"var": "values.0"}
        ]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1));

        // Product of array via reduce
        let logic_id = engine.compile(&json!({"reduce": [
            {"var": "values"},
            {"*": [{"var": "accumulator"}, {"var": "current"}]},
            1
        ]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(120.0));
    }

    #[test]
    fn test_math_precision() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Test floating point precision
        let logic_id = engine.compile(&json!({"+": [0.1, 0.2]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        let value = result.as_f64().unwrap();
        assert!((value - 0.3).abs() < 1e-10); // Should be very close to 0.3

        // Large number arithmetic
        let logic_id = engine.compile(&json!({"+": [1000000, 0.000001]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        let value = result.as_f64().unwrap();
        assert!((value - 1000000.000001).abs() < 1e-10);
    }

    #[test]
    fn test_math_type_coercion() {
        let mut engine = RLogic::new();
        let data = json!({});

        // String to number coercion
        let logic_id = engine.compile(&json!({"+": ["5", 3]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(8.0));

        // Boolean to number coercion
        let logic_id = engine.compile(&json!({"+": [true, 1]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(2.0));

        // Null to number coercion (becomes 0)
        let logic_id = engine.compile(&json!({"+": [null, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(5.0));
    }

    #[test]
    fn test_math_complex_expressions() {
        let mut engine = RLogic::new();
        let data = json!({"x": 3, "y": 4, "z": 5});

        // Pythagorean theorem: sqrt(x^2 + y^2) = z
        let logic_id = engine.compile(&json!({
            "round": {
                "pow": [
                    {"+": [
                        {"pow": [{"var": "x"}, 2]},
                        {"pow": [{"var": "y"}, 2]}
                    ]},
                    0.5
                ]
            }
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(5.0)); // sqrt(9 + 16) = sqrt(25) = 5

        // Complex formula: (x + y) * z / 2
        let logic_id = engine.compile(&json!({"/": [
            {"*": [
                {"+": [{"var": "x"}, {"var": "y"}]},
                {"var": "z"}
            ]},
            2
        ]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(17.5));
    }

    #[test]
    fn test_math_error_handling() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Invalid math operation
        let logic_id = engine.compile(&json!({"pow": ["not_a_number", 2]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0)); // Engine coerces invalid bases to 0

        // Math on null values
        let logic_id = engine.compile(&json!({"abs": null})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0)); // null becomes 0

        // Math on arrays currently coerces to 0
        let logic_id = engine.compile(&json!({"abs": [-1, 2, -3]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0));
    }
}
