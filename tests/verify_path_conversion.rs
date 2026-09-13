use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_minimal_form_path_conversion() {
    // Load the minimal_form.json fixture
    let schema_str = std::fs::read_to_string("tests/fixtures/minimal_form.json")
        .expect("Failed to read minimal_form.json");

    let mut eval = JSONEval::new(&schema_str, None, None).expect("Failed to create JSONEval");

    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    eval.evaluate(&data_str, None, None, None).unwrap();

    let evaluated = eval.get_evaluated_schema_resolved();

    // Test 1: Check the user element (original ref: "#/form/properties/user")
    let user_element = evaluated
        .pointer("/form/$layout/elements/1")
        .expect("User element should exist at form layout");

    // Verify $fullpath is in dotted notation (original ref: "#/form/properties/user")
    assert_eq!(
        user_element.get("$fullpath").and_then(|v| v.as_str()),
        Some("form.properties.user"),
        "$fullpath should be in dotted notation"
    );

    // Verify $path is the last segment only
    assert_eq!(
        user_element.get("$path").and_then(|v| v.as_str()),
        Some("user"),
        "$path should be the last segment only"
    );

    // Test 2: Check a deeply nested element (from user's layout)
    let name_element = evaluated
        .pointer("/form/$layout/elements/1/elements/0")
        .expect("User name element should exist");

    // Verify deeply nested path conversion
    assert_eq!(
        name_element.get("$fullpath").and_then(|v| v.as_str()),
        Some("form.properties.user.properties.name"),
        "$fullpath should convert deep JSON pointer to dotted notation"
    );

    assert_eq!(
        name_element.get("$path").and_then(|v| v.as_str()),
        Some("name"),
        "$path should be the last segment only for deeply nested paths"
    );
}
