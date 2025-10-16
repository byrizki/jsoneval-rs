use indexmap::{IndexMap, IndexSet};
use serde_json::{Map, Value};
use std::sync::Arc;

use crate::{topo_sort, JSONEval, LogicId, RLogic, path_utils};
use crate::table_metadata::{ColumnMetadata, RepeatBoundMetadata, RowMetadata, TableMetadata};

pub fn parse_schema(lib: &mut JSONEval) -> Result<(), String> {
    /// Single-pass schema walker that collects everything
    fn walk(
        value: &Value,
        path: &str,
        engine: &mut RLogic,
        evaluations: &mut IndexMap<String, LogicId>,
        tables: &mut IndexMap<String, Value>,
        deps: &mut IndexMap<String, IndexSet<String>>,
        value_fields: &mut Vec<String>,
        layout_paths: &mut Vec<String>,
        dependents: &mut IndexMap<String, Vec<crate::DependentItem>>,
    ) -> Result<(), String> {
        match value {
            Value::Object(map) => {
                // Check for $evaluation
                if let Some(evaluation) = map.get("$evaluation") {
                    let key = path.to_string();
                    let logic_value = evaluation.get("logic").unwrap_or(evaluation);
                    let logic_id = engine
                        .compile(logic_value)
                        .map_err(|e| format!("failed to compile evaluation at {key}: {e}"))?;
                    evaluations.insert(key.clone(), logic_id);

                    // Collect dependencies with smart table inheritance
                    // Normalize all dependencies to JSON pointer format to avoid duplicates
                    let mut refs: IndexSet<String> = engine
                        .get_referenced_vars(&logic_id)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|dep| path_utils::normalize_to_json_pointer(&dep))
                        .collect();
                    let mut extra_refs = IndexSet::new();
                    collect_refs(logic_value, &mut extra_refs);
                    if !extra_refs.is_empty() {
                        refs.extend(extra_refs.into_iter());
                    }

                    // For table dependencies, inherit parent table path instead of individual rows
                    let refs: IndexSet<String> = refs
                        .into_iter()
                        .filter_map(|dep| {
                            // If dependency is a table row (contains /$table/), inherit table path
                            if let Some(table_idx) = dep.find("/$table/") {
                                let table_path = &dep[..table_idx];
                                Some(table_path.to_string())
                            } else {
                                Some(dep)
                            }
                        })
                        .collect();

                    if !refs.is_empty() {
                        deps.insert(key.clone(), refs);
                    }
                }

                // Check for $table
                if let Some(table) = map.get("$table") {
                    let key = path.to_string();

                    let rows = table.clone();
                    let datas = map
                        .get("$datas")
                        .cloned()
                        .unwrap_or_else(|| Value::Array(vec![]));
                    let skip = map.get("$skip").cloned().unwrap_or(Value::Bool(false));
                    let clear = map.get("$clear").cloned().unwrap_or(Value::Bool(false));

                    let mut table_entry = Map::new();
                    table_entry.insert("rows".to_string(), rows);
                    table_entry.insert("datas".to_string(), datas);
                    table_entry.insert("skip".to_string(), skip);
                    table_entry.insert("clear".to_string(), clear);

                    tables.insert(key, Value::Object(table_entry));
                }

                // Check for $layout with elements
                if let Some(layout_obj) = map.get("$layout") {
                    if let Some(Value::Array(_)) = layout_obj.get("elements") {
                        let layout_elements_path = format!("{}/$layout/elements", path);
                        layout_paths.push(layout_elements_path);
                    }
                }

                // Check for dependents array
                if let Some(Value::Array(dependents_arr)) = map.get("dependents") {
                    let mut dependent_items = Vec::new();
                    
                    for (dep_idx, dep_item) in dependents_arr.iter().enumerate() {
                        if let Value::Object(dep_obj) = dep_item {
                            if let Some(Value::String(ref_path)) = dep_obj.get("$ref") {
                                // Process clear - compile if it's an $evaluation
                                let clear_val = if let Some(clear) = dep_obj.get("clear") {
                                    if let Value::Object(clear_obj) = clear {
                                        if clear_obj.contains_key("$evaluation") {
                                            // Compile and store the evaluation
                                            let clear_eval = clear_obj.get("$evaluation").unwrap();
                                            let clear_key = format!("{}/dependents/{}/clear", path, dep_idx);
                                            let logic_id = engine.compile(clear_eval)
                                                .map_err(|e| format!("Failed to compile dependent clear at {}: {}", clear_key, e))?;
                                            evaluations.insert(clear_key.clone(), logic_id);
                                            // Replace with eval key reference
                                            Some(Value::String(clear_key))
                                        } else {
                                            Some(clear.clone())
                                        }
                                    } else {
                                        Some(clear.clone())
                                    }
                                } else {
                                    None
                                };
                                
                                // Process value - compile if it's an $evaluation
                                let value_val = if let Some(value) = dep_obj.get("value") {
                                    if let Value::Object(value_obj) = value {
                                        if value_obj.contains_key("$evaluation") {
                                            // Compile and store the evaluation
                                            let value_eval = value_obj.get("$evaluation").unwrap();
                                            let value_key = format!("{}/dependents/{}/value", path, dep_idx);
                                            let logic_id = engine.compile(value_eval)
                                                .map_err(|e| format!("Failed to compile dependent value at {}: {}", value_key, e))?;
                                            evaluations.insert(value_key.clone(), logic_id);
                                            // Replace with eval key reference
                                            Some(Value::String(value_key))
                                        } else {
                                            Some(value.clone())
                                        }
                                    } else {
                                        Some(value.clone())
                                    }
                                } else {
                                    None
                                };
                                
                                dependent_items.push(crate::DependentItem {
                                    ref_path: ref_path.clone(),
                                    clear: clear_val,
                                    value: value_val,
                                });
                            }
                        }
                    }
                    
                    if !dependent_items.is_empty() {
                        dependents.insert(path.to_string(), dependent_items);
                    }
                }

                // Recurse into children
                Ok(for (key, val) in map {
                    // Skip special evaluation and dependents keys from recursion (already processed above)
                    if key == "$evaluation" || key == "dependents" {
                        continue;
                    }
                    
                    let next_path = if path == "#" {
                        format!("#/{key}")
                    } else {
                        format!("{path}/{key}")
                    };
                    
                    // Check if this is a "value" field
                    if key == "value" && !next_path.starts_with("#/$") && !next_path.contains("/$layout/") && !next_path.contains("/items/") && !next_path.contains("/options/") && !next_path.contains("/dependents/") && !next_path.contains("/rules/") {
                        value_fields.push(next_path.clone());
                    }
                    
                    // Recurse into all children (including $ keys like $table, $datas, etc.)
                    walk(val, &next_path, engine, evaluations, tables, deps, value_fields, layout_paths, dependents)?;
                })
            }
            Value::Array(arr) => Ok(for (index, item) in arr.iter().enumerate() {
                let next_path = if path == "#" {
                    format!("#/{index}")
                } else {
                    format!("{path}/{index}")
                };
                walk(item, &next_path, engine, evaluations, tables, deps, value_fields, layout_paths, dependents)?;
            }),
            _ => Ok(()),
        }
    }

    fn collect_refs(value: &Value, refs: &mut IndexSet<String>) {
        match value {
            Value::Object(map) => {
                if let Some(path) = map.get("$ref").and_then(Value::as_str) {
                    refs.insert(path_utils::normalize_to_json_pointer(path));
                }
                if let Some(path) = map.get("ref").and_then(Value::as_str) {
                    refs.insert(path_utils::normalize_to_json_pointer(path));
                }
                if let Some(var_val) = map.get("var") {
                    match var_val {
                        Value::String(s) => {
                            refs.insert(s.clone());
                        }
                        Value::Array(arr) => {
                            if let Some(path) = arr.get(0).and_then(Value::as_str) {
                                refs.insert(path.to_string());
                            }
                        }
                        _ => {}
                    }
                }
                for val in map.values() {
                    collect_refs(val, refs);
                }
            }
            Value::Array(arr) => {
                for val in arr {
                    collect_refs(val, refs);
                }
            }
            _ => {}
        }
    }

    // Use centralized path normalization from path_utils
    // This ensures consistent $ref/var handling across the entire pipeline

    // Single-pass collection: walk schema once and collect everything
    let mut evaluations = IndexMap::new();
    let mut tables = IndexMap::new();
    let mut dependencies = IndexMap::new();
    let mut value_fields = Vec::new();
    let mut layout_paths = Vec::new();
    let mut dependents_evaluations = IndexMap::new();
    
    walk(
        &lib.schema,
        "#",
        &mut lib.engine,
        &mut evaluations,
        &mut tables,
        &mut dependencies,
        &mut value_fields,
        &mut layout_paths,
        &mut dependents_evaluations,
    )?;
    
    lib.evaluations = evaluations;
    lib.tables = tables;
    lib.dependencies = dependencies;
    lib.layout_paths = layout_paths;
    lib.dependents_evaluations = dependents_evaluations;
    
    // Collect table-level dependencies by aggregating all column dependencies
    collect_table_dependencies(lib);
    
    lib.sorted_evaluations = topo_sort::topological_sort(lib)?;
    
    // Categorize evaluations for result handling
    categorize_evaluations(lib);
    
    // Process collected value fields
    process_value_fields(lib, value_fields);
    
    // Pre-compile all table metadata for zero-copy evaluation
    build_table_metadata(lib)?;
    
    Ok(())
}

/// Collect dependencies for tables by aggregating all column/cell dependencies
fn collect_table_dependencies(lib: &mut JSONEval) {
    for (table_key, _) in &lib.tables {
        let mut table_deps = IndexSet::new();
        
        // Collect dependencies from all evaluations that belong to this table
        for (eval_key, deps) in &lib.dependencies {
            // Check if this evaluation is within the table
            if eval_key.starts_with(table_key) && eval_key != table_key {
                // Add all dependencies from table cells/columns
                for dep in deps {
                    // Filter out self-references and internal table paths
                    if !dep.starts_with(table_key) {
                        table_deps.insert(dep.clone());
                    }
                }
            }
        }
        
        // Store aggregated dependencies for the table
        if !table_deps.is_empty() {
            lib.dependencies.insert(table_key.clone(), table_deps);
        }
    }
}

/// Categorize evaluations for different result handling
fn categorize_evaluations(lib: &mut JSONEval) {
    // Collect all evaluation keys that are in sorted_evaluations (batches)
    let batched_keys: IndexSet<String> = lib.sorted_evaluations
        .iter()
        .flatten()
        .cloned()
        .collect();
    
    // Find evaluations NOT in batches and categorize them
    for eval_key in lib.evaluations.keys() {
        // Skip if already in sorted_evaluations batches
        if batched_keys.contains(eval_key) {
            continue;
        }
        
        // Skip table-related evaluations
        if lib.tables.iter().any(|(key, _)| eval_key.starts_with(key)) {
            continue;
        }

        // Categorize based on path patterns
        if eval_key.contains("/rules/") {
            lib.rules_evaluations.push(eval_key.clone());
        } else if !eval_key.contains("/dependents/") {
            // Don't add dependents to others_evaluations
            lib.others_evaluations.push(eval_key.clone());
        }
    }
}

/// Process collected value fields and add non-duplicate, non-table, non-dependent ones
fn process_value_fields(lib: &mut JSONEval, value_fields: Vec<String>) {
    for path in value_fields {
        // Skip if already collected from evaluations in categorize_evaluations
        if lib.value_evaluations.contains(&path) {
            continue;
        }
        
        // Skip table-related paths
        if lib.tables.iter().any(|(key, _)| path.starts_with(key)) {
            continue;
        }
        
        lib.value_evaluations.push(path);
    }
}

/// Build pre-compiled table metadata at parse time (moves heavy operations from evaluation)
fn build_table_metadata(lib: &mut JSONEval) -> Result<(), String> {
    let mut table_metadata = IndexMap::new();
    
    for (eval_key, table) in &lib.tables {
        let metadata = compile_table_metadata(lib, eval_key, table)?;
        table_metadata.insert(eval_key.clone(), metadata);
    }
    
    lib.table_metadata = table_metadata;
    Ok(())
}

/// Compile table metadata at parse time (zero-copy design)
fn compile_table_metadata(
    lib: &JSONEval,
    eval_key: &str,
    table: &Value,
) -> Result<TableMetadata, String> {
    let rows = table
        .get("rows")
        .and_then(|v| v.as_array())
        .ok_or("table missing rows")?;
    let empty_datas = Vec::new();
    let datas = table
        .get("datas")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_datas);

    // Pre-compile data plans with Arc sharing
    let mut data_plans = Vec::with_capacity(datas.len());
    for (idx, entry) in datas.iter().enumerate() {
        let Some(name) = entry.get("name").and_then(|v| v.as_str()) else { continue };
        let logic_path = format!("{eval_key}/$datas/{idx}/data");
        let logic = lib.evaluations.get(&logic_path).copied();
        let literal = entry.get("data").map(|v| Arc::new(v.clone()));
        data_plans.push((Arc::from(name), logic, literal));
    }

    // Pre-compile row plans with dependency analysis
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

                let start_literal = Arc::new(repeat_arr.get(0).cloned().unwrap_or(Value::Null));
                let end_literal = Arc::new(repeat_arr.get(1).cloned().unwrap_or(Value::Null));

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
                        
                        // Extract dependencies ONCE at parse time (not during evaluation)
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
                        
                        columns.push(ColumnMetadata::new(col_name, logic, literal, dependencies, has_forward_ref));
                    }

                    // Pre-compute forward column propagation (transitive closure)
                    let (forward_cols, normal_cols) = compute_column_partitions(&columns);

                    row_plans.push(RowMetadata::Repeat {
                        start: RepeatBoundMetadata {
                            logic: start_logic,
                            literal: start_literal,
                        },
                        end: RepeatBoundMetadata {
                            logic: end_logic,
                            literal: end_literal,
                        },
                        columns: columns.into(),
                        forward_cols: forward_cols.into(),
                        normal_cols: normal_cols.into(),
                    });
                    continue;
                }
            }
        }

        // Static row
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
            
            // Extract dependencies ONCE at parse time
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
            
            columns.push(ColumnMetadata::new(col_name, logic, literal, dependencies, has_forward_ref));
        }
        row_plans.push(RowMetadata::Static { columns: columns.into() });
    }

    // Pre-compile skip/clear logic
    let skip_logic = lib.evaluations.get(&format!("{eval_key}/$skip")).copied();
    let skip_literal = table.get("skip").and_then(Value::as_bool).unwrap_or(false);
    let clear_logic = lib.evaluations.get(&format!("{eval_key}/$clear")).copied();
    let clear_literal = table.get("clear").and_then(Value::as_bool).unwrap_or(false);

    Ok(TableMetadata {
        data_plans: data_plans.into(),
        row_plans: row_plans.into(),
        skip_logic,
        skip_literal,
        clear_logic,
        clear_literal,
    })
}

/// Compute forward/normal column partitions with transitive closure
fn compute_column_partitions(columns: &[ColumnMetadata]) -> (Vec<usize>, Vec<usize>) {
    use std::collections::HashSet;
    
    // Build set of all forward-referencing column names (direct + transitive)
    let mut fwd_cols: HashSet<&str> = columns.iter()
        .enumerate()
        .filter(|(_, c)| c.has_forward_ref)
        .map(|(_, c)| c.name.as_ref())
        .collect();
    
    let mut changed = true;
    while changed {
        changed = false;
        for col in columns.iter() {
            if !fwd_cols.contains(col.name.as_ref()) {
                // Check if this column depends on any forward-referencing column
                for dep in col.dependencies.iter() {
                    let dep_name = dep.trim_start_matches('$');
                    if fwd_cols.contains(dep_name) {
                        fwd_cols.insert(col.name.as_ref());
                        changed = true;
                        break;
                    }
                }
            }
        }
    }

    // Separate into forward and normal indices
    let mut forward_indices = Vec::new();
    let mut normal_indices = Vec::new();
    
    for (idx, col) in columns.iter().enumerate() {
        if fwd_cols.contains(col.name.as_ref()) {
            forward_indices.push(idx);
        } else {
            normal_indices.push(idx);
        }
    }
    
    (forward_indices, normal_indices)
}

