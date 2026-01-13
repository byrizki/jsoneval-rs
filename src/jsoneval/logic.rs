use super::JSONEval;
use crate::jsoneval::json_parser;
use crate::rlogic::{
    CompiledLogicId, Evaluator, compiled_logic_store
};
use crate::utils::clean_float_noise;
use serde_json::Value;

impl JSONEval {
    /// Run pre-compiled logic against current data
    pub fn run_logic(
        &mut self, 
        logic_id: CompiledLogicId,
        data: Option<&Value>,
        context: Option<&Value>
    ) -> Result<Value, String> {
        // Get compiled logic from global store
        let compiled_logic = compiled_logic_store::get_compiled_logic(logic_id)
            .ok_or_else(|| format!("Compiled logic ID {:?} not found in store", logic_id))?;

        // If custom data/context provided, update eval_data
        let run_data = if let Some(input_data) = data {
             let context_value = context.unwrap_or(&self.context);
             self.eval_data.replace_data_and_context(input_data.clone(), context_value.clone());
             self.eval_data.data()
        } else {
             self.eval_data.data()
        };

        // Create an evaluator and run the pre-compiled logic with zero-clone pattern
        let evaluator = Evaluator::new();
        let result = evaluator.evaluate(&compiled_logic, run_data)
             .map_err(|e| format!("Execution error: {}", e))?;
            
        Ok(clean_float_noise(result))
    }

    /// Compile a logic expression from a JSON string and store it globally
    pub fn compile_logic(&self, logic_str: &str) -> Result<CompiledLogicId, String> {
        compiled_logic_store::compile_logic(logic_str)
    }

    /// Compile a logic expression from a Value and store it globally
    pub fn compile_logic_value(&self, logic: &Value) -> Result<CompiledLogicId, String> {
        compiled_logic_store::compile_logic_value(logic)
    }

    /// Compile and run logic in one go (convenience method)
    pub fn compile_and_run_logic(
        &mut self, 
        logic_str: &str, 
        data: Option<&str>, 
        context: Option<&str>
    ) -> Result<Value, String> {
        let id = self.compile_logic(logic_str)?;
        
        // Parse data and context if provided
        let data_value = if let Some(d) = data {
            Some(json_parser::parse_json_str(d)?)
        } else {
            None
        };
        
        let context_value = if let Some(c) = context {
            Some(json_parser::parse_json_str(c)?)
        } else {
            None
        };
        
        self.run_logic(id, data_value.as_ref(), context_value.as_ref())
        }
}

/// Run logic evaluation directly without schema/form state
/// 
/// This is a "pure" evaluation that doesn't rely on JSONEval instance state.
/// It creates a temporary evaluator and runs the logic against the provided data.
/// 
/// # Arguments
/// * `logic_str` - JSON logic expression string
/// * `data_str` - Data JSON string (optional)
/// * `context_str` - Context JSON string (optional). Will be merged into data under "$context" key.
pub fn evaluate_logic_pure(
    logic_str: &str,
    data_str: Option<&str>,
    context_str: Option<&str>,
) -> Result<Value, String> {
    // Compile logic
    let logic_value = json_parser::parse_json_str(logic_str)
        .map_err(|e| format!("Invalid logic JSON: {}", e))?;
    let compiled = crate::rlogic::CompiledLogic::compile(&logic_value)
        .map_err(|e| format!("Logic compilation failed: {}", e))?;

    // Parse data
    let mut data_value = if let Some(d) = data_str {
        json_parser::parse_json_str(d)
            .map_err(|e| format!("Invalid data JSON: {}", e))?
    } else {
        Value::Null
    };

    // Merge context if provided
    if let Some(c) = context_str {
        let context_value = json_parser::parse_json_str(c)
            .map_err(|e| format!("Invalid context JSON: {}", e))?;
        
        // Ensure data is an object to merge context
        if data_value.is_null() {
            let mut map = serde_json::Map::new();
            map.insert("$context".to_string(), context_value);
            data_value = Value::Object(map);
        } else if let Some(obj) = data_value.as_object_mut() {
            obj.insert("$context".to_string(), context_value);
        }
        // Note: If data is a primitive value, context cannot be merged
    }

    // Run evaluation
    let evaluator = Evaluator::new();
    evaluator.evaluate(&compiled, &data_value)
        .map(|v| clean_float_noise(v))
        .map_err(|e| format!("Evaluation error: {}", e))
}
