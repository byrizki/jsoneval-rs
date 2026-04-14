# bindings/csharp/JsonEvalRs.Main.cs

[← Back to Module](../modules/bindings-csharp/MODULE.md) | [← Back to INDEX](../INDEX.md)

## Overview

- **Lines:** 1272
- **Language:** CSharp
- **Symbols:** 130
- **Public symbols:** 31

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
| 438 | method | GetEvaluatedSchema | pub | `public JObject GetEvaluatedSchema(...)` |
| 440 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 442 | method | ProcessResult | (private) | `return ProcessResult(...)` |
| 450 | method | GetEvaluatedSchemaMsgpack | pub | `public byte[] GetEvaluatedSchemaMsgpack(...)` |
| 452 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 454 | method | ProcessResultAsBytes | (private) | `return ProcessResultAsBytes(...)` |
| 461 | method | GetSchemaValue | pub | `public JObject GetSchemaValue(...)` |
| 463 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 465 | method | ProcessResult | (private) | `return ProcessResult(...)` |
| 472 | method | GetSchemaValueArray | pub | `public JArray GetSchemaValueArray(...)` |
| 474 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 476 | method | ProcessResultAsArray | (private) | `return ProcessResultAsArray(...)` |
| 483 | method | GetSchemaValueObject | pub | `public JObject GetSchemaValueObject(...)` |
| 485 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 487 | method | ProcessResult | (private) | `return ProcessResult(...)` |
| 495 | method | GetEvaluatedSchemaWithoutParams | pub | `public JObject GetEvaluatedSchemaWithoutParams(...` |
| 497 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 499 | method | ProcessResult | (private) | `return ProcessResult(...)` |
| 510 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 513 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 558 | method | GetEvaluatedSchemaByPaths | pub | `public JToken GetEvaluatedSchemaByPaths(...)` |
| 560 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 563 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 585 | method | InvalidOperationException | (private) | `throw new InvalidOperationException(...)` |
| 617 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 620 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 662 | method | GetSchemaByPath | (private) | `return GetSchemaByPath(...)` |
| 672 | method | GetSchemaByPaths | pub | `public JToken GetSchemaByPaths(...)` |
| 674 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 677 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 699 | method | InvalidOperationException | (private) | `throw new InvalidOperationException(...)` |
| 730 | method | ReloadSchema | pub | `public void ReloadSchema(...)` |
| 732 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 735 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 754 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 765 | method | ReloadSchemaMsgpack | pub | `public void ReloadSchemaMsgpack(...)` |
| 767 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 770 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 795 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 811 | method | ReloadSchemaFromCache | pub | `public void ReloadSchemaFromCache(...)` |
| 813 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 816 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 835 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 843 | method | SetTimezoneOffset | pub | `public void SetTimezoneOffset(...)` |
| 845 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 857 | method | ResolveLayout | pub | `public void ResolveLayout(...)` |
| 859 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 873 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 886 | method | CompileAndRunLogic | pub | `public JToken CompileAndRunLogic(...)` |
| 888 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 891 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 918 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 922 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 926 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 947 | method | CompileLogic | pub | `public ulong CompileLogic(...)` |
| 949 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 952 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 964 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 978 | method | RunLogic | pub | `public JToken RunLogic(...)` |
| 980 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 1001 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1005 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1009 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1031 | method | ValidatePaths | pub | `public ValidationResult ValidatePaths(...)` |
| 1033 | method | ThrowIfDisposed | (private) | `ThrowIfDisposed(...)` |
| 1036 | method | ArgumentNullException | (private) | `throw new ArgumentNullException(...)` |
| 1050 | method | ValidatePathsOnly | pub | `public ValidationResult ValidatePathsOnly(...)` |
| 1052 | method | ValidatePaths | (private) | `return ValidatePaths(...)` |
| 1056 | method | ProcessResult | (private) | `private JObject ProcessResult(...)` |
| 1071 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1075 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1079 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1093 | method | ProcessResultAsArray | (private) | `private JArray ProcessResultAsArray(...)` |
| 1109 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1113 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1117 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1131 | method | ProcessResultAsString | (private) | `private string ProcessResultAsString(...)` |
| 1147 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1182 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1186 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1190 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1205 | method | ProcessResultAsBytes | (private) | `private byte[] ProcessResultAsBytes(...)` |
| 1221 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1225 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1229 | method | JsonEvalException | (private) | `throw new JsonEvalException(...)` |
| 1242 | method | ThrowIfDisposed | (private) | `private void ThrowIfDisposed(...)` |
| 1246 | method | ObjectDisposedException | (private) | `throw new ObjectDisposedException(...)` |
| 1252 | method | Dispose | pub | `public void Dispose(...)` |
| 1269 | method | Dispose | (private) | `Dispose(...)` |

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

**Line:** 438 | **Kind:** method

### `GetEvaluatedSchemaMsgpack`

```
public byte[] GetEvaluatedSchemaMsgpack(...)
```

**Line:** 450 | **Kind:** method

### `GetSchemaValue`

```
public JObject GetSchemaValue(...)
```

**Line:** 461 | **Kind:** method

### `GetSchemaValueArray`

```
public JArray GetSchemaValueArray(...)
```

**Line:** 472 | **Kind:** method

### `GetSchemaValueObject`

```
public JObject GetSchemaValueObject(...)
```

**Line:** 483 | **Kind:** method

### `GetEvaluatedSchemaWithoutParams`

```
public JObject GetEvaluatedSchemaWithoutParams(...)
```

**Line:** 495 | **Kind:** method

### `GetEvaluatedSchemaByPaths`

```
public JToken GetEvaluatedSchemaByPaths(...)
```

**Line:** 558 | **Kind:** method

### `GetSchemaByPaths`

```
public JToken GetSchemaByPaths(...)
```

**Line:** 672 | **Kind:** method

### `ReloadSchema`

```
public void ReloadSchema(...)
```

**Line:** 730 | **Kind:** method

### `ReloadSchemaMsgpack`

```
public void ReloadSchemaMsgpack(...)
```

**Line:** 765 | **Kind:** method

### `ReloadSchemaFromCache`

```
public void ReloadSchemaFromCache(...)
```

**Line:** 811 | **Kind:** method

### `SetTimezoneOffset`

```
public void SetTimezoneOffset(...)
```

**Line:** 843 | **Kind:** method

### `ResolveLayout`

```
public void ResolveLayout(...)
```

**Line:** 857 | **Kind:** method

### `CompileAndRunLogic`

```
public JToken CompileAndRunLogic(...)
```

**Line:** 886 | **Kind:** method

### `CompileLogic`

```
public ulong CompileLogic(...)
```

**Line:** 947 | **Kind:** method

### `RunLogic`

```
public JToken RunLogic(...)
```

**Line:** 978 | **Kind:** method

### `ValidatePaths`

```
public ValidationResult ValidatePaths(...)
```

**Line:** 1031 | **Kind:** method

### `ValidatePathsOnly`

```
public ValidationResult ValidatePathsOnly(...)
```

**Line:** 1050 | **Kind:** method

### `Dispose`

```
public void Dispose(...)
```

**Line:** 1252 | **Kind:** method

