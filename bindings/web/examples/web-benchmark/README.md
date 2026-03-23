# Web Benchmark — Static HTML + Web Worker

A zero-build static HTML benchmark for JSON Eval RS WASM, mirroring the [Node.js WASM benchmark](../nodejs-benchmark).

## Architecture

```
index.html  ←──postMessage──→  worker.js
  (UI shell)                   (WASM owner)
```

- **`index.html`** — pure UI: renders controls, log, and metrics. The main thread is **never blocked**.
- **`worker.js`** — a **module Web Worker** that owns the WASM lifecycle. Does all parsing, evaluation, and diffing, then streams `log`, `result`, and `error` messages back.

## Usage

> **Must be served over HTTP** — `file://` origin blocks Web Workers and module imports.

```bash
# Any static server works. Examples:
npx serve bindings/web/examples/web-benchmark
python3 -m http.server 8080 --directory bindings/web/examples/web-benchmark
```

Then open `http://localhost:8080` in your browser.

## Steps

1. Type a scenario name (e.g. `zcc`, `zpp`, `zip`)
2. Load the **Schema JSON** file (e.g. `samples/zcc.json`)
3. Load the **Data JSON** file (e.g. `samples/zcc-data.json`)
4. Optionally load a **Compare JSON** baseline (e.g. `samples/zcc-evaluated-compare.json`)
5. Click **▶ Run** — the benchmark runs in the worker; the main thread stays responsive

## Message Protocol

| Direction | Shape |
|-----------|-------|
| Main → Worker | `{ type: "run", scenario, schemaText, dataText, compareText? }` |
| Worker → Main | `{ type: "log", text, cls }` |
| Worker → Main | `{ type: "result", scenario, totalMs, parsingMs, evaluationMs, diffs, wasmVersion }` |
| Worker → Main | `{ type: "error", message }` |
