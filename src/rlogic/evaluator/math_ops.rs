use super::Evaluator;
use serde_json::Value;
use super::super::compiled::CompiledLogic;
use super::helpers;

impl Evaluator {
    /// Find min or max in a list
    pub(super) fn eval_min_max(&self, items: &[CompiledLogic], is_max: bool, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        if items.is_empty() {
            return Ok(Value::Null);
        }
        let first_val = self.evaluate_with_context(&items[0], user_data, internal_context, depth + 1)?;
        let mut result = helpers::to_f64(&first_val);
        for item in &items[1..] {
            let val = self.evaluate_with_context(item, user_data, internal_context, depth + 1)?;
            let num = helpers::to_f64(&val);
            if is_max {
                if num > result { result = num; }
            } else {
                if num < result { result = num; }
            }
        }
        Ok(self.f64_to_json(result))
    }

    /// Apply rounding function with optional decimal places (Excel-compatible)
    /// round_type: 0=round, 1=roundup, 2=rounddown
    #[inline]
    pub(super) fn apply_round(&self, expr: &CompiledLogic, decimals_expr: &Option<Box<CompiledLogic>>, round_type: u8, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let val = self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
        let num = helpers::to_f64(&val);
        
        let decimals = if let Some(dec_expr) = decimals_expr {
            let dec_val = self.evaluate_with_context(dec_expr, user_data, internal_context, depth + 1)?;
            helpers::to_number(&dec_val) as i32
        } else {
            0
        };
        
        let result = if decimals == 0 {
            // Backward compatible: round to integer
            match round_type {
                0 => num.round(),
                1 => num.ceil(),
                2 => num.floor(),
                _ => num
            }
        } else if decimals > 0 {
            // Round to decimal places (Excel ROUND)
            let multiplier = 10f64.powi(decimals);
            match round_type {
                0 => (num * multiplier).round() / multiplier,
                1 => (num * multiplier).ceil() / multiplier,
                2 => (num * multiplier).floor() / multiplier,
                _ => num
            }
        } else {
            // Negative decimals: round to left of decimal (Excel ROUND with negative num_digits)
            let divider = 10f64.powi(-decimals);
            match round_type {
                0 => (num / divider).round() * divider,
                1 => (num / divider).ceil() * divider,
                2 => (num / divider).floor() * divider,
                _ => num
            }
        };
        
        Ok(self.f64_to_json(result))
    }
    
    /// Excel CEILING function - rounds up to nearest multiple of significance
    pub(super) fn eval_ceiling(&self, expr: &CompiledLogic, significance_expr: &Option<Box<CompiledLogic>>, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let val = self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
        let num = helpers::to_f64(&val);
        
        let significance = if let Some(sig_expr) = significance_expr {
            let sig_val = self.evaluate_with_context(sig_expr, user_data, internal_context, depth + 1)?;
            helpers::to_f64(&sig_val)
        } else {
            1.0
        };
        
        if significance == 0.0 {
            return Ok(Value::Number(serde_json::Number::from(0)));
        }
        
        let result = (num / significance).ceil() * significance;
        Ok(self.f64_to_json(result))
    }
    
    /// Excel FLOOR function - rounds down to nearest multiple of significance
    pub(super) fn eval_floor(&self, expr: &CompiledLogic, significance_expr: &Option<Box<CompiledLogic>>, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let val = self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
        let num = helpers::to_f64(&val);
        
        let significance = if let Some(sig_expr) = significance_expr {
            let sig_val = self.evaluate_with_context(sig_expr, user_data, internal_context, depth + 1)?;
            helpers::to_f64(&sig_val)
        } else {
            1.0
        };
        
        if significance == 0.0 {
            return Ok(Value::Number(serde_json::Number::from(0)));
        }
        
        let result = (num / significance).floor() * significance;
        Ok(self.f64_to_json(result))
    }
    
    /// Excel TRUNC function - truncates to specified decimals (no rounding)
    pub(super) fn eval_trunc(&self, expr: &CompiledLogic, decimals_expr: &Option<Box<CompiledLogic>>, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let val = self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
        let num = helpers::to_f64(&val);
        
        let decimals = if let Some(dec_expr) = decimals_expr {
            let dec_val = self.evaluate_with_context(dec_expr, user_data, internal_context, depth + 1)?;
            helpers::to_number(&dec_val) as i32
        } else {
            0
        };
        
        let result = if decimals == 0 {
            num.trunc()
        } else if decimals > 0 {
            let multiplier = 10f64.powi(decimals);
            (num * multiplier).trunc() / multiplier
        } else {
            let divider = 10f64.powi(-decimals);
            (num / divider).trunc() * divider
        };
        
        Ok(self.f64_to_json(result))
    }
    
    /// Excel MROUND function - rounds to nearest multiple
    pub(super) fn eval_mround(&self, value_expr: &CompiledLogic, multiple_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let val = self.evaluate_with_context(value_expr, user_data, internal_context, depth + 1)?;
        let num = helpers::to_f64(&val);
        
        let multiple_val = self.evaluate_with_context(multiple_expr, user_data, internal_context, depth + 1)?;
        let multiple = helpers::to_f64(&multiple_val);
        
        if multiple == 0.0 {
            return Ok(Value::Number(serde_json::Number::from(0)));
        }
        
        let result = (num / multiple).round() * multiple;
        Ok(self.f64_to_json(result))
    }

    /// Evaluate unary math operation on expression
    #[inline]
    pub(super) fn eval_unary_math<F>(&self, expr: &CompiledLogic, f: F, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String>
    where F: FnOnce(f64) -> f64
    {
        let val = self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
        let num = helpers::to_f64(&val);
        Ok(self.f64_to_json(f(num)))
    }

    /// Evaluate power operation
    pub(super) fn eval_pow(&self, base_expr: &CompiledLogic, exp_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        self.eval_binary_arith(base_expr, exp_expr, |a, b| Some(a.powf(b)), user_data, internal_context, depth)
    }
}