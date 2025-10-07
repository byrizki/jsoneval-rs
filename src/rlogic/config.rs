/// Configuration options for RLogic engine
#[derive(Debug, Clone, Copy)]
pub struct RLogicConfig {
    /// Enable caching of evaluation results
    pub enable_cache: bool,
    
    /// Enable data mutation tracking (requires TrackedData wrapper)
    pub enable_tracking: bool,
    
    /// Safely ignore NaN errors in math operations (return 0 instead)
    pub safe_nan_handling: bool,
    
    /// Maximum recursion depth for evaluation
    pub recursion_limit: usize,
}

impl RLogicConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Performance-optimized config (caching enabled, tracking disabled, no NaN safety)
    pub fn performance() -> Self {
        Self {
            enable_cache: true,
            enable_tracking: false,
            safe_nan_handling: false,
            recursion_limit: 1000,
        }
    }
    
    /// Safety-optimized config (all safety features enabled)
    pub fn safe() -> Self {
        Self {
            enable_cache: true,
            enable_tracking: true,
            safe_nan_handling: true,
            recursion_limit: 1000,
        }
    }
    
    /// Minimal config (all features disabled for maximum speed)
    pub fn minimal() -> Self {
        Self {
            enable_cache: false,
            enable_tracking: false,
            safe_nan_handling: false,
            recursion_limit: 1000,
        }
    }
    
    /// Builder pattern methods
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }
    
    pub fn with_tracking(mut self, enable: bool) -> Self {
        self.enable_tracking = enable;
        self
    }
    
    pub fn with_safe_nan(mut self, enable: bool) -> Self {
        self.safe_nan_handling = enable;
        self
    }
    
    pub fn with_recursion_limit(mut self, limit: usize) -> Self {
        self.recursion_limit = limit;
        self
    }
}

impl Default for RLogicConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            enable_tracking: true,
            safe_nan_handling: false,
            recursion_limit: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = RLogicConfig::default();
        assert!(config.enable_cache);
        assert!(config.enable_tracking);
        assert!(!config.safe_nan_handling);
        assert_eq!(config.recursion_limit, 1000);
    }
    
    #[test]
    fn test_performance_config() {
        let config = RLogicConfig::performance();
        assert!(config.enable_cache);
        assert!(!config.enable_tracking);
        assert!(!config.safe_nan_handling);
    }
    
    #[test]
    fn test_safe_config() {
        let config = RLogicConfig::safe();
        assert!(config.enable_cache);
        assert!(config.enable_tracking);
        assert!(config.safe_nan_handling);
    }
    
    #[test]
    fn test_minimal_config() {
        let config = RLogicConfig::minimal();
        assert!(!config.enable_cache);
        assert!(!config.enable_tracking);
        assert!(!config.safe_nan_handling);
    }
    
    #[test]
    fn test_builder_pattern() {
        let config = RLogicConfig::new()
            .with_cache(false)
            .with_tracking(false)
            .with_safe_nan(true)
            .with_recursion_limit(200);
        
        assert!(!config.enable_cache);
        assert!(!config.enable_tracking);
        assert!(config.safe_nan_handling);
        assert_eq!(config.recursion_limit, 200);
    }
}
