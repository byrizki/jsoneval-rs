use crate::jsoneval::eval_data::EvalData;
use crate::jsoneval::path_utils;
use crate::jsoneval::table_metadata::RowMetadata;
use crate::time_block;
use crate::JSONEval;
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
) -> Result<(std::sync::Arc<Value>, Option<indexmap::IndexSet<String>>), String> {
    let _total_start: Option<std::time::Instant> = if crate::utils::is_timing_enabled() {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let result = evaluate_table_inner(lib, eval_key, scope_data, token);
    if let Some(start) = _total_start {
        crate::utils::record_timing(&format!("[table::{}] total", eval_key), start.elapsed());
    }
    result
}

#[inline(always)]
fn is_pure_arithmetic(logic: &crate::rlogic::CompiledLogic) -> bool {
    matches!(
        logic,
        crate::rlogic::CompiledLogic::Add(_)
            | crate::rlogic::CompiledLogic::Subtract(_)
            | crate::rlogic::CompiledLogic::Multiply(_)
            | crate::rlogic::CompiledLogic::Divide(_)
            | crate::rlogic::CompiledLogic::Modulo(_, _)
            | crate::rlogic::CompiledLogic::Power(_, _)
            | crate::rlogic::CompiledLogic::Round(_, _)
            | crate::rlogic::CompiledLogic::RoundUp(_, _)
            | crate::rlogic::CompiledLogic::RoundDown(_, _)
            | crate::rlogic::CompiledLogic::Abs(_)
    )
}

fn evaluate_table_inner(
    lib: &JSONEval,
    eval_key: &str,
    scope_data: &EvalData,
    token: Option<&CancellationToken>,
) -> Result<(std::sync::Arc<Value>, Option<indexmap::IndexSet<String>>), String> {
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

    let mut external_deps = indexmap::IndexSet::new();
    let pointer_data_prefix =
        crate::jsoneval::path_utils::schema_path_to_data_pointer(&table_pointer_path).into_owned();
    let pointer_data_prefix_slash = format!("{}/", pointer_data_prefix);

    if let Some(deps) = lib.dependencies.get(eval_key) {
        for dep in deps {
            let is_params_dep = dep.contains("$params");
            let is_other_system_dep = !is_params_dep
                && !dep.contains("$context")
                && (dep.starts_with("/$") || dep.starts_with("$"));

            if is_other_system_dep {
                continue;
            }

            let dep_data_path = crate::jsoneval::path_utils::schema_path_to_data_pointer(dep);
            if dep_data_path == pointer_data_prefix
                || dep_data_path.starts_with(&pointer_data_prefix_slash)
            {
                continue;
            }

            external_deps.insert(dep.clone());
        }
    }

    if crate::utils::is_debug_cache_enabled() && external_deps.is_empty() {
        if !metadata.data_plans.is_empty() {
            eprintln!(
                "[jsoneval DEBUG] table {} has zero external_deps but \
                 non-empty data_plans — $params changes may not \
                 invalidate its cache",
                eval_key
            );
        }
    }

    if let Some(cached_result) = lib.eval_cache.check_table_cache(eval_key, &external_deps) {
        if cached_result.is_array() {
            return Ok((cached_result, None)); // Signal that we had a cache hit
        }
    }

    // PHASE 0: Evaluate $datas first.
    // Instead of writing to a sandbox, we collect overrides into `data_ctx` which
    // gets merged into ctx_value (internal_context). The evaluator checks
    // internal_context before user_data, so $datas are visible to all column logic.
    let mut data_ctx: Map<String, Value> = Map::new();
    time_block!(&format!("[table::{}] phase0 $datas", eval_key), {
        let empty_ctx = Value::Object(Map::new());
        for (name, logic, literal) in metadata.data_plans.iter() {
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
            for dep in deps.iter() {
                let is_params_dep = dep.contains("$params");
                let is_other_system_dep = !is_params_dep
                    && !dep.contains("$context")
                    && (dep.starts_with("/$") || dep.starts_with("$"));

                if is_other_system_dep || is_params_dep {
                    continue;
                }

                // Validate the dep's current value against its schema rules on-demand.
                // If the value is absent from scope_data (Null), skip validation entirely —
                // the dep belongs to a different evaluation context (e.g., a subform path
                // like /riders/prem_pay_period evaluated during main-form context). Treating
                // an absent dep as a required-rule failure causes spurious cache misses.
                let dep_value = scope_data.get_without_properties(dep);
                let dep_value = match dep_value {
                    Some(v) if *v != Value::Null => v,
                    _ => continue,
                };

                if lib.dep_fails_schema_rules(dep, dep_value, scope_data.data()) {
                    if crate::utils::is_debug_cache_enabled() {
                        println!(
                            "Table Cache MISS [table::{}] dep {} fails schema rules",
                            eval_key, dep
                        );
                    }
                    requirement_not_filled = true;
                    break;
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
        if crate::utils::is_debug_cache_enabled() {
            println!("Table Cache MISS [table::{}] should_clear={}, should_skip={}, requirement_not_filled={} (external_deps={:?})", eval_key, should_clear, should_skip, requirement_not_filled, external_deps);
        }
        return Ok((
            std::sync::Arc::new(Value::Array(Vec::new())),
            Some(external_deps),
        ));
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
                                .run_with_context(&logic_id, scope_data.data(), &ctx_value)
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
                    match lib
                        .engine
                        .run_with_context(&logic_id, scope_data.data(), &empty_ctx)
                    {
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
                    match lib
                        .engine
                        .run_with_context(&logic_id, scope_data.data(), &empty_ctx)
                    {
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

                let mut col_map = rapidhash::RapidHashMap::default();
                for (i, col) in columns.iter().enumerate() {
                    col_map.insert(col.name.as_ref().to_string(), i);
                }

                // Build base ctx with data_ctx entries + iteration slots
                let key_iteration = String::from("$iteration");
                let key_threshold = String::from("$threshold");
                let threshold_value = Value::from(end_idx);

                let mut ctx_value = Value::Object({
                    let mut m = data_ctx.clone();
                    m.insert(key_threshold.clone(), threshold_value.clone());
                    m.insert(key_iteration.clone(), Value::Null);
                    m
                });

                // Pre-resolve compiled logic references with Loop Invariant Code Motion (LICM)
                let table_no_hash = table_pointer_path.trim_start_matches('#');
                let folded_col_logics: Vec<Option<crate::rlogic::CompiledLogic>> = columns
                    .iter()
                    .map(|col| {
                        col.logic.as_ref().and_then(|id| {
                            lib.engine.get_compiled(id).map(|ast| {
                                ast.fold_table_invariants(
                                    lib.engine.evaluator(),
                                    scope_data.data(),
                                    &ctx_value,
                                    &table_pointer_path,
                                    table_no_hash,
                                )
                            })
                        })
                    })
                    .collect();
                let col_logics: Vec<Option<&crate::rlogic::CompiledLogic>> =
                    folded_col_logics.iter().map(|opt| opt.as_ref()).collect();

                let col_bytecodes: Vec<Option<crate::rlogic::TableBytecode>> = folded_col_logics
                    .iter()
                    .map(|opt| {
                        opt.as_ref().and_then(|ast| {
                            crate::rlogic::try_lower_to_bytecode(
                                ast,
                                &table_pointer_path,
                                table_no_hash,
                                &col_map,
                            )
                        })
                    })
                    .collect();

                // Pre-allocate flat cells buffer with null cells (1 single contiguous allocation)
                let mut flat_cells = vec![Value::Null; total_rows * col_count];

                // Register this table's scope on the evaluator so self-table
                // Var/Ref/ValueAt lookups resolve from flat_cells / local_rows.
                // The guard is dropped at end of this block, clearing the scope.
                let _scope_guard = lib
                    .engine
                    .enter_table_scope(table_pointer_path.clone(), &local_rows);

                lib.engine.set_table_scope_flat_cells(
                    flat_cells.as_mut_ptr(),
                    col_count,
                    total_rows,
                    existing_row_count,
                    col_map,
                );

                lib.engine.set_table_scope_threshold(end_idx);

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
                            let row_offset = (iteration - start_idx) as usize;
                            let row_idx = existing_row_count + row_offset;

                            // Update $iteration in ctx_value in-place
                            if let Value::Object(ref mut map) = ctx_value {
                                if let Some(slot) = map.get_mut(&key_iteration) {
                                    *slot = Value::from(iteration);
                                }
                            }

                            // Point get_var lookup directly to the actively evaluating cell
                            lib.engine
                                .set_table_scope_cursor(Some(row_idx), Some(iteration));

                            let row_base = row_offset * col_count;
                            let flat_cells_ptr = flat_cells.as_ptr();
                            let static_rows_ptr = &local_rows as *const Vec<Value>;

                            for &col_idx in normal_cols.iter() {
                                let column = &columns[col_idx];
                                if let Some(ref bc) = col_bytecodes[col_idx] {
                                    let num = unsafe {
                                        bc.execute(
                                            flat_cells_ptr,
                                            col_count,
                                            row_offset,
                                            existing_row_count,
                                            total_rows,
                                            iteration,
                                            static_rows_ptr,
                                            &col_names,
                                        )
                                    };
                                    if let Some(n) = num {
                                        flat_cells[row_base + col_idx] = lib.engine.f64_to_value(n);
                                        continue;
                                    }
                                }

                                let value = match col_logics[col_idx] {
                                    Some(compiled) => {
                                        if is_pure_arithmetic(compiled) {
                                            if let Ok(Some(num)) =
                                                lib.engine.run_precompiled_f64_with_context(
                                                    compiled,
                                                    scope_data.data(),
                                                    &ctx_value,
                                                )
                                            {
                                                lib.engine.f64_to_value(num)
                                            } else {
                                                lib.engine
                                                    .run_precompiled_with_context(
                                                        compiled,
                                                        scope_data.data(),
                                                        &ctx_value,
                                                    )
                                                    .unwrap_or(Value::Null)
                                            }
                                        } else {
                                            lib.engine
                                                .run_precompiled_with_context(
                                                    compiled,
                                                    scope_data.data(),
                                                    &ctx_value,
                                                )
                                                .unwrap_or(Value::Null)
                                        }
                                    }
                                    None => column
                                        .literal
                                        .as_ref()
                                        .map(|arc_val| Value::clone(arc_val))
                                        .unwrap_or(Value::Null),
                                };

                                // Write directly into flat_cells — no string hash, no IndexMap search
                                flat_cells[row_base + col_idx] = value;
                            }
                            // Reset cursor after row
                            lib.engine.set_table_scope_cursor(None, None);
                        }
                    }
                );

                // PHASE 5: BACKWARD PASS for forward-ref columns
                if !forward_cols.is_empty() {
                    let max_sweeps = 100;
                    let mut scan_from_down = true;
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

                    // Pre-compute backward dependency mappings to integer arrays preventing string sweeps
                    let table_name_only = table_pointer_path.rsplit('/').next().unwrap_or("");
                    let mut unknown_deps = vec![false; forward_cols.len()];
                    let forward_deps: Vec<Vec<usize>> = forward_cols
                        .iter()
                        .enumerate()
                        .map(|(fwd_idx, &col_idx)| {
                            let mut deps = Vec::new();
                            for dep in columns[col_idx].dependencies.iter() {
                                if dep == "$iteration" || dep == "$threshold" {
                                    continue;
                                }
                                let is_self_table = (!table_name_only.is_empty()
                                    && dep.contains(table_name_only))
                                    || dep.contains(&table_pointer_path);
                                if is_self_table {
                                    unknown_deps[fwd_idx] = true;
                                    continue;
                                }
                                if dep.starts_with('$') {
                                    let dep_name = dep.trim_start_matches('$');
                                    if let Some(&dep_fwd_idx) = forward_col_map.get(dep_name) {
                                        deps.push(dep_fwd_idx);
                                    } else if normal_col_set.contains(dep_name) {
                                        // Dependency is in normal_cols: normal columns are already evaluated in Phase 4 and invariant during Phase 5
                                        continue;
                                    } else if dep_name.starts_with("params")
                                        || dep_name.starts_with("constants")
                                        || dep_name.starts_with("datas")
                                    {
                                        // External data reference: invariant during table evaluation
                                        continue;
                                    } else {
                                        unknown_deps[fwd_idx] = true;
                                    }
                                } else {
                                    // External schema path (e.g. #/illustration/...): invariant during table evaluation
                                    continue;
                                }
                            }
                            deps
                        })
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
                        let _sweep_step_start = if crate::utils::is_timing_enabled() {
                            Some(std::time::Instant::now())
                        } else {
                            None
                        };
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

                            // Explicitly direct column resolution to local stack rows cursor and iteration
                            lib.engine
                                .set_table_scope_cursor(Some(target_idx), Some(iteration));

                            let row_base = row_offset * col_count;
                            let fwd_row_base = row_offset * forward_cols.len();
                            macro_rules! eval_col {
                                ($fwd_idx:expr, $col_idx:expr) => {{
                                    let fwd_idx = $fwd_idx;
                                    let col_idx = $col_idx;
                                    let column = &columns[col_idx];

                                    let should_evaluate = if _sweep_num == 1 {
                                        true
                                    } else if unknown_deps[fwd_idx] {
                                        true
                                    } else {
                                        let deps = &forward_deps[fwd_idx];
                                        let self_changed = curr_changed
                                            .get(fwd_row_base + fwd_idx)
                                            .copied()
                                            .unwrap_or(false);
                                        if self_changed {
                                            true
                                        } else if deps.iter().any(|&dep_fwd_idx| {
                                            curr_changed[fwd_row_base + dep_fwd_idx]
                                        }) {
                                            true
                                        } else if scan_from_down {
                                            if row_offset + 1 < iter_count {
                                                let next_offset =
                                                    (row_offset + 1) * forward_cols.len();
                                                column.has_forward_ref
                                                    && deps.iter().any(|&dep_fwd_idx| {
                                                        curr_changed[next_offset + dep_fwd_idx]
                                                    })
                                            } else {
                                                false
                                            }
                                        } else {
                                            if row_offset > 0 {
                                                let prev_offset =
                                                    (row_offset - 1) * forward_cols.len();
                                                !column.has_forward_ref
                                                    && deps.iter().any(|&dep_fwd_idx| {
                                                        curr_changed[prev_offset + dep_fwd_idx]
                                                    })
                                            } else {
                                                false
                                            }
                                        }
                                    };

                                    if should_evaluate {
                                        let value = match col_logics[col_idx] {
                                            Some(compiled) => {
                                                if is_pure_arithmetic(compiled) {
                                                    if let Ok(Some(num)) =
                                                        lib.engine.run_precompiled_f64_with_context(
                                                            compiled,
                                                            scope_data.data(),
                                                            &ctx_value,
                                                        )
                                                    {
                                                        lib.engine.f64_to_value(num)
                                                    } else {
                                                        lib.engine
                                                            .run_precompiled_with_context(
                                                                compiled,
                                                                scope_data.data(),
                                                                &ctx_value,
                                                            )
                                                            .unwrap_or(Value::Null)
                                                    }
                                                } else {
                                                    lib.engine
                                                        .run_precompiled_with_context(
                                                            compiled,
                                                            scope_data.data(),
                                                            &ctx_value,
                                                        )
                                                        .unwrap_or(Value::Null)
                                                }
                                            }
                                            None => column
                                                .literal
                                                .as_ref()
                                                .map(|arc_val| Value::clone(arc_val))
                                                .unwrap_or(Value::Null),
                                        };

                                        // Write directly to flat_cells
                                        let cell_idx = row_base + col_idx;
                                        if flat_cells[cell_idx] != value {
                                            any_changed = true;
                                            curr_changed[fwd_row_base + fwd_idx] = true;
                                            flat_cells[cell_idx] = value;
                                        }
                                    }
                                }};
                            }

                            for (fwd_idx, &col_idx) in forward_cols.iter().enumerate() {
                                eval_col!(fwd_idx, col_idx);
                            }
                        }

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

                // Assemble evaluated rows into local_rows in a single final pass
                local_rows.reserve(total_rows);
                if total_rows > 0 {
                    let mut prototype_map = Map::with_capacity(col_count);
                    for name in &col_names {
                        prototype_map.insert(name.clone(), Value::Null);
                    }

                    for r in 0..total_rows {
                        let mut row_map = prototype_map.clone();
                        let row_offset = r * col_count;
                        for (slot, cell) in row_map
                            .values_mut()
                            .zip(&mut flat_cells[row_offset..row_offset + col_count])
                        {
                            *slot = std::mem::replace(cell, Value::Null);
                        }
                        local_rows.push(Value::Object(row_map));
                    }
                }

                // _scope_guard dropped here → TableScope cleared on evaluator
            }
        }
    }

    Ok((
        std::sync::Arc::new(Value::Array(local_rows)),
        Some(external_deps),
    ))
}
