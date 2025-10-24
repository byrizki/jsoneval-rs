use super::Evaluator;
use serde_json::Value;
use super::super::compiled::CompiledLogic;
use super::helpers;

impl Evaluator {
    /// Concatenate string values from items - ZERO-COPY
    #[inline]
    pub(super) fn concat_strings(&self, items: &[CompiledLogic], user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let mut result = String::new();
        for item in items {
            let val = self.evaluate_with_context(item, user_data, internal_context, depth + 1)?;
            result.push_str(&helpers::to_string(&val));
        }
        Ok(Value::String(result))
    }

    /// Extract text from left or right (is_left=true for Left, false for Right) - ZERO-COPY
    pub(super) fn extract_text_side(&self, text_expr: &CompiledLogic, num_expr: Option<&CompiledLogic>, is_left: bool, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let text_val = self.evaluate_with_context(text_expr, user_data, internal_context, depth + 1)?;
        let text = helpers::to_string(&text_val);
        let num_chars = if let Some(n_expr) = num_expr {
            let n_val = self.evaluate_with_context(n_expr, user_data, internal_context, depth + 1)?;
            helpers::to_number(&n_val) as usize
        } else { 1 };

        if is_left {
            Ok(Value::String(text.chars().take(num_chars).collect()))
        } else {
            let chars: Vec<char> = text.chars().collect();
            let start = chars.len().saturating_sub(num_chars);
            Ok(Value::String(chars[start..].iter().collect()))
        }
    }

    /// Evaluate substring operation - ZERO-COPY
    pub(super) fn eval_substr(&self, string_expr: &CompiledLogic, start_expr: &CompiledLogic, length_expr: &Option<Box<CompiledLogic>>, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let string_val = self.evaluate_with_context(string_expr, user_data, internal_context, depth + 1)?;
        let start_val = self.evaluate_with_context(start_expr, user_data, internal_context, depth + 1)?;

        let s = helpers::to_string(&string_val);
        let start = helpers::to_number(&start_val) as i32;

        let start_idx = if start < 0 {
            (s.len() as i32 + start).max(0) as usize
        } else {
            start.min(s.len() as i32) as usize
        };

        if let Some(len_expr) = length_expr {
            let length_val = self.evaluate_with_context(len_expr, user_data, internal_context, depth + 1)?;
            let length = helpers::to_number(&length_val) as usize;
            let end_idx = (start_idx + length).min(s.len());
            Ok(Value::String(s[start_idx..end_idx].to_string()))
        } else {
            Ok(Value::String(s[start_idx..].to_string()))
        }
    }

    /// Evaluate search operation - ZERO-COPY
    pub(super) fn eval_search(&self, find_expr: &CompiledLogic, within_expr: &CompiledLogic, start_expr: &Option<Box<CompiledLogic>>, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let find_val = self.evaluate_with_context(find_expr, user_data, internal_context, depth + 1)?;
        let within_val = self.evaluate_with_context(within_expr, user_data, internal_context, depth + 1)?;

        if let (Value::String(find), Value::String(within)) = (&find_val, &within_val) {
            let start = if let Some(start_e) = start_expr {
                let start_val = self.evaluate_with_context(start_e, user_data, internal_context, depth + 1)?;
                (helpers::to_number(&start_val) as usize).saturating_sub(1)
            } else {
                0
            };

            if let Some(pos) = within.to_lowercase()[start..].find(&find.to_lowercase()) {
                Ok(self.f64_to_json((pos + start + 1) as f64))
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(Value::Null)
        }
    }

    /// Evaluate mid operation (substring from position with length) - ZERO-COPY
    pub(super) fn eval_mid(&self, text_expr: &CompiledLogic, start_expr: &CompiledLogic, num_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let text_val = self.evaluate_with_context(text_expr, user_data, internal_context, depth + 1)?;
        let start_val = self.evaluate_with_context(start_expr, user_data, internal_context, depth + 1)?;
        let num_val = self.evaluate_with_context(num_expr, user_data, internal_context, depth + 1)?;

        let text = helpers::to_string(&text_val);
        let start = (helpers::to_number(&start_val) as usize).saturating_sub(1);
        let num_chars = helpers::to_number(&num_val) as usize;

        let chars: Vec<char> = text.chars().collect();
        let end = (start + num_chars).min(chars.len());
        Ok(Value::String(chars[start..end].iter().collect()))
    }

    /// Evaluate split text operation - ZERO-COPY
    pub(super) fn eval_split_text(&self, value_expr: &CompiledLogic, sep_expr: &CompiledLogic, index_expr: &Option<Box<CompiledLogic>>, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let value_val = self.evaluate_with_context(value_expr, user_data, internal_context, depth + 1)?;
        let sep_val = self.evaluate_with_context(sep_expr, user_data, internal_context, depth + 1)?;

        let text = helpers::to_string(&value_val);
        let separator = helpers::to_string(&sep_val);
        let index = if let Some(idx_expr) = index_expr {
            let idx_val = self.evaluate_with_context(idx_expr, user_data, internal_context, depth + 1)?;
            helpers::to_number(&idx_val) as usize
        } else {
            0
        };

        let parts: Vec<&str> = text.split(&separator).collect();
        Ok(Value::String(parts.get(index).unwrap_or(&"").to_string()))
    }

    /// Format number with prefix, suffix, decimals, and thousands separator (Excel-like TEXT function)
    pub(super) fn eval_string_format(
        &self,
        value_expr: &CompiledLogic,
        decimals_expr: &Option<Box<CompiledLogic>>,
        prefix_expr: &Option<Box<CompiledLogic>>,
        suffix_expr: &Option<Box<CompiledLogic>>,
        thousands_sep_expr: &Option<Box<CompiledLogic>>,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let value_val = self.evaluate_with_context(value_expr, user_data, internal_context, depth + 1)?;
        let num = helpers::to_f64(&value_val);
        
        // Get decimals (default 0)
        let decimals = if let Some(dec_expr) = decimals_expr {
            let dec_val = self.evaluate_with_context(dec_expr, user_data, internal_context, depth + 1)?;
            helpers::to_number(&dec_val) as usize
        } else {
            0
        };
        
        // Get prefix (default empty)
        let prefix = if let Some(pre_expr) = prefix_expr {
            let pre_val = self.evaluate_with_context(pre_expr, user_data, internal_context, depth + 1)?;
            helpers::to_string(&pre_val)
        } else {
            String::new()
        };
        
        // Get suffix (default empty)
        let suffix = if let Some(suf_expr) = suffix_expr {
            let suf_val = self.evaluate_with_context(suf_expr, user_data, internal_context, depth + 1)?;
            helpers::to_string(&suf_val)
        } else {
            String::new()
        };
        
        // Get thousands separator (default ",")
        let thousands_sep = if let Some(sep_expr) = thousands_sep_expr {
            let sep_val = self.evaluate_with_context(sep_expr, user_data, internal_context, depth + 1)?;
            helpers::to_string(&sep_val)
        } else {
            ",".to_string()
        };
        
        // Format the number
        let formatted = if decimals == 0 {
            let rounded = num.round() as i64;
            format_with_thousands(rounded.to_string(), &thousands_sep)
        } else {
            let formatted_num = format!("{:.prec$}", num, prec = decimals);
            // Split at decimal point
            if let Some(dot_idx) = formatted_num.find('.') {
                let integer_part = &formatted_num[..dot_idx];
                let decimal_part = &formatted_num[dot_idx..];
                format_with_thousands(integer_part.to_string(), &thousands_sep) + decimal_part
            } else {
                format_with_thousands(formatted_num, &thousands_sep)
            }
        };
        
        Ok(Value::String(format!("{}{}{}", prefix, formatted, suffix)))
    }

    /// Evaluate split value operation - ZERO-COPY
    pub(super) fn eval_split_value(&self, string_expr: &CompiledLogic, sep_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let string_val = self.evaluate_with_context(string_expr, user_data, internal_context, depth + 1)?;
        let sep_val = self.evaluate_with_context(sep_expr, user_data, internal_context, depth + 1)?;

        let text = helpers::to_string(&string_val);
        let separator = helpers::to_string(&sep_val);
        let parts: Vec<Value> = text.split(&separator)
            .map(|s| Value::String(s.to_string()))
            .collect();
        Ok(Value::Array(parts))
    }

    /// Evaluate length operation - ZERO-COPY
    pub(super) fn eval_length(&self, expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let val = self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
        let len = match &val {
            Value::String(s) => s.len(),
            Value::Array(arr) => arr.len(),
            Value::Object(obj) => obj.len(),
            _ => 0,
        };
        Ok(self.f64_to_json(len as f64))
    }

    /// Evaluate len operation (string length) - ZERO-COPY
    pub(super) fn eval_len(&self, expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let val = self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
        let s = helpers::to_string(&val);
        Ok(self.f64_to_json(s.len() as f64))
    }
}

/// Helper function to format a number string with thousands separator
fn format_with_thousands(num_str: String, separator: &str) -> String {
    if separator.is_empty() {
        return num_str;
    }
    
    let chars: Vec<char> = num_str.chars().collect();
    let mut result = String::new();
    let len = chars.len();
    
    for (i, ch) in chars.iter().enumerate() {
        if *ch == '-' || *ch == '+' {
            result.push(*ch);
            continue;
        }
        
        result.push(*ch);
        
        // Add separator every 3 digits from the right
        let remaining = len - i - 1;
        if remaining > 0 && remaining % 3 == 0 {
            result.push_str(separator);
        }
    }
    
    result
}
