use crate::jsoneval::path_utils;
use crate::topo_sort::common::{collect_transitive_deps, compute_parallel_batches};
use crate::ParsedSchema;
/// Topological sorting for ParsedSchema
use indexmap::{IndexMap, IndexSet};

pub fn topological_sort_parsed(parsed: &ParsedSchema) -> Result<Vec<Vec<String>>, String> {
    let mut sorted = IndexSet::new();
    let mut visited = IndexSet::new();
    let mut visiting = IndexSet::new();

    // Filter evaluations to exclude layout, rules, config, dependents, options, condition, value
    let filtered_evaluations: IndexMap<String, IndexSet<String>> = parsed
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
                && !key.ends_with("/options")
                && !key.ends_with("/value")
                && (key.starts_with("#/$") && !key.contains("/value/"))
        })
        .map(|key| {
            let deps = parsed.dependencies.get(key).cloned().unwrap_or_default();
            (key.clone(), deps)
        })
        .collect();

    // Group table evaluations and merge dependencies
    let mut table_groups: IndexMap<String, IndexSet<String>> = IndexMap::new();
    let mut evaluation_to_table: IndexMap<String, String> = IndexMap::new();

    // First pass: identify all table paths from $table keys
    let mut table_paths: IndexSet<String> = IndexSet::new();
    for table_key in parsed.tables.keys() {
        let table_path = table_key.to_string();
        table_paths.insert(table_path);
    }

    // Create a mapping of normalized names to table paths
    let mut normalized_to_table: IndexMap<String, String> = IndexMap::new();
    for tp in &table_paths {
        if let Some(last_segment) = tp.rsplit('/').next() {
            normalized_to_table.insert(last_segment.to_string(), tp.clone());
        }
    }

    // Create a mapping from JSON pointer paths to evaluation keys
    let mut pointer_to_eval: IndexMap<String, String> = IndexMap::new();
    for eval_key in filtered_evaluations.keys() {
        let pointer = path_utils::normalize_to_json_pointer(eval_key);
        pointer_to_eval.insert(pointer, eval_key.clone());
    }

    for table_path in &table_paths {
        let pointer = path_utils::normalize_to_json_pointer(table_path);
        pointer_to_eval.insert(pointer, table_path.clone());
    }

    // Second pass: group evaluations by table and merge dependencies
    for (eval_key, deps) in parsed.evaluations.keys().map(|k| {
        let deps = parsed.dependencies.get(k).cloned().unwrap_or_default();
        (k, deps)
    }) {
        let table_path_opt = table_paths
            .iter()
            .filter(|tp| eval_key.starts_with(tp.as_str()))
            .max_by_key(|tp| tp.len());

        if let Some(table_path) = table_path_opt {
            evaluation_to_table.insert(eval_key.clone(), table_path.clone());

            let normalized_deps: IndexSet<String> = deps
                .iter()
                .filter_map(|dep| {
                    if dep.starts_with('$') && !dep.contains('.') && !dep.contains('/') {
                        return None;
                    }

                    if let Some(eval_key) = pointer_to_eval.get(dep) {
                        return Some(eval_key.clone());
                    }

                    for tp in &table_paths {
                        let tp_str = tp.as_str();
                        let tp_with_slash = format!("{}/", tp_str);
                        if tp_str != table_path.as_str() {
                            if dep == tp_str || dep.starts_with(&tp_with_slash) {
                                return Some(tp.clone());
                            }
                        }
                    }

                    if let Some(target_table) = normalized_to_table.get(dep) {
                        if target_table != table_path {
                            return Some(target_table.clone());
                        }
                    }

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

    let mut unified_graph: IndexMap<String, IndexSet<String>> = IndexMap::new();

    for (table_path, deps) in &table_groups {
        let resolved_deps: IndexSet<String> = deps
            .iter()
            .filter_map(|dep| {
                if dep == table_path {
                    return None;
                }
                if let Some(eval_key) = pointer_to_eval.get(dep) {
                    if eval_key == table_path {
                        return None;
                    }
                    Some(eval_key.clone())
                } else {
                    Some(dep.clone())
                }
            })
            .collect();
        unified_graph.insert(table_path.clone(), resolved_deps);
    }

    for (eval_key, deps) in &filtered_evaluations {
        if !evaluation_to_table.contains_key(eval_key) {
            let mut normalized_deps: IndexSet<String> = IndexSet::new();

            for dep in deps {
                if let Some(eval_key) = pointer_to_eval.get(dep) {
                    normalized_deps.insert(eval_key.clone());
                    continue;
                }

                let mut found_table = false;
                for tp in &table_paths {
                    let tp_str = tp.as_str();
                    let tp_with_slash = format!("{}/", tp_str);
                    if dep == tp_str || dep.starts_with(&tp_with_slash) {
                        normalized_deps.insert(tp.clone());
                        found_table = true;
                        break;
                    }
                }

                if found_table {
                    continue;
                }

                let dep_as_pointer = path_utils::normalize_to_json_pointer(dep);
                let dep_as_eval_prefix = format!("#{}", dep_as_pointer);
                let has_field_evaluations = parsed.evaluations.keys().any(|k| {
                    k.starts_with(&dep_as_eval_prefix)
                        && k.len() > dep_as_eval_prefix.len()
                        && k[dep_as_eval_prefix.len()..].starts_with('/')
                });

                if has_field_evaluations {
                    for field_eval_key in parsed.evaluations.keys() {
                        if field_eval_key.starts_with(&dep_as_eval_prefix)
                            && field_eval_key.len() > dep_as_eval_prefix.len()
                            && field_eval_key[dep_as_eval_prefix.len()..].starts_with('/')
                        {
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

    let mut table_dependencies = IndexSet::new();
    for table_path in &table_paths {
        if let Some(deps) = unified_graph.get(table_path) {
            collect_transitive_deps(deps, &unified_graph, &table_paths, &mut table_dependencies);
        }
    }

    let mut expanded = true;
    while expanded {
        expanded = false;
        let current_deps: Vec<String> = table_dependencies.iter().cloned().collect();
        for dep in &current_deps {
            if let Some(sub_deps) = unified_graph.get(dep) {
                for sub_dep in sub_deps {
                    if !table_paths.contains(sub_dep) {
                        if table_dependencies.insert(sub_dep.clone()) {
                            expanded = true;
                        }
                    }
                }
            }
        }
    }

    let mut phase1_nodes = Vec::new();
    let mut phase2_nodes = Vec::new();
    let mut phase3_nodes = Vec::new();

    for node in unified_graph.keys() {
        if table_paths.contains(node) {
            phase2_nodes.push(node.clone());
        } else if table_dependencies.contains(node) {
            phase1_nodes.push(node.clone());
        } else {
            phase3_nodes.push(node.clone());
        }
    }

    let sort_by_deps = |a: &String, b: &String| {
        let a_deps = unified_graph.get(a).map(|d| d.len()).unwrap_or(0);
        let b_deps = unified_graph.get(b).map(|d| d.len()).unwrap_or(0);
        a_deps.cmp(&b_deps).then_with(|| a.cmp(b))
    };

    phase1_nodes.sort_by(sort_by_deps);
    phase3_nodes.sort_by(sort_by_deps);

    for node in &phase1_nodes {
        if !visited.contains(node) {
            let deps = unified_graph.get(node).cloned().unwrap_or_default();
            visit_node_parsed(
                parsed,
                node,
                &deps,
                &unified_graph,
                &mut visited,
                &mut visiting,
                &mut sorted,
            )?;
        }
    }

    phase2_nodes.sort_by(|a, b| {
        let a_deps = unified_graph.get(a).map(|d| d.len()).unwrap_or(0);
        let b_deps = unified_graph.get(b).map(|d| d.len()).unwrap_or(0);

        let a_deps_on_b = unified_graph
            .get(a)
            .map(|deps| deps.contains(b))
            .unwrap_or(false);
        let b_deps_on_a = unified_graph
            .get(b)
            .map(|deps| deps.contains(a))
            .unwrap_or(false);

        if a_deps_on_b {
            std::cmp::Ordering::Greater
        } else if b_deps_on_a {
            std::cmp::Ordering::Less
        } else {
            a_deps.cmp(&b_deps).then_with(|| a.cmp(b))
        }
    });

    for node in &phase2_nodes {
        if !visited.contains(node) {
            let deps = unified_graph.get(node).cloned().unwrap_or_default();
            visit_node_parsed(
                parsed,
                node,
                &deps,
                &unified_graph,
                &mut visited,
                &mut visiting,
                &mut sorted,
            )?;
        }
    }

    for node in &phase3_nodes {
        if !visited.contains(node) {
            let deps = unified_graph.get(node).cloned().unwrap_or_default();
            visit_node_parsed(
                parsed,
                node,
                &deps,
                &unified_graph,
                &mut visited,
                &mut visiting,
                &mut sorted,
            )?;
        }
    }

    let batches = compute_parallel_batches(&sorted, &unified_graph, &table_paths);

    Ok(batches)
}

fn visit_node_parsed(
    _parsed: &ParsedSchema,
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
            visit_node_parsed(_parsed, dep, dep_deps, graph, visited, visiting, sorted)?;
        }
    }

    visiting.swap_remove(node);
    visited.insert(node.to_string());
    sorted.insert(node.to_string());

    Ok(())
}
