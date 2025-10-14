use super::Evaluator;
use serde_json::Value;
use super::super::compiled::CompiledLogic;
use super::helpers;

impl Evaluator {
    /// Helper for And/Or logical operations in main eval context
    /// Implements short-circuit evaluation matching JS behavior:
    /// - OR (||): returns first truthy value, or last value if all falsy
    /// - AND (&&): returns first falsy value, or last value if all truthy
    #[inline]
    pub(super) fn eval_and_or(&self, items: &[CompiledLogic], is_and: bool, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        if items.is_empty() {
            return Ok(Value::Bool(is_and));
        }
        
        let mut last_result = Value::Null;
        for item in items {
            last_result = self.evaluate_with_context(item, user_data, internal_context, depth + 1)?;
            let truthy = helpers::is_truthy(&last_result);
            
            // Short-circuit evaluation:
            // AND: return first falsy value
            // OR: return first truthy value
            if is_and && !truthy {
                return Ok(last_result);
            } else if !is_and && truthy {
                return Ok(last_result);
            }
        }
        
        // Return the last evaluated value
        Ok(last_result)
    }

}