use super::{Evaluator, types::*};
use serde_json::Value;
use super::super::compiled::CompiledLogic;
use super::helpers;

impl Evaluator {
    /// Fast path for simple arithmetic without recursion (using f64)
    #[inline(always)]
    pub(super) fn eval_arithmetic_fast(&self, op: ArithOp, items: &[CompiledLogic], user_data: &Value, internal_context: &Value) -> Option<Value> {
        // Only use fast path for simple cases (all literals or vars)
        if items.len() > 20 || items.iter().any(|i| !matches!(i, CompiledLogic::Number(_) | CompiledLogic::Var(_, None))) {
            return None;
        }

        let mut result = 0.0_f64;
        for (idx, item) in items.iter().enumerate() {
            let val = match item {
                CompiledLogic::Number(n) => n.parse::<f64>().unwrap_or(0.0),
                CompiledLogic::Var(name, _) => {
                    // Try internal context first, then user data
                    let v = helpers::get_var(internal_context, name)
                        .or_else(|| helpers::get_var(user_data, name));
                    if let Some(value) = v {
                        helpers::to_f64(value)
                    } else {
                        0.0
                    }
                },
                _ => return None,
            };

            if idx == 0 {
                result = val;
            } else {
                result = match op {
                    ArithOp::Add => result + val,
                    ArithOp::Sub => result - val,
                    ArithOp::Mul => result * val,
                    ArithOp::Div => if val != 0.0 { result / val } else { return Some(Value::Null); },
                };
            }
        }
        Some(self.f64_to_json(result))
    }

    /// Evaluate binary arithmetic operation on two expressions
    #[inline]
    pub(super) fn eval_binary_arith<F>(&self, a: &CompiledLogic, b: &CompiledLogic, f: F, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String>
    where F: FnOnce(f64, f64) -> Option<f64>
    {
        let val_a = self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
        let val_b = self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
        let num_a = helpers::to_f64(&val_a);
        let num_b = helpers::to_f64(&val_b);
        match f(num_a, num_b) {
            Some(result) => Ok(self.f64_to_json(result)),
            None => Ok(Value::Null)
        }
    }

    /// Evaluate array arithmetic with fold operation
    #[inline]
    pub(super) fn eval_array_fold<F>(&self, items: &[CompiledLogic], initial: f64, f: F, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String>
    where F: Fn(f64, f64) -> Option<f64>
    {
        let mut result = initial;
        for item in items {
            let val = self.evaluate_with_context(item, user_data, internal_context, depth + 1)?;
            let num = helpers::to_f64(&val);
            match f(result, num) {
                Some(v) => result = v,
                None => return Ok(Value::Null)
            }
        }
        Ok(self.f64_to_json(result))
    }

    /// Flatten array values for arithmetic operations
    pub(super) fn flatten_array_values(&self, items: &[CompiledLogic], user_data: &Value, internal_context: &Value, depth: usize) -> Result<Vec<f64>, String> {
        let mut values = Vec::new();
        for item in items {
            let val = self.evaluate_with_context(item, user_data, internal_context, depth + 1)?;
            if let Value::Array(arr) = val {
                for elem in arr {
                    values.push(helpers::to_f64(&elem));
                }
            } else {
                values.push(helpers::to_f64(&val));
            }
        }
        Ok(values)
    }
}