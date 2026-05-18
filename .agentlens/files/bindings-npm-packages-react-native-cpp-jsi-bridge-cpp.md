# bindings/npm/packages/react-native/cpp/jsi-bridge.cpp

[← Back to Module](../modules/bindings-npm-packages-react-native-cpp/MODULE.md) | [← Back to INDEX](../INDEX.md)

## Overview

- **Lines:** 1080
- **Language:** Cpp
- **Symbols:** 78
- **Public symbols:** 5

## Symbol Table

| Line | Kind | Name | Visibility | Signature |
| ---- | ---- | ---- | ---------- | --------- |
| 7 | method | json_eval_new | (internal) | `JSONEvalHandle* json_eval_new(const char* schem...` |
| 8 | method | json_eval_new_from_msgpack | (internal) | `JSONEvalHandle* json_eval_new_from_msgpack(cons...` |
| 9 | method | json_eval_new_from_cache | (internal) | `JSONEvalHandle* json_eval_new_from_cache(const ...` |
| 10 | method | json_eval_evaluate | (internal) | `FFIResult json_eval_evaluate(JSONEvalHandle* ha...` |
| 11 | method | json_eval_get_evaluated_schema | (internal) | `FFIResult json_eval_get_evaluated_schema(JSONEv...` |
| 12 | method | json_eval_get_schema_value | (internal) | `FFIResult json_eval_get_schema_value(JSONEvalHa...` |
| 13 | method | json_eval_get_schema_value_array | (internal) | `FFIResult json_eval_get_schema_value_array(JSON...` |
| 14 | method | json_eval_get_schema_value_object | (internal) | `FFIResult json_eval_get_schema_value_object(JSO...` |
| 15 | method | json_eval_validate | (internal) | `FFIResult json_eval_validate(JSONEvalHandle* ha...` |
| 16 | method | json_eval_validate_paths | (internal) | `FFIResult json_eval_validate_paths(JSONEvalHand...` |
| 17 | method | json_eval_evaluate_dependents | (internal) | `FFIResult json_eval_evaluate_dependents(JSONEva...` |
| 18 | method | json_eval_get_evaluated_schema_by_path | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 19 | method | json_eval_get_evaluated_schema_by_paths | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 20 | method | json_eval_get_schema_by_path | (internal) | `FFIResult json_eval_get_schema_by_path(JSONEval...` |
| 21 | method | json_eval_get_schema_by_paths | (internal) | `FFIResult json_eval_get_schema_by_paths(JSONEva...` |
| 22 | method | json_eval_get_evaluated_schema_without_params | (internal) | `FFIResult json_eval_get_evaluated_schema_withou...` |
| 23 | method | json_eval_resolve_layout | (internal) | `FFIResult json_eval_resolve_layout(JSONEvalHand...` |
| 24 | method | json_eval_compile_and_run_logic | (internal) | `FFIResult json_eval_compile_and_run_logic(JSONE...` |
| 25 | method | json_eval_compile_logic | (internal) | `uint64_t json_eval_compile_logic(JSONEvalHandle...` |
| 26 | method | json_eval_run_logic | (internal) | `FFIResult json_eval_run_logic(JSONEvalHandle* h...` |
| 27 | method | json_eval_reload_schema | (internal) | `FFIResult json_eval_reload_schema(JSONEvalHandl...` |
| 28 | method | json_eval_reload_schema_msgpack | (internal) | `FFIResult json_eval_reload_schema_msgpack(JSONE...` |
| 29 | method | json_eval_reload_schema_from_cache | (internal) | `FFIResult json_eval_reload_schema_from_cache(JS...` |
| 30 | method | json_eval_set_timezone_offset | (internal) | `void json_eval_set_timezone_offset(JSONEvalHand...` |
| 31 | method | json_eval_free | (internal) | `void json_eval_free(JSONEvalHandle* handle)` |
| 32 | method | json_eval_free_result | (internal) | `void json_eval_free_result(FFIResult result)` |
| 34 | method | json_eval_free_string | (internal) | `void json_eval_free_string(char* ptr)` |
| 37 | method | json_eval_evaluate_subform | (internal) | `FFIResult json_eval_evaluate_subform(JSONEvalHa...` |
| 38 | method | json_eval_validate_subform | (internal) | `FFIResult json_eval_validate_subform(JSONEvalHa...` |
| 39 | method | json_eval_evaluate_dependents_subform | (internal) | `FFIResult json_eval_evaluate_dependents_subform...` |
| 40 | method | json_eval_resolve_layout_subform | (internal) | `FFIResult json_eval_resolve_layout_subform(JSON...` |
| 41 | method | json_eval_get_evaluated_schema_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_subfor...` |
| 42 | method | json_eval_get_schema_value_subform | (internal) | `FFIResult json_eval_get_schema_value_subform(JS...` |
| 43 | method | json_eval_get_schema_value_array_subform | (internal) | `FFIResult json_eval_get_schema_value_array_subf...` |
| 44 | method | json_eval_get_schema_value_object_subform | (internal) | `FFIResult json_eval_get_schema_value_object_sub...` |
| 45 | method | json_eval_get_evaluated_schema_without_params_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_withou...` |
| 46 | method | json_eval_get_evaluated_schema_by_path_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 47 | method | json_eval_get_evaluated_schema_by_paths_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 48 | method | json_eval_get_schema_by_path_subform | (internal) | `FFIResult json_eval_get_schema_by_path_subform(...` |
| 49 | method | json_eval_get_schema_by_paths_subform | (internal) | `FFIResult json_eval_get_schema_by_paths_subform...` |
| 50 | method | json_eval_get_subform_paths | (internal) | `FFIResult json_eval_get_subform_paths(JSONEvalH...` |
| 51 | method | json_eval_has_subform | (internal) | `FFIResult json_eval_has_subform(JSONEvalHandle*...` |
| 52 | method | json_eval_evaluate_logic_pure | (internal) | `FFIResult json_eval_evaluate_logic_pure(const c...` |
| 55 | mod | jsoneval | pub | `namespace jsoneval` |
| 71 | fn | storeHandle | (private) | `static void storeHandle(const std::string& id, ...` |
| 92 | method | JsonEvalJSI::install | pub | `bool JsonEvalJSI::install(jsi::Runtime& runtime)` |
| 109 | method | JsonEvalJSI::checkResult | pub | `void JsonEvalJSI::checkResult(jsi::Runtime& run...` |
| 117 | method | JsonEvalJSI::checkArgCount | pub | `void JsonEvalJSI::checkArgCount(jsi::Runtime& r...` |
| 159 | method | fn | (internal) | `return fn(rt, args, count)` |
| 304 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, schemaResult)` |
| 324 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 346 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 372 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 387 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 400 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 413 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 426 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 442 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 459 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 474 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 490 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 505 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 544 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 584 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 731 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 812 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 837 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 871 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 886 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 901 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 916 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 932 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 949 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 967 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 983 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 1000 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 1014 | method | ffiResultToJsiBuffer | (internal) | `return ffiResultToJsiBuffer(rt, result)` |
| 1044 | method | JsonEvalJSI::set | pub | `void JsonEvalJSI::set(jsi::Runtime& runtime, co...` |

## Public API

### `jsoneval`

```
namespace jsoneval
```

**Line:** 55 | **Kind:** mod

### `JsonEvalJSI::install`

```
bool JsonEvalJSI::install(jsi::Runtime& runtime)
```

**Line:** 92 | **Kind:** method

### `JsonEvalJSI::checkResult`

```
void JsonEvalJSI::checkResult(jsi::Runtime& runtime, const FFIResult& result)
```

**Line:** 109 | **Kind:** method

### `JsonEvalJSI::checkArgCount`

```
void JsonEvalJSI::checkArgCount(jsi::Runtime& runtime, size_t actual, size_t expected)
```

**Line:** 117 | **Kind:** method

### `JsonEvalJSI::set`

```
void JsonEvalJSI::set(jsi::Runtime& runtime, const jsi::PropNameID& name, const jsi::Value& value)
```

**Line:** 1044 | **Kind:** method

