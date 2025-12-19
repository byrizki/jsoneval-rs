/// Demonstration of ParsedSchemaCache for schema reuse
/// 
/// This example shows how to:
/// 1. Create a cache instance (or use the global one)
/// 2. Parse and cache schemas with custom keys
/// 3. Retrieve and reuse cached schemas
/// 4. Manage cache lifecycle (clear, remove)
/// 5. Use cache for high-performance multi-evaluation scenarios

use json_eval_rs::{ParsedSchema, ParsedSchemaCache, JSONEval, PARSED_SCHEMA_CACHE};
use std::sync::Arc;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ParsedSchemaCache Demo\n");
    
    // Example 1: Using a local cache instance
    demo_local_cache()?;
    
    println!("\n{}\n", "=".repeat(60));
    
    // Example 2: Using the global cache instance
    demo_global_cache()?;
    
    println!("\n{}\n", "=".repeat(60));
    
    // Example 3: Performance comparison
    demo_performance_comparison()?;
    
    println!("\n{}\n", "=".repeat(60));
    
    // Example 4: get_or_insert_with pattern
    demo_lazy_insertion()?;
    
    Ok(())
}

fn demo_local_cache() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Example 1: Local Cache Instance");
    println!("Creating a dedicated cache for this application...\n");
    
    let cache = ParsedSchemaCache::new();
    
    // Simple schema
    let schema_json = r#"{
        "$params": {
            "rate": { "type": "number" }
        },
        "result": {
            "type": "number",
            "title": "Calculated Result",
            "$evaluation": {
                "logic": { "*": [{"var": "$rate"}, 100] }
            }
        }
    }"#;
    
    // Parse and cache with a custom key
    println!("📝 Parsing schema and caching with key 'calculation-v1'...");
    let parsed = ParsedSchema::parse(schema_json)?;
    cache.insert("calculation-v1".to_string(), Arc::new(parsed));
    
    println!("✅ Schema cached successfully");
    println!("   Cache size: {} entries", cache.len());
    println!("   Keys: {:?}\n", cache.keys());
    
    // Retrieve and use cached schema
    println!("🔍 Retrieving cached schema...");
    if let Some(cached_schema) = cache.get("calculation-v1") {
        println!("✅ Retrieved from cache");
        
        // Create JSONEval from cached ParsedSchema
        let mut eval = JSONEval::with_parsed_schema(cached_schema, Some(r#"{"rate": 1.5}"#), None)?;
        eval.evaluate("{}", None, None)?;
        
        let evaluated = eval.get_evaluated_schema(false);
        let result = evaluated.pointer("/result")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        println!("   Evaluation result: {}\n", result);
    }
    
    // Check cache stats
    let stats = cache.stats();
    println!("📊 Cache Statistics: {}", stats);
    
    // Remove entry
    println!("\n🗑️  Removing 'calculation-v1' from cache...");
    cache.remove("calculation-v1");
    println!("   Cache size after removal: {}", cache.len());
    
    Ok(())
}

fn demo_global_cache() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 Example 2: Global Cache Instance");
    println!("Using the built-in PARSED_SCHEMA_CACHE...\n");
    
    let schema_json = r#"{
        "$params": {
            "x": { "type": "number" },
            "y": { "type": "number" }
        },
        "sum": {
            "type": "number",
            "$evaluation": { "+": [{"var": "$x"}, {"var": "$y"}] }
        }
    }"#;
    
    // Use global cache
    println!("📝 Caching schema globally with key 'math-operations'...");
    let parsed = ParsedSchema::parse(schema_json)?;
    PARSED_SCHEMA_CACHE.insert("math-operations".to_string(), Arc::new(parsed));
    
    println!("✅ Schema cached globally");
    println!("   Global cache size: {}\n", PARSED_SCHEMA_CACHE.len());
    
    // Access from anywhere in the application
    simulate_another_function()?;
    
    // Clean up
    println!("\n🧹 Clearing global cache...");
    PARSED_SCHEMA_CACHE.clear();
    println!("   Global cache size: {}", PARSED_SCHEMA_CACHE.len());
    
    Ok(())
}

fn simulate_another_function() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 In another function, accessing global cache...");
    
    if let Some(cached) = PARSED_SCHEMA_CACHE.get("math-operations") {
        println!("✅ Retrieved schema from global cache");
        
        let mut eval = JSONEval::with_parsed_schema(cached, Some(r#"{"x": 10, "y": 20}"#), None)?;
        eval.evaluate("{}", None, None)?;
        
        let evaluated = eval.get_evaluated_schema(false);
        let sum = evaluated.pointer("/sum")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        println!("   Result: {}", sum);
    }
    
    Ok(())
}

fn demo_performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Example 3: Performance Comparison");
    println!("Comparing cached vs non-cached schema usage...\n");
    
    let schema_json = r#"{
        "$params": {
            "value": { "type": "number" }
        },
        "doubled": {
            "type": "number",
            "$evaluation": { "*": [{"var": "$value"}, 2] }
        },
        "tripled": {
            "type": "number",
            "$evaluation": { "*": [{"var": "$value"}, 3] }
        }
    }"#;
    
    let iterations = 100;
    
    // WITHOUT CACHE: Parse schema every time
    println!("🐌 Without cache (parse + evaluate each time):");
    let start = Instant::now();
    for i in 0..iterations {
        let context = format!(r#"{{"value": {}}}"#, i);
        let mut eval = JSONEval::new(schema_json, Some(&context), None)?;
        eval.evaluate("{}", None, None)?;
    }
    let without_cache = start.elapsed();
    println!("   Time: {:?}", without_cache);
    println!("   Avg per iteration: {:?}\n", without_cache / iterations);
    
    // WITH CACHE: Parse once, evaluate many times
    println!("🚀 With cache (parse once, reuse for all evaluations):");
    let cache = ParsedSchemaCache::new();
    
    // Parse once
    let parse_start = Instant::now();
    let parsed = ParsedSchema::parse(schema_json)?;
    cache.insert("perf-test".to_string(), Arc::new(parsed));
    let parse_time = parse_start.elapsed();
    
    // Evaluate many times
    let eval_start = Instant::now();
    for i in 0..iterations {
        if let Some(cached) = cache.get("perf-test") {
            let context = format!(r#"{{"value": {}}}"#, i);
            let mut eval = JSONEval::with_parsed_schema(cached.clone(), Some(&context), None)?;
            eval.evaluate("{}", None, None)?;
        }
    }
    let eval_time = eval_start.elapsed();
    let with_cache = parse_time + eval_time;
    
    println!("   Parse time: {:?}", parse_time);
    println!("   Eval time: {:?}", eval_time);
    println!("   Total time: {:?}", with_cache);
    println!("   Avg per iteration: {:?}\n", eval_time / iterations);
    
    let speedup = without_cache.as_secs_f64() / with_cache.as_secs_f64();
    println!("📈 Speedup: {:.2}x faster", speedup);
    
    Ok(())
}

fn demo_lazy_insertion() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Example 4: Lazy Insertion with get_or_insert_with");
    println!("Parse only if not already cached...\n");
    
    let cache = ParsedSchemaCache::new();
    
    let schema_json = r#"{
        "$params": {
            "name": { "type": "string" }
        },
        "greeting": {
            "type": "string",
            "$evaluation": {
                "cat": ["Hello, ", {"var": "$name"}, "!"]
            }
        }
    }"#;
    
    // First access: will parse
    println!("📝 First access (will parse)...");
    let start = Instant::now();
    let schema1 = cache.get_or_insert_with("greeting-schema", || {
        println!("   ⚙️  Parsing schema...");
        Arc::new(ParsedSchema::parse(schema_json).unwrap())
    });
    println!("   Time: {:?}\n", start.elapsed());
    
    // Second access: will use cached
    println!("🔍 Second access (will use cache)...");
    let start = Instant::now();
    let schema2 = cache.get_or_insert_with("greeting-schema", || {
        println!("   ⚙️  Parsing schema...");
        Arc::new(ParsedSchema::parse(schema_json).unwrap())
    });
    println!("   Time: {:?}", start.elapsed());
    
    // Verify they're the same Arc (pointer equality)
    println!("\n✅ Both accesses returned the same cached instance");
    println!("   Same pointer: {}", Arc::ptr_eq(&schema1, &schema2));
    
    Ok(())
}
