use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use json_eval_rs::{json_parser, JSONEval};
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
struct Scenario {
    name: String,
    schema_path: PathBuf,
    data_path: PathBuf,
    comparison_path: Option<PathBuf>,
}

fn pretty_json(value: &Value) -> String {
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

fn discover_scenarios(dir: &Path) -> Vec<Scenario> {
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
            let schema_path = dir.join(format!("{}.json", base));
            if !schema_path.exists() {
                eprintln!(
                    "⚠️  Skipping scenario `{}`: schema file `{}` missing",
                    base,
                    schema_path.display()
                );
                continue;
            }

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
            });
        }
    }

    scenarios.sort_by(|a, b| a.name.cmp(&b.name));
    scenarios
}

fn print_help(program_name: &str) {
    println!("\n🚀 JSON Evaluation Benchmark\n");
    println!("USAGE:");
    println!("    {} [OPTIONS] [FILTER]\n", program_name);
    println!("OPTIONS:");
    println!("    -h, --help                 Show this help message");
    println!("    -v, --version              Show version information");
    println!("    -i, --iterations <COUNT>   Number of evaluation iterations (default: 1)\n");
    println!("ARGUMENTS:");
    println!("    [FILTER]                   Optional filter to match scenario names\n");
    println!("DESCRIPTION:");
    println!("    Discovers and runs JSON evaluation scenarios from the 'example/' directory.");
    println!("    Each scenario consists of:");
    println!("      - <name>.json           (schema file)");
    println!("      - <name>-data.json      (input data file)");
    println!("      - <name>-evaluated-compare.json (optional comparison file)\n");
    println!("EXAMPLES:");
    println!("    {}                      # Run all scenarios", program_name);
    println!("    {} zcc                  # Run scenarios matching 'zcc'", program_name);
    println!("    {} -i 100 zlw           # Run 'zlw' scenario 100 times (cache benchmark)", program_name);
    println!("    {} --help               # Show this help", program_name);
}

fn print_cpu_info() {
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

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let program_name = args.get(0).map(|s| s.as_str()).unwrap_or("json-eval-cli");
    
    let mut iterations = 1usize;
    let mut scenario_filter: Option<String> = None;
    let mut show_cpu_info = false;
    let mut i = 1;
    
    // Parse arguments
    while i < args.len() {
        let arg = &args[i];
        
        if arg == "-h" || arg == "--help" {
            print_help(program_name);
            return;
        } else if arg == "-v" || arg == "--version" {
            println!("json-eval-rs v{}", env!("CARGO_PKG_VERSION"));
            return;
        } else if arg == "--cpu-info" {
            show_cpu_info = true;
        } else if arg == "-i" || arg == "--iterations" {
            if i + 1 >= args.len() {
                eprintln!("Error: {} requires a value", arg);
                print_help(program_name);
                return;
            }
            i += 1;
            match args[i].parse::<usize>() {
                Ok(n) if n > 0 => iterations = n,
                _ => {
                    eprintln!("Error: iterations must be a positive integer, got '{}'", args[i]);
                    return;
                }
            }
        } else if !arg.starts_with('-') {
            scenario_filter = Some(arg.clone());
        } else {
            eprintln!("Error: unknown option '{}'", arg);
            print_help(program_name);
            return;
        }
        
        i += 1;
    }
    
    println!("\n🚀 JSON Evaluation Benchmark\n");
    
    // Show CPU info if requested or if running benchmarks
    if show_cpu_info || iterations > 1 {
        print_cpu_info();
    }
    
    if iterations > 1 {
        println!("🔄 Iterations per scenario: {}\n", iterations);
    }

    let samples_dir = Path::new("samples");
    let mut scenarios = discover_scenarios(samples_dir);
    
    // Filter scenarios if a filter is provided
    if let Some(ref filter) = scenario_filter {
        scenarios.retain(|s| s.name.contains(filter));
        println!("📋 Filtering scenarios matching: '{}'\n", filter);
    }

    if scenarios.is_empty() {
        if let Some(filter) = scenario_filter {
            println!(
                "ℹ️  No scenarios found matching '{}' in `{}`.",
                filter,
                samples_dir.display()
            );
        } else {
            println!(
                "ℹ️  No scenarios discovered in `{}`. Add files like `name.json` and `name-data.json`.",
                samples_dir.display()
            );
            println!("\nRun with --help for usage information.");
        }
        return;
    }
    
    println!("📊 Found {} scenario(s)\n", scenarios.len());

    let mut total_parse_time = std::time::Duration::ZERO;
    let mut total_eval_time = std::time::Duration::ZERO;
    let mut successful_scenarios = 0;

    for scenario in &scenarios {
        println!("==============================");
        println!("Scenario: {}", scenario.name);
        println!("Schema: {}", scenario.schema_path.display());
        println!("Data: {}\n", scenario.data_path.display());

        println!("Loading files...");
        // Use SIMD-accelerated file reading for maximum performance
        let schema_str = fs::read_to_string(&scenario.schema_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", scenario.schema_path.display(), e));
        let data_str = fs::read_to_string(&scenario.data_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", scenario.data_path.display(), e));

        println!("Running evaluation...\n");

        let start_time = Instant::now();
        let mut eval = JSONEval::new(&schema_str, None, Some(&data_str))
            .unwrap_or_else(|e| panic!("failed to create JSONEval: {}", e));
        let parse_time = start_time.elapsed();
        println!("  Schema parsing & compilation: {:?}", parse_time);

        // Run evaluation iterations
        let eval_start = Instant::now();
        let mut evaluated_schema = Value::Null;
        let mut iteration_times = Vec::with_capacity(iterations);
        
        for iter in 0..iterations {
            let iter_start = Instant::now();
            eval
                .evaluate(&data_str, Some("{}"))
                .unwrap_or_else(|e| panic!("evaluation failed: {}", e));
            evaluated_schema = eval.get_evaluated_schema(false);
            let iter_time = iter_start.elapsed();
            iteration_times.push(iter_time);
            
            if iterations > 1 && (iter + 1) % 10 == 0 {
                print!(".");
                if (iter + 1) % 50 == 0 {
                    println!(" {}/{}", iter + 1, iterations);
                }
            }
        }
        
        if iterations > 1 && iterations % 50 != 0 {
            println!(" {}/{}", iterations, iterations);
        }
        
        let eval_time = eval_start.elapsed();
        
        // Calculate statistics
        if iterations == 1 {
            println!("  Evaluation: {:?}", eval_time);
        } else {
            let avg_time = eval_time / iterations as u32;
            let min_time = iteration_times.iter().min().unwrap();
            let max_time = iteration_times.iter().max().unwrap();
            
            println!("  Total evaluation time: {:?}", eval_time);
            println!("  Average per iteration: {:?}", avg_time);
            println!("  Min: {:?} | Max: {:?}", min_time, max_time);
            
            // Show cache statistics
            let cache_stats = eval.cache_stats();
            println!("  Cache: {} entries, {} hits, {} misses ({:.1}% hit rate)",
                cache_stats.entries,
                cache_stats.hits,
                cache_stats.misses,
                cache_stats.hit_rate * 100.0
            );
        }

        let total_time = start_time.elapsed();
        println!("⏱️  Execution time: {:?}\n", total_time);
        
        // Track statistics
        total_parse_time += parse_time;
        total_eval_time += eval_time;
        successful_scenarios += 1;

        let evaluated_value_path = samples_dir.join(format!("{}-evaluated-value.json", scenario.name));
        let evaluated_path = samples_dir.join(format!("{}-evaluated-schema.json", scenario.name));
        let parsed_path = samples_dir.join(format!("{}-parsed-schema.json", scenario.name));
        let sorted_path = samples_dir.join(format!("{}-sorted-evaluations.json", scenario.name));

        fs::write(
            &evaluated_path,
            pretty_json(&evaluated_schema),
        )
        .unwrap_or_else(|e| panic!("failed to write {}: {}", evaluated_path.display(), e));

        let mut metadata_obj = Map::new();
        metadata_obj.insert("dependencies".to_string(), serde_json::to_value(&eval.dependencies).unwrap());
        metadata_obj.insert("evaluations".to_string(), serde_json::to_value(&eval.evaluations).unwrap());
        metadata_obj.insert("tables".to_string(), serde_json::to_value(&eval.tables).unwrap());
        metadata_obj.insert("dependents_evaluations".to_string(), serde_json::to_value(&eval.dependents_evaluations).unwrap());
        metadata_obj.insert("rules_evaluations".to_string(), serde_json::to_value(&eval.rules_evaluations).unwrap());
        metadata_obj.insert("others_evaluations".to_string(), serde_json::to_value(&eval.others_evaluations).unwrap());
        metadata_obj.insert("value_evaluations".to_string(), serde_json::to_value(&eval.value_evaluations).unwrap());

        fs::write(&parsed_path, pretty_json(&Value::Object(metadata_obj)))
            .unwrap_or_else(|e| panic!("failed to write {}: {}", parsed_path.display(), e));

        let evaluated_values = &eval.get_schema_value().clone();
        fs::write(&evaluated_value_path, pretty_json(&evaluated_values))
            .unwrap_or_else(|e| panic!("failed to write {}: {}", evaluated_value_path.display(), e));

        fs::write(
            &sorted_path,
            serde_json::to_string_pretty(&eval.sorted_evaluations)
                .unwrap_or_else(|e| panic!("failed to serialize sorted evaluations: {}", e)),
        )
        .unwrap_or_else(|e| panic!("failed to write {}: {}", sorted_path.display(), e));

        println!("✅ Results saved:");
        println!("  - {}", evaluated_path.display());
        println!("  - {}", parsed_path.display());
        println!("  - {}\n", sorted_path.display());

        if let Some(comp_path) = &scenario.comparison_path {
            match fs::read_to_string(&comp_path) {
                Ok(comp_str) => match json_parser::parse_json_str(&comp_str) {
                    Ok(expected_value) => {
                        let actual_others = evaluated_schema.pointer("/$params/others");
                        let expected_others = expected_value.get("others");
                        match (actual_others, expected_others) {
                            (Some(actual), Some(expected)) => {
                                if actual == expected {
                                    println!(
                                        "🔍 Comparison: `$.others` matches `{}`",
                                        comp_path.display()
                                    );
                                } else {
                                    let mut diffs = Vec::new();
                                    collect_value_diffs(actual, expected, "$.others", &mut diffs);
                                    println!(
                                        "⚠️  Comparison: `$.others` differs from `{}` ({} differences):",
                                        comp_path.display(),
                                        diffs.len()
                                    );
                                    for diff in diffs {
                                        println!("  - {}", diff);
                                    }
                                }
                            }
                            (Some(_), None) => println!(
                                "⚠️  Comparison: `{}` missing `$.others` section",
                                comp_path.display()
                            ),
                            (None, Some(_)) => println!(
                                "⚠️  Comparison: evaluated schema missing `$.others` section"
                            ),
                            (None, None) => println!(
                                "ℹ️  Comparison: both evaluated and `{}` lack `$.others`",
                                comp_path.display()
                            ),
                        }
                    }
                    Err(err) => println!("⚠️  Failed to parse `{}`: {}", comp_path.display(), err),
                },
                Err(err) => println!(
                    "ℹ️  Comparison file `{}` not available: {}",
                    comp_path.display(),
                    err
                ),
            }
        }

    }
    
    // Print summary statistics
    if successful_scenarios > 0 {
        println!("\n{}", "=".repeat(50));
        println!("📊 Summary Statistics");
        println!("{}", "=".repeat(50));
        println!("Total scenarios run: {}", successful_scenarios);
        println!("Total parsing time: {:?}", total_parse_time);
        println!("Total evaluation time: {:?}", total_eval_time);
        println!("Total time: {:?}", total_parse_time + total_eval_time);
        
        if successful_scenarios > 1 {
            println!("\nAverage per scenario:");
            println!("  Parsing: {:?}", total_parse_time / successful_scenarios as u32);
            println!("  Evaluation: {:?}", total_eval_time / successful_scenarios as u32);
        }
        println!("\n✅ All scenarios completed successfully!\n");
    }
}
