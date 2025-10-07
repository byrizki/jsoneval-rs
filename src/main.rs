use std::collections::BTreeSet;
use std::fs;
use std::time::Instant;

use json_eval_rs::JSONEval;
use serde_json::Value;

const MAX_DIFFS_TO_SHOW: usize = 5;

fn collect_differences(actual: &Value, expected: &Value, path: &str, diffs: &mut Vec<String>) {
    if diffs.len() >= MAX_DIFFS_TO_SHOW || actual == expected {
        return;
    }

    match (actual, expected) {
        (Value::Object(map_a), Value::Object(map_b)) => {
            let mut keys = BTreeSet::new();
            keys.extend(map_a.keys().cloned());
            keys.extend(map_b.keys().cloned());

            for key in keys {
                if diffs.len() >= MAX_DIFFS_TO_SHOW {
                    break;
                }

                let next_path = if path.is_empty() {
                    format!("/{}", key)
                } else {
                    format!("{}/{}", path, key)
                };

                match (map_a.get(&key), map_b.get(&key)) {
                    (Some(value_a), Some(value_b)) => {
                        collect_differences(value_a, value_b, &next_path, diffs);
                    }
                    (Some(_), None) => diffs.push(format!("{} present in actual but missing in expected", next_path)),
                    (None, Some(_)) => diffs.push(format!("{} missing in actual but present in expected", next_path)),
                    (None, None) => {}
                }
            }
        }
        (Value::Array(arr_a), Value::Array(arr_b)) => {
            let max_len = arr_a.len().max(arr_b.len());

            for index in 0..max_len {
                if diffs.len() >= MAX_DIFFS_TO_SHOW {
                    break;
                }

                let next_path = if path.is_empty() {
                    format!("/{}", index)
                } else {
                    format!("{}/{}", path, index)
                };

                match (arr_a.get(index), arr_b.get(index)) {
                    (Some(value_a), Some(value_b)) => {
                        collect_differences(value_a, value_b, &next_path, diffs);
                    }
                    (Some(_), None) => diffs.push(format!("{} present in actual but missing in expected", next_path)),
                    (None, Some(_)) => diffs.push(format!("{} missing in actual but present in expected", next_path)),
                    (None, None) => {}
                }
            }
        }
        _ => {
            diffs.push(format!(
                "{} differs: actual={} expected={}",
                if path.is_empty() { "/" } else { path },
                summarize_value(actual),
                summarize_value(expected)
            ));
        }
    }
}

fn summarize_value(value: &Value) -> String {
    let serialized = serde_json::to_string(value).unwrap_or_else(|_| "<unserializable>".to_string());
    if serialized.len() > 80 {
        format!("{}...", &serialized[..77])
    } else {
        serialized
    }
}

fn main() {
    println!("\n🚀 JSON Evaluation Benchmark\n");
    
    let schema_path = "example/zip.json";
    let data_path = "example/zip-data.json";
    
    println!("Loading files...");
    let schema_str = fs::read_to_string(schema_path).expect("read schema");
    let data_str = fs::read_to_string(data_path).expect("read data");
    
    // Single timed run with breakdown
    println!("Running evaluation...\n");
    
    let t0 = Instant::now();
    let mut eval = JSONEval::new(&schema_str, None, Some(&data_str)).expect("create JSONEval");
    let t1 = t0.elapsed();
    println!("  Schema parsing & compilation: {:?}", t1);
    
    let t2 = Instant::now();
    let evaluated_schema = eval.evaluate(&data_str, Some("{}")).expect("evaluate");
    let t3 = t2.elapsed();
    println!("  Evaluation: {:?}", t3);
    
    let duration = t0.elapsed();
    
    println!("⏱️  Execution time: {:?}\n", duration);
    
    // Save results
    let out_path = "example/evaluated-schema.json";
    fs::write(
        out_path,
        serde_json::to_string_pretty(&evaluated_schema).expect("serialize"),
    )
    .expect("write output");

    let out_path = "example/sorted_evaluations.json";
    fs::write(
        out_path,
        serde_json::to_string_pretty(&eval.sorted_evaluations).expect("serialize"),
    )
    .expect("write output");
    
    println!("✅ Results saved to {}", out_path);

    let comparison_path = "example/evaluated-schema-comp.json";
    match fs::read_to_string(comparison_path) {
        Ok(comp_str) => match serde_json::from_str::<Value>(&comp_str) {
            Ok(expected_value) => {
                if evaluated_schema == expected_value {
                    println!("🔍 Comparison: evaluated schema matches `{}`", comparison_path);
                } else {
                    let mut diffs = Vec::new();
                    collect_differences(&evaluated_schema, &expected_value, "", &mut diffs);

                    println!("⚠️  Comparison: differences found vs `{}` (showing up to {}):", comparison_path, MAX_DIFFS_TO_SHOW);
                    for diff in diffs {
                        println!("  - {}", diff);
                    }
                }
            }
            Err(err) => println!("⚠️  Failed to parse `{}`: {}", comparison_path, err),
        },
        Err(err) => println!("ℹ️  Comparison file `{}` not available: {}", comparison_path, err),
    }

    println!("\n📊 Cache Stats: {:?}\n", eval.engine.cache_stats());
}
