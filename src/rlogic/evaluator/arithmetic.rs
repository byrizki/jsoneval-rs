use super::super::compiled::CompiledLogic;
use super::helpers;
use super::Evaluator;
use serde_json::Value;

impl Evaluator {
    /// Fast unboxed evaluation of scalar expressions into f64 registers.
    /// Avoids serde_json::Value allocation and cloning on inner AST nodes.
    #[inline(always)]
    pub(super) fn eval_f64(
        &self,
        logic: &CompiledLogic,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Option<f64>, String> {
        if depth > self.config.recursion_limit {
            return Err(format!("Max evaluation depth exceeded: {}", depth));
        }

        match logic {
            CompiledLogic::Number(n) => Ok(Some(*n)),
            CompiledLogic::Var(name, default) => {
                unsafe {
                    if let Some(ts) = (*self.table_scope.get()).as_ref() {
                        if !name.is_empty()
                            && (*name == ts.path
                                || name.trim_start_matches('#') == ts.path_no_hash.as_str())
                        {
                            return Ok(None);
                        }
                        if name == "/$iteration" || name == "$iteration" {
                            if let Some(raw) = ts.iteration_raw {
                                return Ok(Some(raw as f64));
                            }
                        }
                        if !ts.current_row_base.is_null() {
                            let col_ref = if name.starts_with("/$") && !name[2..].contains('/') {
                                Some(&name[2..])
                            } else if name.starts_with('$')
                                && !name.starts_with("/$")
                                && !name[1..].contains('/')
                            {
                                Some(&name[1..])
                            } else {
                                None
                            };
                            if let Some(field) = col_ref {
                                if let Some(col_idx) = ts.get_col_idx(field) {
                                    let cell = &*ts.current_row_base.add(col_idx);
                                    if let Value::Number(n) = cell {
                                        if let Some(f) = n.as_f64() {
                                            return Ok(Some(f));
                                        }
                                    }
                                    return Ok(Some(helpers::to_f64(cell)));
                                }
                            }
                        }
                    }
                }
                let v = if name.is_empty() {
                    self.get_var(user_data, name)
                } else {
                    self.get_var(internal_context, name)
                        .or_else(|| self.get_var(user_data, name))
                };
                match v {
                    Some(val) if !val.is_null() => match val {
                        Value::Number(n) => Ok(n.as_f64()),
                        Value::String(s) => Ok(s.parse::<f64>().ok()),
                        Value::Bool(b) => Ok(Some(if *b { 1.0 } else { 0.0 })),
                        _ => Ok(None),
                    },
                    _ => {
                        if let Some(def) = default {
                            self.eval_f64(def, user_data, internal_context, depth + 1)
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            CompiledLogic::Ref(path, default) => {
                unsafe {
                    if let Some(ts) = (*self.table_scope.get()).as_ref() {
                        if !path.is_empty()
                            && (*path == ts.path
                                || path.trim_start_matches('#') == ts.path_no_hash.as_str())
                        {
                            return Ok(None);
                        }
                        if path == "/$iteration" || path == "$iteration" {
                            if let Some(raw) = ts.iteration_raw {
                                return Ok(Some(raw as f64));
                            }
                        }
                        if !ts.current_row_base.is_null() {
                            let col_ref = if path.starts_with("/$") && !path[2..].contains('/') {
                                Some(&path[2..])
                            } else if path.starts_with('$')
                                && !path.starts_with("/$")
                                && !path[1..].contains('/')
                            {
                                Some(&path[1..])
                            } else {
                                None
                            };
                            if let Some(field) = col_ref {
                                if let Some(col_idx) = ts.get_col_idx(field) {
                                    let cell = &*ts.current_row_base.add(col_idx);
                                    if let Value::Number(n) = cell {
                                        if let Some(f) = n.as_f64() {
                                            return Ok(Some(f));
                                        }
                                    }
                                    return Ok(Some(helpers::to_f64(cell)));
                                }
                            }
                        }
                    }
                }
                let v = if path.is_empty() {
                    self.get_var(user_data, path)
                } else {
                    self.get_var(internal_context, path)
                        .or_else(|| self.get_var(user_data, path))
                };
                match v {
                    Some(val) if !val.is_null() => match val {
                        Value::Number(n) => Ok(n.as_f64()),
                        Value::String(s) => Ok(s.parse::<f64>().ok()),
                        Value::Bool(b) => Ok(Some(if *b { 1.0 } else { 0.0 })),
                        _ => Ok(None),
                    },
                    _ => {
                        if let Some(def) = default {
                            self.eval_f64(def, user_data, internal_context, depth + 1)
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            CompiledLogic::Add(items) => {
                let mut acc = 0.0;
                for item in items {
                    let val = match self.eval_f64(item, user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => {
                            let v = self.evaluate_with_context(
                                item,
                                user_data,
                                internal_context,
                                depth + 1,
                            )?;
                            helpers::to_f64(&v)
                        }
                    };
                    acc += val;
                }
                Ok(Some(acc))
            }
            CompiledLogic::Subtract(items) => {
                if items.is_empty() {
                    return Ok(Some(0.0));
                }
                let first =
                    match self.eval_f64(&items[0], user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => {
                            let v = self.evaluate_with_context(
                                &items[0],
                                user_data,
                                internal_context,
                                depth + 1,
                            )?;
                            helpers::to_f64(&v)
                        }
                    };
                if items.len() == 1 {
                    return Ok(Some(-first));
                }
                let mut acc = first;
                for item in &items[1..] {
                    let val = match self.eval_f64(item, user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => {
                            let v = self.evaluate_with_context(
                                item,
                                user_data,
                                internal_context,
                                depth + 1,
                            )?;
                            helpers::to_f64(&v)
                        }
                    };
                    acc -= val;
                }
                Ok(Some(acc))
            }
            CompiledLogic::Multiply(items) => {
                if items.is_empty() {
                    return Ok(Some(0.0));
                }
                let mut acc = 1.0;
                for item in items {
                    let val = match self.eval_f64(item, user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => {
                            let v = self.evaluate_with_context(
                                item,
                                user_data,
                                internal_context,
                                depth + 1,
                            )?;
                            helpers::to_f64(&v)
                        }
                    };
                    acc *= val;
                }
                Ok(Some(acc))
            }
            CompiledLogic::Divide(items) => {
                if items.is_empty() {
                    return Ok(Some(0.0));
                }
                let first =
                    match self.eval_f64(&items[0], user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => {
                            let v = self.evaluate_with_context(
                                &items[0],
                                user_data,
                                internal_context,
                                depth + 1,
                            )?;
                            helpers::to_f64(&v)
                        }
                    };
                let mut acc = first;
                for item in &items[1..] {
                    let divisor =
                        match self.eval_f64(item, user_data, internal_context, depth + 1)? {
                            Some(n) => n,
                            None => {
                                let v = self.evaluate_with_context(
                                    item,
                                    user_data,
                                    internal_context,
                                    depth + 1,
                                )?;
                                helpers::to_f64(&v)
                            }
                        };
                    if divisor == 0.0 {
                        return Ok(None);
                    }
                    acc /= divisor;
                }
                Ok(Some(acc))
            }
            CompiledLogic::Modulo(a, b) => {
                let num_a = match self.eval_f64(a, user_data, internal_context, depth + 1)? {
                    Some(n) => n,
                    None => helpers::to_f64(&self.evaluate_with_context(
                        a,
                        user_data,
                        internal_context,
                        depth + 1,
                    )?),
                };
                let num_b = match self.eval_f64(b, user_data, internal_context, depth + 1)? {
                    Some(n) => n,
                    None => helpers::to_f64(&self.evaluate_with_context(
                        b,
                        user_data,
                        internal_context,
                        depth + 1,
                    )?),
                };
                if num_b == 0.0 {
                    Ok(None)
                } else {
                    Ok(Some(num_a % num_b))
                }
            }
            CompiledLogic::Power(a, b) => {
                let base = match self.eval_f64(a, user_data, internal_context, depth + 1)? {
                    Some(n) => n,
                    None => helpers::to_f64(&self.evaluate_with_context(
                        a,
                        user_data,
                        internal_context,
                        depth + 1,
                    )?),
                };
                let exp = match self.eval_f64(b, user_data, internal_context, depth + 1)? {
                    Some(n) => n,
                    None => helpers::to_f64(&self.evaluate_with_context(
                        b,
                        user_data,
                        internal_context,
                        depth + 1,
                    )?),
                };
                Ok(Some(base.powf(exp)))
            }
            CompiledLogic::Round(expr, decimals_expr) => {
                let num = match self.eval_f64(expr, user_data, internal_context, depth + 1)? {
                    Some(n) => n,
                    None => helpers::to_f64(&self.evaluate_with_context(
                        expr,
                        user_data,
                        internal_context,
                        depth + 1,
                    )?),
                };
                let decimals = if let Some(dec) = decimals_expr {
                    let d = match self.eval_f64(dec, user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => helpers::to_f64(&self.evaluate_with_context(
                            dec,
                            user_data,
                            internal_context,
                            depth + 1,
                        )?),
                    };
                    d as i32
                } else {
                    0
                };
                let res = if decimals == 0 {
                    num.round()
                } else if decimals > 0 {
                    let mult = 10f64.powi(decimals);
                    (num * mult).round() / mult
                } else {
                    let div = 10f64.powi(-decimals);
                    (num / div).round() * div
                };
                Ok(Some(res))
            }
            CompiledLogic::RoundUp(expr, decimals_expr) => {
                let num = match self.eval_f64(expr, user_data, internal_context, depth + 1)? {
                    Some(n) => n,
                    None => helpers::to_f64(&self.evaluate_with_context(
                        expr,
                        user_data,
                        internal_context,
                        depth + 1,
                    )?),
                };
                let decimals = if let Some(dec) = decimals_expr {
                    let d = match self.eval_f64(dec, user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => helpers::to_f64(&self.evaluate_with_context(
                            dec,
                            user_data,
                            internal_context,
                            depth + 1,
                        )?),
                    };
                    d as i32
                } else {
                    0
                };
                let res = if decimals == 0 {
                    num.ceil()
                } else if decimals > 0 {
                    let mult = 10f64.powi(decimals);
                    (num * mult).ceil() / mult
                } else {
                    let div = 10f64.powi(-decimals);
                    (num / div).ceil() * div
                };
                Ok(Some(res))
            }
            CompiledLogic::RoundDown(expr, decimals_expr) => {
                let num = match self.eval_f64(expr, user_data, internal_context, depth + 1)? {
                    Some(n) => n,
                    None => helpers::to_f64(&self.evaluate_with_context(
                        expr,
                        user_data,
                        internal_context,
                        depth + 1,
                    )?),
                };
                let decimals = if let Some(dec) = decimals_expr {
                    let d = match self.eval_f64(dec, user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => helpers::to_f64(&self.evaluate_with_context(
                            dec,
                            user_data,
                            internal_context,
                            depth + 1,
                        )?),
                    };
                    d as i32
                } else {
                    0
                };
                let res = if decimals == 0 {
                    num.floor()
                } else if decimals > 0 {
                    let mult = 10f64.powi(decimals);
                    (num * mult).floor() / mult
                } else {
                    let div = 10f64.powi(-decimals);
                    (num / div).floor() * div
                };
                Ok(Some(res))
            }
            CompiledLogic::Abs(expr) => {
                let num = match self.eval_f64(expr, user_data, internal_context, depth + 1)? {
                    Some(n) => n,
                    None => helpers::to_f64(&self.evaluate_with_context(
                        expr,
                        user_data,
                        internal_context,
                        depth + 1,
                    )?),
                };
                Ok(Some(num.abs()))
            }
            CompiledLogic::Min(items) => {
                if items.is_empty() {
                    return Ok(None);
                }
                let mut min_val =
                    match self.eval_f64(&items[0], user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => helpers::to_f64(&self.evaluate_with_context(
                            &items[0],
                            user_data,
                            internal_context,
                            depth + 1,
                        )?),
                    };
                for item in &items[1..] {
                    let num = match self.eval_f64(item, user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => helpers::to_f64(&self.evaluate_with_context(
                            item,
                            user_data,
                            internal_context,
                            depth + 1,
                        )?),
                    };
                    if num < min_val {
                        min_val = num;
                    }
                }
                Ok(Some(min_val))
            }
            CompiledLogic::Max(items) => {
                if items.is_empty() {
                    return Ok(None);
                }
                let mut max_val =
                    match self.eval_f64(&items[0], user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => helpers::to_f64(&self.evaluate_with_context(
                            &items[0],
                            user_data,
                            internal_context,
                            depth + 1,
                        )?),
                    };
                for item in &items[1..] {
                    let num = match self.eval_f64(item, user_data, internal_context, depth + 1)? {
                        Some(n) => n,
                        None => helpers::to_f64(&self.evaluate_with_context(
                            item,
                            user_data,
                            internal_context,
                            depth + 1,
                        )?),
                    };
                    if num > max_val {
                        max_val = num;
                    }
                }
                Ok(Some(max_val))
            }
            CompiledLogic::If(cond, then_expr, else_expr) => {
                if self.eval_truthy(cond, user_data, internal_context, depth + 1)? {
                    self.eval_f64(then_expr, user_data, internal_context, depth + 1)
                } else {
                    self.eval_f64(else_expr, user_data, internal_context, depth + 1)
                }
            }
            CompiledLogic::ValueAt(table_expr, row_idx_expr, col_name_expr) => {
                let var_name = match table_expr.as_ref() {
                    CompiledLogic::Var(name, _) | CompiledLogic::Ref(name, _) => {
                        Some(name.as_str())
                    }
                    _ => None,
                };
                if let Some(name) = var_name {
                    let scope = unsafe { &*self.table_scope.get() };
                    if let Some(ts) = scope.as_ref() {
                        if (name == ts.path
                            || name.trim_start_matches('#') == ts.path_no_hash.as_str())
                            && ts.col_count > 0
                            && !ts.flat_cells.is_null()
                        {
                            let fast_row_idx = match row_idx_expr.as_ref() {
                                CompiledLogic::Number(n) => {
                                    let idx = *n as i64;
                                    if idx >= 0 {
                                        Some(idx)
                                    } else {
                                        None
                                    }
                                }
                                CompiledLogic::Var(var, _) | CompiledLogic::Ref(var, _)
                                    if (var == "$iteration" || var == "/$iteration") =>
                                {
                                    ts.iteration_raw
                                }
                                CompiledLogic::Add(items) if items.len() == 2 => {
                                    match (&items[0], &items[1]) {
                                        (
                                            CompiledLogic::Var(var, _) | CompiledLogic::Ref(var, _),
                                            CompiledLogic::Number(n),
                                        )
                                        | (
                                            CompiledLogic::Number(n),
                                            CompiledLogic::Var(var, _) | CompiledLogic::Ref(var, _),
                                        ) if (var == "$iteration" || var == "/$iteration") => {
                                            ts.iteration_raw.map(|iter| iter + (*n as i64))
                                        }
                                        _ => None,
                                    }
                                }
                                CompiledLogic::Subtract(items) if items.len() == 2 => {
                                    match (&items[0], &items[1]) {
                                        (
                                            CompiledLogic::Var(var, _) | CompiledLogic::Ref(var, _),
                                            CompiledLogic::Number(n),
                                        ) if (var == "$iteration" || var == "/$iteration") => {
                                            ts.iteration_raw.map(|iter| iter - (*n as i64))
                                        }
                                        _ => None,
                                    }
                                }
                                _ => None,
                            };
                            let row_idx = match fast_row_idx {
                                Some(idx) => idx,
                                None => match self.eval_f64(
                                    row_idx_expr,
                                    user_data,
                                    internal_context,
                                    depth + 1,
                                )? {
                                    Some(n) => n as i64,
                                    None => helpers::to_f64(&self.evaluate_with_context(
                                        row_idx_expr,
                                        user_data,
                                        internal_context,
                                        depth + 1,
                                    )?) as i64,
                                },
                            };
                            if row_idx >= 0 {
                                let row_idx = row_idx as usize;
                                if row_idx < ts.existing_row_count {
                                    let rows = unsafe { &*ts.rows };
                                    if let Some(row) = rows.get(row_idx) {
                                        if let Some(col_expr) = col_name_expr {
                                            if let CompiledLogic::String(s) = col_expr.as_ref() {
                                                if let Value::Object(map) = row {
                                                    if let Some(cell) = map.get(s.as_str()) {
                                                        if let Value::Number(n) = cell {
                                                            if let Some(f) = n.as_f64() {
                                                                return Ok(Some(f));
                                                            }
                                                        }
                                                        return Ok(Some(helpers::to_f64(cell)));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else if row_idx < ts.existing_row_count + ts.total_rows {
                                    let row_offset = row_idx - ts.existing_row_count;
                                    if let Some(col_expr) = col_name_expr {
                                        if let CompiledLogic::String(s) = col_expr.as_ref() {
                                            if let Some(col_idx) = ts.get_col_idx(s.as_str()) {
                                                let cell = unsafe {
                                                    &*ts.flat_cells
                                                        .add(row_offset * ts.col_count + col_idx)
                                                };
                                                if let Value::Number(n) = cell {
                                                    if let Some(f) = n.as_f64() {
                                                        return Ok(Some(f));
                                                    }
                                                }
                                                return Ok(Some(helpers::to_f64(cell)));
                                            }
                                        }
                                    }
                                } else {
                                    // row_idx >= ts.existing_row_count + ts.total_rows: out of bounds
                                    return Ok(Some(0.0));
                                }
                            } else {
                                // row_idx < 0: out of bounds
                                return Ok(Some(0.0));
                            }
                        }
                    }
                }
                let v = self.eval_valueat(
                    table_expr,
                    row_idx_expr,
                    col_name_expr,
                    user_data,
                    internal_context,
                    depth,
                )?;
                if let Value::Number(n) = &v {
                    if let Some(f) = n.as_f64() {
                        return Ok(Some(f));
                    }
                }
                Ok(Some(helpers::to_f64(&v)))
            }
            _ => Ok(None),
        }
    }

    /// Evaluate binary arithmetic operation on two expressions
    #[inline]
    pub(super) fn eval_binary_arith<F>(
        &self,
        a: &CompiledLogic,
        b: &CompiledLogic,
        f: F,
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Value, String>
    where
        F: FnOnce(f64, f64) -> Option<f64>,
    {
        let val_a = self.evaluate_with_context(a, user_data, internal_context, depth + 1)?;
        let val_b = self.evaluate_with_context(b, user_data, internal_context, depth + 1)?;
        let num_a = helpers::to_f64(&val_a);
        let num_b = helpers::to_f64(&val_b);
        match f(num_a, num_b) {
            Some(result) => Ok(self.f64_to_json(result)),
            None => Ok(Value::Null),
        }
    }

    /// Flatten array values for arithmetic operations
    pub(super) fn flatten_array_values(
        &self,
        items: &[CompiledLogic],
        user_data: &Value,
        internal_context: &Value,
        depth: usize,
    ) -> Result<Vec<f64>, String> {
        let mut values = Vec::new();
        for item in items {
            let val = self.evaluate_with_context(item, user_data, internal_context, depth + 1)?;
            if let Value::Array(arr) = val {
                for elem in arr {
                    values.push(helpers::to_f64(&elem));
                }
            } else {
                values.push(helpers::to_f64(&val));
            }
        }
        Ok(values)
    }
}
