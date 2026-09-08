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
    /// Direct reference into local_rows inside table_evaluate — zero-copy self-table access
    LocalRows(&'a Vec<Value>),
}

impl<'a> TableRef<'a> {
    #[inline(always)]
    pub fn as_value(&self) -> &Value {
        match self {
            TableRef::Borrowed(v) => v,
            TableRef::Owned(v) => v,
            // LocalRows cannot be expressed as a single &Value without allocation;
            // callers that need the raw slice should use as_array() directly.
            TableRef::LocalRows(_) => &Value::Null,
        }
    }

    #[inline(always)]
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            TableRef::Borrowed(v) => match v {
                Value::Array(arr) => Some(arr.as_slice()),
                _ => None,
            },
            TableRef::Owned(v) => match v {
                Value::Array(arr) => Some(arr.as_slice()),
                _ => None,
            },
            TableRef::LocalRows(rows) => Some(rows.as_slice()),
        }
    }
}

/// Compact representation of a scalar lookup value for zero-allocation cache keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LookupValueKey {
    Num(u64), // to_bits()
    StrHash(u64),
    Bool(bool),
    Null,
}

/// 32-byte zero-allocation cache key for combined array lookups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CombinedLookupKey {
    pub arr_ptr: usize,
    pub field_hash: u64,
    pub is_range: bool,
    pub val_key: LookupValueKey,
}
