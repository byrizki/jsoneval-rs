use json_eval_rs::jsoneval::path_utils::{
    canonicalize_schema_path, dot_notation_to_schema_pointer, get_value_by_pointer,
    normalize_to_json_pointer, pointer_to_dot_notation,
};
use serde_json::json;

#[test]
fn test_normalize_to_json_pointer() {
    // JSON Schema reference format
    assert_eq!(
        normalize_to_json_pointer("#/$params/constants/DEATH_SA"),
        "/$params/constants/DEATH_SA"
    );

    // Dotted notation conversion
    assert_eq!(
        normalize_to_json_pointer("$params.constants.DEATH_SA"),
        "/$params/constants/DEATH_SA"
    );
    assert_eq!(normalize_to_json_pointer("user.name"), "/user/name");
    assert_eq!(normalize_to_json_pointer("items.0"), "/items/0");

    // Already in pointer format (no-op)
    assert_eq!(
        normalize_to_json_pointer("/$params/constants/DEATH_SA"),
        "/$params/constants/DEATH_SA"
    );

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
    assert_eq!(dot_notation_to_schema_pointer("field"), "#/field");

    // Already in schema pointer format - should return as-is
    assert_eq!(
        dot_notation_to_schema_pointer("#/illustration/properties/insured/properties/name"),
        "#/illustration/properties/insured/properties/name"
    );

    assert_eq!(
        dot_notation_to_schema_pointer("/illustration/properties/insured"),
        "/illustration/properties/insured"
    );

    // Explicit schema pointer with schema keywords should be preserved
    assert_eq!(
        dot_notation_to_schema_pointer("illustration.properties.header.properties.app_version.value"),
        "#/illustration/properties/header/properties/app_version/value"
    );

    assert_eq!(
        dot_notation_to_schema_pointer("illustration.properties.header.properties.app_version.type"),
        "#/illustration/properties/header/properties/app_version/type"
    );
}

#[test]
fn test_pointer_to_dot_notation() {
    // JSON Schema pointer to dotted notation
    assert_eq!(
        pointer_to_dot_notation("#/illustration/properties/insured/properties/ins_corrname"),
        "illustration.properties.insured.properties.ins_corrname"
    );

    assert_eq!(
        pointer_to_dot_notation("#/header/properties/form_number"),
        "header.properties.form_number"
    );

    // JSON pointer (without #) to dotted notation
    assert_eq!(pointer_to_dot_notation("/user/name"), "user.name");

    assert_eq!(pointer_to_dot_notation("/items/0"), "items.0");

    // Already in dotted notation - should return as-is
    assert_eq!(pointer_to_dot_notation("person.name"), "person.name");

    assert_eq!(
        pointer_to_dot_notation("illustration.insured.age"),
        "illustration.insured.age"
    );

    // Single field
    assert_eq!(pointer_to_dot_notation("field"), "field");

    // Empty
    assert_eq!(pointer_to_dot_notation(""), "");
}

#[test]
fn test_dollar_params_path_conversion() {
    // Test that paths starting with $ don't get /properties/ inserted
    assert_eq!(
        dot_notation_to_schema_pointer("$params.productName"),
        "#/$params/productName"
    );

    assert_eq!(
        dot_notation_to_schema_pointer("$params.constants.RATE"),
        "#/$params/constants/RATE"
    );

    assert_eq!(
        dot_notation_to_schema_pointer("$params.config.settings.timeout"),
        "#/$params/config/settings/timeout"
    );

    assert_eq!(
        dot_notation_to_schema_pointer("$defs.Person.name"),
        "#/$defs/Person/name"
    );
}

#[test]
fn test_canonicalize_schema_path() {
    // Basic data paths
    assert_eq!(
        canonicalize_schema_path("a.b.c"),
        "/a/properties/b/properties/c"
    );
    assert_eq!(
        canonicalize_schema_path("/a/b/c"),
        "/a/properties/b/properties/c"
    );
    assert_eq!(
        canonicalize_schema_path("illustration.insured.name"),
        "/illustration/properties/insured/properties/name"
    );

    // System paths
    assert_eq!(
        canonicalize_schema_path("$params.constants.RATE"),
        "/$params/constants/RATE"
    );
    assert_eq!(
        canonicalize_schema_path("/$params/constants/RATE"),
        "/$params/constants/RATE"
    );
    assert_eq!(
        canonicalize_schema_path("#/$params/constants/RATE"),
        "/$params/constants/RATE"
    );

    // Already canonical data paths
    assert_eq!(
        canonicalize_schema_path("/a/properties/b/properties/c"),
        "/a/properties/b/properties/c"
    );

    // Mixed dots and slashes (unusual but supported)
    assert_eq!(
        canonicalize_schema_path("a/b.c"),
        "/a/properties/b/properties/c"
    );

    // Schema refs
    assert_eq!(
        canonicalize_schema_path("#/illustration/properties/insured"),
        "/illustration/properties/insured"
    );

    // Simple top-level fields
    assert_eq!(canonicalize_schema_path("field"), "/field");
    assert_eq!(canonicalize_schema_path("/field"), "/field");

    // Explicit schema pointer with schema keywords should be preserved
    assert_eq!(
        canonicalize_schema_path("illustration.properties.header.properties.app_version.value"),
        "/illustration/properties/header/properties/app_version/value"
    );

    assert_eq!(
        canonicalize_schema_path("#/illustration/properties/header/properties/app_version/value"),
        "/illustration/properties/header/properties/app_version/value"
    );

    // Empty
    assert_eq!(canonicalize_schema_path(""), "");
}
