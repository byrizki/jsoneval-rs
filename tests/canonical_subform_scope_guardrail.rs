use json_eval_rs::JSONEval;
use serde_json::{json, Value};

fn schema() -> Value {
    json!({
        "form": {
            "type": "object",
            "properties": {
                "product": {
                    "type": "object",
                    "properties": {
                        "items": {
                            "type": "array",
                            "itemsRootKey": "items",
                            "items": {
                                "properties": {
                                    "amount": { "type": "number" },
                                    "local_amount": {
                                        "type": "number",
                                        "value": {
                                            "$evaluation": {
                                                "$ref": "#/items/properties/amount"
                                            }
                                        }
                                    },
                                    "dependent_amount": {
                                        "type": "number",
                                        "value": {
                                            "$evaluation": {
                                                "$ref": "#/items/properties/amount"
                                            }
                                        }
                                    },
                                    "parent_amount": {
                                        "type": "number",
                                        "value": {
                                            "$evaluation": {
                                                "VALUEAT": [
                                                    { "$ref": "#/form/properties/product/properties/items" },
                                                    1,
                                                    "amount"
                                                ]
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

fn parent_data() -> Value {
    json!({
        "form": {
            "product": {
                "items": [
                    { "amount": 11 },
                    { "amount": 23 }
                ]
            }
        }
    })
}

fn item_values(eval: &mut JSONEval) -> (Value, Value) {
    let evaluated = eval.get_evaluated_schema_subform("form.product.items.1");
    (
        evaluated
            .pointer("/items/properties/local_amount/value")
            .cloned()
            .unwrap_or(Value::Null),
        evaluated
            .pointer("/items/properties/parent_amount/value")
            .cloned()
            .unwrap_or(Value::Null),
    )
}

#[test]
fn indexed_subform_full_parent_and_item_wrapper_have_identical_results() {
    let schema = schema().to_string();
    let parent = parent_data();
    let parent_input = parent.to_string();

    let mut full_eval = JSONEval::new(&schema, None, Some(&parent_input)).unwrap();
    full_eval.evaluate(&parent_input, None, None, None).unwrap();
    full_eval
        .evaluate_subform(
            "form.product.items.1",
            &parent_input,
            None,
            None,
            None,
        )
        .unwrap();

    let item_wrapper = json!({
        "items": parent["form"]["product"]["items"][1].clone()
    })
    .to_string();
    let mut wrapper_eval = JSONEval::new(&schema, None, Some(&parent_input)).unwrap();
    wrapper_eval
        .evaluate(&parent_input, None, None, None)
        .unwrap();
    wrapper_eval
        .evaluate_subform(
            "form.product.items.1",
            &item_wrapper,
            None,
            None,
            None,
        )
        .unwrap();

    assert_eq!(
        item_values(&mut full_eval),
        item_values(&mut wrapper_eval)
    );
    assert_eq!(item_values(&mut wrapper_eval), (json!(23), json!(23)));
}

#[test]
fn indexed_subform_hybrid_parent_and_item_wrapper_maps_wrapper_to_active_index() {
    let schema = schema().to_string();
    let parent_input = parent_data().to_string();
    let hybrid = json!({
        "form": {},
        "items": { "amount": 29 }
    })
    .to_string();

    let mut eval = JSONEval::new(&schema, None, Some(&parent_input)).unwrap();
    eval.evaluate(&parent_input, None, None, None).unwrap();
    eval.evaluate_subform("form.product.items.1", &hybrid, None, None, None)
        .unwrap();

    assert_eq!(item_values(&mut eval), (json!(29), json!(29)));
}

#[test]
fn indexed_subform_dependent_patches_project_active_item_to_local_root() {
    let schema = schema().to_string();
    let parent = parent_data();
    let parent_input = parent.to_string();
    let mut eval = JSONEval::new(&schema, None, Some(&parent_input)).unwrap();
    eval.evaluate(&parent_input, None, None, None).unwrap();

    let warm_payload = json!({
        "form": {},
        "items": { "amount": 23 }
    });
    eval.evaluate_subform(
        "form.product.items.1",
        &warm_payload.to_string(),
        None,
        None,
        None,
    )
    .unwrap();

    let payload = json!({
        "form": {},
        "items": { "amount": 29 }
    });
    let changes = eval
        .evaluate_dependents_subform(
            "form.product.items.1",
            &["items.amount".to_string()],
            Some(&payload.to_string()),
            None,
            true,
            None,
            None,
            true,
        )
        .unwrap();

    assert!(
        changes
            .as_array()
            .unwrap()
            .iter()
            .any(|change| change["$ref"] == "items.dependent_amount"),
        "changes={changes:?}"
    );
    assert!(
        !changes
            .as_array()
            .unwrap()
            .iter()
            .any(|change| { change["$ref"] == "form.product.items.1.dependent_amount" }),
        "changes={changes:?}"
    );
}

#[test]
fn indexed_subform_rejects_payload_without_canonical_item_or_wrapper_root() {
    let schema = schema().to_string();
    let parent_input = parent_data().to_string();
    let mut eval = JSONEval::new(&schema, None, Some(&parent_input)).unwrap();
    eval.evaluate(&parent_input, None, None, None).unwrap();

    let err = eval
        .evaluate_subform(
            "form.product.items.1",
            &json!({ "unexpected": { "amount": 23 } }).to_string(),
            None,
            None,
            None,
        )
        .expect_err("missing active item must not silently evaluate as null");

    assert!(err.contains("expected active item"), "error={err}");
}

#[test]
fn indexed_subform_local_and_parent_table_paths_read_same_active_item() {
    let schema = schema().to_string();
    let parent = parent_data();
    let parent_input = parent.to_string();
    let item_wrapper = json!({
        "items": parent["form"]["product"]["items"][1].clone()
    })
    .to_string();
    let mut eval = JSONEval::new(&schema, None, Some(&parent_input)).unwrap();
    eval.evaluate(&parent_input, None, None, None).unwrap();
    eval.evaluate_subform(
        "form.product.items.1",
        &item_wrapper,
        None,
        None,
        None,
    )
    .unwrap();

    let (local_amount, parent_amount) = item_values(&mut eval);
    assert_eq!(local_amount, json!(23));
    assert_eq!(
        parent_amount, local_amount,
        "VALUEAT parent-array read must observe same active item as local schema reference"
    );
}
