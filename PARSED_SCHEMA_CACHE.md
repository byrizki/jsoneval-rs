# ParsedSchemaCache Documentation

## Overview

The `ParsedSchemaCache` is a built-in, thread-safe cache for storing and reusing `Arc<ParsedSchema>` instances. This enables high-performance scenarios where schemas are parsed once and reused across multiple evaluations.

## Features

- ✅ **Thread-safe**: Uses `Arc<RwLock<>>` for concurrent access
- ✅ **Caller-controlled**: You decide when to parse, cache, clear, and release memory
- ✅ **Flexible keys**: Use any string identifier for your cached schemas
- ✅ **Arc-based**: Cheap cloning via `Arc` reference counting
- ✅ **Global or local**: Use the built-in global cache or create your own instances
- ✅ **Zero-copy**: Cached schemas share memory across evaluations

## Use Cases

### 1. **Web Servers**
Parse schemas once at startup, cache them, and reuse for every request.

### 2. **Batch Processing**
Parse schema once, evaluate against thousands of data files.

### 3. **Multi-tenant Applications**
Cache schemas per tenant, reload when tenant schema changes.

### 4. **Schema Versioning**
Cache multiple versions of a schema simultaneously.

## API Reference

### Creating a Cache

```rust
use json_eval_rs::ParsedSchemaCache;

// Create a local cache instance
let cache = ParsedSchemaCache::new();

// Or use the global cache
use json_eval_rs::PARSED_SCHEMA_CACHE;
```

### Basic Operations

#### Insert
```rust
use json_eval_rs::{ParsedSchema, ParsedSchemaCache};
use std::sync::Arc;

let cache = ParsedSchemaCache::new();
let parsed = ParsedSchema::parse(schema_json)?;
cache.insert("my-schema".to_string(), Arc::new(parsed));
```

#### Get
```rust
if let Some(cached) = cache.get("my-schema") {
    // Use cached schema
    let mut eval = JSONEval::with_parsed_schema(cached, Some(context), None)?;
}
```

#### Remove
```rust
// Remove and return the schema
if let Some(removed) = cache.remove("my-schema") {
    println!("Removed schema");
}
```

#### Clear
```rust
// Remove all cached schemas
cache.clear();
```

### Advanced Operations

#### Check Existence
```rust
if cache.contains_key("my-schema") {
    println!("Schema exists in cache");
}
```

#### Get Cache Size
```rust
println!("Cache has {} entries", cache.len());
println!("Cache is empty: {}", cache.is_empty());
```

#### List All Keys
```rust
let keys = cache.keys();
for key in keys {
    println!("Cached schema: {}", key);
}
```

#### Get Statistics
```rust
let stats = cache.stats();
println!("{}", stats); // "ParsedSchemaCache: 3 entries (keys: schema1, schema2, schema3)"
```

#### Lazy Insertion
```rust
// Parse only if not already cached
let schema = cache.get_or_insert_with("my-schema", || {
    Arc::new(ParsedSchema::parse(schema_json).unwrap())
});
```

#### Batch Operations
```rust
// Insert multiple schemas at once
cache.insert_batch(vec![
    ("schema1".to_string(), Arc::new(parsed1)),
    ("schema2".to_string(), Arc::new(parsed2)),
]);

// Remove multiple schemas at once
let removed = cache.remove_batch(&["schema1".to_string(), "schema2".to_string()]);
```

## Usage Patterns

### Pattern 1: Application-Level Cache

```rust
use json_eval_rs::ParsedSchemaCache;
use std::sync::Arc;

struct MyApp {
    schema_cache: ParsedSchemaCache,
}

impl MyApp {
    fn new() -> Self {
        Self {
            schema_cache: ParsedSchemaCache::new(),
        }
    }
    
    fn load_schema(&self, key: &str, json: &str) -> Result<(), String> {
        let parsed = ParsedSchema::parse(json)?;
        self.schema_cache.insert(key.to_string(), Arc::new(parsed));
        Ok(())
    }
    
    fn evaluate(&self, schema_key: &str, data: &str) -> Result<String, String> {
        let cached = self.schema_cache.get(schema_key)
            .ok_or("Schema not found in cache")?;
        
        let mut eval = JSONEval::with_parsed_schema(cached, None, None)?;
        eval.evaluate(data, None)?;
        
        Ok(serde_json::to_string(&eval.get_evaluated_schema(false)).unwrap())
    }
}
```

### Pattern 2: Global Cache for Simple Applications

```rust
use json_eval_rs::PARSED_SCHEMA_CACHE;
use std::sync::Arc;

// At application startup
fn init() {
    let schema1 = ParsedSchema::parse(schema_json1).unwrap();
    PARSED_SCHEMA_CACHE.insert("v1".to_string(), Arc::new(schema1));
    
    let schema2 = ParsedSchema::parse(schema_json2).unwrap();
    PARSED_SCHEMA_CACHE.insert("v2".to_string(), Arc::new(schema2));
}

// Anywhere in your application
fn process(version: &str, data: &str) -> Result<(), String> {
    let cached = PARSED_SCHEMA_CACHE.get(version)
        .ok_or("Schema version not found")?;
    
    let mut eval = JSONEval::with_parsed_schema(cached, None, None)?;
    eval.evaluate(data, None)?;
    
    Ok(())
}
```

### Pattern 3: Multi-Tenant Schema Management

```rust
use json_eval_rs::ParsedSchemaCache;

struct TenantSchemaManager {
    cache: ParsedSchemaCache,
}

impl TenantSchemaManager {
    fn new() -> Self {
        Self {
            cache: ParsedSchemaCache::new(),
        }
    }
    
    fn load_tenant_schema(&self, tenant_id: &str, schema_json: &str) -> Result<(), String> {
        let key = format!("tenant:{}", tenant_id);
        let parsed = ParsedSchema::parse(schema_json)?;
        self.cache.insert(key, Arc::new(parsed));
        Ok(())
    }
    
    fn reload_tenant_schema(&self, tenant_id: &str, schema_json: &str) -> Result<(), String> {
        let key = format!("tenant:{}", tenant_id);
        // Remove old schema
        self.cache.remove(&key);
        // Parse and cache new schema
        let parsed = ParsedSchema::parse(schema_json)?;
        self.cache.insert(key, Arc::new(parsed));
        Ok(())
    }
    
    fn process_tenant_data(&self, tenant_id: &str, data: &str) -> Result<String, String> {
        let key = format!("tenant:{}", tenant_id);
        let cached = self.cache.get(&key)
            .ok_or("Tenant schema not found")?;
        
        let mut eval = JSONEval::with_parsed_schema(cached, None, None)?;
        eval.evaluate(data, None)?;
        
        Ok(serde_json::to_string(&eval.get_evaluated_schema(false)).unwrap())
    }
    
    fn remove_tenant(&self, tenant_id: &str) {
        let key = format!("tenant:{}", tenant_id);
        self.cache.remove(&key);
    }
}
```

### Pattern 4: Schema Versioning

```rust
use json_eval_rs::ParsedSchemaCache;

struct VersionedSchemaCache {
    cache: ParsedSchemaCache,
}

impl VersionedSchemaCache {
    fn new() -> Self {
        Self {
            cache: ParsedSchemaCache::new(),
        }
    }
    
    fn load_version(&self, version: &str, schema_json: &str) -> Result<(), String> {
        let key = format!("v{}", version);
        let parsed = ParsedSchema::parse(schema_json)?;
        self.cache.insert(key, Arc::new(parsed));
        Ok(())
    }
    
    fn evaluate_with_version(&self, version: &str, data: &str) -> Result<String, String> {
        let key = format!("v{}", version);
        let cached = self.cache.get(&key)
            .ok_or(format!("Schema version {} not found", version))?;
        
        let mut eval = JSONEval::with_parsed_schema(cached, None, None)?;
        eval.evaluate(data, None)?;
        
        Ok(serde_json::to_string(&eval.get_evaluated_schema(false)).unwrap())
    }
    
    fn list_versions(&self) -> Vec<String> {
        self.cache.keys()
            .into_iter()
            .filter_map(|k| k.strip_prefix("v").map(String::from))
            .collect()
    }
}
```

## Performance Characteristics

### Memory
- **Cache overhead**: `O(n)` where n is the number of cached schemas
- **Per-schema**: Shared via `Arc`, minimal overhead per clone
- **Thread safety**: `RwLock` allows multiple concurrent readers

### Speed
- **Insert/Remove**: `O(1)` average case (HashMap under IndexMap)
- **Get**: `O(1)` average case, cheap Arc clone
- **Clear**: `O(n)` where n is the number of entries

### Benchmarks (from example)
Testing with 100 iterations:
- **Without cache**: ~118μs per iteration (parse + evaluate each time)
- **With cache**: ~54μs per iteration (parse once, evaluate many)
- **Speedup**: ~2.15x faster

Real-world improvements can be much higher (10x-100x) for complex schemas.

## Thread Safety

The cache is fully thread-safe and can be used across multiple threads:

```rust
use json_eval_rs::ParsedSchemaCache;
use std::sync::Arc;
use std::thread;

let cache = Arc::new(ParsedSchemaCache::new());

// Thread 1: Insert
let cache1 = cache.clone();
thread::spawn(move || {
    let parsed = ParsedSchema::parse(schema).unwrap();
    cache1.insert("shared".to_string(), Arc::new(parsed));
});

// Thread 2: Read
let cache2 = cache.clone();
thread::spawn(move || {
    if let Some(cached) = cache2.get("shared") {
        // Use cached schema
    }
});
```

## Best Practices

### 1. **Choose the Right Cache**
- Use **global cache** (`PARSED_SCHEMA_CACHE`) for simple applications
- Use **local cache** for better control and testing

### 2. **Key Naming Conventions**
```rust
// Good: Descriptive, namespaced keys
cache.insert("tenant:123:v2".to_string(), schema);
cache.insert("calculation-engine:v1".to_string(), schema);

// Avoid: Generic keys that may conflict
cache.insert("schema".to_string(), schema);
cache.insert("1".to_string(), schema);
```

### 3. **Memory Management**
```rust
// Remove unused schemas to free memory
cache.remove("old-schema");

// Clear cache when doing bulk reloads
cache.clear();
for (key, schema) in new_schemas {
    cache.insert(key, schema);
}
```

### 4. **Error Handling**
```rust
// Always handle missing schemas gracefully
let schema = cache.get("my-schema")
    .ok_or("Schema not found. Please load it first.")?;
```

### 5. **Lazy Loading**
```rust
// Use get_or_insert_with to avoid redundant parsing
let schema = cache.get_or_insert_with("my-schema", || {
    // Only executes if not already cached
    Arc::new(ParsedSchema::parse(schema_json).unwrap())
});
```

## Examples

See `examples/cache_demo.rs` for a comprehensive demonstration of all cache features:

```bash
cargo run --example cache_demo
```

This example demonstrates:
1. Local cache instance usage
2. Global cache usage
3. Performance comparison (cached vs non-cached)
4. Lazy insertion pattern

## Integration with Existing Code

### Before (No Cache)
```rust
fn evaluate_data(schema_json: &str, data: &str) -> Result<String, String> {
    let mut eval = JSONEval::new(schema_json, None, Some(data))?;
    eval.evaluate(data, None)?;
    Ok(serde_json::to_string(&eval.get_evaluated_schema(false)).unwrap())
}
```

### After (With Cache)
```rust
fn evaluate_data(
    cache: &ParsedSchemaCache,
    schema_key: &str,
    data: &str
) -> Result<String, String> {
    let cached = cache.get(schema_key)
        .ok_or("Schema not found in cache")?;
    
    let mut eval = JSONEval::with_parsed_schema(cached, None, None)?;
    eval.evaluate(data, None)?;
    Ok(serde_json::to_string(&eval.get_evaluated_schema(false)).unwrap())
}

// One-time initialization
let cache = ParsedSchemaCache::new();
let parsed = ParsedSchema::parse(schema_json)?;
cache.insert("my-schema".to_string(), Arc::new(parsed));

// Many evaluations (reuses parsed schema)
for data_file in data_files {
    let result = evaluate_data(&cache, "my-schema", &data_file)?;
}
```

## Troubleshooting

### Q: Why is my cache growing indefinitely?
**A**: You're not removing old entries. Call `cache.remove()` or `cache.clear()` when schemas are no longer needed.

### Q: Can I share the cache between async tasks?
**A**: Yes! The cache is thread-safe. Wrap it in `Arc` if needed:
```rust
let cache = Arc::new(ParsedSchemaCache::new());
```

### Q: How do I know what's in the cache?
**A**: Use `cache.stats()` or `cache.keys()` to inspect cache contents.

### Q: Should I use the global cache or create my own?
**A**: 
- Global cache: Simple applications, prototypes, scripts
- Local cache: Libraries, applications with complex lifecycle management, testing

### Q: What happens if I insert with the same key twice?
**A**: The old value is replaced and returned from `insert()`.

## Changelog

### v0.0.10
- ✅ Initial release of `ParsedSchemaCache`
- ✅ Thread-safe Arc-based caching
- ✅ Global `PARSED_SCHEMA_CACHE` instance
- ✅ Comprehensive API with batch operations
- ✅ Full documentation and examples
