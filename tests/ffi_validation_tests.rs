
#[cfg(feature = "ffi")]
#[cfg(test)]
mod tests {
    use json_eval_rs::ffi::core::{json_eval_new, json_eval_free, json_eval_free_result};
    use json_eval_rs::ffi::evaluation::json_eval_validate;
    use json_eval_rs::ffi::layout::json_eval_validate_paths;
    use std::ffi::CString;

    #[test]
    fn test_ffi_validate_integrity() {
        unsafe {
            let schema = r#"{
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "rules": {
                            "pattern": { "value": "^[0-9]+$", "message": "Must be digits" }
                        }
                    }
                }
            }"#;
            let schema_cstr = CString::new(schema).unwrap();
            let handle = json_eval_new(schema_cstr.as_ptr(), std::ptr::null(), std::ptr::null());
            
            let data = r#"{ "code": "abc" }"#; 
            let data_cstr = CString::new(data).unwrap();
            
            // Test 1: json_eval_validate
            let result = json_eval_validate(handle, data_cstr.as_ptr(), std::ptr::null());
            assert!(result.success);
            
            let slice = std::slice::from_raw_parts(result.data_ptr, result.data_len as usize);
            let json_str = std::str::from_utf8(slice).unwrap();
            println!("Validate Output: {}", json_str);
            
            let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
            let error = &parsed["error"]["code"];
            
            assert_eq!(error["type"], "pattern");
            assert!(error.get("fieldValue").is_some());
            assert_eq!(error["fieldValue"].as_str().unwrap_or(""), "abc"); 

            json_eval_free_result(result);
            
            // Test 2: json_eval_validate_paths
            // paths_json must be a JSON array string ["code"]
            let paths = r#"["code"]"#;
            let paths_cstr = CString::new(paths).unwrap();
            
            let result_paths = json_eval_validate_paths(handle, data_cstr.as_ptr(), std::ptr::null(), paths_cstr.as_ptr());
            assert!(result_paths.success);
            
            let slice_paths = std::slice::from_raw_parts(result_paths.data_ptr, result_paths.data_len as usize);
            let json_str_paths = std::str::from_utf8(slice_paths).unwrap();
            println!("Validate Paths Output: {}", json_str_paths);
            
            let parsed_paths: serde_json::Value = serde_json::from_str(json_str_paths).unwrap();
            let error_paths = &parsed_paths["error"]["code"];
            
            assert_eq!(error_paths["type"], "pattern");
            assert!(error_paths.get("fieldValue").is_some());
            assert_eq!(error_paths["fieldValue"].as_str().unwrap_or(""), "abc");

            json_eval_free_result(result_paths);
            json_eval_free(handle);
        }
    }
}
