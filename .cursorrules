# Project Context

This is a mixed project using aspnet, nuxt.
It is a monorepo with workspaces: @json-eval-rs/common (bindings/common), csharp (bindings/csharp), json-eval-rs-react-native-monorepo (bindings/react-native), json-eval-rs-web-monorepo (bindings/web), documentation (docs), products (products).

Middleware includes: custom, validation.

High-impact files (most imported, changes here affect many other files):
- products/apps/riplay-viewer/src/config/products.js (imported by 7 files)
- products/apps/riplay-viewer/src/store.js (imported by 4 files)
- examples/common.rs (imported by 3 files)
- bindings/react-native/examples/rncli/App.tsx (imported by 2 files)
- bindings/web/packages/node/pkg/json_eval_rs.js (imported by 2 files)
- bindings/web/packages/bundler/pkg/json_eval_rs_bg.js (imported by 2 files)
- products/apps/riplay-viewer/src/services/evaluator.js (imported by 2 files)
- products/apps/riplay-viewer/src/services/assets.js (imported by 2 files)

Required environment variables (no defaults):
- AUTH_TOKEN (products/scripts/download.ts)
- DOCEVAL_API_URL (products/scripts/download.ts)
- MZPRO_API_URL (products/scripts/download.ts)

Read .codesight/wiki/index.md for orientation (WHERE things live). Then read actual source files before implementing. Wiki articles are navigation aids, not implementation guides.
Read .codesight/CODESIGHT.md for the complete AI context map including all routes, schema, components, libraries, config, middleware, and dependency graph.
