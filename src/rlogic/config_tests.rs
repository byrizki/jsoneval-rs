use super::*;
use serde_json::json;

#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let engine = RLogic::new();
        let config = engine.config();
        
        assert!(config.enable_cache);
        assert!(config.enable_tracking);
        assert!(!config.safe_nan_handling);
        assert_eq!(config.recursion_limit, 1000);
    }
    
    #[test]
    fn test_performance_config() {
        let config = RLogicConfig::performance();
        let engine = RLogic::with_config(config);
        
        assert!(engine.config().enable_cache);
        assert!(!engine.config().enable_tracking);
        assert!(!engine.config().safe_nan_handling);
    }
    
    #[test]
    fn test_safe_config() {
        let config = RLogicConfig::safe();
        let engine = RLogic::with_config(config);
        
        assert!(engine.config().enable_cache);
        assert!(engine.config().enable_tracking);
        assert!(engine.config().safe_nan_handling);
    }
    
    #[test]
    fn test_minimal_config() {
        let config = RLogicConfig::minimal();
        let engine = RLogic::with_config(config);
        
        assert!(!engine.config().enable_cache);
        assert!(!engine.config().enable_tracking);
        assert!(!engine.config().safe_nan_handling);
    }
    
    #[test]
    fn test_custom_config() {
        let config = RLogicConfig::new()
            .with_cache(false)
            .with_tracking(false)
            .with_safe_nan(true)
            .with_recursion_limit(200);
        
        let engine = RLogic::with_config(config);
        
        assert!(!engine.config().enable_cache);
        assert!(!engine.config().enable_tracking);
        assert!(engine.config().safe_nan_handling);
        assert_eq!(engine.config().recursion_limit, 200);
    }
    
    #[test]
    fn test_cache_disabled() {
        let config = RLogicConfig::new().with_cache(false);
        let mut engine = RLogic::with_config(config);
        
        let logic_id = engine.compile(&json!({"+": [2, 3]})).unwrap();
        let data = TrackedData::new(json!({}));
        
        // First evaluation
        let result1 = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result1, json!(5.0));
        
        // Second evaluation - should work even with cache disabled
        let result2 = engine.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result2, json!(5.0));
        
        // Cache stats should show no hits (cache disabled)
        let stats = engine.cache_stats();
        assert_eq!(stats.hits, 0);
    }
    
    #[test]
    fn test_safe_nan_handling() {
        // Without safe NaN handling (default)
        let mut engine_unsafe = RLogic::new();
        let logic_id = engine_unsafe.compile(&json!({"/": [1, 0]})).unwrap();
        let data = TrackedData::new(json!({}));
        
        let result = engine_unsafe.evaluate(&logic_id, &data).unwrap();
        assert_eq!(*result, json!(null)); // Division by zero returns null
        
        // With safe NaN handling
        let config = RLogicConfig::new().with_safe_nan(true);
        let mut engine_safe = RLogic::with_config(config);
        
        // Test NaN from sqrt of negative
        let logic_id2 = engine_safe.compile(&json!({"pow": [-1, 0.5]})).unwrap();
        let result2 = engine_safe.evaluate(&logic_id2, &data).unwrap();
        // With safe_nan_handling, NaN becomes 0
        assert_eq!(*result2, json!(0));
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
        let data = TrackedData::new(json!({}));
        
        let result = engine.evaluate(&logic_id, &data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Recursion limit"));
    }
    
    #[test]
    fn test_evaluate_raw_no_tracking() {
        let config = RLogicConfig::performance(); // tracking disabled
        let mut engine = RLogic::with_config(config);
        
        let logic_id = engine.compile(&json!({"+": [{"var": "a"}, {"var": "b"}]})).unwrap();
        
        // Use raw evaluation (no TrackedData wrapper needed)
        let data = json!({"a": 10, "b": 20});
        let result = engine.evaluate_raw(&logic_id, &data).unwrap();
        
        assert_eq!(result, json!(30.0));
    }
    
    #[test]
    fn test_evaluate_uncached() {
        let config = RLogicConfig::new().with_cache(true);
        let mut engine = RLogic::with_config(config);
        
        let logic_id = engine.compile(&json!({"+": [1, 2]})).unwrap();
        let data = TrackedData::new(json!({}));
        
        // Use uncached evaluation
        let result1 = engine.evaluate_uncached(&logic_id, &data).unwrap();
        assert_eq!(result1, json!(3.0));
        
        // Cache should not be populated
        let stats = engine.cache_stats();
        assert_eq!(stats.size, 0);
    }
    
    #[test]
    fn test_config_affects_evaluation() {
        // Test that config actually affects evaluation behavior
        let data = TrackedData::new(json!({}));
        
        // Config with cache enabled
        let config_cached = RLogicConfig::new().with_cache(true);
        let mut engine_cached = RLogic::with_config(config_cached);
        let logic_id1 = engine_cached.compile(&json!({"+": [1, 2]})).unwrap();
        
        engine_cached.evaluate(&logic_id1, &data).unwrap();
        engine_cached.evaluate(&logic_id1, &data).unwrap();
        
        let stats_cached = engine_cached.cache_stats();
        assert_eq!(stats_cached.hits, 1); // Second call was cached
        
        // Config with cache disabled
        let config_uncached = RLogicConfig::new().with_cache(false);
        let mut engine_uncached = RLogic::with_config(config_uncached);
        let logic_id2 = engine_uncached.compile(&json!({"+": [1, 2]})).unwrap();
        
        engine_uncached.evaluate(&logic_id2, &data).unwrap();
        engine_uncached.evaluate(&logic_id2, &data).unwrap();
        
        let stats_uncached = engine_uncached.cache_stats();
        assert_eq!(stats_uncached.hits, 0); // No caching occurred
    }
}
