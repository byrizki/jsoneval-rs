use super::compiled::CompiledLogic;
use serde_json::Value;

/// Compact opcodes for table cell evaluation without AST tree recursion
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableOp {
    /// Push literal constant number onto the stack
    PushConst(f64),
    /// Push current $iteration raw integer as f64
    PushIteration,
    /// Push column from current row in flat_cells
    PushCol(usize),
    /// Push cell from self table at (iteration_raw + iter_delta, col_idx)
    PushValueAt {
        col_idx: usize,
        iter_delta: i32,
    },
    /// Push cell from self table at constant row index (e.g. static row 0)
    PushValueAtConstRow {
        col_idx: usize,
        const_row: usize,
    },
    /// Basic arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Negate,
    Power,
    Abs,
    /// Excel rounding
    Round(i32),
    RoundUp(i32),
    RoundDown(i32),
    /// Aggregation
    Min(u8),
    Max(u8),
    /// Comparison
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    /// Logical
    Not,
    /// Control flow
    Jump(usize),
    JumpIfZero(usize),
    JumpIfNotZero(usize),
    /// Lookup in external TypedReferenceTable by key column (IndexAt + ValueAt pattern):
    /// Pops target_val from stack, performs lookup, pushes result
    LookupTypedTable {
        table_id: u16,
        key_col_id: u16,
        val_col_id: u16,
    },
    /// Lookup in external TypedReferenceTable by row index (ValueAt(table, row_idx, val_col)):
    /// Pops row_idx from stack, reads cell, pushes result
    LookupTypedTableRow {
        table_id: u16,
        val_col_id: u16,
    },
    /// Lookup in external TypedReferenceTable by key with fallback default_val if missing / out of bounds
    LookupTypedTableOr {
        table_id: u16,
        key_col_id: u16,
        val_col_id: u16,
        default_val: f64,
    },
    /// Lookup in external TypedReferenceTable by row index with fallback default_val if missing / out of bounds
    LookupTypedTableRowOr {
        table_id: u16,
        val_col_id: u16,
        default_val: f64,
    },
}

/// Compiled linear bytecode for high-speed inner table cell evaluation
#[derive(Debug, Clone)]
pub struct TableBytecode {
    pub ops: Vec<TableOp>,
    pub string_pool: Vec<String>,
}

impl TableBytecode {
    /// Execute bytecode expression against flat cell buffer in a tight stack loop
    ///
    /// # Safety
    /// `flat_cells` must point to a buffer of at least `total_rows * col_count` Values.
    /// `static_rows` must point to valid `Vec<Value>` of previous rows.
    #[inline(always)]
    pub unsafe fn execute(
        &self,
        flat_cells: *const Value,
        col_count: usize,
        row_offset: usize,
        existing_row_count: usize,
        total_rows: usize,
        iteration_raw: i64,
        static_rows: *const Vec<Value>,
        col_names: &[String],
        evaluator: Option<&crate::rlogic::Evaluator>,
    ) -> Option<f64> {
        let mut stack: [f64; 64] = [0.0; 64];
        let mut sp: usize = 0;
        let mut pc: usize = 0;
        let mut steps: usize = 0;
        let num_ops = self.ops.len();

        while pc < num_ops {
            steps += 1;
            if steps > 512 {
                return None;
            }

            match self.ops[pc] {
                TableOp::PushConst(val) => {
                    if sp >= 64 {
                        return None;
                    }
                    stack[sp] = val;
                    sp += 1;
                    pc += 1;
                }
                TableOp::PushIteration => {
                    if sp >= 64 {
                        return None;
                    }
                    stack[sp] = iteration_raw as f64;
                    sp += 1;
                    pc += 1;
                }
                TableOp::PushCol(col_idx) => {
                    if sp >= 64 {
                        return None;
                    }
                    let cell = &*flat_cells.add(row_offset * col_count + col_idx);
                    let val = match cell {
                        Value::Number(n) => n.as_f64().unwrap_or(0.0),
                        _ => super::evaluator::helpers::to_f64(cell),
                    };
                    stack[sp] = val;
                    sp += 1;
                    pc += 1;
                }
                TableOp::PushValueAt {
                    col_idx,
                    iter_delta,
                } => {
                    if sp >= 64 {
                        return None;
                    }
                    let target_iter = iteration_raw + iter_delta as i64;
                    let val = if target_iter >= 0 {
                        let target_row = target_iter as usize;
                        if target_row < existing_row_count {
                            let rows = &*static_rows;
                            if let Some(row) = rows.get(target_row) {
                                if let Value::Object(map) = row {
                                    let col_name = &col_names[col_idx];
                                    if let Some(cell) = map.get(col_name) {
                                        match cell {
                                            Value::Number(n) => n.as_f64().unwrap_or(0.0),
                                            _ => super::evaluator::helpers::to_f64(cell),
                                        }
                                    } else {
                                        0.0
                                    }
                                } else {
                                    0.0
                                }
                            } else {
                                0.0
                            }
                        } else if target_row < existing_row_count + total_rows {
                            let cell_offset = target_row - existing_row_count;
                            let cell = &*flat_cells.add(cell_offset * col_count + col_idx);
                            match cell {
                                Value::Number(n) => n.as_f64().unwrap_or(0.0),
                                _ => super::evaluator::helpers::to_f64(cell),
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    stack[sp] = val;
                    sp += 1;
                    pc += 1;
                }
                TableOp::PushValueAtConstRow { col_idx, const_row } => {
                    if sp >= 64 {
                        return None;
                    }
                    let val = if const_row < existing_row_count {
                        let rows = &*static_rows;
                        if let Some(row) = rows.get(const_row) {
                            if let Value::Object(map) = row {
                                let col_name = &col_names[col_idx];
                                if let Some(cell) = map.get(col_name) {
                                    match cell {
                                        Value::Number(n) => n.as_f64().unwrap_or(0.0),
                                        _ => super::evaluator::helpers::to_f64(cell),
                                    }
                                } else {
                                    0.0
                                }
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        }
                    } else if const_row < existing_row_count + total_rows {
                        let cell_offset = const_row - existing_row_count;
                        let cell = &*flat_cells.add(cell_offset * col_count + col_idx);
                        match cell {
                            Value::Number(n) => n.as_f64().unwrap_or(0.0),
                            _ => super::evaluator::helpers::to_f64(cell),
                        }
                    } else {
                        0.0
                    };
                    stack[sp] = val;
                    sp += 1;
                    pc += 1;
                }
                TableOp::Add => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] += stack[sp];
                    pc += 1;
                }
                TableOp::Subtract => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] -= stack[sp];
                    pc += 1;
                }
                TableOp::Multiply => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] *= stack[sp];
                    pc += 1;
                }
                TableOp::Divide => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    let divisor = stack[sp];
                    if divisor == 0.0 {
                        return None;
                    }
                    stack[sp - 1] /= divisor;
                    pc += 1;
                }
                TableOp::Modulo => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    let divisor = stack[sp];
                    if divisor == 0.0 {
                        return None;
                    }
                    stack[sp - 1] %= divisor;
                    pc += 1;
                }
                TableOp::Negate => {
                    if sp < 1 {
                        return None;
                    }
                    stack[sp - 1] = -stack[sp - 1];
                    pc += 1;
                }
                TableOp::Power => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] = stack[sp - 1].powf(stack[sp]);
                    pc += 1;
                }
                TableOp::Abs => {
                    if sp < 1 {
                        return None;
                    }
                    stack[sp - 1] = stack[sp - 1].abs();
                    pc += 1;
                }
                TableOp::Round(decimals) => {
                    if sp < 1 {
                        return None;
                    }
                    let num = stack[sp - 1];
                    stack[sp - 1] = if decimals == 0 {
                        num.round()
                    } else if decimals > 0 {
                        let mult = 10f64.powi(decimals);
                        (num * mult).round() / mult
                    } else {
                        let div = 10f64.powi(-decimals);
                        (num / div).round() * div
                    };
                    pc += 1;
                }
                TableOp::RoundUp(decimals) => {
                    if sp < 1 {
                        return None;
                    }
                    let num = stack[sp - 1];
                    stack[sp - 1] = if decimals == 0 {
                        num.ceil()
                    } else if decimals > 0 {
                        let mult = 10f64.powi(decimals);
                        (num * mult).ceil() / mult
                    } else {
                        let div = 10f64.powi(-decimals);
                        (num / div).ceil() * div
                    };
                    pc += 1;
                }
                TableOp::RoundDown(decimals) => {
                    if sp < 1 {
                        return None;
                    }
                    let num = stack[sp - 1];
                    stack[sp - 1] = if decimals == 0 {
                        num.floor()
                    } else if decimals > 0 {
                        let mult = 10f64.powi(decimals);
                        (num * mult).floor() / mult
                    } else {
                        let div = 10f64.powi(-decimals);
                        (num / div).floor() * div
                    };
                    pc += 1;
                }
                TableOp::Min(count) => {
                    let n = count as usize;
                    if sp < n || n == 0 {
                        return None;
                    }
                    let mut min_val = stack[sp - n];
                    for i in 1..n {
                        let v = stack[sp - n + i];
                        if v < min_val {
                            min_val = v;
                        }
                    }
                    sp -= n - 1;
                    stack[sp - 1] = min_val;
                    pc += 1;
                }
                TableOp::Max(count) => {
                    let n = count as usize;
                    if sp < n || n == 0 {
                        return None;
                    }
                    let mut max_val = stack[sp - n];
                    for i in 1..n {
                        let v = stack[sp - n + i];
                        if v > max_val {
                            max_val = v;
                        }
                    }
                    sp -= n - 1;
                    stack[sp - 1] = max_val;
                    pc += 1;
                }
                TableOp::Eq => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    let b = stack[sp];
                    let a = stack[sp - 1];
                    stack[sp - 1] = if (a == b) || (a - b).abs() < 1e-9 {
                        1.0
                    } else {
                        0.0
                    };
                    pc += 1;
                }
                TableOp::Ne => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    let b = stack[sp];
                    let a = stack[sp - 1];
                    stack[sp - 1] = if (a != b) && (a - b).abs() >= 1e-9 {
                        1.0
                    } else {
                        0.0
                    };
                    pc += 1;
                }
                TableOp::Lt => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] = if stack[sp - 1] < stack[sp] { 1.0 } else { 0.0 };
                    pc += 1;
                }
                TableOp::Lte => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] = if stack[sp - 1] <= stack[sp] { 1.0 } else { 0.0 };
                    pc += 1;
                }
                TableOp::Gt => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] = if stack[sp - 1] > stack[sp] { 1.0 } else { 0.0 };
                    pc += 1;
                }
                TableOp::Gte => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] = if stack[sp - 1] >= stack[sp] { 1.0 } else { 0.0 };
                    pc += 1;
                }
                TableOp::Not => {
                    if sp < 1 {
                        return None;
                    }
                    stack[sp - 1] = if stack[sp - 1] == 0.0 { 1.0 } else { 0.0 };
                    pc += 1;
                }
                TableOp::Jump(target) => {
                    pc = target;
                }
                TableOp::JumpIfZero(target) => {
                    if sp < 1 {
                        return None;
                    }
                    sp -= 1;
                    if stack[sp] == 0.0 {
                        pc = target;
                    } else {
                        pc += 1;
                    }
                }
                TableOp::JumpIfNotZero(target) => {
                    if sp < 1 {
                        return None;
                    }
                    sp -= 1;
                    if stack[sp] != 0.0 {
                        pc = target;
                    } else {
                        pc += 1;
                    }
                }
                TableOp::LookupTypedTable {
                    table_id,
                    key_col_id,
                    val_col_id,
                } => {
                    if sp < 1 {
                        return None;
                    }
                    let target_val = stack[sp - 1];
                    let eval = evaluator?;
                    let table_name = &self.string_pool[table_id as usize];
                    let key_col = &self.string_pool[key_col_id as usize];
                    let val_col = &self.string_pool[val_col_id as usize];
                    let val = eval.with_typed_table(table_name, |tbl| {
                        tbl.lookup_value(key_col, target_val, val_col)
                    })??;
                    stack[sp - 1] = val;
                    pc += 1;
                }
                TableOp::LookupTypedTableRow {
                    table_id,
                    val_col_id,
                } => {
                    if sp < 1 {
                        return None;
                    }
                    let row_f = stack[sp - 1];
                    if row_f < 0.0 {
                        return None;
                    }
                    let row_idx = row_f as usize;
                    let eval = evaluator?;
                    let table_name = &self.string_pool[table_id as usize];
                    let val_col = &self.string_pool[val_col_id as usize];
                    let val = eval.with_typed_table(table_name, |tbl| {
                        tbl.get_by_name(row_idx, val_col)
                    })??;
                    stack[sp - 1] = val;
                    pc += 1;
                }
                TableOp::LookupTypedTableOr {
                    table_id,
                    key_col_id,
                    val_col_id,
                    default_val,
                } => {
                    if sp < 1 {
                        return None;
                    }
                    let target_val = stack[sp - 1];
                    let eval = evaluator?;
                    let table_name = &self.string_pool[table_id as usize];
                    let key_col = &self.string_pool[key_col_id as usize];
                    let val_col = &self.string_pool[val_col_id as usize];
                    let val = eval.with_typed_table(table_name, |tbl| {
                        tbl.lookup_value(key_col, target_val, val_col)
                    }).flatten().unwrap_or(default_val);
                    stack[sp - 1] = val;
                    pc += 1;
                }
                TableOp::LookupTypedTableRowOr {
                    table_id,
                    val_col_id,
                    default_val,
                } => {
                    if sp < 1 {
                        return None;
                    }
                    let row_f = stack[sp - 1];
                    let eval = evaluator?;
                    let table_name = &self.string_pool[table_id as usize];
                    let val_col = &self.string_pool[val_col_id as usize];
                    let val = if row_f >= 0.0 {
                        let row_idx = row_f as usize;
                        eval.with_typed_table(table_name, |tbl| {
                            tbl.get_by_name(row_idx, val_col)
                        }).flatten().unwrap_or(default_val)
                    } else {
                        default_val
                    };
                    stack[sp - 1] = val;
                    pc += 1;
                }
            }
        }

        if sp == 1 {
            Some(stack[0])
        } else {
            None
        }
    }

    /// Returns true if this bytecode accesses any cells at row index != current iteration
    #[inline]
    pub fn is_recurrent(&self) -> bool {
        self.ops.iter().any(|op| match op {
            TableOp::PushValueAt { iter_delta, .. } => *iter_delta != 0,
            _ => false,
        })
    }

    /// Checks if all column references in this bytecode are marked true in `computed`
    #[inline]
    pub fn are_dependencies_satisfied(&self, computed: &[bool]) -> bool {
        for op in &self.ops {
            match op {
                TableOp::PushCol(c) => {
                    if *c >= computed.len() || !computed[*c] {
                        return false;
                    }
                }
                TableOp::PushValueAt { col_idx, iter_delta } => {
                    if *iter_delta != 0 || *col_idx >= computed.len() || !computed[*col_idx] {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }

    /// Execute bytecode expression directly against a contiguous raw f64 tensor buffer
    ///
    /// # Safety
    /// `tensor_data` must point to at least `total_rows * col_count` contiguous f64 elements.
    /// `static_rows` must point to a valid `Vec<Value>` of previous rows.
    #[inline(always)]
    pub unsafe fn execute_tensor(
        &self,
        tensor_data: *const f64,
        col_count: usize,
        row_offset: usize,
        existing_row_count: usize,
        total_rows: usize,
        iteration_raw: i64,
        static_rows: *const Vec<Value>,
        col_names: &[String],
        evaluator: Option<&crate::rlogic::Evaluator>,
    ) -> Option<f64> {
        let mut stack: [f64; 64] = [0.0; 64];
        let mut sp: usize = 0;
        let mut pc: usize = 0;
        let mut steps: usize = 0;
        let num_ops = self.ops.len();

        while pc < num_ops {
            steps += 1;
            if steps > 512 {
                return None;
            }

            match self.ops[pc] {
                TableOp::PushConst(val) => {
                    if sp >= 64 {
                        return None;
                    }
                    stack[sp] = val;
                    sp += 1;
                    pc += 1;
                }
                TableOp::PushIteration => {
                    if sp >= 64 {
                        return None;
                    }
                    stack[sp] = iteration_raw as f64;
                    sp += 1;
                    pc += 1;
                }
                TableOp::PushCol(col_idx) => {
                    if sp >= 64 {
                        return None;
                    }
                    let val = *tensor_data.add(row_offset * col_count + col_idx);
                    stack[sp] = val;
                    sp += 1;
                    pc += 1;
                }
                TableOp::PushValueAt {
                    col_idx,
                    iter_delta,
                } => {
                    if sp >= 64 {
                        return None;
                    }
                    let target_iter = iteration_raw + iter_delta as i64;
                    let val = if target_iter >= 0 {
                        let target_row = target_iter as usize;
                        if target_row < existing_row_count {
                            let rows = &*static_rows;
                            if let Some(row) = rows.get(target_row) {
                                if let Value::Object(map) = row {
                                    let col_name = &col_names[col_idx];
                                    if let Some(cell) = map.get(col_name) {
                                        match cell {
                                            Value::Number(n) => n.as_f64().unwrap_or(0.0),
                                            _ => super::evaluator::helpers::to_f64(cell),
                                        }
                                    } else {
                                        0.0
                                    }
                                } else {
                                    0.0
                                }
                            } else {
                                0.0
                            }
                        } else if target_row < existing_row_count + total_rows {
                            let cell_offset = target_row - existing_row_count;
                            *tensor_data.add(cell_offset * col_count + col_idx)
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    stack[sp] = val;
                    sp += 1;
                    pc += 1;
                }
                TableOp::PushValueAtConstRow { col_idx, const_row } => {
                    if sp >= 64 {
                        return None;
                    }
                    let val = if const_row < existing_row_count {
                        let rows = &*static_rows;
                        if let Some(row) = rows.get(const_row) {
                            if let Value::Object(map) = row {
                                let col_name = &col_names[col_idx];
                                if let Some(cell) = map.get(col_name) {
                                    match cell {
                                        Value::Number(n) => n.as_f64().unwrap_or(0.0),
                                        _ => super::evaluator::helpers::to_f64(cell),
                                    }
                                } else {
                                    0.0
                                }
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        }
                    } else if const_row < existing_row_count + total_rows {
                        let cell_offset = const_row - existing_row_count;
                        *tensor_data.add(cell_offset * col_count + col_idx)
                    } else {
                        0.0
                    };
                    stack[sp] = val;
                    sp += 1;
                    pc += 1;
                }
                TableOp::Add => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] += stack[sp];
                    pc += 1;
                }
                TableOp::Subtract => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] -= stack[sp];
                    pc += 1;
                }
                TableOp::Multiply => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] *= stack[sp];
                    pc += 1;
                }
                TableOp::Divide => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    let divisor = stack[sp];
                    if divisor == 0.0 {
                        return None;
                    }
                    stack[sp - 1] /= divisor;
                    pc += 1;
                }
                TableOp::Modulo => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    let divisor = stack[sp];
                    if divisor == 0.0 {
                        return None;
                    }
                    stack[sp - 1] %= divisor;
                    pc += 1;
                }
                TableOp::Negate => {
                    if sp < 1 {
                        return None;
                    }
                    stack[sp - 1] = -stack[sp - 1];
                    pc += 1;
                }
                TableOp::Power => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] = stack[sp - 1].powf(stack[sp]);
                    pc += 1;
                }
                TableOp::Abs => {
                    if sp < 1 {
                        return None;
                    }
                    stack[sp - 1] = stack[sp - 1].abs();
                    pc += 1;
                }
                TableOp::Round(decimals) => {
                    if sp < 1 {
                        return None;
                    }
                    let val = stack[sp - 1];
                    let factor = 10f64.powi(decimals);
                    let rounded = if val >= 0.0 {
                        (val * factor + 0.5).floor() / factor
                    } else {
                        (val * factor - 0.5).ceil() / factor
                    };
                    stack[sp - 1] = rounded;
                    pc += 1;
                }
                TableOp::RoundUp(decimals) => {
                    if sp < 1 {
                        return None;
                    }
                    let val = stack[sp - 1];
                    let factor = 10f64.powi(decimals);
                    let rounded = if val >= 0.0 {
                        (val * factor).ceil() / factor
                    } else {
                        (val * factor).floor() / factor
                    };
                    stack[sp - 1] = rounded;
                    pc += 1;
                }
                TableOp::RoundDown(decimals) => {
                    if sp < 1 {
                        return None;
                    }
                    let val = stack[sp - 1];
                    let factor = 10f64.powi(decimals);
                    let rounded = if val >= 0.0 {
                        (val * factor).floor() / factor
                    } else {
                        (val * factor).ceil() / factor
                    };
                    stack[sp - 1] = rounded;
                    pc += 1;
                }
                TableOp::Min(count) => {
                    let n = count as usize;
                    if sp < n || n == 0 {
                        return None;
                    }
                    let start = sp - n;
                    let mut min_val = stack[start];
                    for i in (start + 1)..sp {
                        if stack[i] < min_val {
                            min_val = stack[i];
                        }
                    }
                    sp = start + 1;
                    stack[sp - 1] = min_val;
                    pc += 1;
                }
                TableOp::Max(count) => {
                    let n = count as usize;
                    if sp < n || n == 0 {
                        return None;
                    }
                    let start = sp - n;
                    let mut max_val = stack[start];
                    for i in (start + 1)..sp {
                        if stack[i] > max_val {
                            max_val = stack[i];
                        }
                    }
                    sp = start + 1;
                    stack[sp - 1] = max_val;
                    pc += 1;
                }
                TableOp::Eq => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    let b = stack[sp];
                    let a = stack[sp - 1];
                    stack[sp - 1] = if (a == b) || (a - b).abs() < 1e-9 {
                        1.0
                    } else {
                        0.0
                    };
                    pc += 1;
                }
                TableOp::Ne => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    let b = stack[sp];
                    let a = stack[sp - 1];
                    stack[sp - 1] = if (a != b) && (a - b).abs() >= 1e-9 {
                        1.0
                    } else {
                        0.0
                    };
                    pc += 1;
                }
                TableOp::Lt => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] = if stack[sp - 1] < stack[sp] { 1.0 } else { 0.0 };
                    pc += 1;
                }
                TableOp::Lte => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] = if stack[sp - 1] <= stack[sp] { 1.0 } else { 0.0 };
                    pc += 1;
                }
                TableOp::Gt => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] = if stack[sp - 1] > stack[sp] { 1.0 } else { 0.0 };
                    pc += 1;
                }
                TableOp::Gte => {
                    if sp < 2 {
                        return None;
                    }
                    sp -= 1;
                    stack[sp - 1] = if stack[sp - 1] >= stack[sp] { 1.0 } else { 0.0 };
                    pc += 1;
                }
                TableOp::Not => {
                    if sp < 1 {
                        return None;
                    }
                    stack[sp - 1] = if stack[sp - 1] == 0.0 { 1.0 } else { 0.0 };
                    pc += 1;
                }
                TableOp::Jump(target) => {
                    pc = target;
                }
                TableOp::JumpIfZero(target) => {
                    if sp < 1 {
                        return None;
                    }
                    sp -= 1;
                    if stack[sp] == 0.0 {
                        pc = target;
                    } else {
                        pc += 1;
                    }
                }
                TableOp::JumpIfNotZero(target) => {
                    if sp < 1 {
                        return None;
                    }
                    sp -= 1;
                    if stack[sp] != 0.0 {
                        pc = target;
                    } else {
                        pc += 1;
                    }
                }
                TableOp::LookupTypedTable {
                    table_id,
                    key_col_id,
                    val_col_id,
                } => {
                    if sp < 1 {
                        return None;
                    }
                    let target_val = stack[sp - 1];
                    let eval = evaluator?;
                    let table_name = &self.string_pool[table_id as usize];
                    let key_col = &self.string_pool[key_col_id as usize];
                    let val_col = &self.string_pool[val_col_id as usize];
                    let val = eval.with_typed_table(table_name, |tbl| {
                        tbl.lookup_value(key_col, target_val, val_col)
                    })??;
                    stack[sp - 1] = val;
                    pc += 1;
                }
                TableOp::LookupTypedTableRow {
                    table_id,
                    val_col_id,
                } => {
                    if sp < 1 {
                        return None;
                    }
                    let row_f = stack[sp - 1];
                    if row_f < 0.0 {
                        return None;
                    }
                    let row_idx = row_f as usize;
                    let eval = evaluator?;
                    let table_name = &self.string_pool[table_id as usize];
                    let val_col = &self.string_pool[val_col_id as usize];
                    let val = eval.with_typed_table(table_name, |tbl| {
                        tbl.get_by_name(row_idx, val_col)
                    })??;
                    stack[sp - 1] = val;
                    pc += 1;
                }
                TableOp::LookupTypedTableOr {
                    table_id,
                    key_col_id,
                    val_col_id,
                    default_val,
                } => {
                    if sp < 1 {
                        return None;
                    }
                    let target_val = stack[sp - 1];
                    let eval = evaluator?;
                    let table_name = &self.string_pool[table_id as usize];
                    let key_col = &self.string_pool[key_col_id as usize];
                    let val_col = &self.string_pool[val_col_id as usize];
                    let val = eval.with_typed_table(table_name, |tbl| {
                        tbl.lookup_value(key_col, target_val, val_col)
                    }).flatten().unwrap_or(default_val);
                    stack[sp - 1] = val;
                    pc += 1;
                }
                TableOp::LookupTypedTableRowOr {
                    table_id,
                    val_col_id,
                    default_val,
                } => {
                    if sp < 1 {
                        return None;
                    }
                    let row_f = stack[sp - 1];
                    let eval = evaluator?;
                    let table_name = &self.string_pool[table_id as usize];
                    let val_col = &self.string_pool[val_col_id as usize];
                    let val = if row_f >= 0.0 {
                        let row_idx = row_f as usize;
                        eval.with_typed_table(table_name, |tbl| {
                            tbl.get_by_name(row_idx, val_col)
                        }).flatten().unwrap_or(default_val)
                    } else {
                        default_val
                    };
                    stack[sp - 1] = val;
                    pc += 1;
                }
            }
        }

        if sp == 1 {
            Some(stack[0])
        } else {
            None
        }
    }
}

/// Attempt to lower a CompiledLogic expression to linear TableBytecode
pub fn try_lower_to_bytecode(
    logic: &CompiledLogic,
    table_path: &str,
    table_no_hash: &str,
    col_map: &rapidhash::RapidHashMap<String, usize>,
    evaluator: Option<&crate::rlogic::Evaluator>,
) -> Option<TableBytecode> {
    // If top-level expression is literally a string or empty, do not lower to numeric bytecode!
    match logic {
        CompiledLogic::String(_) | CompiledLogic::Empty => return None,
        _ => {}
    }

    let mut ctx = LowerCtx {
        table_path,
        table_no_hash,
        col_map,
        evaluator,
        ops: Vec::new(),
        strings: Vec::new(),
    };
    if ctx.lower(logic) {
        Some(TableBytecode {
            ops: ctx.ops,
            string_pool: ctx.strings,
        })
    } else {
        None
    }
}

struct LowerCtx<'a> {
    table_path: &'a str,
    table_no_hash: &'a str,
    col_map: &'a rapidhash::RapidHashMap<String, usize>,
    evaluator: Option<&'a crate::rlogic::Evaluator>,
    ops: Vec<TableOp>,
    strings: Vec<String>,
}

impl<'a> LowerCtx<'a> {
    fn intern_str(&mut self, s: &str) -> u16 {
        if let Some(pos) = self.strings.iter().position(|x| x == s) {
            pos as u16
        } else {
            let idx = self.strings.len() as u16;
            self.strings.push(s.to_string());
            idx
        }
    }

    fn lower(&mut self, logic: &CompiledLogic) -> bool {
        match logic {
            CompiledLogic::Number(n) => {
                self.ops.push(TableOp::PushConst(*n));
                true
            }
            CompiledLogic::Bool(b) => {
                self.ops.push(TableOp::PushConst(if *b { 1.0 } else { 0.0 }));
                true
            }
            CompiledLogic::Null => {
                self.ops.push(TableOp::PushConst(0.0));
                true
            }
            CompiledLogic::String(s) => {
                if let Ok(n) = s.parse::<f64>() {
                    self.ops.push(TableOp::PushConst(n));
                    true
                } else {
                    false
                }
            }
            CompiledLogic::Var(name, _) | CompiledLogic::Ref(name, _)
                if name == "$iteration" || name == "/$iteration" =>
            {
                self.ops.push(TableOp::PushIteration);
                true
            }
            CompiledLogic::Var(name, _) | CompiledLogic::Ref(name, _) => {
                let stripped = if name.starts_with("/$") {
                    &name[2..]
                } else if name.starts_with('$') {
                    &name[1..]
                } else {
                    name.as_str()
                };
                if !stripped.contains('/') {
                    if let Some(&col_idx) = self.col_map.get(stripped) {
                        self.ops.push(TableOp::PushCol(col_idx));
                        return true;
                    }
                }
                false
            }
            CompiledLogic::ValueAt(table, row_idx_expr, col_name_expr) => {
                let table_name = self.table_no_hash.rsplit('/').next().unwrap_or(self.table_no_hash);
                let is_self_table = match table.as_ref() {
                    CompiledLogic::Var(name, _) | CompiledLogic::Ref(name, _) => {
                        name == self.table_path
                            || name == self.table_no_hash
                            || name.strip_prefix('#').unwrap_or(name) == self.table_no_hash
                            || (!table_name.is_empty()
                                && (name == table_name
                                    || name.ends_with(&format!("/{}", table_name))
                                    || name.ends_with(&format!(".{}", table_name))))
                    }
                    _ => false,
                };
                if is_self_table {
                    let col_name = match col_name_expr.as_ref() {
                        Some(c) => match c.as_ref() {
                            CompiledLogic::String(s) => s.as_str(),
                            _ => return false,
                        },
                        None => return false,
                    };
                    let col_idx = match self.col_map.get(col_name) {
                        Some(&idx) => idx,
                        None => return false,
                    };

                    match row_idx_expr.as_ref() {
                        CompiledLogic::Var(var, _) | CompiledLogic::Ref(var, _)
                            if var == "$iteration" || var == "/$iteration" =>
                        {
                            self.ops.push(TableOp::PushValueAt {
                                col_idx,
                                iter_delta: 0,
                            });
                            true
                        }
                        CompiledLogic::Subtract(items) if items.len() == 2 => {
                            match (&items[0], &items[1]) {
                                (
                                    CompiledLogic::Var(var, _) | CompiledLogic::Ref(var, _),
                                    CompiledLogic::Number(n),
                                ) if var == "$iteration" || var == "/$iteration" => {
                                    self.ops.push(TableOp::PushValueAt {
                                        col_idx,
                                        iter_delta: -(*n as i32),
                                    });
                                    true
                                }
                                _ => false,
                            }
                        }
                        CompiledLogic::Add(items) if items.len() == 2 => match (&items[0], &items[1]) {
                            (
                                CompiledLogic::Var(var, _) | CompiledLogic::Ref(var, _),
                                CompiledLogic::Number(n),
                            )
                            | (
                                CompiledLogic::Number(n),
                                CompiledLogic::Var(var, _) | CompiledLogic::Ref(var, _),
                            ) if var == "$iteration" || var == "/$iteration" => {
                                self.ops.push(TableOp::PushValueAt {
                                    col_idx,
                                    iter_delta: *n as i32,
                                });
                                true
                            }
                            _ => false,
                        },
                        CompiledLogic::Number(n) if *n >= 0.0 => {
                            self.ops.push(TableOp::PushValueAtConstRow {
                                col_idx,
                                const_row: *n as usize,
                            });
                            true
                        }
                        _ => false,
                    }
                } else {
                    let ext_table_name = match table.as_ref() {
                        CompiledLogic::Var(name, _) | CompiledLogic::Ref(name, _) => name.as_str(),
                        CompiledLogic::String(s) => s.as_str(),
                        _ => return false,
                    };
                    let val_col = match col_name_expr.as_ref() {
                        Some(c) => match c.as_ref() {
                            CompiledLogic::String(s) => s.as_str(),
                            _ => return false,
                        },
                        None => return false,
                    };

                    if let CompiledLogic::IndexAt(lookup_expr, _, field_expr, range_expr) = row_idx_expr.as_ref() {
                        let is_range = match range_expr.as_ref() {
                            Some(r) => match r.as_ref() {
                                CompiledLogic::Bool(b) => *b,
                                CompiledLogic::Number(n) => *n != 0.0,
                                _ => return false,
                            },
                            None => false,
                        };
                        if is_range {
                            return false;
                        }
                        let key_col = match field_expr.as_ref() {
                            CompiledLogic::String(s) => s.as_str(),
                            _ => return false,
                        };

                        if let Some(eval) = self.evaluator {
                            let valid = eval.with_typed_table(ext_table_name, |tbl| {
                                tbl.key_columns.contains_key(key_col) && tbl.col_indices.contains_key(val_col)
                            }).unwrap_or(false);
                            if !valid {
                                return false;
                            }
                        }

                        if !self.lower(lookup_expr) {
                            return false;
                        }
                        let table_id = self.intern_str(ext_table_name);
                        let key_col_id = self.intern_str(key_col);
                        let val_col_id = self.intern_str(val_col);
                        self.ops.push(TableOp::LookupTypedTable {
                            table_id,
                            key_col_id,
                            val_col_id,
                        });
                        true
                    } else {
                        if let Some(eval) = self.evaluator {
                            let valid = eval.with_typed_table(ext_table_name, |tbl| {
                                tbl.col_indices.contains_key(val_col)
                            }).unwrap_or(false);
                            if !valid {
                                return false;
                            }
                        }

                        if !self.lower(row_idx_expr) {
                            return false;
                        }
                        let table_id = self.intern_str(ext_table_name);
                        let val_col_id = self.intern_str(val_col);
                        self.ops.push(TableOp::LookupTypedTableRow {
                            table_id,
                            val_col_id,
                        });
                        true
                    }
                }
            }
            CompiledLogic::Add(items) => {
                if items.is_empty() {
                    return false;
                }
                if !self.lower(&items[0]) {
                    return false;
                }
                for item in &items[1..] {
                    if !self.lower(item) {
                        return false;
                    }
                    self.ops.push(TableOp::Add);
                }
                true
            }
            CompiledLogic::Subtract(items) => {
                if items.is_empty() {
                    return false;
                }
                if !self.lower(&items[0]) {
                    return false;
                }
                if items.len() == 1 {
                    self.ops.push(TableOp::Negate);
                    return true;
                }
                for item in &items[1..] {
                    if !self.lower(item) {
                        return false;
                    }
                    self.ops.push(TableOp::Subtract);
                }
                true
            }
            CompiledLogic::Multiply(items) => {
                if items.is_empty() {
                    return false;
                }
                if !self.lower(&items[0]) {
                    return false;
                }
                for item in &items[1..] {
                    if !self.lower(item) {
                        return false;
                    }
                    self.ops.push(TableOp::Multiply);
                }
                true
            }
            CompiledLogic::Divide(items) => {
                if items.is_empty() {
                    return false;
                }
                if !self.lower(&items[0]) {
                    return false;
                }
                for item in &items[1..] {
                    if !self.lower(item) {
                        return false;
                    }
                    self.ops.push(TableOp::Divide);
                }
                true
            }
            CompiledLogic::Modulo(a, b) => {
                if !self.lower(a) || !self.lower(b) {
                    return false;
                }
                self.ops.push(TableOp::Modulo);
                true
            }
            CompiledLogic::Power(a, b) => {
                if !self.lower(a) || !self.lower(b) {
                    return false;
                }
                self.ops.push(TableOp::Power);
                true
            }
            CompiledLogic::Abs(a) => {
                if !self.lower(a) {
                    return false;
                }
                self.ops.push(TableOp::Abs);
                true
            }
            CompiledLogic::Round(a, decimals_expr) => {
                let decimals = match decimals_expr.as_ref() {
                    Some(d) => match d.as_ref() {
                        CompiledLogic::Number(n) => *n as i32,
                        _ => return false,
                    },
                    None => 0,
                };
                if !self.lower(a) {
                    return false;
                }
                self.ops.push(TableOp::Round(decimals));
                true
            }
            CompiledLogic::RoundUp(a, decimals_expr) => {
                let decimals = match decimals_expr.as_ref() {
                    Some(d) => match d.as_ref() {
                        CompiledLogic::Number(n) => *n as i32,
                        _ => return false,
                    },
                    None => 0,
                };
                if !self.lower(a) {
                    return false;
                }
                self.ops.push(TableOp::RoundUp(decimals));
                true
            }
            CompiledLogic::RoundDown(a, decimals_expr) => {
                let decimals = match decimals_expr.as_ref() {
                    Some(d) => match d.as_ref() {
                        CompiledLogic::Number(n) => *n as i32,
                        _ => return false,
                    },
                    None => 0,
                };
                if !self.lower(a) {
                    return false;
                }
                self.ops.push(TableOp::RoundDown(decimals));
                true
            }
            CompiledLogic::Min(items) => {
                if items.is_empty() || items.len() > 32 {
                    return false;
                }
                for item in items {
                    if !self.lower(item) {
                        return false;
                    }
                }
                self.ops.push(TableOp::Min(items.len() as u8));
                true
            }
            CompiledLogic::Max(items) => {
                if items.is_empty() || items.len() > 32 {
                    return false;
                }
                for item in items {
                    if !self.lower(item) {
                        return false;
                    }
                }
                self.ops.push(TableOp::Max(items.len() as u8));
                true
            }
            CompiledLogic::Equal(a, b) | CompiledLogic::StrictEqual(a, b) => {
                if !self.lower(a) || !self.lower(b) {
                    return false;
                }
                self.ops.push(TableOp::Eq);
                true
            }
            CompiledLogic::NotEqual(a, b) | CompiledLogic::StrictNotEqual(a, b) => {
                if !self.lower(a) || !self.lower(b) {
                    return false;
                }
                self.ops.push(TableOp::Ne);
                true
            }
            CompiledLogic::LessThan(a, b) => {
                if !self.lower(a) || !self.lower(b) {
                    return false;
                }
                self.ops.push(TableOp::Lt);
                true
            }
            CompiledLogic::LessThanOrEqual(a, b) => {
                if !self.lower(a) || !self.lower(b) {
                    return false;
                }
                self.ops.push(TableOp::Lte);
                true
            }
            CompiledLogic::GreaterThan(a, b) => {
                if !self.lower(a) || !self.lower(b) {
                    return false;
                }
                self.ops.push(TableOp::Gt);
                true
            }
            CompiledLogic::GreaterThanOrEqual(a, b) => {
                if !self.lower(a) || !self.lower(b) {
                    return false;
                }
                self.ops.push(TableOp::Gte);
                true
            }
            CompiledLogic::Not(inner) => {
                if !self.lower(inner) {
                    return false;
                }
                self.ops.push(TableOp::Not);
                true
            }
            CompiledLogic::IfNull(cond_expr, alt_expr) => {
                let default_val = match alt_expr.as_ref() {
                    CompiledLogic::Number(n) => *n,
                    CompiledLogic::String(s) => s.parse::<f64>().unwrap_or(0.0),
                    _ => 0.0,
                };
                if let CompiledLogic::ValueAt(table, row_idx_expr, col_name_expr) = cond_expr.as_ref() {
                    let ext_table_name = match table.as_ref() {
                        CompiledLogic::Var(name, _) | CompiledLogic::Ref(name, _) => name.as_str(),
                        CompiledLogic::String(s) => s.as_str(),
                        _ => return self.lower(cond_expr),
                    };
                    let val_col = match col_name_expr.as_ref() {
                        Some(c) => match c.as_ref() {
                            CompiledLogic::String(s) => s.as_str(),
                            _ => return self.lower(cond_expr),
                        },
                        None => return self.lower(cond_expr),
                    };

                    if let CompiledLogic::IndexAt(lookup_expr, _, field_expr, range_expr) = row_idx_expr.as_ref() {
                        let is_range = match range_expr.as_ref() {
                            Some(r) => match r.as_ref() {
                                CompiledLogic::Bool(b) => *b,
                                CompiledLogic::Number(n) => *n != 0.0,
                                _ => return false,
                            },
                            None => false,
                        };
                        if is_range {
                            return false;
                        }
                        let key_col = match field_expr.as_ref() {
                            CompiledLogic::String(s) => s.as_str(),
                            _ => return false,
                        };

                        if let Some(eval) = self.evaluator {
                            let valid = eval.with_typed_table(ext_table_name, |tbl| {
                                tbl.key_columns.contains_key(key_col) && tbl.col_indices.contains_key(val_col)
                            }).unwrap_or(false);
                            if !valid {
                                return false;
                            }
                        }

                        if !self.lower(lookup_expr) {
                            return false;
                        }
                        let table_id = self.intern_str(ext_table_name);
                        let key_col_id = self.intern_str(key_col);
                        let val_col_id = self.intern_str(val_col);
                        self.ops.push(TableOp::LookupTypedTableOr {
                            table_id,
                            key_col_id,
                            val_col_id,
                            default_val,
                        });
                        true
                    } else {
                        if let Some(eval) = self.evaluator {
                            let valid = eval.with_typed_table(ext_table_name, |tbl| {
                                tbl.col_indices.contains_key(val_col)
                            }).unwrap_or(false);
                            if !valid {
                                return false;
                            }
                        }

                        if !self.lower(row_idx_expr) {
                            return false;
                        }
                        let table_id = self.intern_str(ext_table_name);
                        let val_col_id = self.intern_str(val_col);
                        self.ops.push(TableOp::LookupTypedTableRowOr {
                            table_id,
                            val_col_id,
                            default_val,
                        });
                        true
                    }
                } else {
                    self.lower(cond_expr)
                }
            }
            CompiledLogic::If(cond, then, else_expr) => {
                if matches!(then.as_ref(), CompiledLogic::String(_) | CompiledLogic::Empty)
                    || matches!(else_expr.as_ref(), CompiledLogic::String(_) | CompiledLogic::Empty)
                {
                    return false;
                }
                if !self.lower(cond) {
                    return false;
                }
                let jz_idx = self.ops.len();
                self.ops.push(TableOp::JumpIfZero(0));
                if !self.lower(then) {
                    return false;
                }
                let jmp_idx = self.ops.len();
                self.ops.push(TableOp::Jump(0));
                self.ops[jz_idx] = TableOp::JumpIfZero(self.ops.len());
                if !self.lower(else_expr) {
                    return false;
                }
                self.ops[jmp_idx] = TableOp::Jump(self.ops.len());
                true
            }
            CompiledLogic::And(items) => {
                if items.is_empty() {
                    self.ops.push(TableOp::PushConst(1.0));
                    return true;
                }
                if items.len() == 1 {
                    return self.lower(&items[0]);
                }
                let mut jz_indices = Vec::new();
                for item in items {
                    if !self.lower(item) {
                        return false;
                    }
                    jz_indices.push(self.ops.len());
                    self.ops.push(TableOp::JumpIfZero(0));
                }
                self.ops.push(TableOp::PushConst(1.0));
                let jmp_end = self.ops.len();
                self.ops.push(TableOp::Jump(0));
                let false_target = self.ops.len();
                for jz in jz_indices {
                    self.ops[jz] = TableOp::JumpIfZero(false_target);
                }
                self.ops.push(TableOp::PushConst(0.0));
                self.ops[jmp_end] = TableOp::Jump(self.ops.len());
                true
            }
            CompiledLogic::Or(items) => {
                if items.is_empty() {
                    self.ops.push(TableOp::PushConst(0.0));
                    return true;
                }
                if items.len() == 1 {
                    return self.lower(&items[0]);
                }
                let mut jnz_indices = Vec::new();
                for item in items {
                    if !self.lower(item) {
                        return false;
                    }
                    jnz_indices.push(self.ops.len());
                    self.ops.push(TableOp::JumpIfNotZero(0));
                }
                self.ops.push(TableOp::PushConst(0.0));
                let jmp_end = self.ops.len();
                self.ops.push(TableOp::Jump(0));
                let true_target = self.ops.len();
                for jnz in jnz_indices {
                    self.ops[jnz] = TableOp::JumpIfNotZero(true_target);
                }
                self.ops.push(TableOp::PushConst(1.0));
                self.ops[jmp_end] = TableOp::Jump(self.ops.len());
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytecode_arithmetic() {
        let bc = TableBytecode {
            ops: vec![
                TableOp::PushConst(10.0),
                TableOp::PushConst(5.0),
                TableOp::Add,
                TableOp::PushConst(3.0),
                TableOp::Multiply,
            ],
            string_pool: vec![],
        };
        let dummy_cells = vec![];
        let dummy_rows = vec![];
        let col_names = vec![];
        let res = unsafe {
            bc.execute(
                dummy_cells.as_ptr(),
                0,
                0,
                0,
                0,
                1,
                &dummy_rows as *const Vec<Value>,
                &col_names,
                None,
            )
        };
        assert_eq!(res, Some(45.0)); // (10 + 5) * 3 = 45
    }

    #[test]
    fn test_bytecode_conditional() {
        // if 10 > 5 then 100 else 200
        let bc = TableBytecode {
            ops: vec![
                TableOp::PushConst(10.0),
                TableOp::PushConst(5.0),
                TableOp::Gt,
                TableOp::JumpIfZero(6),
                TableOp::PushConst(100.0),
                TableOp::Jump(7),
                TableOp::PushConst(200.0),
            ],
            string_pool: vec![],
        };
        let dummy_cells = vec![];
        let dummy_rows = vec![];
        let col_names = vec![];
        let res = unsafe {
            bc.execute(
                dummy_cells.as_ptr(),
                0,
                0,
                0,
                0,
                1,
                &dummy_rows as *const Vec<Value>,
                &col_names,
                None,
            )
        };
        assert_eq!(res, Some(100.0));
    }

    #[test]
    fn test_bytecode_push_cell_and_value_at() {
        let cells = vec![
            Value::from(10.0),
            Value::from(20.0),
            Value::from(30.0),
            Value::from(40.0),
        ];
        let static_rows = vec![];
        let col_names = vec!["A".to_string(), "B".to_string()];

        // row 1: PushCol(0) [30.0] + PushValueAt(1, -1) [20.0] = 50.0
        let bc = TableBytecode {
            ops: vec![
                TableOp::PushCol(0),
                TableOp::PushValueAt {
                    col_idx: 1,
                    iter_delta: -1,
                },
                TableOp::Add,
            ],
            string_pool: vec![],
        };
        let res = unsafe {
            bc.execute(
                cells.as_ptr(),
                2,
                1, // row_offset = 1
                0, // existing_row_count = 0
                2, // total_rows = 2
                1, // iteration = 1
                &static_rows as *const Vec<Value>,
                &col_names,
                None,
            )
        };
        assert_eq!(res, Some(50.0));
    }
}
