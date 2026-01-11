use crate::jsoneval::eval_data::EvalData;
use crate::jsoneval::table_metadata::RowMetadata;
use crate::jsoneval::path_utils;
use crate::JSONEval;
use serde_json::{Map, Value};
use std::mem;

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
    lib: &JSONEval, // Changed to immutable - parallel-safe, only reads metadata and calls engine
    eval_key: &str,
    scope_data: &EvalData, // Now immutable - we read from parent scope
) -> Result<Vec<Value>, String> {
    // Clone metadata (cheap since it uses Arc internally)
    let metadata = lib
        .table_metadata
        .get(eval_key)
        .ok_or_else(|| format!("Table metadata not found for {}", eval_key))?
        .clone();

    // Pre-compute table path once using JSON pointer format
    let table_pointer_path = path_utils::normalize_to_json_pointer(eval_key);

    // ==========================================
    // CREATE SANDBOXED SCOPE (thread-safe isolation)
    // ==========================================
    // Clone scope_data to create an isolated sandbox for this table evaluation
    // This prevents parallel table evaluations from interfering with each other
    let mut sandbox = scope_data.clone();

    // ==========================================
    // PHASE 0: Evaluate $datas FIRST (before skip/clear)
    // ==========================================
    // Capture existing table value and track if dependencies change
    let existing_table_value = sandbox.get(&table_pointer_path).cloned();

    // Use empty internal context for $data evaluation
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

    // ==========================================
    // PHASE 1: Evaluate $skip - if true, return empty immediately
    // ==========================================
    let mut should_skip = metadata.skip_literal;
    if !should_skip {
        if let Some(logic_id) = metadata.skip_logic {
            let val = lib
                .engine
                .run_with_context(&logic_id, sandbox.data(), &empty_context)?;
            should_skip = val.as_bool().unwrap_or(false);
        }
    }

    // ==========================================
    // PHASE 2: Check dependencies before evaluation
    // ==========================================
    // Skip evaluation if required dependencies (non-$params, non-$ prefixed) are null/empty
    let mut requirement_not_filled = false;
    if let Some(deps) = lib.dependencies.get(eval_key) {
        for dep in deps.iter() {
            // Skip $params and any dependency starting with $
            if dep.contains("$params")
                || (!dep.contains("$context") && (dep.starts_with("/$") || dep.starts_with("$")))
            {
                continue;
            }

            // Check if this dependency is null or empty
            if let Some(dep_value) = sandbox.get_without_properties(dep) {
                let is_empty = match dep_value {
                    Value::Null => true,
                    Value::String(s) => s.is_empty(),
                    Value::Array(arr) => arr.is_empty(),
                    Value::Object(obj) => obj.is_empty(),
                    _ => false,
                };

                if is_empty {
                    // Check if the field is required in the schema before skipping
                    let is_field_required = check_field_required(&lib.evaluated_schema, dep);

                    if is_field_required {
                        requirement_not_filled = true;
                        break;
                    }
                    // If field is not required (optional), continue without skipping
                }
            } else {
                // Dependency doesn't exist
                // Check if the field is required in the schema before skipping
                let is_field_required = check_field_required(&lib.evaluated_schema, dep);

                if is_field_required {
                    requirement_not_filled = true;
                    break;
                }
            }
        }
    }
    // println!("requirement_not_filled: {}", requirement_not_filled);

    // ==========================================
    // PHASE 3: Evaluate $clear - if true, ensure table is empty
    // ==========================================
    let mut should_clear = metadata.clear_literal;
    if !should_clear {
        if let Some(logic_id) = metadata.clear_logic {
            let val = lib
                .engine
                .run_with_context(&logic_id, sandbox.data(), &empty_context)?;
            should_clear = val.as_bool().unwrap_or(false);
        }
    }

    // Initialize empty table array only when: existing table data is not an array
    let table_is_not_array = !existing_table_value
        .as_ref()
        .map_or(false, |v| v.is_array());
    if should_clear || should_skip || table_is_not_array || requirement_not_filled {
        sandbox.set(&table_pointer_path, Value::Array(Vec::new()));
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

    for plan in metadata.row_plans.iter() {
        match plan {
            RowMetadata::Static { columns } => {
                // CRITICAL: Preserve SCHEMA ORDER for static rows (match JavaScript behavior)
                let mut evaluated_row = Map::with_capacity(columns.len());

                // Create internal context for column variables
                let mut internal_context = Map::new();

                // Evaluate columns in schema order (sandboxed)
                for column in columns.iter() {
                    let value = if let Some(logic_id) = column.logic {
                        lib.engine.run_with_context(
                            &logic_id,
                            sandbox.data(),
                            &Value::Object(internal_context.clone()),
                        )?
                    } else {
                        column
                            .literal
                            .as_ref()
                            .map(|arc_val| Value::clone(arc_val))
                            .unwrap_or(Value::Null)
                    };
                    // Pre-compute string key once from Arc<str>
                    let col_name_str = column.name.as_ref().to_string();
                    // Store in internal context (column vars start with $)
                    internal_context.insert(column.var_path.as_ref().to_string(), value.clone());
                    evaluated_row.insert(col_name_str, value);
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
                // Evaluate repeat bounds in sandbox
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

                // Count existing static rows in sandbox
                let existing_row_count = sandbox
                    .get(&table_pointer_path)
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);

                // Pre-allocate all rows in sandbox (zero-copy: pre-compute string keys)
                let total_rows = (end_idx - start_idx + 1) as usize;
                let col_count = columns.len();
                // Pre-compute all column name strings once
                let col_names: Vec<String> = columns
                    .iter()
                    .map(|col| col.name.as_ref().to_string())
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

                // ========================================
                // PHASE 4: TOP TO BOTTOM (Forward Pass)
                // ========================================
                // Evaluate columns WITHOUT forward references in sandbox

                // Create internal context with $threshold
                let mut internal_context = Map::new();
                internal_context.insert("$threshold".to_string(), Value::from(end_idx));

                for iteration in start_idx..=end_idx {
                    let row_idx = (iteration - start_idx) as usize;
                    let target_idx = existing_row_count + row_idx;

                    // Set $iteration in internal context
                    internal_context.insert("$iteration".to_string(), Value::from(iteration));

                    // Evaluate normal columns in sandbox
                    for &col_idx in normal_cols.iter() {
                        let column = &columns[col_idx];
                        let value = match column.logic {
                            Some(logic_id) => lib.engine.run_with_context(
                                &logic_id,
                                sandbox.data(),
                                &Value::Object(internal_context.clone()),
                            )?,
                            None => column
                                .literal
                                .as_ref()
                                .map(|arc_val| Value::clone(arc_val))
                                .unwrap_or(Value::Null),
                        };

                        // Update table cell in sandbox
                        if let Some(row_obj) =
                            sandbox.get_table_row_mut(&table_pointer_path, target_idx)
                        {
                            if let Some(cell) = row_obj.get_mut(column.name.as_ref()) {
                                *cell = value.clone();
                            } else {
                                row_obj.insert(col_names[col_idx].clone(), value.clone());
                            }
                        }
                        // Store in internal context (column vars)
                        internal_context.insert(column.var_path.as_ref().to_string(), value);
                    }
                }
                // TODO: Implement mark_modified if needed for tracking
                // sandbox.mark_modified(&table_pointer_path);

                // ========================================
                // PHASE 5 (BACKWARD PASS):
                // Evaluate columns WITH forward references in sandbox
                // ========================================
                if !forward_cols.is_empty() {
                    let max_sweeps = 100; // Safety limit to prevent infinite loops
                    let mut scan_from_down = false;
                    let iter_count = (end_idx - start_idx + 1) as usize;

                    // Create internal context for backward pass
                    let mut internal_context = Map::new();
                    internal_context.insert("$threshold".to_string(), Value::from(end_idx));

                    // Track which columns changed in previous sweep per row
                    // This enables skipping re-evaluation of columns with unchanged dependencies
                    let mut prev_changed: Vec<Vec<bool>> =
                        vec![vec![true; forward_cols.len()]; iter_count];

                    for _sweep_num in 1..=max_sweeps {
                        let mut any_changed = false;
                        let mut curr_changed: Vec<Vec<bool>> =
                            vec![vec![false; forward_cols.len()]; iter_count];

                        for iter_offset in 0..iter_count {
                            let iteration = if scan_from_down {
                                end_idx - iter_offset as i64
                            } else {
                                start_idx + iter_offset as i64
                            };
                            let row_offset = (iteration - start_idx) as usize;
                            let target_idx = existing_row_count + row_offset;

                            // Set $iteration in internal context
                            internal_context
                                .insert("$iteration".to_string(), Value::from(iteration));

                            // Restore column values from sandbox to internal context
                            if let Some(Value::Array(table_arr)) = sandbox.get(&table_pointer_path)
                            {
                                if let Some(Value::Object(row_obj)) = table_arr.get(target_idx) {
                                    // Collect all column values into internal context
                                    for &col_idx in normal_cols.iter().chain(forward_cols.iter()) {
                                        let column = &columns[col_idx];
                                        if let Some(value) = row_obj.get(column.name.as_ref()) {
                                            internal_context.insert(
                                                column.var_path.as_ref().to_string(),
                                                value.clone(),
                                            );
                                        }
                                    }
                                }
                            }

                            // Evaluate forward columns in sandbox (with dependency-aware skipping)
                            for (fwd_idx, &col_idx) in forward_cols.iter().enumerate() {
                                let column = &columns[col_idx];

                                // Determine if we should evaluate this column
                                let mut should_evaluate = _sweep_num == 1; // Always evaluate first sweep

                                // Skip if no dependencies changed (only for non-forward-ref columns)
                                if !should_evaluate && !column.has_forward_ref {
                                    // Check intra-row column dependencies
                                    should_evaluate = column.dependencies.iter().any(|dep| {
                                        if dep.starts_with('$') {
                                            let dep_name = dep.trim_start_matches('$');
                                            // Check if dependency is in forward_cols and changed in prev sweep
                                            forward_cols.iter().enumerate().any(
                                                |(dep_fwd_idx, &dep_col_idx)| {
                                                    columns[dep_col_idx].name.as_ref() == dep_name
                                                        && prev_changed[row_offset][dep_fwd_idx]
                                                },
                                            )
                                        } else {
                                            // Non-column dependency, always re-evaluate to be safe
                                            true
                                        }
                                    });
                                } else if !should_evaluate {
                                    // For forward-ref columns, re-evaluate if anything changed
                                    should_evaluate = true;
                                }

                                if should_evaluate {
                                    let value = match column.logic {
                                        Some(logic_id) => lib.engine.run_with_context(
                                            &logic_id,
                                            sandbox.data(),
                                            &Value::Object(internal_context.clone()),
                                        )?,
                                        None => column
                                            .literal
                                            .as_ref()
                                            .map(|arc_val| Value::clone(arc_val))
                                            .unwrap_or(Value::Null),
                                    };

                                    // Write to sandbox table and update internal context
                                    if let Some(row_obj) =
                                        sandbox.get_table_row_mut(&table_pointer_path, target_idx)
                                    {
                                        if let Some(cell) = row_obj.get_mut(column.name.as_ref()) {
                                            if *cell != value {
                                                any_changed = true;
                                                curr_changed[row_offset][fwd_idx] = true;
                                                *cell = value.clone();
                                            }
                                        } else {
                                            any_changed = true;
                                            curr_changed[row_offset][fwd_idx] = true;
                                            row_obj
                                                .insert(col_names[col_idx].clone(), value.clone());
                                        }
                                    }
                                    // Update internal context with new value
                                    internal_context
                                        .insert(column.var_path.as_ref().to_string(), value);
                                }
                            }
                        }

                        scan_from_down = !scan_from_down;
                        prev_changed = curr_changed;

                        // Exit early if converged (no changes in this sweep)
                        if !any_changed {
                            break;
                        }
                    }
                }
            }
        }
    }

    // Extract result from sandbox (zero-copy: take from sandbox, no mutation of parent scope)
    let final_rows = if let Some(table_value) = sandbox.get_mut(&table_pointer_path) {
        if let Some(array) = table_value.as_array_mut() {
            mem::take(array)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Sandbox is dropped here, all temporary mutations are discarded
    // Parent scope_data remains unchanged - safe for parallel execution
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
    // Convert the dependency path to schema path
    // For fields like "/properties/field", we need to look at "/properties/field/rules/required"
    let rules_path = format!(
        "{}/rules/required",
        path_utils::dot_notation_to_schema_pointer(dep_path)
    );

    // Try to get the required rule from the schema
    if let Some(required_rule) = path_utils::get_value_by_pointer(schema, &rules_path) {
        // Check if the required rule has value=true
        if let Some(rule_obj) = required_rule.as_object() {
            if let Some(Value::Bool(is_required)) = rule_obj.get("value") {
                return *is_required;
            }
        }
        // If it's a direct boolean value
        if let Some(is_required) = required_rule.as_bool() {
            return is_required;
        }
    }

    // If no required rule found, field is optional
    false
}
