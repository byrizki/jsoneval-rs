
use json_eval_rs::JSONEval;
use serde_json::{json, Value};

#[test]
fn test_disable_cache_propagates_to_subforms() {
    // Schema with a subform (array with items)
    let schema = json!({
        "riders": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                }
            }
        }
    }).to_string();

    // 1. Create JSONEval
    let mut eval = JSONEval::new(&schema, None, None).unwrap();
    
    // ensure subforms exist
    assert!(!eval.subforms.is_empty(), "Subforms should exist for array items");
    
    // 2. Disable cache
    eval.disable_cache();
    
    // 3. Check if main cache is disabled
    assert!(!eval.is_cache_enabled(), "Main cache should be disabled");
    
    // 4. Check if subform cache is disabled
    for (path, subform) in eval.subforms.iter() {
        assert!(!subform.is_cache_enabled(), "Subform at {} should have cache disabled", path);
    }
}

#[test]
fn test_context_dependency_with_cache_disabled() {
    let schema = json!({
        "$params": {
            "accessList": {
                "$evaluation": {
                    "if": [
                        {
                            "and": [
                                {
                                    "in": [
                                        { "$ref": "$context.profile.sob" },
                                        ["AG", "AP"]
                                    ]
                                },
                                {
                                    "==": [
                                        { "$ref": "$context.profile.agentFlag" },
                                        "true"
                                    ]
                                }
                            ]
                        },
                        { "return": ["AG", "AP"] },
                        { "return": [] }
                    ]
                }
            }
        }
    }).to_string();

    // Context 1: Should evaluate to ["AG", "AP"]
    let context1 = json!({
        "profile": {
            "sob": "AG",
            "agentFlag": "true"
        }
    }).to_string();

    // Context 2: Should evaluate to []
    let context2 = json!({
        "profile": {
            "sob": "XY",
            "agentFlag": "false"
        }
    }).to_string();

    // 1. Initialize with Context 1
    let mut eval = JSONEval::new(&schema, Some(&context1), None).unwrap();
    eval.evaluate("{}", Some(&context1), None).unwrap();
    
    // Verify initial state
    let data = eval.eval_data.data().as_object().unwrap();
    let params = data.get("$params").unwrap().as_object().unwrap();
    let access_list = params.get("accessList").unwrap();
    assert_eq!(access_list, &json!(["AG", "AP"]), "Initial context 1 should allow access");

    // 2. Disable Cache
    eval.disable_cache();
    assert!(!eval.is_cache_enabled(), "Cache should be disabled");

    // 3. Switch to Context 2
    eval.evaluate("{}", Some(&context2), None).unwrap();
    
    // Verify update
    let data = eval.eval_data.data().as_object().unwrap();
    let params = data.get("$params").unwrap().as_object().unwrap();
    let access_list = params.get("accessList").unwrap();
    assert_eq!(access_list, &json!([]), "Switching to context 2 should deny access");

    // 4. Switch back to Context 1
    eval.evaluate("{}", Some(&context1), None).unwrap();
    
    // Verify update again
    let data = eval.eval_data.data().as_object().unwrap();
    let params = data.get("$params").unwrap().as_object().unwrap();
    let access_list = params.get("accessList").unwrap();
    assert_eq!(access_list, &json!(["AG", "AP"]), "Switching back to context 1 should allow access");
}
