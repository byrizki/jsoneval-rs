use indexmap::{IndexMap, IndexSet};
use serde_json::{Map, Value};

use crate::{topo_sort, JSONEval, LogicId, RLogic};

pub fn parse_schema(lib: &mut JSONEval) -> Result<(), String> {
    fn walk(
        value: &Value,
        path: &str,
        engine: &mut RLogic,
        evaluations: &mut IndexMap<String, LogicId>,
        tables: &mut IndexMap<String, Value>,
        deps: &mut IndexMap<String, IndexSet<String>>,
    ) -> Result<(), String> {
        match value {
            Value::Object(map) => {
                if let Some(evaluation) = map.get("$evaluation") {
                    let key = path.to_string();
                    let logic_value = evaluation.get("logic").unwrap_or(evaluation);
                    let logic_id = engine
                        .compile(logic_value)
                        .map_err(|e| format!("failed to compile evaluation at {key}: {e}"))?;
                    evaluations.insert(key.clone(), logic_id);

                    // Collect dependencies with smart table inheritance
                    let mut refs: IndexSet<String> = engine
                        .get_referenced_vars(&logic_id)
                        .unwrap_or_default()
                        .into_iter()
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

                Ok(for (key, val) in map {
                    if key == "$evaluation" {
                        continue;
                    }
                    let next_path = if path == "#" {
                        format!("#/{key}")
                    } else {
                        format!("{path}/{key}")
                    };
                    walk(val, &next_path, engine, evaluations, tables, deps)?;
                })
            }
            Value::Array(arr) => Ok(for (index, item) in arr.iter().enumerate() {
                let next_path = if path == "#" {
                    format!("#/{index}")
                } else {
                    format!("{path}/{index}")
                };
                walk(item, &next_path, engine, evaluations, tables, deps)?;
            }),
            _ => Ok(()),
        }
    }

    fn collect_refs(value: &Value, refs: &mut IndexSet<String>) {
        match value {
            Value::Object(map) => {
                if let Some(path) = map.get("$ref").and_then(Value::as_str) {
                    refs.insert(normalize_ref_path(path));
                }
                if let Some(path) = map.get("ref").and_then(Value::as_str) {
                    refs.insert(normalize_ref_path(path));
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

    fn normalize_ref_path(path: &str) -> String {
        let mut normalized = path.to_string();

        if normalized.starts_with("#/") {
            normalized = normalized[2..].to_string();
        } else if normalized.starts_with('/') {
            normalized = normalized[1..].to_string();
        }

        normalized = normalized.replace('/', ".");
        normalized = normalized.replace("properties.", "");
        normalized = normalized.replace(".properties.", ".");

        while normalized.contains("..") {
            normalized = normalized.replace("..", ".");
        }

        normalized.trim_matches('.').to_string()
    }

    let mut evaluations = IndexMap::new();
    let mut tables = IndexMap::new();
    let mut dependencies = IndexMap::new();
    walk(
        &lib.schema,
        "#",
        &mut lib.engine,
        &mut evaluations,
        &mut tables,
        &mut dependencies,
    )?;
    lib.evaluations = evaluations;
    lib.tables = tables;
    lib.dependencies = dependencies;
    lib.sorted_evaluations = topo_sort::topological_sort(lib)?;
    Ok(())
}
