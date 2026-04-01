use json_eval_rs::JSONEval;

use json_eval_rs::jsoneval::parsed_schema::ParsedSchema;
use std::sync::Arc;

fn create_eval_helper(
    schema: &str,
    context: Option<&str>,
    data: Option<&str>,
) -> Result<JSONEval, String> {
    let parsed = Arc::new(ParsedSchema::parse(schema)?);
    JSONEval::with_parsed_schema(parsed, context, data)
}

use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn load_file_content(relative_path: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative_path);
    fs::read_to_string(path).expect("Failed to read file")
}

#[test]
fn test_zip_scenario_base_prem_update() {
    // 1. Load schema @[samples/zip.json]
    let schema_str = load_file_content("samples/zip.json");

    // 2. Preload value with @[samples/zip-data.json]
    let data_str = load_file_content("samples/zip-data.json");

    // Initialize JSONEval
    let mut eval =
        create_eval_helper(&schema_str, None, Some(&data_str)).expect("Failed to create JSONEval");

    // Initial evaluation
    eval.evaluate(&data_str, None, None, None)
        .expect("Initial evaluation failed");

    // Helper to get BASE_PREM
    let get_base_prem = |state: &Value| -> Option<f64> {
        let node = state.pointer("/$params/others/BASE_PREM")?;
        if let Some(v) = node.get("value") {
            v.as_f64()
        } else {
            node.as_f64()
        }
    };

    let initial_eval = eval.get_evaluated_schema(false);
    let initial_base_prem = get_base_prem(&initial_eval);

    // 3. Modify field value uang pertanggungan (tf)
    let mut data_json: Value = serde_json::from_str(&data_str).unwrap();
    let current_tf = data_json["illustration"]["product_benefit"]["benefit_type"]["tf"]
        .as_f64()
        .expect("tf should be a number");
    let new_tf = current_tf + 100_000_000.0;
    data_json["illustration"]["product_benefit"]["benefit_type"]["tf"] = serde_json::json!(new_tf);
    let updated_data_str = data_json.to_string();

    // Use evaluate_dependents to trigger update from the modified path
    let tf_path = "illustration.product_benefit.benefit_type.tf";
    eval.evaluate_dependents(
        &[tf_path.to_string()],
        Some(&updated_data_str),
        None,
        true,
        None,
        None,
        true,
    )
    .expect("evaluate_dependents failed");

    let final_eval = eval.get_evaluated_schema(false);
    let final_base_prem = get_base_prem(&final_eval);

    assert_ne!(
        initial_base_prem, final_base_prem,
        "BASE_PREM should have been updated"
    );
}

const RIDERS_SUBFORM_PATH: &str = "#/illustration/properties/product_benefit/properties/riders";

#[test]
fn test_zpp_scenario_base_prem_update() {
    // 1. Load schema
    let schema_str = load_file_content("samples/zpp.json");

    // 2. Preload value
    let data_str = load_file_content("samples/zpp-data.json");

    // Initialize JSONEval
    let mut eval =
        create_eval_helper(&schema_str, None, Some(&data_str)).expect("Failed to create JSONEval");

    // Verify the riders subform exists
    assert!(
        eval.has_subform(RIDERS_SUBFORM_PATH),
        "riders subform should exist"
    );

    // Helper: apply evaluate_dependents changes back into the raw data JSON
    let apply_changes = |data_json: &mut Value, changes: Value| {
        let Value::Array(change_list) = changes else {
            return;
        };
        for change in change_list {
            let Some(obj) = change.as_object() else {
                continue;
            };
            let Some(Value::String(ref_path)) = obj.get("$ref") else {
                continue;
            };

            let parts: Vec<&str> = ref_path.split('.').collect();
            if parts.is_empty() {
                continue;
            }

            let mut current = &mut *data_json;
            let mut valid = true;
            for part in &parts[..parts.len() - 1] {
                if current.is_object() {
                    if !current.as_object().unwrap().contains_key(*part) {
                        current
                            .as_object_mut()
                            .unwrap()
                            .insert((*part).to_string(), serde_json::json!({}));
                    }
                    current = current.get_mut(*part).unwrap();
                } else if current.is_array() {
                    if let Ok(idx) = part.parse::<usize>() {
                        current = current.get_mut(idx).unwrap();
                    } else {
                        valid = false;
                        break;
                    }
                } else {
                    valid = false;
                    break;
                }
            }

            if valid {
                let last_part = parts.last().unwrap();
                if let Some(map) = current.as_object_mut() {
                    if let Some(Value::Bool(true)) = obj.get("clear") {
                        map.remove(*last_part);
                    } else if let Some(val) = obj.get("value") {
                        map.insert((*last_part).to_string(), val.clone());
                    }
                }
            }
        }
    };

    // Helper to get PREM_WOP_BASIC_PER_PAY from evaluated schema
    let get_base_prem = |state: &Value| -> Option<f64> {
        let node = state.pointer("/$params/others/PREM_WOP_BASIC_PER_PAY")?;
        if let Some(v) = node.get("value") {
            v.as_f64()
        } else {
            node.as_f64()
        }
    };

    // =========================================================================
    // Run 1: Initial evaluation
    // =========================================================================
    let t = std::time::Instant::now();
    eval.evaluate(&data_str, None, None, None)
        .expect("Initial evaluation failed");
    println!("[run 1] initial evaluate: {:?}", t.elapsed());

    let initial_eval = eval.get_evaluated_schema(false);
    let initial_base_prem = get_base_prem(&initial_eval);

    let mut data_json: Value = serde_json::from_str(&data_str).unwrap();

    // =========================================================================
    // Run 2: evaluate_dependents for ill_sign change
    // =========================================================================
    let current_sign = data_json["illustration"]["product_benefit"]["product"]["ill_sign"]
        .as_bool()
        .expect("ill_sign should be a boolean");
    data_json["illustration"]["product_benefit"]["product"]["ill_sign"] =
        serde_json::json!(!current_sign);
    let updated_data_str = data_json.to_string();

    let t = std::time::Instant::now();
    let deps_result = eval
        .evaluate_dependents(
            &["illustration.product_benefit.product.ill_sign".to_string()],
            Some(&updated_data_str),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");
    println!("[run 2] evaluate_dependents (ill_sign): {:?}", t.elapsed());

    // Assert that deps_result includes value changes for rider loading_benefit.first_prem
    // (run_subform_pass with re_evaluate:true now propagates parent evaluate_internal changes into riders)
    let deps_array = deps_result
        .as_array()
        .expect("deps_result should be an array");
    for expected_ref in &[
        "illustration.product_benefit.riders.0.loading_benefit.first_prem",
        "illustration.product_benefit.riders.1.loading_benefit.first_prem",
        "illustration.product_benefit.riders.2.loading_benefit.first_prem",
    ] {
        let matched_item = deps_array.iter().find(|item| {
            item.get("$ref")
                .and_then(|r| r.as_str())
                .map(|r| r == *expected_ref)
                .unwrap_or(false)
                && item.get("value").is_some()
        });

        if let Some(item) = matched_item {
            println!(
                "Value for {}: {:?}",
                expected_ref,
                item.get("value").unwrap()
            );
        }

        assert!(
            matched_item.is_some(),
            "deps_result missing value change for '{}'. Rider refs in result: {:#?}",
            expected_ref,
            deps_array
                .iter()
                .filter_map(|i| i.get("$ref").and_then(|r| r.as_str()))
                .filter(|r| r.contains("riders"))
                .collect::<Vec<_>>()
        );
    }

    // Print any wop_rider_premi changes from ill_sign cascade
    if let Value::Array(ref arr) = deps_result {
        for item in arr {
            if let Some(r) = item.get("$ref").and_then(|v| v.as_str()) {
                if r.contains("wop_rider_premi") || r.contains("wop_rider_benefit") {
                    println!("  [run 2 deps] {}: {:?}", r, item.get("value"));
                }
            }
        }
    }
    apply_changes(&mut data_json, deps_result);

    // =========================================================================
    // Run 3: full re-evaluate after ill_sign cascade (expect tables cache hit)
    // =========================================================================
    let t = std::time::Instant::now();
    eval.evaluate(&data_json.to_string(), None, None, None)
        .expect("Full evaluation after ill_sign failed");
    println!(
        "[run 3] full evaluate after deps (ill_sign) (tables cache hit): {:?}",
        t.elapsed()
    );

    let full_eval = eval.get_evaluated_schema(false);
    let full_eval_base_prem = get_base_prem(&full_eval);

    assert_eq!(
        initial_base_prem, full_eval_base_prem,
        "PREM_WOP_BASIC_PER_PAY should not have been updated"
    );

    // Deeply check no $evaluation object leftover
    fn assert_no_evaluation(v: &Value, path: String) {
        match v {
            Value::Object(map) => {
                for (key, val) in map {
                    let current_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    if key == "$evaluation" {
                        if !current_path.contains("items")
                            && !current_path.contains("dependents")
                            && !current_path.contains("$layout")
                        {
                            assert!(
                                key != "$evaluation",
                                "Found $evaluation key in schema at path: {}",
                                current_path
                            );
                        }
                    } else {
                        assert_no_evaluation(val, current_path);
                    }
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    assert_no_evaluation(val, format!("{}[{}]", path, i));
                }
            }
            _ => {}
        }
    }

    assert_no_evaluation(&full_eval, String::new());

    // =========================================================================
    // Run 3a: evaluate each rider subform individually after ill_sign cascade
    // Expect: all cache hits — ill_sign does NOT affect WOP table inputs
    // =========================================================================
    {
        let riders = data_json["illustration"]["product_benefit"]["riders"]
            .as_array()
            .expect("riders should be an array")
            .clone();

        let t_total = std::time::Instant::now();
        for (idx, rider_item) in riders.iter().enumerate() {
            // Subform schema has `riders` at root — pass subform-rooted data
            let subform_data = serde_json::json!({ "riders": rider_item });
            let subform_data_str = subform_data.to_string();
            let t = std::time::Instant::now();
            let subform_path = format!("{}.{}", RIDERS_SUBFORM_PATH, idx);
            eval.evaluate_subform(&subform_path, &subform_data_str, None, None, None)
                .unwrap_or_else(|e| panic!("evaluate_subform rider[{}] failed: {}", idx, e));
            println!("  [run 3a] rider[{}]: {:?}", idx, t.elapsed());

            // Assert that riders 0, 1, 2 have a non-null loading_benefit.first_prem in the evaluated schema
            if [0usize, 1, 2].contains(&idx) {
                // Pass the subform_path with the index so the schema getter swaps the cache properly
                let subform_schema = eval.get_evaluated_schema_subform(&subform_path, false);

                let first_prem = subform_schema
                    .pointer("/riders/properties/loading_benefit/properties/first_prem/value");

                println!(
                    "  [run 3a] rider[{}] first_prem schema value: {:?}",
                    idx, first_prem
                );

                assert!(
                    first_prem.map(|v| !v.is_null()).unwrap_or(false),
                    "rider[{}]: expected 'riders.properties.loading_benefit.first_prem' to have a non-null value in evaluated schema, but got {:?}",
                    idx,
                    first_prem
                );
            }
        }
        println!(
            "[run 3a] subform riders after ill_sign (expect cache hits): total={:?}",
            t_total.elapsed()
        );
    }

    // =========================================================================
    // Run 4: evaluate_dependents for wop_basic_benefit change (WOP02 -> WOP03)
    // =========================================================================
    let current_basic_wop_benefit = data_json["illustration"]["product_benefit"]
        ["wop_basic_benefit"]
        .as_str()
        .map(String::from)
        .unwrap_or_default();
    let current_basic_wop_premi = data_json["illustration"]["product_benefit"]["wop_basic_premi"]
        .as_f64()
        .unwrap_or(0.0);
    let current_rider1_wop_benefit = data_json["illustration"]["product_benefit"]["riders"][1]
        ["wop_rider_benefit"]
        .as_str()
        .map(String::from)
        .unwrap_or_default();
    let current_rider1_wop_premi = data_json["illustration"]["product_benefit"]["riders"][1]
        ["wop_rider_premi"]
        .as_f64()
        .unwrap_or(0.0);

    data_json["illustration"]["product_benefit"]["wop_basic_benefit"] = serde_json::json!("WOP03");
    let updated_data_str2 = data_json.to_string();

    let t = std::time::Instant::now();
    let deps_result2 = eval
        .evaluate_dependents(
            &["illustration.product_benefit.wop_basic_benefit".to_string()],
            Some(&updated_data_str2),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents 2 failed");
    println!(
        "[run 4] evaluate_dependents (wop_basic_benefit): {:?}",
        t.elapsed()
    );

    apply_changes(&mut data_json, deps_result2.clone());

    let new_basic_wop_benefit = data_json["illustration"]["product_benefit"]["wop_basic_benefit"]
        .as_str()
        .map(String::from)
        .unwrap_or_default();
    let new_basic_wop_premi = data_json["illustration"]["product_benefit"]["wop_basic_premi"]
        .as_f64()
        .unwrap_or(0.0);
    let new_rider1_wop_benefit = data_json["illustration"]["product_benefit"]["riders"][1]
        ["wop_rider_benefit"]
        .as_str()
        .map(String::from)
        .unwrap_or_default();
    let new_rider1_wop_premi = data_json["illustration"]["product_benefit"]["riders"][1]
        ["wop_rider_premi"]
        .as_f64()
        .unwrap_or(0.0);

    assert_ne!(
        current_basic_wop_benefit, new_basic_wop_benefit,
        "wop_basic_benefit should have changed"
    );
    assert_ne!(
        current_basic_wop_premi, new_basic_wop_premi,
        "wop_basic_premi should have changed"
    );
    assert_ne!(
        current_rider1_wop_benefit, new_rider1_wop_benefit,
        "wop_rider_benefit should have changed"
    );
    assert_ne!(
        current_rider1_wop_premi, new_rider1_wop_premi,
        "wop_rider_premi should have changed"
    );

    // =========================================================================
    // Run 5: full re-evaluate after wop_basic_benefit cascade (expect table cache hit)
    // =========================================================================
    let t = std::time::Instant::now();
    eval.evaluate(&data_json.to_string(), None, None, None)
        .expect("Full evaluation after wop_basic_benefit failed");
    println!(
        "[run 5] full evaluate after deps (wop_basic_benefit) (tables cache hit): {:?}",
        t.elapsed()
    );

    // =========================================================================
    // Run 5a: evaluate each rider subform individually after wop cascade
    // Expect: cache hits — wop tables already re-evaluated in run 4
    // =========================================================================
    {
        let riders = data_json["illustration"]["product_benefit"]["riders"]
            .as_array()
            .expect("riders should be an array")
            .clone();

        let t_total = std::time::Instant::now();
        for (idx, rider_item) in riders.iter().enumerate() {
            let subform_data = serde_json::json!({ "riders": rider_item });
            let subform_data_str = subform_data.to_string();
            let t = std::time::Instant::now();
            let subform_path = format!("{}.{}", RIDERS_SUBFORM_PATH, idx);
            eval.evaluate_subform(&subform_path, &subform_data_str, None, None, None)
                .unwrap_or_else(|e| panic!("evaluate_subform rider[{}] failed: {}", idx, e));
            println!("  [run 5a] rider[{}]: {:?}", idx, t.elapsed());
        }
        println!(
            "[run 5a] subform riders after wop cascade (expect cache hits): total={:?}",
            t_total.elapsed()
        );
    }
}

#[test]
fn test_zip_to_zpp_reload_scenario() {
    let zip_schema_str = load_file_content("samples/zip.json");
    let zpp_schema_str = load_file_content("samples/zpp.json");
    let zip_data_str = load_file_content("samples/zip-data.json");
    let zpp_data_str = load_file_content("samples/zpp-data.json");

    let mut eval = create_eval_helper(&zip_schema_str, None, Some(&zip_data_str))
        .expect("Failed to create JSONEval with ZIP");

    eval.evaluate(&zip_data_str, None, None, None).unwrap();

    // 1. Reload with ZPP
    eval.reload_schema(&zpp_schema_str, None, Some(&zpp_data_str))
        .unwrap();
    eval.evaluate(&zpp_data_str, None, None, None).unwrap();

    let get_prem_per_pay = |state: &Value| -> Option<f64> {
        let node = state.pointer("/$params/others/PREM_PER_PAY")?;
        if let Some(v) = node.get("value") {
            v.as_f64()
        } else {
            node.as_f64()
        }
    };

    let initial_eval = eval.get_evaluated_schema(false);
    let initial_prem = get_prem_per_pay(&initial_eval).unwrap_or(0.0);

    // 2. Modify TF
    let mut data_json: Value = serde_json::from_str(&zpp_data_str).unwrap();
    let current_tf = data_json["illustration"]["product_benefit"]["benefit_type"]["tf"]
        .as_f64()
        .unwrap_or(0.0);
    data_json["illustration"]["product_benefit"]["benefit_type"]["tf"] =
        serde_json::json!(current_tf + 100_000_000.0);
    let updated_data_str = data_json.to_string();

    eval.evaluate_dependents(
        &["illustration.product_benefit.benefit_type.tf".to_string()],
        Some(&updated_data_str),
        None,
        true,
        None,
        None,
        true,
    )
    .unwrap();

    let final_eval = eval.get_evaluated_schema(false);
    let final_prem = get_prem_per_pay(&final_eval).unwrap_or(0.0);
    println!("initial_prem: {}, final_prem: {}", initial_prem, final_prem);
    assert_ne!(
        initial_prem, final_prem,
        "PREM_PER_PAY should update after tf change on reloaded schema"
    );
}

#[test]
fn test_zpp_subform_new_rider_scenario() {
    let zpp_schema_str = load_file_content("samples/zpp.json");
    let mut zpp_data_json: Value =
        serde_json::from_str(&load_file_content("samples/zpp-data.json")).unwrap();

    // Use riders[1] as the template for a new ZLOB rider
    let mut new_rider = zpp_data_json["illustration"]["product_benefit"]["riders"][1].clone();

    // Step 1: Start with no riders — simulates fresh form
    if let Some(benefit) = zpp_data_json["illustration"]["product_benefit"].as_object_mut() {
        benefit.remove("riders");
    }
    let no_riders_data_str = zpp_data_json.to_string();

    let mut eval = create_eval_helper(&zpp_schema_str, None, Some(&no_riders_data_str))
        .expect("Failed to create JSONEval");
    eval.evaluate(&no_riders_data_str, None, None, None)
        .unwrap();

    // Step 2: Add the new ZLOB rider without sa, first_prem or wop_rider_premi.
    // Simulates the user placing an empty rider on the form.
    if let Some(rider_obj) = new_rider.as_object_mut() {
        rider_obj.insert("sa".to_string(), serde_json::json!(0.0));
        if let Some(lb) = rider_obj.get_mut("loading_benefit") {
            if let Some(lb_obj) = lb.as_object_mut() {
                lb_obj.remove("first_prem");
            }
        }
        rider_obj.remove("wop_rider_premi");
    }

    // Step 2: Build the subform payload for rider idx=0. In the subform context `riders`
    // is the root, so we set subform_data["riders"] = the single rider object.
    // The parent data `illustration.product_benefit.riders` array IS STILL EMPTY, simulating
    // frontend state before the user saves the array.
    let mut subform_data = zpp_data_json.clone();
    subform_data["riders"] = new_rider.clone();
    let subform_data_str = subform_data.to_string();

    let subform_path = format!("{}.0", RIDERS_SUBFORM_PATH);

    // Warm the per-item cache for rider 0 before triggering dependents
    eval.evaluate_subform(&subform_path, &subform_data_str, None, None, None)
        .expect("evaluate_subform failed");

    // Step 4: User sets sa = 200_000_000 → triggers cascade sa → first_prem → wop_rider_premi.
    subform_data["riders"]["sa"] = serde_json::json!(200000000.0);
    let subform_data_with_sa_str = subform_data.to_string();

    let deps_result = eval
        .evaluate_dependents_subform(
            &subform_path,
            &["riders.sa".to_string()],
            Some(&subform_data_with_sa_str),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents_subform failed");

    let deps_array = deps_result
        .as_array()
        .expect("deps_result should be an array");
    println!(
        "deps array from subform: {}",
        serde_json::to_string_pretty(deps_array).unwrap()
    );

    // first_prem must cascade from sa and be > 0
    let matched_first_prem = deps_array.iter().find(|item| {
        item.get("$ref").and_then(|r| r.as_str()) == Some("riders.loading_benefit.first_prem")
            && item.get("value").is_some()
    });
    assert!(
        matched_first_prem.is_some(),
        "first_prem should be in the dependents result"
    );
    let final_first_prem = matched_first_prem.unwrap()["value"].as_f64().unwrap_or(0.0);
    println!("final_first_prem: {}", final_first_prem);
    assert_ne!(
        final_first_prem, 0.0,
        "first_prem should be > 0 when sa = 200_000_000"
    );

    // wop_rider_premi must also cascade (via WOP_RIDERS table → rider's first_prem)
    let matched_wop = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("riders.wop_rider_premi"));
    println!("wop_rider_premi entry: {:?}", matched_wop);
    assert!(
        matched_wop.is_some(),
        "wop_rider_premi should be in the dependents result"
    );
    let final_wop = matched_wop.unwrap()["value"].as_f64().unwrap_or(0.0);
    println!("final_wop_rider_premi: {}", final_wop);
    assert_ne!(
        final_wop, 0.0,
        "wop_rider_premi should be > 0 after sa cascade"
    );
}

#[test]
fn test_zpp_prod_package_cascade_to_em_dur() {
    let zpp_schema_str = load_file_content("samples/zpp.json");
    let mut zpp_data_json: Value =
        serde_json::from_str(&load_file_content("samples/zpp-data.json")).unwrap();

    // 1. Clear prod_package, em, em_permill, em_dur, em_permilldur
    // Simulates a fresh form state before a package is chosen
    zpp_data_json["illustration"]["product_benefit"]["product"]["prod_package"] =
        serde_json::json!("");

    zpp_data_json["illustration"]["product_benefit"]["loading_benefit"]["em"] =
        serde_json::json!("");
    zpp_data_json["illustration"]["product_benefit"]["loading_benefit"]["em_permill"] =
        serde_json::json!("");
    zpp_data_json["illustration"]["product_benefit"]["loading_benefit"]["em_dur"] =
        serde_json::json!("");
    zpp_data_json["illustration"]["product_benefit"]["loading_benefit"]["em_permilldur"] =
        serde_json::json!("");

    let empty_initial_data_str = zpp_data_json.to_string();

    let mut eval = create_eval_helper(&zpp_schema_str, None, Some(&empty_initial_data_str))
        .expect("Failed to create JSONEval");

    eval.evaluate(&empty_initial_data_str, None, None, None)
        .unwrap();

    // 2. Trigger depends on field prod_package to "Classic 2"
    zpp_data_json["illustration"]["product_benefit"]["product"]["prod_package"] =
        serde_json::json!("Classic 2");
    let updated_data_str = zpp_data_json.to_string();

    let deps_result = eval
        .evaluate_dependents(
            &["illustration.product_benefit.product.prod_package".to_string()],
            Some(&updated_data_str),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    // 3. Assert change for em_dur and em_permilldur not empty string
    let deps_array = deps_result
        .as_array()
        .expect("deps_result should be an array");

    // Ensure em_dur is in deps_result and its value is not empty string
    let matched_em_dur = deps_array.iter().find(|item| {
        item.get("$ref").and_then(|r| r.as_str())
            == Some("illustration.product_benefit.loading_benefit.em_dur")
    });

    if matched_em_dur.is_none() {
        println!("deps payload: {:?}", deps_array);
    }

    assert!(
        matched_em_dur.is_some(),
        "em_dur should be updated by dependents cascade on Classic 2"
    );
    let val_em_dur = matched_em_dur.unwrap().get("value").unwrap();
    assert_ne!(
        val_em_dur,
        &serde_json::json!(""),
        "em_dur should not be empty string"
    );

    // Ensure em_permilldur is in deps_result and its value is not empty string
    let matched_em_permilldur = deps_array.iter().find(|item| {
        item.get("$ref").and_then(|r| r.as_str())
            == Some("illustration.product_benefit.loading_benefit.em_permilldur")
    });

    assert!(
        matched_em_permilldur.is_some(),
        "em_permilldur should be updated by dependents cascade on Classic 2"
    );
    let val_em_permilldur = matched_em_permilldur.unwrap().get("value").unwrap();
    assert_ne!(
        val_em_permilldur,
        &serde_json::json!(""),
        "em_permilldur should not be empty string"
    );
}

#[test]
fn test_zpp_riders() {
    let zpp_schema_str = load_file_content("samples/zpp.json");
    let zpp_data_json: Value =
        serde_json::from_str(&load_file_content("samples/zpp-data.json")).unwrap();

    let initial_data_str = zpp_data_json.to_string();

    let mut eval = create_eval_helper(&zpp_schema_str, None, Some(&initial_data_str))
        .expect("Failed to create JSONEval");

    let subform_path = format!("{}.0", RIDERS_SUBFORM_PATH);
    eval.evaluate_subform(&subform_path, &initial_data_str, None, None, None)
        .expect("evaluate_subform failed");

    let subform_path2 = format!("{}.1", RIDERS_SUBFORM_PATH);
    eval.evaluate_subform(&subform_path2, &initial_data_str, None, None, None)
        .expect("evaluate_subform failed");

    eval.evaluate_dependents(
        &["illustration.product_benefit.riders".to_string()],
        Some(&initial_data_str),
        None,
        true,
        None,
        None,
        true,
    )
    .expect("evaluate_dependents failed");

    let deps_result = eval
        .evaluate_dependents(
            &["illustration.product_benefit.riders".to_string()],
            Some(&initial_data_str),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    // 3. Assert change for em_dur and em_permilldur not empty string
    let deps_array = deps_result
        .as_array()
        .expect("deps_result should be an array");

    println!(
        "deps_array: {}",
        serde_json::to_string_pretty(&deps_array).unwrap()
    );

    // Assert all $ref in deps_array are unique (including all EM riders)
    let mut seen_refs = std::collections::HashSet::new();
    for item in deps_array {
        let r = item
            .get("$ref")
            .and_then(|v| v.as_str())
            .expect("item should have $ref");
        assert!(
            seen_refs.insert(r.to_string()),
            "Duplicate $ref found in deps_array: {}",
            r
        );
    }

    let get_rider_em = |idx: usize| {
        let path = format!(
            "illustration.product_benefit.riders.{}.loading_benefit.em_dur",
            idx
        );
        deps_array
            .iter()
            .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some(&path))
            .expect(&format!("EM for rider {} should be in deps", idx))
            .get("value")
            .expect("value missing")
    };

    let val0 = get_rider_em(0);
    let val1 = get_rider_em(1);
    let val2 = get_rider_em(2);

    println!("val0: {}, val1: {}, val2: {}", val0, val1, val2);

    // Only for rider 0 and 1 should be unique (different values), 1 and 2 should be same
    assert_ne!(
        val0, val1,
        "Rider 0 and 1 EM Dur should be unique (different values)"
    );
    assert_eq!(val1, val2, "Rider 1 and 2 EM Dur should be same");

    // Also ensure em_permilldur is in deps_result and its value is not empty string (from previous code)
    let matched_em_permilldur = deps_array.iter().find(|item| {
        item.get("$ref").and_then(|r| r.as_str())
            == Some("illustration.product_benefit.loading_benefit.em_permilldur")
    });

    assert!(
        matched_em_permilldur.is_some(),
        "em_permilldur should be in deps"
    );
    let val_em_permilldur = matched_em_permilldur.unwrap().get("value").unwrap();
    assert_ne!(
        val_em_permilldur,
        &serde_json::json!(""),
        "em_permilldur should not be empty string"
    );
}

// =============================================================================
// Test: wop_flag = false cascades to clear ZLOB rider wop fields
//
// Exercises:
//   - Null value from schema formula emits clear:true (not silently dropped)
//   - Main-form dependent results feed back into subform item dependent cascade
// =============================================================================
#[test]
fn test_zpp_wop_flag_false_clears_rider_wop_fields() {
    let zpp_schema_str = load_file_content("samples/zpp.json");
    let mut zpp_data_json: Value =
        serde_json::from_str(&load_file_content("samples/zpp-data.json")).unwrap();

    assert_eq!(
        zpp_data_json["illustration"]["product_benefit"]["wop_flag"],
        serde_json::json!(true),
        "fixture must start with wop_flag=true"
    );

    let rider1_wop_premi_before =
        zpp_data_json["illustration"]["product_benefit"]["riders"][1]["wop_rider_premi"].clone();

    let initial_data_str = zpp_data_json.to_string();
    let mut eval = create_eval_helper(&zpp_schema_str, None, Some(&initial_data_str))
        .expect("Failed to create JSONEval");

    eval.evaluate(&initial_data_str, None, None, None)
        .expect("Initial evaluation failed");

    // Warm per-item caches for all riders
    for idx in 0..3usize {
        let subform_data = serde_json::json!({
            "riders": zpp_data_json["illustration"]["product_benefit"]["riders"][idx]
        });
        let sp = format!("{}.{}", RIDERS_SUBFORM_PATH, idx);
        eval.evaluate_subform(&sp, &subform_data.to_string(), None, None, None)
            .unwrap_or_else(|e| panic!("evaluate_subform rider[{}] failed: {}", idx, e));
    }

    // Set main wop_flag = false
    zpp_data_json["illustration"]["product_benefit"]["wop_flag"] = serde_json::json!(false);
    let updated_data_str = zpp_data_json.to_string();

    let deps_result = eval
        .evaluate_dependents(
            &["illustration.product_benefit.wop_flag".to_string()],
            Some(&updated_data_str),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    let deps_array = deps_result.as_array().expect("deps_result must be array");

    println!(
        "wop_flag=false deps ({} items):\n{}",
        deps_array.len(),
        serde_json::to_string_pretty(deps_array).unwrap()
    );

    // wop_flag, wop_rider_benefit, wop_rider_premi must be cleared for rider 1.
    // They may arrive as:
    //   - `{ clear: true }` from the dependents queue, OR
    //   - `{ $readonly: true, value: null }` from the re-evaluate pass
    // Both indicate the field was nulled out. Accept either form.
    for expected_ref in &[
        "illustration.product_benefit.riders.1.wop_flag",
        "illustration.product_benefit.riders.1.wop_rider_benefit",
        "illustration.product_benefit.riders.1.wop_rider_premi",
    ] {
        let matched = deps_array.iter().find(|item| {
            let is_this_ref = item.get("$ref").and_then(|r| r.as_str()) == Some(expected_ref);
            let is_clear = item.get("clear").and_then(Value::as_bool) == Some(true);
            let is_null_value = item.get("value") == Some(&Value::Null);
            is_this_ref && (is_clear || is_null_value)
        });

        if let Some(entry) = &matched {
            println!(
                "  [ok] {} -> {}",
                expected_ref,
                serde_json::to_string(entry).unwrap()
            );
        }

        assert!(
            matched.is_some(),
            "Expected clear:true or value:null for '{}' in deps_result. Rider-1 refs: {:#?}",
            expected_ref,
            deps_array
                .iter()
                .filter(|i| i
                    .get("$ref")
                    .and_then(|r| r.as_str())
                    .map(|r| r.contains("riders.1"))
                    .unwrap_or(false))
                .collect::<Vec<_>>()
        );
    }

    let data = eval.eval_data.data();
    let wop_premi_after = data.pointer("/illustration/product_benefit/riders/1/wop_rider_premi");
    assert!(
        wop_premi_after.is_none() || wop_premi_after == Some(&Value::Null),
        "riders[1].wop_rider_premi={:?} before; must be null after wop_flag=false",
        rider1_wop_premi_before
    );
}

// =============================================================================
// Test: validate_subform — required satisfied but minValue still triggered
//
// The `sa` field on a ZLOB rider has required:true AND minValue:10_000_000.
// When sa has a non-null value that is below the minimum, required passes
// but minValue must still be reported.
// =============================================================================
#[test]
fn test_zpp_rider_sa_minvalue_validation() {
    let zpp_schema_str = load_file_content("samples/zpp.json");
    let zpp_data_json: Value =
        serde_json::from_str(&load_file_content("samples/zpp-data.json")).unwrap();

    let initial_data_str = zpp_data_json.to_string();
    let mut eval = create_eval_helper(&zpp_schema_str, None, Some(&initial_data_str))
        .expect("Failed to create JSONEval");

    eval.evaluate(&initial_data_str, None, None, None)
        .expect("Initial evaluation failed");

    // Warm per-item cache for rider 0
    let sp = format!("{}.0", RIDERS_SUBFORM_PATH);
    let rider0 = zpp_data_json["illustration"]["product_benefit"]["riders"][0].clone();
    let warm_data = serde_json::json!({ "riders": rider0 });
    eval.evaluate_subform(&sp, &warm_data.to_string(), None, None, None)
        .expect("evaluate_subform rider[0] failed");

    // --- Case 1: sa = null → required must fire ---
    let mut rider_null = zpp_data_json["illustration"]["product_benefit"]["riders"][0].clone();
    rider_null["sa"] = Value::Null;
    let null_data = serde_json::json!({ "riders": rider_null });
    let result_null = eval
        .validate_subform(&sp, &null_data.to_string(), None, None, None)
        .expect("validate_subform null sa failed");

    println!("null sa errors: {:?}", result_null.errors);
    assert!(
        result_null.has_error,
        "null sa must produce a validation error"
    );
    let err_null = result_null.errors.get("riders.sa");
    assert!(err_null.is_some(), "error must be keyed to 'riders.sa'");
    assert_eq!(
        err_null.unwrap().rule_type,
        "required",
        "null sa → required"
    );

    // --- Case 2: sa = 5_000_000 (non-null, < 10_000_000) → minValue must fire ---
    let mut rider_low = zpp_data_json["illustration"]["product_benefit"]["riders"][0].clone();
    rider_low["sa"] = serde_json::json!(5_000_000);
    let low_data = serde_json::json!({ "riders": rider_low });
    let result_low = eval
        .validate_subform(&sp, &low_data.to_string(), None, None, None)
        .expect("validate_subform low sa failed");

    println!("low sa (5_000_000) errors: {:?}", result_low.errors);
    assert!(
        result_low.has_error,
        "sa=5_000_000 (below min 10_000_000) must produce a validation error"
    );
    let err_low = result_low.errors.get("riders.sa");
    assert!(
        err_low.is_some(),
        "minValue error must be keyed to 'riders.sa'. All errors: {:?}",
        result_low.errors
    );
    assert_eq!(
        err_low.unwrap().rule_type,
        "minValue",
        "sa=5_000_000 must report minValue (required was satisfied)"
    );

    // --- Case 3: sa = 200_000_000 (valid) → no sa error ---
    let mut rider_valid = zpp_data_json["illustration"]["product_benefit"]["riders"][0].clone();
    rider_valid["sa"] = serde_json::json!(200_000_000);
    let valid_data = serde_json::json!({ "riders": rider_valid });
    let result_valid = eval
        .validate_subform(&sp, &valid_data.to_string(), None, None, None)
        .expect("validate_subform valid sa failed");

    println!("valid sa errors: {:?}", result_valid.errors);
    assert!(
        result_valid.errors.get("riders.sa").is_none(),
        "sa=200_000_000 must not produce a riders.sa error"
    );

    // --- Case 4: sa = "5000000" (string, non-empty, < min) → minValue must fire ---
    // This is the actual frontend bug: JS sends numbers as strings; required passes
    // (non-empty string) but minValue was bypassed because as_f64() returned None.
    let mut rider_str_low = zpp_data_json["illustration"]["product_benefit"]["riders"][0].clone();
    rider_str_low["sa"] = serde_json::json!("5000000");
    let str_low_data = serde_json::json!({ "riders": rider_str_low });
    let result_str_low = eval
        .validate_subform(&sp, &str_low_data.to_string(), None, None, None)
        .expect("validate_subform string sa failed");

    println!(
        "string sa (\"5000000\") errors: {:?}",
        result_str_low.errors
    );
    assert!(
        result_str_low.has_error,
        "sa=\"5000000\" (string below minimum) must produce a validation error"
    );
    let err_str_low = result_str_low.errors.get("riders.sa");
    assert!(
        err_str_low.is_some(),
        "minValue error must be keyed to 'riders.sa' for string input. All errors: {:?}",
        result_str_low.errors
    );
    assert_eq!(
        err_str_low.unwrap().rule_type,
        "minValue",
        "sa=\"5000000\" must report minValue (required was satisfied by non-empty string)"
    );

    // --- Case 5: sa = "200000000" (valid string) → no sa error ---
    let mut rider_str_valid = zpp_data_json["illustration"]["product_benefit"]["riders"][0].clone();
    rider_str_valid["sa"] = serde_json::json!("200000000");
    let str_valid_data = serde_json::json!({ "riders": rider_str_valid });
    let result_str_valid = eval
        .validate_subform(&sp, &str_valid_data.to_string(), None, None, None)
        .expect("validate_subform string valid sa failed");

    println!("string valid sa errors: {:?}", result_str_valid.errors);
    assert!(
        result_str_valid.errors.get("riders.sa").is_none(),
        "sa=\"200000000\" must not produce a riders.sa error"
    );
}

#[test]
fn test_zpp_rider_prem_pay_period_hidden() {
    let zpp_schema_str = load_file_content("samples/zpp.json");
    let mut zpp_data_json: Value =
        serde_json::from_str(&load_file_content("samples/zpp-data.json")).unwrap();

    // 1. load zpp + data with empty rider
    zpp_data_json["illustration"]["product_benefit"]["riders"] = serde_json::json!([]);
    let initial_data_str = zpp_data_json.to_string();

    let mut eval = create_eval_helper(&zpp_schema_str, None, Some(&initial_data_str))
        .expect("Failed to create JSONEval");

    // 2. run evaluate depends, evaluate, evaluate depends subform
    eval.evaluate_dependents(
        &["illustration.product_benefit.riders".to_string()],
        Some(&initial_data_str),
        None,
        true,
        None,
        None,
        true,
    )
    .expect("evaluate_dependents failed");

    eval.evaluate(&initial_data_str, None, None, None)
        .expect("evaluate failed");

    let rider_data = serde_json::json!({
        "name": "Zurich Life Optima - Smart",
    });

    let mut subform_data = zpp_data_json.clone();
    subform_data["riders"] = serde_json::json!({});
    let subform_data_str = subform_data.to_string();

    let subform_path = format!("{}.0", RIDERS_SUBFORM_PATH);

    // Warm up the subform cache first as usual
    eval.evaluate_subform(&subform_path, &subform_data_str, None, None, None)
        .expect("evaluate_subform failed");

    eval.validate_subform(&subform_path, &subform_data_str, None, None, None)
        .expect("validate_subform failed");

    subform_data["riders"] = rider_data;
    let subform_data_str = subform_data.to_string();

    eval.evaluate_dependents_subform(
        &subform_path,
        &["riders.name".to_string()],
        Some(&subform_data_str),
        None,
        true,
        None,
        None,
        true,
    )
    .expect("evaluate_dependents_subform failed");

    eval.evaluate_subform(&subform_path, &subform_data_str, None, None, None)
        .expect("evaluate_subform failed");

    eval.validate_subform(&subform_path, &subform_data_str, None, None, None)
        .expect("validate_subform failed");

    // 3. assert riders.properties.prem_pay_period.condition.hidden = false on evaluated subform schema
    let evaluated_subform_schema = eval.get_evaluated_schema_subform(&subform_path, false);

    let hidden_val = evaluated_subform_schema
        .pointer("/riders/properties/prem_pay_period/condition/hidden")
        .and_then(|v| v.as_bool());

    assert_eq!(
        hidden_val,
        Some(false),
        "rider prem_pay_period should be hidden=false"
    );
}

#[test]
fn test_zpp_occupation_class_cascade() {
    let zpp_schema_str = load_file_content("samples/zpp.json");

    // 1. load zpp with empty data
    let mut data_json: Value = serde_json::json!({
        "illustration": {
            "insured": {},
            "policyholder": {
                "jobinfo": {}
            }
        }
    });

    let mut eval = create_eval_helper(&zpp_schema_str, None, Some(&data_json.to_string()))
        .expect("Failed to create JSONEval");

    // 2. fill insured.ins_corrname, ins_dob, insage, ins_gender
    data_json["illustration"]["insured"]["ins_corrname"] = serde_json::json!("Test Insured");
    data_json["illustration"]["insured"]["ins_dob"] = serde_json::json!("1990-01-01");
    data_json["illustration"]["insured"]["insage"] = serde_json::json!(35);
    data_json["illustration"]["insured"]["ins_gender"] = serde_json::json!("L");

    // 3. trigger evaluate
    eval.evaluate(&data_json.to_string(), None, None, None)
        .expect("evaluate failed");

    // 4. fill ins_occ, trigger dependents -> assert has ins_occclass change
    data_json["illustration"]["insured"]["ins_occ"] = serde_json::json!("PAWN");
    let deps_result = eval
        .evaluate_dependents(
            &["illustration.insured.ins_occ".to_string()],
            Some(&data_json.to_string()),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    let deps_array = deps_result.as_array().expect("deps should be array");

    let matched_occclass = deps_array.iter().find(|item| {
        item.get("$ref").and_then(|r| r.as_str()) == Some("illustration.insured.ins_occclass")
    });

    assert!(
        matched_occclass.is_some(),
        "ins_occclass should have changed"
    );
    let ins_occclass_val = matched_occclass.unwrap().get("value").unwrap();
    assert_ne!(
        ins_occclass_val,
        &serde_json::json!(""),
        "ins_occclass should not be empty"
    );

    // Apply dependent changes back to data_json to simulate frontend behavior
    for change in deps_array {
        if let Some(r) = change.get("$ref").and_then(|v| v.as_str()) {
            if let Some(val) = change.get("value") {
                if r == "illustration.insured.ins_occclass" {
                    data_json["illustration"]["insured"]["ins_occclass"] = val.clone();
                }
            }
        }
    }

    // 5. fill phins_relation = 1, trigger dependents -> assert has policyholder.jobinfo.ph_occupation and ph_occclass change
    data_json["illustration"]["insured"]["phins_relation"] = serde_json::json!("1");
    let deps_result_2 = eval
        .evaluate_dependents(
            &["illustration.insured.phins_relation".to_string()],
            Some(&data_json.to_string()),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    let deps_array_2 = deps_result_2.as_array().expect("deps should be array");
    println!(
        "deps_array_2: {}",
        serde_json::to_string_pretty(deps_array_2).unwrap()
    );

    let matched_ph_occ = deps_array_2.iter().find(|item| {
        item.get("$ref").and_then(|r| r.as_str())
            == Some("illustration.policyholder.jobinfo.ph_occupation")
    });
    assert!(
        matched_ph_occ.is_some(),
        "ph_occupation should have changed"
    );
    let ph_occ_val = matched_ph_occ.unwrap().get("value").unwrap();
    assert_eq!(
        ph_occ_val,
        &serde_json::json!("PAWN"),
        "ph_occupation should be copied from ins_occ"
    );

    let matched_ph_occclass = deps_array_2.iter().find(|item| {
        item.get("$ref").and_then(|r| r.as_str())
            == Some("illustration.policyholder.jobinfo.ph_occclass")
    });
    assert!(
        matched_ph_occclass.is_some(),
        "ph_occclass should have changed"
    );
    let ph_occclass_val = matched_ph_occclass.unwrap().get("value").unwrap();
    assert_eq!(
        ph_occclass_val, ins_occclass_val,
        "ph_occclass should be copied from ins_occclass"
    );
}

/// Regression test: triggering `ins_gender` must not produce `ins_gender` back as a
/// transitive dependent result.
///
/// Background: `phins_relation` has a dependent that writes to `ins_gender`.
/// Our `dep_formula_triggers` will re-enqueue `phins_relation` when `ins_gender` changes
/// (because `phins_relation`'s formula for `ins_gender` references `ins_gender`).
/// Without the "already processed" guard, `phins_relation` would then write back to
/// `ins_gender` and emit it as a transitive change — which is incorrect.
#[test]
fn test_ins_gender_no_self_cycle_in_dependents() {
    let zpp_schema_str = load_file_content("samples/zpp.json");

    let data_json: Value = serde_json::json!({
        "illustration": {
            "insured": {
                "ins_gender": "L",
                "phins_relation": "1"
            },
            "policyholder": {
                "ph_gender": "L",
                "jobinfo": {}
            }
        }
    });

    let mut eval = create_eval_helper(&zpp_schema_str, None, Some(&data_json.to_string()))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data_json.to_string(), None, None, None)
        .expect("evaluate failed");

    // Trigger ins_gender change
    let mut changed_data = data_json.clone();
    changed_data["illustration"]["insured"]["ins_gender"] = serde_json::json!("P");

    let deps_result = eval
        .evaluate_dependents(
            &["illustration.insured.ins_gender".to_string()],
            Some(&changed_data.to_string()),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    let deps_array = deps_result.as_array().expect("deps should be array");

    // ins_gender must NOT appear in the output — it was the trigger, not a dependent result
    let self_reference = deps_array.iter().find(|item| {
        item.get("$ref").and_then(|r| r.as_str()) == Some("illustration.insured.ins_gender")
    });

    assert!(
        self_reference.is_none(),
        "ins_gender must not appear as a transitive dependent of itself, got: {:?}",
        deps_array
            .iter()
            .map(|i| i.get("$ref").and_then(|r| r.as_str()).unwrap_or("?"))
            .collect::<Vec<_>>()
    );
}
