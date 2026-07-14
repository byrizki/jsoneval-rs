use json_eval_rs::{jsoneval::types::LayoutOverlayEntry, JSONEval};
use serde_json::{json, Value};

fn merge_layout_overlay(schema: &mut Value, overlay_entries: &[LayoutOverlayEntry]) {
    let mut entries: Vec<_> = overlay_entries.iter().collect();
    entries.sort_by(|a, b| {
        a.layout_path
            .matches('/')
            .count()
            .cmp(&b.layout_path.matches('/').count())
            .then_with(|| a.element_idx.cmp(&b.element_idx))
    });

    for entry in entries {
        let layout_path = entry.layout_path.trim_start_matches('#');
        let resolved = schema
            .pointer(layout_path)
            .and_then(Value::as_array)
            .and_then(|elements| elements.get(entry.element_idx))
            .and_then(|element| element.get("$ref"))
            .and_then(Value::as_str)
            .and_then(|reference| schema.pointer(reference.trim_start_matches('#')).cloned());

        if let Some(Value::Array(elements)) = schema.pointer_mut(layout_path) {
            if let Some(element) = elements.get_mut(entry.element_idx) {
                if let Some(Value::Object(mut resolved)) = resolved {
                    if let Value::Object(original) = element.take() {
                        for (key, value) in original {
                            if key != "$ref" {
                                resolved.insert(key, value);
                            }
                        }
                    }
                    *element = Value::Object(resolved);
                }
                if let Value::Object(element) = element {
                    for (key, value) in &entry.overlay {
                        element.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }

    fn stamp_properties(value: &mut Value, path: &str, parent_hidden: bool) {
        let Some(map) = value.as_object_mut() else {
            return;
        };
        let hidden = parent_hidden
            || map
                .get("condition")
                .and_then(Value::as_object)
                .and_then(|condition| condition.get("hidden"))
                == Some(&Value::Bool(true));
        if let Some(Value::Object(properties)) = map.get_mut("properties") {
            for (name, property) in properties {
                let property_path = if path.is_empty() {
                    format!("properties.{}", name)
                } else {
                    format!("{}.properties.{}", path, name)
                };
                if let Value::Object(property_map) = property {
                    property_map.insert(
                        "$fullpath".to_string(),
                        Value::String(property_path.clone()),
                    );
                    property_map.insert("$path".to_string(), Value::String(name.clone()));
                    property_map.insert("$parentHide".to_string(), Value::Bool(hidden));
                }
                stamp_properties(property, &property_path, hidden);
            }
        }
        for (name, child) in map {
            if name != "properties" && !name.starts_with('$') && child.is_object() {
                let child_path = if path.is_empty() {
                    name.clone()
                } else {
                    format!("{}.{}", path, name)
                };
                stamp_properties(child, &child_path, hidden);
            }
        }
    }

    stamp_properties(schema, "", false);
}

#[test]
fn resolved_schema_omits_params_and_stamps_properties_and_inline_layout_items() {
    let schema = json!({
        "$params": { "internal": true },
        "illustration": {
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "$layout": {
                "elements": [
                    { "$ref": "#/illustration/properties/name" },
                    { "type": "CustomLayout" }
                ]
            }
        }
    })
    .to_string();
    let mut eval = JSONEval::new(&schema, None, None).unwrap();
    eval.evaluate("{}", None, None, None).unwrap();

    let resolved = eval.get_evaluated_schema_resolved();
    let mut compact_plus_overlay = eval.get_evaluated_schema_without_params();
    merge_layout_overlay(&mut compact_plus_overlay, &eval.get_resolved_layout());

    assert!(
        resolved.get("$params").is_none(),
        "resolved output must match compact schema and omit $params"
    );

    let property = resolved
        .pointer("/illustration/properties/name")
        .expect("property must exist");
    assert_eq!(
        property.pointer("/$fullpath"),
        Some(&json!("illustration.properties.name"))
    );
    assert_eq!(property.pointer("/$path"), Some(&json!("name")));
    assert_eq!(property.pointer("/$parentHide"), Some(&json!(false)));

    let inline_layout = resolved
        .pointer("/illustration/$layout/elements/1")
        .expect("inline custom layout item must exist");
    assert_eq!(
        inline_layout.pointer("/$fullpath"),
        Some(&json!("illustration.$layout.elements.1"))
    );
    assert_eq!(inline_layout.pointer("/$path"), Some(&json!("1")));
    assert_eq!(inline_layout.pointer("/$parentHide"), Some(&json!(false)));

    assert_eq!(
        resolved, compact_plus_overlay,
        "direct resolved schema must equal compact schema merged with resolver overlays"
    );
}
