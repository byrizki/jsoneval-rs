use datalogic_rs::DataLogic;
use json_eval_rs::{RLogic, RLogicConfig, TrackedData};
use serde_json::json;
use std::time::Instant;

fn benchmark_operation<F>(name: &str, iterations: usize, mut op: F)
where
    F: FnMut(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        op();
    }
    let duration = start.elapsed();
    let avg_micros = duration.as_micros() as f64 / iterations as f64;
    println!(
        "{:<40} {:>10} iterations in {:>8.2}ms (avg: {:>8.2}µs)",
        name,
        iterations,
        duration.as_secs_f64() * 1000.0,
        avg_micros
    );
}

fn benchmark_compile_compare(name: &str, logic: &serde_json::Value, iterations: usize) {
    println!("{}", name);
    benchmark_operation(&format!("  {:<32}", "RLogic compile"), iterations, || {
        let mut engine = RLogic::new();
        let _ = engine.compile(logic).unwrap();
    });

    benchmark_operation(&format!("  {:<32}", "datalogic-rs compile"), iterations, || {
        let engine = DataLogic::new();
        let _ = engine.compile(logic).unwrap();
    });
}

fn benchmark_eval_cached_compare(name: &str, logic: &serde_json::Value, data: &serde_json::Value, iterations: usize) {
    println!("{}", name);

    let mut rlogic = RLogic::new();
    let rlogic_id = rlogic.compile(logic).unwrap();
    let rlogic_data = TrackedData::new(data.clone());
    benchmark_operation(&format!("  {:<32}", "RLogic eval (cached)"), iterations, || {
        let _ = rlogic.evaluate(&rlogic_id, &rlogic_data);
    });

    let datalogic = DataLogic::new();
    let datalogic_compiled = datalogic.compile(logic).unwrap();
    let data_template = data.clone();
    benchmark_operation(&format!("  {:<32}", "datalogic-rs eval"), iterations, move || {
        let input = data_template.clone();
        let _ = datalogic.evaluate_owned(&datalogic_compiled, input).unwrap();
    });
}

fn benchmark_eval_newdata_compare(name: &str, logic: &serde_json::Value, data: &serde_json::Value, iterations: usize) {
    println!("{}", name);

    let mut rlogic = RLogic::new();
    let rlogic_id = rlogic.compile(logic).unwrap();
    let data_template = data.clone();
    benchmark_operation(&format!("  {:<32}", "RLogic eval (Tracked, no cache)"), iterations, || {
        let input = TrackedData::new(data_template.clone());
        let _ = rlogic.evaluate_uncached(&rlogic_id, &input);
    });
    benchmark_operation(&format!("  {:<32}", "RLogic eval (raw, no cache)"), iterations, || {
        let input = data_template.clone();
        let _ = rlogic.evaluate_raw(&rlogic_id, &input);
    });

    let datalogic = DataLogic::new();
    let datalogic_compiled = datalogic.compile(logic).unwrap();
    let data_template2 = data.clone();
    benchmark_operation(&format!("  {:<32}", "datalogic-rs eval (new data)"), iterations, move || {
        let input = data_template2.clone();
        let _ = datalogic.evaluate_owned(&datalogic_compiled, input).unwrap();
    });
}

fn benchmark_cache(name: &str, logic: &serde_json::Value, data: &serde_json::Value, iterations: usize) {
    println!("{}", name);

    let mut rlogic = RLogic::new();
    let logic_id = rlogic.compile(logic).unwrap();
    let tracked = TrackedData::new(data.clone());
    for _ in 0..10 {
        let _ = rlogic.evaluate(&logic_id, &tracked);
    }
    benchmark_operation(&format!("  {:<32}", "RLogic cached eval"), iterations, || {
        let _ = rlogic.evaluate(&logic_id, &tracked);
    });

    let datalogic = DataLogic::new();
    let compiled = datalogic.compile(logic).unwrap();
    let data_template = data.clone();
    benchmark_operation(&format!("  {:<32}", "datalogic-rs eval"), iterations, move || {
        let input = data_template.clone();
        let _ = datalogic.evaluate_owned(&compiled, input).unwrap();
    });
}

fn main() {
    println!("=== RLogic vs datalogic-rs Benchmarks ===\n");

    let compile_iterations = 10_000;
    let eval_iterations = 100_000;

    let simple_logic = json!({"+": [{"var": "a"}, {"var": "b"}]});
    let complex_logic = json!({
        "if": [
            {"and": [
                {">": [{"var": "user.age"}, 18]},
                {"==": [{"var": "user.premium"}, true]},
                {">": [{"var": "cart.total"}, 100]}
            ]},
            {"*": [{"var": "cart.total"}, 0.8]},
            {"var": "cart.total"}
        ]
    });
    let filter_logic = json!({
        "filter": [
            {"var": "numbers"},
            {">": [{"var": ""}, 5]}
        ]
    });

    println!("--- Compilation Performance ---");
    benchmark_compile_compare("Simple arithmetic", &simple_logic, compile_iterations);
    benchmark_compile_compare("Complex nested logic", &complex_logic, compile_iterations);
    benchmark_compile_compare("Array filter", &filter_logic, compile_iterations);

    println!("\n--- Evaluation Performance (cached data) ---");
    let simple_data = json!({"a": 10, "b": 20});
    benchmark_eval_cached_compare("Simple addition", &simple_logic, &simple_data, eval_iterations);

    let complex_data = json!({
        "user": {"age": 25, "premium": true},
        "cart": {"total": 150}
    });
    benchmark_eval_cached_compare("Complex conditional", &complex_logic, &complex_data, 50_000);

    let filter_data = json!({"numbers": [1, 3, 5, 7, 9, 11, 13]});
    benchmark_eval_cached_compare("Array filter", &filter_logic, &filter_data, 50_000);

    println!("\n--- Evaluation Performance (new data each time) ---");
    benchmark_eval_newdata_compare("Simple addition", &simple_logic, &simple_data, eval_iterations);

    println!("\n--- Cache Effectiveness ---");
    let cache_logic = json!({
        "+": [
            {"*": [{"var": "x"}, 2]},
            {"*": [{"var": "y"}, 3]},
            {"*": [{"var": "z"}, 4]}
        ]
    });
    let cache_data = json!({"x": 10, "y": 20, "z": 30});
    benchmark_cache("Cached hot path", &cache_logic, &cache_data, 1_000_000);

    println!("\n--- Configuration Comparison ---");
    benchmark_configs(&simple_logic, &simple_data, 100_000);

    println!("\n=== Benchmark Complete ===");
}

fn benchmark_configs(logic: &serde_json::Value, data: &serde_json::Value, iterations: usize) {
    println!("Configuration performance comparison");
    
    // Default config (cache + tracking)
    let mut engine_default = RLogic::new();
    let logic_id = engine_default.compile(logic).unwrap();
    let tracked_data = TrackedData::new(data.clone());
    benchmark_operation(&format!("  {:<32}", "Default (cache + tracking)"), iterations, || {
        let _ = engine_default.evaluate(&logic_id, &tracked_data);
    });
    
    // Performance config (cache, no tracking)
    let mut engine_perf = RLogic::with_config(RLogicConfig::performance());
    let logic_id_perf = engine_perf.compile(logic).unwrap();
    let tracked_data_perf = TrackedData::new(data.clone());
    benchmark_operation(&format!("  {:<32}", "Performance (cache only)"), iterations, || {
        let _ = engine_perf.evaluate(&logic_id_perf, &tracked_data_perf);
    });
    
    // Minimal config (no cache, no tracking)
    let mut engine_minimal = RLogic::with_config(RLogicConfig::minimal());
    let logic_id_minimal = engine_minimal.compile(logic).unwrap();
    let data_raw = data.clone();
    benchmark_operation(&format!("  {:<32}", "Minimal (no cache/tracking)"), iterations, || {
        let _ = engine_minimal.evaluate_raw(&logic_id_minimal, &data_raw);
    });
    
    // Safe config (all features)
    let mut engine_safe = RLogic::with_config(RLogicConfig::safe());
    let logic_id_safe = engine_safe.compile(logic).unwrap();
    let tracked_data_safe = TrackedData::new(data.clone());
    benchmark_operation(&format!("  {:<32}", "Safe (all features)"), iterations, || {
        let _ = engine_safe.evaluate(&logic_id_safe, &tracked_data_safe);
    });
}
