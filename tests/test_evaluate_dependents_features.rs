use json_eval_rs::JSONEval;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn load_fixture_schema() -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures/minimal_form.json");
    fs::read_to_string(path).expect("Failed to read minimal_form.json fixture")
}

#[test]
fn test_evaluate_dependents_recursive_chain() {
    let schema = load_fixture_schema();
    
    // Chain: occupation -> occupation_class -> risk_category
    // Logic from fixture:
    // occupation: "OFFICE" -> occupation_class: "1" -> risk_category: "Low"
    // occupation: "MANUAL" -> occupation_class: "2" -> risk_category: "Medium"
    // occupation: "HIGH_RISK" -> occupation_class: "3" -> risk_category: "High"

    // Initial state: OFFICE
    let data = r#"{ 
        "illustration": {
            "insured": {
                "occupation": "OFFICE",
                "occupation_class": "1",
                "risk_category": "Low"
            }
        }
    }"#;
    
    let mut je = JSONEval::new(&schema, None, Some(data)).unwrap();
    
    // Update occupation to MANUAL
    let new_data_snippet = r#"{ 
        "illustration": {
            "insured": {
                "occupation": "MANUAL"
            }
        }
    }"#;
    
    let changed_paths = vec!["illustration.insured.occupation".to_string()];
    
    let result = je.evaluate_dependents(&changed_paths, Some(new_data_snippet), None, true).unwrap();
    
    let changes = result.as_array().unwrap();
    
    // Verify occupation_class changed to "2"
    let class_change = changes.iter().find(|c| 
        c.get("$ref").and_then(|r| r.as_str()) == Some("illustration.insured.occupation_class")
    ).expect("occupation_class should change");
    
    assert_eq!(class_change.get("value"), Some(&Value::String("2".to_string())));
    
    // Verify risk_category changed to "Medium" (recursive side effect)
    let risk_change = changes.iter().find(|c| 
        c.get("$ref").and_then(|r| r.as_str()) == Some("illustration.insured.risk_category")
    ).expect("risk_category should change recursively");
    
    assert_eq!(risk_change.get("value"), Some(&Value::String("Medium".to_string())));
    
    // Check final data state
    let data = je.eval_data.data();
    assert_eq!(data.pointer("/illustration/insured/occupation_class"), Some(&Value::String("2".to_string())));
    assert_eq!(data.pointer("/illustration/insured/risk_category"), Some(&Value::String("Medium".to_string())));
}

#[test]
fn test_evaluate_dependents_keep_hidden_value() {
    let schema = load_fixture_schema();
    
    // Field: illustration.header.extra_comments
    // Condition: Hidden if form_number == "HIDE"
    // Config: keepHiddenValue = true
    
    // Initial state: form_number="SHOW", extra_comments="Existing Data"
    let data = r#"{ 
        "illustration": {
            "header": {
                "form_number": "SHOW",
                "extra_comments": "Existing Data"
            }
        }
    }"#;
    
    let mut je = JSONEval::new(&schema, None, Some(data)).unwrap();
    
    // Change form_number to HIDE
    // Note: replace_data_and_context replaces top-level objects, so we must provide the full object or use a different update method.
    // For this test, we provide the full header to ensure extra_comments isn't lost due to data loading.
    let new_data_snippet = r#"{ 
        "illustration": {
            "header": {
                "form_number": "HIDE",
                "extra_comments": "Existing Data"
            }
        }
    }"#;
    
    let changed_paths = vec!["illustration.header.form_number".to_string()];
    
    let result = je.evaluate_dependents(&changed_paths, Some(new_data_snippet), None, true).unwrap();
    
    let changes = result.as_array().unwrap();
    
    // We might accept a change event saying it's hidden, but crucially we must NOT see a "clear": true or a null value update
    // Note: implementation details might vary. The key requirement is DATA PRESERVATION.
    
    let comments_change = changes.iter().find(|c| 
        c.get("$ref").and_then(|r| r.as_str()) == Some("illustration.header.extra_comments")
    );
    
    if let Some(change) = comments_change {
        // If there IS a change event, make sure it's not clearing the value
        assert_ne!(change.get("clear"), Some(&Value::Bool(true)), "Should not clear value due to keepHiddenValue");
        assert_ne!(change.get("value"), Some(&Value::Null), "Should not set value to null");
        
        // It SHOULD report hidden state though (if logic emits it)
        if let Some(hidden) = change.get("$hidden") {
            assert_eq!(hidden, &Value::Bool(true), "Should report hidden state");
        }
    }
    
    // Verify Data is STILL there
    let data = je.eval_data.data();
    let comments_val = data.pointer("/illustration/header/extra_comments");
    assert_eq!(comments_val, Some(&Value::String("Existing Data".to_string())), "Data should be preserved");
}

#[test]
fn test_recursive_clearing() {
    // Test case: C changes -> B hidden -> B cleared -> A hidden -> A cleared
    let schema = load_fixture_schema();
    
    // Initial state: C=false, B=false, A="Initial"
    // All visible.
    let data = r#"{ 
        "illustration": {
            "header": {
                "recursive_test": {
                    "field_c": false,
                    "field_b": false,
                    "field_a": "Initial"
                }
            }
        }
    }"#;
    
    let mut je = JSONEval::new(&schema, None, Some(data)).unwrap();
    
    // Set C = true, but keep B and A to verify strict clearing logic
    // Logic: 
    //   B hidden if C==true. So B becomes hidden.
    //   A hidden if B is EMPTY. B gets cleared (null). So A becomes hidden.
    
    let new_data_snippet = r#"{ 
        "illustration": {
            "header": {
                "recursive_test": {
                    "field_c": true,
                    "field_b": false,
                    "field_a": "Initial"
                }
            }
        }
    }"#;
    
    let changed_paths = vec!["illustration.header.recursive_test.field_c".to_string()];
    
    let result = je.evaluate_dependents(&changed_paths, Some(new_data_snippet), None, true).unwrap();
    let changes = result.as_array().unwrap();
    
    let data = je.eval_data.data();
    
    // Verify B is cleared
    let val_b = data.pointer("/illustration/header/recursive_test/field_b");
    assert!(val_b.is_none() || val_b == Some(&Value::Null), "Field B should be cleared");
    
    // Verify A is cleared
    let val_a = data.pointer("/illustration/header/recursive_test/field_a");
    assert!(val_a.is_none() || val_a == Some(&Value::Null), "Field A should be cleared recursively");
    
    // Verify changes
    // Should see clear events for B and A
     
    let b_change = changes.iter().find(|c| 
        c.get("$ref").and_then(|r| r.as_str()) == Some("illustration.header.recursive_test.field_b")
    ).expect("Should report field_b change");
    
    assert_eq!(b_change.get("$hidden"), Some(&Value::Bool(true)));
    
    // A might be in the changes list, or just implicit. But structurally it should be reported.
    let a_change = changes.iter().find(|c| 
        c.get("$ref").and_then(|r| r.as_str()) == Some("illustration.header.recursive_test.field_a")
    );
     // depending on depth of recursion, it should be there because we iterate until stable or process queue
    assert!(a_change.is_some(), "Field A change should be reported");
    if let Some(c) = a_change {
         assert_eq!(c.get("$hidden"), Some(&Value::Bool(true)));
    }
}
