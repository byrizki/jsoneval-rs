use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: String,
    pub schema_path: PathBuf,
    pub data_path: PathBuf,
    pub comparison_path: Option<PathBuf>,
    pub is_msgpack: bool,
}

pub fn discover_scenarios(dir: &Path) -> Vec<Scenario> {
    let mut scenarios = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("⚠️  Unable to read scenario directory `{}`: {}", dir.display(), err);
            return scenarios;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };

        if let Some(base) = file_name.strip_suffix("-data.json") {
            // Check for MessagePack schema first (.bform), then JSON (.json)
            let (schema_path, is_msgpack) = {
                let msgpack_path = dir.join(format!("{}.bform", base));
                let json_path = dir.join(format!("{}.json", base));
                
                if msgpack_path.exists() {
                    (msgpack_path, true)
                } else if json_path.exists() {
                    (json_path, false)
                } else {
                    eprintln!(
                        "⚠️  Skipping scenario `{}`: schema file `{}.json` or `{}.bform` missing",
                        base, base, base
                    );
                    continue;
                }
            };

            let comparison_path = {
                let candidate = dir.join(format!("{}-evaluated-compare.json", base));
                if candidate.exists() {
                    Some(candidate)
                } else {
                    None
                }
            };

            scenarios.push(Scenario {
                name: base.to_string(),
                schema_path,
                data_path: path.clone(),
                comparison_path,
                is_msgpack,
            });
        }
    }

    scenarios.sort_by(|a, b| a.name.cmp(&b.name));
    scenarios
}

pub fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "<failed to serialize>".to_string())
}

fn summarize_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s),
        Value::Number(num) => {
            if let Some(int_val) = num.as_i64() {
                int_val.to_string()
            } else if let Some(u_val) = num.as_u64() {
                u_val.to_string()
            } else if let Some(f_val) = num.as_f64() {
                if (f_val.fract()).abs() <= 1e-9 {
                    format!("{}", f_val.trunc() as i64)
                } else {
                    let mut s = format!("{:.6}", f_val);
                    while s.contains('.') && s.ends_with('0') {
                        s.pop();
                    }
                    if s.ends_with('.') {
                        s.pop();
                    }
                    s
                }
            } else {
                num.to_string()
            }
        }
        _ => value.to_string(),
    }
}

fn numbers_equal(a: &Value, b: &Value) -> bool {
    const EPSILON: f64 = 1e-6;
    match (a, b) {
        (Value::Number(num_a), Value::Number(num_b)) => {
            match (num_a.as_f64(), num_b.as_f64()) {
                (Some(a_f64), Some(b_f64)) => (a_f64 - b_f64).abs() <= EPSILON,
                _ => false,
            }
        }
        _ => false,
    }
}

fn collect_value_diffs(actual: &Value, expected: &Value, path: &str, diffs: &mut Vec<String>) {
    if actual == expected || numbers_equal(actual, expected) {
        return;
    }

    match (actual, expected) {
        (Value::Object(map_a), Value::Object(map_b)) => {
            let mut keys = BTreeSet::new();
            keys.extend(map_a.keys().cloned());
            keys.extend(map_b.keys().cloned());

            for key in keys {
                let next_path = if path.is_empty() {
                    format!("/{}", key)
                } else {
                    format!("{}/{}", path, key)
                };

                match (map_a.get(&key), map_b.get(&key)) {
                    (Some(value_a), Some(value_b)) => {
                        collect_value_diffs(value_a, value_b, &next_path, diffs);
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
                let next_path = if path.is_empty() {
                    format!("/{}", index)
                } else {
                    format!("{}/{}", path, index)
                };

                match (arr_a.get(index), arr_b.get(index)) {
                    (Some(value_a), Some(value_b)) => {
                        collect_value_diffs(value_a, value_b, &next_path, diffs);
                    }
                    (Some(_), None) => diffs.push(format!("{} present in actual but missing in expected", next_path)),
                    (None, Some(_)) => diffs.push(format!("{} missing in actual but present in expected", next_path)),
                    (None, None) => {}
                }
            }
        }
        _ => {
            if numbers_equal(actual, expected) {
                return;
            }
            diffs.push(format!(
                "{} differs: actual={} expected={}",
                if path.is_empty() { "/".to_string() } else { path.to_string() },
                summarize_value(actual),
                summarize_value(expected)
            ));
        }
    }
}

pub fn compare_with_expected(
    evaluated_schema: &Value,
    comparison_path: &Path,
) -> Result<(), String> {
    let comp_str = fs::read_to_string(comparison_path)
        .map_err(|e| format!("Failed to read comparison file: {}", e))?;
    
    let expected_value: Value = serde_json::from_str(&comp_str)
        .map_err(|e| format!("Failed to parse comparison file: {}", e))?;
    
    let actual_others = evaluated_schema.pointer("/$params/others");
    let expected_others = expected_value.get("others");
    
    match (actual_others, expected_others) {
        (Some(actual), Some(expected)) => {
            if actual == expected {
                println!(
                    "🔍 Comparison: `$.others` matches `{}`",
                    comparison_path.display()
                );
                Ok(())
            } else {
                let mut diffs = Vec::new();
                collect_value_diffs(actual, expected, "$.others", &mut diffs);
                println!(
                    "⚠️  Comparison: `$.others` differs from `{}` ({} differences):",
                    comparison_path.display(),
                    diffs.len()
                );
                for diff in &diffs {
                    println!("  - {}", diff);
                }
                Err(format!("{} differences found", diffs.len()))
            }
        }
        (Some(_), None) => {
            println!(
                "⚠️  Comparison: `{}` missing `$.others` section",
                comparison_path.display()
            );
            Err("Expected file missing $.others section".to_string())
        }
        (None, Some(_)) => {
            println!(
                "⚠️  Comparison: evaluated schema missing `$.others` section"
            );
            Err("Evaluated schema missing $.others section".to_string())
        }
        (None, None) => {
            println!(
                "ℹ️  Comparison: both evaluated and `{}` lack `$.others`",
                comparison_path.display()
            );
            Ok(())
        }
    }
}

#[allow(dead_code)]
pub fn print_cpu_info() {
    #[cfg(target_arch = "x86_64")]
    {
        println!("🖥️  CPU Features:");
        println!("  SSE2:    {}", is_x86_feature_detected!("sse2"));
        println!("  SSE4.2:  {}", is_x86_feature_detected!("sse4.2"));
        println!("  AVX:     {}", is_x86_feature_detected!("avx"));
        println!("  AVX2:    {}", is_x86_feature_detected!("avx2"));
        
        #[cfg(windows)]
        println!("  Allocator: mimalloc");
        
        #[cfg(not(windows))]
        println!("  Allocator: system default");
        
        println!();
    }
}
