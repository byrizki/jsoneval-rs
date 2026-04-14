# bindings/web/packages/core/src/index.ts

[← Back to Module](../modules/bindings-web-packages-core-src/MODULE.md) | [← Back to INDEX](../INDEX.md)

## Overview

- **Lines:** 1247
- **Language:** TypeScript
- **Symbols:** 27
- **Public symbols:** 27

## Symbol Table

| Line | Kind | Name | Visibility | Signature |
| ---- | ---- | ---- | ---------- | --------- |
| 15 | fn | getVersion | pub | `export function getVersion(wasmModule: any): st...` |
| 25 | interface | SchemaValueItem | pub | - |
| 47 | interface | ValidationError | pub | - |
| 67 | interface | ValidationResult | pub | - |
| 77 | interface | DependentChange | pub | - |
| 95 | interface | JSONEvalOptions | pub | - |
| 124 | interface | ValidateOptions | pub | - |
| 134 | interface | EvaluateOptions | pub | - |
| 146 | interface | EvaluateDependentsOptions | pub | - |
| 162 | interface | GetEvaluatedSchemaOptions | pub | - |
| 170 | interface | GetValueByPathOptions | pub | - |
| 180 | interface | GetValueByPathsOptions | pub | - |
| 192 | interface | GetSchemaByPathOptions | pub | - |
| 200 | interface | GetSchemaByPathsOptions | pub | - |
| 210 | interface | ReloadSchemaOptions | pub | - |
| 222 | interface | EvaluateSubformOptions | pub | - |
| 236 | interface | ValidateSubformOptions | pub | - |
| 248 | interface | EvaluateDependentsSubformOptions | pub | - |
| 266 | interface | ResolveLayoutSubformOptions | pub | - |
| 276 | interface | GetEvaluatedSchemaSubformOptions | pub | - |
| 286 | interface | GetSchemaValueSubformOptions | pub | - |
| 294 | interface | GetEvaluatedSchemaByPathSubformOptions | pub | - |
| 306 | interface | GetEvaluatedSchemaByPathsSubformOptions | pub | - |
| 320 | interface | GetSchemaByPathSubformOptions | pub | - |
| 330 | interface | GetSchemaByPathsSubformOptions | pub | - |
| 342 | interface | CompileAndRunLogicOptions | pub | - |
| 371 | class | JSONEvalCore | pub | - |

## Public API

### `getVersion`

```
export function getVersion(wasmModule: any): string {
```

**Line:** 15 | **Kind:** fn

## Memory Markers

### 🟢 `NOTE` (line 862)

> schema is not updated as we don't have access to it from the cache key

### 🟢 `NOTE` (line 957)

> You must call .free() on the returned object when done.

