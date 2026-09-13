use serde_json::Value;
use std::fs;

/// Load the minimal form schema from fixtures
pub fn load_minimal_form_schema() -> String {
    let schema_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/minimal_form.json"
    );
    fs::read_to_string(schema_path).expect("Failed to read minimal_form.json")
}

/// Get sample data for minimal form - basic user
pub fn get_minimal_form_data() -> Value {
    serde_json::json!({
        "form": {
            "header": {
                "form_number": "TEST001",
                "form_date": "2024-01-15"
            },
            "user": {
                "name": "John Doe",
                "date_of_birth": "1990-05-15",
                "age": 33,
                "gender": "M",
                "is_smoker": false,
                "occupation": "OFFICE",
                "occupation_class": "1",
                "risk_category": "Low"
            },
            "details": {
                "has_additional_option": false,
                "option_type": "",
                "option_details": {
                    "amount": 0,
                    "fee": 0,
                    "custom_options": {
                        "option_a": "",
                        "option_b": false
                    }
                }
            }
        }
    })
}

/// Get sample data with advanced option
#[allow(dead_code)]
pub fn get_advanced_option_data() -> Value {
    serde_json::json!({
        "form": {
            "header": {
                "form_number": "TEST002",
                "form_date": "2024-01-20"
            },
            "user": {
                "name": "Jane Smith",
                "date_of_birth": "1985-03-20",
                "age": 38,
                "gender": "F",
                "is_smoker": true,
                "occupation": "PROFESSIONAL",
                "occupation_class": "1",
                "risk_category": "High"
            },
            "details": {
                "has_additional_option": true,
                "option_type": "ADVANCED",
                "option_details": {
                    "amount": 100000,
                    "fee": 5000,
                    "custom_options": {
                        "option_a": "",
                        "option_b": false
                    }
                }
            }
        }
    })
}

/// Get sample data with custom option
#[allow(dead_code)]
pub fn get_custom_option_data() -> Value {
    serde_json::json!({
        "form": {
            "header": {
                "form_number": "TEST003",
                "form_date": "2024-02-01"
            },
            "user": {
                "name": "Bob Wilson",
                "date_of_birth": "1995-08-10",
                "age": 28,
                "gender": "M",
                "is_smoker": false,
                "occupation": "MANUAL",
                "occupation_class": "2",
                "risk_category": "Medium"
            },
            "details": {
                "has_additional_option": true,
                "option_type": "CUSTOM",
                "option_details": {
                    "amount": 75000,
                    "fee": 3750,
                    "custom_options": {
                        "option_a": "Custom config",
                        "option_b": true
                    }
                }
            }
        }
    })
}

/// Helper to wrap data in form structure
#[allow(dead_code)]
pub fn wrap_in_form(data: Value) -> Value {
    serde_json::json!({
        "form": data
    })
}
