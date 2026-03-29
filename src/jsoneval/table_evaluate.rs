use crate::jsoneval::eval_data::EvalData;
use crate::jsoneval::table_metadata::RowMetadata;
use crate::jsoneval::path_utils;
use crate::JSONEval;
use crate::time_block;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

use crate::jsoneval::cancellation::CancellationToken;

/// Zero-sandbox table evaluation
///
/// Eliminates the full `EvalData` clone (sandbox) by:
/// 1. **Local row storage**: rows are built into a `Vec<Value>` directly on the stack.
/// 2. **Self-table scope**: the evaluator's `TableScope` intercepts Var/Ref/ValueAt
///    lookups for the current table's path, returning rows from local storage.
/// 3. **Direct mutation**: forward/backward passes index `local_rows` by integer —
///    no `Arc::make_mut`, no JSON-pointer traversal per cell.
/// 4. **$datas in context**: evaluated variable bindings are passed as entries
///    inside `internal_context` (checked first), not written to scope_data.
///
/// The caller (`evaluate_internal`) remains responsible for writing results back
/// to `eval_data` / `static_arrays` / `evaluated_schema`.
pub fn evaluate_table(
    lib: &JSONEval,
    eval_key: &str,
    scope_data: &EvalData,
    token: Option<&CancellationToken>,
) -> Result<Vec<Value>, String> {
    let _total_start: Option<std::time::Instant> = if crate::utils::is_timing_enabled() {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let result = evaluate_table_inner(lib, eval_key, scope_data, token);
    if let Some(start) = _total_start {
        crate::utils::record_timing(
            &format!("[table::{}] total", eval_key),
            start.elapsed(),
        );
    }
    result
}

fn evaluate_table_inner(
    lib: &JSONEval,
    eval_key: &str,
    scope_data: &EvalData,
    token: Option<&CancellationToken>,
) -> Result<Vec<Value>, String> {
    let metadata = lib
        .table_metadata
        .get(eval_key)
        .ok_or_else(|| format!("Table metadata not found for {}", eval_key))?
        .clone();

    if let Some(t) = token {
        if t.is_cancelled() {
            return Err("Cancelled".to_string());
        }
    }

    let table_pointer_path = path_utils::normalize_to_json_pointer(eval_key).into_owned();

    // PHASE 0: Evaluate $datas first.
    // Instead of writing to a sandbox, we collect overrides into `data_ctx` which
    // gets merged into ctx_value (internal_context). The evaluator checks
    // internal_context before user_data, so $datas are visible to all column logic.
    let mut data_ctx: Map<String, Value> = Map::new();
    time_block!(&format!("[table::{}] phase0 $datas", eval_key), {
        let empty_ctx = Value::Object(Map::new());
        for (name, logic, literal) in metadata.data_plans.iter() {
            // Skip if already present in scope_data
            if scope_data.get(name.as_ref()).is_some() {
                continue;
            }

            let value = match logic {
                Some(logic_id) => {
                    match lib
                        .engine
                        .run_with_context(logic_id, scope_data.data(), &empty_ctx)
                    {
                        Ok(val) => val,
                        Err(_) => literal
                            .as_ref()
                            .map(|arc_val| Value::clone(arc_val))
                            .unwrap_or(Value::Null),
                    }
                }
                None => literal
                    .as_ref()
                    .map(|arc_val| Value::clone(arc_val))
                    .unwrap_or(Value::Null),
            };

            // Normalize: strip leading '/' so it becomes a top-level key
            let key = name.as_ref().trim_start_matches('/').to_string();
            data_ctx.insert(key, value);
        }
    });

    // PHASE 1: Evaluate $skip
    let mut should_skip = metadata.skip_literal;
    if !should_skip {
        if let Some(logic_id) = metadata.skip_logic {
            let ctx = Value::Object(data_ctx.clone());
            let val = time_block!(&format!("[table::{}] phase1 $skip", eval_key), {
                lib.engine
                    .run_with_context(&logic_id, scope_data.data(), &ctx)
                    .unwrap_or(Value::Null)
            });
            should_skip = val.as_bool().unwrap_or(false);
        }
    }

    // PHASE 2: Check dependencies
    let mut requirement_not_filled = false;
    time_block!(&format!("[table::{}] phase2 dep-check", eval_key), {
        if let Some(deps) = lib.dependencies.get(eval_key) {
            let mut required_cache: HashMap<&str, bool> = HashMap::new();

            for dep in deps.iter() {
                if dep.contains("$params")
                    || (!dep.contains("$context") && (dep.starts_with("/$") || dep.starts_with("$")))
                {
                    continue;
                }

                let is_empty_or_missing = match scope_data.get_without_properties(dep) {
                    Some(dep_value) => match dep_value {
                        Value::Null => true,
                        Value::String(s) => s.is_empty(),
                        Value::Array(arr) => arr.is_empty(),
                        Value::Object(obj) => obj.is_empty(),
                        _ => false,
                    },
                    None => true,
                };

                if is_empty_or_missing {
                    let is_field_required = *required_cache
                        .entry(dep.as_str())
                        .or_insert_with(|| check_field_required(&lib.evaluated_schema, dep));

                    if is_field_required {
                        requirement_not_filled = true;
                        break;
                    }
                }
            }
        }
    });

    // PHASE 3: Evaluate $clear
    let mut should_clear = metadata.clear_literal;
    if !should_clear {
        if let Some(logic_id) = metadata.clear_logic {
            let ctx = Value::Object(data_ctx.clone());
            let val = time_block!(&format!("[table::{}] phase3 $clear", eval_key), {
                lib.engine
                    .run_with_context(&logic_id, scope_data.data(), &ctx)
                    .unwrap_or(Value::Null)
            });
            should_clear = val.as_bool().unwrap_or(false);
        }
    }

    if should_clear || should_skip || requirement_not_filled {
        return Ok(Vec::new());
    }

    let number_from_value = |value: &Value| -> i64 {
        match value {
            Value::Number(n) => n
                .as_i64()
                .unwrap_or_else(|| n.as_f64().map_or(0, |f| f as i64)),
            Value::String(s) => s.parse::<f64>().map_or(0, |f| f as i64),
            Value::Bool(true) => 1,
            Value::Bool(false) => 0,
            _ => 0,
        }
    };

    // Accumulate all row plans into a single local_rows Vec
    let mut local_rows: Vec<Value> = Vec::new();

    for plan in metadata.row_plans.iter() {
        match plan {
            RowMetadata::Static { columns } => {
                time_block!(&format!("[table::{}] static-row", eval_key), {
                    let mut evaluated_row = Map::with_capacity(columns.len());
                    let mut ctx_value = Value::Object(data_ctx.clone());

                    for column in columns.iter() {
                        let value = if let Some(logic_id) = column.logic {
                            lib.engine
                                .run_with_context(
                                    &logic_id,
                                    scope_data.data(),
                                    &ctx_value,
                                )
                                .unwrap_or(Value::Null)
                        } else {
                            column
                                .literal
                                .as_ref()
                                .map(|arc_val| Value::clone(arc_val))
                                .unwrap_or(Value::Null)
                        };

                        if let Value::Object(ref mut map) = ctx_value {
                            map.insert(column.var_path.as_ref().to_string(), value.clone());
                        }
                        evaluated_row.insert(column.name.as_ref().to_string(), value);
                    }

                    local_rows.push(Value::Object(evaluated_row));
                });
            }
            RowMetadata::Repeat {
                start,
                end,
                columns,
                forward_cols,
                normal_cols,
            } => {
                let empty_ctx = Value::Object(data_ctx.clone());

                let start_val = if let Some(logic_id) = start.logic {
                    match lib.engine.run_with_context(&logic_id, scope_data.data(), &empty_ctx) {
                        Ok(v) => v,
                        Err(_) => {
                            // Logic failed: try to use literal as a number, else skip this row group
                            if let Some(n) = start.literal.as_i64() {
                                Value::from(n)
                            } else {
                                continue; // can't determine bounds, skip
                            }
                        }
                    }
                } else {
                    Value::clone(&start.literal)
                };
                let end_val = if let Some(logic_id) = end.logic {
                    match lib.engine.run_with_context(&logic_id, scope_data.data(), &empty_ctx) {
                        Ok(v) => v,
                        Err(_) => {
                            if let Some(n) = end.literal.as_i64() {
                                Value::from(n)
                            } else {
                                continue;
                            }
                        }
                    }
                } else {
                    Value::clone(&end.literal)
                };

                let start_idx = number_from_value(&start_val);
                let end_idx = number_from_value(&end_val);

                if start_idx > end_idx {
                    continue;
                }

                let existing_row_count = local_rows.len();
                let total_rows = (end_idx - start_idx + 1) as usize;
                let col_count = columns.len();
                let _ = col_count;

                // Pre-compute column name strings once
                let col_names: Vec<String> = columns
                    .iter()
                    .map(|col| col.name.as_ref().to_string())
                    .collect();

                // Pre-allocate rows with null cells
                local_rows.reserve(total_rows);
                for _ in 0..total_rows {
                    let row: Map<String, Value> = col_names
                        .iter()
                        .map(|n| (n.clone(), Value::Null))
                        .collect();
                    local_rows.push(Value::Object(row));
                }

                // Register this table's scope on the evaluator so self-table
                // Var/Ref/ValueAt lookups resolve from local_rows.
                // The guard is dropped at end of this block, clearing the scope.
                let _scope_guard = lib.engine.enter_table_scope(
                    table_pointer_path.clone(),
                    &local_rows,
                );

                let key_iteration = String::from("$iteration");
                let key_threshold = String::from("$threshold");
                let threshold_value = Value::from(end_idx);

                // Build base ctx with data_ctx entries + iteration slots
                let mut ctx_value = Value::Object({
                    let mut m = data_ctx.clone();
                    m.insert(key_threshold.clone(), threshold_value.clone());
                    m.insert(key_iteration.clone(), Value::Null);
                    m
                });

                // PHASE 4: FORWARD PASS — top to bottom
                time_block!(
                    &format!("[table::{}] forward-pass rows={}", eval_key, total_rows),
                    {
                        for iteration in start_idx..=end_idx {
                            if let Some(t) = token {
                                if t.is_cancelled() {
                                    return Err("Cancelled".to_string());
                                }
                            }
                            let row_idx = existing_row_count + (iteration - start_idx) as usize;

                            // Update $iteration in ctx_value in-place
                            if let Value::Object(ref mut map) = ctx_value {
                                if let Some(slot) = map.get_mut(&key_iteration) {
                                    *slot = Value::from(iteration);
                                }
                            }

                            // Update the scope guard so self-table lookups see
                            // the already-populated earlier rows
                            lib.engine.update_table_scope_rows(&local_rows);
                            // Point get_var lookup directly to the actively evaluating cell
                            lib.engine.set_table_scope_row(Some(row_idx));

                            for &col_idx in normal_cols.iter() {
                                let column = &columns[col_idx];
                                let value = match column.logic {
                                    Some(logic_id) => lib.engine
                                        .run_with_context(
                                            &logic_id,
                                            scope_data.data(),
                                            &ctx_value,
                                        )
                                        .unwrap_or(Value::Null),
                                    None => column
                                        .literal
                                        .as_ref()
                                        .map(|arc_val| Value::clone(arc_val))
                                        .unwrap_or(Value::Null),
                                };

                                // Write directly into local_rows — no Arc::make_mut, no pointer traversal
                                if let Value::Object(ref mut row) = local_rows[row_idx] {
                                    if let Some(cell) = row.get_mut(column.name.as_ref()) {
                                        *cell = value;
                                    }
                                }
                            }
                            // Reset cursor after row
                            lib.engine.set_table_scope_row(None);
                        }
                    }
                );

                // PHASE 5: BACKWARD PASS for forward-ref columns
                if !forward_cols.is_empty() {
                    let max_sweeps = 100;
                    let mut scan_from_down = false;
                    let iter_count = (end_idx - start_idx + 1) as usize;

                    // Build backward-pass ctx_value (same structure as forward)
                    let mut ctx_value = Value::Object({
                        let mut m = data_ctx.clone();
                        m.insert(key_threshold.clone(), threshold_value.clone());
                        m.insert(key_iteration.clone(), Value::Null);
                        m
                    });

                    // [Opt 4] Pre-compute HashMap/HashSet for O(1) dep lookups
                    let forward_col_map: HashMap<&str, usize> = forward_cols
                        .iter()
                        .enumerate()
                        .map(|(fwd_idx, &col_idx)| (columns[col_idx].name.as_ref(), fwd_idx))
                        .collect();
                    let normal_col_set: HashSet<&str> = normal_cols
                        .iter()
                        .map(|&col_idx| columns[col_idx].name.as_ref())
                        .collect();

                    let changed_len = iter_count * forward_cols.len();
                    let mut prev_changed = vec![true; changed_len];
                    let mut curr_changed = vec![false; changed_len];

                    let _backward_start: Option<std::time::Instant> =
                        if crate::utils::is_timing_enabled() {
                            Some(std::time::Instant::now())
                        } else {
                            None
                        };
                    let mut total_sweeps: usize = 0;

                    for _sweep_num in 1..=max_sweeps {
                        total_sweeps = _sweep_num;
                        let mut any_changed = false;
                        curr_changed.fill(false);

                        // Update scope so all rows are visible during backward sweep
                        lib.engine.update_table_scope_rows(&local_rows);

                        for iter_offset in 0..iter_count {
                            if let Some(t) = token {
                                if t.is_cancelled() {
                                    return Err("Cancelled".to_string());
                                }
                            }
                            let iteration = if scan_from_down {
                                end_idx - iter_offset as i64
                            } else {
                                start_idx + iter_offset as i64
                            };
                            let row_offset = (iteration - start_idx) as usize;
                            let target_idx = existing_row_count + row_offset;

                            // Update $iteration in ctx_value in-place
                            if let Value::Object(ref mut map) = ctx_value {
                                if let Some(slot) = map.get_mut(&key_iteration) {
                                    *slot = Value::from(iteration);
                                }
                            }

                            // Explicitly direct column resolution to local stack rows cursor
                            lib.engine.set_table_scope_row(Some(target_idx));

                            for (fwd_idx, &col_idx) in forward_cols.iter().enumerate() {
                                let column = &columns[col_idx];

                                let mut should_evaluate = _sweep_num == 1;

                                if !should_evaluate && !column.has_forward_ref {
                                    should_evaluate = column.dependencies.iter().any(|dep| {
                                        if dep == "$iteration" || dep == "$threshold" {
                                            return false;
                                        }

                                        if dep.starts_with('$') {
                                            let dep_name = dep.trim_start_matches('$');

                                            if let Some(&dep_fwd_idx) =
                                                forward_col_map.get(dep_name)
                                            {
                                                return prev_changed[row_offset
                                                    * forward_cols.len()
                                                    + dep_fwd_idx];
                                            }

                                            if normal_col_set.contains(dep_name) {
                                                return false;
                                            }

                                            true
                                        } else {
                                            true
                                        }
                                    });
                                } else if !should_evaluate {
                                    should_evaluate = true;
                                }

                                if should_evaluate {
                                    let value = match column.logic {
                                        Some(logic_id) => lib.engine
                                            .run_with_context(
                                                &logic_id,
                                                scope_data.data(),
                                                &ctx_value,
                                            )
                                            .unwrap_or(Value::Null),
                                        None => column
                                            .literal
                                            .as_ref()
                                            .map(|arc_val| Value::clone(arc_val))
                                            .unwrap_or(Value::Null),
                                    };

                                    // Write directly to local_rows — no Arc::make_mut
                                    if let Value::Object(ref mut row) = local_rows[target_idx] {
                                        if let Some(cell) = row.get_mut(column.name.as_ref()) {
                                            if *cell != value {
                                                any_changed = true;
                                                curr_changed[row_offset * forward_cols.len()
                                                    + fwd_idx] = true;
                                                *cell = value;
                                            }
                                        }
                                    }
                                    }
                            }
                        }
                        // Reset cursor after backwards row evaluating loop
                        lib.engine.set_table_scope_row(None);
                        
                        scan_from_down = !scan_from_down;
                        std::mem::swap(&mut prev_changed, &mut curr_changed);

                        if !any_changed {
                            break;
                        }
                    }

                    if let Some(start) = _backward_start {
                        crate::utils::record_timing(
                            &format!(
                                "[table::{}] backward-pass rows={} sweeps={}",
                                eval_key, iter_count, total_sweeps
                            ),
                            start.elapsed(),
                        );
                    }
                }

                // _scope_guard dropped here → TableScope cleared on evaluator
            }
        }
    }

    Ok(local_rows)
}

/// Check if a field is required based on the schema rules
fn check_field_required(schema: &Value, dep_path: &str) -> bool {
    let rules_path = format!(
        "{}/rules/required",
        path_utils::dot_notation_to_schema_pointer(dep_path)
    );

    if let Some(required_rule) = path_utils::get_value_by_pointer(schema, &rules_path) {
        if let Some(rule_obj) = required_rule.as_object() {
            if let Some(Value::Bool(is_required)) = rule_obj.get("value") {
                return *is_required;
            }
        }
        if let Some(is_required) = required_rule.as_bool() {
            return is_required;
        }
    }

    false
}
