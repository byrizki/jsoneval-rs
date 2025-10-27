mod common;

use std::fs;
use std::path::Path;
use std::time::Instant;
use json_eval_rs::JSONEval;
use serde_json::{Map, Value};

fn print_help(program_name: &str) {
    println!("\n🚀 JSON Evaluation - Basic Example (MessagePack Schema)\n");
    println!("USAGE:");
    println!("    {} [OPTIONS] [FILTER]\n", program_name);
    println!("OPTIONS:");
    println!("    -h, --help         Show this help message");
    println!("    --compare          Enable comparison with expected results");
    println!("    --timing           Show detailed internal timing breakdown\n");
    println!("ARGUMENTS:");
    println!("    [FILTER]           Optional filter to match scenario names\n");
    println!("DESCRIPTION:");
    println!("    Evaluates MessagePack schemas using JSONEval::new_from_msgpack().\n");
    println!("EXAMPLES:");
    println!("    {}                 # Run all MessagePack schema scenarios", program_name);
    println!("    {} zcc             # Run scenarios matching 'zcc'", program_name);
    println!("    {} --compare       # Run with comparison enabled", program_name);
    println!("    {} zcc --timing    # Run with detailed timing breakdown", program_name);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program_name = args.get(0).map(|s| s.as_str()).unwrap_or("basic_msgpack");
    
    let mut scenario_filter: Option<String> = None;
    let mut enable_comparison = false;
    let mut show_timing = false;
    let mut i = 1;
    
    // Parse arguments
    while i < args.len() {
        let arg = &args[i];
        
        if arg == "-h" || arg == "--help" {
            print_help(program_name);
            return;
        } else if arg == "--compare" {
            enable_comparison = true;
        } else if arg == "--timing" {
            show_timing = true;
        } else if !arg.starts_with('-') {
            scenario_filter = Some(arg.clone());
        } else {
            eprintln!("Error: unknown option '{}'", arg);
            print_help(program_name);
            return;
        }
        
        i += 1;
    }
    
    println!("\n🚀 JSON Evaluation - Basic Example (MessagePack Schema)\n");
    
    if enable_comparison {
        println!("🔍 Comparison: enabled");
    }
    if show_timing {
        println!("⏱️  Internal timing: enabled");
    }
    if enable_comparison || show_timing {
        println!();
    }
    
    let samples_dir = Path::new("samples");
    let mut scenarios = common::discover_scenarios(samples_dir);
    
    // Filter to only MessagePack scenarios
    scenarios.retain(|s| s.is_msgpack);
    
    // Filter scenarios if a filter is provided
    if let Some(ref filter) = scenario_filter {
        scenarios.retain(|s| s.name.contains(filter));
        println!("📋 Filtering scenarios matching: '{}'\n", filter);
    }

    if scenarios.is_empty() {
        if let Some(filter) = scenario_filter {
            println!(
                "ℹ️  No MessagePack scenarios found matching '{}' in `{}`.",
                filter,
                samples_dir.display()
            );
        } else {
            println!(
                "ℹ️  No MessagePack scenarios discovered in `{}`. Add files like `name.bform` and `name-data.json`.",
                samples_dir.display()
            );
        }
        return;
    }
    
    println!("📊 Found {} MessagePack scenario(s)\n", scenarios.len());

    let mut total_parse_time = std::time::Duration::ZERO;
    let mut total_eval_time = std::time::Duration::ZERO;
    let mut successful_scenarios = 0;
    let mut comparison_failures = 0;

    for scenario in &scenarios {
        println!("==============================");
        println!("Scenario: {}", scenario.name);
        println!("Schema: {} (MessagePack)", scenario.schema_path.display());
        println!("Data: {}\n", scenario.data_path.display());

        // Clear timing data from previous scenarios
        if show_timing {
            json_eval_rs::enable_timing();
            json_eval_rs::clear_timing_data();
        }

        let data_str = fs::read_to_string(&scenario.data_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", scenario.data_path.display(), e));

        // Step 1: Parse schema (new_from_msgpack)
        let parse_start = Instant::now();
        
        let schema_msgpack = fs::read(&scenario.schema_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", scenario.schema_path.display(), e));
        
        println!("  📦 MessagePack schema size: {} bytes", schema_msgpack.len());
        
        let mut eval = JSONEval::new_from_msgpack(&schema_msgpack, None, Some(&data_str))
            .unwrap_or_else(|e| panic!("failed to create JSONEval from MessagePack: {}", e));
        
        let parse_time = parse_start.elapsed();
        println!("  📝 Parse (msgpack): {:?}", parse_time);
        
        // Step 2: Evaluate
        let eval_start = Instant::now();
        
        eval.evaluate(&data_str, Some("{}"))
            .unwrap_or_else(|e| panic!("evaluation failed: {}", e));
        
        let evaluated_schema = eval.get_evaluated_schema(false);
        let eval_time = eval_start.elapsed();
        
        println!("  ⚡ Eval: {:?}", eval_time);
        println!("  ⏱️  Total: {:?}\n", parse_time + eval_time);
        
        // Print detailed timing breakdown if --timing flag is set
        if show_timing {
            json_eval_rs::print_timing_summary();
        }
        
        total_parse_time += parse_time;
        total_eval_time += eval_time;
        successful_scenarios += 1;

        // Save results
        let evaluated_path = samples_dir.join(format!("{}-evaluated-schema.json", scenario.name));
        let parsed_path = samples_dir.join(format!("{}-parsed-schema.json", scenario.name));

        fs::write(&evaluated_path, common::pretty_json(&evaluated_schema))
            .unwrap_or_else(|e| panic!("failed to write {}: {}", evaluated_path.display(), e));

        let mut metadata_obj = Map::new();
        metadata_obj.insert("dependencies".to_string(), serde_json::to_value(&*eval.dependencies).unwrap());
        metadata_obj.insert("evaluations".to_string(), serde_json::to_value(&*eval.evaluations).unwrap());
        metadata_obj.insert("sorted_evaluations".to_string(), serde_json::to_value(&*eval.sorted_evaluations).unwrap());

        fs::write(&parsed_path, common::pretty_json(&Value::Object(metadata_obj)))
            .unwrap_or_else(|e| panic!("failed to write {}: {}", parsed_path.display(), e));

        println!("✅ Results saved:");
        println!("  - {}", evaluated_path.display());
        println!("  - {}\n", parsed_path.display());

        // Optional comparison
        if enable_comparison {
            if let Some(comp_path) = &scenario.comparison_path {
                if common::compare_with_expected(&evaluated_schema, comp_path).is_err() {
                    comparison_failures += 1;
                }
                println!();
            }
        }
    }
    
    // Print summary
    println!("{}", "=".repeat(50));
    println!("📊 Summary");
    println!("{}", "=".repeat(50));
    println!("Total scenarios run: {}", successful_scenarios);
    println!("Total parse time: {:?}", total_parse_time);
    println!("Total eval time: {:?}", total_eval_time);
    println!("Total time: {:?}", total_parse_time + total_eval_time);
    
    if successful_scenarios > 1 {
        println!("\nAverage per scenario:");
        println!("  Parse: {:?}", total_parse_time / successful_scenarios as u32);
        println!("  Eval: {:?}", total_eval_time / successful_scenarios as u32);
    }
    
    if enable_comparison {
        println!("Comparison failures: {}", comparison_failures);
    }
    
    println!("\n✅ All scenarios completed!\n");
}
