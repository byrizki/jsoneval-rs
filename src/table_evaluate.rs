use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use crate::{JSONEval, LogicId, TrackedData};

pub fn evaluate_table(
    lib: &mut JSONEval,
    eval_key: &str,
    table: &Value,
    scope_data: &mut TrackedData,
) -> Result<Vec<Value>, String> {
    #[derive(Clone)]
    struct ColumnPlan {
        name: String,
        var_path: String,
        logic: Option<LogicId>,
        literal: Option<Value>,
        dependencies: Vec<String>,
        has_forward_ref: bool,
    }

    impl ColumnPlan {
        fn new(name: &str, logic: Option<LogicId>, literal: Option<Value>, dependencies: Vec<String>, has_forward_ref: bool) -> Self {
            Self {
                name: name.to_string(),
                var_path: format!("${name}"),
                logic,
                literal,
                dependencies,
                has_forward_ref,
            }
        }
    }

    #[derive(Clone)]
    struct RepeatBound {
        logic: Option<LogicId>,
        literal: Value,
    }

    #[derive(Clone)]
    enum RowPlan {
        Static {
            columns: Vec<ColumnPlan>,
        },
        Repeat {
            start: RepeatBound,
            end: RepeatBound,
            columns: Vec<ColumnPlan>,
        },
    }

    let rows = table
        .get("rows")
        .and_then(|v| v.as_array())
        .ok_or("table missing rows")?;
    let empty_datas = Vec::new();
    let datas = table
        .get("datas")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_datas);

    // Pre-allocate with exact capacity
    let mut data_plans = Vec::with_capacity(datas.len());
    for (idx, entry) in datas.iter().enumerate() {
        let Some(name) = entry.get("name").and_then(|v| v.as_str()) else { continue };
        let logic_path = format!("{eval_key}/$datas/{idx}/data");
        let logic = lib.evaluations.get(&logic_path).copied();
        let literal = entry.get("data").cloned();
        data_plans.push((name.to_string(), logic, literal));
    }

    let mut row_plans = Vec::with_capacity(rows.len());
    for (row_idx, row_val) in rows.iter().enumerate() {
        let Some(row_obj) = row_val.as_object() else {
            continue;
        };

        if let Some(repeat_arr) = row_obj.get("$repeat").and_then(|v| v.as_array()) {
            if repeat_arr.len() == 3 {
                let start_logic_path = format!("{eval_key}/$table/{row_idx}/$repeat/0");
                let end_logic_path = format!("{eval_key}/$table/{row_idx}/$repeat/1");
                let start_logic = lib.evaluations.get(&start_logic_path).copied();
                let end_logic = lib.evaluations.get(&end_logic_path).copied();

                let start_literal = repeat_arr.get(0).cloned().unwrap_or(Value::Null);
                let end_literal = repeat_arr.get(1).cloned().unwrap_or(Value::Null);

                if let Some(template) = repeat_arr.get(2).and_then(|v| v.as_object()) {
                    let mut columns = Vec::with_capacity(template.len());
                    for (col_name, col_val) in template {
                        let col_eval_path =
                            format!("{eval_key}/$table/{row_idx}/$repeat/2/{col_name}");
                        let logic = lib.evaluations.get(&col_eval_path).copied();
                        let literal = if logic.is_none() {
                            Some(col_val.clone())
                        } else {
                            None
                        };
                        
                        // Extract dependencies for this column
                        let (dependencies, has_forward_ref) = if let Some(logic_id) = logic {
                            let deps = lib.engine.get_referenced_vars(&logic_id)
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|v| v.starts_with('$') && v != "$iteration" && v != "$threshold")
                                .collect();
                            let has_fwd = lib.engine.has_forward_reference(&logic_id);
                            (deps, has_fwd)
                        } else {
                            (Vec::new(), false)
                        };
                        columns.push(ColumnPlan::new(col_name, logic, literal, dependencies, has_forward_ref));
                    }

                    row_plans.push(RowPlan::Repeat {
                        start: RepeatBound {
                            logic: start_logic,
                            literal: start_literal,
                        },
                        end: RepeatBound {
                            logic: end_logic,
                            literal: end_literal,
                        },
                        columns,
                    });
                    continue;
                }
            }
        }

        let mut columns = Vec::with_capacity(row_obj.len());
        for (col_name, col_val) in row_obj {
            if col_name == "$repeat" {
                continue;
            }
            let col_eval_path = format!("{eval_key}/$table/{row_idx}/{col_name}");
            let logic = lib.evaluations.get(&col_eval_path).copied();
            let literal = if logic.is_none() {
                Some(col_val.clone())
            } else {
                None
            };
            
            // Extract dependencies for this column
            let (dependencies, has_forward_ref) = if let Some(logic_id) = logic {
                let deps = lib.engine.get_referenced_vars(&logic_id)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|v| v.starts_with('$') && v != "$iteration" && v != "$threshold")
                    .collect();
                let has_fwd = lib.engine.has_forward_reference(&logic_id);
                (deps, has_fwd)
            } else {
                (Vec::new(), false)
            };
            
            columns.push(ColumnPlan::new(col_name, logic, literal, dependencies, has_forward_ref));
        }
        row_plans.push(RowPlan::Static { columns });
    }

    let table_dotted_path = eval_key.trim_start_matches("#/").replace('/', ".");
    
    // ==========================================
    // PHASE 0: Evaluate $datas FIRST (before skip/clear)
    // ==========================================
    // This matches JS behavior: datas must be available before skip/clear evaluation
    for (name, logic, literal) in &data_plans {
        let value = if let Some(logic_id) = logic {
            match lib.engine.evaluate(logic_id, scope_data) {
                Ok(val) => (*val).clone(),
                Err(_) => literal.clone().unwrap_or(Value::Null),
            }
        } else {
            literal.clone().unwrap_or(Value::Null)
        };
        scope_data.set(name, value);
    }

    // Initialize empty table array
    scope_data.set(
        &table_dotted_path,
        Value::Array(Vec::with_capacity(rows.len())),
    );

    // ==========================================
    // Evaluate $skip - if true, return empty immediately
    // ==========================================
    let skip_logic = lib.evaluations.get(&format!("{eval_key}/$skip")).copied();
    let mut should_skip = table.get("skip").and_then(Value::as_bool).unwrap_or(false);
    if !should_skip {
        if let Some(logic_id) = skip_logic {
            let val = lib.engine.evaluate(&logic_id, scope_data)?;
            should_skip = val.as_bool().unwrap_or(false);
        }
    }
    if should_skip {
        return Ok(Vec::new());
    }

    // ==========================================
    // Evaluate $clear - if true, ensure table is empty
    // ==========================================
    let clear_logic = lib.evaluations.get(&format!("{eval_key}/$clear")).copied();
    let mut should_clear = table.get("clear").and_then(Value::as_bool).unwrap_or(false);
    if !should_clear {
        if let Some(logic_id) = clear_logic {
            let val = lib.engine.evaluate(&logic_id, scope_data)?;
            should_clear = val.as_bool().unwrap_or(false);
        }
    }
    if should_clear {
        scope_data.set(&table_dotted_path, Value::Array(Vec::new()));
    }

    let number_from_value = |value: &Value| -> i64 {
        match value {
            Value::Number(n) => n
                .as_i64()
                .or_else(|| n.as_f64().map(|f| f as i64))
                .unwrap_or(0),
            Value::String(s) => s.parse::<f64>().map(|f| f as i64).unwrap_or(0),
            Value::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    };

    for plan in &row_plans {
        match plan {
            RowPlan::Static { columns } => {
                // CRITICAL: Preserve SCHEMA ORDER for static rows (match JavaScript behavior)
                // Do NOT use topological sort as it reorders columns incorrectly
                let mut evaluated_row = Map::with_capacity(columns.len());
                
                // Evaluate columns in schema order
                for column in columns {
                    let value = if let Some(logic_id) = column.logic {
                        lib.engine.evaluate_uncached(&logic_id, scope_data)?
                    } else {
                        column.literal.clone().unwrap_or(Value::Null)
                    };
                    evaluated_row.insert(column.name.clone(), value);
                    if let Some(v) = evaluated_row.get(&column.name) {
                        scope_data.set(&column.var_path, v.clone());
                    }
                }
                
                scope_data.push_to_array(&table_dotted_path, Value::Object(evaluated_row));
            }
            RowPlan::Repeat {
                start,
                end,
                columns,
            } => {
                let start_literal = start.literal.clone();
                let end_literal = end.literal.clone();
                let start_val = if let Some(logic_id) = start.logic {
                    lib.engine.evaluate_uncached(&logic_id, scope_data)?
                } else {
                    start_literal
                };
                let end_val = if let Some(logic_id) = end.logic {
                    lib.engine.evaluate_uncached(&logic_id, scope_data)?
                } else {
                    end_literal
                };

                let start_idx = number_from_value(&start_val);
                let end_idx = number_from_value(&end_val);

                if start_idx > end_idx {
                    continue;
                }
                
                // Count existing static rows (avoid cloning until needed)
                let existing_row_count = scope_data
                    .get(&table_dotted_path)
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);

                // Propagate forward-referencing flag to dependent columns
                // Build set of all forward-referencing column names (direct + transitive)
                let mut fwd_cols: HashSet<String> = columns.iter()
                    .filter(|c| c.has_forward_ref)
                    .map(|c| c.name.clone())
                    .collect();
                
                let mut changed = true;
                while changed {
                    changed = false;
                    for col in columns.iter() {
                        if !fwd_cols.contains(&col.name) {
                            // Check if this column depends on any forward-referencing column
                            for dep in &col.dependencies {
                                let dep_name = dep.trim_start_matches('$');
                                if fwd_cols.contains(dep_name) {
                                    fwd_cols.insert(col.name.clone());
                                    changed = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                // Separate columns into forward-referencing and normal columns
                let (forward_cols, normal_cols): (Vec<_>, Vec<_>) = columns.iter()
                    .partition(|col| fwd_cols.contains(&col.name));
                
                // CRITICAL: Preserve SCHEMA ORDER for normal columns (match JavaScript behavior)
                // Do NOT use topological sort as it reorders columns incorrectly
                let sorted_normal_cols: Vec<String> = normal_cols.iter()
                    .map(|c| c.name.clone())
                    .collect();
                
                let column_map: HashMap<_, _> = columns.iter()
                    .map(|c| (c.name.clone(), c))
                    .collect();
                
                // Pre-allocate all rows directly in scope_data
                let total_rows = (end_idx - start_idx + 1) as usize;
                if let Some(Value::Array(table_arr)) = scope_data.get_mut(&table_dotted_path) {
                    for _ in 0..total_rows {
                        table_arr.push(Value::Object(Map::with_capacity(columns.len())));
                    }
                }
                scope_data.mark_modified(&table_dotted_path);
                
                // ========================================
                // PHASE 1: TOP TO BOTTOM (Forward Pass)
                // ========================================
                // Evaluate columns WITHOUT forward references
                // Direction: iteration 1 → N
                
                for iteration in start_idx..=end_idx {
                    let row_idx = (iteration - start_idx) as usize;
                    let target_idx = existing_row_count + row_idx;
                    
                    scope_data.set("$iteration", Value::from(iteration));
                    scope_data.set("$threshold", Value::from(end_idx));
                    
                    // Evaluate normal columns in dependency order
                    for col_name in &sorted_normal_cols {
                        if let Some(column) = column_map.get(col_name) {
                            let value = if let Some(logic_id) = column.logic {
                                lib.engine.evaluate_uncached(&logic_id, scope_data)?
                            } else {
                                column.literal.clone().unwrap_or(Value::Null)
                            };
                            
                            // Write directly to scope_data table
                            if let Some(Value::Array(table_arr)) = scope_data.get_mut(&table_dotted_path) {
                                if let Some(Value::Object(row_obj)) = table_arr.get_mut(target_idx) {
                                    row_obj.insert(column.name.clone(), value.clone());
                                }
                            }
                            scope_data.set(&column.var_path, value);
                        }
                    }
                    // Mark table as modified after each row
                    scope_data.mark_modified(&table_dotted_path);
                }
                
                // ========================================
                // PHASE 2 (BACKWARD PASS):
                // Evaluate columns WITH forward references (depends on future values)
                // Direction: iteration N → 1
                // CRITICAL: Match JavaScript - evaluate ALL columns together per row
                // ========================================
                if !forward_cols.is_empty() {
                    let mut sweep_count = 0;
                    let max_sweeps = 3; // MAX_LOOP_THRESHOLD from JavaScript
                    let mut scan_from_down = true; // Start bottom-up like JavaScript
                    
                    while sweep_count < max_sweeps {
                        sweep_count += 1;
                        let iter_count = (end_idx - start_idx + 1) as usize;
                        
                        for iter_offset in 0..iter_count {
                            let iteration = if scan_from_down {
                                end_idx - iter_offset as i64
                            } else {
                                start_idx + iter_offset as i64
                            };
                            let row_idx = (iteration - start_idx) as usize;
                            let target_idx = existing_row_count + row_idx;
                            
                            scope_data.set("$iteration", Value::from(iteration));
                            scope_data.set("$threshold", Value::from(end_idx));
                            
                            // Restore ALL column values to scope ONCE at the start (frozen snapshot)
                            // This matches JavaScript: ...this.lodash.mapKeys(nrow, (_, k) => `$${k}`)
                            // Read current row from scope_data
                            let current_row_snapshot = if let Some(Value::Array(table_arr)) = scope_data.get(&table_dotted_path) {
                                table_arr.get(target_idx)
                                    .and_then(|v| v.as_object())
                                    .cloned()
                            } else {
                                None
                            };
                            
                            if let Some(row_obj) = current_row_snapshot {
                                for col_name in &sorted_normal_cols {
                                    if let Some(value) = row_obj.get(col_name.as_str()) {
                                        scope_data.set(&format!("${}", col_name), value.clone());
                                    }
                                }
                                for fwd_col in &forward_cols {
                                    if let Some(value) = row_obj.get(&fwd_col.name) {
                                        scope_data.set(&fwd_col.var_path, value.clone());
                                    }
                                }
                            }
                            
                            // Evaluate ALL forward columns WITHOUT updating scope during loop
                            // Each column sees the SAME frozen snapshot of current row values
                            for column in &forward_cols {
                                let value = if let Some(logic_id) = column.logic {
                                    lib.engine.evaluate_uncached(&logic_id, scope_data)?
                                } else {
                                    column.literal.clone().unwrap_or(Value::Null)
                                };
                                
                                // Write directly to scope_data table
                                if let Some(Value::Array(table_arr)) = scope_data.get_mut(&table_dotted_path) {
                                    if let Some(Value::Object(row_obj)) = table_arr.get_mut(target_idx) {
                                        row_obj.insert(column.name.clone(), value);
                                    }
                                }
                                // DON'T update scope_data variables - scope stays frozen for this iteration
                            }
                            
                            // CRITICAL: Mark modified after EACH row so VALUEAT in subsequent rows sees the update
                            scope_data.mark_modified(&table_dotted_path);
                        }
                        
                        // Alternate scan direction (matches JavaScript line 1285)
                        scan_from_down = !scan_from_down;
                    }
                }
            }
        }
    }

    let final_rows = scope_data
        .get(&table_dotted_path)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_else(Vec::new);

    Ok(final_rows)
}
