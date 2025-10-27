# Timing Instrumentation Guide

## Overview

The library includes built-in timing instrumentation to identify performance bottlenecks. The instrumentation has **zero overhead** when disabled and provides detailed timing breakdowns when enabled.

## Quick Start

### Enable Timing

Set the `JSONEVAL_TIMING` environment variable:

```bash
# Linux/macOS
JSONEVAL_TIMING=1 cargo run --release --example basic -- zcc

# Windows PowerShell
$env:JSONEVAL_TIMING=1; cargo run --release --example basic -- zcc
```

### Example Output

```
📊 Timing Summary (JSONEVAL_TIMING enabled)
============================================================
parse schema JSON                       84.96ms
parse context JSON                       3.34µs
parse data JSON                         28.61µs
create instance struct                  83.82ms
parse_schema                           315.57ms
JSONEval::new() [total]                538.59ms
  parse data                            74.26µs
  parse context                          0.76µs
  replace_data_and_context              11.09µs
  purge_cache                           76.16µs
    process batches                    327.53ms
      evaluate_options_templates         5.47µs
      evaluate rules+others            577.61µs
    evaluate_others()                   583.42µs
  evaluate_internal() [total]          328.16ms
evaluate() [total]                     328.34ms
    resolve_layout_elements              0.93ms
    propagate_parent_conditions          0.81ms
  resolve_layout_internal()              1.74ms
get_evaluated_schema()                  11.04ms
============================================================
TOTAL                                    2.42s
```

## API Functions

### Clear Timing Data

```rust
use json_eval_rs;

// Clear timing data between scenarios
json_eval_rs::clear_timing_data();
```

### Print Timing Summary

```rust
use json_eval_rs;

// Print detailed timing breakdown
json_eval_rs::print_timing_summary();
```

### Complete Example

```rust
use json_eval_rs::JSONEval;

fn main() {
    // Clear any previous timing data
    json_eval_rs::clear_timing_data();
    
    // Your code here
    let mut eval = JSONEval::new(schema, None, Some(data))?;
    eval.evaluate(data, None)?;
    let result = eval.get_evaluated_schema(false);
    
    // Print timing summary (only shows output if JSONEVAL_TIMING=1)
    json_eval_rs::print_timing_summary();
}
```

## Instrumented Methods

The following methods are instrumented with timing:

### Initialization
- `JSONEval::new()` - Total initialization time
  - `parse schema JSON` - JSON parsing
  - `parse context JSON` - Context parsing  
  - `parse data JSON` - Data parsing
  - `create instance struct` - Instance creation
  - `parse_schema` - Schema analysis and compilation

### Evaluation
- `evaluate()` - Total evaluation time
  - `parse data` - Data parsing
  - `parse context` - Context parsing
  - `replace_data_and_context` - Data replacement
  - `purge_cache` - Cache invalidation
  - `evaluate_internal()` - Core evaluation logic
    - `process batches` - Batch processing
    - `evaluate_others()` - Other evaluations
      - `evaluate_options_templates` - Template evaluation
      - `evaluate rules+others` - Rules evaluation

### Finalization
- `get_evaluated_schema()` - Schema retrieval
  - `resolve_layout_internal()` - Layout resolution
    - `resolve_layout_elements` - Element resolution
    - `propagate_parent_conditions` - Condition propagation

## Performance Characteristics

### Zero Overhead When Disabled

When `JSONEVAL_TIMING` is not set:
- All timing checks compile to a single boolean check (`is_timing_enabled()`)
- The optimizer inlines and eliminates dead code
- **No measurable performance impact**

### Thread-Local Storage

Timing data is stored in thread-local storage:
- Each thread has its own timing data
- No synchronization overhead
- Safe for multi-threaded applications

### Output Format

Timing output is written to `stderr` using `eprintln!`:
- Doesn't interfere with normal output (`stdout`)
- Can be redirected separately: `2>timing.log`
- Indentation shows call hierarchy

## Interpreting Results

### Understanding Indentation

```
JSONEval::new() [total]      500ms    ← Top-level method
  parse_schema               300ms    ← Sub-operation (2 spaces)
    process batches          200ms    ← Nested operation (4 spaces)
```

### Identifying Bottlenecks

1. Look for methods with high absolute time (>100ms)
2. Look for methods with high relative time (>20% of total)
3. Check nested operations for optimization opportunities

### Example Analysis

```
parse_schema                315ms  ← PRIMARY BOTTLENECK (35%)
process batches             327ms  ← SECONDARY BOTTLENECK (37%)
resolve_layout                2ms  ← Not a concern (<1%)
```

## Integration with External Tools

### Cargo Flamegraph

Combine timing instrumentation with flame graphs:

```bash
# Install flamegraph
cargo install flamegraph

# Generate flame graph with timing
JSONEVAL_TIMING=1 cargo flamegraph --example basic -- zcc
```

### Criterion Benchmarks

Use timing data to validate benchmark improvements:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use json_eval_rs::JSONEval;

fn bench_evaluate(c: &mut Criterion) {
    json_eval_rs::clear_timing_data();
    
    c.bench_function("evaluate zcc", |b| {
        b.iter(|| {
            let mut eval = JSONEval::new(schema, None, Some(data)).unwrap();
            eval.evaluate(data, None).unwrap();
            black_box(eval.get_evaluated_schema(false))
        });
    });
    
    json_eval_rs::print_timing_summary();
}
```

## Troubleshooting

### No Timing Output

**Problem**: Running with `JSONEVAL_TIMING=1` but no output appears

**Solutions**:
1. Ensure you're calling `print_timing_summary()`
2. Check that timing code is being executed
3. Verify stderr isn't being redirected

### Unexpected Timing Values

**Problem**: Timing values seem too high or too low

**Causes**:
1. Debug build vs release build (use `--release`)
2. Cold start vs warm start (first run is slower)
3. Background processes affecting CPU
4. File system caching effects

**Best Practice**: Run multiple times and average:
```bash
for i in {1..5}; do JSONEVAL_TIMING=1 cargo run --release --example basic -- zcc; done
```

### Memory Usage

**Problem**: Concerned about memory overhead

**Facts**:
- Timing data stores `(String, Duration)` tuples
- Typical scenario: <100 entries = ~10KB
- Automatically cleared with `clear_timing_data()`
- Thread-local = isolated per thread

## Contributing

To add timing to new methods:

```rust
fn my_method(&self) -> Result<Value, String> {
    time_block!("my_method", {
        // Your code here
        let result = expensive_operation();
        result
    })
}
```

Naming conventions:
- Top-level methods: `method_name()`
- Sub-operations: `  sub_operation` (2 spaces)
- Nested operations: `    nested_op` (4 spaces)

## See Also

- [PERFORMANCE_ANALYSIS.md](./PERFORMANCE_ANALYSIS.md) - Detailed performance analysis of ZCC scenario
- [Cargo Book - Profiling](https://doc.rust-lang.org/cargo/guide/build-cache.html)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
