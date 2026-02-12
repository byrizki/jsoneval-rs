
#[cfg(feature = "wasm")]
#[cfg(test)]
mod tests {
    use json_eval_rs::wasm::types::JSONEvalWasm;
    use serde_json::json;

    #[test]
    fn test_wasm_validate_integrity() {
        // Correct schema with pattern rule to force all fields
        let schema = json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "rules": {
                        "pattern": { 
                            "value": "^[0-9]+$", 
                            "message": "Must be digits",
                            "code": "code.pattern"
                        }
                    }
                }
            }
        });
        
        let schema_str = serde_json::to_string(&schema).unwrap();
        
        // Initialize WASM wrapper
        // Note: JSONEvalWasm::new args: (schema, context, data)
        let mut wasm_eval = JSONEvalWasm::new(&schema_str, None, None).expect("Failed to create WASM instance");
        
        // Invalid data
        let data = json!({ "code": "abc" });
        let data_str = serde_json::to_string(&data).unwrap();
        
        // Validate using the testable helper
        let result = wasm_eval.validate_to_value(&data_str, None, None).expect("Validation failed");
        
        println!("WASM Validate Output: {}", result);
        
        assert_eq!(result["has_error"], true);
        
        let error = &result["error"]["code"];
        
        // Check all keys required by the interface
        assert_eq!(error["type"], "pattern");
        assert_eq!(error["message"], "Must be digits");
        assert_eq!(error["code"], "code.pattern");
        assert!(error.get("pattern").is_some(), "pattern field missing");
        assert!(error.get("fieldValue").is_some(), "fieldValue field missing");
        assert_eq!(error["fieldValue"], "abc");
        assert!(error.get("data").is_some() || error.get("data").is_none()); // data is optional, check presence if set in rule (it wasn't here)
        
        // Check keys needed for interface compliance basically
        let obj = error.as_object().unwrap();
        assert!(obj.contains_key("type"));
        assert!(obj.contains_key("message"));
        assert!(obj.contains_key("code"));
        assert!(obj.contains_key("pattern"));
        assert!(obj.contains_key("fieldValue"));
    }
}
