#[cfg(feature = "ffi")]
use json_eval_rs::ffi::*;
use serde_json::json;
use std::ffi::CString;

#[test]
fn test_ffi_methods_parity() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { 
                "type": "string",
                "options": ["A", "B"]
            },
            "riders": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" }
                    }
                },
                "$layout": {
                    "type": "VerticalLayout",
                    "elements": [
                        { "$ref": "#/riders/properties/name" }
                    ]
                }
            },
            "form": {
                "$layout": {
                    "type": "VerticalLayout",
                    "elements": [
                        { "$ref": "name" }
                    ]
                }
            }
        }
    });

    let schema_str = CString::new(serde_json::to_string(&schema).unwrap()).unwrap();
    
    unsafe {
        // 1. json_eval_new
        let handle = json_eval_new(schema_str.as_ptr(), std::ptr::null(), std::ptr::null());
        assert!(!handle.is_null(), "Handle should not be null");

        // 2. json_eval_evaluate
        let data = json!({
            "riders": [{"name": "Rider 1"}]
        });
        let data_str = CString::new(serde_json::to_string(&data).unwrap()).unwrap();
        let result = json_eval_evaluate(handle, data_str.as_ptr(), std::ptr::null(), std::ptr::null());
        assert!(result.success, "Evaluation should succeed: {}", if result.error.is_null() { "none" } else { std::ffi::CStr::from_ptr(result.error).to_str().unwrap() });
        json_eval_free_result(result);

        // 3. json_eval_get_evaluated_schema_resolved
        let result = json_eval_get_evaluated_schema_resolved(handle);
        assert!(result.success, "Get evaluated schema resolved should succeed");
        assert!(result.data_len > 0, "Should return data");
        let schema_bytes = std::slice::from_raw_parts(result.data_ptr, result.data_len);
        let schema_val: serde_json::Value = serde_json::from_slice(schema_bytes).unwrap();
        // Check if $fullpath is present in the layout elements (resolved)
        assert!(schema_val.pointer("/properties/form/$layout/elements/0/$fullpath").is_some(), "$fullpath should be present in resolved layout");
        assert_eq!(schema_val.pointer("/properties/form/$layout/elements/0/$fullpath").and_then(|v| v.as_str()), Some("properties.name"));
        json_eval_free_result(result);

        // 4. json_eval_get_resolved_layout
        let result = json_eval_get_resolved_layout(handle);
        assert!(result.success, "Get resolved layout should succeed");
        assert!(result.data_len > 0, "Should return layout entries");
        json_eval_free_result(result);

        // 5. json_eval_get_field_options
        let field_path = CString::new("properties.name").unwrap();
        let result = json_eval_get_field_options(handle, field_path.as_ptr());
        assert!(result.success, "Get field options should succeed");
        json_eval_free_result(result);

        // 6. Subform methods parity
        let subform_path = CString::new("#/riders/0").unwrap();
        
        // 6a. json_eval_get_resolved_layout_subform
        let result = json_eval_get_resolved_layout_subform(handle, subform_path.as_ptr());
        assert!(result.success, "Get resolved layout subform should succeed");
        json_eval_free_result(result);

        // 6b. json_eval_get_evaluated_schema_resolved_subform
        let result = json_eval_get_evaluated_schema_resolved_subform(handle, subform_path.as_ptr());
        assert!(result.success, "Get evaluated schema resolved subform should succeed");
        assert!(result.data_len > 0);
        let subform_schema_bytes = std::slice::from_raw_parts(result.data_ptr, result.data_len);
        let subform_schema: serde_json::Value = serde_json::from_slice(subform_schema_bytes).unwrap();
        // Check for $fullpath in subform layout
        // In the subform instance, the schema is wrapped in the root key (riders)
        assert!(subform_schema.pointer("/riders/$layout/elements/0/$fullpath").is_some(), "$fullpath should be present in subform resolved layout");
        json_eval_free_result(result);

        // 7. Cleanup
        json_eval_free(handle);
    }
}
