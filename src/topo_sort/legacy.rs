use crate::jsoneval::path_utils;
use crate::topo_sort::common::{collect_transitive_deps, compute_parallel_batches};
use crate::JSONEval;
/// Topological sorting for legacy JSONEval
use indexmap::{IndexMap, IndexSet};

pub fn topological_sort(lib: &JSONEval) -> Result<Vec<Vec<String>>, String> {
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
                && !key.ends_with("/options")
                && !key.ends_with("/value")
                && (key.starts_with("#/$") && !key.contains("/value/"))
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

    // Create a mapping from JSON pointer paths to evaluation keys for dependency resolution
    let mut pointer_to_eval: IndexMap<String, String> = IndexMap::new();
    for eval_key in filtered_evaluations.keys() {
        // Convert evaluation keys to JSON pointers
        let pointer = path_utils::normalize_to_json_pointer(eval_key).into_owned();
        pointer_to_eval.insert(pointer, eval_key.clone());
    }

    // Also add table paths to pointer_to_eval for dependency resolution
    for table_path in &table_paths {
        let pointer = path_utils::normalize_to_json_pointer(table_path).into_owned();
        pointer_to_eval.insert(pointer, table_path.clone());
    }

    // Second pass: group ALL evaluations by table and merge dependencies
    // Process ALL evaluations (not just filtered ones) to capture table dependencies
    for (eval_key, deps) in lib.evaluations.keys().map(|k| {
        let deps = lib.dependencies.get(k).cloned().unwrap_or_default();
        (k, deps)
    }) {
        // Find which table this evaluation belongs to
        // Use longest match to handle nested table names correctly
        // (e.g., ILB_SURRENDER vs ILB_SURRENDER_BENPAY_CLONE)
        let table_path_opt = table_paths
            .iter()
            .filter(|tp| eval_key.starts_with(tp.as_str()))
            .max_by_key(|tp| tp.len());

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

                    // Check if dependency is a JSON pointer path that maps to an evaluation
                    if let Some(eval_key) = pointer_to_eval.get(dep) {
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

    // Create a unified graph and resolve JSON pointer dependencies in table groups
    let mut unified_graph: IndexMap<String, IndexSet<String>> = IndexMap::new();

    // Add table groups with resolved dependencies
    for (table_path, deps) in &table_groups {
        let resolved_deps: IndexSet<String> = deps
            .iter()
            .filter_map(|dep| {
                // Filter out self-references (table depending on itself)
                // This is common for iterative calculations within the same table
                if dep == table_path {
                    return None;
                }

                // Try to resolve JSON pointer path to evaluation key
                if let Some(eval_key) = pointer_to_eval.get(dep) {
                    // Also filter out resolved self-references
                    if eval_key == table_path {
                        return None;
                    }
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
                // Check if dependency is a JSON pointer path that maps to an evaluation
                if let Some(eval_key) = pointer_to_eval.get(dep) {
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

                // OPTIMIZED: Check if dependency is a static array with evaluated fields
                // Use consistent path utilities for conversion
                let dep_as_pointer = path_utils::normalize_to_json_pointer(dep);
                let dep_as_eval_prefix = format!("#{}", dep_as_pointer);
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

    // ==========================================
    // 3-PHASE PROCESSING: Dependencies → Tables → Rest
    // ==========================================

    // Identify all table dependencies (transitive)
    // This includes all non-table nodes that tables transitively depend on
    let mut table_dependencies = IndexSet::new();
    for table_path in &table_paths {
        if let Some(deps) = unified_graph.get(table_path) {
            collect_transitive_deps(deps, &unified_graph, &table_paths, &mut table_dependencies);
        }
    }

    // CRITICAL: Expand to complete transitive closure
    // Ensure ALL non-table dependencies of phase 1 nodes are also in phase 1
    // Example: If table depends on A, and A depends on B, then both A and B are in phase 1
    let mut expanded = true;
    while expanded {
        expanded = false;
        let current_deps: Vec<String> = table_dependencies.iter().cloned().collect();
        for dep in &current_deps {
            if let Some(sub_deps) = unified_graph.get(dep) {
                for sub_dep in sub_deps {
                    // Skip tables - they stay in phase 2
                    if !table_paths.contains(sub_dep) {
                        if table_dependencies.insert(sub_dep.clone()) {
                            expanded = true; // Found new dependency, need another pass
                        }
                    }
                }
            }
        }
    }

    // Separate nodes into phases
    let mut phase1_nodes = Vec::new(); // Table dependencies (non-tables needed by tables)
    let mut phase2_nodes = Vec::new(); // Tables
    let mut phase3_nodes = Vec::new(); // Everything else

    for node in unified_graph.keys() {
        if table_paths.contains(node) {
            // Phase 2: Tables
            phase2_nodes.push(node.clone());
        } else if table_dependencies.contains(node) {
            // Phase 1: Non-table dependencies of tables
            phase1_nodes.push(node.clone());
        } else {
            // Phase 3: Remaining nodes
            phase3_nodes.push(node.clone());
        }
    }

    // Sort phase 1 and phase 3 by dependency order (nodes with fewer deps first)
    // This provides a better starting order for topological processing
    let sort_by_deps = |a: &String, b: &String| {
        let a_deps = unified_graph.get(a).map(|d| d.len()).unwrap_or(0);
        let b_deps = unified_graph.get(b).map(|d| d.len()).unwrap_or(0);
        a_deps.cmp(&b_deps).then_with(|| a.cmp(b))
    };

    phase1_nodes.sort_by(sort_by_deps);
    phase3_nodes.sort_by(sort_by_deps);

    // PHASE 1: Process table dependencies (respecting their internal dependencies)
    for node in &phase1_nodes {
        if !visited.contains(node) {
            let deps = unified_graph.get(node).cloned().unwrap_or_default();
            // visit_node will recursively process dependencies in correct order
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

    // PHASE 2: Process tables in dependency order
    // Sort tables by their dependencies (tables with fewer/no table deps come first)
    phase2_nodes.sort_by(|a, b| {
        let a_deps = unified_graph.get(a).map(|d| d.len()).unwrap_or(0);
        let b_deps = unified_graph.get(b).map(|d| d.len()).unwrap_or(0);

        // Check if A depends on B or B depends on A
        let a_deps_on_b = unified_graph
            .get(a)
            .map(|deps| deps.contains(b))
            .unwrap_or(false);
        let b_deps_on_a = unified_graph
            .get(b)
            .map(|deps| deps.contains(a))
            .unwrap_or(false);

        if a_deps_on_b {
            std::cmp::Ordering::Greater // A depends on B, so B comes first
        } else if b_deps_on_a {
            std::cmp::Ordering::Less // B depends on A, so A comes first
        } else {
            // No direct dependency, sort by dependency count then alphabetically
            a_deps.cmp(&b_deps).then_with(|| a.cmp(b))
        }
    });

    for node in &phase2_nodes {
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

    // PHASE 3: Process remaining nodes (respecting their internal dependencies)
    for node in &phase3_nodes {
        if !visited.contains(node) {
            let deps = unified_graph.get(node).cloned().unwrap_or_default();
            // visit_node will recursively process dependencies in correct order
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

    // Now convert the flat sorted list into parallel batches
    // Batch nodes by their "level" - all nodes at the same level can run in parallel
    let batches = compute_parallel_batches(&sorted, &unified_graph, &table_paths);

    Ok(batches)
}

/// Compute parallel execution batches from a topologically sorted list
///
/// Algorithm: Assign each node to the earliest batch where all its dependencies
/// have been processed in previous batches.

pub fn visit_node_with_priority(
    lib: &JSONEval,
    node: &str,
    deps: &IndexSet<String>,
    graph: &IndexMap<String, IndexSet<String>>,
    visited: &mut IndexSet<String>,
    visiting: &mut IndexSet<String>,
    sorted: &mut IndexSet<String>,
    table_paths: &IndexSet<String>,
) -> Result<(), String> {
    if visiting.contains(node) {
        return Err(format!("Circular dependency detected involving: {}", node));
    }

    if visited.contains(node) {
        return Ok(());
    }

    visiting.insert(node.to_string());

    // Sort dependencies by priority: non-tables first
    let mut sorted_deps: Vec<String> = deps.iter().cloned().collect();
    sorted_deps.sort_by(|a, b| {
        let a_is_table = table_paths.contains(a);
        let b_is_table = table_paths.contains(b);

        match (a_is_table, b_is_table) {
            (false, true) => std::cmp::Ordering::Less, // non-table before table
            (true, false) => std::cmp::Ordering::Greater, // table after non-table
            _ => a.cmp(b),                             // same priority, sort alphabetically
        }
    });

    // Process dependencies in priority order
    for dep in sorted_deps {
        if let Some(dep_deps) = graph.get(&dep) {
            visit_node_with_priority(
                lib,
                &dep,
                dep_deps,
                graph,
                visited,
                visiting,
                sorted,
                table_paths,
            )?;
        }
    }

    visiting.swap_remove(node);
    visited.insert(node.to_string());
    sorted.insert(node.to_string());

    Ok(())
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
