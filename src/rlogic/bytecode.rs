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
}

/// Compiled linear bytecode for high-speed inner table cell evaluation
#[derive(Debug, Clone)]
pub struct TableBytecode {
    pub ops: Vec<TableOp>,
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
) -> Option<TableBytecode> {
    let mut ops = Vec::new();
    if lower_node(logic, &mut ops, table_path, table_no_hash, col_map) {
        Some(TableBytecode { ops })
    } else {
        None
    }
}

fn lower_node(
    logic: &CompiledLogic,
    ops: &mut Vec<TableOp>,
    table_path: &str,
    table_no_hash: &str,
    col_map: &rapidhash::RapidHashMap<String, usize>,
) -> bool {
    match logic {
        CompiledLogic::Number(n) => {
            ops.push(TableOp::PushConst(*n));
            true
        }
        CompiledLogic::Bool(b) => {
            ops.push(TableOp::PushConst(if *b { 1.0 } else { 0.0 }));
            true
        }
        CompiledLogic::Null => {
            ops.push(TableOp::PushConst(0.0));
            true
        }
        CompiledLogic::String(s) => {
            if let Ok(n) = s.parse::<f64>() {
                ops.push(TableOp::PushConst(n));
                true
            } else {
                false
            }
        }
        CompiledLogic::Var(name, _) | CompiledLogic::Ref(name, _)
            if name == "$iteration" || name == "/$iteration" =>
        {
            ops.push(TableOp::PushIteration);
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
                if let Some(&col_idx) = col_map.get(stripped) {
                    ops.push(TableOp::PushCol(col_idx));
                    return true;
                }
            }
            false
        }
        CompiledLogic::ValueAt(table, row_idx_expr, col_name_expr) => {
            let table_name = table_no_hash.rsplit('/').next().unwrap_or(table_no_hash);
            let is_self_table = match table.as_ref() {
                CompiledLogic::Var(name, _) | CompiledLogic::Ref(name, _) => {
                    name == table_path
                        || name == table_no_hash
                        || name.strip_prefix('#').unwrap_or(name) == table_no_hash
                        || (!table_name.is_empty()
                            && (name == table_name
                                || name.ends_with(&format!("/{}", table_name))
                                || name.ends_with(&format!(".{}", table_name))))
                }
                _ => false,
            };
            if !is_self_table {
                return false;
            }

            let col_name = match col_name_expr.as_ref() {
                Some(c) => match c.as_ref() {
                    CompiledLogic::String(s) => s.as_str(),
                    _ => return false,
                },
                None => return false,
            };
            let col_idx = match col_map.get(col_name) {
                Some(&idx) => idx,
                None => return false,
            };

            match row_idx_expr.as_ref() {
                CompiledLogic::Var(var, _) | CompiledLogic::Ref(var, _)
                    if var == "$iteration" || var == "/$iteration" =>
                {
                    ops.push(TableOp::PushValueAt {
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
                            ops.push(TableOp::PushValueAt {
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
                        ops.push(TableOp::PushValueAt {
                            col_idx,
                            iter_delta: *n as i32,
                        });
                        true
                    }
                    _ => false,
                },
                CompiledLogic::Number(n) if *n >= 0.0 => {
                    ops.push(TableOp::PushValueAtConstRow {
                        col_idx,
                        const_row: *n as usize,
                    });
                    true
                }
                _ => false,
            }
        }
        CompiledLogic::Add(items) => {
            if items.is_empty() {
                return false;
            }
            if !lower_node(&items[0], ops, table_path, table_no_hash, col_map) {
                return false;
            }
            for item in &items[1..] {
                if !lower_node(item, ops, table_path, table_no_hash, col_map) {
                    return false;
                }
                ops.push(TableOp::Add);
            }
            true
        }
        CompiledLogic::Subtract(items) => {
            if items.is_empty() {
                return false;
            }
            if !lower_node(&items[0], ops, table_path, table_no_hash, col_map) {
                return false;
            }
            if items.len() == 1 {
                ops.push(TableOp::Negate);
                return true;
            }
            for item in &items[1..] {
                if !lower_node(item, ops, table_path, table_no_hash, col_map) {
                    return false;
                }
                ops.push(TableOp::Subtract);
            }
            true
        }
        CompiledLogic::Multiply(items) => {
            if items.is_empty() {
                return false;
            }
            if !lower_node(&items[0], ops, table_path, table_no_hash, col_map) {
                return false;
            }
            for item in &items[1..] {
                if !lower_node(item, ops, table_path, table_no_hash, col_map) {
                    return false;
                }
                ops.push(TableOp::Multiply);
            }
            true
        }
        CompiledLogic::Divide(items) => {
            if items.is_empty() {
                return false;
            }
            if !lower_node(&items[0], ops, table_path, table_no_hash, col_map) {
                return false;
            }
            for item in &items[1..] {
                if !lower_node(item, ops, table_path, table_no_hash, col_map) {
                    return false;
                }
                ops.push(TableOp::Divide);
            }
            true
        }
        CompiledLogic::Modulo(a, b) => {
            if !lower_node(a, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            if !lower_node(b, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            ops.push(TableOp::Modulo);
            true
        }
        CompiledLogic::Power(a, b) => {
            if !lower_node(a, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            if !lower_node(b, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            ops.push(TableOp::Power);
            true
        }
        CompiledLogic::Abs(a) => {
            if !lower_node(a, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            ops.push(TableOp::Abs);
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
            if !lower_node(a, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            ops.push(TableOp::Round(decimals));
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
            if !lower_node(a, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            ops.push(TableOp::RoundUp(decimals));
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
            if !lower_node(a, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            ops.push(TableOp::RoundDown(decimals));
            true
        }
        CompiledLogic::Min(items) => {
            if items.is_empty() || items.len() > 32 {
                return false;
            }
            for item in items {
                if !lower_node(item, ops, table_path, table_no_hash, col_map) {
                    return false;
                }
            }
            ops.push(TableOp::Min(items.len() as u8));
            true
        }
        CompiledLogic::Max(items) => {
            if items.is_empty() || items.len() > 32 {
                return false;
            }
            for item in items {
                if !lower_node(item, ops, table_path, table_no_hash, col_map) {
                    return false;
                }
            }
            ops.push(TableOp::Max(items.len() as u8));
            true
        }
        CompiledLogic::Equal(a, b) | CompiledLogic::StrictEqual(a, b) => {
            if !lower_node(a, ops, table_path, table_no_hash, col_map)
                || !lower_node(b, ops, table_path, table_no_hash, col_map)
            {
                return false;
            }
            ops.push(TableOp::Eq);
            true
        }
        CompiledLogic::NotEqual(a, b) | CompiledLogic::StrictNotEqual(a, b) => {
            if !lower_node(a, ops, table_path, table_no_hash, col_map)
                || !lower_node(b, ops, table_path, table_no_hash, col_map)
            {
                return false;
            }
            ops.push(TableOp::Ne);
            true
        }
        CompiledLogic::LessThan(a, b) => {
            if !lower_node(a, ops, table_path, table_no_hash, col_map)
                || !lower_node(b, ops, table_path, table_no_hash, col_map)
            {
                return false;
            }
            ops.push(TableOp::Lt);
            true
        }
        CompiledLogic::LessThanOrEqual(a, b) => {
            if !lower_node(a, ops, table_path, table_no_hash, col_map)
                || !lower_node(b, ops, table_path, table_no_hash, col_map)
            {
                return false;
            }
            ops.push(TableOp::Lte);
            true
        }
        CompiledLogic::GreaterThan(a, b) => {
            if !lower_node(a, ops, table_path, table_no_hash, col_map)
                || !lower_node(b, ops, table_path, table_no_hash, col_map)
            {
                return false;
            }
            ops.push(TableOp::Gt);
            true
        }
        CompiledLogic::GreaterThanOrEqual(a, b) => {
            if !lower_node(a, ops, table_path, table_no_hash, col_map)
                || !lower_node(b, ops, table_path, table_no_hash, col_map)
            {
                return false;
            }
            ops.push(TableOp::Gte);
            true
        }
        CompiledLogic::Not(inner) => {
            if !lower_node(inner, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            ops.push(TableOp::Not);
            true
        }
        CompiledLogic::If(cond, then, else_expr) => {
            if !lower_node(cond, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            let jz_idx = ops.len();
            ops.push(TableOp::JumpIfZero(0));
            if !lower_node(then, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            let jmp_idx = ops.len();
            ops.push(TableOp::Jump(0));
            ops[jz_idx] = TableOp::JumpIfZero(ops.len());
            if !lower_node(else_expr, ops, table_path, table_no_hash, col_map) {
                return false;
            }
            ops[jmp_idx] = TableOp::Jump(ops.len());
            true
        }
        CompiledLogic::And(items) => {
            if items.is_empty() {
                ops.push(TableOp::PushConst(1.0));
                return true;
            }
            if items.len() == 1 {
                return lower_node(&items[0], ops, table_path, table_no_hash, col_map);
            }
            let mut jz_indices = Vec::new();
            for item in items {
                if !lower_node(item, ops, table_path, table_no_hash, col_map) {
                    return false;
                }
                jz_indices.push(ops.len());
                ops.push(TableOp::JumpIfZero(0));
            }
            ops.push(TableOp::PushConst(1.0));
            let jmp_end = ops.len();
            ops.push(TableOp::Jump(0));
            let false_target = ops.len();
            for jz in jz_indices {
                ops[jz] = TableOp::JumpIfZero(false_target);
            }
            ops.push(TableOp::PushConst(0.0));
            ops[jmp_end] = TableOp::Jump(ops.len());
            true
        }
        CompiledLogic::Or(items) => {
            if items.is_empty() {
                ops.push(TableOp::PushConst(0.0));
                return true;
            }
            if items.len() == 1 {
                return lower_node(&items[0], ops, table_path, table_no_hash, col_map);
            }
            let mut jnz_indices = Vec::new();
            for item in items {
                if !lower_node(item, ops, table_path, table_no_hash, col_map) {
                    return false;
                }
                jnz_indices.push(ops.len());
                ops.push(TableOp::JumpIfNotZero(0));
            }
            ops.push(TableOp::PushConst(0.0));
            let jmp_end = ops.len();
            ops.push(TableOp::Jump(0));
            let true_target = ops.len();
            for jnz in jnz_indices {
                ops[jnz] = TableOp::JumpIfNotZero(true_target);
            }
            ops.push(TableOp::PushConst(1.0));
            ops[jmp_end] = TableOp::Jump(ops.len());
            true
        }
        _ => false,
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
            )
        };
        assert_eq!(res, Some(50.0));
    }
}
