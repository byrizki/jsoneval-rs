use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use json_eval_rs::{json_parser, JSONEval, ParsedSchema};
use serde_json::Value;
use rmp_serde;

fn print_parsed_schema_info(
    parsed_schema: &ParsedSchema,
    print_sorted: bool,
    print_deps: bool,
    print_tables: bool,
    print_evals: bool,
) {
    if !print_sorted && !print_deps && !print_tables && !print_evals {
        return;
    }
    
    println!("\n{}", "=".repeat(60));
    println!("📋 PARSED SCHEMA INFORMATION");
    println!("{}", "=".repeat(60));
    
    if print_sorted {
        println!("\n🔄 Sorted Evaluations (Batches for parallel execution):");
        println!("   {} batches total", parsed_schema.sorted_evaluations.len());
        for (batch_idx, batch) in parsed_schema.sorted_evaluations.iter().enumerate() {
            println!("\n   Batch {} ({} evaluations):", batch_idx + 1, batch.len());
            for eval_key in batch {
                println!("     - {}", eval_key);
            }
        }
    }
    
    if print_deps {
        println!("\n🔗 Dependencies:");
        println!("   {} evaluation(s) with dependencies", parsed_schema.dependencies.len());
        for (eval_key, deps) in &parsed_schema.dependencies {
            if !deps.is_empty() {
                println!("\n   {} depends on:", eval_key);
                for dep in deps {
                    println!("     → {}", dep);
                }
            }
        }
    }
    
    if print_tables {
        println!("\n📊 Tables:");
        println!("   {} table(s) defined", parsed_schema.tables.len());
        for (table_key, table_value) in &parsed_schema.tables {
            println!("\n   {}:", table_key);
            println!("     {}", serde_json::to_string_pretty(table_value).unwrap_or_default());
        }
    }
    
    if print_evals {
        println!("\n⚙️  Evaluations:");
        println!("   {} evaluation(s) compiled", parsed_schema.evaluations.len());
        for (eval_key, logic_id) in &parsed_schema.evaluations {
            println!("     {} → {:?}", eval_key, logic_id);
        }
    }
    
    println!("\n{}", "=".repeat(60));
    println!();
}

fn print_help(program_name: &str) {
    println!("\n🚀 JSON Evaluation CLI\n");
    println!("USAGE:");
    println!("    {} <SCHEMA_FILE> [OPTIONS]\n", program_name);
    println!("ARGUMENTS:");
    println!("    <SCHEMA_FILE>              Path to schema file (.json or .bform)\n");
    println!("OPTIONS:");
    println!("    -h, --help                 Show this help message");
    println!("    -d, --data <FILE>          Input data file (JSON or .bform)");
    println!("    -c, --compare <FILE>       Expected output file for comparison");
    println!("    --compare-path <PATH>      JSON pointer path for comparison (default: \"$.$params.others\")");
    println!("    -p, --parsed               Use ParsedSchema for efficient caching");
    println!("    -i, --iterations <N>       Number of evaluation iterations (default: 1)");
    println!("    -o, --output <FILE>        Output file for evaluated schema (default: stdout)");
    println!("    --no-output                Suppress output (for benchmarking)");
    println!("\nPARSED SCHEMA INSPECTION:");
    println!("    --print-sorted-evaluations Print sorted evaluation batches");
    println!("    --print-dependencies       Print dependency graph");
    println!("    --print-tables             Print table definitions");
    println!("    --print-evaluations        Print all evaluations with compiled logic IDs");
    println!("    --print-all                Print all parsed schema information\n");
    println!("EXAMPLES:");
    println!("    # Simple evaluation");
    println!("    {} schema.json -d data.json\n", program_name);
    println!("    # With comparison");
    println!("    {} schema.json -d data.json -c expected.json\n", program_name);
    println!("    # Using ParsedSchema with iterations");
    println!("    {} schema.json -d data.json --parsed -i 100\n", program_name);
    println!("    # Full benchmark with custom comparison path");
    println!("    {} schema.json -d data.json -c expected.json --compare-path \"$.result\" --parsed -i 100", program_name);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program_name = args.get(0).map(|s| s.as_str()).unwrap_or("json-eval-cli");
    
    // Parse arguments
    let mut schema_file: Option<PathBuf> = None;
    let mut data_file: Option<PathBuf> = None;
    let mut compare_file: Option<PathBuf> = None;
    let mut compare_path = "$.$params.others".to_string();
    let mut use_parsed = false;
    let mut iterations = 1usize;
    let mut output_file: Option<PathBuf> = None;
    let mut no_output = false;
    let mut print_sorted_evaluations = false;
    let mut print_dependencies = false;
    let mut print_tables = false;
    let mut print_evaluations = false;
    let mut i = 1;
    
    while i < args.len() {
        let arg = &args[i];
        
        if arg == "-h" || arg == "--help" {
            print_help(program_name);
            return;
        } else if arg == "-d" || arg == "--data" {
            if i + 1 >= args.len() {
                eprintln!("Error: {} requires a value", arg);
                print_help(program_name);
                std::process::exit(1);
            }
            i += 1;
            data_file = Some(PathBuf::from(&args[i]));
        } else if arg == "-c" || arg == "--compare" {
            if i + 1 >= args.len() {
                eprintln!("Error: {} requires a value", arg);
                print_help(program_name);
                std::process::exit(1);
            }
            i += 1;
            compare_file = Some(PathBuf::from(&args[i]));
        } else if arg == "--compare-path" {
            if i + 1 >= args.len() {
                eprintln!("Error: {} requires a value", arg);
                print_help(program_name);
                std::process::exit(1);
            }
            i += 1;
            compare_path = args[i].clone();
        } else if arg == "-p" || arg == "--parsed" {
            use_parsed = true;
        } else if arg == "-i" || arg == "--iterations" {
            if i + 1 >= args.len() {
                eprintln!("Error: {} requires a value", arg);
                print_help(program_name);
                std::process::exit(1);
            }
            i += 1;
            match args[i].parse::<usize>() {
                Ok(n) if n > 0 => iterations = n,
                _ => {
                    eprintln!("Error: iterations must be a positive integer, got '{}'", args[i]);
                    std::process::exit(1);
                }
            }
        } else if arg == "-o" || arg == "--output" {
            if i + 1 >= args.len() {
                eprintln!("Error: {} requires a value", arg);
                print_help(program_name);
                std::process::exit(1);
            }
            i += 1;
            output_file = Some(PathBuf::from(&args[i]));
        } else if arg == "--no-output" {
            no_output = true;
        } else if arg == "--print-sorted-evaluations" {
            print_sorted_evaluations = true;
        } else if arg == "--print-dependencies" {
            print_dependencies = true;
        } else if arg == "--print-tables" {
            print_tables = true;
        } else if arg == "--print-evaluations" {
            print_evaluations = true;
        } else if arg == "--print-all" {
            print_sorted_evaluations = true;
            print_dependencies = true;
            print_tables = true;
            print_evaluations = true;
        } else if !arg.starts_with('-') {
            if schema_file.is_none() {
                schema_file = Some(PathBuf::from(arg));
            } else {
                eprintln!("Error: unexpected positional argument '{}'", arg);
                print_help(program_name);
                std::process::exit(1);
            }
        } else {
            eprintln!("Error: unknown option '{}'", arg);
            print_help(program_name);
            std::process::exit(1);
        }
        
        i += 1;
    }
    
    // Validate required arguments
    let schema_file = match schema_file {
        Some(f) => f,
        None => {
            eprintln!("Error: schema file is required\n");
            print_help(program_name);
            std::process::exit(1);
        }
    };
    
    if !schema_file.exists() {
        eprintln!("Error: schema file '{}' not found", schema_file.display());
        std::process::exit(1);
    }
    
    // Determine if schema is MessagePack
    let is_schema_msgpack = schema_file.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "bform")
        .unwrap_or(false);
    
    // Load data file if provided, otherwise use empty object
    // Detect if data file is MessagePack based on extension
    let (data_str, is_data_msgpack) = if let Some(ref data_path) = data_file {
        if !data_path.exists() {
            eprintln!("Error: data file '{}' not found", data_path.display());
            std::process::exit(1);
        }
        
        let is_msgpack = data_path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "bform")
            .unwrap_or(false);
        
        if is_msgpack {
            // For MessagePack data, we still need JSON string for some operations
            // Parse MessagePack to JSON and convert back to string
            let data_bytes = fs::read(data_path)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to read data file '{}': {}", data_path.display(), e);
                    std::process::exit(1);
                });
            
            let data_value: Value = rmp_serde::from_slice(&data_bytes)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to parse MessagePack data file '{}': {}", data_path.display(), e);
                    std::process::exit(1);
                });
            
            (serde_json::to_string(&data_value).unwrap_or_else(|_| "{}".to_string()), true)
        } else {
            let data_str = fs::read_to_string(data_path)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to read data file '{}': {}", data_path.display(), e);
                    std::process::exit(1);
                });
            (data_str, false)
        }
    } else {
        ("{}".to_string(), false)
    };
    
    println!("\n🚀 JSON Evaluation CLI\n");
    println!("📄 Schema: {} ({})", schema_file.display(), if is_schema_msgpack { "MessagePack" } else { "JSON" });
    if let Some(ref data_path) = data_file {
        println!("📊 Data: {} ({})", data_path.display(), if is_data_msgpack { "MessagePack" } else { "JSON" });
    }
    if use_parsed {
        println!("📦 Mode: ParsedSchema (parse once, reuse)");
    }
    if iterations > 1 {
        println!("🔄 Iterations: {}", iterations);
    }
    if compare_file.is_some() {
        println!("🔍 Comparison: enabled (path: {})", compare_path);
    }
    println!();
    
    // Run evaluation
    let total_start = Instant::now();
    
    let (evaluated_schema, _parse_time, eval_time) = if use_parsed {
        // ParsedSchema mode
        let parse_start = Instant::now();
        let parsed_schema = if is_schema_msgpack {
            let schema_bytes = fs::read(&schema_file)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to read schema: {}", e);
                    std::process::exit(1);
                });
            Arc::new(ParsedSchema::parse_msgpack(&schema_bytes)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to parse MessagePack schema: {}", e);
                    std::process::exit(1);
                }))
        } else {
            let schema_str = fs::read_to_string(&schema_file)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to read schema: {}", e);
                    std::process::exit(1);
                });
            Arc::new(ParsedSchema::parse(&schema_str)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to parse schema: {}", e);
                    std::process::exit(1);
                }))
        };
        let parse_time = parse_start.elapsed();
        
        println!("⏱️  Schema parsing: {:?}", parse_time);
        
        // Print parsed schema information if requested
        print_parsed_schema_info(
            &parsed_schema,
            print_sorted_evaluations,
            print_dependencies,
            print_tables,
            print_evaluations,
        );
        
        // Run iterations
        let eval_start = Instant::now();
        let mut result_schema = Value::Null;
        
        for iter in 0..iterations {
            let mut eval = JSONEval::with_parsed_schema(
                parsed_schema.clone(),
                Some("{}"),
                Some(&data_str)
            ).unwrap_or_else(|e| {
                eprintln!("Error: failed to create JSONEval: {}", e);
                std::process::exit(1);
            });
            
            eval.evaluate(&data_str, Some("{}"))
                .unwrap_or_else(|e| {
                    eprintln!("Error: evaluation failed: {}", e);
                    std::process::exit(1);
                });
            
            result_schema = eval.get_evaluated_schema(false);
            
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
        (result_schema, parse_time, eval_time)
    } else {
        // Traditional mode
        let start = Instant::now();
        let mut eval = if is_schema_msgpack {
            let schema_bytes = fs::read(&schema_file)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to read schema: {}", e);
                    std::process::exit(1);
                });
            JSONEval::new_from_msgpack(&schema_bytes, None, Some(&data_str))
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to create JSONEval from MessagePack: {}", e);
                    std::process::exit(1);
                })
        } else {
            let schema_str = fs::read_to_string(&schema_file)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to read schema: {}", e);
                    std::process::exit(1);
                });
            JSONEval::new(&schema_str, None, Some(&data_str))
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to create JSONEval: {}", e);
                    std::process::exit(1);
                })
        };
        let parse_time = start.elapsed();
        
        println!("⏱️  Schema parsing & compilation: {:?}", parse_time);
        
        // Run iterations
        let eval_start = Instant::now();
        let mut result_schema = Value::Null;
        
        for iter in 0..iterations {
            eval.evaluate(&data_str, Some("{}"))
                .unwrap_or_else(|e| {
                    eprintln!("Error: evaluation failed: {}", e);
                    std::process::exit(1);
                });
            
            result_schema = eval.get_evaluated_schema(false);
            
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
        (result_schema, parse_time, eval_time)
    };
    
    let total_time = total_start.elapsed();
    
    // Print timing statistics
    println!("\n⏱️  Evaluation: {:?}", eval_time);
    if iterations > 1 {
        println!("   Average per iteration: {:?}", eval_time / iterations as u32);
    }
    println!("⏱️  Total time: {:?}\n", total_time);
    
    // Handle output
    if !no_output {
        let output_json = serde_json::to_string_pretty(&evaluated_schema)
            .unwrap_or_else(|e| {
                eprintln!("Error: failed to serialize output: {}", e);
                std::process::exit(1);
            });
        
        if let Some(output_path) = output_file {
            fs::write(&output_path, output_json)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to write output file: {}", e);
                    std::process::exit(1);
                });
            println!("✅ Output written to: {}\n", output_path.display());
        } else {
            println!("📋 Evaluated Schema:");
            println!("{}\n", output_json);
        }
    }
    
    // Handle comparison
    if let Some(compare_path_file) = compare_file {
        if !compare_path_file.exists() {
            eprintln!("Warning: comparison file '{}' not found", compare_path_file.display());
        } else {
            let expected_str = fs::read_to_string(&compare_path_file)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to read comparison file: {}", e);
                    std::process::exit(1);
                });
            
            let expected: Value = json_parser::parse_json_str(&expected_str)
                .unwrap_or_else(|e| {
                    eprintln!("Error: failed to parse comparison file: {}", e);
                    std::process::exit(1);
                });
            
            // Extract values from comparison path
            let actual_value = evaluated_schema.pointer(&compare_path);
            let expected_value = expected.pointer(&compare_path)
                .or_else(|| expected.get("others")); // Fallback to "others" for compatibility
            
            match (actual_value, expected_value) {
                (Some(actual), Some(expected)) => {
                    if actual == expected {
                        println!("✅ Comparison passed: values match at '{}'", compare_path);
                    } else {
                        println!("❌ Comparison failed: values differ at '{}'", compare_path);
                        println!("   Expected: {}", serde_json::to_string(expected).unwrap_or_default());
                        println!("   Actual:   {}", serde_json::to_string(actual).unwrap_or_default());
                        std::process::exit(1);
                    }
                }
                (None, Some(_)) => {
                    println!("❌ Comparison failed: path '{}' not found in evaluated schema", compare_path);
                    std::process::exit(1);
                }
                (Some(_), None) => {
                    println!("❌ Comparison failed: path '{}' not found in expected output", compare_path);
                    std::process::exit(1);
                }
                (None, None) => {
                    println!("⚠️  Warning: path '{}' not found in either schema", compare_path);
                }
            }
        }
    }
    
    println!("✅ Evaluation completed successfully!\n");
}
