---
layout: default
title: Caching & Performance
---

# Caching & Performance

`json-eval-rs` is designed for high performance, especially in scenarios where schemas are large or reused frequently. Two key features enable this: **Global Schema Caching** and **MessagePack Serialization**.

## Global Schema Caching

Parsing a JSON schema is an expensive operation. If you use the same schema for multiple evaluations (e.g., validating 1000s of records or handling high-throughput web requests), you should avoid re-parsing the schema every time.

The library provides a thread-safe, global **ParsedSchemaCache**.

### Concept

1.  **Parse Once**: You parse the schema string once and assign it a unique `cache_key`.
2.  **Store**: The parsed, compiled, and optimized schema structure is stored in the global cache.
3.  **Reuse**: Subsequent evaluations simply reference the `cache_key`. The engine creates a lightweight instance that shares the heavy read-only schema data from the cache.

### Usage

#### Rust

```rust
use json_eval_rs::{JSONEval, PARSED_SCHEMA_CACHE};

// 1. Pre-populate cache (e.g., at startup)
let schema = r#"{...}"#;
PARSED_SCHEMA_CACHE.insert("user_schema", schema)?;

// 2. Efficient reuse
let mut eval = JSONEval::from_cache("user_schema", None, Some(data1))?;
eval.evaluate(data1, None, None)?;

let mut eval2 = JSONEval::from_cache("user_schema", None, Some(data2))?;
eval2.evaluate(data2, None, None)?;
```

#### C#

```csharp
// 1. Create a "Template" instance to populate the cache
// The constructor automatically parses and can optionally verify the schema
using (var template = new JSONEval(schemaJson)) {
   // In C#, caching is managed internally by the Rust core,
   // but you can explicitely use FromCache if you have a mechanism 
   // to pre-register schemas (advanced usage).
   //
   // Common pattern: Parse once, keep the instance or handle if feasible.
   // Or use FromCache if the native side has been pre-warmed.
}

// 2. Reuse efficient instances
using (var eval = JSONEval.FromCache("my_schema_key")) {
    eval.Evaluate(data);
}
```

> **Note**: For C# and other bindings, the `FromCache` API assumes the schema has been registered in the global Rust cache. Currently, the most common way to populate this is via specific initialization calls or by ensuring the schema is loaded once via a "loader" pattern if supported by your binding version.

#### Web / React Native

```typescript
// 1. Efficient instantiation referencing cached schema
// (Requires schema to be pre-loaded in the WASM/Native environment)
const eval = JSONEval.fromCache('user_schema');
await eval.evaluate({ data });
```

## MessagePack Serialization

Standard JSON serialization/deserialization can be a bottleneck. `json-eval-rs` supports **MessagePack** (a binary serialization format) to drastically reduce overhead when crossing the FFI boundary (Rust <-> JS/C#).

### Zero-Copy Optimization

When you load a schema via MessagePack, or retrieve the evaluated schema as MessagePack:
*   **Smaller Size**: MessagePack is more compact than JSON string.
*   **Faster Processing**: Binary parsing is significantly faster than text parsing.
*   **Zero-Copy (Internal)**: The library uses optimizations to minimize memory copying when handing binaries between Rust and the host language.

### Usage

#### C#

```csharp
// Get result as MessagePack bytes
byte[] resultBytes = eval.GetEvaluatedSchemaMsgpack();

// Create/Reload from MessagePack
byte[] schemaBytes = ...; // Your pre-compiled MessagePack schema
eval.ReloadSchemaMsgpack(schemaBytes);
```

#### React Native / Web

```typescript
const schemaMsgpack = new Uint8Array([...]); // Binary schema

// Reload using binary data
await eval.reloadSchemaMsgpack(schemaMsgpack);
```

## Best Practices

1.  **Cache Heavy Schemas**: Always cache schemas that are larger than a few kilobytes.
2.  **Use Selective Evaluation**: Combine caching with [Selective Evaluation](selective-evaluation) for maximum performance.
3.  ** Reuse Instances**: In stateful environments (like a UI component), keep the `JSONEval` instance alive and use `reload_schema` or `evaluate` with new data, rather than recreating the instance.
