use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_selective_evaluation_basic() {
    let schema = json!({
        "type": "object",
        "properties": {
            "input1": { "type": "string" },
            "input2": { "type": "string" },
            "computed1": {
                "type": "string",
                "value": {
                    "$evaluation": {
                        "concat": [
                            { "var": "input1" },
                            "_processed"
                        ]
                    }
                }
            },
            "computed2": {
                "type": "string",
                "value": {
                    "$evaluation": {
                        "concat": [
                            { "var": "input2" },
                            "_processed"
                        ]
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let data = json!({
        "input1": "value1",
        "input2": "value2",
        "computed1": "",
        "computed2": ""
    });
    let data_str = serde_json::to_string(&data).unwrap();

    let mut eval = JSONEval::new(&schema_str, None, Some(&data_str)).unwrap();

    // 1. Full evaluation
    eval.evaluate(&data_str, None, None).unwrap();
    
    // Check results
    let evaluated = eval.get_evaluated_schema(false);
    assert_eq!(*evaluated.pointer("/properties/computed1/value").unwrap(), json!("value1_processed"));
    assert_eq!(*evaluated.pointer("/properties/computed2/value").unwrap(), json!("value2_processed"));
    
    // 2. Update inputs, but selectively evaluate ONLY computed1
    let data_updated = json!({
        "input1": "value1_updated",
        "input2": "value2_updated",
    });
    // For selective eval on data that hasn't changed structure, we can just pass changed data via helper or NEW eval
    // But here we are reusing `eval`. `evaluate` will replace data.
    // NOTE: If we pass data w/o computed fields, `evaluate` keeps existing schema values unless re-evaluated.
    let data_updated_str = serde_json::to_string(&data_updated).unwrap();
    
    let paths = vec!["computed1".to_string()];
    eval.evaluate(&data_updated_str, None, Some(&paths)).unwrap();
    
    let evaluated_2 = eval.get_evaluated_schema(false);
    let res1 = evaluated_2.pointer("/properties/computed1/value").unwrap();
    let res2 = evaluated_2.pointer("/properties/computed2/value").unwrap();
    
    assert_eq!(*res1, json!("value1_updated_processed"), "computed1 should be re-evaluated");
    // computed2 was NOT re-evaluated. It retains previous value "value2_processed".
    assert_eq!(*res2, json!("value2_processed"), "computed2 should NOT be re-evaluated (should hold old value)");
    
    // 3. Evaluate computed2
    let paths2 = vec!["computed2".to_string()];
    eval.evaluate(&data_updated_str, None, Some(&paths2)).unwrap();
    
    let evaluated_3 = eval.get_evaluated_schema(false);
    let res2_new = evaluated_3.pointer("/properties/computed2/value").unwrap();
    assert_eq!(*res2_new, json!("value2_updated_processed"), "computed2 should now be re-evaluated");
}

#[test]
fn test_selective_evaluation_nested() {
    let schema = json!({
        "type": "object",
        "properties": {
            "root": {
                "type": "object",
                "properties": {
                    "child1_in": { "type": "string" },
                    "child2_in": { "type": "string" },
                    "child1_out": {
                        "type": "string",
                        "value": { "$evaluation": { "concat": [{"var": "root.child1_in"}, "_1"] } }
                    },
                    "child2_out": {
                        "type": "string",
                        "value": { "$evaluation": { "concat": [{"var": "root.child2_in"}, "_2"] } }
                    }
                }
            }
        }
    });
    
    let schema_str = serde_json::to_string(&schema).unwrap();
    let data = json!({
        "root": {
            "child1_in": "a",
            "child2_in": "b",
            "child1_out": "",
            "child2_out": ""
        }
    });
    let data_str = serde_json::to_string(&data).unwrap();
    
    let mut eval = JSONEval::new(&schema_str, None, Some(&data_str)).unwrap();
    eval.evaluate(&data_str, None, None).unwrap(); // Full eval
    
    // Update data (without computed values provided in input, they should be retained/recalculated)
    let data_upd = json!({
        "root": {
            "child1_in": "a_new",
            "child2_in": "b_new"
        }
    });
    let data_upd_str = serde_json::to_string(&data_upd).unwrap();
    
    // Eval only root.child2_out
    let paths = vec!["root.child2_out".to_string()];
    eval.evaluate(&data_upd_str, None, Some(&paths)).unwrap();
    
    // Check
    let evaluated = eval.get_evaluated_schema(false);
    let res1 = evaluated.pointer("/properties/root/properties/child1_out/value").unwrap();
    let res2 = evaluated.pointer("/properties/root/properties/child2_out/value").unwrap();
    
    // child1_out was NOT re-evaluated. It retains the OLD value from previous run "a_1".
    // child2_out WAS re-evaluated. It should be "b_new_2".
    assert_eq!(*res1, json!("a_1"), "child1_out should NOT be updated (kept old value)");
    assert_eq!(*res2, json!("b_new_2"), "child2_out SHOULD be updated");
}
