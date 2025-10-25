---
layout: default
---

# JSONEval-Rs

**High-performance JSON Logic evaluation library with 80+ operators**

[![Crates.io](https://img.shields.io/crates/v/json-eval-rs)](https://crates.io/crates/json-eval-rs)
[![GitHub](https://img.shields.io/github/stars/byrizki/json-eval-rs?style=social)](https://github.com/byrizki/json-eval-rs)

## Features

- 🚀 **High Performance** - Built in Rust for maximum speed
- 📦 **80+ Operators** - Comprehensive operator support
- 🔌 **Multiple Bindings** - C#, React Native, WASM, and Web
- 🔒 **Type Safety** - Strong typing with MessagePack support
- 📚 **Well Documented** - Extensive documentation and examples

## Quick Start

```rust
use json_eval_rs::eval;

let logic = json!({
    "if": [
        {">": [{"var": "user.age"}, 18]},
        "adult",
        "minor"
    ]
});

let data = json!({
    "user": {
        "name": "John",
        "age": 25
    }
});

let result = eval(&logic, &data)?;
// Returns: "adult"
```

## Documentation

### Operator Categories

- [Core Operators](operators-core) - Variables, references, and literals
- [Logical Operators](operators-logical) - Boolean logic and conditionals  
- [Comparison Operators](operators-comparison) - Value comparisons
- [Arithmetic Operators](operators-arithmetic) - Mathematical operations
- [String Operators](operators-string) - Text manipulation
- [Math Functions](operators-math) - Advanced math operations
- [Date Functions](operators-date) - Date and time operations
- [Array Operators](operators-array) - Array transformations
- [Table Operators](operators-table) - Data table operations
- [Utility Operators](operators-utility) - Helper functions

### Quick Reference

See the [Operators Summary](OPERATORS_SUMMARY) for a complete alphabetical list of all available operators.

## Installation

### Rust
```toml
[dependencies]
json-eval-rs = "0.1"
```

### C# / .NET
```bash
dotnet add package JsonEvalRs
```

### Web / WASM
```bash
npm install @json-eval-rs/bundler
```

### React Native
```bash
npm install @json-eval-rs/react-native
```

## Repository

[View on GitHub](https://github.com/byrizki/jsoneval-rs)

## License

MIT License
