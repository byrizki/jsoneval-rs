use crate::jsoneval::types::{ValidationError, ValidationResult};
use indexmap::IndexMap;
use serde_json::Value;

/// Cached validation result for a single field
#[derive(Clone, Debug)]
pub struct CachedFieldValidation {
    /// Field value at last validation
    pub field_data: Value,
    /// Visibility state at last validation
    pub is_hidden: bool,
    /// Rule evaluation snapshot (to detect changes in dynamic $evaluation rules)
    pub rules_snapshot: Value,
    /// Cached error if invalid, None if valid
    pub error: Option<ValidationError>,
}

/// Cache for validation results (whole-form and per-field)
#[derive(Clone, Default, Debug)]
pub struct ValidationCache {
    /// Last input data string for whole-form caching
    pub last_data_str: Option<String>,
    /// Last context string for whole-form caching
    pub last_context_str: Option<String>,
    /// Last full validation result
    pub last_result: Option<ValidationResult>,
    /// Per-field validation cache
    pub field_cache: IndexMap<String, CachedFieldValidation>,
}

impl ValidationCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the full validation result can be returned immediately
    #[inline]
    pub fn get_cached_full_result(
        &self,
        data: &str,
        context: Option<&str>,
    ) -> Option<ValidationResult> {
        let last_data = self.last_data_str.as_deref()?;
        if last_data != data {
            return None;
        }

        let ctx_matches = match (self.last_context_str.as_deref(), context) {
            (None, None) => true,
            (Some(""), None) | (None, Some("")) => true,
            (Some("{}"), None) | (None, Some("{}")) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        };

        if ctx_matches {
            self.last_result.clone()
        } else {
            None
        }
    }

    /// Check if a single field has a valid cache entry matching current state
    #[inline]
    pub fn check_field_cache(
        &self,
        field_path: &str,
        field_data: &Value,
        is_hidden: bool,
        rules: &Value,
    ) -> Option<Option<ValidationError>> {
        let cached = self.field_cache.get(field_path)?;
        if cached.is_hidden == is_hidden
            && &cached.field_data == field_data
            && &cached.rules_snapshot == rules
        {
            Some(cached.error.clone())
        } else {
            None
        }
    }

    /// Update cache for a single field
    #[inline]
    pub fn update_field(
        &mut self,
        field_path: String,
        field_data: Value,
        is_hidden: bool,
        rules_snapshot: Value,
        error: Option<ValidationError>,
    ) {
        self.field_cache.insert(
            field_path,
            CachedFieldValidation {
                field_data,
                is_hidden,
                rules_snapshot,
                error,
            },
        );
    }

    /// Save full validation result
    #[inline]
    pub fn save_full_result(
        &mut self,
        data_str: String,
        context_str: Option<String>,
        result: ValidationResult,
    ) {
        self.last_data_str = Some(data_str);
        self.last_context_str = context_str;
        self.last_result = Some(result);
    }

    /// Invalidate whole-result cache (e.g. on partial path validation)
    #[inline]
    pub fn invalidate_full_result(&mut self) {
        self.last_data_str = None;
        self.last_context_str = None;
        self.last_result = None;
    }

    /// Clear all cached validation state
    #[inline]
    pub fn clear(&mut self) {
        self.last_data_str = None;
        self.last_context_str = None;
        self.last_result = None;
        self.field_cache.clear();
    }
}
