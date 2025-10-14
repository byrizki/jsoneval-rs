use super::{Evaluator, types::*};
use serde_json::Value;
use super::super::compiled::CompiledLogic;
use super::helpers;

impl Evaluator {
    /// Execute binary comparison
    #[inline]
    pub(super) fn eval_binary_compare(&self, op: CompOp, a: &CompiledLogic, b: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
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
}