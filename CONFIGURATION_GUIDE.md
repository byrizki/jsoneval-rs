# RLogic Configuration Guide

## Overview

RLogic provides a flexible configuration system that allows you to optimize the engine for your specific use case. You can control caching, data tracking, NaN handling, and recursion limits.

## Quick Start

```rust
use json_eval_rs::{RLogic, RLogicConfig};

// Use default configuration
let engine = RLogic::new();

// Use a preset configuration
let engine = RLogic::with_config(RLogicConfig::performance());

// Build a custom configuration
let config = RLogicConfig::new()
    .with_cache(true)
    .with_tracking(false)
    .with_safe_nan(true)
    .with_recursion_limit(200);
let engine = RLogic::with_config(config);
```

## Configuration Options

### 1. `enable_cache` (default: `true`)

Controls whether evaluation results are cached.

**When enabled:**
- Results are cached based on logic ID, data instance, and data version
- Repeated evaluations with same data are 20-60x faster
- Automatic cache invalidation when data mutates
- Uses ~56 bytes per cache entry + result size

**When disabled:**
- No caching overhead
- Every evaluation recomputes from scratch
- Better for streaming data or unique evaluations

```rust
// Enable caching (default)
let config = RLogicConfig::new().with_cache(true);

// Disable caching
let config = RLogicConfig::new().with_cache(false);
```

### 2. `enable_tracking` (default: `true`)

Controls whether data mutations are tracked via `TrackedData` wrapper.

**When enabled:**
- Data changes trigger cache invalidation
- Requires using `TrackedData` wrapper
- Adds ~40 bytes overhead per data instance
- Tracks which fields have been modified

**When disabled:**
- No mutation tracking
- Can use raw `Value` directly
- Lower memory overhead
- Faster for read-only data

```rust
// Enable tracking (default)
let config = RLogicConfig::new().with_tracking(true);

// Disable tracking
let config = RLogicConfig::new().with_tracking(false);
```

### 3. `safe_nan_handling` (default: `false`)

Controls how NaN and Infinity values are handled in math operations.

**When enabled:**
- NaN and Infinity return `0` instead of `null`
- Safer for untrusted data
- Prevents unexpected null propagation

**When disabled:**
- NaN and Infinity return `null`
- Standard JSON Logic behavior
- Slightly faster (no extra checks)

```rust
// Disable safe NaN (default)
let config = RLogicConfig::new().with_safe_nan(false);

// Enable safe NaN
let config = RLogicConfig::new().with_safe_nan(true);

// Example behavior
let mut engine_unsafe = RLogic::new();
let mut engine_safe = RLogic::with_config(
    RLogicConfig::new().with_safe_nan(true)
);

// sqrt(-1) = NaN
// unsafe: returns null
// safe: returns 0
```

### 4. `recursion_limit` (default: `100`)

Maximum depth for nested logic evaluation.

```rust
let config = RLogicConfig::new().with_recursion_limit(200);
```

Prevents stack overflow from deeply nested logic. Increase if you have legitimately deep logic trees.

## Preset Configurations

### Default Configuration

Balanced for most use cases.

```rust
let engine = RLogic::new();
// Equivalent to:
let config = RLogicConfig::default();
// enable_cache: true
// enable_tracking: true
// safe_nan_handling: false
// recursion_limit: 100
```

**Use when:**
- General purpose evaluation
- Data may be mutated
- Need cache invalidation
- Standard JSON Logic behavior

### Performance Configuration

Optimized for maximum speed.

```rust
let engine = RLogic::with_config(RLogicConfig::performance());
// enable_cache: true
// enable_tracking: false
// safe_nan_handling: false
// recursion_limit: 100
```

**Use when:**
- Data is read-only (no mutations)
- Maximum performance is critical
- Don't need mutation tracking
- Processing large volumes

**Performance gain:** ~10-15% faster than default

### Safe Configuration

All safety features enabled.

```rust
let engine = RLogic::with_config(RLogicConfig::safe());
// enable_cache: true
// enable_tracking: true
// safe_nan_handling: true
// recursion_limit: 100
```

**Use when:**
- Working with untrusted data
- Need NaN safety
- Debugging or development
- Maximum safety over performance

**Performance cost:** ~5% slower than default

### Minimal Configuration

Bare metal, no overhead.

```rust
let engine = RLogic::with_config(RLogicConfig::minimal());
// enable_cache: false
// enable_tracking: false
// safe_nan_handling: false
// recursion_limit: 100
```

**Use when:**
- Streaming data (new data every time)
- Memory constrained
- Each evaluation uses unique data
- Cache provides no benefit

**Performance:** Matches datalogic-rs (~0.60µs per eval)

## Usage Examples

### Example 1: High-throughput API

```rust
// Use performance config for read-only data
let config = RLogicConfig::performance();
let mut engine = RLogic::with_config(config);

let logic_id = engine.compile(&business_rules).unwrap();

// Process requests
for request in requests {
    let data = TrackedData::new(request.data);
    let result = engine.evaluate(&logic_id, &data).unwrap();
    // Cache hits make this very fast
}
```

### Example 2: Interactive Form Validation

```rust
// Use default config for reactive updates
let mut engine = RLogic::new();
let logic_id = engine.compile(&validation_rules).unwrap();

let mut form_data = TrackedData::new(initial_data);

// User changes a field
form_data.set("email", json!("new@example.com"));

// Cache automatically invalidated, re-evaluates
let result = engine.evaluate(&logic_id, &form_data).unwrap();
```

### Example 3: Batch Processing

```rust
// Use minimal config for unique data
let config = RLogicConfig::minimal();
let mut engine = RLogic::with_config(config);

let logic_id = engine.compile(&transform_logic).unwrap();

// Process stream of unique records
for record in records {
    let result = engine.evaluate_raw(&logic_id, &record).unwrap();
    // No cache overhead, direct evaluation
}
```

### Example 4: Untrusted Data

```rust
// Use safe config for user-provided data
let config = RLogicConfig::safe();
let mut engine = RLogic::with_config(config);

let logic_id = engine.compile(&user_formula).unwrap();
let data = TrackedData::new(user_data);

// Safe from NaN propagation
let result = engine.evaluate(&logic_id, &data).unwrap();
```

## Performance Comparison

| Config | Compilation | Cached Eval | Uncached Eval | Memory |
|--------|-------------|-------------|---------------|--------|
| Default | ~0.37µs | ~0.03µs | ~0.70µs | Medium |
| Performance | ~0.37µs | ~0.03µs | ~0.60µs | Low |
| Safe | ~0.37µs | ~0.03µs | ~0.75µs | Medium |
| Minimal | ~0.37µs | N/A | ~0.60µs | Minimal |

## Migration from RLogicBuilder

The old `RLogicBuilder` is deprecated. Migrate to `RLogicConfig`:

```rust
// Old way (deprecated)
let engine = RLogicBuilder::new()
    .with_recursion_limit(200)
    .build();

// New way
let config = RLogicConfig::new()
    .with_recursion_limit(200);
let engine = RLogic::with_config(config);
```

## Best Practices

1. **Choose the right preset** - Start with a preset that matches your use case
2. **Customize as needed** - Use builder pattern to fine-tune
3. **Profile your workload** - Measure before optimizing
4. **Consider memory vs speed** - Caching trades memory for speed
5. **Use safe mode for untrusted data** - Better safe than sorry

## Troubleshooting

### Cache not working?

Check if `enable_cache` is true and you're using the same data instance:

```rust
let config = engine.config();
println!("Cache enabled: {}", config.enable_cache);

let stats = engine.cache_stats();
println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
```

### NaN returning null?

Enable safe NaN handling:

```rust
let config = RLogicConfig::new().with_safe_nan(true);
```

### Recursion limit errors?

Increase the limit:

```rust
let config = RLogicConfig::new().with_recursion_limit(500);
```

## Summary

RLogic's configuration system gives you fine-grained control over performance, safety, and memory usage. Choose the preset that matches your use case, or build a custom configuration for optimal results.
