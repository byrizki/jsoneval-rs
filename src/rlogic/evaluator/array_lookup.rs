use super::super::compiled::CompiledLogic;
use super::helpers;
use super::{types::*, Evaluator};
use serde_json::Value;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

// Lower threshold for parallel processing - even smaller tables benefit from optimization
#[cfg(feature = "parallel")]
const PARALLEL_THRESHOLD: usize = 1000;

impl Evaluator {
    /// Resolve table reference directly - ZERO-COPY with optimized lookup
    #[inline]
    pub(super) fn resolve_table_ref<'a>(
        &self,
        table_expr: &CompiledLogic,
        user_data: &'a Value,
        internal_context: &'a Value,
        depth: usize,
    ) -> Result<TableRef<'a>, String> {
        match table_expr {
            CompiledLogic::Var(name, _) => {
                // OPTIMIZATION: Fast path for common table names (no empty check overhead)
                let value = if name.is_empty() {
                    helpers::get_var(user_data, name)
                } else {
                    // Try user_data first for tables (internal_context rarely has tables)
                    helpers::get_var(user_data, name)
                        .or_else(|| helpers::get_var(internal_context, name))
                };
                value
                    .filter(|v| !v.is_null())
                    .map(TableRef::Borrowed)
                    .ok_or_else(|| format!("Variable not found: {}", name))
            }
            CompiledLogic::Ref(path, _) => {
                // Tables are usually in user_data, not internal_context
                let value = helpers::get_var(user_data, path)
                    .or_else(|| helpers::get_var(internal_context, path));
                value
                    .filter(|v| !v.is_null())
                    .map(TableRef::Borrowed)
                    .ok_or_else(|| format!("Reference not found: {}", path))
            }
            _ => self
                .evaluate_with_context(table_expr, user_data, internal_context, depth + 1)
                .map(TableRef::Owned),
        }
    }

    /// Fast path to get borrowed array slice from table expression - ZERO-COPY
    #[inline]
    pub(super) fn get_table_array<'a>(
        &self,
        table_expr: &CompiledLogic,
        user_data: &'a Value,
        internal_context: &'a Value,
        depth: usize,
    ) -> Result<TableRef<'a>, String> {
        let table_ref = self.resolve_table_ref(table_expr, user_data, internal_context, depth)?;
        match table_ref.as_value() {
            Value::Array(_) => Ok(table_ref),
            Value::Null => Err("Table reference is null".to_string()),
            _ => Err("Table reference is not an array".to_string()),
        }
    }

    /// Resolve column name with fast path for literals and variables - ZERO-COPY
    #[inline]
    pub(super) fn resolve_column_name(
        &self,
        col_expr: &CompiledLogic,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        match col_expr {
            // OPTIMIZATION: Direct return for string literals (most common case)
            CompiledLogic::String(s) => Ok(Value::String(s.clone())),
            CompiledLogic::Var(name, _) => {
                // OPTIMIZATION: Check user_data first (column names usually come from there)
                let value = if name.is_empty() {
                    helpers::get_var(user_data, name)
                } else {
                    helpers::get_var(user_data, name)
                        .or_else(|| helpers::get_var(internal_context, name))
                };
                Ok(value.cloned().unwrap_or(Value::Null))
            }
            _ => self.evaluate_with_context(col_expr, user_data, internal_context, depth),
        }
    }

    /// Evaluate ValueAt operation - ZERO-COPY with aggressive optimizations
    pub(super) fn eval_valueat(
        &self,
        table_expr: &CompiledLogic,
        row_idx_expr: &CompiledLogic,
        col_name_expr: &Option<Box<CompiledLogic>>,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        // OPTIMIZATION 1: Try combined single-loop operations first
        if let Some(result) = self.try_eval_valueat_combined(
            table_expr,
            row_idx_expr,
            col_name_expr,
            user_data,
            internal_context,
            depth,
        )? {
            return Ok(result);
        }

        // OPTIMIZATION 2: Fast path for literal row indices (avoid evaluation overhead)
        let row_idx = match row_idx_expr {
            CompiledLogic::Number(n) => {
                // Direct parse for literal numbers
                match n.parse::<i64>() {
                    Ok(idx) if idx >= 0 => Some(idx as usize),
                    _ => None,
                }
            }
            _ => {
                // Evaluate row index expression
                let row_idx_val = self.evaluate_with_context(
                    row_idx_expr,
                    user_data,
                    internal_context,
                    depth + 1,
                )?;
                let row_idx_num = helpers::to_number(&row_idx_val) as i64;
                if row_idx_num >= 0 {
                    Some(row_idx_num as usize)
                } else {
                    None
                }
            }
        };

        // Early exit if invalid index
        let row_idx = match row_idx {
            Some(idx) => idx,
            None => return Ok(Value::Null),
        };

        // OPTIMIZATION 3: Resolve table and check bounds together (reduce overhead)
        let table_ref = self.get_table_array(table_expr, user_data, internal_context, depth)?;
        let arr = match table_ref.as_array() {
            Some(arr) if !arr.is_empty() && row_idx < arr.len() => arr,
            _ => return Ok(Value::Null),
        };

        let row = &arr[row_idx];

        // OPTIMIZATION 4: Fast path for column resolution
        let result = if let Some(col_expr) = col_name_expr {
            // Try fast path for string literals first
            let col_name = match col_expr.as_ref() {
                CompiledLogic::String(s) => s.as_str(),
                _ => {
                    // Fall back to full resolution
                    match self.resolve_column_name(col_expr, user_data, internal_context, depth)? {
                        Value::String(s) => {
                            // Need to return cloned value since we can't borrow from temporary
                            if let Value::Object(obj) = row {
                                return Ok(obj.get(&s).cloned().unwrap_or(Value::Null));
                            } else {
                                return Ok(Value::Null);
                            }
                        }
                        _ => return Ok(Value::Null),
                    }
                }
            };

            // Direct object field access
            if let Value::Object(obj) = row {
                obj.get(col_name).cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        } else {
            row.clone()
        };

        Ok(result)
    }

    /// Evaluate MaxAt operation - ZERO-COPY
    pub(super) fn eval_maxat(
        &self,
        table_expr: &CompiledLogic,
        col_name_expr: &CompiledLogic,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let table_ref = self.get_table_array(table_expr, user_data, internal_context, depth)?;
        let col_val =
            self.resolve_column_name(col_name_expr, user_data, internal_context, depth)?;

        if let Value::String(col_name) = &col_val {
            let result = if let Some(arr) = table_ref.as_array() {
                arr.last()
                    .and_then(|last_row| last_row.as_object())
                    .and_then(|obj| obj.get(col_name))
                    .cloned()
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            };
            Ok(result)
        } else {
            Ok(Value::Null)
        }
    }

    /// Evaluate IndexAt operation - ZERO-COPY
    pub(super) fn eval_indexat(
        &self,
        lookup_expr: &CompiledLogic,
        table_expr: &CompiledLogic,
        field_expr: &CompiledLogic,
        range_expr: &Option<Box<CompiledLogic>>,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let lookup_val =
            self.evaluate_with_context(lookup_expr, user_data, internal_context, depth + 1)?;
        let field_val = self.resolve_column_name(field_expr, user_data, internal_context, depth)?;
        let field_name = match field_val {
            Value::String(s) => s,
            _ => return Ok(self.f64_to_json(-1.0)),
        };

        let is_range = if let Some(r_expr) = range_expr {
            let r_val =
                self.evaluate_with_context(r_expr, user_data, internal_context, depth + 1)?;
            helpers::is_truthy(&r_val)
        } else {
            false
        };

        let table_ref = self.get_table_array(table_expr, user_data, internal_context, depth)?;
        let arr = match table_ref.as_array() {
            Some(arr) if !arr.is_empty() => arr,
            _ => return Ok(self.f64_to_json(-1.0)),
        };

        let lookup_num = if is_range {
            helpers::to_number(&lookup_val)
        } else {
            0.0
        };

        #[cfg(feature = "parallel")]
        if !is_range && arr.len() >= PARALLEL_THRESHOLD {
            let result = arr.par_iter().enumerate().find_map_first(|(idx, row)| {
                if let Value::Object(obj) = row {
                    if let Some(cell_val) = obj.get(&field_name) {
                        if helpers::loose_equal(&lookup_val, cell_val) {
                            return Some(idx as f64);
                        }
                    }
                }
                None
            });
            return Ok(self.f64_to_json(result.unwrap_or(-1.0)));
        }

        if is_range {
            for (idx, row) in arr.iter().enumerate() {
                if let Value::Object(obj) = row {
                    if let Some(cell_val) = obj.get(&field_name) {
                        let cell_num = helpers::to_number(cell_val);
                        if cell_num <= lookup_num {
                            return Ok(self.f64_to_json(idx as f64));
                        }
                    }
                }
            }
            Ok(self.f64_to_json(-1.0))
        } else {
            for (idx, row) in arr.iter().enumerate() {
                if let Value::Object(obj) = row {
                    if let Some(cell_val) = obj.get(&field_name) {
                        if helpers::loose_equal(&lookup_val, cell_val) {
                            return Ok(self.f64_to_json(idx as f64));
                        }
                    }
                }
            }
            Ok(self.f64_to_json(-1.0))
        }
    }

    /// Evaluate Match operation - ZERO-COPY
    pub(super) fn eval_match(
        &self,
        table_expr: &CompiledLogic,
        conditions: &[CompiledLogic],
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let table_ref = self.get_table_array(table_expr, user_data, internal_context, depth)?;

        // Pre-evaluate all condition pairs (value, field) ONCE
        let mut evaluated_conditions = Vec::with_capacity(conditions.len() / 2);
        for chunk in conditions.chunks(2) {
            if chunk.len() == 2 {
                let value_val =
                    self.evaluate_with_context(&chunk[0], user_data, internal_context, depth + 1)?;
                let field_val =
                    self.evaluate_with_context(&chunk[1], user_data, internal_context, depth + 1)?;
                if let Value::String(field) = field_val {
                    evaluated_conditions.push((value_val, field));
                }
            }
        }

        if let Some(arr) = table_ref.as_array() {
            #[cfg(feature = "parallel")]
            if arr.len() >= PARALLEL_THRESHOLD {
                let result = arr.par_iter().enumerate().find_map_first(|(idx, row)| {
                    if let Value::Object(obj) = row {
                        let all_match = evaluated_conditions.iter().all(|(value_val, field)| {
                            obj.get(field)
                                .map(|cell_val| helpers::loose_equal(value_val, cell_val))
                                .unwrap_or(false)
                        });
                        if all_match {
                            Some(idx as f64)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
                return Ok(self.f64_to_json(result.unwrap_or(-1.0)));
            }

            for (idx, row) in arr.iter().enumerate() {
                if let Value::Object(obj) = row {
                    let all_match = evaluated_conditions.iter().all(|(value_val, field)| {
                        obj.get(field)
                            .map(|cell_val| helpers::loose_equal(value_val, cell_val))
                            .unwrap_or(false)
                    });
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

    /// Evaluate MatchRange operation - ZERO-COPY
    pub(super) fn eval_matchrange(
        &self,
        table_expr: &CompiledLogic,
        conditions: &[CompiledLogic],
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let table_ref = self.get_table_array(table_expr, user_data, internal_context, depth)?;

        let mut evaluated_conditions = Vec::with_capacity(conditions.len() / 3);
        for chunk in conditions.chunks(3) {
            if chunk.len() == 3 {
                let min_col_val =
                    self.evaluate_with_context(&chunk[0], user_data, internal_context, depth + 1)?;
                let max_col_val =
                    self.evaluate_with_context(&chunk[1], user_data, internal_context, depth + 1)?;
                let check_val =
                    self.evaluate_with_context(&chunk[2], user_data, internal_context, depth + 1)?;

                if let (Value::String(min_col), Value::String(max_col)) =
                    (&min_col_val, &max_col_val)
                {
                    let check_num = helpers::to_number(&check_val);
                    evaluated_conditions.push((min_col.clone(), max_col.clone(), check_num));
                }
            }
        }

        if let Some(arr) = table_ref.as_array() {
            #[cfg(feature = "parallel")]
            if arr.len() >= PARALLEL_THRESHOLD {
                let result = arr.par_iter().enumerate().find_map_first(|(idx, row)| {
                    if let Value::Object(obj) = row {
                        let all_match =
                            evaluated_conditions
                                .iter()
                                .all(|(min_col, max_col, check_num)| {
                                    let min_num = obj
                                        .get(min_col)
                                        .map(|v| helpers::to_number(v))
                                        .unwrap_or(0.0);
                                    let max_num = obj
                                        .get(max_col)
                                        .map(|v| helpers::to_number(v))
                                        .unwrap_or(0.0);
                                    *check_num >= min_num && *check_num <= max_num
                                });
                        if all_match {
                            Some(idx as f64)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
                return Ok(self.f64_to_json(result.unwrap_or(-1.0)));
            }

            for (idx, row) in arr.iter().enumerate() {
                if let Value::Object(obj) = row {
                    let all_match =
                        evaluated_conditions
                            .iter()
                            .all(|(min_col, max_col, check_num)| {
                                let min_num = obj
                                    .get(min_col)
                                    .map(|v| helpers::to_number(v))
                                    .unwrap_or(0.0);
                                let max_num = obj
                                    .get(max_col)
                                    .map(|v| helpers::to_number(v))
                                    .unwrap_or(0.0);
                                *check_num >= min_num && *check_num <= max_num
                            });
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

    /// Evaluate Choose operation - ZERO-COPY
    pub(super) fn eval_choose(
        &self,
        table_expr: &CompiledLogic,
        conditions: &[CompiledLogic],
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let table_ref = self.get_table_array(table_expr, user_data, internal_context, depth)?;

        let mut evaluated_conditions = Vec::with_capacity(conditions.len() / 2);
        for chunk in conditions.chunks(2) {
            if chunk.len() == 2 {
                let value_val =
                    self.evaluate_with_context(&chunk[0], user_data, internal_context, depth + 1)?;
                let field_val =
                    self.evaluate_with_context(&chunk[1], user_data, internal_context, depth + 1)?;
                if let Value::String(field) = field_val {
                    evaluated_conditions.push((value_val, field));
                }
            }
        }

        if let Some(arr) = table_ref.as_array() {
            #[cfg(feature = "parallel")]
            if arr.len() >= PARALLEL_THRESHOLD {
                let result = arr.par_iter().enumerate().find_map_first(|(idx, row)| {
                    if let Value::Object(obj) = row {
                        let any_match = evaluated_conditions.iter().any(|(value_val, field)| {
                            obj.get(field)
                                .map(|cell_val| helpers::loose_equal(value_val, cell_val))
                                .unwrap_or(false)
                        });
                        if any_match {
                            Some(idx as f64)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
                return Ok(self.f64_to_json(result.unwrap_or(-1.0)));
            }

            for (idx, row) in arr.iter().enumerate() {
                if let Value::Object(obj) = row {
                    let any_match = evaluated_conditions.iter().any(|(value_val, field)| {
                        obj.get(field)
                            .map(|cell_val| helpers::loose_equal(value_val, cell_val))
                            .unwrap_or(false)
                    });
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

    /// Evaluate FindIndex operation - ZERO-COPY
    pub(super) fn eval_findindex(
        &self,
        table_expr: &CompiledLogic,
        conditions: &[CompiledLogic],
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let table_ref = self.get_table_array(table_expr, user_data, internal_context, depth)?;
        let arr = match table_ref.as_array() {
            Some(arr) if !arr.is_empty() => arr,
            _ => return Ok(self.f64_to_json(-1.0)),
        };

        if conditions.is_empty() {
            return Ok(self.f64_to_json(0.0));
        }

        #[cfg(feature = "parallel")]
        if arr.len() >= PARALLEL_THRESHOLD {
            let result = arr.par_iter().enumerate().find_map_first(|(idx, row)| {
                for condition in conditions {
                    // Use row as primary context, user_data as fallback
                    match self.evaluate_with_context(condition, row, user_data, depth + 1) {
                        Ok(result) if helpers::is_truthy(&result) => continue,
                        _ => return None,
                    }
                }
                Some(idx as f64)
            });
            return Ok(self.f64_to_json(result.unwrap_or(-1.0)));
        }

        for (idx, row) in arr.iter().enumerate() {
            let mut all_match = true;
            for condition in conditions {
                // Use row as primary context, user_data as fallback
                match self.evaluate_with_context(condition, row, user_data, depth + 1) {
                    Ok(result) if helpers::is_truthy(&result) => continue,
                    _ => {
                        all_match = false;
                        break;
                    }
                }
            }
            if all_match {
                return Ok(self.f64_to_json(idx as f64));
            }
        }
        Ok(self.f64_to_json(-1.0))
    }
}
