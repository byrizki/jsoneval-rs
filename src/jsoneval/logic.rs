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
