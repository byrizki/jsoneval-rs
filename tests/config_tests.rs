use json_eval_rs::*;
use serde_json::json;

/// Configuration tests - recursion limits, NaN handling, tracking, etc.
#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_custom_config() {
        let config = RLogicConfig::new()
            .with_tracking(false)
            .with_safe_nan(true)
            .with_recursion_limit(200);

        let mut engine = RLogic::with_config(config);
        let logic_id = engine.compile(&json!({"+": [2, 3]})).unwrap();
        let data = json!({});

        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(5.0));
    }

    #[test]
    fn test_evaluation_works() {
        let mut engine = RLogic::new();
        let logic_id = engine.compile(&json!({"+": [2, 3]})).unwrap();
        let data = json!({});

        // First evaluation
        let result1 = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result1, json!(5.0));

        // Second evaluation
        let result2 = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result2, json!(5.0));
    }

    #[test]
    fn test_safe_nan_handling() {
        // Test with safe NaN handling config
        let config_safe = RLogicConfig::new().with_safe_nan(true);
        let mut engine = RLogic::new();
        let data = json!({});

        // Test NaN from sqrt of negative
        let logic_id = engine.compile(&json!({"pow": [-1, 0.5]})).unwrap();
        let result = engine.run_with_config(&logic_id, &data, &config_safe).unwrap();
        // With safe_nan_handling, NaN becomes 0
        assert_eq!(result, json!(0.0));

        // Without safe NaN handling - NaN becomes null in JSON
        let config_unsafe = RLogicConfig::new().with_safe_nan(false);
        let result2 = engine.run_with_config(&logic_id, &data, &config_unsafe).unwrap();
        // Without safe_nan_handling, NaN becomes null in JSON
        assert_eq!(result2, json!(null));
    }

    #[test]
    fn test_recursion_limit() {
        let config = RLogicConfig::new().with_recursion_limit(5);
        let mut engine = RLogic::with_config(config);

        // Create deeply nested logic that exceeds limit
        let deep_logic = json!({
            "if": [
                true,
                {"if": [
                    true,
                    {"if": [
                        true,
                        {"if": [
                            true,
                            {"if": [
                                true,
                                {"if": [
                                    true,
                                    1,
                                    0
                                ]},
                                0
                            ]},
                            0
                        ]},
                        0
                    ]},
                    0
                ]},
                0
            ]
        });

        let logic_id = engine.compile(&deep_logic).unwrap();
        let data = json!({});

        let result = engine.run(&logic_id, &data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Recursion limit"));
    }

    #[test]
    fn test_performance_config() {
        let config = RLogicConfig::performance(); // tracking disabled
        let mut engine = RLogic::with_config(config);

        let logic_id = engine.compile(&json!({"+": [{"var": "a"}, {"var": "b"}]})).unwrap();

        // Use evaluation with plain JSON
        let data = json!({"a": 10, "b": 20});
        let result = engine.run(&logic_id, &data).unwrap();

        assert_eq!(result, json!(30.0));
    }

    #[test]
    fn test_safe_config() {
        let config = RLogicConfig::safe();
        let mut engine = RLogic::with_config(config);

        // Test that safe config enables safe NaN handling
        let logic_id = engine.compile(&json!({"pow": [-1, 0.5]})).unwrap();
        let data = json!({});
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0));
    }

    #[test]
    fn test_config_defaults() {
        let config = RLogicConfig::default();
        assert_eq!(config.enable_tracking, true);
        assert_eq!(config.safe_nan_handling, false); // Default is false
        assert_eq!(config.recursion_limit, 1000);
    }

    #[test]
    fn test_config_affects_evaluation() {
        // Test that config actually affects evaluation behavior
        let data = json!({});

        // Config with tracking disabled
        let config_perf = RLogicConfig::performance();
        let mut engine_perf = RLogic::with_config(config_perf);
        let logic_id1 = engine_perf.compile(&json!({"+": [1, 2]})).unwrap();

        engine_perf.run(&logic_id1, &data).unwrap();
        engine_perf.run(&logic_id1, &data).unwrap();

        // Config with tracking enabled
        let config_safe = RLogicConfig::safe();
        let mut engine_safe = RLogic::with_config(config_safe);
        let logic_id2 = engine_safe.compile(&json!({"+": [1, 2]})).unwrap();

        engine_safe.run(&logic_id2, &data).unwrap();
        engine_safe.run(&logic_id2, &data).unwrap();

        // Both should produce correct results regardless of tracking
        assert_eq!(engine_perf.run(&logic_id1, &data).unwrap(), json!(3.0));
        assert_eq!(engine_safe.run(&logic_id2, &data).unwrap(), json!(3.0));
    }

    #[test]
    fn test_config_edge_cases() {
        // Test very high recursion limit (should work)
        let config = RLogicConfig::new().with_recursion_limit(1000);
        let mut engine = RLogic::with_config(config);

        // Simple operation should still work
        let logic_id = engine.compile(&json!({"+": [1, 2]})).unwrap();
        let data = json!({});
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(3.0));
    }

    #[test]
    fn test_config_nan_handling_detailed() {
        let data = json!({});

        // Test various NaN-producing operations
        let nan_operations = vec![
            (json!({"pow": [-1, 0.5]}), "sqrt of negative"),
        ];

        for (operation, description) in nan_operations {
            // With safe NaN handling
            let config_safe = RLogicConfig::new().with_safe_nan(true);
            let mut engine_safe = RLogic::with_config(config_safe);
            let logic_id = engine_safe.compile(&operation).unwrap();
            let result_safe = engine_safe.run(&logic_id, &data).unwrap();
            
            // Without safe NaN handling
            let config_unsafe = RLogicConfig::new().with_safe_nan(false);
            let mut engine_unsafe = RLogic::with_config(config_unsafe);
            let logic_id = engine_unsafe.compile(&operation).unwrap();
            let result_unsafe = engine_unsafe.run(&logic_id, &data).unwrap();

            // NaN should be converted to 0 with safe handling, null without
            assert_eq!(result_safe, json!(0.0), "Safe result for {} should be 0", description);
            assert_eq!(result_unsafe, json!(null), "Unsafe result for {} should be null", description);
        }
    }

    #[test]
    fn test_config_isolation() {
        // Test that different engine instances maintain separate configs
        let config1 = RLogicConfig::new().with_recursion_limit(10).with_safe_nan(true);
        let config2 = RLogicConfig::new().with_recursion_limit(20).with_safe_nan(false);

        let mut engine1 = RLogic::with_config(config1);
        let mut engine2 = RLogic::with_config(config2);

        // Both should work with simple operations
        let data = json!({});
        let logic_id1 = engine1.compile(&json!({"+": [1, 2]})).unwrap();
        let logic_id2 = engine2.compile(&json!({"+": [1, 2]})).unwrap();

        let result1 = engine1.run(&logic_id1, &data).unwrap();
        let result2 = engine2.run(&logic_id2, &data).unwrap();

        assert_eq!(result1, json!(3.0));
        assert_eq!(result2, json!(3.0));
    }

    #[test]
    fn test_config_runtime_changes() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Test with default config
        let logic_id = engine.compile(&json!({"pow": [-1, 0.5]})).unwrap();
        let result1 = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result1, json!(null)); // Default is NOT safe NaN

        // Test with runtime config override
        let config_unsafe = RLogicConfig::new().with_safe_nan(false);
        let result2 = engine.run_with_config(&logic_id, &data, &config_unsafe).unwrap();
        assert_eq!(result2, json!(null)); // Override to unsafe NaN
    }

    #[test]
    fn test_config_complex_scenarios() {
        let data = json!({
            "nested": {
                "value": -1
            }
        });

        // Test config with complex nested operations
        let mut engine = RLogic::new();
        let logic_id = engine.compile(&json!({
            "+": [
                {"pow": [{"var": "nested.value"}, 0.5]}, // This produces NaN
                10
            ]
        })).unwrap();

        // With safe NaN - should work
        let config_safe = RLogicConfig::new().with_safe_nan(true);
        let result_safe = engine.run_with_config(&logic_id, &data, &config_safe).unwrap();
        // NaN should become 0, then 0 + 10 = 10
        assert_eq!(result_safe, json!(10.0));

        // Without safe NaN - null + 10 = 10 (null treated as 0)
        let config_unsafe = RLogicConfig::new().with_safe_nan(false);
        let result_unsafe = engine.run_with_config(&logic_id, &data, &config_unsafe).unwrap();
        assert_eq!(result_unsafe, json!(10.0)); // null + 10 = 10
    }
}
