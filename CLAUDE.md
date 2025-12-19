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
- `selective_eval.rs` - Selective evaluation feature

Example scenarios live in `samples/` with schema, data, and expected output files.

## Key Patterns

- Schema paths use JSON Pointer format (e.g., `#/properties/fieldName/rules/required/value`)
- Logic expressions use JSON Logic syntax with `{"var": "path.to.field"}` for data access
- Dependencies are tracked transitively for efficient re-evaluation
- Results are cleaned for floating-point noise (near-zero values → 0)
