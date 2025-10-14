use json_eval_rs::path_utils::{normalize_to_json_pointer, get_value_by_pointer};
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
