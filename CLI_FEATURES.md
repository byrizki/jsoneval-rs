# JSON Evaluation CLI Features

## Overview

The `json-eval-cli` tool provides a powerful command-line interface for evaluating JSON schemas with support for:
- JSON and MessagePack schema formats
- JSON and MessagePack data formats
- ParsedSchema caching and inspection
- Performance benchmarking
- Result comparison and validation

## Input File Support

### Schema Files
- **JSON** (`.json`) - Standard JSON schema files
- **MessagePack** (`.bform`) - Binary compiled schema files

### Data Files
- **JSON** (`.json`) - Standard JSON data files
- **MessagePack** (`.bform`) - Binary data files (automatically converted to JSON for evaluation)

Both schema and data files support both formats independently:
```bash
# JSON schema + JSON data
./json-eval-cli schema.json -d data.json

# MessagePack schema + JSON data
./json-eval-cli schema.bform -d data.json

# JSON schema + MessagePack data
./json-eval-cli schema.json -d data.bform

# MessagePack schema + MessagePack data
./json-eval-cli schema.bform -d data.bform
```

## Core Options

### Basic Evaluation
```bash
-d, --data <FILE>          Input data file (JSON or .bform)
-o, --output <FILE>        Output file for evaluated schema
--no-output                Suppress output (for benchmarking)
```

### Performance
```bash
-p, --parsed               Use ParsedSchema for efficient caching
-i, --iterations <N>       Number of evaluation iterations (default: 1)
```

### Validation
```bash
-c, --compare <FILE>       Expected output file for comparison
--compare-path <PATH>      JSON pointer path for comparison 
                          (default: "$.$params.others")
```

## Parsed Schema Inspection

New feature for inspecting the internal structure of ParsedSchema:

### Individual Inspection Flags

**`--print-sorted-evaluations`**
Shows evaluation batches organized for parallel execution:
```bash
./json-eval-cli schema.json -d data.json --parsed \
  --print-sorted-evaluations
```
Output:
- Number of batches
- Evaluations in each batch (can run concurrently)
- Batch execution order

**`--print-dependencies`**
Shows dependency graph between evaluations:
```bash
./json-eval-cli schema.json -d data.json --parsed \
  --print-dependencies
```
Output:
- Which evaluations depend on which data paths
- Dependency relationships for topological sorting

**`--print-tables`**
Shows table definitions extracted from schema:
```bash
./json-eval-cli schema.json -d data.json --parsed \
  --print-tables
```
Output:
- Table names and their definitions
- Rows, data, skip, and clear configurations

**`--print-evaluations`**
Shows compiled logic expressions with their IDs:
```bash
./json-eval-cli schema.json -d data.json --parsed \
  --print-evaluations
```
Output:
- Evaluation path → LogicId mapping
- Total number of compiled expressions

**`--print-all`**
Combines all inspection flags:
```bash
./json-eval-cli schema.json -d data.json --parsed \
  --print-all
```

## Usage Examples

### 1. Simple Evaluation
```bash
cargo run --bin json-eval-cli -- schema.json -d data.json
```

### 2. Benchmark with ParsedSchema
```bash
cargo run --release --bin json-eval-cli -- schema.json \
  -d data.json --parsed -i 100 --no-output
```

### 3. Validate Against Expected Output
```bash
cargo run --bin json-eval-cli -- schema.json \
  -d data.json \
  -c expected.json \
  --compare-path "/$params/others"
```

### 4. Inspect Schema Structure
```bash
cargo run --bin json-eval-cli -- schema.json \
  -d data.json --parsed --no-output \
  --print-sorted-evaluations \
  --print-dependencies
```

### 5. Full Feature Demo
```bash
cargo run --bin json-eval-cli -- schema.bform \
  --data data.bform \
  --compare expected.json \
  --compare-path "$.$params.others" \
  --parsed \
  --iterations 50 \
  --output result.json \
  --print-all
```

### 6. MessagePack Schema Inspection
```bash
cargo run --bin json-eval-cli -- schema.bform \
  -d data.json --parsed --no-output \
  --print-sorted-evaluations
```

## Output Format

### Standard Output
```
🚀 JSON Evaluation CLI

📄 Schema: samples/zcc.json (JSON)
📊 Data: samples/zcc-data.json (JSON)
📦 Mode: ParsedSchema (parse once, reuse)
🔄 Iterations: 100

⏱️  Schema parsing: 470.561ms
.....................
⏱️  Evaluation: 85.234s
   Average per iteration: 852.34ms
⏱️  Total time: 85.704s

✅ Evaluation completed successfully!
```

### Parsed Schema Inspection Output
```
============================================================
📋 PARSED SCHEMA INFORMATION
============================================================

🔄 Sorted Evaluations (Batches for parallel execution):
   6 batches total

   Batch 1 (13 evaluations):
     - #/$params/others/CURR_A_VALUE
     - #/$params/others/CURR_B_VALUE
     ...

🔗 Dependencies:
   133 evaluation(s) with dependencies

   #/$params/constants/POL_DURATION depends on:
     → /illustration/insured/insage
     → /illustration/product_benefit/product/prod_package_comp
     ...

📊 Tables:
   3 table(s) defined

   #/$params/references/ZLOB_RATE:
     { ... }

⚙️  Evaluations:
   206 evaluation(s) compiled
     #/$params/constants/POL_DURATION → LogicId(0)
     ...

============================================================
```

## Performance Tips

1. **Use `--parsed` for multiple iterations**
   - Parses schema once, reuses compiled logic
   - Significant performance improvement

2. **Use `--no-output` for benchmarking**
   - Skips result serialization
   - Measures pure evaluation performance

3. **Use release build for accurate benchmarks**
   ```bash
   cargo build --release --bin json-eval-cli
   ./target/release/json-eval-cli schema.json -d data.json --parsed -i 1000
   ```

4. **MessagePack schemas are faster to parse**
   - Pre-compiled binary format
   - ~2-3x faster parsing than JSON

## Error Handling

The CLI provides clear error messages:
- Missing files
- Invalid schema/data format
- Evaluation failures
- Comparison mismatches

Exit codes:
- `0` - Success
- `1` - Error (file not found, parse error, evaluation error, comparison failed)

## See Also

- [examples/README.md](examples/README.md) - Example applications
- [EXAMPLES_MIGRATION.md](EXAMPLES_MIGRATION.md) - Migration guide
- [README.md](README.md) - Main documentation
