use super::super::compiled::CompiledLogic;
use super::helpers;
use super::{types::*, Evaluator};
use serde_json::Value;


impl Evaluator {
    /// Resolve table reference directly - ZERO-COPY with optimized lookup
    #[inline]
    pub(super) fn resolve_table_ref<'a>(
        &'a self,
        table_expr: &CompiledLogic,
        user_data: &'a Value,
        internal_context: &'a Value,
        depth: usize,
    ) -> Result<TableRef<'a>, String> {
        // Fast path: check active table scope for self-table references
        let var_name = match table_expr {
            CompiledLogic::Var(name, _) | CompiledLogic::Ref(name, _) => Some(name.as_str()),
            _ => None,
        };

        if let Some(name) = var_name {
            // SAFETY: single-threaded (eval_lock), UnsafeCell access
            let scope = unsafe { &*self.table_scope.get() };
            if let Some(ts) = scope.as_ref() {
                if name == ts.path {
                    // SAFETY: local_rows outlives this call (table_evaluate_inner scope)
                    let rows = unsafe { &*ts.rows };
                    return Ok(TableRef::LocalRows(rows));
                }
            }
        }

        match table_expr {
            CompiledLogic::Var(name, _) => {
                // OPTIMIZATION: Fast path for common table names (no empty check overhead)
                let value = if name.is_empty() {
                    self.get_var(user_data, name)
                } else {
                    // Try user_data first for tables (internal_context rarely has tables)
                    self.get_var(user_data, name)
                        .or_else(|| self.get_var(internal_context, name))
                };
                value
                    .filter(|v: &&Value| !v.is_null())
                    .map(|v| TableRef::Borrowed(v))
                    .ok_or_else(|| format!("Variable not found: {}", name))
            }
            CompiledLogic::Ref(path, _) => {
                // Tables are usually in user_data, not internal_context
                let value = self.get_var(user_data, path)
                    .or_else(|| self.get_var(internal_context, path));
                value
                    .filter(|v: &&Value| !v.is_null())
                    .map(|v| TableRef::Borrowed(v))
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
        &'a self,
        table_expr: &CompiledLogic,
        user_data: &'a Value,
        internal_context: &'a Value,
        depth: usize,
    ) -> Result<TableRef<'a>, String> {
        let table_ref = self.resolve_table_ref(table_expr, user_data, internal_context, depth)?;
        // LocalRows is always an array (from local_rows Vec) — handle before as_value() check
        if matches!(table_ref, TableRef::LocalRows(_)) {
            return Ok(table_ref);
        }
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
                    self.get_var(user_data, name)
                } else {
                    self.get_var(user_data, name)
                        .or_else(|| self.get_var(internal_context, name))
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
                let idx = *n as i64;
                if idx >= 0 { Some(idx as usize) } else { None }
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

        // OPTIMIZATION: Check for existing index (only for exact matches, not range)
        if !is_range {
            let table_name = match table_expr {
                CompiledLogic::Var(name, _) => Some(name.as_str()),
                CompiledLogic::Ref(path, _) => Some(path.as_str()),
                _ => None,
            };

            if let Some(name) = table_name {
                if let Ok(indices) = self.indices.read() {
                    if let Some(index) = indices.get(name) {
                        if index.has_column(&field_name) {
                            if let Some(rows) = index.lookup(&field_name, &lookup_val) {
                                if let Some(min_idx) = rows.iter().min() {
                                    return Ok(self.f64_to_json(*min_idx as f64));
                                }
                            } else {
                                return Ok(self.f64_to_json(-1.0));
                            }
                        }
                    }
                }
            }
        }

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

        // OPTIMIZATION: Check for existing index
        let table_name = match table_expr {
            CompiledLogic::Var(name, _) => Some(name.as_str()),
            CompiledLogic::Ref(path, _) => Some(path.as_str()),
            _ => None,
        };

        if let Some(name) = table_name {
            if let Ok(indices) = self.indices.read() {
                if let Some(index) = indices.get(name) {
                    // We have an index for this table!
                    
                    // 1. Check if all columns in conditions are indexed
                    let all_columns_indexed = evaluated_conditions.iter().all(|(_, field)| index.has_column(field));
                    
                    if all_columns_indexed {
                        // 2. Perform intersection of matching row indices
                        let mut candidate_rows: Option<std::collections::HashSet<usize>> = None;
                        
                        for (val, field) in &evaluated_conditions {
                            if let Some(rows) = index.lookup(field, val) {
                                if let Some(candidates) = &mut candidate_rows {
                                     // Intersect with existing candidates
                                     candidates.retain(|r| rows.contains(r));
                                     if candidates.is_empty() {
                                         return Ok(self.f64_to_json(-1.0));
                                     }
                                } else {
                                    // Initialize candidates
                                    // We clone here because we need to mutate the set for intersection
                                    // but AHashSet clone is relatively cheap compared to iteration
                                    candidate_rows = Some(rows.iter().cloned().collect());
                                }
                            } else {
                                // Value not found in this column -> no match possible
                                return Ok(self.f64_to_json(-1.0));
                            }
                        }
                        
                        // 3. Return the first matching row index (min)
                        if let Some(candidates) = candidate_rows {
                            if let Some(min_idx) = candidates.iter().min() {
                                return Ok(self.f64_to_json(*min_idx as f64));
                            }
                        } else {
                            // No conditions provided? Technically logic allows it, but usually conditions exist.
                            // If no conditions, match returns first row?
                            // Logic usually implies "match based on conditions".
                            // If conditions empty, existing logic returns -1 (or loops 0 times).
                            // Let's stick to existing behavior if processed conditions is empty
                            if conditions.is_empty() {
                                 // Fall through to standard logic (which returns -1)
                            } else if index.len() > 0 {
                                // If we had conditions but they were all filtered out (e.g. invalid format)
                                // fall through.
                                // But here evaluated_conditions is populated.
                                // If evaluated_conditions is empty but conditions wasn't, it means format error.
                                // If evaluated_conditions is not empty, candidate_rows would be Some.
                                // So this block is reached only if evaluated_conditions is empty.
                            }
                        }
                    }
                }
            }
        }

        let table_ref = self.get_table_array(table_expr, user_data, internal_context, depth)?;

        if let Some(arr) = table_ref.as_array() {

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

    /// Evaluate FindIndex operation (zero-alloc hot path)
    ///
    /// Conditions are evaluated with a three-layer variable resolution:
    ///   `internal_context` → `row` → `user_data`
    ///
    /// This lets conditions reference both:
    /// - Table column names via `Var` (e.g. `"PAYOR_AGE"` from the current row)
    /// - Outer context variables via `$ref` (e.g. `$PAYORAGE_YEAR` from the parent iteration)
    /// without any heap allocation or map cloning per row.
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

        // Fast-path optimization: check if we can use the `MATCH` pattern
        let mut can_use_index = true;
        // Flatten any top-level `And` conditions directly into our evaluation check
        let mut actual_conditions = Vec::with_capacity(conditions.len());
        for cond in conditions {
            if let CompiledLogic::And(items) = cond {
                actual_conditions.extend(items.iter());
            } else {
                actual_conditions.push(cond);
            }
        }

        let mut evaluated_match_conditions = Vec::with_capacity(actual_conditions.len());

        let is_provably_row_independent = |expr: &CompiledLogic| -> bool {
            match expr {
                CompiledLogic::Null | CompiledLogic::Bool(_) | CompiledLogic::Number(_) | CompiledLogic::String(_) | CompiledLogic::Ref(..) => true,
                _ => expr.referenced_vars().is_empty()
            }
        };

        for cond in actual_conditions {
            match cond {
                CompiledLogic::Equal(a, b) | CompiledLogic::StrictEqual(a, b) => {
                    let (var_name, val_expr) = if let CompiledLogic::Var(f, _) = &**a {
                        (f.clone(), &**b)
                    } else if let CompiledLogic::Var(f, _) = &**b {
                        (f.clone(), &**a)
                    } else {
                        can_use_index = false;
                        break;
                    };

                    if !is_provably_row_independent(val_expr) {
                        can_use_index = false;
                        break;
                    }

                    // Pre-evaluate the val_expr ONCE outside the row loop
                    let evaluated_val = self.evaluate_with_context(val_expr, user_data, internal_context, depth + 1)?;
                    
                    // Var names are normalized to JSON Pointers (e.g. "/col"). Strip the leading '/' 
                    // to match the behavior of MATCH which uses bare strings ("col") for index caching and obj.get()
                    let bare_field = var_name.strip_prefix('/').unwrap_or(&var_name).to_string();
                    
                    evaluated_match_conditions.push((evaluated_val, bare_field));
                }
                _ => {
                    can_use_index = false;
                    break;
                }
            }
        }

        if can_use_index && !evaluated_match_conditions.is_empty() {
            // 1. Check if index cache is accessible
            let table_name = match table_expr {
                CompiledLogic::Var(name, _) => Some(name.as_str()),
                CompiledLogic::Ref(path, _) => Some(path.as_str()),
                _ => None,
            };

            if let Some(name) = table_name {
                if let Ok(indices) = self.indices.read() {
                    if let Some(index) = indices.get(name) {
                        let all_columns_indexed = evaluated_match_conditions.iter().all(|(_, field)| index.has_column(field));
                        if all_columns_indexed {
                            let mut candidate_rows: Option<std::collections::HashSet<usize>> = None;
                            for (val, field) in &evaluated_match_conditions {
                                if let Some(rows) = index.lookup(field, val) {
                                    if let Some(candidates) = &mut candidate_rows {
                                        candidates.retain(|r| rows.contains(r));
                                        if candidates.is_empty() {
                                            return Ok(self.f64_to_json(-1.0));
                                        }
                                    } else {
                                        candidate_rows = Some(rows.iter().cloned().collect());
                                    }
                                } else {
                                    return Ok(self.f64_to_json(-1.0));
                                }
                            }
                            if let Some(candidates) = candidate_rows {
                                if let Some(min_idx) = candidates.iter().min() {
                                    return Ok(self.f64_to_json(*min_idx as f64));
                                }
                            }
                        }
                    }
                }
            }

            // 2. Fallback to `O(N)` loop using direct column GET instead of eval_condition_with_row
            for (idx, row) in arr.iter().enumerate() {
                if let Value::Object(obj) = row {
                    let all_match = evaluated_match_conditions.iter().all(|(value_val, field)| {
                        obj.get(field)
                            .map(|cell_val| helpers::loose_equal(value_val, cell_val))
                            .unwrap_or(false)
                    });
                    if all_match {
                        return Ok(self.f64_to_json(idx as f64));
                    }
                }
            }
            return Ok(self.f64_to_json(-1.0));
        }

        // SLOW PATH: Dynamic fallback for complex or row-dependent conditions
        for (idx, row) in arr.iter().enumerate() {
            let mut all_match = true;
            for condition in conditions {
                match self.eval_condition_with_row(
                    condition,
                    user_data,
                    internal_context,
                    row,
                    depth + 1,
                ) {
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

    /// Evaluate a condition with a three-layer variable resolution (zero-alloc).
    ///
    /// Lookup order for `Var`: `internal_context` → `row` → `user_data`
    /// Lookup order for `Ref`: `internal_context` → `user_data`
    ///
    /// After `preprocess_table_condition`, bare column name strings become `Var` nodes
    /// so they resolve from the current `row`. `$ref` nodes that point to outer context
    /// variables (e.g. `$PAYORAGE_YEAR`) resolve from `internal_context` first.
    #[inline]
    pub(super) fn eval_condition_with_row(
        &self,
        condition: &CompiledLogic,
        user_data: &Value,
        internal_context: &Value,
        row: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        macro_rules! recurse {
            ($expr:expr) => {
                self.eval_condition_with_row($expr, user_data, internal_context, row, depth + 1)
            };
        }

        match condition {
            // Literals (no context needed)
            CompiledLogic::Null => Ok(Value::Null),
            CompiledLogic::Bool(b) => Ok(Value::Bool(*b)),
            CompiledLogic::Number(n) => Ok(self.f64_to_json(*n)),
            CompiledLogic::String(s) => Ok(Value::String(s.clone())),

            // Var: check internal_context → row → user_data
            CompiledLogic::Var(name, default) => {
                let value = if name.is_empty() {
                    self.get_var(user_data, name)
                } else {
                    self.get_var(internal_context, name)
                        .or_else(|| self.get_var(row, name))
                        .or_else(|| self.get_var(user_data, name))
                };
                match value {
                    Some(v) if !v.is_null() => Ok(v.clone()),
                    _ => match default {
                        Some(def) => recurse!(def),
                        None => Ok(Value::Null),
                    },
                }
            }

            // Ref ($ref): check internal_context → user_data (NOT row, $ref is outer-scope)
            CompiledLogic::Ref(path, default) => {
                let value = self.get_var(internal_context, path)
                    .or_else(|| self.get_var(user_data, path));
                match value {
                    Some(v) if !v.is_null() => Ok(v.clone()),
                    _ => match default {
                        Some(def) => recurse!(def),
                        None => Ok(Value::Null),
                    },
                }
            }

            // Comparisons
            CompiledLogic::Equal(a, b) => {
                Ok(Value::Bool(helpers::loose_equal(&recurse!(a)?, &recurse!(b)?)))
            }
            CompiledLogic::NotEqual(a, b) => {
                Ok(Value::Bool(!helpers::loose_equal(&recurse!(a)?, &recurse!(b)?)))
            }
            CompiledLogic::StrictEqual(a, b) => Ok(Value::Bool(recurse!(a)? == recurse!(b)?)),
            CompiledLogic::StrictNotEqual(a, b) => Ok(Value::Bool(recurse!(a)? != recurse!(b)?)),
            CompiledLogic::LessThan(a, b) => Ok(Value::Bool(
                helpers::to_f64(&recurse!(a)?) < helpers::to_f64(&recurse!(b)?),
            )),
            CompiledLogic::LessThanOrEqual(a, b) => Ok(Value::Bool(
                helpers::to_f64(&recurse!(a)?) <= helpers::to_f64(&recurse!(b)?),
            )),
            CompiledLogic::GreaterThan(a, b) => Ok(Value::Bool(
                helpers::to_f64(&recurse!(a)?) > helpers::to_f64(&recurse!(b)?),
            )),
            CompiledLogic::GreaterThanOrEqual(a, b) => Ok(Value::Bool(
                helpers::to_f64(&recurse!(a)?) >= helpers::to_f64(&recurse!(b)?),
            )),

            // Logical
            CompiledLogic::And(items) => {
                for item in items {
                    if !helpers::is_truthy(&recurse!(item)?) {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            CompiledLogic::Or(items) => {
                for item in items {
                    if helpers::is_truthy(&recurse!(item)?) {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }
            CompiledLogic::Not(expr) => Ok(Value::Bool(!helpers::is_truthy(&recurse!(expr)?))),
            CompiledLogic::If(cond, then_expr, else_expr) => {
                if helpers::is_truthy(&recurse!(cond)?) {
                    recurse!(then_expr)
                } else {
                    recurse!(else_expr)
                }
            }

            // Fallback: for any other operator (arithmetic, string ops, nested table ops…),
            // fall back to the merge-context approach. These are uncommon in FINDINDEX conditions.
            _ => {
                let mut merged = match internal_context {
                    Value::Object(ctx_map) => ctx_map.clone(),
                    _ => serde_json::Map::new(),
                };
                if let Value::Object(row_map) = row {
                    for (k, v) in row_map {
                        merged.entry(k.clone()).or_insert_with(|| v.clone());
                    }
                }
                let row_ctx = Value::Object(merged);
                self.evaluate_with_context(condition, user_data, &row_ctx, depth)
            }
        }
    }
}
