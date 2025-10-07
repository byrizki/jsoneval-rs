# json-eval-rs

Rust implementation of the JSON evaluation engine with WebAssembly and C ABI bindings.

## Features

- JSONLogic-based $evaluation using `datalogic-rs`
- Condition/hideLayout parity mapped to `$fieldHide`, `$fieldDisabled`, `$fieldReadonly`
- Minimal `validate()` parity for `required`, `minLength`, `maxLength`, `minValue`, `maxValue`, `pattern`
- WebAssembly bindings (wasm-bindgen) suitable for `wasm-pack`
- C ABI bindings for .NET (P/Invoke)

## Build: native

```
cargo build
```

## Run demo

```
cargo run
```

## Build: WebAssembly with wasm-pack

Prereqs:

- Install target and wasm-pack:

```
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Build:

```
wasm-pack build --release --target bundler --features wasm
```

This creates `pkg/` with `json_eval_rs` wasm package.

### Using from TypeScript

Example wrapper usage:

```ts
import init, { WasmJSONEval } from "./pkg/json_eval_rs";

export async function createEngine() {
  await init();
  const engine = new WasmJSONEval();
  return {
    loadSchema: (schema: any, context?: any, data?: any) =>
      engine.load_schema(JSON.stringify(schema), context ? JSON.stringify(context) : undefined, data ? JSON.stringify(data) : undefined),
    evaluate: async (data?: any, context?: any) =>
      JSON.parse(await engine.evaluate(data ? JSON.stringify(data) : undefined, context ? JSON.stringify(context) : undefined)),
    validate: async (data: any, context?: any) =>
      JSON.parse(await engine.validate(JSON.stringify(data), context ? JSON.stringify(context) : undefined)),
    getEvaluatedSchema: async () => JSON.parse(await engine.get_evaluated_schema()),
  };
}
```

## C# / .NET usage (P/Invoke)

- Build native cdylib:

```
cargo build --release
```

- Import functions (example signatures):

```csharp
[DllImport("json_eval_rs")]
public static extern bool json_eval_load_schema(string schemaJson, string contextJson, string dataJson);

[DllImport("json_eval_rs")]
public static extern IntPtr json_eval_evaluate(string dataJson, string contextJson);

[DllImport("json_eval_rs")]
public static extern IntPtr json_eval_get_evaluated_schema();

[DllImport("json_eval_rs")]
public static extern IntPtr json_eval_validate(string dataJson, string contextJson);

[DllImport("json_eval_rs")]
public static extern void json_eval_free_string(IntPtr ptr);
```

Remember to call `json_eval_free_string` on any returned pointer once converted to a C# string.

## Roadmap to full parity

- Dependency graph ordering + caching
- Table evaluation lifecycle ($skip/$clear, repeat scan)
- Rule evaluation messages/data parity
- Expanded helper operators to cover all of `src/internal/runner.ts`
