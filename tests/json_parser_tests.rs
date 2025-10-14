use json_eval_rs::json_parser::{parse_json_str, parse_json_bytes};

#[test]
fn test_parse_simple_json() {
    let json = r#"{"name": "test", "value": 42}"#;
    let result = parse_json_str(json).unwrap();
    assert_eq!(result["name"], "test");
    assert_eq!(result["value"], 42);
}

#[test]
fn test_parse_array() {
    let json = r#"[1, 2, 3, 4, 5]"#;
    let result = parse_json_str(json).unwrap();
    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 5);
}

#[test]
fn test_parse_nested() {
    let json = r#"{"outer": {"inner": {"deep": "value"}}}"#;
    let result = parse_json_str(json).unwrap();
    assert_eq!(result["outer"]["inner"]["deep"], "value");
}

#[test]
fn test_fallback_to_serde() {
    // This should work even if SIMD fails
    let json = r#"{"valid": true}"#;
    let result = parse_json_str(json).unwrap();
    assert_eq!(result["valid"], true);
}

#[test]
fn test_parse_bytes() {
    let json_bytes = br#"{"a":1}"#.to_vec();
    let result = parse_json_bytes(json_bytes).unwrap();
    assert_eq!(result["a"], 1);
}
