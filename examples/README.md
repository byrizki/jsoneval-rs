# JSON Evaluation Examples

This directory contains example applications demonstrating different use cases of the `json-eval-rs` library.

## Available Examples

### 1. Basic Examples

Three basic examples demonstrating different schema input methods:

#### a) Basic (`basic.rs`) - JSON Schema String
Evaluates JSON schemas using `JSONEval::new()` with JSON string input.

**Usage:**
```bash
# Run all JSON schema scenarios
cargo run --example basic

# Run specific scenario
cargo run --example basic zcc

# Enable comparison with expected results
cargo run --example basic --compare
```

#### b) Basic MessagePack (`basic_msgpack.rs`) - MessagePack Schema
Evaluates MessagePack schemas using `JSONEval::new_from_msgpack()`.

**Usage:**
```bash
# Run all MessagePack schema scenarios
cargo run --example basic_msgpack

# Run with comparison
cargo run --example basic_msgpack --compare zccbin
```

#### c) Basic ParsedSchema (`basic_parsed.rs`) - ParsedSchema with Arc
Demonstrates efficient schema caching with `Arc<ParsedSchema>`.

**Usage:**
```bash
# Run scenarios with ParsedSchema
cargo run --example basic_parsed

# With comparison
cargo run --example basic_parsed --compare
```

**Common Options:**
- `-h, --help` - Show help message
- `--compare` - Enable comparison with expected results
- `[FILTER]` - Filter scenarios by name

---

### 2. Benchmark Example (`benchmark.rs`)

Advanced benchmarking tool with support for iterations, concurrent execution, and ParsedSchema caching.

**Features:**
- Multiple evaluation iterations for performance testing
- ParsedSchema caching (parse once, reuse many times)
- Concurrent evaluation testing with thread pools
- Detailed performance statistics
- Cache hit rate reporting
- CPU feature detection

**Usage:**
```bash
# Simple benchmark with 100 iterations
cargo run --example benchmark -- -i 100 zcc

# Use ParsedSchema for efficient caching (recommended for multiple iterations)
cargo run --example benchmark -- --parsed -i 100 zcc

# Test concurrent evaluations (4 threads, 10 iterations each)
cargo run --example benchmark -- --parsed --concurrent 4 -i 10

# Run with comparison enabled
cargo run --example benchmark -- --parsed -i 50 --compare zcc

# Show CPU features
cargo run --example benchmark -- --cpu-info

# Release mode for accurate performance testing
cargo run --release --example benchmark -- --parsed -i 100 zcc
```

**Options:**
- `-h, --help` - Show help message
- `-i, --iterations <COUNT>` - Number of iterations (default: 1)
- `--parsed` - Use ParsedSchema for caching (parse once, reuse)
- `--concurrent <COUNT>` - Test concurrent evaluations with N threads
- `--compare` - Enable comparison with expected results
- `--cpu-info` - Show CPU feature information
- `[FILTER]` - Filter scenarios by name

**Performance Tips:**
1. Always use `--parsed` flag when running multiple iterations
2. Use `--release` mode for accurate benchmarks
3. Combine `--parsed` with `--concurrent` to test thread safety

---

## Scenario Discovery

Both examples automatically discover test scenarios from the `samples/` directory.

**Required files per scenario:**
- `<name>.json` or `<name>.bform` - Schema file (JSON or MessagePack)
- `<name>-data.json` - Input data file
- `<name>-evaluated-compare.json` - Optional comparison file (for `--compare` flag)

**Example structure:**
```
samples/
├── zcc.json                      # Schema (JSON format)
├── zcc-data.json                 # Input data
├── zcc-evaluated-compare.json    # Expected output (optional)
├── zccbin.bform                  # Schema (MessagePack format)
├── zccbin-data.json              # Input data
└── zccbin-evaluated-compare.json # Expected output (optional)
```

**Note:** MessagePack schemas (`.bform`) are prioritized over JSON schemas (`.json`) when both exist.

---

## Output Files

Both examples generate these output files in `samples/`:

- `<name>-evaluated-schema.json` - The evaluated schema with all expressions computed
- `<name>-parsed-schema.json` - Metadata including dependencies and sorted evaluations

---

## Common Module

The `common/mod.rs` module provides shared functionality:

- `discover_scenarios()` - Automatic scenario discovery
- `compare_with_expected()` - Result comparison logic
- `pretty_json()` - Pretty JSON formatting
- `print_cpu_info()` - CPU feature detection

---

## Examples Comparison

| Feature | Basic (JSON) | Basic (MsgPack) | Basic (Parsed) | Benchmark |
|---------|-------------|-----------------|----------------|-----------|
| Simple execution | ✅ | ✅ | ✅ | ✅ |
| JSON schema | ✅ | ❌ | ✅ | ✅ |
| MessagePack schema | ❌ | ✅ | ✅ | ✅ |
| ParsedSchema caching | ❌ | ❌ | ✅ | ✅ Optional |
| Multiple iterations | ❌ | ❌ | ❌ | ✅ |
| Concurrent testing | ❌ | ❌ | ❌ | ✅ |
| Performance stats | ❌ | ❌ | ✅ Basic | ✅ Detailed |
| Cache statistics | ❌ | ❌ | ❌ | ✅ |
| CPU info | ❌ | ❌ | ❌ | ✅ |
| Comparison | ✅ Optional | ✅ Optional | ✅ Optional | ✅ Optional |
| Auto-discovery | ✅ | ✅ | ✅ | ✅ |

---

## Best Practices

1. **For quick JSON schema testing:** Use `basic` example
   ```bash
   cargo run --example basic zcc
   ```

2. **For MessagePack schemas:** Use `basic_msgpack` example
   ```bash
   cargo run --example basic_msgpack zccbin
   ```

3. **To understand ParsedSchema:** Use `basic_parsed` example
   ```bash
   cargo run --example basic_parsed
   ```

4. **For performance testing:** Use `benchmark` with `--parsed` and `--release`
   ```bash
   cargo run --release --example benchmark -- --parsed -i 100 zcc
   ```

5. **For concurrent testing:** Use `benchmark` with `--concurrent`
   ```bash
   cargo run --example benchmark -- --parsed --concurrent 4 -i 10
   ```

6. **For validation:** Use `--compare` flag
   ```bash
   cargo run --example basic -- --compare
   ```

---

## Creating Custom Examples

You can create your own examples by:

1. Creating a new `.rs` file in `examples/`
2. Adding `mod common;` at the top
3. Using shared utilities from `common` module
4. Adding example definition to `Cargo.toml`:

```toml
[[example]]
name = "my_example"
path = "examples/my_example.rs"
```

Then run with:
```bash
cargo run --example my_example
```

---

## See Also

- Main documentation: `../README.md`
- Library source: `../src/lib.rs`
- Test files: `../tests/`
