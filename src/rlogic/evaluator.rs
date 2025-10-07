use rustc_hash::FxHashSet as HashSet;
use serde_json::{Value, Number};
use super::compiled::CompiledLogic;
use super::config::RLogicConfig;
use chrono::Datelike;
use serde_json::map::Map as JsonMap;

const HASH_SET_THRESHOLD: usize = 32;

/// Arithmetic operation types for fast path evaluation
#[derive(Debug, Clone, Copy)]
enum ArithOp { Add, Sub, Mul, Div }

/// High-performance evaluator for compiled JSON Logic
pub struct Evaluator {
    config: RLogicConfig,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            config: RLogicConfig::default(),
        }
    }
    
    pub fn with_config(mut self, config: RLogicConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Fast path for simple arithmetic without recursion (using f64)
    #[inline]
    fn eval_arithmetic_fast(&self, op: ArithOp, items: &[CompiledLogic], data: &Value) -> Option<Value> {
        // Only use fast path for simple cases (all literals or vars)
        if items.len() > 10 || items.iter().any(|i| !matches!(i, CompiledLogic::Number(_) | CompiledLogic::Var(_, None))) {
            return None;
        }
        
        let mut result = 0.0_f64;
        for (idx, item) in items.iter().enumerate() {
            let val = match item {
                CompiledLogic::Number(n) => n.parse::<f64>().unwrap_or(0.0),
                CompiledLogic::Var(name, _) => {
                    if let Some(v) = self.get_var(data, name) {
                        self.to_f64(v)
                    } else {
                        0.0
                    }
                },
                _ => return None,
            };
            
            if idx == 0 {
                result = val;
            } else {
                result = match op {
                    ArithOp::Add => result + val,
                    ArithOp::Sub => result - val,
                    ArithOp::Mul => result * val,
                    ArithOp::Div => if val != 0.0 { result / val } else { return Some(Value::Null); },
                };
            }
        }
        Some(Number::from_f64(result).map(Value::Number).unwrap_or(Value::Null))
    }
    
    pub fn with_recursion_limit(mut self, limit: usize) -> Self {
        self.config.recursion_limit = limit;
        self
    }
    
    /// Evaluate compiled logic against data (uses iterative evaluation for better performance)
    pub fn evaluate(&self, logic: &CompiledLogic, data: &Value) -> Result<Value, String> {
        // Try fast path for simple cases
        match logic {
            CompiledLogic::Null => return Ok(Value::Null),
            CompiledLogic::Bool(b) => return Ok(Value::Bool(*b)),
            CompiledLogic::Number(n) => {
                let f = n.parse::<f64>().unwrap_or(0.0);
                return Ok(Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null));
            },
            CompiledLogic::String(s) => return Ok(Value::String(s.clone())),
            CompiledLogic::Var(name, None) => {
                return Ok(self.get_var(data, name).cloned().unwrap_or(Value::Null));
            }
            CompiledLogic::Add(items) if items.len() <= 3 => {
                if let Some(result) = self.eval_arithmetic_fast(ArithOp::Add, items, data) {
                    return Ok(result);
                }
            }
            _ => {}
        }
        
        // Fall back to recursive evaluation for complex cases
        self.eval_with_depth(logic, data, 0)
    }
    
    /// Convert Decimal to JSON number (deprecated, use decimal_to_json)
    #[inline]
    fn to_json_number(&self, n: f64) -> Value {
        // Legacy method for compatibility with Power/Pow operations
        if n.is_finite() {
            Value::Number(Number::from_f64(n).unwrap_or_else(|| Number::from(0)))
        } else if self.config.safe_nan_handling {
            Value::Number(Number::from(0))
        } else {
            Value::Null
        }
    }
    
    /// Convert f64 to JSON number
    #[inline]
    fn f64_to_json(&self, f: f64) -> Value {
        Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null)
    }
    
    /// Convert JSON value to f64
    #[inline]
    fn to_f64(&self, value: &Value) -> f64 {
        match value {
            Value::Number(n) => n.as_f64().unwrap_or(0.0),
            Value::Bool(true) => 1.0,
            Value::Bool(false) => 0.0,
            Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
            Value::Array(arr) => {
                if arr.len() == 1 {
                    self.to_f64(&arr[0])
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
    
    fn eval_with_depth(&self, logic: &CompiledLogic, data: &Value, depth: usize) -> Result<Value, String> {
        if depth > self.config.recursion_limit {
            return Err("Recursion limit exceeded".to_string());
        }
        
        match logic {
            // Literals
            CompiledLogic::Null => Ok(Value::Null),
            CompiledLogic::Bool(b) => Ok(Value::Bool(*b)),
            CompiledLogic::Number(n) => {
                let f = n.parse::<f64>().unwrap_or(0.0);
                Ok(self.f64_to_json(f))
            },
            CompiledLogic::String(s) => Ok(Value::String(s.clone())),
            CompiledLogic::Array(arr) => {
                let results: Result<Vec<_>, _> = arr.iter()
                    .map(|item| self.eval_with_depth(item, data, depth + 1))
                    .collect();
                Ok(Value::Array(results?))
            }
            
            // Variable access
            CompiledLogic::Var(name, default) => {
                let value = self.get_var(data, name);
                match value {
                    Some(v) if !v.is_null() => Ok(v.clone()),
                    _ => {
                        if let Some(def) = default {
                            self.eval_with_depth(def, data, depth + 1)
                        } else {
                            Ok(Value::Null)
                        }
                    }
                }
            }
            
            // JSON Schema reference access
            CompiledLogic::Ref(path, default) => {
                let normalized_path = Self::normalize_ref_path(path);
                let value = self.get_var(data, &normalized_path);
                match value {
                    Some(v) if !v.is_null() => Ok(v.clone()),
                    _ => {
                        if let Some(def) = default {
                            self.eval_with_depth(def, data, depth + 1)
                        } else {
                            Ok(Value::Null)
                        }
                    }
                }
            }
            
            // Logical operators
            CompiledLogic::And(items) => {
                for item in items {
                    let result = self.eval_with_depth(item, data, depth + 1)?;
                    if !self.is_truthy(&result) {
                        return Ok(result);
                    }
                }
                Ok(items.last()
                    .map(|item| self.eval_with_depth(item, data, depth + 1))
                    .transpose()?
                    .unwrap_or(Value::Bool(true)))
            }
            CompiledLogic::Or(items) => {
                for item in items {
                    let result = self.eval_with_depth(item, data, depth + 1)?;
                    if self.is_truthy(&result) {
                        return Ok(result);
                    }
                }
                Ok(items.last()
                    .map(|item| self.eval_with_depth(item, data, depth + 1))
                    .transpose()?
                    .unwrap_or(Value::Bool(false)))
            }
            CompiledLogic::Not(expr) => {
                let result = self.eval_with_depth(expr, data, depth + 1)?;
                Ok(Value::Bool(!self.is_truthy(&result)))
            }
            CompiledLogic::If(cond, then_expr, else_expr) => {
                let condition = self.eval_with_depth(cond, data, depth + 1)?;
                if self.is_truthy(&condition) {
                    self.eval_with_depth(then_expr, data, depth + 1)
                } else {
                    self.eval_with_depth(else_expr, data, depth + 1)
                }
            }
            
            // Comparison operators
            CompiledLogic::Equal(a, b) => {
                let val_a = self.eval_with_depth(a, data, depth + 1)?;
                let val_b = self.eval_with_depth(b, data, depth + 1)?;
                Ok(Value::Bool(self.loose_equal(&val_a, &val_b)))
            }
            CompiledLogic::StrictEqual(a, b) => {
                let val_a = self.eval_with_depth(a, data, depth + 1)?;
                let val_b = self.eval_with_depth(b, data, depth + 1)?;
                Ok(Value::Bool(val_a == val_b))
            }
            CompiledLogic::NotEqual(a, b) => {
                let val_a = self.eval_with_depth(a, data, depth + 1)?;
                let val_b = self.eval_with_depth(b, data, depth + 1)?;
                Ok(Value::Bool(!self.loose_equal(&val_a, &val_b)))
            }
            CompiledLogic::StrictNotEqual(a, b) => {
                let val_a = self.eval_with_depth(a, data, depth + 1)?;
                let val_b = self.eval_with_depth(b, data, depth + 1)?;
                Ok(Value::Bool(val_a != val_b))
            }
            CompiledLogic::LessThan(a, b) => {
                let val_a = self.eval_with_depth(a, data, depth + 1)?;
                let val_b = self.eval_with_depth(b, data, depth + 1)?;
                Ok(Value::Bool(self.compare(&val_a, &val_b) < 0.0))
            }
            CompiledLogic::LessThanOrEqual(a, b) => {
                let val_a = self.eval_with_depth(a, data, depth + 1)?;
                let val_b = self.eval_with_depth(b, data, depth + 1)?;
                Ok(Value::Bool(self.compare(&val_a, &val_b) <= 0.0))
            }
            CompiledLogic::GreaterThan(a, b) => {
                let val_a = self.eval_with_depth(a, data, depth + 1)?;
                let val_b = self.eval_with_depth(b, data, depth + 1)?;
                Ok(Value::Bool(self.compare(&val_a, &val_b) > 0.0))
            }
            CompiledLogic::GreaterThanOrEqual(a, b) => {
                let val_a = self.eval_with_depth(a, data, depth + 1)?;
                let val_b = self.eval_with_depth(b, data, depth + 1)?;
                Ok(Value::Bool(self.compare(&val_a, &val_b) >= 0.0))
            }
            
            // Arithmetic operators (with fast path)
            CompiledLogic::Add(items) => {
                // Try fast path first for simple expressions
                if let Some(result) = self.eval_arithmetic_fast(ArithOp::Add, items, data) {
                    return Ok(result);
                }
                let mut sum = 0.0_f64;
                for item in items {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    sum += self.to_f64(&val);
                }
                Ok(self.f64_to_json(sum))
            }
            CompiledLogic::Subtract(items) => {
                if let Some(result) = self.eval_arithmetic_fast(ArithOp::Sub, items, data) {
                    return Ok(result);
                }
                if items.is_empty() {
                    return Ok(self.f64_to_json(0.0));
                }
                let first = self.eval_with_depth(&items[0], data, depth + 1)?;
                let mut result = self.to_f64(&first);
                
                if items.len() == 1 {
                    return Ok(self.f64_to_json(-result));
                }
                
                for item in &items[1..] {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    result -= self.to_f64(&val);
                }
                Ok(self.f64_to_json(result))
            }
            CompiledLogic::Multiply(items) => {
                if let Some(result) = self.eval_arithmetic_fast(ArithOp::Mul, items, data) {
                    return Ok(result);
                }
                let mut product = 1.0_f64;
                for item in items {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    product *= self.to_f64(&val);
                }
                Ok(self.f64_to_json(product))
            }
            CompiledLogic::Divide(items) => {
                if let Some(result) = self.eval_arithmetic_fast(ArithOp::Div, items, data) {
                    return Ok(result);
                }
                if items.is_empty() {
                    return Ok(self.f64_to_json(0.0_f64));
                }
                let first = self.eval_with_depth(&items[0], data, depth + 1)?;
                let mut result = self.to_f64(&first);
                
                for item in &items[1..] {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    let divisor = self.to_f64(&val);
                    if divisor == 0.0 {
                        return Ok(Value::Null);
                    }
                    result /= divisor;
                }
                Ok(self.f64_to_json(result))
            }
            CompiledLogic::Modulo(a, b) => {
                let val_a = self.eval_with_depth(a, data, depth + 1)?;
                let val_b = self.eval_with_depth(b, data, depth + 1)?;
                let num_a = self.to_f64(&val_a);
                let num_b = self.to_f64(&val_b);
                if num_b == 0.0 {
                    Ok(Value::Null)
                } else {
                    Ok(self.f64_to_json(num_a % num_b))
                }
            }
            CompiledLogic::Power(a, b) => {
                let val_a = self.eval_with_depth(a, data, depth + 1)?;
                let val_b = self.eval_with_depth(b, data, depth + 1)?;
                // Power requires f64 for exponentiation
                let num_a = self.to_number(&val_a);
                let num_b = self.to_number(&val_b);
                Ok(self.to_json_number(num_a.powf(num_b)))
            }
            
            // Array operators
            CompiledLogic::Map(array_expr, logic_expr) => {
                let array_val = self.eval_with_depth(array_expr, data, depth + 1)?;
                if let Value::Array(arr) = array_val {
                    let mut results = Vec::with_capacity(arr.len());
                    for item in &arr {
                        results.push(self.eval_with_depth(logic_expr, item, depth + 1)?);
                    }
                    Ok(Value::Array(results))
                } else {
                    Ok(Value::Array(vec![]))
                }
            }
            CompiledLogic::Filter(array_expr, logic_expr) => {
                let array_val = self.eval_with_depth(array_expr, data, depth + 1)?;
                if let Value::Array(arr) = array_val {
                    let mut results = Vec::with_capacity(arr.len());
                    for item in arr.into_iter() {
                        let result = self.eval_with_depth(logic_expr, &item, depth + 1)?;
                        if self.is_truthy(&result) {
                            results.push(item);
                        }
                    }
                    Ok(Value::Array(results))
                } else {
                    Ok(Value::Array(vec![]))
                }
            }
            CompiledLogic::Reduce(array_expr, logic_expr, initial_expr) => {
                let array_val = self.eval_with_depth(array_expr, data, depth + 1)?;
                let mut accumulator = self.eval_with_depth(initial_expr, data, depth + 1)?;
                
                if let Value::Array(arr) = array_val {
                    for item in arr {
                        let mut context = JsonMap::with_capacity(2);
                        context.insert("current".to_string(), item);
                        context.insert("accumulator".to_string(), accumulator);
                        let combined = Value::Object(context);
                        accumulator = self.eval_with_depth(logic_expr, &combined, depth + 1)?;
                    }
                }
                Ok(accumulator)
            }
            CompiledLogic::All(array_expr, logic_expr) => {
                let array_val = self.eval_with_depth(array_expr, data, depth + 1)?;
                if let Value::Array(arr) = array_val {
                    for item in arr {
                        let result = self.eval_with_depth(logic_expr, &item, depth + 1)?;
                        if !self.is_truthy(&result) {
                            return Ok(Value::Bool(false));
                        }
                    }
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            CompiledLogic::Some(array_expr, logic_expr) => {
                let array_val = self.eval_with_depth(array_expr, data, depth + 1)?;
                if let Value::Array(arr) = array_val {
                    for item in arr {
                        let result = self.eval_with_depth(logic_expr, &item, depth + 1)?;
                        if self.is_truthy(&result) {
                            return Ok(Value::Bool(true));
                        }
                    }
                    Ok(Value::Bool(false))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            CompiledLogic::None(array_expr, logic_expr) => {
                let array_val = self.eval_with_depth(array_expr, data, depth + 1)?;
                if let Value::Array(arr) = array_val {
                    for item in arr {
                        let result = self.eval_with_depth(logic_expr, &item, depth + 1)?;
                        if self.is_truthy(&result) {
                            return Ok(Value::Bool(false));
                        }
                    }
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::Bool(true))
                }
            }
            CompiledLogic::Merge(items) => {
                let mut merged = Vec::new();
                for item in items {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    if let Value::Array(arr) = val {
                        merged.extend(arr);
                    } else {
                        merged.push(val);
                    }
                }
                Ok(Value::Array(merged))
            }
            CompiledLogic::In(value_expr, array_expr) => {
                let value = self.eval_with_depth(value_expr, data, depth + 1)?;
                let array_val = self.eval_with_depth(array_expr, data, depth + 1)?;
                
                if let Value::Array(arr) = array_val {
                    if arr.len() > HASH_SET_THRESHOLD {
                        if let Some(key) = Self::scalar_hash_key(&value) {
                            let mut set = HashSet::with_capacity_and_hasher(arr.len(), Default::default());
                            let mut all_scalar = true;
                            for item in &arr {
                                if let Some(item_key) = Self::scalar_hash_key(item) {
                                    set.insert(item_key);
                                } else {
                                    all_scalar = false;
                                    break;
                                }
                            }
                            if all_scalar {
                                return Ok(Value::Bool(set.contains(&key)));
                            }
                        }
                    }
                    for item in arr {
                        if self.loose_equal(&value, &item) {
                            return Ok(Value::Bool(true));
                        }
                    }
                    Ok(Value::Bool(false))
                } else if let Value::String(s) = array_val {
                    if let Value::String(needle) = value {
                        Ok(Value::Bool(s.contains(&needle)))
                    } else {
                        Ok(Value::Bool(false))
                    }
                } else {
                    Ok(Value::Bool(false))
                }
            }
            
            // String operators
            CompiledLogic::Cat(items) => {
                let mut result = String::new();
                for item in items {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    result.push_str(&self.to_string(&val));
                }
                Ok(Value::String(result))
            }
            CompiledLogic::Substr(string_expr, start_expr, length_expr) => {
                let string_val = self.eval_with_depth(string_expr, data, depth + 1)?;
                let start_val = self.eval_with_depth(start_expr, data, depth + 1)?;
                
                let s = self.to_string(&string_val);
                let start = self.to_number(&start_val) as i32;
                
                let start_idx = if start < 0 {
                    (s.len() as i32 + start).max(0) as usize
                } else {
                    start.min(s.len() as i32) as usize
                };
                
                if let Some(len_expr) = length_expr {
                    let length_val = self.eval_with_depth(len_expr, data, depth + 1)?;
                    let length = self.to_number(&length_val) as usize;
                    let end_idx = (start_idx + length).min(s.len());
                    Ok(Value::String(s[start_idx..end_idx].to_string()))
                } else {
                    Ok(Value::String(s[start_idx..].to_string()))
                }
            }
            
            // Utility operators
            CompiledLogic::Missing(keys) => {
                let mut missing = Vec::new();
                for key in keys {
                    if let Some(val) = self.get_var(data, key) {
                        if val.is_null() {
                            missing.push(Value::String(key.clone()));
                        }
                    } else {
                        missing.push(Value::String(key.clone()));
                    }
                }
                Ok(Value::Array(missing))
            }
            CompiledLogic::MissingSome(min_expr, keys) => {
                let min_val = self.eval_with_depth(min_expr, data, depth + 1)?;
                let minimum = self.to_number(&min_val) as usize;
                
                let mut present = 0;
                for key in keys {
                    if let Some(val) = self.get_var(data, key) {
                        if !val.is_null() {
                            present += 1;
                        }
                    }
                }
                
                if present >= minimum {
                    Ok(Value::Array(vec![]))
                } else {
                    let missing: Vec<_> = keys.iter()
                        .filter(|key| {
                            self.get_var(data, key)
                                .map(|v| v.is_null())
                                .unwrap_or(true)
                        })
                        .map(|k| Value::String(k.clone()))
                        .collect();
                    Ok(Value::Array(missing))
                }
            }
            
            // Custom operators - Math
            CompiledLogic::Abs(expr) => {
                let val = self.eval_with_depth(expr, data, depth + 1)?;
                let num = self.to_f64(&val);
                Ok(self.f64_to_json(num.abs()))
            }
            CompiledLogic::Max(items) => {
                if items.is_empty() {
                    return Ok(Value::Null);
                }
                let first_val = self.eval_with_depth(&items[0], data, depth + 1)?;
                let mut max = self.to_f64(&first_val);
                for item in &items[1..] {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    let num = self.to_f64(&val);
                    if num > max {
                        max = num;
                    }
                }
                Ok(self.f64_to_json(max))
            }
            CompiledLogic::Min(items) => {
                if items.is_empty() {
                    return Ok(Value::Null);
                }
                let first_val = self.eval_with_depth(&items[0], data, depth + 1)?;
                let mut min = self.to_f64(&first_val);
                for item in &items[1..] {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    let num = self.to_f64(&val);
                    if num < min {
                        min = num;
                    }
                }
                Ok(self.f64_to_json(min))
            }
            CompiledLogic::Pow(base_expr, exp_expr) => {
                let base_val = self.eval_with_depth(base_expr, data, depth + 1)?;
                let exp_val = self.eval_with_depth(exp_expr, data, depth + 1)?;
                let base = self.to_number(&base_val);
                let exp = self.to_number(&exp_val);
                Ok(self.to_json_number(base.powf(exp)))
            }
            CompiledLogic::Round(expr) => {
                let val = self.eval_with_depth(expr, data, depth + 1)?;
                let num = self.to_f64(&val);
                Ok(self.f64_to_json(num.round()))
            }
            CompiledLogic::RoundUp(expr) => {
                let val = self.eval_with_depth(expr, data, depth + 1)?;
                let num = self.to_f64(&val);
                Ok(self.f64_to_json(num.ceil()))
            }
            CompiledLogic::RoundDown(expr) => {
                let val = self.eval_with_depth(expr, data, depth + 1)?;
                let num = self.to_f64(&val);
                Ok(self.f64_to_json(num.floor()))
            }
            
            // Custom operators - String
            CompiledLogic::Length(expr) => {
                let val = self.eval_with_depth(expr, data, depth + 1)?;
                let len = match &val {
                    Value::String(s) => s.len(),
                    Value::Array(arr) => arr.len(),
                    Value::Object(obj) => obj.len(),
                    _ => 0,
                };
                Ok(self.f64_to_json(len as f64))
            }
            CompiledLogic::Len(expr) => {
                let val = self.eval_with_depth(expr, data, depth + 1)?;
                let s = self.to_string(&val);
                Ok(self.f64_to_json(s.len() as f64))
            }
            CompiledLogic::Search(find_expr, within_expr, start_expr) => {
                let find_val = self.eval_with_depth(find_expr, data, depth + 1)?;
                let within_val = self.eval_with_depth(within_expr, data, depth + 1)?;
                
                if let (Value::String(find), Value::String(within)) = (&find_val, &within_val) {
                    let start = if let Some(start_e) = start_expr {
                        let start_val = self.eval_with_depth(start_e, data, depth + 1)?;
                        (self.to_number(&start_val) as usize).saturating_sub(1)
                    } else {
                        0
                    };
                    
                    if let Some(pos) = within.to_lowercase()[start..].find(&find.to_lowercase()) {
                        Ok(self.f64_to_json((pos + start + 1) as f64))
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    Ok(Value::Null)
                }
            }
            CompiledLogic::Left(text_expr, num_expr) => {
                let text_val = self.eval_with_depth(text_expr, data, depth + 1)?;
                let text = self.to_string(&text_val);
                let num_chars = if let Some(n_expr) = num_expr {
                    let n_val = self.eval_with_depth(n_expr, data, depth + 1)?;
                    self.to_number(&n_val) as usize
                } else {
                    1
                };
                Ok(Value::String(text.chars().take(num_chars).collect()))
            }
            CompiledLogic::Right(text_expr, num_expr) => {
                let text_val = self.eval_with_depth(text_expr, data, depth + 1)?;
                let text = self.to_string(&text_val);
                let num_chars = if let Some(n_expr) = num_expr {
                    let n_val = self.eval_with_depth(n_expr, data, depth + 1)?;
                    self.to_number(&n_val) as usize
                } else {
                    1
                };
                let chars: Vec<char> = text.chars().collect();
                let start = chars.len().saturating_sub(num_chars);
                Ok(Value::String(chars[start..].iter().collect()))
            }
            CompiledLogic::Mid(text_expr, start_expr, num_expr) => {
                let text_val = self.eval_with_depth(text_expr, data, depth + 1)?;
                let start_val = self.eval_with_depth(start_expr, data, depth + 1)?;
                let num_val = self.eval_with_depth(num_expr, data, depth + 1)?;
                
                let text = self.to_string(&text_val);
                let start = (self.to_number(&start_val) as usize).saturating_sub(1);
                let num_chars = self.to_number(&num_val) as usize;
                
                let chars: Vec<char> = text.chars().collect();
                let end = (start + num_chars).min(chars.len());
                Ok(Value::String(chars[start..end].iter().collect()))
            }
            CompiledLogic::SplitText(value_expr, sep_expr, index_expr) => {
                let value_val = self.eval_with_depth(value_expr, data, depth + 1)?;
                let sep_val = self.eval_with_depth(sep_expr, data, depth + 1)?;
                
                let text = self.to_string(&value_val);
                let separator = self.to_string(&sep_val);
                let index = if let Some(idx_expr) = index_expr {
                    let idx_val = self.eval_with_depth(idx_expr, data, depth + 1)?;
                    self.to_number(&idx_val) as usize
                } else {
                    0
                };
                
                let parts: Vec<&str> = text.split(&separator).collect();
                Ok(Value::String(parts.get(index).unwrap_or(&"").to_string()))
            }
            CompiledLogic::Concat(items) => {
                let mut result = String::new();
                for item in items {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    result.push_str(&self.to_string(&val));
                }
                Ok(Value::String(result))
            }
            CompiledLogic::SplitValue(string_expr, sep_expr) => {
                let string_val = self.eval_with_depth(string_expr, data, depth + 1)?;
                let sep_val = self.eval_with_depth(sep_expr, data, depth + 1)?;
                
                let text = self.to_string(&string_val);
                let separator = self.to_string(&sep_val);
                let parts: Vec<Value> = text.split(&separator)
                    .map(|s| Value::String(s.to_string()))
                    .collect();
                Ok(Value::Array(parts))
            }
            
            // Custom operators - Logical
            CompiledLogic::Xor(a_expr, b_expr) => {
                let a_val = self.eval_with_depth(a_expr, data, depth + 1)?;
                let b_val = self.eval_with_depth(b_expr, data, depth + 1)?;
                let a = self.is_truthy(&a_val);
                let b = self.is_truthy(&b_val);
                Ok(Value::Bool(a ^ b))
            }
            CompiledLogic::IfNull(cond_expr, alt_expr) => {
                let cond_val = self.eval_with_depth(cond_expr, data, depth + 1)?;
                match &cond_val {
                    Value::Null => self.eval_with_depth(alt_expr, data, depth + 1),
                    Value::String(s) if s.is_empty() => self.eval_with_depth(alt_expr, data, depth + 1),
                    Value::Number(n) if n.is_f64() && n.as_f64().unwrap().is_nan() => {
                        self.eval_with_depth(alt_expr, data, depth + 1)
                    }
                    _ => Ok(cond_val),
                }
            }
            CompiledLogic::IsEmpty(expr) => {
                let val = self.eval_with_depth(expr, data, depth + 1)?;
                let empty = match &val {
                    Value::Null => true,
                    Value::String(s) => s.is_empty(),
                    _ => false,
                };
                Ok(Value::Bool(empty))
            }
            CompiledLogic::Empty => Ok(Value::String(String::new())),
            
            // Custom operators - Date
            CompiledLogic::Today => {
                let now = chrono::Utc::now();
                let date = now.date_naive();
                // Optimize: manually build ISO date string (faster than format!)
                let mut result = String::with_capacity(24);
                result.push_str(&date.format("%Y-%m-%d").to_string());
                result.push_str("T00:00:00.000Z");
                Ok(Value::String(result))
            }
            CompiledLogic::Now => {
                let now = chrono::Utc::now();
                Ok(Value::String(now.to_rfc3339()))
            }
            CompiledLogic::Days(end_expr, start_expr) => {
                let end_val = self.eval_with_depth(end_expr, data, depth + 1)?;
                let start_val = self.eval_with_depth(start_expr, data, depth + 1)?;
                
                if let (Value::String(end), Value::String(start)) = (&end_val, &start_val) {
                    use chrono::NaiveDate;
                    let end_date = NaiveDate::parse_from_str(end, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .ok()
                        .or_else(|| NaiveDate::parse_from_str(end, "%Y-%m-%d").ok());
                    let start_date = NaiveDate::parse_from_str(start, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .ok()
                        .or_else(|| NaiveDate::parse_from_str(start, "%Y-%m-%d").ok());
                    
                    if let (Some(e), Some(s)) = (end_date, start_date) {
                        Ok(self.f64_to_json((e - s).num_days() as f64))
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    Ok(Value::Null)
                }
            }
            CompiledLogic::Year(expr) => {
                let val = self.eval_with_depth(expr, data, depth + 1)?;
                if let Value::String(date_str) = &val {
                    use chrono::NaiveDate;
                    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .ok()
                        .or_else(|| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok());
                    if let Some(d) = date {
                        Ok(self.f64_to_json(d.year() as f64))
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    Ok(Value::Null)
                }
            }
            CompiledLogic::Month(expr) => {
                let val = self.eval_with_depth(expr, data, depth + 1)?;
                if let Value::String(date_str) = &val {
                    use chrono::NaiveDate;
                    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .ok()
                        .or_else(|| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok());
                    if let Some(d) = date {
                        Ok(self.f64_to_json(d.month() as f64))
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    Ok(Value::Null)
                }
            }
            CompiledLogic::Day(expr) => {
                let val = self.eval_with_depth(expr, data, depth + 1)?;
                if let Value::String(date_str) = &val {
                    use chrono::NaiveDate;
                    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .ok()
                        .or_else(|| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok());
                    if let Some(d) = date {
                        Ok(self.f64_to_json(d.day() as f64))
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    Ok(Value::Null)
                }
            }
            CompiledLogic::Date(year_expr, month_expr, day_expr) => {
                let year_val = self.eval_with_depth(year_expr, data, depth + 1)?;
                let month_val = self.eval_with_depth(month_expr, data, depth + 1)?;
                let day_val = self.eval_with_depth(day_expr, data, depth + 1)?;
                
                let year = self.to_number(&year_val) as i32;
                let month = self.to_number(&month_val) as u32;
                let day = self.to_number(&day_val) as u32;
                
                use chrono::NaiveDate;
                if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                    // Optimize: build ISO date string efficiently
                    let mut result = String::with_capacity(24);
                    result.push_str(&date.format("%Y-%m-%d").to_string());
                    result.push_str("T00:00:00.000Z");
                    Ok(Value::String(result))
                } else {
                    Ok(Value::Null)
                }
            }
            
            // Custom operators - Array/Table
            CompiledLogic::Sum(array_expr, field_expr) => {
                let array_val = self.eval_with_depth(array_expr, data, depth + 1)?;
                let mut sum = 0.0_f64;
                
                match &array_val {
                    Value::Array(arr) => {
                        if let Some(field_e) = field_expr {
                            let field_val = self.eval_with_depth(field_e, data, depth + 1)?;
                            if let Value::String(field_name) = field_val {
                                for item in arr {
                                    if let Value::Object(obj) = item {
                                        if let Some(val) = obj.get(&field_name) {
                                            sum += self.to_f64(val);
                                        }
                                    }
                                }
                            }
                        } else {
                            for item in arr {
                                sum += self.to_f64(item);
                            }
                        }
                    }
                    _ => {
                        sum = self.to_f64(&array_val);
                    }
                }
                
                Ok(self.f64_to_json(sum))
            }
            CompiledLogic::For(start_expr, end_expr, logic_expr) => {
                let next_depth = depth + 1;
                let start_val = self.eval_with_depth(start_expr, data, next_depth)?;
                let end_val = self.eval_with_depth(end_expr, data, next_depth)?;
                let start = self.to_number(&start_val) as i64;
                let end = self.to_number(&end_val) as i64;

                // CRITICAL: FOR returns an ARRAY of all iteration results (for use with MULTIPLIES, etc.)
                let mut results = Vec::new();
                let mut iteration_context = match data {
                    Value::Object(obj) => Value::Object(obj.clone()),
                    _ => Value::Object(JsonMap::new()),
                };

                for i in start..end {
                    if let Some(map) = iteration_context.as_object_mut() {
                        // CRITICAL FIX: FOR operator uses $loopIteration, not $iteration
                        // $iteration is reserved for table $repeat to avoid conflicts
                        map.insert("$loopIteration".to_string(), Value::from(i));
                    }
                    // println!("{}", iteration_context);

                    // panic!("test");
                    let result = self.eval_with_depth(logic_expr, &iteration_context, next_depth)?;
                    results.push(result);
                }

                Ok(Value::Array(results))
            }
            
            // Complex table operations
            CompiledLogic::ValueAt(table_expr, row_idx_expr, col_name_expr) => {
                // OPTIMIZATION: Try to get table reference directly without cloning
                let table_ref = match table_expr.as_ref() {
                    CompiledLogic::Var(name, _) => self.get_var(data, name),
                    CompiledLogic::Ref(path, _) => {
                        let normalized_path = Self::normalize_ref_path(path);
                        self.get_var(data, &normalized_path)
                    }
                    _ => None,
                };
                
                let row_idx_val = self.eval_with_depth(row_idx_expr, data, depth + 1)?;
                let row_idx_num = self.to_number(&row_idx_val) as i64;
                
                
                // CRITICAL: Return Null for negative indices (out of bounds)
                // This handles cases like VALUEAT(table, $iteration - 1, col) when iteration=0
                if row_idx_num < 0 {
                    return Ok(Value::Null);
                }
                
                let row_idx = row_idx_num as usize;
                
                // Use direct reference if available, otherwise evaluate
                let result = if let Some(table_val) = table_ref {
                    match table_val {
                        Value::Array(arr) => {
                            if row_idx >= arr.len() {
                                Value::Null
                            } else {
                                let row = &arr[row_idx];
                                
                                if let Some(col_expr) = col_name_expr {
                                    let col_val = self.eval_with_depth(col_expr, data, depth + 1)?;
                                    if let Value::String(col_name) = col_val {
                                        if let Value::Object(obj) = row {
                                            obj.get(&col_name).cloned().unwrap_or(Value::Null)
                                        } else {
                                            Value::Null
                                        }
                                    } else {
                                        Value::Null
                                    }
                                } else {
                                    row.clone()
                                }
                            }
                        }
                        _ => Value::Null,
                    }
                } else {
                    // Fallback to evaluating table expression
                    let table_val = self.eval_with_depth(table_expr, data, depth + 1)?;
                    match &table_val {
                        Value::Array(arr) => {
                            if row_idx >= arr.len() {
                                Value::Null
                            } else {
                                let row = &arr[row_idx];
                                
                                if let Some(col_expr) = col_name_expr {
                                    let col_val = self.eval_with_depth(col_expr, data, depth + 1)?;
                                    if let Value::String(col_name) = col_val {
                                        if let Value::Object(obj) = row {
                                            obj.get(&col_name).cloned().unwrap_or(Value::Null)
                                        } else {
                                            Value::Null
                                        }
                                    } else {
                                        Value::Null
                                    }
                                } else {
                                    row.clone()
                                }
                            }
                        }
                        _ => Value::Null,
                    }
                };
                
                Ok(result)
            }
            CompiledLogic::MaxAt(table_expr, col_name_expr) => {
                let table_val = self.eval_with_depth(table_expr, data, depth + 1)?;
                let col_name_val = self.eval_with_depth(col_name_expr, data, depth + 1)?;
                
                if let (Value::Array(arr), Value::String(col_name)) = (&table_val, &col_name_val) {
                    if let Some(last_row) = arr.last() {
                        if let Value::Object(obj) = last_row {
                            Ok(obj.get(col_name).cloned().unwrap_or(Value::Null))
                        } else {
                            Ok(Value::Null)
                        }
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    Ok(Value::Null)
                }
            }
            CompiledLogic::IndexAt(lookup_expr, table_expr, field_expr, range_expr) => {
                let lookup_val = self.eval_with_depth(lookup_expr, data, depth + 1)?;
                let table_val = self.eval_with_depth(table_expr, data, depth + 1)?;
                let field_val = self.eval_with_depth(field_expr, data, depth + 1)?;
                let is_range = if let Some(r_expr) = range_expr {
                    let r_val = self.eval_with_depth(r_expr, data, depth + 1)?;
                    self.is_truthy(&r_val)
                } else {
                    false
                };
                
                if let (Value::Array(arr), Value::String(field)) = (&table_val, &field_val) {
                    let lookup_num = self.to_number(&lookup_val);
                    let mut found_idx = -1i32;
                    
                    for (idx, row) in arr.iter().enumerate() {
                        if let Value::Object(obj) = row {
                            if let Some(cell_val) = obj.get(field) {
                                if is_range {
                                    let cell_num = self.to_number(cell_val);
                                    if cell_num <= lookup_num {
                                        found_idx = idx as i32;
                                    }
                                } else {
                                    if self.loose_equal(&lookup_val, cell_val) {
                                        return Ok(self.f64_to_json(idx as f64));
                                    }
                                }
                            }
                        }
                    }
                    Ok(self.f64_to_json(found_idx as f64))
                } else {
                    Ok(self.f64_to_json(-1.0))
                }
            }
            CompiledLogic::Match(table_expr, conditions) => {
                let table_val = self.eval_with_depth(table_expr, data, depth + 1)?;
                
                if let Value::Array(arr) = &table_val {
                    for (idx, row) in arr.iter().enumerate() {
                        if let Value::Object(obj) = row {
                            let mut all_match = true;
                            
                            // Conditions come in pairs: [value, field, value, field, ...]
                            for chunk in conditions.chunks(2) {
                                if chunk.len() == 2 {
                                    let value_val = self.eval_with_depth(&chunk[0], data, depth + 1)?;
                                    let field_val = self.eval_with_depth(&chunk[1], data, depth + 1)?;
                                    
                                    if let Value::String(field) = field_val {
                                        if let Some(cell_val) = obj.get(&field) {
                                            if !self.loose_equal(&value_val, cell_val) {
                                                all_match = false;
                                                break;
                                            }
                                        } else {
                                            all_match = false;
                                            break;
                                        }
                                    }
                                }
                            }
                            
                            if all_match {
                                return Ok(self.f64_to_json(idx as f64));
                            }
                        }
                    }
                    Ok(self.f64_to_json(-1.0))
                } else {
                    Ok(Value::Null)
                }
            }
            CompiledLogic::MatchRange(table_expr, conditions) => {
                let table_val = self.eval_with_depth(table_expr, data, depth + 1)?;
                
                if let Value::Array(arr) = &table_val {
                    for (idx, row) in arr.iter().enumerate() {
                        if let Value::Object(obj) = row {
                            let mut all_match = true;
                            
                            // Conditions come in triplets: [min_col, max_col, value, ...]
                            for chunk in conditions.chunks(3) {
                                if chunk.len() == 3 {
                                    let min_col_val = self.eval_with_depth(&chunk[0], data, depth + 1)?;
                                    let max_col_val = self.eval_with_depth(&chunk[1], data, depth + 1)?;
                                    let check_val = self.eval_with_depth(&chunk[2], data, depth + 1)?;
                                    
                                    if let (Value::String(min_col), Value::String(max_col)) = (&min_col_val, &max_col_val) {
                                        let min_num = obj.get(min_col).map(|v| self.to_number(v)).unwrap_or(0.0);
                                        let max_num = obj.get(max_col).map(|v| self.to_number(v)).unwrap_or(0.0);
                                        let check_num = self.to_number(&check_val);
                                        
                                        if !(check_num >= min_num && check_num <= max_num) {
                                            all_match = false;
                                            break;
                                        }
                                    }
                                }
                            }
                            
                            if all_match {
                                return Ok(self.f64_to_json(idx as f64));
                            }
                        }
                    }
                    Ok(self.f64_to_json(-1.0))
                } else {
                    Ok(Value::Null)
                }
            }
            CompiledLogic::Choose(table_expr, conditions) => {
                let table_val = self.eval_with_depth(table_expr, data, depth + 1)?;
                
                if let Value::Array(arr) = &table_val {
                    for (idx, row) in arr.iter().enumerate() {
                        if let Value::Object(obj) = row {
                            let mut any_match = false;
                            
                            // Conditions come in pairs: [value, field, value, field, ...]
                            for chunk in conditions.chunks(2) {
                                if chunk.len() == 2 {
                                    let value_val = self.eval_with_depth(&chunk[0], data, depth + 1)?;
                                    let field_val = self.eval_with_depth(&chunk[1], data, depth + 1)?;
                                    
                                    if let Value::String(field) = field_val {
                                        if let Some(cell_val) = obj.get(&field) {
                                            if self.loose_equal(&value_val, cell_val) {
                                                any_match = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            
                            if any_match {
                                return Ok(self.f64_to_json(idx as f64));
                            }
                        }
                    }
                    Ok(self.f64_to_json(-1.0))
                } else {
                    Ok(Value::Null)
                }
            }
            CompiledLogic::FindIndex(table_expr, conditions) => {
                let table_val = self.eval_with_depth(table_expr, data, depth + 1)?;
                
                if let Value::Array(arr) = &table_val {
                    for (idx, row) in arr.iter().enumerate() {
                        let mut all_match = true;
                        
                        // Evaluate all conditions against this row
                        for condition in conditions {
                            let result = self.eval_with_depth(condition, row, depth + 1)?;
                            if !self.is_truthy(&result) {
                                all_match = false;
                                break;
                            }
                        }
                        
                        if all_match {
                            return Ok(self.f64_to_json(idx as f64));
                        }
                    }
                    Ok(self.f64_to_json(-1.0))
                } else {
                    Ok(Value::Null)
                }
            }
            
            // Array operations
            CompiledLogic::Multiplies(items) => {
                let mut values = Vec::new();
                
                for item in items {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    if let Value::Array(arr) = val {
                        for elem in arr {
                            values.push(self.to_f64(&elem));
                        }
                    } else {
                        values.push(self.to_f64(&val));
                    }
                }
                
                if values.is_empty() {
                    return Ok(Value::Null);
                }
                
                if values.len() == 1 {
                    return Ok(self.f64_to_json(values[0]));
                }
                
                let result = values.iter().skip(1).fold(values[0], |acc, n| acc * n);
                Ok(self.f64_to_json(result))
            }
            CompiledLogic::Divides(items) => {
                let mut values = Vec::new();
                
                for item in items {
                    let val = self.eval_with_depth(item, data, depth + 1)?;
                    if let Value::Array(arr) = val {
                        for elem in arr {
                            values.push(self.to_f64(&elem));
                        }
                    } else {
                        values.push(self.to_f64(&val));
                    }
                }
                
                if values.is_empty() {
                    return Ok(Value::Null);
                }
                
                if values.len() == 1 {
                    return Ok(self.f64_to_json(values[0]));
                }
                
                let result = values.iter().skip(1).fold(values[0], |acc, n| {
                    if *n == 0.0 {
                        acc
                    } else {
                        acc / n
                    }
                });
                Ok(self.f64_to_json(result))
            }
            
            // Advanced date functions
            CompiledLogic::YearFrac(start_expr, end_expr, basis_expr) => {
                use chrono::NaiveDate;
                
                let start_val = self.eval_with_depth(start_expr, data, depth + 1)?;
                let end_val = self.eval_with_depth(end_expr, data, depth + 1)?;
                let basis = if let Some(b_expr) = basis_expr {
                    let b_val = self.eval_with_depth(b_expr, data, depth + 1)?;
                    self.to_number(&b_val) as i32
                } else {
                    0
                };
                
                if let (Value::String(start_str), Value::String(end_str)) = (&start_val, &end_val) {
                    let start_date = NaiveDate::parse_from_str(start_str, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .ok()
                        .or_else(|| NaiveDate::parse_from_str(start_str, "%Y-%m-%d").ok());
                    let end_date = NaiveDate::parse_from_str(end_str, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .ok()
                        .or_else(|| NaiveDate::parse_from_str(end_str, "%Y-%m-%d").ok());
                    
                    if let (Some(start), Some(end)) = (start_date, end_date) {
                        let days = (end - start).num_days() as f64;
                        
                        let result = match basis {
                            0 => days / 360.0, // 30/360 (simplified)
                            1 => days / 365.25, // Actual/actual (simplified)
                            2 => days / 360.0, // Actual/360
                            3 => days / 365.0, // Actual/365
                            4 => days / 360.0, // European 30/360
                            _ => days / 365.0,
                        };
                        
                        Ok(self.f64_to_json(result))
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    Ok(Value::Null)
                }
            }
            CompiledLogic::DateDif(start_expr, end_expr, unit_expr) => {
                use chrono::NaiveDate;
                
                let start_val = self.eval_with_depth(start_expr, data, depth + 1)?;
                let end_val = self.eval_with_depth(end_expr, data, depth + 1)?;
                let unit_val = self.eval_with_depth(unit_expr, data, depth + 1)?;
                
                if let (Value::String(start_str), Value::String(end_str), Value::String(unit)) = 
                    (&start_val, &end_val, &unit_val) {
                    let start_date = NaiveDate::parse_from_str(start_str, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .ok()
                        .or_else(|| NaiveDate::parse_from_str(start_str, "%Y-%m-%d").ok());
                    let end_date = NaiveDate::parse_from_str(end_str, "%Y-%m-%dT%H:%M:%S%.fZ")
                        .ok()
                        .or_else(|| NaiveDate::parse_from_str(end_str, "%Y-%m-%d").ok());
                    
                    if let (Some(start), Some(end)) = (start_date, end_date) {
                        let result = match unit.to_uppercase().as_str() {
                            "D" => (end - start).num_days() as f64,
                            "M" => {
                                let years = end.year() - start.year();
                                let months = end.month() as i32 - start.month() as i32;
                                let mut total_months = years * 12 + months;
                                if end.day() < start.day() {
                                    total_months -= 1;
                                }
                                total_months as f64
                            }
                            "Y" => {
                                let mut years = end.year() - start.year();
                                if end.month() < start.month() || 
                                   (end.month() == start.month() && end.day() < start.day()) {
                                    years -= 1;
                                }
                                years as f64
                            }
                            "MD" => {
                                if start.day() <= end.day() {
                                    (end.day() - start.day()) as f64
                                } else {
                                    let days_in_month = 30u32; // Simplified
                                    (days_in_month as i32 - (start.day() as i32 - end.day() as i32)) as f64
                                }
                            }
                            "YM" => {
                                let months = end.month() as i32 - start.month() as i32;
                                let mut result = if months < 0 { months + 12 } else { months };
                                if end.day() < start.day() {
                                    result -= 1;
                                    if result < 0 {
                                        result += 12;
                                    }
                                }
                                result as f64
                            }
                            "YD" => {
                                let mut temp_start = start.with_year(end.year()).unwrap_or(start);
                                if temp_start > end {
                                    temp_start = start.with_year(end.year() - 1).unwrap_or(start);
                                }
                                (end - temp_start).num_days() as f64
                            }
                            _ => return Ok(Value::Null),
                        };
                        
                        Ok(self.f64_to_json(result))
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    Ok(Value::Null)
                }
            }
            
            // UI helpers
            CompiledLogic::RangeOptions(min_expr, max_expr) => {
                let min_val = self.eval_with_depth(min_expr, data, depth + 1)?;
                let max_val = self.eval_with_depth(max_expr, data, depth + 1)?;
                
                let min = self.to_number(&min_val) as i32;
                let max = self.to_number(&max_val) as i32;
                
                if min > max {
                    return Ok(Value::Array(vec![]));
                }
                
                let options: Vec<Value> = (min..=max)
                    .map(|i| {
                        serde_json::json!({
                            "label": i.to_string(),
                            "value": i.to_string()
                        })
                    })
                    .collect();
                
                Ok(Value::Array(options))
            }
            CompiledLogic::MapOptions(table_expr, label_expr, value_expr) => {
                let table_val = self.eval_with_depth(table_expr, data, depth + 1)?;
                let label_val = self.eval_with_depth(label_expr, data, depth + 1)?;
                let value_val = self.eval_with_depth(value_expr, data, depth + 1)?;
                
                if let (Value::Array(arr), Value::String(label_field), Value::String(value_field)) = 
                    (&table_val, &label_val, &value_val) {
                    let options: Vec<Value> = arr.iter()
                        .filter_map(|row| {
                            if let Value::Object(obj) = row {
                                let label = obj.get(label_field)?;
                                let value = obj.get(value_field)?;
                                Some(serde_json::json!({
                                    "label": label,
                                    "value": value
                                }))
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    Ok(Value::Array(options))
                } else {
                    Ok(Value::Array(vec![]))
                }
            }
            CompiledLogic::MapOptionsIf(table_expr, label_expr, value_expr, conditions) => {
                let table_val = self.eval_with_depth(table_expr, data, depth + 1)?;
                let label_val = self.eval_with_depth(label_expr, data, depth + 1)?;
                let value_val = self.eval_with_depth(value_expr, data, depth + 1)?;
                
                if let (Value::Array(arr), Value::String(label_field), Value::String(value_field)) = 
                    (&table_val, &label_val, &value_val) {
                    
                    // Evaluate conditions (should be [comp_value, comparator, lookup_field])
                    let comp_value = if !conditions.is_empty() {
                        self.eval_with_depth(&conditions[0], data, depth + 1)?
                    } else {
                        Value::Null
                    };
                    let comparator = if conditions.len() > 1 {
                        self.eval_with_depth(&conditions[1], data, depth + 1)?
                    } else {
                        Value::String("==".to_string())
                    };
                    let lookup_field = if conditions.len() > 2 {
                        self.eval_with_depth(&conditions[2], data, depth + 1)?
                    } else {
                        Value::Null
                    };
                    
                    let options: Vec<Value> = arr.iter()
                        .filter_map(|row| {
                            if let Value::Object(obj) = row {
                                // Apply filter condition
                                if let (Value::String(comp_op), Value::String(lookup_f)) = (&comparator, &lookup_field) {
                                    if let Some(cell_val) = obj.get(lookup_f) {
                                        let matches = match comp_op.as_str() {
                                            "==" => self.loose_equal(&comp_value, cell_val),
                                            "!=" => !self.loose_equal(&comp_value, cell_val),
                                            "<" => self.to_number(&comp_value) < self.to_number(cell_val),
                                            "<=" => self.to_number(&comp_value) <= self.to_number(cell_val),
                                            ">" => self.to_number(&comp_value) > self.to_number(cell_val),
                                            ">=" => self.to_number(&comp_value) >= self.to_number(cell_val),
                                            _ => true,
                                        };
                                        
                                        if !matches {
                                            return None;
                                        }
                                    }
                                }
                                
                                let label = obj.get(label_field)?;
                                let value = obj.get(value_field)?;
                                Some(serde_json::json!({
                                    "label": label,
                                    "value": value
                                }))
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    Ok(Value::Array(options))
                } else {
                    Ok(Value::Array(vec![]))
                }
            }
            CompiledLogic::Return(expr) => {
                // Simply evaluate and return the expression
                self.eval_with_depth(expr, data, depth + 1)
            }
        }
    }
    
    // Helper methods
    
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
    
    fn get_var<'a>(&self, data: &'a Value, name: &str) -> Option<&'a Value> {
        if name.is_empty() {
            return Some(data);
        }
        
        let parts: Vec<&str> = name.split('.').collect();
        let mut current = data;
        
        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        current = arr.get(index)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        
        Some(current)
    }
    
    #[inline]
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(_) => true,
        }
    }
    
    /// Legacy to_number for f64 (only used for Power operations)
    #[inline]
    fn to_number(&self, value: &Value) -> f64 {
        self.to_f64(value)
    }
    
    #[inline]
    fn to_string(&self, value: &Value) -> String {
        match value {
            Value::Null => String::new(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => {
                // JavaScript-like number to string conversion:
                // Integer-valued numbers should not have decimal point
                if let Some(f) = n.as_f64() {
                    if f.is_finite() && f == f.floor() && f.abs() < 1e15 {
                        // It's an integer value, format without decimal
                        format!("{}", f as i64)
                    } else {
                        n.to_string()
                    }
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Array(_) | Value::Object(_) => value.to_string(),
        }
    }
    
    fn loose_equal(&self, a: &Value, b: &Value) -> bool {
        // JavaScript-like type coercion for loose equality (==)
        match (a, b) {
            // Same type comparisons
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => {
                let a_f64 = a.as_f64().unwrap_or(0.0);
                let b_f64 = b.as_f64().unwrap_or(0.0);
                a_f64 == b_f64
            }
            (Value::String(a), Value::String(b)) => a == b,
            
            // Number and String: convert string to number
            (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
                let n_val = n.as_f64().unwrap_or(0.0);
                // Empty string converts to 0 in JavaScript
                if s.is_empty() {
                    return n_val == 0.0;
                }
                if let Ok(parsed) = s.parse::<f64>() {
                    n_val == parsed
                } else {
                    false
                }
            }
            
            // Boolean and Number: convert boolean to number (true=1, false=0)
            (Value::Bool(b), Value::Number(n)) | (Value::Number(n), Value::Bool(b)) => {
                let b_num = if *b { 1.0 } else { 0.0 };
                let n_val = n.as_f64().unwrap_or(0.0);
                b_num == n_val
            }
            
            // Boolean and String: convert both to number
            (Value::Bool(b), Value::String(s)) | (Value::String(s), Value::Bool(b)) => {
                let b_num = if *b { 1.0 } else { 0.0 };
                // Empty string converts to 0
                if s.is_empty() {
                    return b_num == 0.0;
                }
                if let Ok(parsed) = s.parse::<f64>() {
                    b_num == parsed
                } else {
                    false
                }
            }
            
            // Null comparisons: null only equals null (and undefined, but we don't have that)
            (Value::Null, _) | (_, Value::Null) => false,
            
            // Default: strict equality
            _ => a == b,
        }
    }
    
    #[inline]
    fn compare(&self, a: &Value, b: &Value) -> f64 {
        let num_a = self.to_f64(a);
        let num_b = self.to_f64(b);
        num_a - num_b
    }
    
    #[inline]
    fn scalar_hash_key(value: &Value) -> Option<String> {
        match value {
            Value::Null => Some(String::from("null")),
            Value::Bool(b) => Some(b.to_string()),
            Value::Number(n) => Some(n.to_string()),
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

