use serde_json::Value;
use std::fs;

/// Load the minimal form schema from fixtures
pub fn load_minimal_form_schema() -> String {
    let schema_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/minimal_form.json");
    fs::read_to_string(schema_path)
        .expect("Failed to read minimal_form.json")
}

/// Get sample data for minimal form - basic insured person
pub fn get_minimal_form_data() -> Value {
    serde_json::json!({
        "illustration": {
            "header": {
                "form_number": "TEST001",
                "form_date": "2024-01-15"
            },
            "insured": {
                "name": "John Doe",
                "date_of_birth": "1990-05-15",
                "age": 33,
                "gender": "M",
                "is_smoker": false,
                "occupation": "OFFICE",
                "occupation_class": "1",
                "risk_category": "Low"
            },
            "policy_container": {
                "has_additional_coverage": false,
                "coverage_type": "",
                "coverage_details": {
                    "sum_assured": 0,
                    "premium_amount": 0,
                    "custom_options": {
                        "option_a": "",
                        "option_b": false
                    }
                }
            }
        }
    })
}

/// Get sample data with premium coverage
#[allow(dead_code)]
pub fn get_premium_coverage_data() -> Value {
    serde_json::json!({
        "illustration": {
            "header": {
                "form_number": "TEST002",
                "form_date": "2024-01-20"
            },
            "insured": {
                "name": "Jane Smith",
                "date_of_birth": "1985-03-20",
                "age": 38,
                "gender": "F",
                "is_smoker": true,
                "occupation": "PROFESSIONAL",
                "occupation_class": "1",
                "risk_category": "High"
            },
            "policy_container": {
                "has_additional_coverage": true,
                "coverage_type": "PREMIUM",
                "coverage_details": {
                    "sum_assured": 100000,
                    "premium_amount": 5000,
                    "custom_options": {
                        "option_a": "",
                        "option_b": false
                    }
                }
            }
        }
    })
}

/// Get sample data with custom coverage
#[allow(dead_code)]
pub fn get_custom_coverage_data() -> Value {
    serde_json::json!({
        "illustration": {
            "header": {
                "form_number": "TEST003",
                "form_date": "2024-02-01"
            },
            "insured": {
                "name": "Bob Wilson",
                "date_of_birth": "1995-08-10",
                "age": 28,
                "gender": "M",
                "is_smoker": false,
                "occupation": "MANUAL",
                "occupation_class": "2",
                "risk_category": "Medium"
            },
            "policy_container": {
                "has_additional_coverage": true,
                "coverage_type": "CUSTOM",
                "coverage_details": {
                    "sum_assured": 75000,
                    "premium_amount": 3750,
                    "custom_options": {
                        "option_a": "Custom config",
                        "option_b": true
                    }
                }
            }
        }
    })
}

/// Helper to wrap data in illustration structure
#[allow(dead_code)]
pub fn wrap_in_illustration(data: Value) -> Value {
    serde_json::json!({
        "illustration": data
    })
}
