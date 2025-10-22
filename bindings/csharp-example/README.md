# JSON Eval RS - C# Benchmark

This project benchmarks the C# bindings for the JSON Eval RS library using the ZCC (Zurich Critical Care) schema.

## Features

- 🚀 **Performance Benchmarking**: Measures schema parsing and evaluation times
- 📊 **Comparison**: Compares results with Rust baseline performance
- 💾 **Memory Tracking**: Shows memory usage statistics
- ✅ **Result Validation**: Compares output with expected results
- 📝 **Detailed Output**: Saves evaluated schema, parsed schema, and results

## Requirements

- .NET 8.0 SDK or later
- JsonEvalRs C# library (referenced from `../csharp`)
- Native Rust library (json_eval_rs.dll/.so/.dylib)

## Building

```bash
cd bindings/csharp-example
dotnet build --configuration Release
```

## Running

### Default (ZCC scenario)
```bash
dotnet run --configuration Release
```

### Specific scenario
```bash
dotnet run --configuration Release -- <scenario_name>
```

## Project Structure

```
csharp-example/
├── JsonEvalBenchmark.csproj   # Project file
├── Program.cs                  # Main benchmark code
├── README.md                   # This file
└── samples/                    # Output directory (created at runtime)
    ├── zcc.json               # Schema file (copied from ../../samples)
    ├── zcc-data.json          # Input data (copied from ../../samples)
    ├── zcc-evaluated-schema.json      # Generated output
    ├── zcc-parsed-schema.json         # Generated output
    └── zcc-sorted-evaluations.json    # Generated output
```

## Benchmark Metrics

The benchmark measures:

1. **Schema Parsing & Compilation**: Time to parse JSON schema and compile evaluation logic
2. **Evaluation**: Time to evaluate the schema with input data
3. **Total Execution Time**: Combined time for both operations
4. **Memory Usage**: Working set and private memory size

## Example Output

```
🚀 JSON Eval RS - C# Benchmark
📦 Library Version: 0.0.11

📋 Running scenario: 'zcc'

==============================
Scenario: zcc
Schema: samples/zcc.json
Data: samples/zcc-data.json

Loading files...
Running evaluation...

  Schema parsing & compilation: 205.123456ms
  Evaluation: 320.456789ms
⏱️  Total execution time: 525.580245ms

✅ Results saved:
  - samples/zcc-evaluated-schema.json
  - samples/zcc-parsed-schema.json
  - samples/zcc-sorted-evaluations.json

⚠️  Comparison: Results differ from `samples/zcc-evaluated-compare.json` (1 difference(s)):
  - $.others.CURRENT_RIDER_FIRST_PREM_PER_PAY differs: actual=null expected=790736

==================================================
📊 Performance Comparison
==================================================
C# Total Time:     525.580ms
  - Parsing:       205.123ms
  - Evaluation:    320.457ms

Rust Baseline:     ~518ms (from cargo run)
  - Parsing:       ~201ms
  - Evaluation:    ~316ms

FFI Overhead:      +1.5%

💾 Memory Usage:
  Working Set:     45.23 MB
  Private Memory:  42.18 MB

✅ Benchmark completed successfully!
```

## Performance Notes

### FFI Overhead
The C# binding adds a small overhead due to:
- **P/Invoke marshalling**: Converting strings between managed/unmanaged memory
- **JIT compilation**: First-run JIT overhead (subsequent runs are faster)
- **GC pressure**: Managed memory allocation and garbage collection

### Optimization Tips

1. **Use Release builds**: Always benchmark with `--configuration Release`
2. **Warm-up runs**: First run includes JIT overhead, run multiple times for accurate results
3. **Large datasets**: FFI overhead is proportionally smaller for large schemas
4. **Memory**: Native library memory is not tracked by .NET GC

## Comparison with Rust

### Rust (Native)
```bash
cd ../..
cargo run --release -- zcc
```

Typical results:
- Parsing: ~201ms
- Evaluation: ~316ms
- Total: ~518ms

### C# (FFI Bindings)
```bash
dotnet run --configuration Release
```

Typical results:
- Parsing: ~205ms (+2%)
- Evaluation: ~320ms (+1.3%)
- Total: ~525ms (+1.5% overhead)

**FFI overhead is typically 1-5%**, which is excellent for cross-language bindings!

## Troubleshooting

### Native Library Not Found

**Error**: `DllNotFoundException: Unable to load DLL 'json_eval_rs'`

**Solution**: Build the native library first:
```bash
cd ../..
cargo build --release
```

The library will be in:
- Windows: `target/release/json_eval_rs.dll`
- Linux: `target/release/libjson_eval_rs.so`
- macOS: `target/release/libjson_eval_rs.dylib`

### Missing Schema Files

**Error**: `FileNotFoundException: Schema file not found`

**Solution**: Ensure schema files exist:
```bash
ls ../../samples/zcc.json
ls ../../samples/zcc-data.json
```

The project automatically copies these files to the output directory during build.

### Version Mismatch

**Error**: Unexpected behavior or crashes

**Solution**: Ensure C# library and native library versions match:
```csharp
Console.WriteLine(JSONEval.Version); // Should match Cargo.toml version
```

## Advanced Usage

### Custom Schema

To benchmark a custom schema:

1. Place your schema in `../../samples/yourschema.json`
2. Place your data in `../../samples/yourschema-data.json`
3. Run: `dotnet run --configuration Release -- yourschema`

### Multiple Runs

To average multiple runs:
```bash
for i in {1..10}; do
    dotnet run --configuration Release
done
```

### Profiling

Use a profiler to analyze performance:
```bash
# dotTrace (JetBrains)
dotnet run --configuration Release

# PerfView (Windows)
PerfView.exe run dotnet run --configuration Release
```

## API Usage Example

The benchmark demonstrates key API features:

```csharp
using JsonEvalRs;

// Create evaluator with schema
var eval = new JSONEval(schemaJson);

// Evaluate with data
var result = eval.Evaluate(dataJson);

// Get evaluated schema
var schema = eval.GetEvaluatedSchema(skipLayout: true);

// Get schema value
var schemaValue = eval.GetSchemaValue();

// Check version
Console.WriteLine(JSONEval.Version);

// Cleanup
eval.Dispose();
```

## Contributing

To improve the benchmark:

1. Add more test scenarios
2. Implement warm-up runs
3. Add statistical analysis (mean, median, std dev)
4. Compare different .NET versions (net6.0, net7.0, net8.0)
5. Add async benchmarks

## License

MIT License - Same as the main json-eval-rs project
