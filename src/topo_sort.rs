use indexmap::{IndexMap, IndexSet};

use crate::JSONEval;

pub fn topological_sort(lib: &JSONEval) -> Result<IndexSet<String>, String> {
    let mut sorted = IndexSet::new();
    let mut visited = IndexSet::new();
    let mut visiting = IndexSet::new();

    // Filter evaluations to exclude layout, rules, config, dependents, options, condition, value
    let filtered_evaluations: IndexMap<String, IndexSet<String>> = lib
        .evaluations
        .keys()
        .filter(|key| {
            !key.contains("/dependents/")
                && !key.contains("/rules/")
                && !key.contains("/options/")
                && !key.contains("/condition/")
                && !key.contains("/$layout/")
                && !key.contains("/config/")
                && !key.contains("/items/")
        })
        .map(|key| {
            let deps = lib.dependencies.get(key).cloned().unwrap_or_default();
            (key.clone(), deps)
        })
        .collect();

    // Group table evaluations and merge dependencies
    let mut table_groups: IndexMap<String, IndexSet<String>> = IndexMap::new();
    let mut evaluation_to_table: IndexMap<String, String> = IndexMap::new();

    // First pass: identify all table paths from $table keys
    let mut table_paths: IndexSet<String> = IndexSet::new();
    for table_key in lib.tables.keys() {
        // Extract table path by removing "/$table" suffix
        let table_path = table_key.to_string();
        table_paths.insert(table_path);
    }

    // Create a mapping of normalized names to table paths
    let mut normalized_to_table: IndexMap<String, String> = IndexMap::new();
    for tp in &table_paths {
        // Extract the last segment (table name) for matching
        if let Some(last_segment) = tp.rsplit('/').next() {
            normalized_to_table.insert(last_segment.to_string(), tp.clone());
        }
    }

    // Create a mapping from dotted paths to evaluation keys for dependency resolution
    let mut dotted_to_eval: IndexMap<String, String> = IndexMap::new();
    for eval_key in filtered_evaluations.keys() {
        // Convert #/$params/constants/DEATH_SA/$evaluation to $params.constants.DEATH_SA
        if let Some(stripped) = eval_key.strip_prefix("#/") {
            let path_part = &stripped[..];
            let dotted = path_part.replace('/', ".");
            // Don't add $ prefix if it already starts with $
            dotted_to_eval.insert(dotted.clone(), eval_key.clone());
        }
    }

    // Second pass: group ALL evaluations by table and merge dependencies
    // Process ALL evaluations (not just filtered ones) to capture table dependencies
    for (eval_key, deps) in lib.evaluations.keys().map(|k| {
        let deps = lib.dependencies.get(k).cloned().unwrap_or_default();
        (k, deps)
    }) {
        // Find which table this evaluation belongs to
        let table_path_opt = table_paths
            .iter()
            .find(|tp| eval_key.starts_with(tp.as_str()));

        if let Some(table_path) = table_path_opt {
            evaluation_to_table.insert(eval_key.clone(), table_path.clone());

            // Normalize dependencies to table paths where applicable
            let normalized_deps: IndexSet<String> = deps
                .iter()
                .filter_map(|dep| {
                    // Ignore self column dependencies (starts with $ and no dot/slash)
                    if dep.starts_with('$') && !dep.contains('.') && !dep.contains('/') {
                        return None;
                    }

                    // Check if dependency is a dotted path that maps to an evaluation
                    if let Some(eval_key) = dotted_to_eval.get(dep) {
                        return Some(eval_key.clone());
                    }

                    // Check if dependency references another table path (flexible matching)
                    for tp in &table_paths {
                        let tp_str = tp.as_str();
                        let tp_with_slash = format!("{}/", tp_str);

                        // Match if:
                        // 1. dep equals table path exactly (for static tables)
                        // 2. dep starts with table path (for sub-fields like table.0.field)
                        if tp_str != table_path.as_str() {
                            if dep == tp_str || dep.starts_with(&tp_with_slash) {
                                return Some(tp.clone());
                            }
                        }
                    }

                    // Check if dependency matches a normalized table name
                    if let Some(target_table) = normalized_to_table.get(dep) {
                        if target_table != table_path {
                            return Some(target_table.clone());
                        }
                    }

                    // Keep non-table dependencies as-is (but not self-table deps)
                    let table_path_with_slash = format!("{}/", table_path.as_str());
                    if !dep.starts_with(table_path.as_str())
                        && !dep.starts_with(&table_path_with_slash)
                    {
                        Some(dep.clone())
                    } else {
                        None
                    }
                })
                .collect();

            table_groups
                .entry(table_path.clone())
                .or_insert_with(IndexSet::new)
                .extend(normalized_deps);
        }
    }

    // Create a unified graph and resolve dotted dependencies in table groups
    let mut unified_graph: IndexMap<String, IndexSet<String>> = IndexMap::new();

    // Add table groups with resolved dependencies
    for (table_path, deps) in &table_groups {
        let resolved_deps: IndexSet<String> = deps
            .iter()
            .filter_map(|dep| {
                // Try to resolve dotted path to evaluation key
                if let Some(eval_key) = dotted_to_eval.get(dep) {
                    Some(eval_key.clone())
                } else {
                    // Keep as-is if not resolvable
                    Some(dep.clone())
                }
            })
            .collect();
        unified_graph.insert(table_path.clone(), resolved_deps);
    }

    // Add non-table evaluations to the unified graph
    for (eval_key, deps) in &filtered_evaluations {
        if !evaluation_to_table.contains_key(eval_key) {
            // Normalize dependencies for non-table evaluations
            let mut normalized_deps: IndexSet<String> = IndexSet::new();
            
            for dep in deps {
                // Check if dependency is a dotted path that maps to an evaluation
                if let Some(eval_key) = dotted_to_eval.get(dep) {
                    normalized_deps.insert(eval_key.clone());
                    continue;
                }

                // Check if dependency references a table/array path
                let mut found_table = false;
                for tp in &table_paths {
                    let tp_str = tp.as_str();
                    let tp_with_slash = format!("{}/", tp_str);

                    // Match if:
                    // 1. dep equals table path exactly (for static tables/arrays)
                    // 2. dep starts with table path/ (for sub-fields)
                    if dep == tp_str || dep.starts_with(&tp_with_slash) {
                        normalized_deps.insert(tp.clone());
                        found_table = true;
                        break;
                    }
                }
                
                if found_table {
                    continue;
                }
                
                // CRITICAL FIX: Check if dependency is a static array with evaluated fields
                // e.g., $params.others.RIDER_FIRST_PREM_PER_PAY_TABLE
                // Should expand to include all evaluated fields like /0/premi, /1/premi, etc.
                let dep_as_eval_prefix = format!("#/{}", dep.replace('.', "/"));
                let has_field_evaluations = lib.evaluations.keys().any(|k| {
                    k.starts_with(&dep_as_eval_prefix) 
                    && k.len() > dep_as_eval_prefix.len()
                    && k[dep_as_eval_prefix.len()..].starts_with('/')
                });
                
                if has_field_evaluations {
                    // Add all field evaluations as dependencies
                    for field_eval_key in lib.evaluations.keys() {
                        if field_eval_key.starts_with(&dep_as_eval_prefix) 
                            && field_eval_key.len() > dep_as_eval_prefix.len()
                            && field_eval_key[dep_as_eval_prefix.len()..].starts_with('/') {
                            normalized_deps.insert(field_eval_key.clone());
                        }
                    }
                } else {
                    normalized_deps.insert(dep.clone());
                }
            }

            unified_graph.insert(eval_key.clone(), normalized_deps);
        }
    }

    // Process all nodes in the unified graph
    for node in unified_graph.keys() {
        if !visited.contains(node) {
            let deps = unified_graph.get(node).cloned().unwrap_or_default();
            visit_node(
                lib,
                node,
                &deps,
                &unified_graph,
                &mut visited,
                &mut visiting,
                &mut sorted,
            )?;
        }
    }

    Ok(sorted)
}

pub fn visit_node(
    lib: &JSONEval,
    node: &str,
    deps: &IndexSet<String>,
    graph: &IndexMap<String, IndexSet<String>>,
    visited: &mut IndexSet<String>,
    visiting: &mut IndexSet<String>,
    sorted: &mut IndexSet<String>,
) -> Result<(), String> {
    if visiting.contains(node) {
        return Err(format!("Circular dependency detected involving: {}", node));
    }

    if visited.contains(node) {
        return Ok(());
    }

    visiting.insert(node.to_string());

    for dep in deps {
        if let Some(dep_deps) = graph.get(dep) {
            visit_node(lib, dep, dep_deps, graph, visited, visiting, sorted)?;
        }
    }

    visiting.swap_remove(node);
    visited.insert(node.to_string());
    sorted.insert(node.to_string());

    Ok(())
}
