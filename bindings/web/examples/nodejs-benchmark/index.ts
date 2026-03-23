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
  const parsingMs = performance.now() - parseStart;
  console.log(`  📝 Parse (new): ${parsingMs.toFixed(3)}ms`);

  // Benchmark 2: Evaluation
  const evalStart = performance.now();
  await je.evaluateOnly({ data, context: {} });
  const evaluationMs = performance.now() - evalStart;
  console.log(`  ⚡ Eval: ${evaluationMs.toFixed(3)}ms`);

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
    console.log(
      `  Heap Used:  ${(mem.heapUsed / 1024 / 1024).toFixed(2)} MB`
    );
    console.log(
      `  RSS:        ${(mem.rss / 1024 / 1024).toFixed(2)} MB`
    );
    console.log();

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
