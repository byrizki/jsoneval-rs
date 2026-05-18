/**
 * web-benchmark/worker.js
 *
 * Module Web Worker — owns the WASM lifecycle and runs the benchmark.
 * Imports directly from the raw wasm-bindgen pkg JS to avoid bare specifiers
 * that would require a bundler or import map.
 *
 * Inbound:  { type: "run", scenario, schemaText, dataText, compareText? }
 * Outbound: { type: "log",    text, cls }
 *           { type: "result", scenario, totalMs, parsingMs, evaluationMs, diffs, wasmVersion, cacheStats }
 *           { type: "error",  message }
 */

const PKG_JS_URL   = new URL("../../packages/vanilla/pkg/json_eval_rs.js",   import.meta.url).href;
const PKG_WASM_URL = new URL("../../packages/vanilla/pkg/json_eval_rs_bg.wasm", import.meta.url).href;

// ─────────────────────────────────────────────────────
// Lazy WASM (initialized once per worker lifetime)
// ─────────────────────────────────────────────────────
let pkg = null;
let wasmVersion = null;
let persistentJe = null;
let lastSchemaStr = null;

async function ensureWasm() {
  if (pkg) return;

  postLog("⏳ Loading WASM module…", "log-accent");

  pkg = await import(PKG_JS_URL);

  // The raw pkg default export is the wasm-bindgen init function.
  // Passing the .wasm URL avoids bundler magic and works in any static server.
  await pkg.default(PKG_WASM_URL);

  wasmVersion = pkg.version();
  postLog(`✅ WASM loaded — version ${wasmVersion}`, "log-ok");
}

// ─────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────
function postLog(text, cls = "log-info") {
  self.postMessage({ type: "log", text, cls });
}

function findDifferences(actual, expected, currentPath) {
  const diffs = [];

  for (const key of Object.keys(expected)) {
    const fullPath = `${currentPath}.${key}`;
    if (!(key in actual)) {
      diffs.push(`${fullPath} missing in actual`);
    } else if (Array.isArray(actual[key]) && Array.isArray(expected[key])) {
      if (actual[key].length !== expected[key].length) {
        diffs.push(
          `${fullPath} array length differs: actual=${actual[key].length} expected=${expected[key].length}`
        );
      } else {
        for (let i = 0; i < actual[key].length; i++) {
          diffs.push(
            ...findDifferences(actual[key][i], expected[key][i], `${fullPath}[${i}]`)
          );
        }
      }
    } else if (
      typeof actual[key] === "object" &&
      actual[key] !== null &&
      expected[key] !== null &&
      typeof expected[key] === "object"
    ) {
      diffs.push(...findDifferences(actual[key], expected[key], fullPath));
    } else if (actual[key] !== expected[key]) {
      diffs.push(
        `${fullPath} differs: actual=${JSON.stringify(actual[key])} expected=${JSON.stringify(expected[key])}`
      );
    }
  }

  for (const key of Object.keys(actual)) {
    if (!(key in expected)) {
      diffs.push(`${currentPath}.${key} extra in actual`);
    }
  }

  return diffs;
}

// ─────────────────────────────────────────────────────
// Benchmark
// ─────────────────────────────────────────────────────
async function runBenchmark({ scenario, schemaText, dataText, compareText }) {
  const { JSONEvalWasm } = pkg;

  postLog("==================================================");
  postLog("🎯 Running WASM Benchmark", "log-accent");
  postLog("==================================================");
  postLog(`Scenario: ${scenario}`);
  postLog("");

  postLog("📂 Parsing files…");
  const schema      = JSON.parse(schemaText);
  const data        = JSON.parse(dataText);
  const compareData = compareText ? JSON.parse(compareText) : null;

  const schemaStr  = JSON.stringify(schema);
  const dataStr    = JSON.stringify(data);
  const contextStr = JSON.stringify({});

  postLog("⏱️  Running evaluation…");
  postLog("");

  const totalStart = performance.now();

  // Benchmark 1: Schema parse + compile (constructor)
  let parsingMs = 0;
  let je;

  if (persistentJe && lastSchemaStr === schemaStr) {
    je = persistentJe;
    postLog(`  📝 Parse (cached): ${parsingMs.toFixed(3)}ms`);
  } else {
    const parseStart = performance.now();
    je = new JSONEvalWasm(schemaStr, contextStr, dataStr);
    parsingMs = performance.now() - parseStart;
    postLog(`  📝 Parse (new): ${parsingMs.toFixed(3)}ms`);
    persistentJe = je;
    lastSchemaStr = schemaStr;
  }

  je.reloadSchema(schemaStr, contextStr, dataStr);

  // Benchmark 2: Evaluate (void, mirrors evaluateOnly)
  const evalStart = performance.now();
  je.evaluate(dataStr, contextStr, null);
  const evaluationMs = performance.now() - evalStart;
  postLog(`  ⚡ Eval: ${evaluationMs.toFixed(3)}ms`);

  const evalStart1 = performance.now();
  je.evaluateDependentsJS("[]", dataStr, contextStr, true);
  const evaluationMs1 = performance.now() - evalStart1;
  postLog(`  ⚡ Eval dependents: ${evaluationMs1.toFixed(3)}ms`);

  // Benchmark 3: Evaluate (void, mirrors evaluateOnly)
  const evalStart2 = performance.now();
  je.evaluate(dataStr, contextStr, null);
  const evaluationMs2 = performance.now() - evalStart2;
  postLog(`  ⚡ Eval 2: ${evaluationMs2.toFixed(3)}ms`);

  let cacheStats = null;
  try {
    cacheStats = je.cacheStats();
    postLog(`  🧠 Cache: hits=${cacheStats.hits} misses=${cacheStats.misses} entries=${cacheStats.entries}`);
  } catch (e) {
    // optional method
  }

  const totalMs = performance.now() - totalStart;
  postLog(`  ⏱️  Total: ${totalMs.toFixed(3)}ms`);
  postLog("");

  // Get evaluated schema (outside timing)
  const evaluatedSchema = je.getEvaluatedSchemaJS(true);

  // Comparison
  let diffs = [];
  if (compareData) {
    const actualOthers   = evaluatedSchema?.["$params"]?.others ?? {};
    const expectedOthers = compareData?.others ?? {};
    diffs = findDifferences(actualOthers, expectedOthers, "$");

    if (diffs.length > 0) {
      postLog(`⚠️  Comparison: ${diffs.length} difference(s) from baseline`, "log-warn");
    } else {
      postLog("✅ Comparison: Results match baseline", "log-ok");
    }
    postLog("");
  }

  // Summary
  postLog("==================================================");
  postLog("📊 Benchmark Summary");
  postLog("==================================================");
  postLog("🟦 Browser (WASM / Web Worker):");
  postLog(`  Total:    ${totalMs.toFixed(3)}ms`);
  postLog(`  - Parse:  ${parsingMs.toFixed(3)}ms`);
  postLog(`  - Eval:   ${evaluationMs.toFixed(3)}ms`);
  if (cacheStats) {
    postLog(`  - Cache:  hits=${cacheStats.hits} misses=${cacheStats.misses}`);
  }
  if (diffs.length > 0) {
    postLog(`  ⚠️  ${diffs.length} differences from baseline`, "log-warn");
  }
  postLog("");
  postLog("✅ Benchmark completed successfully!", "log-ok");

  self.postMessage({
    type: "result",
    scenario,
    totalMs,
    parsingMs,
    evaluationMs,
    diffs,
    wasmVersion,
    cacheStats,
  });
}

// ─────────────────────────────────────────────────────
// Message handler
// ─────────────────────────────────────────────────────
self.addEventListener("message", async (event) => {
  const { type, ...payload } = event.data;

  if (type === "run") {
    try {
      await ensureWasm();
      await runBenchmark(payload);
    } catch (err) {
      self.postMessage({ type: "error", message: err.message ?? String(err) });
    }
  }
});
