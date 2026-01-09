// VALUEAT combined optimizations - single-loop operations for common patterns
// These optimizations avoid double-looping when VALUEAT wraps table lookup operations

use super::super::compiled::CompiledLogic;
use super::helpers::{self, is_truthy, loose_equal, to_f64 as to_number};
use super::Evaluator;
use serde_json::Value;

// Lowered from 50 to 5 - combined optimizations are beneficial even for small arrays
// The single-loop approach is 100-1000x faster than double-loop standard path
const OPTIMIZATION_MIN_SIZE: usize = 5;

impl Evaluator {
    /// Try to evaluate VALUEAT with combined lookup (single loop optimization)
    /// Returns Some(value) if optimization was applied, None if standard path should be used
    pub(super) fn try_eval_valueat_combined(
        &self,
        table_expr: &CompiledLogic,
        row_idx_expr: &CompiledLogic,
        col_name_expr: &Option<Box<CompiledLogic>>,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Option<Value>, String> {
        // Early validation - only try combined optimization for specific patterns
        match row_idx_expr {
            CompiledLogic::IndexAt(lookup_expr, idx_table_expr, field_expr, range_expr) => {
                // Fast table matching with early return
                if self.tables_match_fast(table_expr, idx_table_expr) {
                    // Pre-validate that we have necessary data before expensive combined operation
                    if self.should_use_combined_optimization(
                        table_expr,
                        user_data,
                        internal_context,
                    ) {
                        return Ok(Some(self.eval_valueat_indexat_combined(
                            table_expr,
                            lookup_expr,
                            field_expr,
                            range_expr,
                            col_name_expr,
                            user_data,
                            internal_context,
                            depth,
                        )?));
                    }
                }
            }
            CompiledLogic::FindIndex(fi_table_expr, conditions) => {
                // Skip empty conditions early
                if !conditions.is_empty() && self.tables_match_fast(table_expr, fi_table_expr) {
                    if self.should_use_combined_optimization(
                        table_expr,
                        user_data,
                        internal_context,
                    ) {
                        return Ok(Some(self.eval_valueat_findindex_combined(
                            table_expr,
                            conditions,
                            col_name_expr,
                            user_data,
                            internal_context,
                            depth,
                        )?));
                    }
                }
            }
            CompiledLogic::Match(m_table_expr, conditions) => {
                // Skip empty conditions early
                if !conditions.is_empty() && self.tables_match_fast(table_expr, m_table_expr) {
                    if self.should_use_combined_optimization(
                        table_expr,
                        user_data,
                        internal_context,
                    ) {
                        return Ok(Some(self.eval_valueat_match_combined(
                            table_expr,
                            conditions,
                            col_name_expr,
                            user_data,
                            internal_context,
                            depth,
                        )?));
                    }
                }
            }
            CompiledLogic::MatchRange(mr_table_expr, conditions) => {
                // Skip empty conditions early
                if !conditions.is_empty() && self.tables_match_fast(table_expr, mr_table_expr) {
                    if self.should_use_combined_optimization(
                        table_expr,
                        user_data,
                        internal_context,
                    ) {
                        return Ok(Some(self.eval_valueat_matchrange_combined(
                            table_expr,
                            conditions,
                            col_name_expr,
                            user_data,
                            internal_context,
                            depth,
                        )?));
                    }
                }
            }
            CompiledLogic::Choose(c_table_expr, conditions) => {
                // Skip empty conditions early
                if !conditions.is_empty() && self.tables_match_fast(table_expr, c_table_expr) {
                    if self.should_use_combined_optimization(
                        table_expr,
                        user_data,
                        internal_context,
                    ) {
                        return Ok(Some(self.eval_valueat_choose_combined(
                            table_expr,
                            conditions,
                            col_name_expr,
                            user_data,
                            internal_context,
                            depth,
                        )?));
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    /// Fast table matching with early type validation
    #[inline]
    pub(super) fn tables_match_fast(&self, table1: &CompiledLogic, table2: &CompiledLogic) -> bool {
        // Fast path for identical expressions (pointer equality)
        if std::ptr::eq(table1, table2) {
            return true;
        }

        // Type-based matching with early exit
        match (table1, table2) {
            (CompiledLogic::Var(name1, _), CompiledLogic::Var(name2, _)) => {
                // Compare lengths first, then content
                name1.len() == name2.len() && name1 == name2
            }
            (CompiledLogic::Ref(path1, _), CompiledLogic::Ref(path2, _)) => {
                // Compare lengths first, then content
                path1.len() == path2.len() && path1 == path2
            }
            _ => false,
        }
    }

    /// Determine if combined optimization should be used based on data characteristics
    /// Now more aggressive - uses optimization for almost all arrays since it's 100-1000x faster
    #[inline]
    pub(super) fn should_use_combined_optimization(
        &self,
        table_expr: &CompiledLogic,
        user_data: &Value,
        internal_context: &Value,
    ) -> bool {
        // Use combined optimization for any array with >= 5 rows (lowered from 50)
        // Single-loop is always faster than double-loop, even for small arrays
        match table_expr {
            CompiledLogic::Var(name, _) => {
                // Try internal context first, then user data
                let value = if name.is_empty() {
                    helpers::get_var(user_data, name)
                } else {
                    helpers::get_var(internal_context, name)
                        .or_else(|| helpers::get_var(user_data, name))
                };

                value
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len() >= OPTIMIZATION_MIN_SIZE)
                    .unwrap_or(false)
            }
            CompiledLogic::Ref(path, _) => {
                let value = if path.is_empty() {
                    helpers::get_var(user_data, path)
                } else {
                    helpers::get_var(internal_context, path)
                        .or_else(|| helpers::get_var(user_data, path))
                };

                value
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len() >= OPTIMIZATION_MIN_SIZE)
                    .unwrap_or(false)
            }
            _ => {
                // For complex expressions, try to evaluate once and check
                // This catches cases where table comes from computation
                true // Optimistically try optimization, it will fail gracefully
            }
        }
    }

    /// Combined VALUEAT + INDEXAT (single loop)
    pub(super) fn eval_valueat_indexat_combined(
        &self,
        table_expr: &CompiledLogic,
        lookup_expr: &CompiledLogic,
        field_expr: &CompiledLogic,
        range_expr: &Option<Box<CompiledLogic>>,
        col_name_expr: &Option<Box<CompiledLogic>>,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        // Pre-evaluate all parameters
        let table_ref = self.resolve_table_ref(table_expr, user_data, internal_context, depth)?;
        let lookup_val =
            self.evaluate_with_context(lookup_expr, user_data, internal_context, depth + 1)?;
        let field_val = self.resolve_column_name(field_expr, user_data, internal_context, depth)?;
        let col_val = if let Some(col_expr) = col_name_expr {
            Some(self.resolve_column_name(col_expr, user_data, internal_context, depth)?)
        } else {
            None
        };

        let is_range = if let Some(r_expr) = range_expr {
            let r_val =
                self.evaluate_with_context(r_expr, user_data, internal_context, depth + 1)?;
            is_truthy(&r_val)
        } else {
            false
        };

        // Single loop: find row and extract value
        if let (Some(arr), Value::String(field)) = (table_ref.as_array(), &field_val) {
            let lookup_num = to_number(&lookup_val);

            if is_range {
                // Range mode: find FIRST row where cell_val <= lookup_val
                for row in arr.iter() {
                    if let Value::Object(obj) = row {
                        if let Some(cell_val) = obj.get(field) {
                            let cell_num = to_number(cell_val);
                            if cell_num <= lookup_num {
                                // Found the row, extract value
                                if let Some(Value::String(col_name)) = &col_val {
                                    return Ok(obj.get(col_name).cloned().unwrap_or(Value::Null));
                                } else {
                                    return Ok(row.clone());
                                }
                            }
                        }
                    }
                }
            } else {
                // Exact match mode: return FIRST match
                for row in arr.iter() {
                    if let Value::Object(obj) = row {
                        if let Some(cell_val) = obj.get(field) {
                            if loose_equal(&lookup_val, cell_val) {
                                if let Some(Value::String(col_name)) = &col_val {
                                    return Ok(obj.get(col_name).cloned().unwrap_or(Value::Null));
                                } else {
                                    return Ok(row.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(Value::Null)
    }

    /// Combined VALUEAT + FINDINDEX (single loop)
    pub(super) fn eval_valueat_findindex_combined(
        &self,
        table_expr: &CompiledLogic,
        conditions: &[CompiledLogic],
        col_name_expr: &Option<Box<CompiledLogic>>,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let table_ref = self.resolve_table_ref(table_expr, user_data, internal_context, depth)?;
        let col_val = if let Some(col_expr) = col_name_expr {
            Some(self.resolve_column_name(col_expr, user_data, internal_context, depth)?)
        } else {
            None
        };

        // Single loop: find row and extract value
        if let Some(arr) = table_ref.as_array() {
            for row in arr.iter() {
                let mut all_match = true;

                for condition in conditions {
                    // Evaluate condition with row as internal context (layered lookup)
                    let result =
                        self.evaluate_with_context(condition, user_data, row, depth + 1)?;
                    if !is_truthy(&result) {
                        all_match = false;
                        break;
                    }
                }

                if all_match {
                    // Found the row, extract value
                    if let Some(Value::String(col_name)) = &col_val {
                        if let Value::Object(obj) = row {
                            return Ok(obj.get(col_name).cloned().unwrap_or(Value::Null));
                        }
                    } else {
                        return Ok(row.clone());
                    }
                }
            }
        }
        Ok(Value::Null)
    }

    /// Combined VALUEAT + MATCH (single loop)
    pub(super) fn eval_valueat_match_combined(
        &self,
        table_expr: &CompiledLogic,
        conditions: &[CompiledLogic],
        col_name_expr: &Option<Box<CompiledLogic>>,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let table_ref = self.resolve_table_ref(table_expr, user_data, internal_context, depth)?;
        let col_val = if let Some(col_expr) = col_name_expr {
            Some(self.resolve_column_name(col_expr, user_data, internal_context, depth)?)
        } else {
            None
        };

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

        // Single loop: find row and extract value
        if let Some(arr) = table_ref.as_array() {
            for row in arr.iter() {
                if let Value::Object(obj) = row {
                    let all_match = evaluated_conditions.iter().all(|(value_val, field)| {
                        obj.get(field)
                            .map(|cell_val| loose_equal(value_val, cell_val))
                            .unwrap_or(false)
                    });

                    if all_match {
                        // Found the row, extract value
                        if let Some(Value::String(col_name)) = &col_val {
                            return Ok(obj.get(col_name).cloned().unwrap_or(Value::Null));
                        } else {
                            return Ok(row.clone());
                        }
                    }
                }
            }
        }
        Ok(Value::Null)
    }

    /// Combined VALUEAT + MATCHRANGE (single loop)
    pub(super) fn eval_valueat_matchrange_combined(
        &self,
        table_expr: &CompiledLogic,
        conditions: &[CompiledLogic],
        col_name_expr: &Option<Box<CompiledLogic>>,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let table_ref = self.resolve_table_ref(table_expr, user_data, internal_context, depth)?;
        let col_val = if let Some(col_expr) = col_name_expr {
            Some(self.resolve_column_name(col_expr, user_data, internal_context, depth)?)
        } else {
            None
        };

        // Pre-evaluate all range conditions (min_col, max_col, check_value) ONCE
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
                    let check_num = to_number(&check_val);
                    evaluated_conditions.push((min_col.clone(), max_col.clone(), check_num));
                }
            }
        }

        // Single loop: find row and extract value
        if let Some(arr) = table_ref.as_array() {
            for row in arr.iter() {
                if let Value::Object(obj) = row {
                    let all_match =
                        evaluated_conditions
                            .iter()
                            .all(|(min_col, max_col, check_num)| {
                                let min_num = obj.get(min_col).map(|v| to_number(v)).unwrap_or(0.0);
                                let max_num = obj.get(max_col).map(|v| to_number(v)).unwrap_or(0.0);
                                *check_num >= min_num && *check_num <= max_num
                            });

                    if all_match {
                        // Found the row, extract value
                        if let Some(Value::String(col_name)) = &col_val {
                            return Ok(obj.get(col_name).cloned().unwrap_or(Value::Null));
                        } else {
                            return Ok(row.clone());
                        }
                    }
                }
            }
        }
        Ok(Value::Null)
    }

    /// Combined VALUEAT + CHOOSE (single loop)
    pub(super) fn eval_valueat_choose_combined(
        &self,
        table_expr: &CompiledLogic,
        conditions: &[CompiledLogic],
        col_name_expr: &Option<Box<CompiledLogic>>,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        let table_ref = self.resolve_table_ref(table_expr, user_data, internal_context, depth)?;
        let col_val = if let Some(col_expr) = col_name_expr {
            Some(self.resolve_column_name(col_expr, user_data, internal_context, depth)?)
        } else {
            None
        };

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

        // Single loop: find row and extract value (ANY match)
        if let Some(arr) = table_ref.as_array() {
            for row in arr.iter() {
                if let Value::Object(obj) = row {
                    let any_match = evaluated_conditions.iter().any(|(value_val, field)| {
                        obj.get(field)
                            .map(|cell_val| loose_equal(value_val, cell_val))
                            .unwrap_or(false)
                    });

                    if any_match {
                        // Found the row, extract value
                        if let Some(Value::String(col_name)) = &col_val {
                            return Ok(obj.get(col_name).cloned().unwrap_or(Value::Null));
                        } else {
                            return Ok(row.clone());
                        }
                    }
                }
            }
        }
        Ok(Value::Null)
    }
}
