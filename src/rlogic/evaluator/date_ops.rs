use super::Evaluator;
use serde_json::Value;
use super::super::compiled::CompiledLogic;
use super::helpers;
use chrono::Datelike;

impl Evaluator {
    /// Unwrap single-element arrays (common pattern: [{"TODAY": []}])
    #[inline]
    fn unwrap_array(val: Value) -> Value {
        if let Value::Array(arr) = &val {
            if arr.len() == 1 {
                return arr[0].clone();
            }
        }
        val
    }

    /// Parse date string with fallback formats
    #[inline]
    pub(super) fn parse_date(&self, date_str: &str) -> Option<chrono::NaiveDate> {
        use chrono::NaiveDate;
        NaiveDate::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ")
            .ok()
            .or_else(|| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
    }

    /// Extract date component (year/month/day) - ZERO-COPY
    pub(super) fn extract_date_component(&self, expr: &CompiledLogic, component: &str, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let val = self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
        let val = Self::unwrap_array(val);
        
        if let Value::String(date_str) = &val {
            if let Some(d) = self.parse_date(date_str) {
                let value = match component {
                    "year" => d.year() as f64,
                    "month" => d.month() as f64,
                    "day" => d.day() as f64,
                    _ => return Ok(Value::Null),
                };
                return Ok(self.f64_to_json(value));
            }
        }
        Ok(Value::Null)
    }

    /// Evaluate Today operation
    pub(super) fn eval_today(&self) -> Result<Value, String> {
        let now = chrono::Utc::now();
        Ok(Value::String(helpers::build_iso_date_string(now.date_naive())))
    }

    /// Evaluate Now operation
    pub(super) fn eval_now(&self) -> Result<Value, String> {
        let now = chrono::Utc::now();
        Ok(Value::String(now.to_rfc3339()))
    }

    /// Evaluate Days operation - ZERO-COPY
    pub(super) fn eval_days(&self, end_expr: &CompiledLogic, start_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let end_val = self.evaluate_with_context(end_expr, user_data, internal_context, depth + 1)?;
        let start_val = self.evaluate_with_context(start_expr, user_data, internal_context, depth + 1)?;
        
        let end_val = Self::unwrap_array(end_val);
        let start_val = Self::unwrap_array(start_val);

        if let (Value::String(end), Value::String(start)) = (&end_val, &start_val) {
            if let (Some(e), Some(s)) = (self.parse_date(end), self.parse_date(start)) {
                return Ok(self.f64_to_json((e - s).num_days() as f64));
            }
        }
        Ok(Value::Null)
    }

    /// Evaluate Date operation with JavaScript-compatible normalization
    /// Handles overflow/underflow of day values (e.g., day=-16 subtracts from month)
    pub(super) fn eval_date(&self, year_expr: &CompiledLogic, month_expr: &CompiledLogic, day_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let year_val = self.evaluate_with_context(year_expr, user_data, internal_context, depth + 1)?;
        let month_val = self.evaluate_with_context(month_expr, user_data, internal_context, depth + 1)?;
        let day_val = self.evaluate_with_context(day_expr, user_data, internal_context, depth + 1)?;

        let year = helpers::to_number(&year_val) as i32;
        let month = helpers::to_number(&month_val) as i32;
        let day = helpers::to_number(&day_val) as i32;

        use chrono::{NaiveDate, Duration};
        
        // JavaScript-compatible date normalization:
        // Start with year/month, then add days offset
        // This allows negative days to roll back months/years
        
        // First normalize month (can also be out of range)
        let mut normalized_year = year;
        let mut normalized_month = month;
        
        // Handle month overflow/underflow (JS allows month=-1, month=13, etc.)
        if normalized_month < 1 {
            let months_back = (1 - normalized_month) / 12 + 1;
            normalized_year -= months_back;
            normalized_month += months_back * 12;
        } else if normalized_month > 12 {
            let months_forward = (normalized_month - 1) / 12;
            normalized_year += months_forward;
            normalized_month = ((normalized_month - 1) % 12) + 1;
        }
        
        // Create base date at day 1 of the normalized month
        if let Some(base_date) = NaiveDate::from_ymd_opt(normalized_year, normalized_month as u32, 1) {
            // Add (day - 1) days to get final date
            // This handles negative days and days > month length automatically
            if let Some(final_date) = base_date.checked_add_signed(Duration::days((day - 1) as i64)) {
                Ok(Value::String(helpers::build_iso_date_string(final_date)))
            } else {
                // Date overflow (e.g., year > 9999 or < -9999)
                Ok(Value::Null)
            }
        } else {
            // Invalid base date
            Ok(Value::Null)
        }
    }

    /// Evaluate YearFrac operation - ZERO-COPY
    pub(super) fn eval_year_frac(&self, start_expr: &CompiledLogic, end_expr: &CompiledLogic, basis_expr: &Option<Box<CompiledLogic>>, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let start_val = self.evaluate_with_context(start_expr, user_data, internal_context, depth + 1)?;
        let end_val = self.evaluate_with_context(end_expr, user_data, internal_context, depth + 1)?;
        
        let start_val = Self::unwrap_array(start_val);
        let end_val = Self::unwrap_array(end_val);
        
        let basis = if let Some(b_expr) = basis_expr {
            let b_val = self.evaluate_with_context(b_expr, user_data, internal_context, depth + 1)?;
            helpers::to_number(&b_val) as i32
        } else { 0 };

        if let (Value::String(start_str), Value::String(end_str)) = (&start_val, &end_val) {
            if let (Some(start), Some(end)) = (self.parse_date(start_str), self.parse_date(end_str)) {
                let days = (end - start).num_days() as f64;
                let result = match basis {
                    0 => days / 360.0,
                    1 => days / 365.25,
                    2 => days / 360.0,
                    3 => days / 365.0,
                    4 => days / 360.0,
                    _ => days / 365.0,
                };
                return Ok(self.f64_to_json(result));
            }
        }
        Ok(Value::Null)
    }

    /// Evaluate DateDif operation - ZERO-COPY
    pub(super) fn eval_date_dif(&self, start_expr: &CompiledLogic, end_expr: &CompiledLogic, unit_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let start_val = self.evaluate_with_context(start_expr, user_data, internal_context, depth + 1)?;
        let end_val = self.evaluate_with_context(end_expr, user_data, internal_context, depth + 1)?;
        let unit_val = self.evaluate_with_context(unit_expr, user_data, internal_context, depth + 1)?;
        
        let start_val = Self::unwrap_array(start_val);
        let end_val = Self::unwrap_array(end_val);
        let unit_val = Self::unwrap_array(unit_val);

        if let (Value::String(start_str), Value::String(end_str), Value::String(unit)) =
            (&start_val, &end_val, &unit_val) {
            if let (Some(start), Some(end)) = (self.parse_date(start_str), self.parse_date(end_str)) {
                let result = match unit.to_uppercase().as_str() {
                    "D" => (end - start).num_days() as f64,
                    "M" => {
                        let years = end.year() - start.year();
                        let months = end.month() as i32 - start.month() as i32;
                        let mut total_months = years * 12 + months;
                        if end.day() < start.day() {
                            total_months -= 1;
                        }
                        total_months as f64
                    }
                    "Y" => {
                        let mut years = end.year() - start.year();
                        if end.month() < start.month() ||
                           (end.month() == start.month() && end.day() < start.day()) {
                            years -= 1;
                        }
                        years as f64
                    }
                    "MD" => {
                        if start.day() <= end.day() {
                            (end.day() - start.day()) as f64
                        } else {
                            let days_in_month = 30u32; // Simplified
                            (days_in_month as i32 - (start.day() as i32 - end.day() as i32)) as f64
                        }
                    }
                    "YM" => {
                        let months = end.month() as i32 - start.month() as i32;
                        let mut result = if months < 0 { months + 12 } else { months };
                        if end.day() < start.day() {
                            result -= 1;
                            if result < 0 {
                                result += 12;
                            }
                        }
                        result as f64
                    }
                    "YD" => {
                        let mut temp_start = start.with_year(end.year()).unwrap_or(start);
                        if temp_start > end {
                            temp_start = start.with_year(end.year() - 1).unwrap_or(start);
                        }
                        (end - temp_start).num_days() as f64
                    }
                    _ => return Ok(Value::Null),
                };

                Ok(self.f64_to_json(result))
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }
    }
}
