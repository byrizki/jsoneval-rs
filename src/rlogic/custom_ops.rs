use serde_json::Value;
use chrono::{Datelike, NaiveDate};

/// Helper functions for custom operators
pub struct CustomOps;

impl CustomOps {
    /// SEARCH - Find text within text (case insensitive, 1-indexed)
    pub fn search(find_text: &str, within_text: &str, start_num: Option<f64>) -> Option<f64> {
        let start = start_num.unwrap_or(1.0).max(1.0) as usize - 1;
        let found_at = within_text
            .to_lowercase()
            .get(start..)?
            .find(&find_text.to_lowercase())?;
        Some((found_at + start + 1) as f64)
    }
    
    /// Calculate days between two dates (Excel serial number style)
    pub fn days_between(end_date: &str, start_date: &str) -> Option<f64> {
        let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%dT%H:%M:%S%.fZ")
            .ok()
            .or_else(|| NaiveDate::parse_from_str(end_date, "%Y-%m-%d").ok())?;
        let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%dT%H:%M:%S%.fZ")
            .ok()
            .or_else(|| NaiveDate::parse_from_str(start_date, "%Y-%m-%d").ok())?;
        
        Some((end - start).num_days() as f64)
    }
    
    /// Extract year from date string
    pub fn year_from_date(date_str: &str) -> Option<f64> {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ")
            .ok()
            .or_else(|| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
            .map(|d| d.year() as f64)
    }
    
    /// Extract month from date string (1-12)
    pub fn month_from_date(date_str: &str) -> Option<f64> {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ")
            .ok()
            .or_else(|| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
            .map(|d| d.month() as f64)
    }
    
    /// Extract day from date string
    pub fn day_from_date(date_str: &str) -> Option<f64> {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ")
            .ok()
            .or_else(|| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
            .map(|d| d.day() as f64)
    }
    
    /// Create date from year, month, day
    pub fn create_date(year: f64, month: f64, day: f64) -> Option<String> {
        NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
            .map(|d| {
                let mut result = String::with_capacity(24);
                result.push_str(&d.format("%Y-%m-%d").to_string());
                result.push_str("T00:00:00.000Z");
                result
            })
    }
}

pub fn to_number(value: &Value) -> f64 {
    match value {
        Value::Bool(b) => if *b { 1.0 } else { 0.0 },
        Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        Value::Array(arr) if arr.len() == 1 => to_number(&arr[0]),
        _ => 0.0,
    }
}

pub fn to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

pub fn is_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        _ => false,
    }
}
