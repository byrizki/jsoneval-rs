use super::super::compiled::CompiledLogic;
use super::helpers;
use super::{types::*, Evaluator};
use serde_json::Value;

impl Evaluator {
    /// Execute binary comparison
    #[inline]
    pub(super) fn eval_binary_compare(
        &self,
        op: CompOp,
        a: &CompiledLogic,
        b: &CompiledLogic,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        if let (Some(na), Some(nb)) = (
            self.eval_f64(a, user_data, internal_context, depth + 1)?,
            self.eval_f64(b, user_data, internal_context, depth + 1)?,
        ) {
            let result = match op {
                CompOp::Eq | CompOp::StrictEq => na == nb,
                CompOp::Ne | CompOp::StrictNe => na != nb,
                CompOp::Lt => na < nb,
                CompOp::Le => na <= nb,
                CompOp::Gt => na > nb,
                CompOp::Ge => na >= nb,
            };
            return Ok(Value::Bool(result));
        }
        let val_a = self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
        let val_b = self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
        let result = match op {
            CompOp::Eq => helpers::loose_equal(&val_a, &val_b),
            CompOp::StrictEq => val_a == val_b,
            CompOp::Ne => !helpers::loose_equal(&val_a, &val_b),
            CompOp::StrictNe => val_a != val_b,
            CompOp::Lt => helpers::compare(&val_a, &val_b) < 0.0,
            CompOp::Le => helpers::compare(&val_a, &val_b) <= 0.0,
            CompOp::Gt => helpers::compare(&val_a, &val_b) > 0.0,
            CompOp::Ge => helpers::compare(&val_a, &val_b) >= 0.0,
        };
        Ok(Value::Bool(result))
    }

    /// Fast unboxed evaluation of boolean truthiness without serde_json::Value allocations
    #[inline]
    pub(crate) fn eval_truthy(
        &self,
        logic: &CompiledLogic,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<bool, String> {
        if depth > self.config.recursion_limit {
            return Err("Recursion limit exceeded".to_string());
        }

        match logic {
            CompiledLogic::Bool(b) => Ok(*b),
            CompiledLogic::Null => Ok(false),
            CompiledLogic::Number(n) => Ok(*n != 0.0),
            CompiledLogic::String(s) => Ok(!s.is_empty()),
            CompiledLogic::Not(expr) => {
                let inner = self.eval_truthy(expr, user_data, internal_context, depth + 1)?;
                Ok(!inner)
            }
            CompiledLogic::And(items) => {
                for item in items {
                    if !self.eval_truthy(item, user_data, internal_context, depth + 1)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            CompiledLogic::Or(items) => {
                for item in items {
                    if self.eval_truthy(item, user_data, internal_context, depth + 1)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            CompiledLogic::Equal(a, b) => {
                if let (Some(na), Some(nb)) = (
                    self.eval_f64(a, user_data, internal_context, depth + 1)?,
                    self.eval_f64(b, user_data, internal_context, depth + 1)?,
                ) {
                    return Ok(na == nb);
                }
                let val_a =
                    self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
                let val_b =
                    self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
                Ok(helpers::loose_equal(&val_a, &val_b))
            }
            CompiledLogic::StrictEqual(a, b) => {
                if let (Some(na), Some(nb)) = (
                    self.eval_f64(a, user_data, internal_context, depth + 1)?,
                    self.eval_f64(b, user_data, internal_context, depth + 1)?,
                ) {
                    return Ok(na == nb);
                }
                let val_a =
                    self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
                let val_b =
                    self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
                Ok(val_a == val_b)
            }
            CompiledLogic::NotEqual(a, b) => {
                if let (Some(na), Some(nb)) = (
                    self.eval_f64(a, user_data, internal_context, depth + 1)?,
                    self.eval_f64(b, user_data, internal_context, depth + 1)?,
                ) {
                    return Ok(na != nb);
                }
                let val_a =
                    self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
                let val_b =
                    self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
                Ok(!helpers::loose_equal(&val_a, &val_b))
            }
            CompiledLogic::StrictNotEqual(a, b) => {
                if let (Some(na), Some(nb)) = (
                    self.eval_f64(a, user_data, internal_context, depth + 1)?,
                    self.eval_f64(b, user_data, internal_context, depth + 1)?,
                ) {
                    return Ok(na != nb);
                }
                let val_a =
                    self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
                let val_b =
                    self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
                Ok(val_a != val_b)
            }
            CompiledLogic::LessThan(a, b) => {
                if let (Some(na), Some(nb)) = (
                    self.eval_f64(a, user_data, internal_context, depth + 1)?,
                    self.eval_f64(b, user_data, internal_context, depth + 1)?,
                ) {
                    return Ok(na < nb);
                }
                let val_a =
                    self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
                let val_b =
                    self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
                Ok(helpers::compare(&val_a, &val_b) < 0.0)
            }
            CompiledLogic::LessThanOrEqual(a, b) => {
                if let (Some(na), Some(nb)) = (
                    self.eval_f64(a, user_data, internal_context, depth + 1)?,
                    self.eval_f64(b, user_data, internal_context, depth + 1)?,
                ) {
                    return Ok(na <= nb);
                }
                let val_a =
                    self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
                let val_b =
                    self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
                Ok(helpers::compare(&val_a, &val_b) <= 0.0)
            }
            CompiledLogic::GreaterThan(a, b) => {
                if let (Some(na), Some(nb)) = (
                    self.eval_f64(a, user_data, internal_context, depth + 1)?,
                    self.eval_f64(b, user_data, internal_context, depth + 1)?,
                ) {
                    return Ok(na > nb);
                }
                let val_a =
                    self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
                let val_b =
                    self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
                Ok(helpers::compare(&val_a, &val_b) > 0.0)
            }
            CompiledLogic::GreaterThanOrEqual(a, b) => {
                if let (Some(na), Some(nb)) = (
                    self.eval_f64(a, user_data, internal_context, depth + 1)?,
                    self.eval_f64(b, user_data, internal_context, depth + 1)?,
                ) {
                    return Ok(na >= nb);
                }
                let val_a =
                    self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
                let val_b =
                    self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
                Ok(helpers::compare(&val_a, &val_b) >= 0.0)
            }
            _ => {
                let val = self.evaluate_with_context(logic, user_data, internal_context, depth)?;
                Ok(helpers::is_truthy(&val))
            }
        }
    }
}
