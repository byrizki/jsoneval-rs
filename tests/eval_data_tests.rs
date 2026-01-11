use json_eval_rs::jsoneval::eval_data::EvalData;
use serde_json::json;

#[test]
fn test_nested_path() {
    let mut data = EvalData::new(json!({"user": {"name": "John"}}));
    assert_eq!(data.get("user.name"), Some(&json!("John")));

    data.set("user.age", json!(30));
    assert_eq!(data.get("user.age"), Some(&json!(30)));
}
