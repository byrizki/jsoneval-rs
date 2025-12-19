# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build
cargo build                          # Debug build
cargo build --release                # Release build (uses LTO and max optimizations)

# Run tests
cargo test                           # Run all tests
cargo test <test_name>               # Run specific test
cargo test --test <file>             # Run specific test file (e.g., cargo test --test array_tests)

# Linting
cargo fmt                            # Format code
cargo clippy                         # Run linter

# Examples
cargo run --example basic            # Run basic example with all scenarios
cargo run --example basic zcc        # Run with specific scenario filter
cargo run --example benchmark -- --parsed -i 100 zcc  # Performance benchmark

# CLI tool
cargo run --bin json-eval-cli -- schema.json -d data.json
```

## Feature Flags

- `parallel` - Enables multi-threaded evaluation using rayon (disabled for WASM)
- `wasm` - WebAssembly bindings via wasm-bindgen
- `ffi` - C FFI bindings for C#/.NET and other languages

## Architecture Overview

### Core Evaluation Pipeline

1. **Schema Parsing** (`parse_schema/`) → Extracts logic expressions and builds dependency graph
2. **Logic Compilation** (`rlogic/`) → Pre-compiles JSON Logic expressions into `CompiledLogic`
3. **Topological Sort** (`topo_sort/`) → Groups evaluations into dependency-ordered parallel batches
4. **Parallel Evaluation** (`lib.rs`) → Executes batches concurrently with caching
5. **Result Aggregation** → Cleans results and resolves `$ref` layout references

### Key Components

- **`JSONEval`** (`lib.rs`): Main orchestrator. Holds schema, engine, cache, and evaluation state. Entry point for `evaluate()` and `validate()`.

- **`RLogic` / `Evaluator`** (`rlogic/`): Custom JSON Logic engine. `CompiledLogic` stores pre-compiled expressions. Operators are split across modules: `arithmetic.rs`, `comparison.rs`, `logical.rs`, `string_ops.rs`, `date_ops.rs`, `math_ops.rs`, `array_ops.rs`, `array_lookup.rs`.

- **`EvalData`** (`eval_data.rs`): Proxy-like data wrapper for thread-safe mutations. All data access goes through this.

- **`EvalCache`** (`eval_cache.rs`): Content-based caching using `Arc<Value>` for zero-copy storage.

- **`ParsedSchema`** (`parsed_schema.rs`): Cached parsed representation of a schema. Can be shared via `Arc` for multi-instance reuse.

- **`ParsedSchemaCache`** (`parsed_schema_cache.rs`): Global cache for parsed schemas keyed by content hash.

- **Table Evaluator** (`table_evaluate.rs`): Specialized parallel processing for table/array data with `VALUEAT`, `INDEXAT` operators.

### Platform Bindings

- **`ffi/`** - C FFI for C#/.NET (P/Invoke)
- **`wasm/`** - WebAssembly bindings for web/JavaScript
- **`bindings/`** - Language-specific binding projects:
  - `csharp/` - .NET NuGet package
  - `web/` - npm package (@json-eval-rs/webcore, @json-eval-rs/bundler)
  - `react-native/` - React Native native module

### Building Bindings

```bash
./build-bindings.sh all           # Build all bindings
./build-bindings.sh csharp        # C# only
./build-bindings.sh web           # Web/WASM only
./build-bindings.sh react-native  # React Native only
./build-bindings.sh package       # Package for publishing
```

## Testing

Test files are in `tests/`. Each test file focuses on a specific area:
- `operator_tests.rs` - JSON Logic operators
- `array_tests.rs`, `string_tests.rs`, `date_tests.rs`, `math_tests.rs` - Type-specific operators
- `table_tests.rs` - Table lookup operators
- `json_eval_tests.rs` - Full JSONEval integration
- `selective_eval.rs`, `selective_eval_advanced.rs` - Selective evaluation feature
- `timezone_integration_tests.rs`, `date_offset_tests.rs` - Timezone offset functionality

Example scenarios live in `samples/` with schema, data, and expected output files.

## Key Features

### Selective Evaluation

Selective evaluation allows you to re-evaluate only specific fields in your schema, rather than reprocessing the entire schema. This is particularly useful for large schemas where only a subset of fields need to be updated.

**How it works:**
- Pass an optional `paths` parameter to `evaluate()` with a list of field paths to re-evaluate
- Only the specified fields and their dependencies will be recalculated
- Other fields retain their previously evaluated values
- The cache is selectively purged only for affected fields

**Usage:**

```rust
use json_eval_rs::JSONEval;
use serde_json::json;

let mut eval = JSONEval::new(&schema_str, None, Some(&data_str)).unwrap();

// Full evaluation
eval.evaluate(&data_str, None, None).unwrap();

// Update data and selectively re-evaluate only specific fields
let paths = vec!["computed1".to_string(), "nested.field.computed2".to_string()];
eval.evaluate(&updated_data_str, None, Some(&paths)).unwrap();
```

**Path formats supported:**
- Dotted notation: `"field.nested.property"`
- Explicit properties: `"field.properties.nested.properties.property"`
- Works with both `/properties/` schema paths and `$params` paths

**Benefits:**
- Improved performance for partial updates
- Reduced cache invalidation
- Efficient for large schemas with many computed fields
- Ideal for interactive forms where only one field changes at a time

**Test coverage:**
- `tests/selective_eval.rs` - Basic selective evaluation with nested paths
- `tests/selective_eval_advanced.rs` - Advanced scenarios with $params and explicit properties

### Timezone Offset Configuration

The library supports timezone offset configuration for date/time operations, allowing you to work with dates in different timezones without requiring external datetime libraries.

**How it works:**
- Configure timezone offset in minutes from UTC
- Affects all date operations: `TODAY`, `NOW`, `DATEFORMAT`, etc.
- Defaults to UTC (offset = 0) if not specified
- Can be changed dynamically during runtime

**Usage:**

```rust
use json_eval_rs::JSONEval;

let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

// Set timezone to UTC+7 (420 minutes)
eval.set_timezone_offset(Some(420));

// Evaluate with timezone applied
eval.evaluate(&data_str, None, None).unwrap();

// Reset to UTC
eval.set_timezone_offset(None);
```

**Common timezone offsets:**
- UTC: `None` or `Some(0)`
- UTC+7 (Bangkok, Jakarta): `Some(420)`
- UTC-5 (EST): `Some(-300)`
- UTC+9 (Tokyo): `Some(540)`
- UTC-8 (PST): `Some(-480)`

**Configuration via RLogicConfig:**

```rust
use json_eval_rs::{RLogicConfig, JSONEval};

// Create config with timezone
let config = RLogicConfig::default()
    .with_timezone_offset(420); // UTC+7

// Config is applied when creating the evaluator engine
```

**Benefits:**
- No external datetime dependencies
- Consistent date handling across platforms
- Runtime timezone switching
- Affects all date operations uniformly

**Affected operators:**
- `TODAY` - Returns current date at midnight in specified timezone
- `NOW` - Returns current timestamp in specified timezone
- `DATEFORMAT` - Formats dates using the timezone offset
- Date arithmetic operations

**Test coverage:**
- `tests/timezone_integration_tests.rs` - Integration tests for timezone offset with JSONEval
- `tests/date_offset_tests.rs` - Date operation tests with various timezone offsets
- `tests/date_tests.rs` - General date operator tests

## Key Patterns

- Schema paths use JSON Pointer format (e.g., `#/properties/fieldName/rules/required/value`)
- Logic expressions use JSON Logic syntax with `{"var": "path.to.field"}` for data access
- Dependencies are tracked transitively for efficient re-evaluation
- Results are cleaned for floating-point noise (near-zero values → 0)
- Selective evaluation uses dotted paths (e.g., `"field.nested.property"`)
- Timezone offset is specified in minutes from UTC (positive for east, negative for west)
