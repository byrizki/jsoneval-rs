# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Overview

This is `json-eval-rs`, a high-performance JSON Logic evaluation library written in Rust with language bindings for C#, JavaScript/TypeScript (Web), and React Native. It features a custom-built JSON Logic compiler and evaluator with advanced caching, parallel evaluation, and schema validation capabilities.

## Core Development Commands

### Building and Testing

```bash
# Build the main Rust library
cargo build --release

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test modules
cargo test basic_tests
cargo test table_tests
cargo test edge_case_tests

# Run a single test
cargo test test_name -- --exact

# Build CLI tool
cargo build --bin json-eval-cli --release

# Run CLI with examples (uses scenarios from samples/ directory)
cargo run --bin json-eval-cli
cargo run --bin json-eval-cli zcc    # Run specific scenario
cargo run --bin json-eval-cli -- -i 100 zcc  # Benchmark with iterations
```

### Feature-Specific Builds

```bash
# Build with FFI support (for C# bindings)
cargo build --release --features ffi

# Build with WASM support (for Web bindings)
cargo build --release --features wasm --target wasm32-unknown-unknown

# Build for different targets
cargo build --release --target x86_64-pc-windows-msvc --features ffi
cargo build --release --target aarch64-apple-darwin --features ffi
```

### Language Bindings

```bash
# Build all bindings
./build-bindings.sh all

# Build specific bindings
./build-bindings.sh csharp
./build-bindings.sh web
./build-bindings.sh react-native

# Package for publishing
./build-bindings.sh package
```

### WASM/Web Development

```bash
# Install wasm-pack (if not installed)
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM packages
wasm-pack build --target web --out-dir bindings/web/pkg --features wasm
wasm-pack build --target nodejs --out-dir bindings/web/pkg-node --features wasm
wasm-pack build --target bundler --out-dir bindings/web/pkg-bundler --features wasm
```

## Architecture & Code Structure

### Core Engine (`src/`)

- **`lib.rs`** - Main `JSONEval` struct that orchestrates the entire evaluation pipeline
- **`rlogic/`** - Custom JSON Logic compiler and evaluator (replaces external dependencies)
  - Pre-compiles logic expressions for optimal performance  
  - Thread-safe parallel evaluation with caching
  - Supports both user data and internal context variables
- **`eval_data.rs`** - Proxy-like data wrapper that tracks mutations and enables safe concurrent access
- **`eval_cache.rs`** - Content-based caching system with zero-copy storage using Arc
- **`table_evaluate.rs`** - Specialized table evaluation with row-level parallel processing
- **`parse_schema.rs`** - Schema parser that extracts evaluations, dependencies, and metadata
- **`topo_sort.rs`** - Topological sorting for dependency-aware evaluation order

### Evaluation Pipeline

1. **Schema Parsing** - Extract evaluations, build dependency graph, pre-compile logic
2. **Topological Sort** - Group evaluations into parallel-executable batches
3. **Parallel Evaluation** - Execute batches with dependency order, leveraging caching
4. **Result Aggregation** - Clean floating-point noise, update schema and data structures
5. **Layout Resolution** - Resolve `$ref` references in UI layouts

### Key Features

- **Zero-Copy Caching** - Uses Arc<Value> for cached results to avoid deep cloning
- **Parallel Table Processing** - Individual table rows evaluated concurrently  
- **SIMD JSON Parsing** - Fast JSON parsing using `simd-json` for data, `serde_json` for schemas
- **Dependency Tracking** - Automatic detection of field dependencies for selective re-evaluation
- **Multi-Target Bindings** - FFI for C#, WASM for Web, JNI for React Native

### Language Bindings (`bindings/`)

Each binding follows a consistent pattern:
- **C#** (`bindings/csharp/`) - P/Invoke FFI wrapper with native memory management
- **Web** (`bindings/web/`) - WASM with wasm-bindgen for JavaScript interop
- **React Native** (`bindings/react-native/`) - Native modules using JSI

## Development Patterns

### Testing Strategy

Tests are organized by functionality:
- `basic_tests.rs` - Core evaluation scenarios
- `table_tests.rs` - Table-specific evaluation
- `edge_case_tests.rs` - Edge cases and error handling
- `*_operator_*` - Operator-specific test suites

### Performance Considerations

- Use `rayon` for parallel processing (automatically disabled for WASM)
- Cache evaluation results based on dependency content hashes
- Pre-compile JSON Logic expressions during schema parsing
- Minimize allocations in hot paths using `smallvec` and zero-copy techniques

### Error Handling

- Library functions return `Result<T, String>` for detailed error messages
- FFI bindings convert errors to appropriate language conventions
- Validation errors provide field-level granular information

### Memory Management

- Rust core owns all data structures
- FFI bindings use explicit cleanup (`free()` methods)
- WASM bindings automatically manage memory via wasm-bindgen
- Caching uses reference counting (Arc) to prevent deep copies

## CLI Usage

The `json-eval-cli` binary (`src/main.rs`) provides a testing and benchmarking tool:

```bash
# Run scenarios from samples/ directory
cargo run --bin json-eval-cli

# Run specific scenarios (zcc, zip, zlw are available)
cargo run --bin json-eval-cli zcc
cargo run --bin json-eval-cli zip
cargo run --bin json-eval-cli zlw

# Benchmark with multiple iterations
cargo run --bin json-eval-cli -- -i 1000 zcc

# Show help
cargo run --bin json-eval-cli -- --help
```

### Sample File Structure

Each scenario requires:
- `<name>.json` - Schema definition
- `<name>-data.json` - Input data  
- `<name>-evaluated-compare.json` - Expected output (optional)

## CI/CD & Publishing

### GitHub Actions

- **`ci.yml`** - Run tests, linting, format checks
- **`build-bindings.yml`** - Build all language bindings for multiple platforms
- **`publish.yml`** - Publish packages to crates.io, NuGet, and npm

### Publishing Targets

- **Rust** - `json-eval-rs` on crates.io
- **C#** - `JsonEvalRs` on NuGet
- **Web** - `@json-eval-rs/web` on npm  
- **React Native** - `@json-eval-rs/react-native` on npm

### Platform Support

- **Linux** - x86_64 native libraries
- **Windows** - x86_64 native libraries  
- **macOS** - x86_64 and ARM64 native libraries
- **WASM** - All modern browsers and Node.js
- **Mobile** - iOS (ARM64/x86_64) and Android (ARM64)

## Configuration

### Cargo Features

- `default` - Core functionality only
- `ffi` - Enable FFI bindings for C# 
- `wasm` - Enable WASM bindings for Web/Node.js

### Build Configuration

- Release builds use LTO, single codegen unit, and symbol stripping for maximum performance
- WASM builds configured in `.cargo/config.toml` with appropriate flags
- Platform-specific rustflags for getrandom WASM backend

### Environment Setup

For full development environment:
1. Rust toolchain (latest stable)
2. .NET SDK 6.0+ (for C# bindings)  
3. Node.js 18+ (for Web/React Native bindings)
4. wasm-pack (for WASM builds)
5. Platform-specific tools (Android NDK, Xcode for mobile)

The codebase is designed for high-performance evaluation scenarios with complex dependency graphs and supports both single-threaded (WASM) and multi-threaded (native) execution environments.