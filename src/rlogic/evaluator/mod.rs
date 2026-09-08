use super::compiled::CompiledLogic;
use super::config::RLogicConfig;
use index::TableIndex;
use serde_json::Value;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::RwLock;

pub mod arithmetic;
pub mod array_lookup;
pub mod array_ops;
pub mod comparison;
pub mod date_ops;
pub mod helpers;
pub mod index;
pub mod logical;
pub mod math_ops;
pub mod optimizations;
pub mod string_ops;
pub mod types;

pub use helpers::*;
pub use types::*;

/// Active self-table scope set during `evaluate_table_inner`.
///
/// # Safety
/// `rows` is a raw pointer to `local_rows` on the stack of `evaluate_table_inner`.
/// Valid lifetime: from `enter_table_scope()` to `TableScopeGuard::drop()`.
/// Evaluation is single-threaded (protected by `eval_lock` in `evaluate_internal`).
const EMPTY_CACHE_SLOT: std::cell::Cell<(usize, u32, u32)> = std::cell::Cell::new((0, 0, 0));

pub(crate) struct TableScope {
    /// Normalized JSON pointer path to the table being evaluated
    pub path: String,
    /// Path without leading '#' for zero-overhead matching
    pub path_no_hash: String,
    /// Pointer to the local rows being built in table_evaluate_inner
    pub rows: *const Vec<Value>,
    /// Pointer to flat cells storage (total_rows * col_count) during Repeat
    pub flat_cells: *mut Value,
    pub col_count: usize,
    pub total_rows: usize,
    pub existing_row_count: usize,
    /// Fast mapping from column name to column index
    pub col_map: rapidhash::RapidHashMap<String, usize>,
    /// Direct-mapped 256-slot cache with pointer-identity fast path for rapid column resolution
    pub col_cache: [std::cell::Cell<(usize, u32, u32)>; 256],
    /// Optional cursor to the current row index being evaluated (for fast $column lookup)
    pub current_row: Option<usize>,
    /// Precomputed row base pointer in flat_cells for O(1) cell access
    pub current_row_base: *mut Value,
    /// Raw iteration integer value
    pub iteration_raw: Option<i64>,
    /// Optional pre-computed iteration value for O(1) $iteration resolution
    pub iteration_val: Option<Value>,
    /// Optional pre-computed threshold value for O(1) $threshold resolution
    pub threshold_val: Option<Value>,
    /// Memoization cache for combined array lookups on immutable borrowed reference tables
    pub lookup_cache:
        std::cell::RefCell<rapidhash::RapidHashMap<types::CombinedLookupKey, Option<usize>>>,
}

impl TableScope {
    #[inline(always)]
    pub fn get_col_idx(&self, col_name: &str) -> Option<usize> {
        let ptr = col_name.as_ptr() as usize;
        let len = col_name.len() as u32;
        let slot = (ptr ^ (ptr >> 6) ^ (len as usize)) & 255;
        let entry = self.col_cache[slot].get();
        if entry.0 == ptr && entry.1 == len && ptr != 0 {
            return Some(entry.2 as usize);
        }
        if let Some(&col_idx) = self.col_map.get(col_name) {
            self.col_cache[slot].set((ptr, len, col_idx as u32));
            Some(col_idx)
        } else {
            None
        }
    }
}

// SAFETY: table evaluation is protected by eval_lock (single-threaded access).
// UnsafeCell provides interior mutability without adding Sync constraints.
// The raw *const pointer in TableScope is only accessed under eval_lock.
unsafe impl Send for TableScope {}
unsafe impl Send for Evaluator {}
unsafe impl Sync for Evaluator {}

/// RAII guard that clears the active TableScope on drop
pub struct TableScopeGuard<'a> {
    evaluator: &'a Evaluator,
}

impl<'a> Drop for TableScopeGuard<'a> {
    fn drop(&mut self) {
        // SAFETY: single-threaded (eval_lock), no concurrent access
        unsafe {
            *self.evaluator.table_scope.get() = None;
        }
    }
}

/// High-performance zero-copy evaluator with dual-context support
///
/// ## Design Principles
/// 1. **Zero-copy**: All data access via references, no cloning
/// 2. **Dual-context**: Separate user_data and internal_context for scoped variables
/// 3. **Recursive**: Clean recursive evaluation with depth tracking
///
/// ## Context Resolution
/// - Variables ($var) lookup order: internal_context → user_data
/// - Internal context holds: $iteration, $threshold, $loopIteration, etc.
pub struct Evaluator {
    config: RLogicConfig,
    /// Upfront indices for large tables (name -> index)
    indices: RwLock<HashMap<String, TableIndex>>,
    /// Extracted large static arrays for zero-copy resolution
    static_arrays:
        UnsafeCell<Option<std::sync::Arc<indexmap::IndexMap<String, std::sync::Arc<Value>>>>>,
    /// Active self-table scope during table evaluation (None outside table eval)
    pub(crate) table_scope: UnsafeCell<Option<TableScope>>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            config: RLogicConfig::default(),
            indices: RwLock::new(HashMap::new()),
            static_arrays: UnsafeCell::new(None),
            table_scope: UnsafeCell::new(None),
        }
    }

    /// Register a table scope for self-reference interception.
    ///
    /// Returns a guard that clears the scope on drop.
    ///
    /// # Safety
    /// `rows` must outlive the returned guard. The guard MUST be dropped before
    /// `rows` is moved or dropped. Caller (table_evaluate_inner) is responsible.
    pub(crate) fn enter_table_scope<'a>(
        &'a self,
        path: String,
        rows: &Vec<Value>,
    ) -> TableScopeGuard<'a> {
        let path_no_hash = path.trim_start_matches('#').to_string();
        // SAFETY: single-threaded (eval_lock held by caller)
        unsafe {
            *self.table_scope.get() = Some(TableScope {
                path,
                path_no_hash,
                rows: rows as *const Vec<Value>,
                flat_cells: std::ptr::null_mut(),
                col_count: 0,
                total_rows: 0,
                existing_row_count: 0,
                col_map: rapidhash::RapidHashMap::default(),
                col_cache: [EMPTY_CACHE_SLOT; 256],
                current_row: None,
                current_row_base: std::ptr::null_mut(),
                iteration_raw: None,
                iteration_val: None,
                threshold_val: None,
                lookup_cache: std::cell::RefCell::new(rapidhash::RapidHashMap::default()),
            });
        }
        TableScopeGuard { evaluator: self }
    }

    /// Register flat cell buffer and column mappings for fast direct indexed evaluation
    pub(crate) fn set_table_scope_flat_cells(
        &self,
        cells: *mut Value,
        col_count: usize,
        total_rows: usize,
        existing_row_count: usize,
        col_map: rapidhash::RapidHashMap<String, usize>,
    ) {
        // SAFETY: single-threaded (eval_lock held by caller)
        unsafe {
            if let Some(ts) = (*self.table_scope.get()).as_mut() {
                ts.flat_cells = cells;
                ts.col_count = col_count;
                ts.total_rows = total_rows;
                ts.existing_row_count = existing_row_count;
                ts.col_map = col_map;
                ts.col_cache = [EMPTY_CACHE_SLOT; 256];
                ts.current_row_base = std::ptr::null_mut();
            }
        }
    }

    /// Update the rows pointer in the active table scope.
    pub(crate) fn update_table_scope_rows(&self, rows: &Vec<Value>) {
        // SAFETY: single-threaded (eval_lock held by caller)
        unsafe {
            if let Some(ts) = (*self.table_scope.get()).as_mut() {
                ts.rows = rows as *const Vec<Value>;
            }
        }
    }

    /// Set the row cursor for the active table scope
    pub(crate) fn set_table_scope_row(&self, row_idx: Option<usize>) {
        // SAFETY: single-threaded (eval_lock held by caller)
        unsafe {
            if let Some(ts) = (*self.table_scope.get()).as_mut() {
                ts.current_row = row_idx;
                if let Some(r) = row_idx {
                    if ts.col_count > 0
                        && !ts.flat_cells.is_null()
                        && r >= ts.existing_row_count
                        && r < ts.existing_row_count + ts.total_rows
                    {
                        ts.current_row_base = ts
                            .flat_cells
                            .add((r - ts.existing_row_count) * ts.col_count);
                    } else {
                        ts.current_row_base = std::ptr::null_mut();
                    }
                } else {
                    ts.current_row_base = std::ptr::null_mut();
                }
            }
        }
    }

    /// Set the row cursor and pre-computed iteration value for the active table scope
    pub(crate) fn set_table_scope_cursor(&self, row_idx: Option<usize>, iteration: Option<i64>) {
        // SAFETY: single-threaded (eval_lock held by caller)
        unsafe {
            if let Some(ts) = (*self.table_scope.get()).as_mut() {
                ts.current_row = row_idx;
                ts.iteration_raw = iteration;
                ts.iteration_val = iteration.map(Value::from);
                if let Some(r) = row_idx {
                    if ts.col_count > 0
                        && !ts.flat_cells.is_null()
                        && r >= ts.existing_row_count
                        && r < ts.existing_row_count + ts.total_rows
                    {
                        ts.current_row_base = ts
                            .flat_cells
                            .add((r - ts.existing_row_count) * ts.col_count);
                    } else {
                        ts.current_row_base = std::ptr::null_mut();
                    }
                } else {
                    ts.current_row_base = std::ptr::null_mut();
                }
            }
        }
    }

    /// Set the threshold value for the active table scope
    pub(crate) fn set_table_scope_threshold(&self, threshold: i64) {
        // SAFETY: single-threaded (eval_lock held by caller)
        unsafe {
            if let Some(ts) = (*self.table_scope.get()).as_mut() {
                ts.threshold_val = Some(Value::from(threshold));
            }
        }
    }

    pub fn with_config(mut self, config: RLogicConfig) -> Self {
        self.config = config;
        self
    }

    /// Set static arrays for evaluation context
    pub fn set_static_arrays(
        &self,
        static_arrays: std::sync::Arc<indexmap::IndexMap<String, std::sync::Arc<Value>>>,
    ) {
        // SAFETY: single-threaded (eval_lock held by caller)
        unsafe {
            *self.static_arrays.get() = Some(static_arrays);
        }
    }

    /// Build and store index for a table
    pub fn index_table(&self, name: &str, data: &Value) {
        if let Some(index) = TableIndex::new(data) {
            if let Ok(mut indices) = self.indices.write() {
                indices.insert(name.to_string(), index);
            }
        }
    }

    /// Clear all stored indices
    pub fn clear_indices(&self) {
        if let Ok(mut indices) = self.indices.write() {
            indices.clear();
        }
    }

    /// Public API: Evaluate compiled logic with user data only
    /// Uses fast path for simple cases to avoid recursion overhead
    #[inline]
    pub fn evaluate(&self, logic: &CompiledLogic, data: &Value) -> Result<Value, String> {
        // Fast path for literals (most common cases)
        match logic {
            CompiledLogic::Null => return Ok(Value::Null),
            CompiledLogic::Bool(b) => return Ok(Value::Bool(*b)),
            CompiledLogic::Number(n) => {
                return Ok(self.f64_to_json(*n));
            }
            CompiledLogic::String(s) => return Ok(Value::String(s.clone())),
            CompiledLogic::Var(name, None) if !name.is_empty() => {
                // Simple variable without default
                return self.eval_var_or_default(name, &None, data, &Value::Null, 0);
            }
            CompiledLogic::Ref(path, None) if !path.is_empty() => {
                // Simple variable without default
                return self.eval_var_or_default(path, &None, data, &Value::Null, 0);
            }
            // Fast path for arithmetic operations
            CompiledLogic::Add(_)
            | CompiledLogic::Subtract(_)
            | CompiledLogic::Multiply(_)
            | CompiledLogic::Divide(_) => {
                if let Some(result) = self.eval_f64(logic, data, &Value::Null, 0)? {
                    return Ok(self.f64_to_json(result));
                }
            }
            _ => {}
        }

        // Fall back to full evaluation for complex cases
        self.evaluate_with_context(logic, data, &Value::Null, 0)
    }

    /// Evaluate with internal context (for scoped variables)
    ///
    /// # Arguments
    /// * `logic` - The compiled logic expression to evaluate
    /// * `user_data` - User's data (primary lookup source)
    /// * `internal_context` - Internal variables (e.g., $iteration, $loopIteration)
    ///
    /// # Zero-Copy Guarantee
    /// This method uses only references and never clones the data contexts.
    /// Internal variables are looked up first in `internal_context`, then fall back to `user_data`.
    #[inline]
    pub fn evaluate_with_internal_context(
        &self,
        logic: &CompiledLogic,
        user_data: &Value,
        internal_context: &Value,
    ) -> Result<Value, String> {
        self.evaluate_with_context(logic, user_data, internal_context, 0)
    }

    /// Internal recursive evaluation with depth tracking
    ///
    /// # Context Resolution Order
    /// 1. Check internal_context first (for scoped variables like $loopIteration)
    /// 2. Fall back to user_data (for regular user variables)
    ///
    /// This enables zero-copy scoped variable handling without merging contexts.
    fn evaluate_with_context(
        &self,
        logic: &CompiledLogic,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        // Recursion limit check
        if depth > self.config.recursion_limit {
            return Err("Recursion limit exceeded".to_string());
        }

        match logic {
            // ========== Literals ==========
            CompiledLogic::Null => Ok(Value::Null),
            CompiledLogic::Bool(b) => Ok(Value::Bool(*b)),
            CompiledLogic::Number(n) => Ok(self.f64_to_json(*n)),
            CompiledLogic::String(s) => Ok(Value::String(s.clone())),
            CompiledLogic::Array(arr) => {
                let results: Result<Vec<_>, _> = arr
                    .iter()
                    .map(|item| {
                        self.evaluate_with_context(item, user_data, internal_context, depth + 1)
                    })
                    .collect();
                Ok(Value::Array(results?))
            }

            // ========== Variable Access (Zero-Copy) ==========
            CompiledLogic::Var(name, default) => {
                self.eval_var_or_default(name, default, user_data, internal_context, depth)
            }

            CompiledLogic::Ref(path, default) => {
                self.eval_var_or_default(path, default, user_data, internal_context, depth)
            }

            // ========== Logical Operators ==========
            CompiledLogic::And(items) => {
                self.eval_and_or(items, true, user_data, internal_context, depth)
            }
            CompiledLogic::Or(items) => {
                self.eval_and_or(items, false, user_data, internal_context, depth)
            }
            CompiledLogic::Not(expr) => {
                let result =
                    self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
                Ok(Value::Bool(!is_truthy(&result)))
            }
            CompiledLogic::If(cond, then_expr, else_expr) => {
                if self.eval_truthy(cond, user_data, internal_context, depth + 1)? {
                    self.evaluate_with_context(then_expr, user_data, internal_context, depth + 1)
                } else {
                    self.evaluate_with_context(else_expr, user_data, internal_context, depth + 1)
                }
            }

            // ========== Comparison Operators ==========
            CompiledLogic::Equal(a, b) => {
                self.eval_binary_compare(CompOp::Eq, a, b, user_data, internal_context, depth)
            }
            CompiledLogic::StrictEqual(a, b) => {
                self.eval_binary_compare(CompOp::StrictEq, a, b, user_data, internal_context, depth)
            }
            CompiledLogic::NotEqual(a, b) => {
                self.eval_binary_compare(CompOp::Ne, a, b, user_data, internal_context, depth)
            }
            CompiledLogic::StrictNotEqual(a, b) => {
                self.eval_binary_compare(CompOp::StrictNe, a, b, user_data, internal_context, depth)
            }
            CompiledLogic::LessThan(a, b) => {
                self.eval_binary_compare(CompOp::Lt, a, b, user_data, internal_context, depth)
            }
            CompiledLogic::LessThanOrEqual(a, b) => {
                self.eval_binary_compare(CompOp::Le, a, b, user_data, internal_context, depth)
            }
            CompiledLogic::GreaterThan(a, b) => {
                self.eval_binary_compare(CompOp::Gt, a, b, user_data, internal_context, depth)
            }
            CompiledLogic::GreaterThanOrEqual(a, b) => {
                self.eval_binary_compare(CompOp::Ge, a, b, user_data, internal_context, depth)
            }

            // ========== Arithmetic Operators ==========
            CompiledLogic::Add(_)
            | CompiledLogic::Subtract(_)
            | CompiledLogic::Multiply(_)
            | CompiledLogic::Divide(_)
            | CompiledLogic::Power(_, _)
            | CompiledLogic::Modulo(_, _) => {
                match self.eval_f64(logic, user_data, internal_context, depth)? {
                    Some(result) => Ok(self.f64_to_json(result)),
                    None => Ok(Value::Null),
                }
            }

            // ========== Array Operations ==========
            CompiledLogic::Map(array_expr, logic_expr) => {
                self.eval_map(array_expr, logic_expr, user_data, internal_context, depth)
            }
            CompiledLogic::Filter(array_expr, logic_expr) => {
                self.eval_filter(array_expr, logic_expr, user_data, internal_context, depth)
            }
            CompiledLogic::Reduce(array_expr, logic_expr, initial_expr) => self.eval_reduce(
                array_expr,
                logic_expr,
                initial_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::All(array_expr, logic_expr) => self.eval_quantifier(
                Quantifier::All,
                array_expr,
                logic_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::Some(array_expr, logic_expr) => self.eval_quantifier(
                Quantifier::Some,
                array_expr,
                logic_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::None(array_expr, logic_expr) => self.eval_quantifier(
                Quantifier::None,
                array_expr,
                logic_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::Merge(items) => {
                self.eval_merge(items, user_data, internal_context, depth)
            }
            CompiledLogic::In(value_expr, array_expr) => {
                self.eval_in(value_expr, array_expr, user_data, internal_context, depth)
            }
            CompiledLogic::Sum(array_expr, field_expr, threshold_expr) => self.eval_sum(
                array_expr,
                field_expr,
                threshold_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::For(start_expr, end_expr, logic_expr) => self.eval_for(
                start_expr,
                end_expr,
                logic_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::Multiplies(items) => {
                self.eval_multiplies(items, user_data, internal_context, depth)
            }
            CompiledLogic::Divides(items) => {
                self.eval_divides(items, user_data, internal_context, depth)
            }

            // ========== Array Lookup Operations ==========
            CompiledLogic::ValueAt(table_expr, row_idx_expr, col_name_expr) => self.eval_valueat(
                table_expr,
                row_idx_expr,
                col_name_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::MaxAt(table_expr, col_name_expr) => self.eval_maxat(
                table_expr,
                col_name_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::IndexAt(lookup_expr, table_expr, field_expr, range_expr) => self
                .eval_indexat(
                    lookup_expr,
                    table_expr,
                    field_expr,
                    range_expr,
                    user_data,
                    internal_context,
                    depth,
                ),
            CompiledLogic::Match(table_expr, conditions) => {
                self.eval_match(table_expr, conditions, user_data, internal_context, depth)
            }
            CompiledLogic::MatchRange(table_expr, conditions) => {
                self.eval_matchrange(table_expr, conditions, user_data, internal_context, depth)
            }
            CompiledLogic::Choose(table_expr, conditions) => {
                self.eval_choose(table_expr, conditions, user_data, internal_context, depth)
            }
            CompiledLogic::FindIndex(table_expr, conditions) => {
                self.eval_findindex(table_expr, conditions, user_data, internal_context, depth)
            }

            // ========== String Operations ==========
            CompiledLogic::Cat(items) => {
                self.concat_strings(items, user_data, internal_context, depth)
            }
            CompiledLogic::Substr(string_expr, start_expr, length_expr) => self.eval_substr(
                string_expr,
                start_expr,
                length_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::Search(find_expr, within_expr, start_expr) => self.eval_search(
                find_expr,
                within_expr,
                start_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::Left(text_expr, num_expr) => self.extract_text_side(
                text_expr,
                num_expr.as_deref(),
                true,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::Right(text_expr, num_expr) => self.extract_text_side(
                text_expr,
                num_expr.as_deref(),
                false,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::Mid(text_expr, start_expr, num_expr) => self.eval_mid(
                text_expr,
                start_expr,
                num_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::SplitText(value_expr, sep_expr, index_expr) => self.eval_split_text(
                value_expr,
                sep_expr,
                index_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::Concat(items) => {
                self.concat_strings(items, user_data, internal_context, depth)
            }
            CompiledLogic::SplitValue(string_expr, sep_expr) => {
                self.eval_split_value(string_expr, sep_expr, user_data, internal_context, depth)
            }
            CompiledLogic::StringFormat(value_expr, decimals, prefix, suffix, thousands_sep) => {
                self.eval_string_format(
                    value_expr,
                    decimals,
                    prefix,
                    suffix,
                    thousands_sep,
                    user_data,
                    internal_context,
                    depth,
                )
            }
            CompiledLogic::Length(expr) => {
                self.eval_length(expr, user_data, internal_context, depth)
            }
            CompiledLogic::Len(expr) => self.eval_len(expr, user_data, internal_context, depth),

            // ========== Math Operations ==========
            CompiledLogic::Abs(expr) => {
                self.eval_unary_math(expr, |n| n.abs(), user_data, internal_context, depth)
            }
            CompiledLogic::Max(items) => {
                self.eval_min_max(items, true, user_data, internal_context, depth)
            }
            CompiledLogic::Min(items) => {
                self.eval_min_max(items, false, user_data, internal_context, depth)
            }
            CompiledLogic::Pow(base_expr, exp_expr) => {
                self.eval_pow(base_expr, exp_expr, user_data, internal_context, depth)
            }
            CompiledLogic::Round(expr, decimals) => {
                self.apply_round(expr, decimals, 0, user_data, internal_context, depth)
            }
            CompiledLogic::RoundUp(expr, decimals) => {
                self.apply_round(expr, decimals, 1, user_data, internal_context, depth)
            }
            CompiledLogic::RoundDown(expr, decimals) => {
                self.apply_round(expr, decimals, 2, user_data, internal_context, depth)
            }
            CompiledLogic::Ceiling(expr, significance) => {
                self.eval_ceiling(expr, significance, user_data, internal_context, depth)
            }
            CompiledLogic::Floor(expr, significance) => {
                self.eval_floor(expr, significance, user_data, internal_context, depth)
            }
            CompiledLogic::Trunc(expr, decimals) => {
                self.eval_trunc(expr, decimals, user_data, internal_context, depth)
            }
            CompiledLogic::Mround(value_expr, multiple_expr) => self.eval_mround(
                value_expr,
                multiple_expr,
                user_data,
                internal_context,
                depth,
            ),

            // ========== Date Operations ==========
            CompiledLogic::Today => self.eval_today(),
            CompiledLogic::Now => self.eval_now(),
            CompiledLogic::Days(end_expr, start_expr) => {
                self.eval_days(end_expr, start_expr, user_data, internal_context, depth)
            }
            CompiledLogic::Year(expr) => {
                self.extract_date_component(expr, "year", user_data, internal_context, depth)
            }
            CompiledLogic::Month(expr) => {
                self.extract_date_component(expr, "month", user_data, internal_context, depth)
            }
            CompiledLogic::Day(expr) => {
                self.extract_date_component(expr, "day", user_data, internal_context, depth)
            }
            CompiledLogic::Date(year_expr, month_expr, day_expr) => self.eval_date(
                year_expr,
                month_expr,
                day_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::DateFormat(date_expr, format_expr) => {
                self.eval_date_format(date_expr, format_expr, user_data, internal_context, depth)
            }
            CompiledLogic::YearFrac(start_expr, end_expr, basis_expr) => self.eval_year_frac(
                start_expr,
                end_expr,
                basis_expr,
                user_data,
                internal_context,
                depth,
            ),
            CompiledLogic::DateDif(start_expr, end_expr, unit_expr) => self.eval_date_dif(
                start_expr,
                end_expr,
                unit_expr,
                user_data,
                internal_context,
                depth,
            ),

            // ========== Utility Operators ==========
            CompiledLogic::Missing(keys) => {
                let missing: Vec<_> = keys
                    .iter()
                    .filter(|key| self.is_key_missing(user_data, key))
                    .map(|k| Value::String(k.clone()))
                    .collect();
                Ok(Value::Array(missing))
            }
            CompiledLogic::MissingSome(min_expr, keys) => {
                let min_val =
                    self.evaluate_with_context(min_expr, user_data, internal_context, depth + 1)?;
                let minimum = to_number(&min_val) as usize;

                let present = keys
                    .iter()
                    .filter(|key| !self.is_key_missing(user_data, key))
                    .count();

                if present >= minimum {
                    Ok(Value::Array(vec![]))
                } else {
                    let missing: Vec<_> = keys
                        .iter()
                        .filter(|key| self.is_key_missing(user_data, key))
                        .map(|k| Value::String(k.clone()))
                        .collect();
                    Ok(Value::Array(missing))
                }
            }

            // ========== Logical Utility Operators ==========
            CompiledLogic::Xor(a_expr, b_expr) => {
                let a_val =
                    self.evaluate_with_context(a_expr, user_data, internal_context, depth + 1)?;
                let b_val =
                    self.evaluate_with_context(b_expr, user_data, internal_context, depth + 1)?;
                Ok(Value::Bool(is_truthy(&a_val) ^ is_truthy(&b_val)))
            }
            CompiledLogic::IfNull(cond_expr, alt_expr) => {
                let cond_val =
                    self.evaluate_with_context(cond_expr, user_data, internal_context, depth + 1)?;
                if is_null_like(&cond_val) {
                    self.evaluate_with_context(alt_expr, user_data, internal_context, depth + 1)
                } else {
                    Ok(cond_val)
                }
            }
            CompiledLogic::IsEmpty(expr) => {
                let val =
                    self.evaluate_with_context(expr, user_data, internal_context, depth + 1)?;
                let empty = match &val {
                    Value::Null => true,
                    Value::String(s) => s.is_empty(),
                    _ => false,
                };
                Ok(Value::Bool(empty))
            }
            CompiledLogic::Empty => Ok(Value::String(String::new())),

            // ========== UI Helper Operators ==========
            CompiledLogic::RangeOptions(min_expr, max_expr) => {
                let min_val =
                    self.evaluate_with_context(min_expr, user_data, internal_context, depth + 1)?;
                let max_val =
                    self.evaluate_with_context(max_expr, user_data, internal_context, depth + 1)?;

                let min = to_number(&min_val) as i32;
                let max = to_number(&max_val) as i32;

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
                let table_val =
                    self.evaluate_with_context(table_expr, user_data, internal_context, depth + 1)?;
                let label_val =
                    self.evaluate_with_context(label_expr, user_data, internal_context, depth + 1)?;
                let value_val =
                    self.evaluate_with_context(value_expr, user_data, internal_context, depth + 1)?;

                if let (Value::Array(arr), Value::String(label_field), Value::String(value_field)) =
                    (&table_val, &label_val, &value_val)
                {
                    let options: Vec<Value> = arr
                        .iter()
                        .filter_map(|row| {
                            row.as_object().and_then(|obj| {
                                Some(create_option(obj.get(label_field)?, obj.get(value_field)?))
                            })
                        })
                        .collect();
                    Ok(Value::Array(options))
                } else {
                    Ok(Value::Array(vec![]))
                }
            }
            CompiledLogic::MapOptionsIf(table_expr, label_expr, value_expr, conditions) => {
                let table_val =
                    self.evaluate_with_context(table_expr, user_data, internal_context, depth + 1)?;
                let label_val =
                    self.evaluate_with_context(label_expr, user_data, internal_context, depth + 1)?;
                let value_val =
                    self.evaluate_with_context(value_expr, user_data, internal_context, depth + 1)?;

                if let (Value::Array(arr), Value::String(label_field), Value::String(value_field)) =
                    (&table_val, &label_val, &value_val)
                {
                    let mut options = Vec::new();

                    for row in arr {
                        let obj = match row.as_object() {
                            Some(obj) => obj,
                            None => continue,
                        };

                        let mut all_match = true;

                        for condition in conditions {
                            // Evaluate condition with row as primary context, user_data as fallback
                            let result =
                                self.evaluate_with_context(condition, row, user_data, depth + 1)?;
                            if !is_truthy(&result) {
                                all_match = false;
                                break;
                            }
                        }

                        if all_match {
                            if let (Some(label), Some(value)) =
                                (obj.get(label_field), obj.get(value_field))
                            {
                                options.push(create_option(label, value));
                            }
                        }
                    }

                    Ok(Value::Array(options))
                } else {
                    Ok(Value::Array(vec![]))
                }
            }
            CompiledLogic::Return(value) => {
                // Return the raw value as-is without any evaluation
                Ok(value.as_ref().clone())
            }
        }
    }

    /// Helper for evaluating variable/ref with default (zero-copy)
    #[inline]
    fn eval_var_or_default(
        &self,
        name: &str,
        default: &Option<Box<CompiledLogic>>,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String> {
        // Fast path: check active table scope first.
        // When evaluating a table's own columns (forward/backward pass), Var/Ref nodes
        // that resolve to the table's own path (e.g. used in MAP/FILTER/REDUCE over self)
        // must see local_rows, not stale data in scope_data.
        if !name.is_empty() {
            // SAFETY: single-threaded (eval_lock), UnsafeCell
            let scope = unsafe { &*self.table_scope.get() };
            if let Some(ts) = scope.as_ref() {
                if name == ts.path || name.trim_start_matches('#') == ts.path_no_hash.as_str() {
                    if ts.col_count > 0 && !ts.flat_cells.is_null() {
                        let mut arr = Vec::with_capacity(ts.existing_row_count + ts.total_rows);
                        let rows = unsafe { &*ts.rows };
                        for r in 0..ts.existing_row_count {
                            if let Some(row) = rows.get(r) {
                                arr.push(row.clone());
                            }
                        }
                        for r in 0..ts.total_rows {
                            let mut row_map = serde_json::Map::with_capacity(ts.col_count);
                            let row_offset = r * ts.col_count;
                            for (c_name, &c_idx) in ts.col_map.iter() {
                                let cell = unsafe { &*ts.flat_cells.add(row_offset + c_idx) };
                                row_map.insert(c_name.clone(), cell.clone());
                            }
                            arr.push(Value::Object(row_map));
                        }
                        return Ok(Value::Array(arr));
                    }
                    // SAFETY: local_rows outlives this evaluation frame
                    let rows = unsafe { &*ts.rows };
                    return Ok(Value::Array(rows.clone()));
                }
            }
        }

        // Special case: empty name "" refers to root context (user_data only)
        // For named variables, try internal context first (for $loopIteration, $iteration, etc.)
        let value = if name.is_empty() {
            self.get_var(user_data, name)
        } else {
            self.get_var(internal_context, name)
                .or_else(|| self.get_var(user_data, name))
        };
        match value {
            Some(v) if !v.is_null() => Ok(v.clone()), // Only clone the resolved value
            _ => {
                if let Some(def) = default {
                    self.evaluate_with_context(def, user_data, internal_context, depth + 1)
                } else {
                    Ok(Value::Null)
                }
            }
        }
    }

    /// Convert f64 to JSON number
    #[inline(always)]
    pub fn f64_to_value(&self, f: f64) -> Value {
        helpers::f64_to_json(f, self.config.safe_nan_handling)
    }

    #[inline(always)]
    fn f64_to_json(&self, f: f64) -> Value {
        self.f64_to_value(f)
    }

    #[inline(always)]
    pub fn eval_fast_f64(
        &self,
        logic: &CompiledLogic,
        user_data: &Value,
        internal_context: &Value,
    ) -> Result<Option<f64>, String> {
        self.eval_f64(logic, user_data, internal_context, 0)
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
