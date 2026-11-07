//! Tensor and Columnar Computation Subsystem for RLogic & JSONEval
//!
//! Provides dense contiguous 2D float matrices (`Tensor2D`), accelerated typed
//! reference tables (`TypedReferenceTable`), and vectorized column operations (`VectorOp`).

pub mod planner;
pub mod reference_table;
pub mod vector_op;

pub use planner::TableTensorPlan;
pub use reference_table::TypedReferenceTable;
pub use vector_op::{Tensor2D, VectorOp};
