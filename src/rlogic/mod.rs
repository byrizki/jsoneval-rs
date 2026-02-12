//! RLogic - High-performance JSON Logic compiler and evaluator
//!
//! This module provides a complete implementation of JSON Logic with:
//! - Pre-compilation of logic expressions for fast repeated evaluation
//! - Mutation tracking via proxy-like data wrapper (EvalData)
//! - All data mutations gated through EvalData for safety
//! - Zero external dependencies (uses only standard library and serde_json)
//! - Global compiled logic storage for cross-instance sharing

pub mod compiled;
pub mod compiled_logic_store;
pub mod config;
pub mod evaluator;

use serde_json::Value;

pub use compiled::{CompiledLogic, CompiledLogicStore, LogicId};
pub use compiled_logic_store::{CompiledLogicId, CompiledLogicStoreStats};
pub use config::RLogicConfig;
pub use evaluator::Evaluator;

/// Main RLogic engine combining compilation and evaluation
pub struct RLogic {
    store: CompiledLogicStore,
    evaluator: Evaluator,
}

impl RLogic {
    /// Create a new RLogic instance with default configuration
    pub fn new() -> Self {
        Self::with_config(RLogicConfig::default())
    }

    /// Create a new RLogic instance with custom configuration
    pub fn with_config(config: RLogicConfig) -> Self {
        Self {
            store: CompiledLogicStore::new(),
            evaluator: Evaluator::new().with_config(config),
        }
    }

    /// Compile a JSON Logic expression
    pub fn compile(&mut self, logic: &Value) -> Result<LogicId, String> {
        self.store.compile(logic)
    }

    /// Evaluate a compiled logic expression against data
    pub fn run(&self, logic_id: &LogicId, data: &Value) -> Result<Value, String> {
        let logic = self
            .store
            .get(logic_id)
            .ok_or_else(|| "Logic ID not found".to_string())?;

        self.evaluator.evaluate(logic, data)
    }

    /// Evaluate a compiled logic expression with internal context
    ///
    /// # Arguments
    /// * `logic_id` - The compiled logic ID
    /// * `user_data` - User's data (primary lookup source)
    /// * `internal_context` - Internal variables (e.g., $iteration, $threshold, column vars)
    ///
    /// Variables are resolved in order: internal_context → user_data
    pub fn run_with_context(
        &self,
        logic_id: &LogicId,
        user_data: &Value,
        internal_context: &Value,
    ) -> Result<Value, String> {
        let logic = self
            .store
            .get(logic_id)
            .ok_or_else(|| "Logic ID not found".to_string())?;

        self.evaluator
            .evaluate_with_internal_context(logic, user_data, internal_context)
    }

    /// Evaluate a compiled logic expression with custom config
    pub fn run_with_config(
        &self,
        logic_id: &LogicId,
        data: &Value,
        config: &RLogicConfig,
    ) -> Result<Value, String> {
        let logic = self
            .store
            .get(logic_id)
            .ok_or_else(|| "Logic ID not found".to_string())?;

        let evaluator = Evaluator::new().with_config(*config);
        evaluator.evaluate(logic, data)
    }

    /// Compile and evaluate a logic expression directly
    pub fn evaluate(&self, logic: &Value, data: &Value) -> Result<Value, String> {
        let compiled = CompiledLogic::compile(logic)?;
        self.evaluator.evaluate(&compiled, data)
    }

    /// Get the referenced variables in a compiled logic
    pub fn get_referenced_vars(&self, logic_id: &LogicId) -> Option<Vec<String>> {
        self.store
            .get(logic_id)
            .map(|logic| logic.referenced_vars())
    }

    /// Check if a compiled logic has forward references (e.g., references future iterations)
    pub fn has_forward_reference(&self, logic_id: &LogicId) -> bool {
        self.store
            .get(logic_id)
            .map(|logic| logic.has_forward_reference())
            .unwrap_or(false)
    }

    /// Build and store index for a table
    pub fn index_table(&self, name: &str, data: &Value) {
        self.evaluator.index_table(name, data);
    }

    /// Clear all stored indices
    pub fn clear_indices(&self) {
        self.evaluator.clear_indices();
    }
}

impl Default for RLogic {
    fn default() -> Self {
        Self::new()
    }
}
