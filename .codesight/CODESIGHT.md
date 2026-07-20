# jsoneval-rs — AI Context Map

> **Stack:** aspnet, nuxt | none | react | mixed
> **Monorepo:** csharp, json-eval-rs-npm-monorepo, documentation, products

> 0 routes | 0 models | 8 components | 14 lib files | 0 env vars | 2 middleware | 0% test coverage
> **Token savings:** this file is ~3,000 tokens. Without it, AI exploration would cost ~13,900 tokens. **Saves ~11,000 tokens per conversation.**
> **Last scanned:** 2026-07-20 04:18 — re-run after significant changes

---

# Components

- **DependentFields** [client] — `bindings/npm/examples/nextjs/components/DependentFields.tsx`
- **FormValidator** [client] — `bindings/npm/examples/nextjs/components/FormValidator.tsx`
- **InsuranceForm** [client] — `bindings/npm/examples/nextjs/components/InsuranceForm.tsx`
- **WorkerExample** [client] — `bindings/npm/examples/nextjs/components/WorkerExample.tsx`
- **App** — props: Component, pageProps — `bindings/npm/examples/nextjs/pages/_app.tsx`
- **Home** — `bindings/npm/examples/nextjs/pages/index.tsx`
- **App** — `bindings/npm/examples/rncli/App.tsx`
- **LINKING_ERROR** — `bindings/npm/packages/react-native/src/index.tsx`

---

# Libraries

- `bindings/csharp/JsonEvalRs.DependencyInjection.cs`
  - class JsonEvalRsServiceCollectionExtensions
  - interface IStartupCacheInitializer
  - class JsonEvalRsApplicationBuilderExtensions
  - function AddJsonEvalRsCache
  - function AddJsonEvalRsLocalCache
  - function Initialize
- `bindings/csharp/JsonEvalRs.Main.cs`
  - class JSONEval
  - function EvaluateLogic
  - function FromCache
  - function FromMsgpack
  - function Cancel
  - function EvaluateOnly
  - _...26 more_
- `bindings/csharp/JsonEvalRs.ParsedCache.cs`
  - class ParsedSchemaCache
  - class ParsedCacheStats
  - function Insert
  - function InsertMsgpack
  - function Contains
  - function Remove
  - _...5 more_
- `bindings/csharp/JsonEvalRs.Shared.cs`
  - class ValidationError
  - class ValidationResult
  - class SchemaValueItem
  - class JsonEvalException
- `bindings/csharp/JsonEvalRs.Subforms.cs`
  - class JSONEval
  - function EvaluateSubform
  - function ValidateSubform
  - function EvaluateDependentsSubform
  - function EvaluateDependentsSubformString
  - function ResolveLayoutSubform
  - _...11 more_
- `bindings/npm/examples/nextjs/hooks/useJSONEvalWorker.ts` — function useJSONEvalWorker: ({...}, context, data, }) => UseJSONEvalWorkerReturn
- `bindings/npm/packages/bundler/pkg/json_eval_rs_bg.js`
  - function getVersion: () => void
  - function init: () => void
  - function version: () => void
  - function __wbg_Error_83742b46f01ce22d: (arg0, arg1) => void
  - function __wbg_String_8564e559799eccda: (arg0, arg1) => void
  - function __wbg___wbindgen_is_string_7ef6b97b02428fae: (arg0) => void
  - _...24 more_
- `bindings/npm/packages/bundler/src/index.ts` — function version: () => string, class JSONEval
- `bindings/npm/packages/common/src/utils.ts`
  - function stringifyValue: (value) => string
  - function parseValue: (value) => any
  - function stringifyOrNull: (value) => string | null
  - function extractErrorMessage: (error) => string
  - function mergeLayoutOverlay: (schema, overlayEntries) => any
  - function resolveEvaluatedLayout: (getSchema) => void
- `bindings/npm/packages/node/src/index.ts` — function version: () => string, class JSONEval
- `bindings/npm/packages/react-native/src/jsi-bridge.ts`
  - function getJSIGlobal: () => JsonEvalJSIGlobal | null
  - function isJSIAvailable: () => boolean
  - interface JsonEvalJSIGlobal
- `bindings/npm/packages/vanilla/pkg/json_eval_rs.js`
  - function getVersion: () => void
  - function init: () => void
  - function version: () => void
  - class JSONEvalWasm
  - class ValidationError
  - class ValidationResult
- `bindings/npm/packages/vanilla/src/index.ts` — function version: () => string, class JSONEval
- `bindings/npm/packages/webcore/src/index.ts` — function getVersion: (wasmModule) => string, class JSONEvalCore

---

# Config

## Config Files

- `Cargo.toml`
- `bindings/npm/examples/nextjs/next.config.js`
- `bindings/npm/examples/nextjs/tailwind.config.js`

---

# Middleware

## custom
- canonical_subform_scope_guardrail — `tests/canonical_subform_scope_guardrail.rs`
- compile_and_run_separate_tests — `tests/compile_and_run_separate_tests.rs`

---

# Dependency Graph

## Most Imported Files (change these carefully)

- `examples/common.rs` — imported by **3** files
- `bindings/npm/packages/node/pkg/json_eval_rs.js` — imported by **2** files
- `bindings/npm/examples/rncli/App.tsx` — imported by **2** files
- `bindings/npm/packages/bundler/pkg/json_eval_rs_bg.js` — imported by **1** files
- `bindings/npm/packages/bundler/pkg/json_eval_rs.js` — imported by **1** files
- `bindings/npm/packages/common/src/utils.ts` — imported by **1** files
- `bindings/npm/packages/common/src/types.ts` — imported by **1** files
- `bindings/npm/packages/react-native/src/jsi-bridge.ts` — imported by **1** files
- `bindings/npm/packages/vanilla/pkg/json_eval_rs.js` — imported by **1** files
- `tests/common.rs` — imported by **1** files

## Import Map (who imports what)

- `examples/common.rs` ← `examples/basic.rs`, `examples/basic_parsed.rs`, `examples/benchmark.rs`
- `bindings/npm/packages/node/pkg/json_eval_rs.js` ← `bindings/npm/examples/nodejs-benchmark/simulate_cache_miss.js`, `bindings/npm/packages/node/src/index.ts`
- `bindings/npm/examples/rncli/App.tsx` ← `bindings/npm/examples/rncli/__tests__/App.test.tsx`, `bindings/npm/examples/rncli/index.js`
- `bindings/npm/packages/bundler/pkg/json_eval_rs_bg.js` ← `bindings/npm/packages/bundler/pkg/json_eval_rs.js`
- `bindings/npm/packages/bundler/pkg/json_eval_rs.js` ← `bindings/npm/packages/bundler/src/index.ts`
- `bindings/npm/packages/common/src/utils.ts` ← `bindings/npm/packages/common/src/index.ts`
- `bindings/npm/packages/common/src/types.ts` ← `bindings/npm/packages/common/src/utils.ts`
- `bindings/npm/packages/react-native/src/jsi-bridge.ts` ← `bindings/npm/packages/react-native/src/index.tsx`
- `bindings/npm/packages/vanilla/pkg/json_eval_rs.js` ← `bindings/npm/packages/vanilla/src/index.ts`
- `tests/common.rs` ← `tests/json_eval_tests.rs`

---

# Test Coverage

> **0%** of routes and models are covered by tests
> 65 test files found

---

# CI/CD Pipelines

## GitHub Actions (3 workflows)

| Workflow | Triggers | Jobs | Deploy | Environments |
|---|---|---|---|---|
| Deploy docs to GitHub Pages | push, workflow_dispatch | 2 | — | github-pages |
| Publish Packages | workflow_dispatch | 7 | — | — |
| Release and Build | push, workflow_dispatch | 13 | — | — |

### Deploy docs to GitHub Pages

> `.github/workflows/deploy-docs.yml`

> Concurrency: `pages`

- **build** on `ubuntu-latest` — 6 steps
  - `actions/checkout@v6`
  - `actions/setup-node@v6`
  - `actions/upload-pages-artifact@v4`
- **deploy** on `ubuntu-latest` — 1 steps (needs: build)
  - `actions/deploy-pages@v5`

### Publish Packages

> `.github/workflows/publish.yml`

> Concurrency: `publish-${{ github.event.inputs.release_tag }}`

- **check-workflow** on `ubuntu-latest` — 1 steps
- **publish-csharp** on `ubuntu-latest` — 4 steps (needs: check-workflow)
  - `actions/checkout@v6`
  - `actions/setup-dotnet@v5`
- **publish-common** on `ubuntu-latest` — 4 steps (needs: check-workflow)
  - `actions/checkout@v6`
  - `actions/setup-node@v6`
- **publish-web** on `ubuntu-latest` — 4 steps (needs: check-workflow, publish-common)
  - `actions/checkout@v6`
  - `actions/setup-node@v6`
- **publish-react-native** on `ubuntu-latest` — 4 steps (needs: check-workflow, publish-common)
  - `actions/checkout@v6`
  - `actions/setup-node@v6`
- **publish-crates-io** on `ubuntu-latest` — 4 steps (needs: check-workflow)
  - `actions/checkout@v6`
  - `actions-rust-lang/setup-rust-toolchain@v1`
- **publish-summary** on `ubuntu-latest` — 1 steps

### Release and Build

> `.github/workflows/release.yml`

- **create-release** on `ubuntu-latest` — 5 steps
  - `actions/checkout@v6`
  - `actions/upload-artifact@v6`
- **check-rn-tests** on `ubuntu-latest` — 2 steps
  - `actions/checkout@v6`
- **build-native** on `${{ matrix.os }}` — 6 steps
  - `actions/checkout@v6`
  - `actions-rust-lang/setup-rust-toolchain@v1`
  - `actions/cache@v5`
  - `actions/upload-artifact@v6`
- **build-csharp** on `ubuntu-latest` — 8 steps (needs: build-native)
  - `actions/checkout@v6`
  - `actions/setup-dotnet@v5`
  - `actions/download-artifact@v7`
  - `actions/upload-artifact@v6`
- **build-web** on `ubuntu-latest` — 20 steps
  - `actions/checkout@v6`
  - `actions-rust-lang/setup-rust-toolchain@v1`
  - `actions/setup-node@v6`
  - `actions/cache@v5`
  - `actions/upload-artifact@v6`
  - `actions/upload-artifact@v6`
- **build-android-jni** on `ubuntu-latest` — 11 steps
  - `actions/checkout@v6`
  - `actions-rust-lang/setup-rust-toolchain@v1`
  - `nttld/setup-ndk@v1`
  - `actions/cache@v5`
  - `actions/upload-artifact@v6`
- **build-ios-xcframework** on `macos-latest` — 4 steps (needs: build-native)
  - `actions/checkout@v6`
  - `actions/download-artifact@v7`
  - `actions/upload-artifact@v6`
- **build-react-native** on `ubuntu-latest` — 12 steps (needs: build-native, build-android-jni, build-ios-xcframework)
  - `actions/checkout@v6`
  - `actions/setup-node@v6`
  - `actions/download-artifact@v7`
  - `actions/download-artifact@v7`
  - `actions/upload-artifact@v6`
- **test** on `ubuntu-latest` — 7 steps
  - `actions/checkout@v6`
  - `actions-rust-lang/setup-rust-toolchain@v1`
  - `actions/cache@v5`
- **test-react-native-android** on `ubuntu-latest` — 13 steps (needs: build-android-jni, build-react-native, check-rn-tests)
  - `actions/checkout@v6`
  - `actions/setup-node@v6`
  - `actions/setup-java@v5`
  - `android-actions/setup-android@v3`
  - `actions/download-artifact@v7`
  - `actions/cache@v5`
- **test-react-native-ios** on `macos-latest` — 11 steps (needs: build-ios-xcframework, build-react-native, check-rn-tests)
  - `actions/checkout@v6`
  - `actions/setup-node@v6`
  - `actions/download-artifact@v7`
  - `actions/cache@v5`
  - `actions/upload-artifact@v6`
- **upload-to-release** on `ubuntu-latest` — 6 steps
  - `actions/checkout@v6`
  - `actions/download-artifact@v7`
  - `softprops/action-gh-release@v2`
- **summary** on `ubuntu-latest` — 1 steps

### Secrets

- `CARGO_REGISTRY_TOKEN`
- `GITHUB_TOKEN`
- `NPM_TOKEN`
- `NUGET_API_KEY`

---
_Source: .github/workflows/deploy-docs.yml, .github/workflows/publish.yml, .github/workflows/release.yml_
_Generated by codesight-cicd-plugin_

---

_Generated by [codesight](https://github.com/Houseofmvps/codesight) — see your codebase clearly_