use ahash::AHashMap;
use serde::Serialize;
use serde_json::Value;
use crate::path_utils;

/// Unique identifier for compiled logic expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct LogicId(pub(crate) u64);

/// Compiled JSON Logic expression optimized for fast evaluation
#[derive(Debug, Clone)]
pub enum CompiledLogic {
    // Literal values
    Null,
    Bool(bool),
    Number(String), // Store as string to preserve precision with arbitrary_precision
    String(String),
    Array(Vec<CompiledLogic>),
    
    // Variable access
    Var(String, Option<Box<CompiledLogic>>), // name, default
    Ref(String, Option<Box<CompiledLogic>>), // JSON Schema reference path, default
    
    // Logical operators
    And(Vec<CompiledLogic>),
    Or(Vec<CompiledLogic>),
    Not(Box<CompiledLogic>),
    If(Box<CompiledLogic>, Box<CompiledLogic>, Box<CompiledLogic>), // condition, then, else
    
    // Comparison operators
    Equal(Box<CompiledLogic>, Box<CompiledLogic>),
    StrictEqual(Box<CompiledLogic>, Box<CompiledLogic>),
    NotEqual(Box<CompiledLogic>, Box<CompiledLogic>),
    StrictNotEqual(Box<CompiledLogic>, Box<CompiledLogic>),
    LessThan(Box<CompiledLogic>, Box<CompiledLogic>),
    LessThanOrEqual(Box<CompiledLogic>, Box<CompiledLogic>),
    GreaterThan(Box<CompiledLogic>, Box<CompiledLogic>),
    GreaterThanOrEqual(Box<CompiledLogic>, Box<CompiledLogic>),
    
    // Arithmetic operators
    Add(Vec<CompiledLogic>),
    Subtract(Vec<CompiledLogic>),
    Multiply(Vec<CompiledLogic>),
    Divide(Vec<CompiledLogic>),
    Modulo(Box<CompiledLogic>, Box<CompiledLogic>),
    Power(Box<CompiledLogic>, Box<CompiledLogic>),
    
    // Array operators
    Map(Box<CompiledLogic>, Box<CompiledLogic>), // array, logic
    Filter(Box<CompiledLogic>, Box<CompiledLogic>), // array, logic
    Reduce(Box<CompiledLogic>, Box<CompiledLogic>, Box<CompiledLogic>), // array, logic, initial
    All(Box<CompiledLogic>, Box<CompiledLogic>), // array, logic
    Some(Box<CompiledLogic>, Box<CompiledLogic>), // array, logic
    None(Box<CompiledLogic>, Box<CompiledLogic>), // array, logic
    Merge(Vec<CompiledLogic>),
    In(Box<CompiledLogic>, Box<CompiledLogic>), // value, array
    
    // String operators
    Cat(Vec<CompiledLogic>),
    Substr(Box<CompiledLogic>, Box<CompiledLogic>, Option<Box<CompiledLogic>>), // string, start, length
    
    // Utility operators
    Missing(Vec<String>),
    MissingSome(Box<CompiledLogic>, Vec<String>), // minimum, keys
    
    // Custom operators - Math
    Abs(Box<CompiledLogic>),
    Max(Vec<CompiledLogic>),
    Min(Vec<CompiledLogic>),
    Pow(Box<CompiledLogic>, Box<CompiledLogic>),
    Round(Box<CompiledLogic>),
    RoundUp(Box<CompiledLogic>),
    RoundDown(Box<CompiledLogic>),
    
    // Custom operators - String
    Length(Box<CompiledLogic>),
    Search(Box<CompiledLogic>, Box<CompiledLogic>, Option<Box<CompiledLogic>>), // find, within, start_num
    Left(Box<CompiledLogic>, Option<Box<CompiledLogic>>), // text, num_chars
    Right(Box<CompiledLogic>, Option<Box<CompiledLogic>>), // text, num_chars
    Mid(Box<CompiledLogic>, Box<CompiledLogic>, Box<CompiledLogic>), // text, start, num_chars
    Len(Box<CompiledLogic>),
    SplitText(Box<CompiledLogic>, Box<CompiledLogic>, Option<Box<CompiledLogic>>), // value, separator, index
    Concat(Vec<CompiledLogic>),
    SplitValue(Box<CompiledLogic>, Box<CompiledLogic>), // string, separator
    
    // Custom operators - Logical
    Xor(Box<CompiledLogic>, Box<CompiledLogic>),
    IfNull(Box<CompiledLogic>, Box<CompiledLogic>),
    IsEmpty(Box<CompiledLogic>),
    Empty,
    
    // Custom operators - Date
    Today,
    Now,
    Days(Box<CompiledLogic>, Box<CompiledLogic>), // end_date, start_date
    Year(Box<CompiledLogic>),
    Month(Box<CompiledLogic>),
    Day(Box<CompiledLogic>),
    Date(Box<CompiledLogic>, Box<CompiledLogic>, Box<CompiledLogic>), // year, month, day
    
    // Custom operators - Array/Table
    Sum(Box<CompiledLogic>, Option<Box<CompiledLogic>>, Option<Box<CompiledLogic>>), // array/value, optional field name, optional index threshold
    For(Box<CompiledLogic>, Box<CompiledLogic>, Box<CompiledLogic>), // start, end, logic (with $iteration variable)
    
    // Complex table operations
    ValueAt(Box<CompiledLogic>, Box<CompiledLogic>, Option<Box<CompiledLogic>>), // table, row_idx, col_name
    MaxAt(Box<CompiledLogic>, Box<CompiledLogic>), // table, col_name
    IndexAt(Box<CompiledLogic>, Box<CompiledLogic>, Box<CompiledLogic>, Option<Box<CompiledLogic>>), // lookup_val, table, field, range
    Match(Box<CompiledLogic>, Vec<CompiledLogic>), // table, conditions (pairs of value, field)
    MatchRange(Box<CompiledLogic>, Vec<CompiledLogic>), // table, conditions (triplets of min_col, max_col, value)
    Choose(Box<CompiledLogic>, Vec<CompiledLogic>), // table, conditions (pairs of value, field)
    FindIndex(Box<CompiledLogic>, Vec<CompiledLogic>), // table, conditions (complex nested logic)
    
    // Array operations
    Multiplies(Vec<CompiledLogic>), // flatten and multiply
    Divides(Vec<CompiledLogic>), // flatten and divide
    
    // Advanced date functions
    YearFrac(Box<CompiledLogic>, Box<CompiledLogic>, Option<Box<CompiledLogic>>), // start_date, end_date, basis
    DateDif(Box<CompiledLogic>, Box<CompiledLogic>, Box<CompiledLogic>), // start_date, end_date, unit
    
    // UI helpers
    RangeOptions(Box<CompiledLogic>, Box<CompiledLogic>), // min, max
    MapOptions(Box<CompiledLogic>, Box<CompiledLogic>, Box<CompiledLogic>), // table, field_label, field_value
    MapOptionsIf(Box<CompiledLogic>, Box<CompiledLogic>, Box<CompiledLogic>, Vec<CompiledLogic>), // table, field_label, field_value, conditions
    Return(Box<Value>), // return value as-is (no-op, just returns the raw value)
}

impl CompiledLogic {
    /// Compile a JSON Logic expression from JSON Value
    pub fn compile(logic: &Value) -> Result<Self, String> {
        match logic {
            Value::Null => Ok(CompiledLogic::Null),
            Value::Bool(b) => Ok(CompiledLogic::Bool(*b)),
            Value::Number(n) => {
                // With arbitrary_precision, store as string to preserve precision
                Ok(CompiledLogic::Number(n.to_string()))
            }
            Value::String(s) => Ok(CompiledLogic::String(s.clone())),
            Value::Array(arr) => {
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                Ok(CompiledLogic::Array(compiled?))
            }
            Value::Object(obj) => {
                if obj.is_empty() {
                    return Ok(CompiledLogic::Null);
                }
                
                // Get the operator (first key)
                let (op, args) = obj.iter().next().unwrap();
                
                Self::compile_operator(op, args)
            }
        }
    }
    
    fn compile_operator(op: &str, args: &Value) -> Result<Self, String> {
        match op {
            // Variable access
            "var" => {
                if let Value::String(name) = args {
                    // OPTIMIZED: Pre-normalize path during compilation
                    let normalized = path_utils::normalize_to_json_pointer(name);
                    Ok(CompiledLogic::Var(normalized, None))
                } else if let Value::Array(arr) = args {
                    if arr.is_empty() {
                        return Err("var requires at least one argument".to_string());
                    }
                    let name = arr[0].as_str()
                        .ok_or("var name must be a string")?;
                    // OPTIMIZED: Pre-normalize path during compilation
                    let normalized = path_utils::normalize_to_json_pointer(name);
                    let default = if arr.len() > 1 {
                        Some(Box::new(Self::compile(&arr[1])?))
                    } else {
                        None
                    };
                    Ok(CompiledLogic::Var(normalized, default))
                } else {
                    Err("var requires string or array".to_string())
                }
            }
            "$ref" | "ref" => {
                if let Value::String(path) = args {
                    // OPTIMIZED: Pre-normalize path during compilation
                    let normalized = path_utils::normalize_to_json_pointer(path);
                    Ok(CompiledLogic::Ref(normalized, None))
                } else if let Value::Array(arr) = args {
                    if arr.is_empty() {
                        return Err("$ref requires at least one argument".to_string());
                    }
                    let path = arr[0].as_str()
                        .ok_or("$ref path must be a string")?;
                    // OPTIMIZED: Pre-normalize path during compilation
                    let normalized = path_utils::normalize_to_json_pointer(path);
                    let default = if arr.len() > 1 {
                        Some(Box::new(Self::compile(&arr[1])?))
                    } else {
                        None
                    };
                    Ok(CompiledLogic::Ref(normalized, default))
                } else {
                    Err("$ref requires string or array".to_string())
                }
            }
            
            // Logical operators
            "and" => {
                let arr = args.as_array().ok_or("and requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                let items = compiled?;
                // OPTIMIZATION: Flatten nested and operations (And(And(a,b),c) -> And(a,b,c))
                Ok(CompiledLogic::And(Self::flatten_and(items)))
            }
            "or" => {
                let arr = args.as_array().ok_or("or requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                let items = compiled?;
                // OPTIMIZATION: Flatten nested or operations (Or(Or(a,b),c) -> Or(a,b,c))
                Ok(CompiledLogic::Or(Self::flatten_or(items)))
            }
            "!" | "not" => {
                // Handle both array format [value] and direct value format
                let value_to_negate = if let Value::Array(arr) = args {
                    if arr.is_empty() {
                        return Err("! requires an argument".to_string());
                    }
                    &arr[0]
                } else {
                    args
                };
                
                let inner = Self::compile(value_to_negate)?;
                // OPTIMIZATION: Eliminate double negation (!(!x) -> x)
                if let CompiledLogic::Not(inner_expr) = inner {
                    Ok(*inner_expr)
                } else {
                    Ok(CompiledLogic::Not(Box::new(inner)))
                }
            }
            "if" => {
                let arr = args.as_array().ok_or("if requires array")?;
                if arr.len() < 3 {
                    return Err("if requires at least 3 arguments".to_string());
                }
                Ok(CompiledLogic::If(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    Box::new(Self::compile(&arr[2])?),
                ))
            }
            
            // Comparison operators
            "==" => Self::compile_binary(args, |a, b| CompiledLogic::Equal(a, b)),
            "===" => Self::compile_binary(args, |a, b| CompiledLogic::StrictEqual(a, b)),
            "!=" => Self::compile_binary(args, |a, b| CompiledLogic::NotEqual(a, b)),
            "!==" => Self::compile_binary(args, |a, b| CompiledLogic::StrictNotEqual(a, b)),
            "<" => Self::compile_binary(args, |a, b| CompiledLogic::LessThan(a, b)),
            "<=" => Self::compile_binary(args, |a, b| CompiledLogic::LessThanOrEqual(a, b)),
            ">" => Self::compile_binary(args, |a, b| CompiledLogic::GreaterThan(a, b)),
            ">=" => Self::compile_binary(args, |a, b| CompiledLogic::GreaterThanOrEqual(a, b)),
            
            // Arithmetic operators
            "+" => {
                let arr = args.as_array().ok_or("+ requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                let items = compiled?;
                // OPTIMIZATION: Flatten nested additions (Add(Add(a,b),c) -> Add(a,b,c))
                Ok(CompiledLogic::Add(Self::flatten_add(items)))
            }
            "-" => {
                let arr = args.as_array().ok_or("- requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                Ok(CompiledLogic::Subtract(compiled?))
            }
            "*" => {
                let arr = args.as_array().ok_or("* requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                let items = compiled?;
                // OPTIMIZATION: Flatten nested multiplications
                Ok(CompiledLogic::Multiply(Self::flatten_multiply(items)))
            }
            "/" => {
                let arr = args.as_array().ok_or("/ requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                Ok(CompiledLogic::Divide(compiled?))
            }
            "%" => Self::compile_binary(args, |a, b| CompiledLogic::Modulo(a, b)),
            "^" => Self::compile_binary(args, |a, b| CompiledLogic::Power(a, b)),
            
            // Array operators
            "map" => Self::compile_binary(args, |a, b| CompiledLogic::Map(a, b)),
            "filter" => Self::compile_binary(args, |a, b| CompiledLogic::Filter(a, b)),
            "reduce" => {
                let arr = args.as_array().ok_or("reduce requires array")?;
                if arr.len() < 3 {
                    return Err("reduce requires 3 arguments".to_string());
                }
                Ok(CompiledLogic::Reduce(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    Box::new(Self::compile(&arr[2])?),
                ))
            }
            "all" => Self::compile_binary(args, |a, b| CompiledLogic::All(a, b)),
            "some" => Self::compile_binary(args, |a, b| CompiledLogic::Some(a, b)),
            "none" => Self::compile_binary(args, |a, b| CompiledLogic::None(a, b)),
            "merge" => {
                let arr = args.as_array().ok_or("merge requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                Ok(CompiledLogic::Merge(compiled?))
            }
            "in" => Self::compile_binary(args, |a, b| CompiledLogic::In(a, b)),
            
            // String operators
            "cat" => {
                let arr = args.as_array().ok_or("cat requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                let items = compiled?;
                // OPTIMIZATION: Flatten nested concatenations (Cat(Cat(a,b),c) -> Cat(a,b,c))
                Ok(CompiledLogic::Cat(Self::flatten_cat(items)))
            }
            "substr" => {
                let arr = args.as_array().ok_or("substr requires array")?;
                if arr.len() < 2 {
                    return Err("substr requires at least 2 arguments".to_string());
                }
                let length = if arr.len() > 2 {
                    Some(Box::new(Self::compile(&arr[2])?))
                } else {
                    None
                };
                Ok(CompiledLogic::Substr(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    length,
                ))
            }
            
            // Utility operators
            "missing" => {
                let keys = if let Value::Array(arr) = args {
                    arr.iter()
                        .map(|v| v.as_str().ok_or("missing key must be string").map(|s| s.to_string()))
                        .collect::<Result<Vec<_>, _>>()?
                } else if let Value::String(s) = args {
                    vec![s.clone()]
                } else {
                    return Err("missing requires string or array".to_string());
                };
                Ok(CompiledLogic::Missing(keys))
            }
            "missing_some" => {
                let arr = args.as_array().ok_or("missing_some requires array")?;
                if arr.len() < 2 {
                    return Err("missing_some requires at least 2 arguments".to_string());
                }
                let minimum = Box::new(Self::compile(&arr[0])?);
                let keys = if let Value::Array(key_arr) = &arr[1] {
                    key_arr.iter()
                        .map(|v| v.as_str().ok_or("key must be string").map(|s| s.to_string()))
                        .collect::<Result<Vec<_>, _>>()?
                } else {
                    return Err("missing_some keys must be array".to_string());
                };
                Ok(CompiledLogic::MissingSome(minimum, keys))
            }
            
            // Custom operators - Math
            "abs" => Ok(CompiledLogic::Abs(Box::new(Self::compile(args)?))),
            "max" => {
                let arr = args.as_array().ok_or("max requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                Ok(CompiledLogic::Max(compiled?))
            }
            "min" => {
                let arr = args.as_array().ok_or("min requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                Ok(CompiledLogic::Min(compiled?))
            }
            "pow" | "**" => Self::compile_binary(args, |a, b| CompiledLogic::Pow(a, b)),
            "round" | "ROUND" => Ok(CompiledLogic::Round(Box::new(Self::compile(args)?))),
            "roundup" | "ROUNDUP" => Ok(CompiledLogic::RoundUp(Box::new(Self::compile(args)?))),
            "rounddown" | "ROUNDDOWN" => Ok(CompiledLogic::RoundDown(Box::new(Self::compile(args)?))),
            
            // Custom operators - String
            "length" => Ok(CompiledLogic::Length(Box::new(Self::compile(args)?))),
            "len" | "LEN" => Ok(CompiledLogic::Len(Box::new(Self::compile(args)?))),
            "search" | "SEARCH" => {
                let arr = args.as_array().ok_or("search requires array")?;
                if arr.len() < 2 {
                    return Err("search requires at least 2 arguments".to_string());
                }
                let start_num = if arr.len() > 2 {
                    Some(Box::new(Self::compile(&arr[2])?))
                } else {
                    None
                };
                Ok(CompiledLogic::Search(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    start_num,
                ))
            }
            "left" | "LEFT" => {
                if let Value::Array(arr) = args {
                    let num_chars = if arr.len() > 1 {
                        Some(Box::new(Self::compile(&arr[1])?))
                    } else {
                        None
                    };
                    Ok(CompiledLogic::Left(Box::new(Self::compile(&arr[0])?), num_chars))
                } else {
                    Ok(CompiledLogic::Left(Box::new(Self::compile(args)?), None))
                }
            }
            "right" | "RIGHT" => {
                if let Value::Array(arr) = args {
                    let num_chars = if arr.len() > 1 {
                        Some(Box::new(Self::compile(&arr[1])?))
                    } else {
                        None
                    };
                    Ok(CompiledLogic::Right(Box::new(Self::compile(&arr[0])?), num_chars))
                } else {
                    Ok(CompiledLogic::Right(Box::new(Self::compile(args)?), None))
                }
            }
            "mid" | "MID" => {
                let arr = args.as_array().ok_or("mid requires array")?;
                if arr.len() < 3 {
                    return Err("mid requires 3 arguments".to_string());
                }
                Ok(CompiledLogic::Mid(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    Box::new(Self::compile(&arr[2])?),
                ))
            }
            "splittext" | "SPLITTEXT" => {
                let arr = args.as_array().ok_or("splittext requires array")?;
                if arr.len() < 2 {
                    return Err("splittext requires at least 2 arguments".to_string());
                }
                let index = if arr.len() > 2 {
                    Some(Box::new(Self::compile(&arr[2])?))
                } else {
                    None
                };
                Ok(CompiledLogic::SplitText(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    index,
                ))
            }
            "concat" | "CONCAT" => {
                let arr = args.as_array().ok_or("concat requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                Ok(CompiledLogic::Concat(compiled?))
            }
            "splitvalue" | "SPLITVALUE" => Self::compile_binary(args, |a, b| CompiledLogic::SplitValue(a, b)),
            
            // Custom operators - Logical
            "xor" => Self::compile_binary(args, |a, b| CompiledLogic::Xor(a, b)),
            "ifnull" | "IFNULL" => Self::compile_binary(args, |a, b| CompiledLogic::IfNull(a, b)),
            "isempty" | "ISEMPTY" => Ok(CompiledLogic::IsEmpty(Box::new(Self::compile(args)?))),
            "empty" | "EMPTY" => Ok(CompiledLogic::Empty),
            
            // Custom operators - Date
            "today" | "TODAY" => Ok(CompiledLogic::Today),
            "now" | "NOW" => Ok(CompiledLogic::Now),
            "days" | "DAYS" => Self::compile_binary(args, |a, b| CompiledLogic::Days(a, b)),
            "year" | "YEAR" => Ok(CompiledLogic::Year(Box::new(Self::compile(args)?))),
            "month" | "MONTH" => Ok(CompiledLogic::Month(Box::new(Self::compile(args)?))),
            "day" | "DAY" => Ok(CompiledLogic::Day(Box::new(Self::compile(args)?))),
            "date" | "DATE" => {
                let arr = args.as_array().ok_or("date requires array")?;
                if arr.len() < 3 {
                    return Err("date requires 3 arguments".to_string());
                }
                Ok(CompiledLogic::Date(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    Box::new(Self::compile(&arr[2])?),
                ))
            }
            
            // Custom operators - Array/Table
            "sum" | "SUM" => {
                if let Value::Array(arr) = args {
                    if arr.is_empty() {
                        return Err("sum requires at least 1 argument".to_string());
                    }
                    let field = if arr.len() > 1 {
                        Some(Box::new(Self::compile(&arr[1])?))
                    } else {
                        None
                    };
                    let threshold = if arr.len() > 2 {
                        Some(Box::new(Self::compile(&arr[2])?))
                    } else {
                        None
                    };
                    Ok(CompiledLogic::Sum(Box::new(Self::compile(&arr[0])?), field, threshold))
                } else {
                    Ok(CompiledLogic::Sum(Box::new(Self::compile(args)?), None, None))
                }
            }
            "FOR" => {
                let arr = args.as_array().ok_or("FOR requires array")?;
                if arr.len() < 3 {
                    return Err("FOR requires 3 arguments: start, end, logic".to_string());
                }
                Ok(CompiledLogic::For(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    Box::new(Self::compile(&arr[2])?),
                ))
            }
            
            // Complex table operations
            "VALUEAT" => {
                let arr = args.as_array().ok_or("VALUEAT requires array")?;
                if arr.len() < 2 {
                    return Err("VALUEAT requires at least 2 arguments".to_string());
                }
                let col_name = if arr.len() > 2 {
                    Some(Box::new(Self::compile(&arr[2])?))
                } else {
                    None
                };
                Ok(CompiledLogic::ValueAt(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    col_name,
                ))
            }
            "MAXAT" => Self::compile_binary(args, |a, b| CompiledLogic::MaxAt(a, b)),
            "INDEXAT" => {
                let arr = args.as_array().ok_or("INDEXAT requires array")?;
                if arr.len() < 3 {
                    return Err("INDEXAT requires at least 3 arguments".to_string());
                }
                let range = if arr.len() > 3 {
                    Some(Box::new(Self::compile(&arr[3])?))
                } else {
                    None
                };
                Ok(CompiledLogic::IndexAt(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    Box::new(Self::compile(&arr[2])?),
                    range,
                ))
            }
            "MATCH" => {
                let arr = args.as_array().ok_or("MATCH requires array")?;
                if arr.is_empty() {
                    return Err("MATCH requires at least 1 argument".to_string());
                }
                let table = Box::new(Self::compile(&arr[0])?);
                let conditions: Result<Vec<_>, _> = arr[1..].iter().map(Self::compile).collect();
                Ok(CompiledLogic::Match(table, conditions?))
            }
            "MATCHRANGE" => {
                let arr = args.as_array().ok_or("MATCHRANGE requires array")?;
                if arr.is_empty() {
                    return Err("MATCHRANGE requires at least 1 argument".to_string());
                }
                let table = Box::new(Self::compile(&arr[0])?);
                let conditions: Result<Vec<_>, _> = arr[1..].iter().map(Self::compile).collect();
                Ok(CompiledLogic::MatchRange(table, conditions?))
            }
            "CHOOSE" => {
                let arr = args.as_array().ok_or("CHOOSE requires array")?;
                if arr.is_empty() {
                    return Err("CHOOSE requires at least 1 argument".to_string());
                }
                let table = Box::new(Self::compile(&arr[0])?);
                let conditions: Result<Vec<_>, _> = arr[1..].iter().map(Self::compile).collect();
                Ok(CompiledLogic::Choose(table, conditions?))
            }
            "FINDINDEX" => {
                let arr = args.as_array().ok_or("FINDINDEX requires array")?;
                if arr.len() < 2 {
                    return Err("FINDINDEX requires at least 2 arguments".to_string());
                }
                let table = Box::new(Self::compile(&arr[0])?);
                // CRITICAL: Convert string literals to var references in conditions
                // This allows ergonomic syntax: "INSAGE" instead of {"var": "INSAGE"}
                let conditions: Result<Vec<_>, _> = arr[1..]
                    .iter()
                    .map(|cond| Self::compile(&Self::preprocess_table_condition(cond)))
                    .collect();
                Ok(CompiledLogic::FindIndex(table, conditions?))
            }
            
            // Array operations
            "MULTIPLIES" => {
                let arr = args.as_array().ok_or("MULTIPLIES requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                Ok(CompiledLogic::Multiplies(compiled?))
            }
            "DIVIDES" => {
                let arr = args.as_array().ok_or("DIVIDES requires array")?;
                let compiled: Result<Vec<_>, _> = arr.iter().map(Self::compile).collect();
                Ok(CompiledLogic::Divides(compiled?))
            }
            
            // Advanced date functions
            "YEARFRAC" => {
                let arr = args.as_array().ok_or("YEARFRAC requires array")?;
                if arr.len() < 2 {
                    return Err("YEARFRAC requires at least 2 arguments".to_string());
                }
                let basis = if arr.len() > 2 {
                    Some(Box::new(Self::compile(&arr[2])?))
                } else {
                    None
                };
                Ok(CompiledLogic::YearFrac(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    basis,
                ))
            }
            "DATEDIF" => {
                let arr = args.as_array().ok_or("DATEDIF requires array")?;
                if arr.len() < 3 {
                    return Err("DATEDIF requires 3 arguments".to_string());
                }
                Ok(CompiledLogic::DateDif(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    Box::new(Self::compile(&arr[2])?),
                ))
            }
            
            // UI helpers
            "RANGEOPTIONS" => Self::compile_binary(args, |a, b| CompiledLogic::RangeOptions(a, b)),
            "MAPOPTIONS" => {
                let arr = args.as_array().ok_or("MAPOPTIONS requires array")?;
                if arr.len() < 3 {
                    return Err("MAPOPTIONS requires 3 arguments".to_string());
                }
                Ok(CompiledLogic::MapOptions(
                    Box::new(Self::compile(&arr[0])?),
                    Box::new(Self::compile(&arr[1])?),
                    Box::new(Self::compile(&arr[2])?),
                ))
            }
            "MAPOPTIONSIF" => {
                let arr = args.as_array().ok_or("MAPOPTIONSIF requires array")?;
                if arr.len() < 4 {
                    return Err("MAPOPTIONSIF requires at least 4 arguments".to_string());
                }
                let table = Box::new(Self::compile(&arr[0])?);
                let field_label = Box::new(Self::compile(&arr[1])?);
                let field_value = Box::new(Self::compile(&arr[2])?);
                
                // Handle triplet syntax: value, operator, field -> {operator: [value, {var: field}]}
                let condition_args = &arr[3..];
                let mut conditions = Vec::new();
                
                let mut i = 0;
                while i + 2 < condition_args.len() {
                    let value = &condition_args[i];
                    let operator = &condition_args[i + 1];
                    let field = &condition_args[i + 2];
                    
                    if let Value::String(op) = operator {
                        // Create comparison: {op: [value, {var: field}]}
                        let field_var = if let Value::String(f) = field {
                            serde_json::json!({"var": f})
                        } else {
                            field.clone()
                        };
                        
                        let comparison = serde_json::json!({
                            op: [value.clone(), field_var]
                        });
                        
                        conditions.push(Self::compile(&comparison)?);
                    }
                    
                    i += 3;
                }
                
                // Handle any remaining individual conditions
                while i < condition_args.len() {
                    conditions.push(Self::compile(&Self::preprocess_table_condition(&condition_args[i]))?);
                    i += 1;
                }
                
                Ok(CompiledLogic::MapOptionsIf(table, field_label, field_value, conditions))
            }
            "return" => Ok(CompiledLogic::Return(Box::new(args.clone()))),
            
            _ => Err(format!("Unknown operator: {}", op)),
        }
    }
    
    fn compile_binary<F>(args: &Value, f: F) -> Result<Self, String>
    where
        F: FnOnce(Box<CompiledLogic>, Box<CompiledLogic>) -> CompiledLogic,
    {
        let arr = args.as_array().ok_or("Binary operator requires array")?;
        if arr.len() != 2 {
            return Err("Binary operator requires exactly 2 arguments".to_string());
        }
        Ok(f(
            Box::new(Self::compile(&arr[0])?),
            Box::new(Self::compile(&arr[1])?),
        ))
    }
    
    /// Preprocess table condition to convert string literals to var references
    /// This allows ergonomic syntax in FINDINDEX/MATCH/CHOOSE conditions
    /// 
    /// Handles formats:
    /// - Comparison triplets: ["==", value, "col"] -> {"==": [value, {"var": "col"}]}
    /// - Logical operators: ["&&", cond1, cond2] -> {"and": [cond1, cond2]}
    /// - String field names: "col" -> {"var": "col"}
    fn preprocess_table_condition(value: &Value) -> Value {
        match value {
            Value::String(s) => {
                // Convert standalone strings to var references
                serde_json::json!({"var": s})
            }
            Value::Array(arr) => {
                // Check if this is an operator in array shorthand format
                if !arr.is_empty() {
                    if let Some(op_str) = arr[0].as_str() {
                        // Check for comparison operators: [op, value, col]
                        let is_comparison = matches!(op_str, "==" | "!=" | "===" | "!==" | "<" | ">" | "<=" | ">=");
                        
                        if is_comparison && arr.len() >= 3 {
                            // Comparison triplet: [op, value, col] -> {op: [col_var, value]}
                            // Evaluates as: row[col] op value
                            // DON'T preprocess the value (2nd arg) - keep it as-is
                            let value_arg = arr[1].clone();
                            
                            // Only convert the column name (3rd arg) to var reference if it's a string
                            let col_arg = if let Value::String(col) = &arr[2] {
                                // Convert column name string to var reference
                                serde_json::json!({"var": col})
                            } else {
                                // If it's not a string, preprocess it (could be nested expression)
                                Self::preprocess_table_condition(&arr[2])
                            };
                            
                            // Order matters: {op: [col_var, value]} means row[col] op value
                            let mut obj = serde_json::Map::new();
                            obj.insert(op_str.to_string(), Value::Array(vec![col_arg, value_arg]));
                            return Value::Object(obj);
                        }
                        
                        // Check for logical operators: [op, arg1, arg2, ...]
                        let canonical_op = match op_str {
                            "&&" => Some("and"),
                            "||" => Some("or"),
                            "and" | "or" | "!" | "not" | "if" => Some(op_str),
                            _ => None,
                        };
                        
                        if let Some(op_name) = canonical_op {
                            // Convert ["op", arg1, arg2, ...] to {"op": [arg1, arg2, ...]}
                            let args: Vec<Value> = arr[1..].iter()
                                .map(Self::preprocess_table_condition)
                                .collect();
                            let mut obj = serde_json::Map::new();
                            obj.insert(op_name.to_string(), Value::Array(args));
                            return Value::Object(obj);
                        }
                    }
                }
                // Regular array - recursively process elements
                Value::Array(arr.iter().map(Self::preprocess_table_condition).collect())
            }
            Value::Object(obj) => {
                // Recursively process object values, but preserve operators
                let mut new_obj = serde_json::Map::new();
                for (key, val) in obj {
                    // Don't convert strings inside $ref, var, or other special operators
                    if key == "$ref" || key == "ref" || key == "var" {
                        new_obj.insert(key.clone(), val.clone());
                    } else {
                        new_obj.insert(key.clone(), Self::preprocess_table_condition(val));
                    }
                }
                Value::Object(new_obj)
            }
            _ => value.clone(),
        }
    }
    
    /// Check if this is a simple reference that doesn't need caching
    pub fn is_simple_ref(&self) -> bool {
        matches!(self, CompiledLogic::Ref(_, None) | CompiledLogic::Var(_, None))
    }
    
    /// Extract all variable names referenced in this logic
    pub fn referenced_vars(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_vars(&mut vars);
        vars.sort();
        vars.dedup();
        vars
    }
    
    /// Flatten nested And operations only
    fn flatten_and(items: Vec<CompiledLogic>) -> Vec<CompiledLogic> {
        let mut flattened = Vec::new();
        for item in items {
            match item {
                CompiledLogic::And(nested) => {
                    // Recursively flatten nested And operations
                    flattened.extend(Self::flatten_and(nested));
                }
                _ => flattened.push(item),
            }
        }
        flattened
    }
    
    /// Flatten nested Or operations only
    fn flatten_or(items: Vec<CompiledLogic>) -> Vec<CompiledLogic> {
        let mut flattened = Vec::new();
        for item in items {
            match item {
                CompiledLogic::Or(nested) => {
                    // Recursively flatten nested Or operations
                    flattened.extend(Self::flatten_or(nested));
                }
                _ => flattened.push(item),
            }
        }
        flattened
    }
    
    /// Flatten nested Add operations only
    fn flatten_add(items: Vec<CompiledLogic>) -> Vec<CompiledLogic> {
        let mut flattened = Vec::new();
        for item in items {
            match item {
                CompiledLogic::Add(nested) => {
                    // Recursively flatten nested Adds
                    flattened.extend(Self::flatten_add(nested));
                }
                _ => flattened.push(item),
            }
        }
        flattened
    }
    
    /// Flatten nested Multiply operations only
    fn flatten_multiply(items: Vec<CompiledLogic>) -> Vec<CompiledLogic> {
        let mut flattened = Vec::new();
        for item in items {
            match item {
                CompiledLogic::Multiply(nested) => {
                    // Recursively flatten nested Multiplies
                    flattened.extend(Self::flatten_multiply(nested));
                }
                _ => flattened.push(item),
            }
        }
        flattened
    }
    
    /// Flatten nested Cat (concatenation) operations
    /// Combines nested Cat operations into a single flat operation
    fn flatten_cat(items: Vec<CompiledLogic>) -> Vec<CompiledLogic> {
        let mut flattened = Vec::new();
        for item in items {
            match &item {
                CompiledLogic::Cat(nested) => {
                    // Recursively flatten nested Cat operations
                    flattened.extend(Self::flatten_cat(nested.clone()));
                }
                _ => flattened.push(item),
            }
        }
        flattened
    }
    
    /// Check if this logic contains forward references (e.g., VALUEAT with $iteration + N where N > 0)
    /// Returns true if it references future iterations in a table
    pub fn has_forward_reference(&self) -> bool {
        let result = self.check_forward_reference();
        result
    }
    
    fn check_forward_reference(&self) -> bool {        
        match self {
            // VALUEAT with $iteration arithmetic
            CompiledLogic::ValueAt(table, idx_logic, col_name) => {
                // Check if index contains $iteration + positive_number
                let has_fwd = idx_logic.contains_iteration_plus_positive();
                if has_fwd {
                    return true;
                }
                // Recursively check all parameters
                let table_fwd = table.check_forward_reference();
                let idx_fwd = idx_logic.check_forward_reference();
                let col_fwd = col_name.as_ref().map(|c| c.check_forward_reference()).unwrap_or(false);
                table_fwd || idx_fwd || col_fwd
            }
            // Recursively check compound operations
            CompiledLogic::Array(arr) => {
                arr.iter().any(|item| item.check_forward_reference())
            }
            CompiledLogic::And(items) | CompiledLogic::Or(items) 
            | CompiledLogic::Add(items) | CompiledLogic::Subtract(items)
            | CompiledLogic::Multiply(items) | CompiledLogic::Divide(items)
            | CompiledLogic::Merge(items) | CompiledLogic::Cat(items)
            | CompiledLogic::Max(items) | CompiledLogic::Min(items)
            | CompiledLogic::Concat(items) | CompiledLogic::Multiplies(items)
            | CompiledLogic::Divides(items) => {
                items.iter().any(|item| item.check_forward_reference())
            }
            CompiledLogic::Not(a) | CompiledLogic::Abs(a) | CompiledLogic::Round(a)
            | CompiledLogic::RoundUp(a) | CompiledLogic::RoundDown(a)
            | CompiledLogic::Length(a) | CompiledLogic::Len(a) | CompiledLogic::IsEmpty(a)
            | CompiledLogic::Year(a) | CompiledLogic::Month(a) | CompiledLogic::Day(a) => a.check_forward_reference(),
            CompiledLogic::Return(_) => false, // Raw values don't have forward references
            CompiledLogic::If(cond, then_val, else_val) => {
                cond.check_forward_reference() || then_val.check_forward_reference() || else_val.check_forward_reference()
            }
            CompiledLogic::Equal(a, b) | CompiledLogic::StrictEqual(a, b)
            | CompiledLogic::NotEqual(a, b) | CompiledLogic::StrictNotEqual(a, b)
            | CompiledLogic::LessThan(a, b) | CompiledLogic::LessThanOrEqual(a, b)
            | CompiledLogic::GreaterThan(a, b) | CompiledLogic::GreaterThanOrEqual(a, b)
            | CompiledLogic::Modulo(a, b) | CompiledLogic::Power(a, b)
            | CompiledLogic::Map(a, b) | CompiledLogic::Filter(a, b) 
            | CompiledLogic::All(a, b) | CompiledLogic::Some(a, b) 
            | CompiledLogic::None(a, b) | CompiledLogic::In(a, b) 
            | CompiledLogic::Pow(a, b) | CompiledLogic::Xor(a, b) 
            | CompiledLogic::IfNull(a, b) | CompiledLogic::Days(a, b) 
            | CompiledLogic::SplitValue(a, b) | CompiledLogic::MaxAt(a, b) 
            | CompiledLogic::RangeOptions(a, b) => {
                a.check_forward_reference() || b.check_forward_reference()
            }
            CompiledLogic::Reduce(a, b, c) | CompiledLogic::Mid(a, b, c)
            | CompiledLogic::Date(a, b, c) | CompiledLogic::DateDif(a, b, c)
            | CompiledLogic::MapOptions(a, b, c) | CompiledLogic::For(a, b, c) => {
                a.check_forward_reference() || b.check_forward_reference() || c.check_forward_reference()
            }
            CompiledLogic::Substr(s, start, len) | CompiledLogic::Search(s, start, len)
            | CompiledLogic::SplitText(s, start, len) | CompiledLogic::YearFrac(s, start, len) => {
                s.check_forward_reference() || start.check_forward_reference() 
                || len.as_ref().map(|l| l.check_forward_reference()).unwrap_or(false)
            }
            CompiledLogic::Left(a, opt) | CompiledLogic::Right(a, opt) => {
                a.check_forward_reference() || opt.as_ref().map(|o| o.check_forward_reference()).unwrap_or(false)
            }
            CompiledLogic::Sum(a, opt1, opt2) => {
                a.check_forward_reference() 
                || opt1.as_ref().map(|o| o.check_forward_reference()).unwrap_or(false)
                || opt2.as_ref().map(|o| o.check_forward_reference()).unwrap_or(false)
            }
            CompiledLogic::IndexAt(a, b, c, opt) => {
                a.check_forward_reference() || b.check_forward_reference() 
                || c.check_forward_reference() || opt.as_ref().map(|o| o.check_forward_reference()).unwrap_or(false)
            }
            CompiledLogic::Match(table, conds) | CompiledLogic::MatchRange(table, conds)
            | CompiledLogic::Choose(table, conds) | CompiledLogic::FindIndex(table, conds) => {
                table.check_forward_reference() || conds.iter().any(|c| c.check_forward_reference())
            }
            CompiledLogic::MapOptionsIf(table, label, value, conds) => {
                table.check_forward_reference() || label.check_forward_reference() 
                || value.check_forward_reference() || conds.iter().any(|c| c.check_forward_reference())
            }
            CompiledLogic::MissingSome(min, _) => {
                min.check_forward_reference()
            }
            _ => false,
        }
    }
    
    /// Check if logic contains $iteration + positive_number pattern
    fn contains_iteration_plus_positive(&self) -> bool {
        match self {
            CompiledLogic::Add(items) => {
                // Check if one operand references $iteration and another is a positive number literal
                let has_iteration = items.iter().any(|item| {
                    item.referenced_vars().iter().any(|v| v == "$iteration")
                });

                let has_positive = items.iter().any(|item| match item {
                    CompiledLogic::Number(n) => {
                        n.parse::<f64>().unwrap_or(0.0) > 0.0
                    },
                    _ => false,
                });

                let result = has_iteration && has_positive;
                result
            }
            _ => false,
        }
    }
    
    /// Normalize JSON Schema reference path to dot notation
    /// Handles: #/schema/path, #/properties/field, /properties/field, field.path
    /// Trims /properties/ and .properties. segments
    fn normalize_ref_path(path: &str) -> String {
        let mut normalized = path.to_string();
        
        // Remove leading #/ if present
        if normalized.starts_with("#/") {
            normalized = normalized[2..].to_string();
        } else if normalized.starts_with('/') {
            normalized = normalized[1..].to_string();
        }
        
        // Replace / with . for JSON pointer notation
        normalized = normalized.replace('/', ".");
        
        // Remove /properties/ or .properties. segments
        normalized = normalized.replace("properties.", "");
        normalized = normalized.replace(".properties.", ".");
        
        // Clean up any double dots
        while normalized.contains("..") {
            normalized = normalized.replace("..", ".");
        }
        
        // Remove leading/trailing dots
        normalized = normalized.trim_matches('.').to_string();
        
        normalized
    }
    
    pub fn collect_vars(&self, vars: &mut Vec<String>) {
        match self {
            CompiledLogic::Var(name, default) => {
                vars.push(name.clone());
                if let Some(def) = default {
                    def.collect_vars(vars);
                }
            }
            CompiledLogic::Ref(path, default) => {
                // Normalize the path and add it
                vars.push(Self::normalize_ref_path(path));
                if let Some(def) = default {
                    def.collect_vars(vars);
                }
            }
            CompiledLogic::Array(arr) => {
                for item in arr {
                    item.collect_vars(vars);
                }
            }
            CompiledLogic::And(items) | CompiledLogic::Or(items) 
            | CompiledLogic::Add(items) | CompiledLogic::Subtract(items)
            | CompiledLogic::Multiply(items) | CompiledLogic::Divide(items)
            | CompiledLogic::Merge(items) | CompiledLogic::Cat(items)
            | CompiledLogic::Max(items) | CompiledLogic::Min(items)
            | CompiledLogic::Concat(items) => {
                for item in items {
                    item.collect_vars(vars);
                }
            }
            CompiledLogic::Not(a) | CompiledLogic::Abs(a) | CompiledLogic::Round(a)
            | CompiledLogic::RoundUp(a) | CompiledLogic::RoundDown(a)
            | CompiledLogic::Length(a) | CompiledLogic::Len(a) | CompiledLogic::IsEmpty(a)
            | CompiledLogic::Year(a) | CompiledLogic::Month(a) | CompiledLogic::Day(a) => {
                a.collect_vars(vars);
            }
            CompiledLogic::Return(_) => {} // Raw values don't contain vars
            CompiledLogic::If(cond, then_val, else_val) => {
                cond.collect_vars(vars);
                then_val.collect_vars(vars);
                else_val.collect_vars(vars);
            }
            CompiledLogic::Equal(a, b) | CompiledLogic::StrictEqual(a, b)
            | CompiledLogic::NotEqual(a, b) | CompiledLogic::StrictNotEqual(a, b)
            | CompiledLogic::LessThan(a, b) | CompiledLogic::LessThanOrEqual(a, b)
            | CompiledLogic::GreaterThan(a, b) | CompiledLogic::GreaterThanOrEqual(a, b)
            | CompiledLogic::Modulo(a, b) | CompiledLogic::Power(a, b)
            | CompiledLogic::Map(a, b) | CompiledLogic::Filter(a, b) 
            | CompiledLogic::All(a, b) | CompiledLogic::Some(a, b) 
            | CompiledLogic::None(a, b) | CompiledLogic::In(a, b) 
            | CompiledLogic::Pow(a, b) | CompiledLogic::Xor(a, b) 
            | CompiledLogic::IfNull(a, b) | CompiledLogic::Days(a, b) 
            | CompiledLogic::SplitValue(a, b) | CompiledLogic::MaxAt(a, b) 
            | CompiledLogic::RangeOptions(a, b) => {
                a.collect_vars(vars);
                b.collect_vars(vars);
            }
            CompiledLogic::Reduce(a, b, c) | CompiledLogic::Mid(a, b, c)
            | CompiledLogic::Date(a, b, c) | CompiledLogic::DateDif(a, b, c)
            | CompiledLogic::MapOptions(a, b, c) | CompiledLogic::For(a, b, c) => {
                a.collect_vars(vars);
                b.collect_vars(vars);
                c.collect_vars(vars);
            }
            CompiledLogic::Substr(s, start, len) | CompiledLogic::Search(s, start, len)
            | CompiledLogic::SplitText(s, start, len) | CompiledLogic::YearFrac(s, start, len) => {
                s.collect_vars(vars);
                start.collect_vars(vars);
                if let Some(l) = len {
                    l.collect_vars(vars);
                }
            }
            CompiledLogic::Left(a, opt) | CompiledLogic::Right(a, opt)
            | CompiledLogic::ValueAt(a, _, opt) => {
                a.collect_vars(vars);
                if let Some(o) = opt {
                    o.collect_vars(vars);
                }
            }
            CompiledLogic::Sum(a, opt1, opt2) => {
                a.collect_vars(vars);
                if let Some(o) = opt1 {
                    o.collect_vars(vars);
                }
                if let Some(o) = opt2 {
                    o.collect_vars(vars);
                }
            }
            CompiledLogic::IndexAt(a, b, c, opt) => {
                a.collect_vars(vars);
                b.collect_vars(vars);
                c.collect_vars(vars);
                if let Some(o) = opt {
                    o.collect_vars(vars);
                }
            }
            CompiledLogic::Match(table, conds) | CompiledLogic::MatchRange(table, conds)
            | CompiledLogic::Choose(table, conds) | CompiledLogic::FindIndex(table, conds) => {
                table.collect_vars(vars);
                for cond in conds {
                    cond.collect_vars(vars);
                }
            }
            CompiledLogic::Multiplies(items) | CompiledLogic::Divides(items) => {
                for item in items {
                    item.collect_vars(vars);
                }
            }
            CompiledLogic::MapOptionsIf(table, label, value, conds) => {
                table.collect_vars(vars);
                label.collect_vars(vars);
                value.collect_vars(vars);
                for cond in conds {
                    cond.collect_vars(vars);
                }
            }
            CompiledLogic::MissingSome(min, _) => {
                min.collect_vars(vars);
            }
            _ => {}
        }
    }
}

/// Storage for compiled logic expressions with dependency tracking
/// 
/// This store uses the global compiled logic cache to avoid recompiling
/// the same logic across different instances. Each instance maintains
/// its own local ID mapping to the global storage.
pub struct CompiledLogicStore {
    next_id: u64,
    store: AHashMap<LogicId, CompiledLogic>,
    dependencies: AHashMap<LogicId, Vec<String>>,
}

impl CompiledLogicStore {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            store: AHashMap::default(),
            dependencies: AHashMap::default(),
        }
    }
    
    /// Compile and store a JSON Logic expression
    /// 
    /// Uses global storage to avoid recompiling the same logic across instances.
    /// The logic is compiled once globally and reused, with this instance maintaining
    /// its own local ID for tracking dependencies.
    pub fn compile(&mut self, logic: &Value) -> Result<LogicId, String> {
        // Use global storage - compiles once and caches globally
        let _global_id = super::compiled_logic_store::compile_logic_value(logic)?;
        
        // Get the compiled logic from global store (O(1) lookup)
        let compiled = super::compiled_logic_store::get_compiled_logic(_global_id)
            .ok_or_else(|| "Failed to retrieve compiled logic from global store".to_string())?;
        
        // Track dependencies locally
        let deps = compiled.referenced_vars();
        
        // Assign local ID for this instance
        let id = LogicId(self.next_id);
        self.next_id += 1;
        
        // Store locally with instance-specific ID
        self.store.insert(id, compiled);
        self.dependencies.insert(id, deps);
        
        Ok(id)
    }
    
    /// Get a compiled logic by ID
    pub fn get(&self, id: &LogicId) -> Option<&CompiledLogic> {
        self.store.get(id)
    }
    
    /// Remove a compiled logic by ID
    pub fn remove(&mut self, id: &LogicId) -> Option<CompiledLogic> {
        self.dependencies.remove(id);
        self.store.remove(id)
    }
    
    /// Get dependencies for a logic ID
    pub fn get_dependencies(&self, id: &LogicId) -> Option<&[String]> {
        self.dependencies.get(id).map(|v| v.as_slice())
    }
}

impl Default for CompiledLogicStore {
    fn default() -> Self {
        Self::new()
    }
}
