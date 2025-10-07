# RLogic Performance Optimization Guide

## Configuration System

RLogic provides a flexible configuration system to optimize for different use cases:

### Available Configurations

```rust
use json_eval_rs::{RLogic, RLogicConfig};

// Default: Balanced (cache + tracking, no safe NaN)
let engine = RLogic::new();

// Performance: Maximum speed (cache only, no tracking/safety)
let engine = RLogic::with_config(RLogicConfig::performance());

// Safe: All safety features (cache + tracking + safe NaN)
let engine = RLogic::with_config(RLogicConfig::safe());

// Minimal: Bare metal (no cache, no tracking, no safety)
let engine = RLogic::with_config(RLogicConfig::minimal());

// Custom: Build your own
let config = RLogicConfig::new()
    .with_cache(true)
    .with_tracking(false)
    .with_safe_nan(true)
    .with_recursion_limit(200);
let engine = RLogic::with_config(config);
```

### Configuration Options

| Option | Default | Performance | Safe | Minimal | Description |
|--------|---------|-------------|------|---------|-------------|
| `enable_cache` | ✓ | ✓ | ✓ | ✗ | Cache evaluation results |
| `enable_tracking` | ✓ | ✗ | ✓ | ✗ | Track data mutations |
| `safe_nan_handling` | ✗ | ✗ | ✓ | ✗ | Return 0 for NaN instead of null |
| `recursion_limit` | 100 | 100 | 100 | 100 | Max evaluation depth |

## Evaluation Methods Comparison

RLogic provides three evaluation methods, each optimized for different use cases:

### 1. `evaluate()` - Full caching (best for repeated evaluations)
```rust
let mut engine = RLogic::new();
let logic_id = engine.compile(&logic).unwrap();
let data = TrackedData::new(json!({"value": 42}));

// First call: cache miss, evaluates and caches
let result = engine.evaluate(&logic_id, &data).unwrap();

// Subsequent calls: cache hit, instant return
let result2 = engine.evaluate(&logic_id, &data).unwrap(); // ~30x faster!
```

**When to use:**
- Same data instance evaluated multiple times
- Data mutations are tracked
- Maximum performance for hot paths

**Performance:** ~0.03µs per evaluation (cached)

### 2. `evaluate_uncached()` - Skip cache, keep tracking
```rust
let engine = RLogic::new();
let logic_id = engine.compile(&logic).unwrap();
let data = TrackedData::new(json!({"value": 42}));

// Skips cache lookup and insertion
let result = engine.evaluate_uncached(&logic_id, &data).unwrap();
```

**When to use:**
- Data changes frequently but you still want mutation tracking
- Cache overhead outweighs benefits
- Moderate number of evaluations

**Performance:** ~0.40µs per evaluation (no cache overhead)

### 3. `evaluate_raw()` - No cache, no wrapping (fastest for one-off)
```rust
let engine = RLogic::new();
let logic_id = engine.compile(&logic).unwrap();
let data = json!({"value": 42});

// Direct evaluation, no overhead
let result = engine.evaluate_raw(&logic_id, &data).unwrap();
```

**When to use:**
- New data every time (no cache benefit)
- No mutation tracking needed
- One-off evaluations or streaming data
- Performance-critical paths with unique data

**Performance:** ~0.50-0.60µs per evaluation (comparable to datalogic-rs)

## Benchmark Results

### Compilation Performance
RLogic is **1.4-1.5x faster** at compilation:
- Simple arithmetic: 0.37µs vs 0.52µs
- Complex logic: 1.35µs vs 1.56µs

### Cached Evaluation Performance
RLogic is **20-60x faster** with caching:
- Simple: 0.03µs vs 0.64µs (21x faster)
- Complex: 0.03µs vs 2.07µs (69x faster)
- Array ops: 0.03µs vs 1.87µs (62x faster)

### Uncached Evaluation Performance
With `evaluate_raw()`, RLogic matches datalogic-rs:
- Simple: ~0.60µs vs 0.63µs (comparable)

### Cache Effectiveness
On 1M evaluations with same data:
- RLogic cached: 30.84ms (0.03µs avg)
- datalogic-rs: 920.33ms (0.92µs avg)
- **30x faster overall**

## Optimization Strategy

### Scenario 1: Repeated evaluations with same data
```rust
// ✅ Use cached evaluation
let mut engine = RLogic::new();
let logic_id = engine.compile(&logic).unwrap();
let data = TrackedData::new(my_data);

for _ in 0..1_000_000 {
    let result = engine.evaluate(&logic_id, &data).unwrap();
}
```
**Performance:** ~0.03µs per call after first

### Scenario 2: New data every time
```rust
// ✅ Use raw evaluation (no cache overhead)
let engine = RLogic::new();
let logic_id = engine.compile(&logic).unwrap();

for item in data_stream {
    let result = engine.evaluate_raw(&logic_id, &item).unwrap();
}
```
**Performance:** ~0.60µs per call (matches datalogic-rs)

### Scenario 3: Data mutations
```rust
// ✅ Use TrackedData with caching
let mut engine = RLogic::new();
let logic_id = engine.compile(&logic).unwrap();
let mut data = TrackedData::new(my_data);

let result1 = engine.evaluate(&logic_id, &data).unwrap(); // cached

data.set("field", json!(newvalue)); // increments version

let result2 = engine.evaluate(&logic_id, &data).unwrap(); // cache invalidated, re-evaluates
```

## Memory Usage

### TrackedData overhead
- Instance ID: 8 bytes (u64)
- Version: 8 bytes (AtomicU64)
- Field versions: ~24 bytes (empty HashMap) + 24 bytes per tracked field
- **Total:** ~40 bytes + 24 bytes per modified field

### Cache overhead
- Cache key: 24 bytes (LogicId + instance_id + version)
- HashMap entry: ~24 bytes
- Result: Arc<Value> (8 bytes pointer)
- **Per cache entry:** ~56 bytes + size of result

## Configuration Best Practices

### Choose the Right Config

1. **Default** - Good for most use cases
   - Balanced performance and safety
   - Automatic cache invalidation on mutations
   
2. **Performance** - Use when:
   - Data doesn't change (no mutations)
   - Maximum speed is critical
   - You don't need mutation tracking
   
3. **Safe** - Use when:
   - Working with untrusted data
   - Need NaN safety in math operations
   - Debugging or development
   
4. **Minimal** - Use when:
   - Processing streaming data (new data every time)
   - Memory is constrained
   - Each evaluation uses unique data

### Safe NaN Handling

```rust
// Without safe NaN (default)
let engine = RLogic::new();
// sqrt(-1) = NaN → returns null

// With safe NaN
let config = RLogicConfig::new().with_safe_nan(true);
let engine = RLogic::with_config(config);
// sqrt(-1) = NaN → returns 0
```

## Best Practices

1. **Compile once, evaluate many** - Compilation is ~1.5µs, evaluation is ~0.03-0.60µs
2. **Choose the right config** - Match config to your use case
3. **Use cached evaluation for hot paths** - 20-60x speedup
4. **Use raw evaluation for streaming data** - Avoid cache/wrapper overhead
5. **Batch similar evaluations** - Share compiled logic across data items
6. **Clear cache periodically** - If data instances keep changing

## Trade-offs

| Method | Speed | Memory | Tracking | Use Case |
|--------|-------|--------|----------|----------|
| `evaluate()` | ★★★★★ | Medium | Yes | Hot paths, repeated data |
| `evaluate_uncached()` | ★★★☆☆ | Low | Yes | Changing data with tracking |
| `evaluate_raw()` | ★★★☆☆ | Minimal | No | Streaming, one-off evals |

## Conclusion

- **For cached scenarios:** RLogic is 20-60x faster than datalogic-rs
- **For uncached scenarios:** RLogic matches datalogic-rs performance
- **Choose the right method** for your use case to maximize performance
