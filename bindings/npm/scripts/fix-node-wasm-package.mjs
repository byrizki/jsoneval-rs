#!/usr/bin/env node
/**
 * wasm-pack's `nodejs` target emits CommonJS glue (`pkg/json_eval_rs.js`).
 * The enclosing @json-eval-rs/node package is ESM, so its `type: module`
 * would otherwise make Node parse that generated glue as ESM.
 *
 * Run immediately after every node-target wasm-pack build.
 */
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const pkgDir = path.resolve(scriptDir, "../packages/node/pkg");
const packageJsonPath = path.join(pkgDir, "package.json");
const gitignorePath = path.join(pkgDir, ".gitignore");

writeFileSync(packageJsonPath, `${JSON.stringify({ type: "commonjs" }, null, 2)}\n`);
// wasm-pack writes `*`, which makes npm omit every runtime asset even when the
// parent package declares `files: ["pkg"]`. Keep build output untracked while
// explicitly allowing required files into npm tarballs.
writeFileSync(gitignorePath, `*\n!json_eval_rs.js\n!json_eval_rs_bg.wasm\n!*.d.ts\n!package.json\n`);
console.log(`Wrote CommonJS module boundary: ${packageJsonPath}`);
