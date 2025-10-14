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

    /// Apply rounding function: 0=round, 1=ceil, 2=floor
    #[inline]
    pub(super) fn apply_round(&self, expr: &CompiledLogic, round_type: u8, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let val = self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
        let num = helpers::to_f64(&val);
        let result = match round_type {
            0 => num.round(),
            1 => num.ceil(),
            2 => num.floor(),
            _ => num
        };
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