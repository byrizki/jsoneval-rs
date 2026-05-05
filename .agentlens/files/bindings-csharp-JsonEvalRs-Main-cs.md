# bindings/csharp/JsonEvalRs.Main.cs

[← Back to Module](../modules/bindings-csharp/MODULE.md) | [← Back to INDEX](../INDEX.md)

## Overview

- **Lines:** 1289
- **Language:** CSharp
- **Symbols:** 136
- **Public symbols:** 33

## Symbol Table

| Line | Kind | Name | Visibility | Signature |
| ---- | ---- | ---- | ---------- | --------- |
| 6 | mod | JsonEvalRs | pub | `namespace JsonEvalRs` |
| 20 | class | JSONEval | pub | `public partial class JSONEval` |
| 28 | const | Version | pub | - |
| 49 | method | EvaluateLogic | pub | `public static JToken EvaluateLogic(...)` |
| 52 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 72 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 106 | method | FromCache | pub | `public static JSONEval FromCache(...)` |
| 109 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 118 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 157 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 161 | method | JSONEval | (private) | `return new JSONEval(...)` |
| 171 | method | FromMsgpack | pub | `public static JSONEval FromMsgpack(...)` |
| 173 | method | JSONEval | (private) | `return new JSONEval(...)` |
| 182 | method | JSONEval | (private) | `public JSONEval(...)` |
| 185 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 194 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 232 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 242 | method | JSONEval | (private) | `public JSONEval(...)` |
| 245 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 254 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 289 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 297 | method | JSONEval | (private) | `private JSONEval(...)` |
| 306 | method | Cancel | pub | `public void Cancel(...)` |
| 308 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 319 | method | EvaluateOnly | pub | `public void EvaluateOnly(...)` |
| 321 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 324 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 347 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 355 | method | Evaluate | pub | `public void Evaluate(...)` |
| 357 | method | EvaluateOnly | (private) | `EvaluateOnly(...)` |
| 366 | method | Validate | pub | `public ValidationResult Validate(...)` |
| 368 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 371 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 389 | method | EvaluateDependents | pub | `public JArray EvaluateDependents(...)` |
| 392 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 395 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 404 | method | ProcessResultAsArray | (private) | `return ProcessResultAsArray(...)` |
| 415 | method | EvaluateDependentsString | pub | `public string EvaluateDependentsString(...)` |
| 418 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 421 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 430 | method | ProcessResultAsString | (private) | `return ProcessResultAsString(...)` |
| 437 | method | GetEvaluatedSchema | pub | `public JObject GetEvaluatedSchema(...)` |
| 439 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 441 | method | ProcessResult | (private) | `return ProcessResult(...)` |
| 448 | method | GetResolvedLayout | pub | `public JArray GetResolvedLayout(...)` |
| 450 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 452 | method | ProcessResultAsArray | (private) | `return ProcessResultAsArray(...)` |
| 459 | method | GetEvaluatedSchemaResolved | pub | `public JObject GetEvaluatedSchemaResolved(...)` |
| 461 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 463 | method | ProcessResult | (private) | `return ProcessResult(...)` |
| 470 | method | GetEvaluatedSchemaMsgpack | pub | `public byte[] GetEvaluatedSchemaMsgpack(...)` |
| 472 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 474 | method | ProcessResultAsBytes | (private) | `return ProcessResultAsBytes(...)` |
| 481 | method | GetSchemaValue | pub | `public JObject GetSchemaValue(...)` |
| 483 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 485 | method | ProcessResult | (private) | `return ProcessResult(...)` |
| 492 | method | GetSchemaValueArray | pub | `public JArray GetSchemaValueArray(...)` |
| 494 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 496 | method | ProcessResultAsArray | (private) | `return ProcessResultAsArray(...)` |
| 503 | method | GetSchemaValueObject | pub | `public JObject GetSchemaValueObject(...)` |
| 505 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 507 | method | ProcessResult | (private) | `return ProcessResult(...)` |
| 514 | method | GetEvaluatedSchemaWithoutParams | pub | `public JObject GetEvaluatedSchemaWithoutParams(...` |
| 516 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 518 | method | ProcessResult | (private) | `return ProcessResult(...)` |
| 528 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 531 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 575 | method | GetEvaluatedSchemaByPaths | pub | `public JToken GetEvaluatedSchemaByPaths(...)` |
| 577 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 580 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 602 | method | InvalidOperationException | (private) | `throw new InvalidOperationException(...)` |
| 634 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 637 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 679 | method | GetSchemaByPath | (private) | `return GetSchemaByPath(...)` |
| 689 | method | GetSchemaByPaths | pub | `public JToken GetSchemaByPaths(...)` |
| 691 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 694 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 716 | method | InvalidOperationException | (private) | `throw new InvalidOperationException(...)` |
| 747 | method | ReloadSchema | pub | `public void ReloadSchema(...)` |
| 749 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 752 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 771 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 782 | method | ReloadSchemaMsgpack | pub | `public void ReloadSchemaMsgpack(...)` |
| 784 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 787 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 812 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 828 | method | ReloadSchemaFromCache | pub | `public void ReloadSchemaFromCache(...)` |
| 830 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 833 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 852 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 860 | method | SetTimezoneOffset | pub | `public void SetTimezoneOffset(...)` |
| 862 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 874 | method | ResolveLayout | pub | `public void ResolveLayout(...)` |
| 876 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 890 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 903 | method | CompileAndRunLogic | pub | `public JToken CompileAndRunLogic(...)` |
| 905 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 908 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 935 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 939 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 943 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 964 | method | CompileLogic | pub | `public ulong CompileLogic(...)` |
| 966 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 969 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 981 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 995 | method | RunLogic | pub | `public JToken RunLogic(...)` |
| 997 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 1018 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1022 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1026 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1048 | method | ValidatePaths | pub | `public ValidationResult ValidatePaths(...)` |
| 1050 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 1053 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 1067 | method | ValidatePathsOnly | pub | `public ValidationResult ValidatePathsOnly(...)` |
| 1069 | method | ValidatePaths | (private) | `return ValidatePaths(...)` |
| 1073 | method | ProcessResult | (private) | `private JObject ProcessResult(...)` |
| 1088 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1092 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1096 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1110 | method | ProcessResultAsArray | (private) | `private JArray ProcessResultAsArray(...)` |
| 1126 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1130 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1134 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1148 | method | ProcessResultAsString | (private) | `private string ProcessResultAsString(...)` |
| 1164 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1199 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1203 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1207 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1222 | method | ProcessResultAsBytes | (private) | `private byte[] ProcessResultAsBytes(...)` |
| 1238 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1242 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1246 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1259 | method | ThrowIfDisposed | (private) | `private void ThrowIfDisposed(...)` |
| 1263 | method | ObjectDisposedException | (private) | `throw new ObjectDisposedException(...)` |
| 1269 | method | Dispose | pub | `public void Dispose(...)` |
| 1286 | method | Dispose | (private) | `Dispose(...)` |

## Public API

### `JsonEvalRs`

```
namespace JsonEvalRs
```

**Line:** 6 | **Kind:** mod

### `JSONEval`

```
public partial class JSONEval
```

**Line:** 20 | **Kind:** class

### `EvaluateLogic`

```
public static JToken EvaluateLogic(...)
```

**Line:** 49 | **Kind:** method

### `FromCache`

```
public static JSONEval FromCache(...)
```

**Line:** 106 | **Kind:** method

### `FromMsgpack`

```
public static JSONEval FromMsgpack(...)
```

**Line:** 171 | **Kind:** method

### `Cancel`

```
public void Cancel(...)
```

**Line:** 306 | **Kind:** method

### `EvaluateOnly`

```
public void EvaluateOnly(...)
```

**Line:** 319 | **Kind:** method

### `Evaluate`

```
public void Evaluate(...)
```

**Line:** 355 | **Kind:** method

### `Validate`

```
public ValidationResult Validate(...)
```

**Line:** 366 | **Kind:** method

### `EvaluateDependents`

```
public JArray EvaluateDependents(...)
```

**Line:** 389 | **Kind:** method

### `EvaluateDependentsString`

```
public string EvaluateDependentsString(...)
```

**Line:** 415 | **Kind:** method

### `GetEvaluatedSchema`

```
public JObject GetEvaluatedSchema(...)
```

**Line:** 437 | **Kind:** method

### `GetResolvedLayout`

```
public JArray GetResolvedLayout(...)
```

**Line:** 448 | **Kind:** method

### `GetEvaluatedSchemaResolved`

```
public JObject GetEvaluatedSchemaResolved(...)
```

**Line:** 459 | **Kind:** method

### `GetEvaluatedSchemaMsgpack`

```
public byte[] GetEvaluatedSchemaMsgpack(...)
```

**Line:** 470 | **Kind:** method

### `GetSchemaValue`

```
public JObject GetSchemaValue(...)
```

**Line:** 481 | **Kind:** method

### `GetSchemaValueArray`

```
public JArray GetSchemaValueArray(...)
```

**Line:** 492 | **Kind:** method

### `GetSchemaValueObject`

```
public JObject GetSchemaValueObject(...)
```

**Line:** 503 | **Kind:** method

### `GetEvaluatedSchemaWithoutParams`

```
public JObject GetEvaluatedSchemaWithoutParams(...)
```

**Line:** 514 | **Kind:** method

### `GetEvaluatedSchemaByPaths`

```
public JToken GetEvaluatedSchemaByPaths(...)
```

**Line:** 575 | **Kind:** method

### `GetSchemaByPaths`

```
public JToken GetSchemaByPaths(...)
```

**Line:** 689 | **Kind:** method

### `ReloadSchema`

```
public void ReloadSchema(...)
```

**Line:** 747 | **Kind:** method

### `ReloadSchemaMsgpack`

```
public void ReloadSchemaMsgpack(...)
```

**Line:** 782 | **Kind:** method

### `ReloadSchemaFromCache`

```
public void ReloadSchemaFromCache(...)
```

**Line:** 828 | **Kind:** method

### `SetTimezoneOffset`

```
public void SetTimezoneOffset(...)
```

**Line:** 860 | **Kind:** method

### `ResolveLayout`

```
public void ResolveLayout(...)
```

**Line:** 874 | **Kind:** method

### `CompileAndRunLogic`

```
public JToken CompileAndRunLogic(...)
```

**Line:** 903 | **Kind:** method

### `CompileLogic`

```
public ulong CompileLogic(...)
```

**Line:** 964 | **Kind:** method

### `RunLogic`

```
public JToken RunLogic(...)
```

**Line:** 995 | **Kind:** method

### `ValidatePaths`

```
public ValidationResult ValidatePaths(...)
```

**Line:** 1048 | **Kind:** method

### `ValidatePathsOnly`

```
public ValidationResult ValidatePathsOnly(...)
```

**Line:** 1067 | **Kind:** method

### `Dispose`

```
public void Dispose(...)
```

**Line:** 1269 | **Kind:** method

