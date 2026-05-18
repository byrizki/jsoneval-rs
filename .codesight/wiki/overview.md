# jsoneval-rs — Overview

> **Navigation aid.** This article shows WHERE things live (routes, models, files). Read actual source files before implementing new features or making changes.

**jsoneval-rs** is a mixed project built with aspnet, nuxt, organized as a monorepo.

**Workspaces:** `@json-eval-rs/common` (`bindings/common`), `csharp` (`bindings/csharp`), `json-eval-rs-react-native-monorepo` (`bindings/react-native`), `json-eval-rs-web-monorepo` (`bindings/web`), `documentation` (`docs`), `products` (`products`)

## Scale

25 library files · 4 middleware layers · 3 environment variables

**Libraries:** 25 files — see [libraries.md](./libraries.md)

## High-Impact Files

Changes to these files have the widest blast radius across the codebase:

- `products/apps/riplay-viewer/src/config/products.js` — imported by **7** files
- `products/apps/riplay-viewer/src/store.js` — imported by **4** files
- `examples/common.rs` — imported by **3** files
- `bindings/react-native/examples/rncli/App.tsx` — imported by **2** files
- `bindings/web/packages/node/pkg/json_eval_rs.js` — imported by **2** files
- `bindings/web/packages/bundler/pkg/json_eval_rs_bg.js` — imported by **2** files

## Required Environment Variables

- `AUTH_TOKEN` — `products/scripts/download.ts`
- `DOCEVAL_API_URL` — `products/scripts/download.ts`
- `MZPRO_API_URL` — `products/scripts/download.ts`

---
_Back to [index.md](./index.md) · Generated 2026-05-18_