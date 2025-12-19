/// Configuration options for RLogic engine
#[derive(Debug, Clone, Copy)]
pub struct RLogicConfig {
    /// Enable data mutation tracking (enabled by default, required for safety)
    /// All data mutations are gated through EvalData when enabled
    pub enable_tracking: bool,
    
    /// Safely ignore NaN errors in math operations (return 0 instead)
    pub safe_nan_handling: bool,
    
    /// Maximum recursion depth for evaluation
    pub recursion_limit: usize,

    /// Timezone offset in minutes from UTC
    pub timezone_offset: Option<i32>,
}

impl RLogicConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Performance-optimized config (tracking disabled, no NaN safety)
    pub fn performance() -> Self {
        Self {
            enable_tracking: false,
            safe_nan_handling: false,
            recursion_limit: 1000,
            timezone_offset: None,
        }
    }
    
    /// Safety-optimized config (all safety features enabled)
    pub fn safe() -> Self {
        Self {
            enable_tracking: true,
            safe_nan_handling: true,
            recursion_limit: 1000,
            timezone_offset: None,
        }
    }
    
    /// Minimal config (all features disabled for maximum speed)
    pub fn minimal() -> Self {
        Self {
            enable_tracking: false,
            safe_nan_handling: false,
            recursion_limit: 1000,
            timezone_offset: None,
        }
    }
    
    /// Builder pattern methods
    
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

    pub fn with_timezone_offset(mut self, offset: i32) -> Self {
        self.timezone_offset = Some(offset);
        self
    }
}

impl Default for RLogicConfig {
    fn default() -> Self {
        Self {
            enable_tracking: true,
            safe_nan_handling: false,
            recursion_limit: 1000,
            timezone_offset: None,
        }
    }
}

