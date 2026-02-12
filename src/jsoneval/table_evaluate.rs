use crate::jsoneval::eval_data::EvalData;
use crate::jsoneval::table_metadata::RowMetadata;
use crate::jsoneval::path_utils;
use crate::JSONEval;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::mem;

use crate::jsoneval::cancellation::CancellationToken;

/// Sandboxed table evaluation for safe parallel execution
///
/// All heavy operations (dependency analysis, forward reference checks) are done at parse time.
/// This function creates an isolated scope to prevent interference between parallel table evaluations.
///
/// # Parallel Safety
///
/// This function is designed for safe parallel execution:
/// - Takes `scope_data` as an immutable reference (read-only parent scope)
/// - Creates an isolated sandbox (clone) for all table-specific mutations
/// - All temporary variables (`$iteration`, `$threshold`, column vars) exist only in the sandbox
/// - The parent `scope_data` remains unchanged, preventing race conditions
/// - Multiple tables can be evaluated concurrently without interference
///
/// # Mutation Safety
///
/// **ALL data mutations go through EvalData methods:**
/// - `sandbox.set()` - sets field values with version tracking
/// - `sandbox.push_to_array()` - appends to arrays with version tracking
/// - `sandbox.get_table_row_mut()` - gets mutable row references (followed by mark_modified)
/// - `sandbox.mark_modified()` - explicitly marks paths as modified
///
/// This ensures proper version tracking and mutation safety throughout evaluation.
///
/// # Sandboxing Strategy
///
/// 1. Clone `scope_data` to create an isolated sandbox at the start
/// 2. All evaluations and mutations happen within the sandbox via EvalData methods
/// 3. Extract the final table array from the sandbox
/// 4. Sandbox is dropped, discarding all temporary state
/// 5. Parent scope remains pristine and can be safely shared across threads
pub fn evaluate_table(
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

    let table_pointer_path = path_utils::normalize_to_json_pointer(eval_key);

    // CREATE SANDBOXED SCOPE (thread-safe isolation)
    let mut sandbox = scope_data.clone();

    // PHASE 0: Evaluate $datas FIRST (before skip/clear)
    let _existing_table_value = sandbox.get(&table_pointer_path).cloned();

    let empty_context = Value::Object(Map::new());
    for (name, logic, literal) in metadata.data_plans.iter() {
        let value = match logic {
            Some(logic_id) => {
                match lib
                    .engine
                    .run_with_context(logic_id, sandbox.data(), &empty_context)
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

        sandbox.set(name.as_ref(), value);
    }

    // PHASE 1: Evaluate $skip - if true, return empty immediately
    let mut should_skip = metadata.skip_literal;
    if !should_skip {
        if let Some(logic_id) = metadata.skip_logic {
            let val = lib
                .engine
                .run_with_context(&logic_id, sandbox.data(), &empty_context)?;
            should_skip = val.as_bool().unwrap_or(false);
        }
    }

    // PHASE 2: Check dependencies before evaluation
    // [Opt 7] Cache required-field results to avoid repeated format!() + pointer lookups
    let mut requirement_not_filled = false;
    if let Some(deps) = lib.dependencies.get(eval_key) {
        let mut required_cache: HashMap<&str, bool> = HashMap::new();

        for dep in deps.iter() {
            if dep.contains("$params")
                || (!dep.contains("$context") && (dep.starts_with("/$") || dep.starts_with("$")))
            {
                continue;
            }

            let is_empty_or_missing = match sandbox.get_without_properties(dep) {
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

    // PHASE 3: Evaluate $clear - if true, ensure table is empty
    let mut should_clear = metadata.clear_literal;
    if !should_clear {
        if let Some(logic_id) = metadata.clear_logic {
            let val = lib
                .engine
                .run_with_context(&logic_id, sandbox.data(), &empty_context)?;
            should_clear = val.as_bool().unwrap_or(false);
        }
    }

    sandbox.set(&table_pointer_path, Value::Array(Vec::new()));

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

    for plan in metadata.row_plans.iter() {
        match plan {
            RowMetadata::Static { columns } => {
                let mut evaluated_row = Map::with_capacity(columns.len());

                // [Opt 1] Build context Value once, mutate in-place between columns
                let mut ctx_value = Value::Object(Map::new());

                for column in columns.iter() {
                    let value = if let Some(logic_id) = column.logic {
                        lib.engine.run_with_context(
                            &logic_id,
                            sandbox.data(),
                            &ctx_value,
                        )?
                    } else {
                        column
                            .literal
                            .as_ref()
                            .map(|arc_val| Value::clone(arc_val))
                            .unwrap_or(Value::Null)
                    };

                    // [Opt 1] Update context in-place instead of cloning the whole map
                    if let Value::Object(ref mut map) = ctx_value {
                        map.insert(column.var_path.as_ref().to_string(), value.clone());
                    }
                    evaluated_row.insert(column.name.as_ref().to_string(), value);
                }

                sandbox.push_to_array(&table_pointer_path, Value::Object(evaluated_row));
            }
            RowMetadata::Repeat {
                start,
                end,
                columns,
                forward_cols,
                normal_cols,
            } => {
                let start_val = if let Some(logic_id) = start.logic {
                    lib.engine
                        .run_with_context(&logic_id, sandbox.data(), &empty_context)?
                } else {
                    Value::clone(&start.literal)
                };
                let end_val = if let Some(logic_id) = end.logic {
                    lib.engine
                        .run_with_context(&logic_id, sandbox.data(), &empty_context)?
                } else {
                    Value::clone(&end.literal)
                };

                let start_idx = number_from_value(&start_val);
                let end_idx = number_from_value(&end_val);

                if start_idx > end_idx {
                    continue;
                }

                let existing_row_count = sandbox
                    .get(&table_pointer_path)
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);

                let total_rows = (end_idx - start_idx + 1) as usize;
                let col_count = columns.len();

                // [Opt 2] Pre-compute both col_names and var_paths once
                let col_names: Vec<String> = columns
                    .iter()
                    .map(|col| col.name.as_ref().to_string())
                    .collect();
                let var_paths: Vec<String> = columns
                    .iter()
                    .map(|col| col.var_path.as_ref().to_string())
                    .collect();

                if let Some(Value::Array(table_arr)) = sandbox.get_mut(&table_pointer_path) {
                    table_arr.reserve(total_rows);
                    for _ in 0..total_rows {
                        let mut row = Map::with_capacity(col_count);
                        for col_name in &col_names {
                            row.insert(col_name.clone(), Value::Null);
                        }
                        table_arr.push(Value::Object(row));
                    }
                }

                // PHASE 4: TOP TO BOTTOM (Forward Pass)
                // [Opt 1] Build context Value once, mutate in-place per iteration
                let mut ctx_value = Value::Object(Map::new());
                if let Value::Object(ref mut map) = ctx_value {
                    map.insert("$threshold".to_string(), Value::from(end_idx));
                }

                for iteration in start_idx..=end_idx {
                    if let Some(t) = token {
                        if t.is_cancelled() {
                            return Err("Cancelled".to_string());
                        }
                    }
                    let row_idx = (iteration - start_idx) as usize;
                    let target_idx = existing_row_count + row_idx;

                    // [Opt 1] Update $iteration in-place
                    if let Value::Object(ref mut map) = ctx_value {
                        map.insert("$iteration".to_string(), Value::from(iteration));
                    }

                    for &col_idx in normal_cols.iter() {
                        let column = &columns[col_idx];
                        let value = match column.logic {
                            Some(logic_id) => lib.engine.run_with_context(
                                &logic_id,
                                sandbox.data(),
                                &ctx_value,
                            )?,
                            None => column
                                .literal
                                .as_ref()
                                .map(|arc_val| Value::clone(arc_val))
                                .unwrap_or(Value::Null),
                        };

                        // [Opt 5] Pre-allocated rows guarantee the key exists
                        if let Some(row_obj) =
                            sandbox.get_table_row_mut(&table_pointer_path, target_idx)
                        {
                            if let Some(cell) = row_obj.get_mut(column.name.as_ref()) {
                                *cell = value.clone();
                            }
                        }
                        // [Opt 1+2] Update context in-place with pre-computed var_path
                        if let Value::Object(ref mut map) = ctx_value {
                            map.insert(var_paths[col_idx].clone(), value);
                        }
                    }
                }
                // TODO: Implement mark_modified if needed for tracking
                // sandbox.mark_modified(&table_pointer_path);

                // PHASE 5 (BACKWARD PASS):
                // Evaluate columns WITH forward references in sandbox
                if !forward_cols.is_empty() {
                    let max_sweeps = 100;
                    let mut scan_from_down = false;
                    let iter_count = (end_idx - start_idx + 1) as usize;

                    // [Opt 1] Reuse a single context Value for backward pass
                    let mut ctx_value = Value::Object(Map::new());
                    if let Value::Object(ref mut map) = ctx_value {
                        map.insert("$threshold".to_string(), Value::from(end_idx));
                    }

                    // [Opt 4] Pre-compute HashMap/HashSet for O(1) dependency lookups
                    let forward_col_map: HashMap<&str, usize> = forward_cols
                        .iter()
                        .enumerate()
                        .map(|(fwd_idx, &col_idx)| (columns[col_idx].name.as_ref(), fwd_idx))
                        .collect();
                    let normal_col_set: HashSet<&str> = normal_cols
                        .iter()
                        .map(|&col_idx| columns[col_idx].name.as_ref())
                        .collect();

                    // [Opt 6] Flatten to 1D and reuse buffers instead of re-allocating
                    let changed_len = iter_count * forward_cols.len();
                    let mut prev_changed = vec![true; changed_len];
                    let mut curr_changed = vec![false; changed_len];

                    for _sweep_num in 1..=max_sweeps {
                        let mut any_changed = false;
                        curr_changed.fill(false);

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

                            // [Opt 1] Update $iteration in-place
                            if let Value::Object(ref mut map) = ctx_value {
                                map.insert("$iteration".to_string(), Value::from(iteration));
                            }

                            // [Opt 3] Restore column values from sandbox row into context
                            if let Some(Value::Array(table_arr)) = sandbox.get(&table_pointer_path)
                            {
                                if let Some(Value::Object(row_obj)) = table_arr.get(target_idx) {
                                    if let Value::Object(ref mut ctx_map) = ctx_value {
                                        for &col_idx in
                                            normal_cols.iter().chain(forward_cols.iter())
                                        {
                                            if let Some(value) =
                                                row_obj.get(columns[col_idx].name.as_ref())
                                            {
                                                ctx_map.insert(
                                                    var_paths[col_idx].clone(),
                                                    value.clone(),
                                                );
                                            }
                                        }
                                    }
                                }
                            }

                            for (fwd_idx, &col_idx) in forward_cols.iter().enumerate() {
                                let column = &columns[col_idx];

                                let mut should_evaluate = _sweep_num == 1;

                                // [Opt 4] Use HashMap/HashSet for O(1) dependency lookups
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
                                                return prev_changed
                                                    [row_offset * forward_cols.len() + dep_fwd_idx];
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
                                        Some(logic_id) => lib.engine.run_with_context(
                                            &logic_id,
                                            sandbox.data(),
                                            &ctx_value,
                                        )?,
                                        None => column
                                            .literal
                                            .as_ref()
                                            .map(|arc_val| Value::clone(arc_val))
                                            .unwrap_or(Value::Null),
                                    };

                                    // [Opt 5] Pre-allocated rows guarantee the key exists
                                    if let Some(row_obj) = sandbox
                                        .get_table_row_mut(&table_pointer_path, target_idx)
                                    {
                                        if let Some(cell) = row_obj.get_mut(column.name.as_ref()) {
                                            if *cell != value {
                                                any_changed = true;
                                                // [Opt 6] Flat 1D indexing
                                                curr_changed
                                                    [row_offset * forward_cols.len() + fwd_idx] =
                                                    true;
                                                *cell = value.clone();
                                            }
                                        }
                                    }

                                    // [Opt 1+2] Update context in-place
                                    if let Value::Object(ref mut map) = ctx_value {
                                        map.insert(var_paths[col_idx].clone(), value);
                                    }
                                }
                            }
                        }

                        scan_from_down = !scan_from_down;
                        mem::swap(&mut prev_changed, &mut curr_changed);

                        if !any_changed {
                            break;
                        }
                    }
                }
            }
        }
    }

    let final_rows = if let Some(table_value) = sandbox.get_mut(&table_pointer_path) {
        if let Some(array) = table_value.as_array_mut() {
            mem::take(array)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(final_rows)
}

/// Check if a field is required based on the schema rules
///
/// This function looks up the field in the evaluated schema and checks if it has
/// a "required" rule with value=true. If the field doesn't exist in the schema
/// or doesn't have a required rule, it's considered optional.
///
/// # Arguments
///
/// * `schema` - The evaluated schema Value
/// * `dep_path` - The dependency path (JSON pointer format, e.g., "/properties/field")
///
/// # Returns
///
/// * `true` if the field is required, `false` if optional or not found
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
