# Libraries

> **Navigation aid.** Library inventory extracted via AST. Read the source files listed here before modifying exported functions.

**25 library files** across 3 modules

## Bindings (16 files)

- `bindings/csharp/JsonEvalRs.Main.cs` — JSONEval, EvaluateLogic, FromCache, FromMsgpack, Cancel, EvaluateOnly, …
- `bindings/web/packages/bundler/pkg/json_eval_rs_bg.js` — __wbg_set_wasm, getVersion, version, init, __wbg_Error_e17e777aac105295, __wbg_String_8f0eb39a4a4c2f66, …
- `bindings/csharp/JsonEvalRs.Subforms.cs` — JSONEval, EvaluateSubform, ValidateSubform, EvaluateDependentsSubform, EvaluateDependentsSubformString, ResolveLayoutSubform, …
- `bindings/csharp/JsonEvalRs.ParsedCache.cs` — ParsedSchemaCache, ParsedCacheStats, Insert, InsertMsgpack, Contains, Remove, …
- `bindings/common/src/utils.ts` — stringifyValue, parseValue, stringifyOrNull, extractErrorMessage, mergeLayoutOverlay, resolveEvaluatedLayout
- `bindings/csharp/JsonEvalRs.DependencyInjection.cs` — JsonEvalRsServiceCollectionExtensions, IStartupCacheInitializer, JsonEvalRsApplicationBuilderExtensions, AddJsonEvalRsCache, AddJsonEvalRsLocalCache, Initialize
- `bindings/web/packages/vanilla/pkg/json_eval_rs.js` — getVersion, init, version, JSONEvalWasm, ValidationError, ValidationResult
- `bindings/csharp/JsonEvalRs.Shared.cs` — ValidationError, ValidationResult, SchemaValueItem, JsonEvalException
- `bindings/react-native/packages/react-native/lib/module/index.js` — useJSONEval, multiply, JSONEval
- `bindings/react-native/packages/react-native/src/jsi-bridge.ts` — getJSIGlobal, isJSIAvailable, JsonEvalJSIGlobal
- `bindings/react-native/packages/react-native/lib/module/jsi-bridge.js` — getJSIGlobal, isJSIAvailable
- `bindings/web/packages/bundler/src/index.ts` — version, JSONEval
- `bindings/web/packages/core/src/index.ts` — getVersion, JSONEvalCore
- `bindings/web/packages/node/src/index.ts` — version, JSONEval
- `bindings/web/packages/vanilla/src/index.ts` — version, JSONEval
- `bindings/web/examples/nextjs/hooks/useJSONEvalWorker.ts` — useJSONEvalWorker

## Products (8 files)

- `products/apps/riplay-viewer/src/services/assets.js` — getTemplate, getStyle, getSchema, getSample, TEMPLATES, STYLES, …
- `products/apps/riplay-viewer/src/store.js` — subscribe, getState, setState, buildDefaultFormData
- `products/apps/riplay-viewer/src/ui/preview.js` — mountPreview, showPreviewLoading, showPreviewContent, showPreviewError
- `products/apps/riplay-viewer/src/services/evaluator.js` — evaluateSchema, disposeEvaluator
- `products/apps/riplay-viewer/src/services/renderer.js` — renderTemplate, buildIframeDocument
- `products/apps/riplay-viewer/src/ui/editor.js` — mountEditor, syncEditorFromState
- `products/apps/riplay-viewer/src/ui/sidebar.js` — mountSidebar, updateSidebarStatus
- `products/apps/riplay-viewer/src/app.js` — initApp

## Generate_parity_table.py (1 files)

- `generate_parity_table.py` — get_matches, snake_to_camel, check

---
_Back to [overview.md](./overview.md)_