use super::vector_op::{Tensor2D, VectorOp};
use rapidhash::RapidHashMap;
use serde_json::Value;

/// Plan for executing a table's Repeat section using a hybrid tensor + categorical engine
#[derive(Debug, Clone)]
pub struct TableTensorPlan {
    pub total_cols: usize,
    pub numeric_col_count: usize,
    /// Maps column name to its column index in the table
    pub col_indices: RapidHashMap<String, usize>,
    /// Column names in original order
    pub col_names: Vec<String>,
    /// Flags whether each column index is numeric (true) or categorical (false)
    pub is_numeric_col: Vec<bool>,
    /// Static categorical values (e.g., col_idx -> "ROP")
    pub categorical_literals: RapidHashMap<usize, Value>,
    /// Pre-compiled vector ops that can be executed across all rows at once
    pub vector_ops: Vec<VectorOp>,
}

impl TableTensorPlan {
    /// Construct a TableTensorPlan from column names and initial literal/logic inspection
    pub fn new(col_names: &[String]) -> Self {
        let total_cols = col_names.len();
        let mut col_indices = RapidHashMap::default();
        for (i, name) in col_names.iter().enumerate() {
            col_indices.insert(name.clone(), i);
        }

        Self {
            total_cols,
            numeric_col_count: total_cols,
            col_indices,
            col_names: col_names.to_vec(),
            is_numeric_col: vec![true; total_cols],
            categorical_literals: RapidHashMap::default(),
            vector_ops: Vec::new(),
        }
    }

    /// Mark a column as categorical with a literal value (e.g. BENPAY_DEATH_TYPE = "ROP")
    pub fn mark_categorical(&mut self, col_idx: usize, literal: Value) {
        if col_idx < self.is_numeric_col.len() {
            self.is_numeric_col[col_idx] = false;
            self.categorical_literals.insert(col_idx, literal);
        }
    }

    /// Reconstruct the final JSON array of row objects from a computed Tensor2D
    /// and categorical literals, strictly matching the original column order and keys.
    pub fn reconstruct_rows(
        &self,
        tensor: &Tensor2D,
        start_iteration: i64,
        existing_rows: &[Value],
    ) -> Vec<Value> {
        let rows_to_build = tensor.rows;
        let mut out = Vec::with_capacity(existing_rows.len() + rows_to_build);

        // Include any existing static rows (e.g. Row 0)
        for r in existing_rows {
            out.push(r.clone());
        }

        for r in 0..rows_to_build {
            let mut row_map = serde_json::Map::with_capacity(self.total_cols);

            for (col_idx, col_name) in self.col_names.iter().enumerate() {
                if self.is_numeric_col[col_idx] {
                    let num = tensor.get(r, col_idx);
                    // Match integer vs float representation
                    let num_val = if num.fract() == 0.0 && num.abs() < (i64::MAX as f64) {
                        Value::Number(serde_json::Number::from(num as i64))
                    } else if let Some(n) = serde_json::Number::from_f64(num) {
                        Value::Number(n)
                    } else {
                        Value::Null
                    };
                    row_map.insert(col_name.clone(), num_val);
                } else if let Some(cat_val) = self.categorical_literals.get(&col_idx) {
                    row_map.insert(col_name.clone(), cat_val.clone());
                } else {
                    row_map.insert(col_name.clone(), Value::Null);
                }
            }

            out.push(Value::Object(row_map));
        }

        let _ = start_iteration;
        out
    }
}
