/// Shared utilities for topological sorting
use indexmap::{IndexMap, IndexSet};

/// Compute parallel execution batches from sorted dependencies
///
/// Groups evaluations into batches where items in the same batch can be evaluated in parallel.
/// Each batch depends only on items from previous batches.
///
/// Handles table column dependencies by mapping them back to their parent table.
pub fn compute_parallel_batches(
    sorted: &IndexSet<String>,
    graph: &IndexMap<String, IndexSet<String>>,
    table_paths: &IndexSet<String>,
) -> Vec<Vec<String>> {
    let mut batches: Vec<Vec<String>> = Vec::new();
    let mut node_to_batch: IndexMap<String, usize> = IndexMap::new();

    for node in sorted {
        // Find the maximum batch level of all dependencies that are in the graph
        let deps = graph.get(node);

        let max_dep_batch = if let Some(deps) = deps {
            deps.iter()
                .filter_map(|dep| {
                    // Check if dependency is in sorted list directly
                    if sorted.contains(dep) {
                        return node_to_batch.get(dep).copied();
                    }

                    // Check if dependency is a table column path (e.g., TABLE/$table/0/column)
                    // If so, map it to the parent table path
                    if dep.contains("/$table/") {
                        for table_path in table_paths {
                            if dep.starts_with(table_path) {
                                // Found the parent table, check its batch
                                return node_to_batch.get(table_path).copied();
                            }
                        }
                    }

                    // Check for other table-related paths like $datas, $skip, $clear
                    if dep.contains("/$datas/")
                        || dep.ends_with("/$skip")
                        || dep.ends_with("/$clear")
                    {
                        for table_path in table_paths {
                            if dep.starts_with(table_path) {
                                return node_to_batch.get(table_path).copied();
                            }
                        }
                    }

                    // Dependency is external (not in our graph), ignore it
                    None
                })
                .max()
        } else {
            None
        };

        // This node goes in the batch after the max dependency batch
        let batch_idx = max_dep_batch.map(|b| b + 1).unwrap_or(0);

        // Ensure we have enough batches
        while batches.len() <= batch_idx {
            batches.push(Vec::new());
        }

        batches[batch_idx].push(node.clone());
        node_to_batch.insert(node.clone(), batch_idx);
    }

    batches
}

/// Recursively collect all transitive dependencies, excluding tables themselves
pub fn collect_transitive_deps(
    deps: &IndexSet<String>,
    graph: &IndexMap<String, IndexSet<String>>,
    table_paths: &IndexSet<String>,
    result: &mut IndexSet<String>,
) {
    for dep in deps {
        // Skip if it's a table (we only want non-table dependencies)
        if table_paths.contains(dep) {
            continue;
        }

        // Add this dependency if not already added
        if result.insert(dep.clone()) {
            // Recursively collect its dependencies
            if let Some(sub_deps) = graph.get(dep) {
                collect_transitive_deps(sub_deps, graph, table_paths, result);
            }
        }
    }
}
