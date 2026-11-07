use json_eval_rs::jsoneval::eval_data::EvalData;
use json_eval_rs::jsoneval::table_evaluate;
use json_eval_rs::{JSONEval, ParsedSchema};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

const PRODUCTS: &[&str] = &[
    "ilb", "ilo", "zcc", "zep", "zia", "zip", "zlw", "zpp", "zpr", "zwp",
];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TableMetric {
    pub name: String,
    pub row_count: usize,
    pub cold_duration_us: f64,
    pub warm_avg_duration_us: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductMetric {
    pub name: String,
    pub schema_size_bytes: usize,
    pub data_size_bytes: usize,
    pub cold_compile_ms: f64,
    pub cold_evaluate_ms: f64,
    pub warm_avg_evaluate_ms: f64,
    pub warm_min_evaluate_ms: f64,
    pub warm_max_evaluate_ms: f64,
    pub total_tables_count: usize,
    pub tables_total_cold_us: f64,
    pub tables_total_warm_us: f64,
    pub tables: Vec<TableMetric>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BenchmarkReport {
    pub label: String,
    pub iterations: usize,
    pub products: Vec<ProductMetric>,
}

fn load_file(rel_path: &str) -> Option<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(rel_path);
    fs::read_to_string(&path).ok()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut iterations = 10usize;
    let mut output_json: Option<String> = None;
    let mut label = "current".to_string();

    let mut i = 1;
    while i < args.len() {
        if args[i] == "-i" || args[i] == "--iterations" {
            if i + 1 < args.len() {
                iterations = args[i + 1].parse().unwrap_or(10);
                i += 1;
            }
        } else if args[i] == "--json" {
            if i + 1 < args.len() {
                output_json = Some(args[i + 1].clone());
                i += 1;
            }
        } else if args[i] == "--label" {
            if i + 1 < args.len() {
                label = args[i + 1].clone();
                i += 1;
            }
        }
        i += 1;
    }

    println!("\n================================================================================");
    println!("  RUNNING DEV PRODUCTS BENCHMARK [Label: {}, Iterations: {}]", label, iterations);
    println!("================================================================================\n");

    let mut product_metrics = Vec::new();

    for &product in PRODUCTS {
        let schema_path = format!("products/dev/schemas/{}.json", product);
        let data_path = format!("products/datas/{}.json", product);

        let schema_str = match load_file(&schema_path) {
            Some(s) => s,
            None => {
                eprintln!("Skipping {} (schema not found: {})", product, schema_path);
                continue;
            }
        };

        let data_str = match load_file(&data_path) {
            Some(d) => d,
            None => {
                eprintln!("Skipping {} (data not found: {})", product, data_path);
                continue;
            }
        };

        println!("--------------------------------------------------------------------------------");
        println!("  Product: [{}]", product.to_uppercase());
        println!("  Schema : {} ({} KB)", schema_path, schema_str.len() / 1024);
        println!("  Data   : {} ({} KB)", data_path, data_str.len() / 1024);
        println!("--------------------------------------------------------------------------------");

        // 1. Cold compile
        let compile_start = Instant::now();
        let parsed_schema = match ParsedSchema::parse(&schema_str) {
            Ok(ps) => Arc::new(ps),
            Err(e) => {
                eprintln!("  ❌ Failed to parse schema for {}: {}", product, e);
                continue;
            }
        };
        let cold_compile_dur = compile_start.elapsed();

        // 2. Cold evaluate
        let mut eval = match JSONEval::with_parsed_schema(Arc::clone(&parsed_schema), None, Some(&data_str)) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("  ❌ Failed to init eval for {}: {}", product, e);
                continue;
            }
        };

        let cold_eval_start = Instant::now();
        if let Err(e) = eval.evaluate(&data_str, None, None, None) {
            eprintln!("  ❌ Failed cold evaluate for {}: {}", product, e);
            continue;
        }
        let cold_eval_dur = cold_eval_start.elapsed();

        println!(
            "  ✓ Cold Compile : {:>9.3?} | Cold Full Evaluate: {:>9.3?}",
            cold_compile_dur, cold_eval_dur
        );

        // 3. Warm evaluate iterations
        let mut warm_durations = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            let mut iter_eval = JSONEval::with_parsed_schema(Arc::clone(&parsed_schema), None, Some(&data_str))
                .expect("warm eval init");
            let iter_start = Instant::now();
            iter_eval
                .evaluate(&data_str, None, None, None)
                .expect("warm eval evaluate");
            warm_durations.push(iter_start.elapsed());
        }

        let warm_total_ns: u128 = warm_durations.iter().map(|d| d.as_nanos()).sum();
        let warm_avg_ns = warm_total_ns as f64 / iterations as f64;
        let warm_avg_ms = warm_avg_ns / 1_000_000.0;
        let warm_min_ms = warm_durations.iter().map(|d| d.as_secs_f64() * 1000.0).fold(f64::INFINITY, f64::min);
        let warm_max_ms = warm_durations.iter().map(|d| d.as_secs_f64() * 1000.0).fold(0.0, f64::max);

        println!(
            "  ✓ Warm Eval (avg/min/max over {} iters): {:.3} ms / {:.3} ms / {:.3} ms",
            iterations, warm_avg_ms, warm_min_ms, warm_max_ms
        );

        // 4. Measure tables individually
        let table_keys: Vec<String> = eval.tables.keys().cloned().collect();
        println!("  ✓ Tables Declared: {}", table_keys.len());

        let scope_data = EvalData::from_arc(eval.eval_data.snapshot_data());
        let mut table_metrics = Vec::new();
        let mut tables_total_cold_us = 0.0;
        let mut tables_total_warm_us = 0.0;

        for table_key in &table_keys {
            // Cold table run
            let t_cold_start = Instant::now();
            let cold_res = table_evaluate::evaluate_table(&eval, table_key, &scope_data, None);
            let t_cold_dur = t_cold_start.elapsed();
            let t_cold_us = t_cold_dur.as_secs_f64() * 1_000_000.0;
            tables_total_cold_us += t_cold_us;

            let row_count = match &cold_res {
                Ok((rows_val, _)) => rows_val.as_array().map(|a| a.len()).unwrap_or(0),
                Err(_) => 0,
            };

            // Warm table runs (5 iters)
            let warm_iters = 5;
            let mut t_warm_total_us = 0.0;
            for _ in 0..warm_iters {
                let t_start = Instant::now();
                let _ = table_evaluate::evaluate_table(&eval, table_key, &scope_data, None);
                t_warm_total_us += t_start.elapsed().as_secs_f64() * 1_000_000.0;
            }
            let t_warm_avg_us = t_warm_total_us / warm_iters as f64;
            tables_total_warm_us += t_warm_avg_us;

            println!(
                "    - Table [{}] -> {} rows | cold: {:>8.2} µs | warm avg: {:>8.2} µs",
                table_key, row_count, t_cold_us, t_warm_avg_us
            );

            table_metrics.push(TableMetric {
                name: table_key.clone(),
                row_count,
                cold_duration_us: t_cold_us,
                warm_avg_duration_us: t_warm_avg_us,
            });
        }

        product_metrics.push(ProductMetric {
            name: product.to_string(),
            schema_size_bytes: schema_str.len(),
            data_size_bytes: data_str.len(),
            cold_compile_ms: cold_compile_dur.as_secs_f64() * 1000.0,
            cold_evaluate_ms: cold_eval_dur.as_secs_f64() * 1000.0,
            warm_avg_evaluate_ms: warm_avg_ms,
            warm_min_evaluate_ms: warm_min_ms,
            warm_max_evaluate_ms: warm_max_ms,
            total_tables_count: table_keys.len(),
            tables_total_cold_us,
            tables_total_warm_us,
            tables: table_metrics,
        });

        println!();
    }

    let report = BenchmarkReport {
        label: label.clone(),
        iterations,
        products: product_metrics,
    };

    if let Some(ref path) = output_json {
        let json_str = serde_json::to_string_pretty(&report).expect("serialize report");
        fs::write(path, json_str).expect("write report json");
        println!("✅ Benchmark metrics written to: {}\n", path);
    }
}
