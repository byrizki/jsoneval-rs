use json_eval_rs::path_utils::{normalize_to_json_pointer, get_value_by_pointer, dot_notation_to_schema_pointer};
use serde_json::json;

#[test]
fn test_normalize_to_json_pointer() {
    // JSON Schema reference format
    assert_eq!(normalize_to_json_pointer("#/$params/constants/DEATH_SA"), "/$params/constants/DEATH_SA");
    
    // Dotted notation conversion
    assert_eq!(normalize_to_json_pointer("$params.constants.DEATH_SA"), "/$params/constants/DEATH_SA");
    assert_eq!(normalize_to_json_pointer("user.name"), "/user/name");
    assert_eq!(normalize_to_json_pointer("items.0"), "/items/0");
    
    // Already in pointer format (no-op)
    assert_eq!(normalize_to_json_pointer("/$params/constants/DEATH_SA"), "/$params/constants/DEATH_SA");

    // Simple field
    assert_eq!(normalize_to_json_pointer("field"), "/field");
    
    // Empty
    assert_eq!(normalize_to_json_pointer(""), "");
}

#[test]
fn test_fast_value_access() {
    let data = json!({
        "$params": {
            "constants": {
                "DEATH_SA": 100000
            }
        }
    });

    let value = get_value_by_pointer(&data, "/$params/constants/DEATH_SA");
    assert_eq!(value, Some(&json!(100000)));
}

#[test]
fn test_dot_notation_to_schema_pointer() {
    // Dot notation to schema pointer conversion
    assert_eq!(
        dot_notation_to_schema_pointer("illustration.insured.name"),
        "#/illustration/properties/insured/properties/name"
    );
    
    assert_eq!(
        dot_notation_to_schema_pointer("header.form_number"),
        "#/header/properties/form_number"
    );
    
    assert_eq!(
        dot_notation_to_schema_pointer("insured.date_of_birth"),
        "#/insured/properties/date_of_birth"
    );
    
    // Single field (no dots)
    assert_eq!(
        dot_notation_to_schema_pointer("field"),
        "#/field"
    );
    
    // Already in schema pointer format - should return as-is
    assert_eq!(
        dot_notation_to_schema_pointer("#/illustration/properties/insured/properties/name"),
        "#/illustration/properties/insured/properties/name"
    );
    
    assert_eq!(
        dot_notation_to_schema_pointer("/illustration/properties/insured"),
        "/illustration/properties/insured"
    );
}
