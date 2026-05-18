# Rust Examples

This directory contains Rust-only examples for the `json-eval-rs` crate. They are intended for local crate development, smoke testing, and showing direct Rust API usage.

Binding examples live under planned canonical locations after restructuring:

- npm/JavaScript: `bindings/npm/examples/`
- C#: `bindings/csharp/examples/`

Do not use this directory for npm, WebAssembly, React Native, or C# package examples.

## Run Examples

```bash
cargo run --example basic
cargo run --example basic_parsed
cargo run --example benchmark -- --parsed -i 100
cargo run --example cache_demo
cargo run --example spaj_toggle
```

Use release mode for benchmark numbers:

```bash
cargo run --release --example benchmark -- --parsed -i 100
```

## Available Rust Examples

### `basic.rs`

Evaluates discovered JSON (`.json`) and MessagePack (`.bform`) schemas with `JSONEval::new()` or `JSONEval::new_from_msgpack()`.

```bash
cargo run --example basic
cargo run --example basic zcc
cargo run --example basic -- --compare
cargo run --example basic -- --timing zcc
```

Options:

- `-h`, `--help` — show help
- `--compare` — compare `$.others` with optional expected output
- `--timing` — print internal timing breakdown
- `[FILTER]` — run scenarios whose names contain filter text

### `basic_parsed.rs`

Parses schemas into `Arc<ParsedSchema>` first, then evaluates with `JSONEval::with_parsed_schema()`.

```bash
cargo run --example basic_parsed
cargo run --example basic_parsed zcc
cargo run --example basic_parsed -- --compare
```

### `benchmark.rs`

Runs repeated and concurrent evaluations for performance checks.

```bash
cargo run --example benchmark -- -i 100 zcc
cargo run --example benchmark -- --parsed -i 100 zcc
cargo run --example benchmark -- --parsed --concurrent 4 -i 10
cargo run --example benchmark -- --cpu-info
```

Options:

- `-i`, `--iterations <COUNT>` — iterations per scenario, default `1`
- `--parsed` — parse once into `ParsedSchema`, then reuse
- `--cache` — reuse `JSONEval` instance across iterations
- `--concurrent <COUNT>` — run concurrent evaluations with N threads
- `--compare` — compare `$.others` with optional expected output
- `--timing` — print internal timing breakdown
- `--cpu-info` — show detected CPU features
- `[FILTER]` — run scenarios whose names contain filter text

### `cache_demo.rs`

Shows `ParsedSchemaCache` and `PARSED_SCHEMA_CACHE` usage with small inline schemas.

```bash
cargo run --example cache_demo
```

### `spaj_toggle.rs`

Debug example for toggling visibility in `samples/spaj.json`.

```bash
cargo run --example spaj_toggle
```

## Scenario-Based Examples

`basic`, `basic_parsed`, and `benchmark` discover scenarios from root `samples/`.

Required files per scenario:

- `<name>.json` or `<name>.bform` — schema file
- `<name>-data.json` — input data
- `<name>-evaluated-compare.json` — optional expected output for `--compare`

Example:

```text
samples/
├── zcc.json
├── zcc-data.json
├── zcc-evaluated-compare.json
├── zccbin.bform
├── zccbin-data.json
└── zccbin-evaluated-compare.json
```

When JSON and MessagePack schemas share one data file, both scenarios are discovered. MessagePack scenario names get `-msgpack` suffix when needed to avoid output collisions.

## Generated Files

Scenario examples write output files under `samples/`:

- `<name>-evaluated-schema.json`
- `<name>-parsed-schema.json`
- `<name>-schema-value.json` (`basic` only)
- `<name>-validation.json` (`basic` only)

Generated files are development artifacts. Review before committing.

## Shared Rust Example Support

`examples/common/mod.rs` is shared support for Rust examples only:

- scenario discovery
- JSON formatting
- optional `$.others` comparison

Keep example-specific demos in individual example files when possible.
