# jsoneval-rs — Overview

> **Navigation aid.** This article shows WHERE things live (routes, models, files). Read actual source files before implementing new features or making changes.

**jsoneval-rs** is a mixed project built with aspnet, nuxt, organized as a monorepo.

**Workspaces:** `csharp` (`bindings/csharp`), `json-eval-rs-npm-monorepo` (`bindings/npm`), `documentation` (`docs`), `products` (`products`)

## Scale

11 UI components · 16 library files · 2 middleware layers

**UI:** 11 components (react) — see [ui.md](./ui.md)

**Libraries:** 16 files — see [libraries.md](./libraries.md)

## High-Impact Files

Changes to these files have the widest blast radius across the codebase:

- `examples/common.rs` — imported by **3** files
- `bindings/npm/packages/node/pkg/json_eval_rs.js` — imported by **2** files
- `bindings/npm/examples/rncli/App.tsx` — imported by **2** files
- `bindings/npm/examples/rncli/src/screens/FormValidationScreen.tsx` — imported by **1** files
- `bindings/npm/examples/rncli/src/screens/DependentFieldsScreen.tsx` — imported by **1** files
- `bindings/npm/packages/bundler/pkg/json_eval_rs_bg.js` — imported by **1** files

---
_Back to [index.md](./index.md) · Generated 2026-07-20_