use super::{Evaluator, types::*};
use serde_json::{Value, Map as JsonMap};
use super::super::compiled::CompiledLogic;
use super::helpers;
use rayon::prelude::*;

const PARALLEL_THRESHOLD: usize = 10; // Parallelize arrays with 10+ elements for better performance

impl Evaluator {
    /// Execute array quantifier (all/some/none) - ZERO-COPY
    pub(super) fn eval_quantifier(&self, quantifier: Quantifier, array_expr: &CompiledLogic, logic_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let array_val = self.evaluate_with_context(array_expr, user_data, internal_context, depth + 1)?;
        if let Value::Array(arr) = array_val {
            // Parallelize for large arrays
            if arr.len() >= PARALLEL_THRESHOLD {
                let result = match quantifier {
                    Quantifier::All => {
                        arr.par_iter()
                            .try_fold(|| true, |_, item| -> Result<bool, String> {
                                // Use item as user_data, no internal context needed
                                let result = self.evaluate_with_context(logic_expr, item, &Value::Null, depth + 1)?;
                                Ok(helpers::is_truthy(&result))
                            })
                            .try_reduce(|| true, |a, b| -> Result<bool, String> { Ok(a && b) })?
                    },
                    Quantifier::Some => {
                        arr.par_iter()
                            .try_fold(|| false, |_, item| -> Result<bool, String> {
                                let result = self.evaluate_with_context(logic_expr, item, &Value::Null, depth + 1)?;
                                Ok(helpers::is_truthy(&result))
                            })
                            .try_reduce(|| false, |a, b| -> Result<bool, String> { Ok(a || b) })?
                    },
                    Quantifier::None => {
                        arr.par_iter()
                            .try_fold(|| true, |_, item| -> Result<bool, String> {
                                let result = self.evaluate_with_context(logic_expr, item, &Value::Null, depth + 1)?;
                                Ok(!helpers::is_truthy(&result))
                            })
                            .try_reduce(|| true, |a, b| -> Result<bool, String> { Ok(a && b) })?
                    }
                };
                Ok(Value::Bool(result))
            } else {
                for item in arr {
                    let result = self.evaluate_with_context(logic_expr, &item, &Value::Null, depth + 1)?;
                    let truthy = helpers::is_truthy(&result);
                    match quantifier {
                        Quantifier::All if !truthy => return Ok(Value::Bool(false)),
                        Quantifier::Some if truthy => return Ok(Value::Bool(true)),
                        Quantifier::None if truthy => return Ok(Value::Bool(false)),
                        _ => {}
                    }
                }
                Ok(Value::Bool(match quantifier {
                    Quantifier::All => true,
                    Quantifier::Some => false,
                    Quantifier::None => true,
                }))
            }
        } else {
            Ok(Value::Bool(match quantifier {
                Quantifier::All | Quantifier::Some => false,
                Quantifier::None => true,
            }))
        }
    }

    /// Evaluate map operation - ZERO-COPY
    pub(super) fn eval_map(&self, array_expr: &CompiledLogic, logic_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let array_val = self.evaluate_with_context(array_expr, user_data, internal_context, depth + 1)?;
        if let Value::Array(arr) = array_val {
            // Parallelize for large arrays
            if arr.len() >= PARALLEL_THRESHOLD {
                let results: Result<Vec<_>, String> = arr.par_iter()
                    .map(|item| self.evaluate_with_context(logic_expr, item, &Value::Null, depth + 1))
                    .collect();
                Ok(Value::Array(results?))
            } else {
                let mut results = Vec::with_capacity(arr.len());
                for item in &arr {
                    results.push(self.evaluate_with_context(logic_expr, item, &Value::Null, depth + 1)?);
                }
                Ok(Value::Array(results))
            }
        } else {
            Ok(Value::Array(vec![]))
        }
    }

    /// Evaluate filter operation - ZERO-COPY
    pub(super) fn eval_filter(&self, array_expr: &CompiledLogic, logic_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let array_val = self.evaluate_with_context(array_expr, user_data, internal_context, depth + 1)?;
        if let Value::Array(arr) = array_val {
            // Parallelize for large arrays
            if arr.len() >= PARALLEL_THRESHOLD {
                let results: Result<Vec<_>, String> = arr.into_par_iter()
                    .filter_map(|item| {
                        match self.evaluate_with_context(logic_expr, &item, &Value::Null, depth + 1) {
                            Ok(result) if helpers::is_truthy(&result) => Some(Ok(item)),
                            Ok(_) => None,
                            Err(e) => Some(Err(e)),
                        }
                    })
                    .collect();
                Ok(Value::Array(results?))
            } else {
                let mut results = Vec::with_capacity(arr.len());
                for item in arr.into_iter() {
                    let result = self.evaluate_with_context(logic_expr, &item, &Value::Null, depth + 1)?;
                    if helpers::is_truthy(&result) {
                        results.push(item);
                    }
                }
                Ok(Value::Array(results))
            }
        } else {
            Ok(Value::Array(vec![]))
        }
    }

    /// Evaluate reduce operation - ZERO-COPY
    pub(super) fn eval_reduce(&self, array_expr: &CompiledLogic, logic_expr: &CompiledLogic, initial_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let array_val = self.evaluate_with_context(array_expr, user_data, internal_context, depth + 1)?;
        let mut accumulator = self.evaluate_with_context(initial_expr, user_data, internal_context, depth + 1)?;

        if let Value::Array(arr) = array_val {
            for item in arr {
                // Create small context with current and accumulator
                let mut context = JsonMap::with_capacity(2);
                context.insert("current".to_string(), item);
                context.insert("accumulator".to_string(), accumulator);
                let combined = Value::Object(context);
                accumulator = self.evaluate_with_context(logic_expr, &combined, &Value::Null, depth + 1)?;
            }
        }
        Ok(accumulator)
    }

    /// Evaluate merge operation - ZERO-COPY
    pub(super) fn eval_merge(&self, items: &[CompiledLogic], user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let mut merged = Vec::new();
        for item in items {
            let val = self.evaluate_with_context(item, user_data, internal_context, depth + 1)?;
            if let Value::Array(arr) = val {
                merged.extend(arr);
            } else {
                merged.push(val);
            }
        }
        Ok(Value::Array(merged))
    }

    /// Evaluate in operation (check if value exists in array) - ZERO-COPY
    pub(super) fn eval_in(&self, value_expr: &CompiledLogic, array_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        use ahash::{AHashSet, RandomState};

        const HASH_SET_THRESHOLD: usize = 32;

        let value = self.evaluate_with_context(value_expr, user_data, internal_context, depth + 1)?;
        let array_val = self.evaluate_with_context(array_expr, user_data, internal_context, depth + 1)?;

        if let Value::Array(arr) = array_val {
            if arr.len() > HASH_SET_THRESHOLD {
                if let Some(key) = helpers::scalar_hash_key(&value) {
                    let mut set = AHashSet::with_capacity_and_hasher(arr.len(), RandomState::new());
                    let mut all_scalar = true;
                    for item in &arr {
                        if let Some(item_key) = helpers::scalar_hash_key(item) {
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
                if helpers::loose_equal(&value, &item) {
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

    /// Evaluate Sum operation - ZERO-COPY
    pub(super) fn eval_sum(&self, array_expr: &CompiledLogic, field_expr: &Option<Box<CompiledLogic>>, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let array_val = self.evaluate_with_context(array_expr, user_data, internal_context, depth + 1)?;

        let sum = match &array_val {
            Value::Array(arr) => {
                if let Some(field_e) = field_expr {
                    let field_val = self.evaluate_with_context(field_e, user_data, internal_context, depth + 1)?;
                    if let Value::String(field_name) = field_val {
                        // Parallelize for large arrays
                        if arr.len() >= PARALLEL_THRESHOLD {
                            arr.par_iter()
                                .filter_map(|item| {
                                    if let Value::Object(obj) = item {
                                        obj.get(&field_name).map(|val| helpers::to_f64(val))
                                    } else {
                                        None
                                    }
                                })
                                .sum()
                        } else {
                            let mut sum = 0.0_f64;
                            for item in arr {
                                if let Value::Object(obj) = item {
                                    if let Some(val) = obj.get(&field_name) {
                                        sum += helpers::to_f64(val);
                                    }
                                }
                            }
                            sum
                        }
                    } else {
                        0.0
                    }
                } else {
                    // Parallelize for large arrays
                    if arr.len() >= PARALLEL_THRESHOLD {
                        arr.par_iter().map(|item| helpers::to_f64(item)).sum()
                    } else {
                        arr.iter().map(|item| helpers::to_f64(item)).sum()
                    }
                }
            }
            _ => helpers::to_f64(&array_val),
        };

        Ok(self.f64_to_json(sum))
    }

    /// Evaluate For loop operation - TRUE ZERO-COPY IMPLEMENTATION
    /// 
    /// This is the key optimization: instead of cloning the entire user_data context,
    /// we create a tiny internal_context with just $loopIteration and pass user_data by reference.
    pub(super) fn eval_for(&self, start_expr: &CompiledLogic, end_expr: &CompiledLogic, logic_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let next_depth = depth + 1;
        let start_val = self.evaluate_with_context(start_expr, user_data, internal_context, next_depth)?;
        let end_val = self.evaluate_with_context(end_expr, user_data, internal_context, next_depth)?;
        let start = helpers::to_number(&start_val) as i64;
        let end = helpers::to_number(&end_val) as i64;

        // CRITICAL: FOR returns an ARRAY of all iteration results (for use with MULTIPLIES, etc.)
        let mut results = Vec::new();

        // ZERO-COPY: Create tiny contexts for each iteration, no cloning of user_data!
        for i in start..end {
            // Create minimal internal context with just $loopIteration
            let loop_context = serde_json::json!({
                "$loopIteration": i
            });
            
            // Evaluate with loop context as internal_context, user_data remains untouched
            let result = self.evaluate_with_context(logic_expr, user_data, &loop_context, next_depth)?;
            results.push(result);
        }

        Ok(Value::Array(results))
    }

    /// Evaluate Multiplies operation (product of array values) - ZERO-COPY
    /// Optimized for MULTIPLIES+FOR pattern
    pub(super) fn eval_multiplies(&self, items: &[CompiledLogic], user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        // OPTIMIZATION: Detect MULTIPLIES containing single FOR loop
        // Pattern: MULTIPLIES([FOR(start, end, body)])
        // Instead of: FOR creates array -> flatten -> multiply
        // Optimize to: compute product directly in loop (with parallelization)
        if items.len() == 1 {
            if let CompiledLogic::For(start_expr, end_expr, logic_expr) = &items[0] {
                return self.eval_multiplies_for(start_expr, end_expr, logic_expr, user_data, internal_context, depth);
            }
        }

        // Standard path: flatten and multiply
        let values = self.flatten_array_values(items, user_data, internal_context, depth)?;
        if values.is_empty() { return Ok(Value::Null); }
        if values.len() == 1 { return Ok(self.f64_to_json(values[0])); }
        let result = values.iter().skip(1).fold(values[0], |acc, n| acc * n);
        Ok(self.f64_to_json(result))
    }

    /// Optimized MULTIPLIES+FOR: compute product directly without intermediate array
    /// Uses parallel computation for large ranges (>= PARALLEL_THRESHOLD)
    fn eval_multiplies_for(&self, start_expr: &CompiledLogic, end_expr: &CompiledLogic, logic_expr: &CompiledLogic, user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let next_depth = depth + 1;
        let start_val = self.evaluate_with_context(start_expr, user_data, internal_context, next_depth)?;
        let end_val = self.evaluate_with_context(end_expr, user_data, internal_context, next_depth)?;
        let start = helpers::to_number(&start_val) as i64;
        let end = helpers::to_number(&end_val) as i64;

        if start >= end {
            return Ok(Value::Null);
        }

        let range_size = (end - start) as usize;

        // Parallelize for large ranges
        if range_size >= PARALLEL_THRESHOLD {
            let result = (start..end)
                .into_par_iter()
                .try_fold(|| 1.0_f64, |product, i| -> Result<f64, String> {
                    let loop_context = serde_json::json!({
                        "$loopIteration": i
                    });
                    let val = self.evaluate_with_context(logic_expr, user_data, &loop_context, next_depth)?;
                    Ok(product * helpers::to_f64(&val))
                })
                .try_reduce(|| 1.0, |a, b| -> Result<f64, String> { Ok(a * b) })?;
            
            Ok(self.f64_to_json(result))
        } else {
            // Sequential for small ranges
            let mut product = 1.0_f64;
            for i in start..end {
                let loop_context = serde_json::json!({
                    "$loopIteration": i
                });
                let val = self.evaluate_with_context(logic_expr, user_data, &loop_context, next_depth)?;
                product *= helpers::to_f64(&val);
            }
            Ok(self.f64_to_json(product))
        }
    }

    /// Evaluate Divides operation (division of array values) - ZERO-COPY
    pub(super) fn eval_divides(&self, items: &[CompiledLogic], user_data: &Value, internal_context: &Value, depth: usize) -> Result<Value, String> {
        let values = self.flatten_array_values(items, user_data, internal_context, depth)?;
        if values.is_empty() { return Ok(Value::Null); }
        if values.len() == 1 { return Ok(self.f64_to_json(values[0])); }
        let result = values.iter().skip(1).fold(values[0], |acc, n| {
            if *n == 0.0 { acc } else { acc / n }
        });
        Ok(self.f64_to_json(result))
    }
}
