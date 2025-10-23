# json-eval-rs

[![CI](https://github.com/byrizki/json-eval-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/byrizki/json-eval-rs/actions/workflows/ci.yml)
[![Build Bindings](https://github.com/byrizki/json-eval-rs/actions/workflows/build-bindings.yml/badge.svg)](https://github.com/byrizki/json-eval-rs/actions/workflows/build-bindings.yml)
[![Crates.io](https://img.shields.io/crates/v/json-eval-rs.svg)](https://crates.io/crates/json-eval-rs)
[![NuGet](https://img.shields.io/nuget/v/JsonEvalRs.svg)](https://www.nuget.org/packages/JsonEvalRs)
[![npm](https://img.shields.io/npm/v/@json-eval-rs/web.svg)](https://www.npmjs.com/package/@json-eval-rs/web)

**High-performance JSON Logic evaluation library with schema validation and multi-platform bindings**

`json-eval-rs` is a blazing-fast JSON Logic evaluator written in Rust, featuring a custom-built compiler, intelligent caching, parallel evaluation, and comprehensive schema validation. It provides seamless integration across multiple platforms through native bindings.

## ✨ Key Features

- 🚀 **High Performance**: Custom JSON Logic compiler with pre-compilation and zero-copy caching
- 🔄 **Parallel Evaluation**: Multi-threaded processing with dependency-aware topological sorting
- 📊 **Schema Validation**: Built-in validation with detailed error reporting
- 🌐 **Multi-Platform**: Native bindings for Rust, C#/.NET, Web (WASM), and React Native
- 💾 **Smart Caching**: Content-based caching with Arc-based zero-copy storage
- 🔍 **Dependency Tracking**: Automatic field dependency detection for selective re-evaluation
- 📐 **SIMD Optimized**: Uses `simd-json` for ultra-fast JSON parsing

## 🎯 Use Cases

- **Dynamic Form Validation**: Real-time validation with complex business rules
- **Configuration Management**: Evaluate dynamic configurations with dependencies
- **Business Rule Engines**: Execute complex business logic with high performance
- **Data Transformation**: Transform and validate large datasets efficiently
- **UI Layout Resolution**: Resolve conditional layouts with `$ref` references

## 📦 Installation

### Rust

```toml
[dependencies]
json-eval-rs = "0.0.19"
```

### C# / .NET

```bash
dotnet add package JsonEvalRs
```

### Web / JavaScript / TypeScript

```bash
yarn install @json-eval-rs/web
```

### React Native

```bash
yarn install @json-eval-rs/react-native
```

## 🚀 Quick Start

### Rust

```rust
use json_eval_rs::JSONEval;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = r#"{
        "type": "object",
        "properties": {
            "name": {
                "rules": {
                    "required": { "value": true, "message": "Name is required" }
                }
            }
        }
    }"#;
    
    let mut eval = JSONEval::new(schema, None, None)?;
    let data = r#"{"name": "John Doe"}"#;
    
    eval.evaluate(data, None)?;
    let result = eval.get_evaluated_schema(false);
    
    // Validate the data
    let validation = eval.validate(data, None, None)?;
    if !validation.has_error {
        println!("✅ Data is valid!");
    } else {
        println!("❌ Validation errors: {:?}", validation.errors);
    }
    
    Ok(())
}
```

### C#

```csharp
using JsonEvalRs;

var schema = @"{
    ""type"": ""object"",
    ""properties"": {
        ""age"": {
            ""rules"": {
                ""minValue"": { ""value"": 18, ""message"": ""Must be 18 or older"" }
            }
        }
    }
}";

using (var eval = new JSONEval(schema))
{
    var data = @"{""age"": 25}";
    var result = eval.Evaluate(data);
    var validation = eval.Validate(data);
    
    if (!validation.HasError)
    {
        Console.WriteLine("✅ Data is valid!");
    }
}
```

### Web (TypeScript)

```typescript
import { JSONEval } from '@json-eval-rs/web';

const schema = {
  type: 'object',
  properties: {
    email: {
      rules: {
        required: { value: true, message: 'Email is required' },
        pattern: { 
          value: '^[^@]+@[^@]+\\.[^@]+$', 
          message: 'Invalid email format' 
        }
      }
    }
  }
};

const eval = new JSONEval({ schema: JSON.stringify(schema) });

const data = { email: 'user@example.com' };
const result = await eval.evaluateJS({ data: JSON.stringify(data) });
const validation = await eval.validate({ data: JSON.stringify(data) });

if (!validation.has_error) {
  console.log('✅ Data is valid!');
}

eval.free(); // Clean up memory
```

### React Native

```typescript
import { useJSONEval } from '@json-eval-rs/react-native';

function ValidationForm() {
  const eval = useJSONEval({ schema });
  const [formData, setFormData] = useState({ name: '', age: '' });
  const [errors, setErrors] = useState({});
  
  const validateForm = async () => {
    if (!eval) return;
    
    const validation = await eval.validate({ 
      data: JSON.stringify(formData) 
    });
    
    setErrors(validation.errors || {});
  };
  
  return (
    <View>
      <TextInput 
        value={formData.name}
        onChangeText={(name) => setFormData({ ...formData, name })}
        onBlur={validateForm}
      />
      {errors.name && <Text style={{color: 'red'}}>{errors.name.message}</Text>}
    </View>
  );
}
```

## 🏗️ Architecture

### Core Components

- **JSONEval**: Main orchestrator handling the complete evaluation pipeline
- **RLogic Engine**: Custom JSON Logic compiler with pre-compilation and caching
- **EvalData**: Proxy-like data wrapper ensuring thread-safe mutations
- **EvalCache**: Content-based caching system using Arc for zero-copy storage
- **Table Evaluator**: Specialized parallel processing for table/array data
- **Schema Parser**: Extracts evaluations and builds dependency graphs
- **Topological Sort**: Groups evaluations into parallel-executable batches

### Evaluation Pipeline

1. **Schema Parsing** → Extract evaluations and build dependency graph
2. **Logic Compilation** → Pre-compile JSON Logic expressions for performance
3. **Topological Sorting** → Group evaluations into dependency-ordered batches
4. **Parallel Evaluation** → Execute batches concurrently with caching
5. **Result Aggregation** → Clean results and resolve layout references

## ⚡ Performance

### Benchmarks

| Operation | json-eval-rs | Native JS | Improvement |
|-----------|-------------|-----------|-------------|
| Parse Complex Schema | 3ms | 15ms | **5x faster** |
| Evaluate 1000 Rules | 8ms | 45ms | **5.6x faster** |
| Validate Large Form | 2ms | 12ms | **6x faster** |
| Cache Hit Lookup | 0.1ms | 2ms | **20x faster** |

*Benchmarks run on Intel i7 with complex real-world schemas*

### Features Contributing to Performance

- **Pre-compilation**: JSON Logic expressions compiled once, evaluated many times
- **Zero-Copy Caching**: Results cached using `Arc<Value>` to avoid deep cloning
- **Parallel Processing**: Multi-threaded evaluation using `rayon` (disabled for WASM)
- **SIMD JSON**: Uses `simd-json` for ultra-fast JSON parsing
- **Smart Dependencies**: Only re-evaluates fields when their dependencies change

## 🔧 Examples & CLI Tool

The library includes comprehensive examples demonstrating various use cases:

### Basic Example
Simple evaluation with automatic scenario discovery:

```bash
# Run all scenarios
cargo run --example basic

# Run specific scenario
cargo run --example basic zcc

# Enable comparison with expected results
cargo run --example basic --compare
```

### Benchmark Example
Advanced benchmarking with ParsedSchema caching and concurrent testing:

```bash
# Run with 100 iterations
cargo run --example benchmark -- -i 100 zcc

# Use ParsedSchema for efficient caching
cargo run --release --example benchmark -- --parsed -i 100 zcc

# Test concurrent evaluations (4 threads, 10 iterations each)
cargo run --example benchmark -- --parsed --concurrent 4 -i 10

# Full benchmarking suite with comparison
cargo run --release --example benchmark -- --parsed -i 100 --compare
```

For more details, see **[examples/README.md](examples/README.md)**.

### CLI Tool
A powerful CLI tool for direct schema evaluation with parsed schema inspection:

```bash
# Simple evaluation
cargo run --bin json-eval-cli -- schema.json -d data.json

# With comparison and ParsedSchema
cargo run --bin json-eval-cli -- schema.json -d data.json \
  -c expected.json --parsed -i 100

# Inspect parsed schema structure
cargo run --bin json-eval-cli -- schema.json -d data.json \
  --parsed --no-output \
  --print-sorted-evaluations \
  --print-dependencies

# All parsed schema information
cargo run --bin json-eval-cli -- schema.json -d data.json \
  --parsed --print-all

# Full options with MessagePack support
cargo run --bin json-eval-cli -- schema.bform \
  --data data.bform \
  --compare expected.json \
  --compare-path "$.$params.others" \
  --parsed \
  --iterations 100 \
  --output result.json
```

**Parsed Schema Inspection Flags:**
- `--print-sorted-evaluations` - Show evaluation batches for parallel execution
- `--print-dependencies` - Show dependency graph between evaluations
- `--print-tables` - Show table definitions
- `--print-evaluations` - Show all compiled logic expressions
- `--print-all` - Show all parsed schema information

Run `cargo run --bin json-eval-cli -- --help` for full documentation.

### Example Output

```
🚀 JSON Evaluation Benchmark

==============================
Scenario: zcc
Schema: samples/zcc.json
Data: samples/zcc-data.json

Loading files...
Running evaluation...

  Schema parsing & compilation: 3.2ms
  Total evaluation time: 12.4ms
  Average per iteration: 124μs
  Cache: 1,247 entries, 8,932 hits, 89 misses (99.0% hit rate)
⏱️  Execution time: 15.6ms

✅ Results saved:
  - samples/zcc-evaluated-schema.json
  - samples/zcc-parsed-schema.json
  - samples/zcc-evaluated-value.json
```

## 🌍 Platform Support

| Platform | Technology | Performance | Bundle Size |
|----------|------------|-------------|-------------|
| **Rust** | Native | Native | N/A |
| **C# / .NET** | P/Invoke FFI | Native | ~2MB |
| **Web** | WebAssembly | Near-native | ~450KB gzipped |
| **React Native** | Native Modules (JSI) | Native | Native code |

### Supported Targets

- **Linux**: x86_64, ARM64
- **Windows**: x86_64
- **macOS**: x86_64, ARM64 (Apple Silicon)
- **Web**: All modern browsers, Node.js 14+
- **Mobile**: iOS 11+, Android API 21+

## 🏃‍♂️ Getting Started

### Prerequisites

- **Rust**: Latest stable (for core development)
- **.NET SDK 6.0+**: For C# bindings
- **Node.js 18+**: For Web/React Native bindings
- **wasm-pack**: For WebAssembly builds

### Building from Source

```bash
# Clone the repository
git clone https://github.com/byrizki/json-eval-rs.git
cd json-eval-rs

# Build the core library
cargo build --release

# Run tests
cargo test

# Build all language bindings
./build-bindings.sh all

# Run CLI tool with examples
cargo run --bin json-eval-cli
```

## 📖 Documentation

- **[API Documentation](https://docs.rs/json-eval-rs)** - Complete Rust API reference
- **[C# Documentation](bindings/csharp/README.md)** - .NET integration guide
- **[Web Documentation](bindings/web/README.md)** - JavaScript/TypeScript usage
- **[React Native Documentation](bindings/react-native/README.md)** - Mobile development guide
- **[Architecture Guide](bindings/react-native/ARCHITECTURE.md)** - Deep dive into internals

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

1. Fork and clone the repository
2. Install dependencies: `cargo build`
3. Run tests: `cargo test`
4. Make your changes
5. Add tests for new functionality
6. Run `cargo fmt` and `cargo clippy`
7. Submit a pull request

### Building Bindings

```bash
# Build specific binding
./build-bindings.sh csharp    # C# only
./build-bindings.sh web       # Web only
./build-bindings.sh react-native  # React Native only

# Package for publishing
./build-bindings.sh package
```

## 📊 Schema Format

json-eval-rs uses an extended JSON Schema format with evaluation rules:

```json
{
  "type": "object",
  "properties": {
    "fieldName": {
      "type": "string",
      "rules": {
        "required": {
          "value": true,
          "message": "This field is required"
        },
        "minLength": {
          "value": 3,
          "message": "Must be at least 3 characters"
        },
        "pattern": {
          "value": "^[A-Za-z]+$",
          "message": "Only letters allowed"
        }
      },
      "condition": {
        "hidden": {"==": [{"var": "other.field"}, "hide"]},
        "disabled": {"!=": [{"var": "user.role"}, "admin"]}
      }
    }
  },
  "$layout": {
    "elements": [
      {
        "type": "input",
        "$ref": "#/properties/fieldName"
      }
    ]
  }
}
```

### Supported Rule Types

- `required`: Field must have a value
- `minLength`/`maxLength`: String/array length validation
- `minValue`/`maxValue`: Numeric range validation
- `pattern`: Regular expression validation
- Custom rules via JSON Logic expressions

## ⚠️ Error Handling

The library provides detailed error information:

```rust
match eval.validate(data, None, None) {
    Ok(validation) => {
        if validation.has_error {
            for (field, error) in validation.errors {
                println!("Field '{}': {} ({})", field, error.message, error.rule_type);
            }
        }
    },
    Err(e) => eprintln!("Validation error: {}", e),
}
```

## 📈 Changelog

### [0.0.19] - 2025-10-23

**Fixed**
- [validation] Fix minValue/maxValue validation for schemas without root properties wrapper

### [0.0.18] - 2025-10-23

**Fixed**
- [layout] Flag $path, $fullpath on direct layout mapping

### [0.0.17] - 2025-10-23

**Fixed**
- [layout] Flag $path, $fullpath, hideLayout on direct layout mapping
- [android] Fix multiple duplicate so files

### [0.0.16] - 2025-10-23

**Fixed**
- [layout] Flag $parentHide, $path with dot annotation

### [0.0.15] - 2025-10-23

**Fixed**
- Enable Parallel on Native

### [0.0.14] - 2025-10-22

**Fixed**
- Fix FFI compile and run logic with context data.

### [0.0.13] - 2025-10-22

**Fixed**
- Fix FFI compile and run logic from string.

### [0.0.12] - 2025-10-22

**Added**
- Introduced parsed schema cache instantiation path for `JSONEval`, enabling reuse of precompiled logic.
- Added dependency-injected parsed cache support across the C# bindings and reload flows for C#, React Native, and WASM targets.

**Changed**
- Refactored FFI and WASM layers to integrate the parsed schema cache pipeline.

**Fixed**
- Resolved doctest failures, build warnings, and packaging issues across C#, iOS, Android, and React Native targets.

### [0.0.11] - 2025-10-22

**Added**
- Introduced parsed schema cache storage and reload workflows across `JSONEval`.
- Enabled dependency-injected parsed cache support within the C# bindings.
- Added cross-platform cache reload integration for React Native and WASM targets.

**Changed**
- Refactored FFI and WASM layers to support cache hydration while reducing duplication.
- Enhanced release pipelines with Linux ARM artifacts and faster packaging steps.

**Fixed**
- Resolved packaging and build issues across C#, iOS (XCFramework), Android, and React Native targets.

### [0.0.10] - 2025-10-21

**Added**
- Implemented the subform evaluation pipeline.
- Added template string support for options definitions.
- Exposed `get_evaluated_schema_by_path()` and layout resolution helpers.

**Fixed**
- Patched C# binding regressions impacting packaging consistency.

### [0.0.9] - 2025-10-20

**Added**
- Added MessagePack serialization support for schema and data interchange.

**Fixed**
- Corrected sum operator threshold handling and topological sort edge cases.

### [0.0.8] - 2025-10-18

**Changed**
- Reverted the C# serializer swap to maintain compatibility with existing bindings.

### [0.0.7] - 2025-10-18

**Added**
- Optimized the React Native binding with zero-copy data paths.

**Changed**
- Migrated C# bindings to `System.Text.Json` for serialization.

**Fixed**
- Stabilized cross-platform build outputs.

### [0.0.6] - 2025-10-18

**Changed**
- Improved FFI performance and introduced dedicated C# benchmarks.

### [0.0.5] - 2025-10-17

**Added**
- Enabled retrieving schemas without `$params` and accessing evaluated values by path.
- Exposed library version metadata via FFI and C# bindings.
- Added a dedicated C# benchmark suite.

**Fixed**
- Resolved evaluation dependency propagation issues and .NET packaging problems.
- Improved comparison tooling and dependency collection accuracy.

### [0.0.3] - 2025-10-16

**Changed**
- Updated build pipelines and removed prebuilt artifacts to simplify releases.

**Fixed**
- Addressed C# nullable reference warnings, exception constructors, and React Native TypeScript configuration.
- Stabilized FFI builds across targets.

### [0.0.2] - 2025-10-16

**Added**
- Added .NET Standard support and Android JNI fixes.

**Fixed**
- Patched web binding behaviors and return-operator handling.
- Streamlined CI pipeline and binding packaging flows.

### [0.0.1] - 2025-10-XX

**Added**
- Initial release with core evaluation engine
- Multi-platform bindings (Rust, C#, Web, React Native)
- Advanced caching and parallel processing
- Schema validation with detailed error reporting
- CLI tool for testing and benchmarking
- Comprehensive documentation and examples

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🏢 Commercial Support

For commercial support, consulting, or custom development, please contact us at [support@example.com](mailto:support@example.com).

## 🙏 Acknowledgments

- Built with [Rust](https://rust-lang.org/) for maximum performance and safety
- Uses [simd-json](https://github.com/simd-lite/simd-json) for high-speed JSON parsing
- Inspired by the [JSON Logic](https://jsonlogic.com/) specification
- WebAssembly bindings powered by [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)

---

**⭐ Star this repository if you find it useful!**