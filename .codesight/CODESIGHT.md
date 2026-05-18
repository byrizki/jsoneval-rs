# jsoneval-rs — AI Context Map

> **Stack:** aspnet, nuxt | none | react | mixed
> **Monorepo:** csharp, json-eval-rs-npm-monorepo, documentation, products

> 0 routes | 0 models | 11 components | 25 lib files | 3 env vars | 4 middleware | 0% test coverage
> **Token savings:** this file is ~2,900 tokens. Without it, AI exploration would cost ~20,600 tokens. **Saves ~17,700 tokens per conversation.**
> **Last scanned:** 2026-05-18 05:13 — re-run after significant changes

---

# Components

- **DependentFields** [client] — `bindings/npm/examples/nextjs/components/DependentFields.tsx`
- **FormValidator** [client] — `bindings/npm/examples/nextjs/components/FormValidator.tsx`
- **InsuranceForm** [client] — `bindings/npm/examples/nextjs/components/InsuranceForm.tsx`
- **WorkerExample** [client] — `bindings/npm/examples/nextjs/components/WorkerExample.tsx`
- **App** — props: Component, pageProps — `bindings/npm/examples/nextjs/pages/_app.tsx`
- **Home** — `bindings/npm/examples/nextjs/pages/index.tsx`
- **App** — `bindings/npm/examples/rncli/App.tsx`
- **DependentFieldsScreen** — `bindings/npm/examples/rncli/src/screens/DependentFieldsScreen.tsx`
- **FormValidationScreen** — `bindings/npm/examples/rncli/src/screens/FormValidationScreen.tsx`
- **InsuranceFormScreen** — `bindings/npm/examples/rncli/src/screens/InsuranceFormScreen.tsx`
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
  - _...25 more_
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
- `bindings/npm/packages/bundler/src/index.ts` — function version: () => string, class JSONEval
- `bindings/npm/packages/common/src/utils.ts`
  - function stringifyValue: (value) => string
  - function parseValue: (value) => any
  - function stringifyOrNull: (value) => string | null
  - function extractErrorMessage: (error) => string
  - function mergeLayoutOverlay: (schema, overlayEntries) => any
  - function resolveEvaluatedLayout: (getSchema) => void
- `bindings/npm/packages/node/src/index.ts` — function version: () => string, class JSONEval
- `bindings/npm/packages/react-native/lib/module/index.js`
  - function useJSONEval: (options) => void
  - function multiply
  - class JSONEval
- `bindings/npm/packages/react-native/lib/module/jsi-bridge.js` — function getJSIGlobal: () => void, function isJSIAvailable: () => void
- `bindings/npm/packages/react-native/src/jsi-bridge.ts`
  - function getJSIGlobal: () => JsonEvalJSIGlobal | null
  - function isJSIAvailable: () => boolean
  - interface JsonEvalJSIGlobal
- `bindings/npm/packages/vanilla/src/index.ts` — function version: () => string, class JSONEval
- `bindings/npm/packages/webcore/src/index.ts` — function getVersion: (wasmModule) => string, class JSONEvalCore
- `bindings/web/packages/bundler/pkg/json_eval_rs_bg.js`
  - function __wbg_set_wasm: (val) => void
  - function getVersion: () => void
  - function version: () => void
  - function init: () => void
  - function __wbg_Error_e17e777aac105295: (arg0, arg1) => void
  - function __wbg_String_8f0eb39a4a4c2f66: (arg0, arg1) => void
  - _...25 more_
- `bindings/web/packages/vanilla/pkg/json_eval_rs.js`
  - function getVersion: () => void
  - function init: () => void
  - function version: () => void
  - class JSONEvalWasm
  - class ValidationError
  - class ValidationResult
- `generate_parity_table.py`
  - function get_matches: (file_paths, pattern, group)
  - function snake_to_camel: (s)
  - function check: (method, method_set)
- `products/apps/riplay-viewer/src/app.js` — function initApp: () => void
- `products/apps/riplay-viewer/src/services/assets.js`
  - function getTemplate: (env, folder, file) => void
  - function getStyle: (env, folder, file) => void
  - function getSchema: (env, schemaKey) => void
  - function getSample: (productCode) => void
  - const TEMPLATES
  - const STYLES
  - _...2 more_
- `products/apps/riplay-viewer/src/services/evaluator.js` — function evaluateSchema: (schema, formData, context) => void, function disposeEvaluator: () => void
- `products/apps/riplay-viewer/src/services/renderer.js` — function renderTemplate: (templateStr, context) => void, function buildIframeDocument: (renderedHtml, templateCss, pagedCss, polyfillRaw) => void
- `products/apps/riplay-viewer/src/store.js`
  - function subscribe: (fn) => void
  - function getState: () => void
  - function setState: (patch) => void
  - function buildDefaultFormData: (product) => void
- `products/apps/riplay-viewer/src/ui/editor.js` — function mountEditor: (onRenderRequest) => void, function syncEditorFromState: (textarea) => void
- `products/apps/riplay-viewer/src/ui/preview.js`
  - function mountPreview: () => void
  - function showPreviewLoading: (phase) => void
  - function showPreviewContent: (html, badgeLabel) => void
  - function showPreviewError: (message) => void
- `products/apps/riplay-viewer/src/ui/sidebar.js` — function mountSidebar: (onRenderRequest) => void, function updateSidebarStatus: (status, message) => void

---

# Config

## Environment Variables

- `AUTH_TOKEN` **required** — products/scripts/download.ts
- `DOCEVAL_API_URL` **required** — products/scripts/download.ts
- `MZPRO_API_URL` **required** — products/scripts/download.ts

## Config Files

- `Cargo.toml`
- `bindings/npm/examples/nextjs/next.config.js`
- `bindings/npm/examples/nextjs/tailwind.config.js`
- `products/apps/riplay-viewer/vite.config.js`

---

# Middleware

## custom
- 08.migrate-legacy-jsoneval — `docs/content/en/03.advance-guide/08.migrate-legacy-jsoneval.md`
- 08.migrate-legacy-jsoneval — `docs/content/id/03.advance-guide/08.migrate-legacy-jsoneval.md`
- compile_and_run_separate_tests — `tests/compile_and_run_separate_tests.rs`

## validation
- generate_parity_table — `generate_parity_table.py`

---

# Dependency Graph

## Most Imported Files (change these carefully)

- `products/apps/riplay-viewer/src/config/products.js` — imported by **7** files
- `products/apps/riplay-viewer/src/store.js` — imported by **4** files
- `examples/common.rs` — imported by **3** files
- `bindings/npm/examples/rncli/App.tsx` — imported by **2** files
- `bindings/web/packages/bundler/pkg/json_eval_rs_bg.js` — imported by **2** files
- `products/apps/riplay-viewer/src/services/evaluator.js` — imported by **2** files
- `products/apps/riplay-viewer/src/services/assets.js` — imported by **2** files
- `bindings/npm/examples/rncli/src/screens/FormValidationScreen.tsx` — imported by **1** files
- `bindings/npm/examples/rncli/src/screens/DependentFieldsScreen.tsx` — imported by **1** files
- `bindings/npm/packages/common/src/utils.ts` — imported by **1** files
- `bindings/npm/packages/common/src/types.ts` — imported by **1** files
- `bindings/npm/packages/react-native/lib/commonjs/jsi-bridge.js` — imported by **1** files
- `bindings/npm/packages/react-native/lib/module/jsi-bridge.js` — imported by **1** files
- `bindings/npm/packages/react-native/src/jsi-bridge.ts` — imported by **1** files
- `products/apps/riplay-viewer/src/services/renderer.js` — imported by **1** files
- `products/apps/riplay-viewer/src/ui/sidebar.js` — imported by **1** files
- `products/apps/riplay-viewer/src/ui/editor.js` — imported by **1** files
- `products/apps/riplay-viewer/src/ui/preview.js` — imported by **1** files
- `products/apps/riplay-viewer/src/app.js` — imported by **1** files
- `tests/common.rs` — imported by **1** files

## Import Map (who imports what)

- `products/apps/riplay-viewer/src/config/products.js` ← `products/apps/riplay-viewer/src/app.js`, `products/apps/riplay-viewer/src/store.js`, `products/apps/riplay-viewer/src/store.js`, `products/apps/riplay-viewer/src/store.js`, `products/apps/riplay-viewer/src/ui/sidebar.js` +2 more
- `products/apps/riplay-viewer/src/store.js` ← `products/apps/riplay-viewer/src/app.js`, `products/apps/riplay-viewer/src/ui/editor.js`, `products/apps/riplay-viewer/src/ui/preview.js`, `products/apps/riplay-viewer/src/ui/sidebar.js`
- `examples/common.rs` ← `examples/basic.rs`, `examples/basic_parsed.rs`, `examples/benchmark.rs`
- `bindings/npm/examples/rncli/App.tsx` ← `bindings/npm/examples/rncli/__tests__/App.test.tsx`, `bindings/npm/examples/rncli/index.js`
- `bindings/web/packages/bundler/pkg/json_eval_rs_bg.js` ← `bindings/web/packages/bundler/pkg/json_eval_rs.js`, `bindings/web/packages/bundler/pkg/json_eval_rs.js`
- `products/apps/riplay-viewer/src/services/evaluator.js` ← `products/apps/riplay-viewer/src/app.js`, `products/apps/riplay-viewer/src/ui/sidebar.js`
- `products/apps/riplay-viewer/src/services/assets.js` ← `products/apps/riplay-viewer/src/app.js`, `products/apps/riplay-viewer/src/ui/sidebar.js`
- `bindings/npm/examples/rncli/src/screens/FormValidationScreen.tsx` ← `bindings/npm/examples/rncli/App.tsx`
- `bindings/npm/examples/rncli/src/screens/DependentFieldsScreen.tsx` ← `bindings/npm/examples/rncli/App.tsx`
- `bindings/npm/packages/common/src/utils.ts` ← `bindings/npm/packages/common/src/index.ts`

---

# Test Coverage

> **0%** of routes and models are covered by tests
> 49 test files found

---

_Generated by [codesight](https://github.com/Houseofmvps/codesight) — see your codebase clearly_