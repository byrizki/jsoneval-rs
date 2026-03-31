import { JSONEval, version } from "@json-eval-rs/node";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { JSONStringify, JSONParse } from "json-with-bigint";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

interface BenchmarkResult {
  success: boolean;
  totalMs: number;
  parsingMs: number;
  evaluationMs: number;
  scenario: string;
  evaluatedSchema?: any;
  differenceCount: number;
}

function findProjectRoot(): string | null {
  let dir = __dirname;
  while (dir !== path.parse(dir).root) {
    if (fs.existsSync(path.join(dir, "Cargo.toml"))) {
      return dir;
    }
    dir = path.dirname(dir);
  }
  return null;
}

function findDifferences(
  actual: Record<string, any>,
  expected: Record<string, any>,
  currentPath: string
): string[] {
  const diffs: string[] = [];

  for (const key of Object.keys(expected)) {
    const fullPath = `${currentPath}.${key}`;
    if (!(key in actual)) {
      diffs.push(`${fullPath} missing in actual`);
    } else if (
      typeof actual[key] !== typeof expected[key] ||
      (typeof actual[key] === "object" &&
        actual[key] !== null &&
        expected[key] !== null)
    ) {
      if (Array.isArray(actual[key]) && Array.isArray(expected[key])) {
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
        expected[key] !== null
      ) {
        diffs.push(...findDifferences(actual[key], expected[key], fullPath));
      }
    } else if (actual[key] !== expected[key]) {
      diffs.push(
        `${fullPath} differs: actual=${JSONStringify(actual[key])} expected=${JSONStringify(expected[key])}`
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

async function runNodeBenchmark(
  scenario: string,
  projectRoot: string
): Promise<BenchmarkResult> {
  console.log("==================================================");
  console.log("\x1b[36m🎯 Running Node.js WASM Benchmark\x1b[0m");
  console.log("==================================================");
  console.log(`Scenario: ${scenario}`);

  const samplesPath = path.join(projectRoot, "samples");
  const schemaPath = path.join(samplesPath, `${scenario}.json`);
  const dataPath = path.join(samplesPath, `${scenario}-data.json`);
  const comparePath = path.join(samplesPath, `${scenario}-evaluated-compare.json`);

  console.log(`Schema: ${schemaPath}`);
  console.log(`Data:   ${dataPath}`);
  console.log();

  if (!fs.existsSync(schemaPath)) {
    throw new Error(`Schema file not found: ${schemaPath}`);
  }
  if (!fs.existsSync(dataPath)) {
    throw new Error(`Data file not found: ${dataPath}`);
  }

  console.log("📂 Loading files...");
  const schemaJson = fs.readFileSync(schemaPath, "utf8");
  const dataJson = fs.readFileSync(dataPath, "utf8");

  const schema = JSONParse(schemaJson);
  const data = JSONParse(dataJson);

  let compareData: Record<string, any> | null = null;
  if (fs.existsSync(comparePath)) {
    compareData = JSONParse(fs.readFileSync(comparePath, "utf8"));
  }

  console.log("⏱️  Running evaluation...");
  console.log();

  const totalStart = performance.now();

  // Benchmark 1: Schema parsing & compilation (constructor + init)
  const parseStart = performance.now();
  const je = new JSONEval({ schema, data, context: {} });
  await je.init();
  je.reloadSchema({ schema, context: {}, data });
  const parsingMs = performance.now() - parseStart;
  console.log(`  📝 Parse (new): ${parsingMs.toFixed(3)}ms`);

  // Benchmark 2: Evaluation
  const evalStart = performance.now();
  await je.evaluateOnly({ data, context: {} });
  const evaluationMs = performance.now() - evalStart;
  console.log(`  ⚡ Eval: ${evaluationMs.toFixed(3)}ms`);

  // Benchmark 3: Evaluation
  const evalStart2 = performance.now();
  await je.evaluateOnly({ data, context: {} });
  const evaluationMs2 = performance.now() - evalStart2;
  console.log(`  ⚡ Eval 2: ${evaluationMs2.toFixed(3)}ms`);

  const totalMs = performance.now() - totalStart;
  console.log(`  ⏱️  Total: ${totalMs.toFixed(3)}ms`);
  console.log();

  // Get evaluated schema (not included in timing)
  const evaluatedSchema = await je.getEvaluatedSchema({ skipLayout: true });

  // Save results
  console.log("💾 Saving results...");
  const outputDir = samplesPath;
  fs.mkdirSync(outputDir, { recursive: true });

  const evaluatedPath = path.join(outputDir, `${scenario}-evaluated-schema.json`);
  const parsedPath = path.join(outputDir, `${scenario}-parsed-schema.json`);
  const sortedPath = path.join(outputDir, `${scenario}-sorted-evaluations.json`);

  const resultJson = JSONStringify(evaluatedSchema, null, 2);
  fs.writeFileSync(evaluatedPath, resultJson);

  const schemaValue = evaluatedSchema?.["$params"] ?? {};
  fs.writeFileSync(parsedPath, JSONStringify(schemaValue, null, 2));
  fs.writeFileSync(sortedPath, resultJson);

  console.log("\x1b[32m✅ Results saved:\x1b[0m");
  console.log(`  - ${evaluatedPath}`);
  console.log(`  - ${parsedPath}`);
  console.log(`  - ${sortedPath}`);
  console.log();

  // Compare results if comparison file exists
  let differenceCount = 0;
  if (compareData !== null) {
    const actualOthers: Record<string, any> =
      evaluatedSchema?.["$params"]?.others ?? {};
    const expectedOthers: Record<string, any> =
      (compareData as any)?.others ?? {};
    const diffs = findDifferences(actualOthers, expectedOthers, "$");
    differenceCount = diffs.length;

    if (differenceCount > 0) {
      console.log(
        `\x1b[33m⚠️  Comparison: Results differ from baseline (${differenceCount} difference(s)):\x1b[0m`
      );
      for (const diff of diffs) {
        console.log(`  - ${diff}`);
      }
      console.log();
    } else {
      console.log("\x1b[32m✅ Comparison: Results match baseline\x1b[0m");
      console.log();
    }
  }

  return {
    success: true,
    totalMs,
    parsingMs,
    evaluationMs,
    scenario,
    evaluatedSchema,
    differenceCount,
  };
}

// =============================================================================
// wop_flag → false dependents test
//
// Mirrors the Rust test `test_zpp_wop_flag_false_clears_rider_wop_fields`.
// Verifies that flipping wop_flag to false at the product_benefit level causes
// all rider wop fields (wop_flag, wop_rider_benefit, wop_rider_premi) to be
// cleared in the evaluateDependents result for every rider item.
//
// Expected result shape for each rider field:
//   { $ref: "...riders.N.wop_flag", $readonly: true, value: null }   — re-evaluate pass
//   or: { $ref: "...riders.N.wop_flag", clear: true }                — dependents queue
// =============================================================================
async function runWopFlagDependentsTest(projectRoot: string): Promise<void> {
  console.log("==================================================");
  console.log("\x1b[36m🧪 Test: wop_flag=false clears ZLOB rider wop fields\x1b[0m");
  console.log("==================================================");

  const samplesPath = path.join(projectRoot, "samples");
  const schemaJson = fs.readFileSync(path.join(samplesPath, "zpp.json"), "utf8");
  const dataJson = fs.readFileSync(path.join(samplesPath, "zpp-data.json"), "utf8");

  const schema = JSONParse(schemaJson);
  const data = JSONParse(dataJson);

  // Sanity check — fixture must start with wop_flag = true
  if (data?.illustration?.product_benefit?.wop_flag !== true) {
    throw new Error("Fixture must start with wop_flag=true");
  }

  const je = new JSONEval({ schema, data, context: {} });
  await je.init();

  // Initial evaluation
  const evalStart = performance.now();
  await je.evaluateOnly({ data, context: {} });
  console.log(`  ⚡ Initial eval: ${(performance.now() - evalStart).toFixed(3)}ms`);

  // Warm per-item caches for all three riders
  const riders: any[] = data?.illustration?.product_benefit?.riders ?? [];
  for (let idx = 0; idx < riders.length; idx++) {
    const subformData = { riders: riders[idx] };
    await je.evaluateSubform({
      subformPath: `riders.${idx}`,
      data: subformData,
      context: {},
    });
    console.log(`  🔄 Warmed rider[${idx}] subform cache`);
  }

  // Flip wop_flag to false
  const updatedData = JSONParse(dataJson);
  updatedData.illustration.product_benefit.wop_flag = false;
  console.log("  🔁 wop_flag set to false — calling evaluateDependents...");

  const depsStart = performance.now();
  const deps = await je.evaluateDependents({
    changedPaths: ["illustration.product_benefit.wop_flag"],
    data: updatedData,
    context: {},
    reEvaluate: true,
    includeSubforms: true,
  });
  console.log(`  ⚡ evaluateDependents: ${(performance.now() - depsStart).toFixed(3)}ms`);
  console.log(`  📦 Total dep changes: ${deps.length}`);
  console.log();

  // Validate that each rider's wop fields are cleared
  const wopFields = ["wop_flag", "wop_rider_benefit", "wop_rider_premi"] as const;
  let passed = 0;
  let failed = 0;

  for (let idx = 0; idx < riders.length; idx++) {
    for (const field of wopFields) {
      const refPath = `illustration.product_benefit.riders.${idx}.${field}`;
      const match = deps.find((d: any) => {
        const isRef = d["$ref"] === refPath;
        const isCleared = d["clear"] === true || d["value"] === null;
        return isRef && isCleared;
      });

      if (match) {
        console.log(`  \x1b[32m✅ riders[${idx}].${field} → ${JSONStringify(match)}\x1b[0m`);
        passed++;
      } else {
        const found = deps.find((d: any) => d["$ref"] === refPath);
        console.log(
          `  \x1b[31m❌ riders[${idx}].${field} — not cleared. Found: ${found ? JSONStringify(found) : "not in deps"}\x1b[0m`
        );
        failed++;
      }
    }
  }

  console.log();
  if (failed === 0) {
    console.log(`\x1b[32m✅ wop_flag test passed (${passed}/${passed + failed} assertions)\x1b[0m`);
  } else {
    console.log(
      `\x1b[31m❌ wop_flag test FAILED: ${failed} assertion(s) failed, ${passed} passed\x1b[0m`
    );
    console.log();
    console.log("Full dep list (riders only):");
    console.log(
      deps
        .filter((d: any) => (d["$ref"] ?? "").includes("riders"))
        .map((d: any) => `  ${JSONStringify(d)}`)
        .join("\n")
    );
    throw new Error(`wop_flag dependents test failed: ${failed} assertion(s) failed`);
  }
}

async function main(): Promise<void> {
  console.log("🚀 JSON Eval RS - Node.js WASM Benchmark");
  console.log(`📦 WASM version: ${version()}`);
  console.log();

  const scenario = process.argv[2] ?? "zcc";
  console.log(`📋 Scenario: '${scenario}'`);
  console.log();

  const projectRoot = findProjectRoot();
  if (!projectRoot) {
    console.error("\x1b[31m❌ Error: Could not find project root (Cargo.toml not found)\x1b[0m");
    process.exit(1);
  }

  console.log(`📁 Project Root: ${projectRoot}`);
  console.log();

  try {
    const result = await runNodeBenchmark(scenario, projectRoot);

    console.log("==================================================");
    console.log("\x1b[33m📊 Benchmark Summary\x1b[0m");
    console.log("==================================================");
    console.log("🟦 Node.js (WASM):");
    console.log(`  Total:    ${result.totalMs.toFixed(3)}ms`);
    console.log(`  - Parse:  ${result.parsingMs.toFixed(3)}ms`);
    console.log(`  - Eval:   ${result.evaluationMs.toFixed(3)}ms`);

    if (result.differenceCount > 0) {
      console.log(
        `\x1b[33m  ⚠️  ${result.differenceCount} differences from baseline\x1b[0m`
      );
    }
    console.log();

    // Memory usage
    const mem = process.memoryUsage();
    console.log("💾 Memory Usage:");
    console.log(`  Heap Used:  ${(mem.heapUsed / 1024 / 1024).toFixed(2)} MB`);
    console.log(`  RSS:        ${(mem.rss / 1024 / 1024).toFixed(2)} MB`);
    console.log();

    // Run wop_flag dependents test only when the zpp scenario is available
    if (
      fs.existsSync(path.join(projectRoot, "samples", "zpp.json")) &&
      fs.existsSync(path.join(projectRoot, "samples", "zpp-data.json"))
    ) {
      await runWopFlagDependentsTest(projectRoot);
      console.log();
    }

    console.log("\x1b[32m✅ Benchmark completed successfully!\x1b[0m");
  } catch (err: any) {
    console.error(`\x1b[31m❌ Error: ${err.message}\x1b[0m`);
    if (err.stack) {
      console.error(err.stack);
    }
    process.exit(1);
  }
}

main();
