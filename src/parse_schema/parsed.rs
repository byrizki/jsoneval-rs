/// ParsedSchema parsing for schema caching and reuse
use indexmap::IndexMap;
use serde_json::Value;
use std::sync::Arc;

use crate::topo_sort;
use crate::ParsedSchema;

pub fn parse_schema_into(parsed: &mut ParsedSchema) -> Result<(), String> {
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
    let engine = Arc::get_mut(&mut parsed.engine)
        .ok_or("Cannot get mutable reference to engine - ParsedSchema is shared")?;

    let mut fields_with_rules = Vec::new();
    let mut conditional_hidden_fields = Vec::new();
    let mut conditional_readonly_fields = Vec::new();

    crate::parse_schema::common::walk_schema(
        &parsed.schema,
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

    parsed.evaluations = Arc::new(evaluations);
    parsed.tables = Arc::new(tables);
    parsed.dependencies = Arc::new(dependencies);
    parsed.conditional_hidden_fields = Arc::new(conditional_hidden_fields);
    parsed.conditional_readonly_fields = Arc::new(conditional_readonly_fields);
    // Sort layout paths by depth descending (deepest first)
    // This ensures nested layouts are resolved before their parents
    // Count '/' to determine depth
    layout_paths.sort_by(|a, b| {
        let depth_a = a.matches('/').count();
        let depth_b = b.matches('/').count();
        depth_b.cmp(&depth_a)
    });

    parsed.layout_paths = Arc::new(layout_paths);
    parsed.dependents_evaluations = Arc::new(dependents_evaluations);
    parsed.options_templates = Arc::new(options_templates);
    parsed.fields_with_rules = Arc::new(fields_with_rules);

    // Build subforms from collected data (after walk completes)
    parsed.subforms = build_subforms_from_data_parsed(subforms_data, parsed)?;

    // Collect table-level dependencies by aggregating all column dependencies
    let mut deps = (*parsed.dependencies).clone();
    crate::parse_schema::common::collect_table_dependencies(&parsed.tables, &mut deps);
    parsed.dependencies = std::sync::Arc::new(deps);

    parsed.sorted_evaluations = Arc::new(topo_sort::parsed::topological_sort_parsed(parsed)?);

    // Categorize evaluations for result handling
    let (rules, others) = crate::parse_schema::common::categorize_evaluations(
        &parsed.sorted_evaluations,
        &parsed.evaluations,
        &parsed.tables,
    );
    parsed.rules_evaluations = std::sync::Arc::new(rules);
    parsed.others_evaluations = std::sync::Arc::new(others);

    // Process collected value fields
    let value_evals =
        crate::parse_schema::common::process_value_fields(value_fields, &parsed.tables);
    parsed.value_evaluations = std::sync::Arc::new(value_evals);

    // Build reffed_by graph (reverse dependencies for hidden conditions)
    let reffed = crate::parse_schema::common::build_reffed_by(&parsed.dependencies);
    parsed.reffed_by = std::sync::Arc::new(reffed);

    // Build dep_formula_triggers graph (formula context dependency reverse map)
    let trig = crate::parse_schema::common::build_dep_formula_triggers(
        &parsed.dependents_evaluations,
        &parsed.evaluations,
        &parsed.engine,
    );
    parsed.dep_formula_triggers = std::sync::Arc::new(trig);

    // Pre-compile all table metadata for zero-copy evaluation
    build_table_metadata_parsed(parsed)?;

    Ok(())
}

// ============================================================================

/// Build subforms from collected data for ParsedSchema
fn build_subforms_from_data_parsed(
    subforms_data: Vec<(String, serde_json::Map<String, Value>, Value)>,
    parsed: &ParsedSchema,
) -> Result<IndexMap<String, Arc<ParsedSchema>>, String> {
    let mut subforms = IndexMap::new();

    for (path, field_map, items) in subforms_data {
        create_subform_parsed(&path, &field_map, &items, &mut subforms, parsed)?;
    }

    Ok(subforms)
}

/// Create an isolated ParsedSchema for a subform (ParsedSchema version)
///
/// This creates an Arc<ParsedSchema> instead of Box<JSONEval> for efficient sharing.
/// Subforms can now be cloned cheaply across multiple JSONEval instances.
fn create_subform_parsed(
    path: &str,
    field_map: &serde_json::Map<String, Value>,
    items: &Value,
    subforms: &mut IndexMap<String, Arc<ParsedSchema>>,
    parsed: &ParsedSchema,
) -> Result<(), String> {
    // Extract field key from path (e.g., "#/properties/riders" -> "riders")
    let field_key = path.split('/').last().unwrap_or(path);

    // Build subform schema: { $params: from parent, [field_key]: items content }
    let mut subform_schema = serde_json::Map::new();

    // Copy $params from parent schema
    if let Some(params) = parsed.schema.get("$params") {
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
        if key != "items" && key != "type" {
            field_obj.insert(key.clone(), value.clone());
        }
    }

    // Set type to "object" for the subform root
    field_obj.insert("type".to_string(), Value::String("object".to_string()));

    subform_schema.insert(field_key.to_string(), Value::Object(field_obj));

    // Parse into ParsedSchema (more efficient than JSONEval)
    // This allows the subform to be shared via Arc across multiple evaluations
    let subform_schema_value = Value::Object(subform_schema);
    let subform_parsed = ParsedSchema::parse_value(subform_schema_value)
        .map_err(|e| format!("Failed to parse subform schema for {}: {}", field_key, e))?;

    subforms.insert(path.to_string(), Arc::new(subform_parsed));

    Ok(())
}

/// Build pre-compiled table metadata (ParsedSchema version)
fn build_table_metadata_parsed(parsed: &mut ParsedSchema) -> Result<(), String> {
    let mut table_metadata = IndexMap::new();

    for (eval_key, table) in parsed.tables.iter() {
        let metadata = crate::parse_schema::common::compile_table_metadata(
            &parsed.evaluations,
            &parsed.engine,
            eval_key,
            table,
        )?;
        table_metadata.insert(eval_key.to_string(), metadata);
    }

    parsed.table_metadata = Arc::new(table_metadata);
    Ok(())
}
