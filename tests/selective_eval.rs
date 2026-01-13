use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn test_selective_evaluation_basic() {
    let schema = json!({
        "$params": {
            "type": "illustration",
            "accessList": {
                "$evaluation": {
                    "if": [
                        {
                            "and": [
                            {
                                "in": [
                                {
                                    "$ref": "$context.agentProfile.sob"
                                },
                                [
                                    "AG",
                                    "AP"
                                ]
                                ]
                            },
                            {
                                "==": [
                                {
                                    "$ref": "$context.agentProfile.agentFlag"
                                },
                                "true"
                                ]
                            }
                            ]
                        },
                        {
                            "return": [
                            "AG",
                            "AP"
                            ]
                        },
                        {
                            "return": []
                        }
                        ]
                    }
                },
            "constants": {
                "POL_DURATION": 8
            },
            "others": {
                "MIN_SA": {
                    "$evaluation": {
                    "/": [
                        {
                        "*": [
                            {
                            "/": [
                                4000000,
                                {
                                "$ref": "#/illustration/properties/product_benefit/properties/benefit_type/properties/prem_freq"
                                }
                            ]
                            },
                            1000
                        ]
                        },
                        {
                        "*": [
                            {
                            "$ref": "#/$params/others/OTHER_SA"
                            },
                            {
                            "$ref": "#/$params/others/MODAL_FACTOR_CALC"
                            }
                        ]
                        }
                    ]
                    }
                }
            }
        }
    });

    let schema_str = serde_json::to_string(&schema).unwrap();
    let data = json!({});
    let data_str = serde_json::to_string(&data).unwrap();
    let ctx = json!({
        "agentProfile": {
            "agentFlag": "true",
            "sob": "AP"
        }
    });
    let ctx_str = serde_json::to_string(&ctx).unwrap();

    let mut eval = JSONEval::new(&schema_str, Some(&ctx_str), Some(&data_str)).unwrap();

    // 1. Full evaluation
    eval.evaluate(&data_str, Some(&ctx_str), None, None).unwrap();
    
    // Check results
    let evaluated = eval.get_evaluated_schema(false);
    assert_eq!(*evaluated.pointer("/$params/accessList").unwrap(), json!(["AG", "AP"]));


    // 2. Selective evaluation, not target the value must be persists
    let nctx = json!({
        "agentProfile": {
            "agentFlag": "false",
            "sob": "AP"
        }
    });
    let nctx_str = serde_json::to_string(&nctx).unwrap();
    eval.evaluate(&data_str, Some(&nctx_str), Some(&["$params.others.MIN_SA".to_string()]), None).unwrap();
    
    // Check results
    let evaluated = eval.get_evaluated_schema(false);
    assert_eq!(*evaluated.pointer("/$params/accessList").unwrap(), json!(["AG", "AP"]));

    // 3. Selective evaluation, target the value must be re-evaluated
    let nctx = json!({
        "agentProfile": {
            "agentFlag": "false",
            "sob": "AP"
        }
    });
    let nctx_str = serde_json::to_string(&nctx).unwrap();
    eval.evaluate(&data_str, Some(&nctx_str), Some(&["$params.accessList".to_string()]), None).unwrap();
    
    // Check results
    let evaluated = eval.get_evaluated_schema(false);
    assert_eq!(*evaluated.pointer("/$params/accessList").unwrap(), json!([]));
}
