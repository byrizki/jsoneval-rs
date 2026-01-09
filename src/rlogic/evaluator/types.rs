use serde_json::Value;

/// Arithmetic operation types for fast path evaluation
#[derive(Debug, Clone, Copy)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Comparison operation types
#[derive(Debug, Clone, Copy)]
pub enum CompOp {
    Eq,
    StrictEq,
    Ne,
    StrictNe,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Array quantifier types
#[derive(Debug, Clone, Copy)]
pub enum Quantifier {
    All,
    Some,
    None,
}

/// Helper enum for zero-copy or owned table access
pub enum TableRef<'a> {
    Borrowed(&'a Value),
    Owned(Value),
}

impl<'a> TableRef<'a> {
    #[inline(always)]
    pub fn as_value(&self) -> &Value {
        match self {
            TableRef::Borrowed(v) => v,
            TableRef::Owned(v) => v,
        }
    }

    #[inline(always)]
    pub fn as_array(&self) -> Option<&[Value]> {
        match self.as_value() {
            Value::Array(arr) => Some(arr.as_slice()),
            _ => None,
        }
    }
}
