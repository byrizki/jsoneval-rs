# bindings/react-native/packages/react-native/cpp/json-eval-bridge.cpp

[← Back to Module](../modules/root/MODULE.md) | [← Back to INDEX](../INDEX.md)

## Overview

- **Lines:** 1466
- **Language:** Cpp
- **Symbols:** 50
- **Public symbols:** 5

## Symbol Table

| Line | Kind | Name | Visibility | Signature |
| ---- | ---- | ---- | ---------- | --------- |
| 19 | method | json_eval_new | (internal) | `JSONEvalHandle* json_eval_new(const char* schem...` |
| 21 | method | json_eval_new_from_msgpack | (internal) | `JSONEvalHandle* json_eval_new_from_msgpack(cons...` |
| 22 | method | json_eval_evaluate | (internal) | `FFIResult json_eval_evaluate(JSONEvalHandle* ha...` |
| 23 | method | json_eval_get_evaluated_schema_msgpack | (internal) | `FFIResult json_eval_get_evaluated_schema_msgpac...` |
| 24 | method | json_eval_validate | (internal) | `FFIResult json_eval_validate(JSONEvalHandle* ha...` |
| 25 | method | json_eval_evaluate_dependents | (internal) | `FFIResult json_eval_evaluate_dependents(JSONEva...` |
| 26 | method | json_eval_get_evaluated_schema | (internal) | `FFIResult json_eval_get_evaluated_schema(JSONEv...` |
| 27 | method | json_eval_get_schema_value | (internal) | `FFIResult json_eval_get_schema_value(JSONEvalHa...` |
| 28 | method | json_eval_get_schema_value_array | (internal) | `FFIResult json_eval_get_schema_value_array(JSON...` |
| 29 | method | json_eval_get_schema_value_object | (internal) | `FFIResult json_eval_get_schema_value_object(JSO...` |
| 30 | method | json_eval_get_evaluated_schema_without_params | (internal) | `FFIResult json_eval_get_evaluated_schema_withou...` |
| 31 | method | json_eval_get_evaluated_schema_by_path | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 32 | method | json_eval_get_evaluated_schema_by_paths | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 33 | method | json_eval_get_schema_by_path | (internal) | `FFIResult json_eval_get_schema_by_path(JSONEval...` |
| 34 | method | json_eval_get_schema_by_paths | (internal) | `FFIResult json_eval_get_schema_by_paths(JSONEva...` |
| 35 | method | json_eval_resolve_layout | (internal) | `FFIResult json_eval_resolve_layout(JSONEvalHand...` |
| 36 | method | json_eval_compile_and_run_logic | (internal) | `FFIResult json_eval_compile_and_run_logic(JSONE...` |
| 37 | method | json_eval_compile_logic | (internal) | `uint64_t json_eval_compile_logic(JSONEvalHandle...` |
| 38 | method | json_eval_run_logic | (internal) | `FFIResult json_eval_run_logic(JSONEvalHandle* h...` |
| 39 | method | json_eval_reload_schema | (internal) | `FFIResult json_eval_reload_schema(JSONEvalHandl...` |
| 40 | method | json_eval_reload_schema_msgpack | (internal) | `FFIResult json_eval_reload_schema_msgpack(JSONE...` |
| 41 | method | json_eval_reload_schema_from_cache | (internal) | `FFIResult json_eval_reload_schema_from_cache(JS...` |
| 42 | method | json_eval_new_from_cache | (internal) | `JSONEvalHandle* json_eval_new_from_cache(const ...` |
| 43 | method | json_eval_validate_paths | (internal) | `FFIResult json_eval_validate_paths(JSONEvalHand...` |
| 44 | method | json_eval_evaluate_logic_pure | (internal) | `FFIResult json_eval_evaluate_logic_pure(const c...` |
| 47 | method | json_eval_evaluate_subform | (internal) | `FFIResult json_eval_evaluate_subform(JSONEvalHa...` |
| 48 | method | json_eval_validate_subform | (internal) | `FFIResult json_eval_validate_subform(JSONEvalHa...` |
| 49 | method | json_eval_evaluate_dependents_subform | (internal) | `FFIResult json_eval_evaluate_dependents_subform...` |
| 50 | method | json_eval_resolve_layout_subform | (internal) | `FFIResult json_eval_resolve_layout_subform(JSON...` |
| 51 | method | json_eval_get_evaluated_schema_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_subfor...` |
| 52 | method | json_eval_get_schema_value_subform | (internal) | `FFIResult json_eval_get_schema_value_subform(JS...` |
| 53 | method | json_eval_get_schema_value_array_subform | (internal) | `FFIResult json_eval_get_schema_value_array_subf...` |
| 54 | method | json_eval_get_schema_value_object_subform | (internal) | `FFIResult json_eval_get_schema_value_object_sub...` |
| 55 | method | json_eval_get_evaluated_schema_without_params_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_withou...` |
| 56 | method | json_eval_get_evaluated_schema_by_path_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 57 | method | json_eval_get_evaluated_schema_by_paths_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 58 | method | json_eval_get_schema_by_path_subform | (internal) | `FFIResult json_eval_get_schema_by_path_subform(...` |
| 59 | method | json_eval_get_schema_by_paths_subform | (internal) | `FFIResult json_eval_get_schema_by_paths_subform...` |
| 60 | method | json_eval_get_subform_paths | (internal) | `FFIResult json_eval_get_subform_paths(JSONEvalH...` |
| 61 | method | json_eval_has_subform | (internal) | `FFIResult json_eval_has_subform(JSONEvalHandle*...` |
| 62 | method | json_eval_set_timezone_offset | (internal) | `void json_eval_set_timezone_offset(JSONEvalHand...` |
| 64 | method | json_eval_free | (internal) | `void json_eval_free(JSONEvalHandle* handle)` |
| 66 | method | json_eval_cancel | (internal) | `void json_eval_cancel(JSONEvalHandle* handle)` |
| 67 | method | json_eval_free_result | (internal) | `void json_eval_free_result(FFIResult result)` |
| 69 | method | json_eval_free_string | (internal) | `void json_eval_free_string(char* ptr)` |
| 72 | mod | jsoneval | pub | `namespace jsoneval` |
| 242 | method | JsonEvalBridge::compileLogic | pub | `uint64_t JsonEvalBridge::compileLogic(   const ...` |
| 1411 | method | JsonEvalBridge::dispose | pub | `void JsonEvalBridge::dispose(const std::string&...` |
| 1437 | method | JsonEvalBridge::setTimezoneOffset | pub | `void JsonEvalBridge::setTimezoneOffset(   const...` |
| 1452 | method | JsonEvalBridge::cancel | pub | `void JsonEvalBridge::cancel(const std::string& ...` |

## Public API

### `jsoneval`

```
namespace jsoneval
```

**Line:** 72 | **Kind:** mod

### `JsonEvalBridge::compileLogic`

```
uint64_t JsonEvalBridge::compileLogic(
    const std::string& handleId,
    const std::string& logicStr
)
```

**Line:** 242 | **Kind:** method

### `JsonEvalBridge::dispose`

```
void JsonEvalBridge::dispose(const std::string& handleId)
```

**Line:** 1411 | **Kind:** method

### `JsonEvalBridge::setTimezoneOffset`

```
void JsonEvalBridge::setTimezoneOffset(
    const std::string& handleId,
    int32_t offsetMinutes
)
```

**Line:** 1437 | **Kind:** method

### `JsonEvalBridge::cancel`

```
void JsonEvalBridge::cancel(const std::string& handle)
```

**Line:** 1452 | **Kind:** method

