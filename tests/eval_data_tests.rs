use json_eval_rs::jsoneval::eval_data::EvalData;
use serde_json::json;

#[test]
fn test_nested_path() {
    let mut data = EvalData::new(json!({"user": {"name": "John"}}));
    assert_eq!(data.get("user.name"), Some(&json!("John")));

    data.set("user.age", json!(30));
    assert_eq!(data.get("user.age"), Some(&json!(30)));
}

#[test]
fn replace_data_ignores_non_object_root_without_panicking() {
    let mut data = EvalData::new(json!({"existing": true}));

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        data.replace_data_and_context(json!([]), json!({}));
    }));

    assert!(result.is_ok(), "array root must not panic");
    assert_eq!(data.get("existing"), Some(&json!(true)));
}
