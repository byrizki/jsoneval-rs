use crate::LogicId;
use serde_json::Value;
use std::sync::Arc;

/// Pre-compiled column metadata computed at parse time (zero-copy design)
#[derive(Clone, Debug)]
pub struct ColumnMetadata {
    /// Column name (Arc to avoid clones)
    pub name: Arc<str>,
    /// Variable path like "$columnName" (pre-computed)
    pub var_path: Arc<str>,
    /// Logic ID if this column has evaluation logic
    pub logic: Option<LogicId>,
    /// Literal value if no logic (Arc to share across evaluations)
    pub literal: Option<Arc<Value>>,
    /// Variable names this column depends on (e.g., ["$other_col"])
    pub dependencies: Arc<[String]>,
    /// Whether this column has forward references (computed once)
    pub has_forward_ref: bool,
}

impl ColumnMetadata {
    #[inline]
    pub fn new(
        name: &str,
        logic: Option<LogicId>,
        literal: Option<Value>,
        dependencies: Vec<String>,
        has_forward_ref: bool,
    ) -> Self {
        let var_path = format!("${}", name);
        Self {
            name: Arc::from(name),
            var_path: Arc::from(var_path.as_str()),
            logic,
            literal: literal.map(Arc::new),
            dependencies: dependencies.into(),
            has_forward_ref,
        }
    }
}

/// Pre-compiled repeat bound metadata
#[derive(Clone, Debug)]
pub struct RepeatBoundMetadata {
    pub logic: Option<LogicId>,
    /// Literal value (Arc to share)
    pub literal: Arc<Value>,
}

/// Pre-compiled row metadata (computed at parse time)
#[derive(Clone, Debug)]
pub enum RowMetadata {
    Static {
        columns: Arc<[ColumnMetadata]>,
    },
    Repeat {
        start: RepeatBoundMetadata,
        end: RepeatBoundMetadata,
        columns: Arc<[ColumnMetadata]>,
        /// Pre-computed set of forward-referencing column names (transitive closure)
        forward_cols: Arc<[usize]>, // indices into columns array
        /// Pre-computed normal columns in schema order
        normal_cols: Arc<[usize]>, // indices into columns array
    },
}

/// Pre-compiled table metadata (computed once at parse time)
#[derive(Clone, Debug)]
pub struct TableMetadata {
    /// Data columns to evaluate before skip/clear
    pub data_plans: Arc<[(Arc<str>, Option<LogicId>, Option<Arc<Value>>)]>,
    /// Row plans with pre-computed metadata
    pub row_plans: Arc<[RowMetadata]>,
    /// Skip logic
    pub skip_logic: Option<LogicId>,
    /// Skip literal value
    pub skip_literal: bool,
    /// Clear logic
    pub clear_logic: Option<LogicId>,
    /// Clear literal value
    pub clear_literal: bool,
}
