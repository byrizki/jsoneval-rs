# Node.js WASM Benchmark Example

A Node.js benchmark for JSON Eval RS using the WASM binding — mirroring the [C# FFI benchmark](../../../csharp-example).

## Overview

This example:
- **Parses** a schema and compiles it (measures parse time)
- **Evaluates** data against the schema (measures eval time)
- **Saves** results to the `samples/` directory
- **Compares** results against the baseline if `{scenario}-evaluated-compare.json` exists

## Usage

From the monorepo root (`bindings/web`):

```bash
yarn install
yarn workspace nodejs-benchmark start [scenario]
```

Or directly from this directory:

```bash
yarn install
yarn start [scenario]
# e.g.
yarn start zcc
yarn start zpp
```

**`scenario`** defaults to `zcc` if not provided. It must match a pair of files in `samples/`:
- `samples/{scenario}.json` — schema
- `samples/{scenario}-data.json` — input data

## Output

Results are written to `samples/`:
| File | Description |
|------|-------------|
| `{scenario}-evaluated-schema.json` | Full evaluated schema |
| `{scenario}-parsed-schema.json` | Parsed `$params` from the evaluated schema |
| `{scenario}-sorted-evaluations.json` | Same as evaluated schema (for comparison) |

## Example Output

```
🚀 JSON Eval RS - Node.js WASM Benchmark
📦 WASM version: 0.0.71

📋 Scenario: 'zcc'
📁 Project Root: /path/to/jsoneval-rs

==================================================
🎯 Running Node.js WASM Benchmark
==================================================
Scenario: zcc
Schema: /path/to/samples/zcc.json
Data:   /path/to/samples/zcc-data.json

📂 Loading files...
⏱️  Running evaluation...

  📝 Parse (new): 312.451ms
  ⚡ Eval: 209.873ms
  ⏱️  Total: 522.324ms

💾 Saving results...
✅ Results saved

==================================================
📊 Benchmark Summary
==================================================
🟦 Node.js (WASM):
  Total:    522.324ms
  - Parse:  312.451ms
  - Eval:   209.873ms

💾 Memory Usage:
  Heap Used:  45.23 MB
  RSS:        112.67 MB

✅ Benchmark completed successfully!
```
