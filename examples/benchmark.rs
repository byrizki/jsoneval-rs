mod common;

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use json_eval_rs::{JSONEval, ParsedSchema};
use serde_json::{Map, Value};

fn print_help(program_name: &str) {
    println!("\n🚀 JSON Evaluation - Benchmark Example\n");
    println!("USAGE:");
    println!("    {} [OPTIONS] [FILTER]\n", program_name);
    println!("OPTIONS:");
    println!("    -h, --help                   Show this help message");
    println!("    -i, --iterations <COUNT>     Number of evaluation iterations (default: 1)");
    println!("    --parsed                     Use ParsedSchema for caching (parse once, reuse)");
    println!("    --concurrent <COUNT>         Test concurrent evaluations with N threads");
    println!("    --compare                    Enable comparison with expected results");
    println!("    --timing                     Show detailed internal timing breakdown");
    println!("    --cpu-info                   Show CPU feature information\n");
    println!("ARGUMENTS:");
    println!("    [FILTER]                     Optional filter to match scenario names\n");
    println!("EXAMPLES:");
    println!("    {} -i 100 zlw                # Run 'zlw' scenario 100 times", program_name);
    println!("    {} --parsed -i 100           # Use ParsedSchema, 100 iterations", program_name);
    println!("    {} --parsed --concurrent 4   # Test 4 concurrent evaluations", program_name);
    println!("    {} --compare                 # Run with comparison enabled", program_name);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program_name = args.get(0).map(|s| s.as_str()).unwrap_or("benchmark");
    
    let mut iterations = 1usize;
    let mut scenario_filter: Option<String> = None;
    let mut show_cpu_info = false;
    let mut use_parsed_schema = false;
    let mut concurrent_count: Option<usize> = None;
    let mut enable_comparison = false;
    let mut show_timing = false;
    let mut i = 1;
    
    // Parse arguments
    while i < args.len() {
        let arg = &args[i];
        
        if arg == "-h" || arg == "--help" {
            print_help(program_name);
            return;
        } else if arg == "--cpu-info" {
            show_cpu_info = true;
        } else if arg == "--parsed" {
            use_parsed_schema = true;
        } else if arg == "--compare" {
            enable_comparison = true;
        } else if arg == "--timing" {
            show_timing = true;
        } else if arg == "--concurrent" {
            if i + 1 >= args.len() {
                eprintln!("Error: {} requires a value", arg);
                print_help(program_name);
                return;
            }
            i += 1;
            match args[i].parse::<usize>() {
                Ok(n) if n > 0 => concurrent_count = Some(n),
                _ => {
                    eprintln!("Error: concurrent count must be a positive integer, got '{}'", args[i]);
                    return;
                }
            }
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
    
    println!("\n🚀 JSON Evaluation - Benchmark\n");
    
    // Show CPU info if requested or if running benchmarks
    if show_cpu_info || iterations > 1 || concurrent_count.is_some() {
        common::print_cpu_info();
    }
    
    if use_parsed_schema {
        println!("📦 Mode: ParsedSchema (parse once, reuse for all iterations)\n");
    }
    
    if let Some(count) = concurrent_count {
        println!("🔀 Concurrent evaluations: {} threads\n", count);
    } else if iterations > 1 {
        println!("🔄 Iterations per scenario: {}\n", iterations);
    }
    
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
        }
        return;
    }
    
    println!("📊 Found {} scenario(s)\n", scenarios.len());

    let mut total_parse_time = std::time::Duration::ZERO;
    let mut total_eval_time = std::time::Duration::ZERO;
    let mut successful_scenarios = 0;
    let mut comparison_failures = 0;

    for scenario in &scenarios {
        println!("==============================");
        println!("Scenario: {}", scenario.name);
        println!("Schema: {} ({})", 
            scenario.schema_path.display(),
            if scenario.is_msgpack { "MessagePack" } else { "JSON" }
        );
        println!("Data: {}\n", scenario.data_path.display());

        // Clear timing data from previous scenarios
        if show_timing {
            json_eval_rs::enable_timing();
            json_eval_rs::clear_timing_data();
        }

        let data_str = fs::read_to_string(&scenario.data_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", scenario.data_path.display(), e));

        println!("Running evaluation...\n");

        let (parse_time, eval_time, evaluated_schema, eval, iteration_times) = if use_parsed_schema {
            // ParsedSchema mode: parse once, reuse for all iterations/threads
            let start_time = Instant::now();
            
            let parsed_schema = if scenario.is_msgpack {
                let schema_msgpack = fs::read(&scenario.schema_path)
                    .unwrap_or_else(|e| panic!("failed to read {}: {}", scenario.schema_path.display(), e));
                println!("  📦 MessagePack schema size: {} bytes", schema_msgpack.len());
                Arc::new(ParsedSchema::parse_msgpack(&schema_msgpack)
                    .unwrap_or_else(|e| panic!("failed to parse MessagePack schema: {}", e)))
            } else {
                let schema_str = fs::read_to_string(&scenario.schema_path)
                    .unwrap_or_else(|e| panic!("failed to read {}: {}", scenario.schema_path.display(), e));
                Arc::new(ParsedSchema::parse(&schema_str)
                    .unwrap_or_else(|e| panic!("failed to parse schema: {}", e)))
            };
            
            let parse_time = start_time.elapsed();
            println!("  Schema parsing & compilation: {:?}", parse_time);
            
            // Concurrent mode with ParsedSchema
            if let Some(thread_count) = concurrent_count {
                use std::thread;
                
                let eval_start = Instant::now();
                let mut handles = vec![];
                
                for thread_id in 0..thread_count {
                    let parsed_clone = parsed_schema.clone();
                    let data_str_clone = data_str.clone();
                    let iter_count = iterations;
                    
                    let handle = thread::spawn(move || {
                        let mut thread_times = Vec::with_capacity(iter_count);
                        let mut last_schema = Value::Null;
                        
                        for _ in 0..iter_count {
                            let iter_start = Instant::now();
                            let mut eval_instance = JSONEval::with_parsed_schema(
                                parsed_clone.clone(),
                                Some("{}"),
                                Some(&data_str_clone)
                            ).unwrap();
                            
                            eval_instance.evaluate(&data_str_clone, Some("{}")).unwrap();
                            last_schema = eval_instance.get_evaluated_schema(false);
                            thread_times.push(iter_start.elapsed());
                        }
                        
                        (thread_times, last_schema, thread_id)
                    });
                    handles.push(handle);
                }
                
                let mut all_iteration_times = Vec::new();
                let mut evaluated_schema = Value::Null;
                
                for handle in handles {
                    let (thread_times, thread_schema, thread_id) = handle.join().unwrap();
                    println!("  Thread {} completed {} iterations", thread_id, thread_times.len());
                    all_iteration_times.extend(thread_times);
                    evaluated_schema = thread_schema; // Use last thread's result
                }
                
                let eval_time = eval_start.elapsed();
                
                // Create a temp eval for metadata export
                let temp_eval = JSONEval::with_parsed_schema(
                    parsed_schema.clone(),
                    Some("{}"),
                    Some(&data_str)
                ).unwrap();
                
                (parse_time, eval_time, evaluated_schema, temp_eval, all_iteration_times)
            } else {
                // Sequential iterations with ParsedSchema
                let eval_start = Instant::now();
                let mut evaluated_schema = Value::Null;
                let mut iteration_times = Vec::with_capacity(iterations);
                let mut eval_instance = JSONEval::with_parsed_schema(
                    parsed_schema.clone(),
                    Some("{}"),
                    Some(&data_str)
                ).unwrap();
                
                for iter in 0..iterations {
                    let iter_start = Instant::now();
                    eval_instance.evaluate(&data_str, Some("{}"))
                        .unwrap_or_else(|e| panic!("evaluation failed: {}", e));
                    evaluated_schema = eval_instance.get_evaluated_schema(false);
                    iteration_times.push(iter_start.elapsed());
                    
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
                (parse_time, eval_time, evaluated_schema, eval_instance, iteration_times)
            }
        } else {
            // Traditional mode: parse and create JSONEval each time
            let start_time = Instant::now();
            let mut eval = if scenario.is_msgpack {
                let schema_msgpack = fs::read(&scenario.schema_path)
                    .unwrap_or_else(|e| panic!("failed to read {}: {}", scenario.schema_path.display(), e));
                println!("  📦 MessagePack schema size: {} bytes", schema_msgpack.len());
                JSONEval::new_from_msgpack(&schema_msgpack, None, Some(&data_str))
                    .unwrap_or_else(|e| panic!("failed to create JSONEval from MessagePack: {}", e))
            } else {
                let schema_str = fs::read_to_string(&scenario.schema_path)
                    .unwrap_or_else(|e| panic!("failed to read {}: {}", scenario.schema_path.display(), e));
                JSONEval::new(&schema_str, None, Some(&data_str))
                    .unwrap_or_else(|e| panic!("failed to create JSONEval: {}", e))
            };
            let parse_time = start_time.elapsed();
            println!("  Schema parsing & compilation: {:?}", parse_time);
            
            let eval_start = Instant::now();
            let mut evaluated_schema = Value::Null;
            let mut iteration_times = Vec::with_capacity(iterations);
            
            for iter in 0..iterations {
                let iter_start = Instant::now();
                eval.evaluate(&data_str, Some("{}"))
                    .unwrap_or_else(|e| panic!("evaluation failed: {}", e));
                evaluated_schema = eval.get_evaluated_schema(false);
                iteration_times.push(iter_start.elapsed());
                
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
            (parse_time, eval_time, evaluated_schema, eval, iteration_times)
        };
        
        // Calculate statistics
        let total_iterations = iteration_times.len();
        if total_iterations == 1 {
            println!("  Evaluation: {:?}", eval_time);
        } else {
            let avg_time = eval_time / total_iterations as u32;
            let min_time = iteration_times.iter().min().unwrap();
            let max_time = iteration_times.iter().max().unwrap();
            
            println!("  Total evaluation time: {:?}", eval_time);
            println!("  Total iterations: {}", total_iterations);
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

        let total_time = parse_time + eval_time;
        println!("⏱️  Execution time: {:?}\n", total_time);
        
        // Print detailed timing breakdown if --timing flag is set
        if show_timing {
            json_eval_rs::print_timing_summary();
        }
        
        // Track statistics
        total_parse_time += parse_time;
        total_eval_time += eval_time;
        successful_scenarios += 1;

        let evaluated_path = samples_dir.join(format!("{}-evaluated-schema.json", scenario.name));
        let parsed_path = samples_dir.join(format!("{}-parsed-schema.json", scenario.name));

        fs::write(&evaluated_path, common::pretty_json(&evaluated_schema))
            .unwrap_or_else(|e| panic!("failed to write {}: {}", evaluated_path.display(), e));

        let mut metadata_obj = Map::new();
        metadata_obj.insert("dependencies".to_string(), serde_json::to_value(&*eval.dependencies).unwrap());
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
        
        if enable_comparison {
            println!("\nComparison failures: {}", comparison_failures);
        }
        
        println!("\n✅ All scenarios completed successfully!\n");
    }
}
