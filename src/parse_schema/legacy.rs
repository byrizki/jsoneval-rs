/// Legacy schema parsing for JSONEval (direct evaluation)
use indexmap::IndexMap;
use serde_json::Value;
use std::sync::Arc;

use crate::{topo_sort, JSONEval};

pub fn parse_schema(lib: &mut JSONEval) -> Result<(), String> {
    // Use centralized path normalization from path_utils
    // This ensures consistent $ref/var handling across the entire pipeline

    // Single-pass collection: walk schema once and collect everything
    let mut evaluations = IndexMap::new();
    let mut tables = IndexMap::new();
    let mut dependencies = IndexMap::new();
    let mut value_fields = Vec::new();
    let mut layout_paths = Vec::new();
    let mut dependents_evaluations = IndexMap::new();
    let mut options_templates = Vec::new();
    let mut subforms_data = Vec::new();

    // Get mutable access to engine through Arc
    let engine = Arc::get_mut(&mut lib.engine)
        .ok_or("Cannot get mutable reference to engine - JSONEval engine is shared")?;

    let mut fields_with_rules = Vec::new();
    let mut conditional_hidden_fields = Vec::new();
    let mut conditional_readonly_fields = Vec::new();

    crate::parse_schema::common::walk_schema(
        &lib.schema,
        "#",
        engine,
        &mut evaluations,
        &mut tables,
        &mut dependencies,
        &mut value_fields,
        &mut layout_paths,
        &mut dependents_evaluations,
        &mut options_templates,
        &mut subforms_data,
        &mut fields_with_rules,
        &mut conditional_hidden_fields,
        &mut conditional_readonly_fields,
    )?;

    lib.evaluations = Arc::new(evaluations);
    lib.tables = Arc::new(tables);
    lib.dependencies = Arc::new(dependencies);
    lib.conditional_hidden_fields = Arc::new(conditional_hidden_fields);
    lib.conditional_readonly_fields = Arc::new(conditional_readonly_fields);
    // Sort layout paths by depth descending (deepest first)
    // This ensures nested layouts are resolved before their parents
    // Count '/' to determine depth
    layout_paths.sort_by(|a, b| {
        let depth_a = a.matches('/').count();
        let depth_b = b.matches('/').count();
        depth_b.cmp(&depth_a)
    });

    lib.layout_paths = Arc::new(layout_paths);
    lib.dependents_evaluations = Arc::new(dependents_evaluations);
    lib.options_templates = Arc::new(options_templates);
    lib.fields_with_rules = Arc::new(fields_with_rules);

    // Build subforms from collected data (after walk completes)
    lib.subforms = build_subforms_from_data(subforms_data, lib)?;

    // Collect table-level dependencies by aggregating all column dependencies
    let mut deps = (*lib.dependencies).clone();
    crate::parse_schema::common::collect_table_dependencies(&lib.tables, &mut deps);
    lib.dependencies = std::sync::Arc::new(deps);

    lib.sorted_evaluations = Arc::new(topo_sort::legacy::topological_sort(lib)?);

    // Categorize evaluations for result handling
    let (rules, others) = crate::parse_schema::common::categorize_evaluations(
        &lib.sorted_evaluations,
        &lib.evaluations,
        &lib.tables,
    );
    lib.rules_evaluations = std::sync::Arc::new(rules);
    lib.others_evaluations = std::sync::Arc::new(others);

    // Process collected value fields
    let value_evals = crate::parse_schema::common::process_value_fields(value_fields, &lib.tables);
    lib.value_evaluations = std::sync::Arc::new(value_evals);

    // Build reffed_by graph (reverse dependencies for hidden conditions)
    let reffed = crate::parse_schema::common::build_reffed_by(&lib.dependencies);
    lib.reffed_by = std::sync::Arc::new(reffed);

    // Build dep_formula_triggers graph (formula context dependency reverse map)
    let trig = crate::parse_schema::common::build_dep_formula_triggers(
        &lib.dependents_evaluations,
        &lib.evaluations,
        &lib.engine,
    );
    lib.dep_formula_triggers = std::sync::Arc::new(trig);

    // Pre-compile all table metadata for zero-copy evaluation
    build_table_metadata(lib)?;

    Ok(())
}

/// Build subforms from collected data during walk
fn build_subforms_from_data(
    subforms_data: Vec<(String, serde_json::Map<String, Value>, Value)>,
    parent: &JSONEval,
) -> Result<IndexMap<String, Box<JSONEval>>, String> {
    let mut subforms = IndexMap::new();

    for (path, field_map, items) in subforms_data {
        create_subform(&path, &field_map, &items, &mut subforms, parent)?;
    }

    Ok(subforms)
}

/// Create an isolated sub-JSONEval for a subform
fn create_subform(
    path: &str,
    field_map: &serde_json::Map<String, Value>,
    items: &Value,
    subforms: &mut IndexMap<String, Box<JSONEval>>,
    parent: &JSONEval,
) -> Result<(), String> {
    // Extract field key from path (e.g., "#/properties/riders" -> "riders")
    let field_key = path.split('/').last().unwrap_or(path);

    // Build subform schema: { $params: from parent, [field_key]: items content }
    let mut subform_schema = serde_json::Map::new();

    // Copy $params from parent schema
    if let Some(params) = parent.schema.get("$params") {
        subform_schema.insert("$params".to_string(), params.clone());
    }

    // Create field object with items content
    let mut field_obj = serde_json::Map::new();

    // Copy properties from items
    if let Value::Object(items_map) = items {
        for (key, value) in items_map {
            field_obj.insert(key.clone(), value.clone());
        }
    }

    // Copy field-level properties (title, etc.) but exclude items and type="array"
    for (key, value) in field_map {
        if key != "items" && key != "type" && key != "value" {
            field_obj.insert(key.clone(), value.clone());
        }
    }

    // Set type to "object" for the subform root
    field_obj.insert("type".to_string(), Value::String("object".to_string()));

    subform_schema.insert(field_key.to_string(), Value::Object(field_obj));

    // Create sub-JSONEval with isolated schema, zero-copying context and static_arrays
    let sub_eval = crate::JSONEval::new_subform(
        Value::Object(subform_schema),
        parent.context.clone(),
        std::sync::Arc::clone(&parent.static_arrays),
    )
    .map_err(|e| format!("Failed to create subform for {}: {}", field_key, e))?;

    subforms.insert(path.to_string(), Box::new(sub_eval));

    Ok(())
}

/// Build pre-compiled table metadata at parse time (moves heavy operations from evaluation)
fn build_table_metadata(lib: &mut JSONEval) -> Result<(), String> {
    let mut table_metadata = IndexMap::new();

    for (eval_key, table) in lib.tables.iter() {
        let metadata = crate::parse_schema::common::compile_table_metadata(
            &lib.evaluations,
            &lib.engine,
            eval_key,
            table,
        )?;
        table_metadata.insert(eval_key.to_string(), metadata);
    }

    lib.table_metadata = Arc::new(table_metadata);
    Ok(())
}
