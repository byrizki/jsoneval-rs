# bindings/react-native/packages/react-native/cpp/json-eval-bridge.cpp

[← Back to Module](../modules/bindings-react-native-packages-react-native-cpp/MODULE.md) | [← Back to INDEX](../INDEX.md)

## Overview

- **Lines:** 1253
- **Language:** Cpp
- **Symbols:** 51
- **Public symbols:** 6

## Symbol Table

| Line | Kind | Name | Visibility | Signature |
| ---- | ---- | ---- | ---------- | --------- |
| 12 | class | SimpleThreadPool | pub | `class SimpleThreadPool` |
| 77 | method | json_eval_new | (internal) | `JSONEvalHandle* json_eval_new(const char* schem...` |
| 79 | method | json_eval_new_from_msgpack | (internal) | `JSONEvalHandle* json_eval_new_from_msgpack(cons...` |
| 80 | method | json_eval_evaluate | (internal) | `FFIResult json_eval_evaluate(JSONEvalHandle* ha...` |
| 81 | method | json_eval_get_evaluated_schema_msgpack | (internal) | `FFIResult json_eval_get_evaluated_schema_msgpac...` |
| 82 | method | json_eval_validate | (internal) | `FFIResult json_eval_validate(JSONEvalHandle* ha...` |
| 83 | method | json_eval_evaluate_dependents | (internal) | `FFIResult json_eval_evaluate_dependents(JSONEva...` |
| 84 | method | json_eval_get_evaluated_schema | (internal) | `FFIResult json_eval_get_evaluated_schema(JSONEv...` |
| 85 | method | json_eval_get_schema_value | (internal) | `FFIResult json_eval_get_schema_value(JSONEvalHa...` |
| 86 | method | json_eval_get_schema_value_array | (internal) | `FFIResult json_eval_get_schema_value_array(JSON...` |
| 87 | method | json_eval_get_schema_value_object | (internal) | `FFIResult json_eval_get_schema_value_object(JSO...` |
| 88 | method | json_eval_get_evaluated_schema_without_params | (internal) | `FFIResult json_eval_get_evaluated_schema_withou...` |
| 89 | method | json_eval_get_evaluated_schema_by_path | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 90 | method | json_eval_get_evaluated_schema_by_paths | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 91 | method | json_eval_get_schema_by_path | (internal) | `FFIResult json_eval_get_schema_by_path(JSONEval...` |
| 92 | method | json_eval_get_schema_by_paths | (internal) | `FFIResult json_eval_get_schema_by_paths(JSONEva...` |
| 93 | method | json_eval_resolve_layout | (internal) | `FFIResult json_eval_resolve_layout(JSONEvalHand...` |
| 94 | method | json_eval_compile_and_run_logic | (internal) | `FFIResult json_eval_compile_and_run_logic(JSONE...` |
| 95 | method | json_eval_compile_logic | (internal) | `uint64_t json_eval_compile_logic(JSONEvalHandle...` |
| 96 | method | json_eval_run_logic | (internal) | `FFIResult json_eval_run_logic(JSONEvalHandle* h...` |
| 97 | method | json_eval_reload_schema | (internal) | `FFIResult json_eval_reload_schema(JSONEvalHandl...` |
| 98 | method | json_eval_reload_schema_msgpack | (internal) | `FFIResult json_eval_reload_schema_msgpack(JSONE...` |
| 99 | method | json_eval_reload_schema_from_cache | (internal) | `FFIResult json_eval_reload_schema_from_cache(JS...` |
| 100 | method | json_eval_new_from_cache | (internal) | `JSONEvalHandle* json_eval_new_from_cache(const ...` |
| 101 | method | json_eval_validate_paths | (internal) | `FFIResult json_eval_validate_paths(JSONEvalHand...` |
| 102 | method | json_eval_evaluate_logic_pure | (internal) | `FFIResult json_eval_evaluate_logic_pure(const c...` |
| 105 | method | json_eval_evaluate_subform | (internal) | `FFIResult json_eval_evaluate_subform(JSONEvalHa...` |
| 106 | method | json_eval_validate_subform | (internal) | `FFIResult json_eval_validate_subform(JSONEvalHa...` |
| 107 | method | json_eval_evaluate_dependents_subform | (internal) | `FFIResult json_eval_evaluate_dependents_subform...` |
| 108 | method | json_eval_resolve_layout_subform | (internal) | `FFIResult json_eval_resolve_layout_subform(JSON...` |
| 109 | method | json_eval_get_evaluated_schema_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_subfor...` |
| 110 | method | json_eval_get_schema_value_subform | (internal) | `FFIResult json_eval_get_schema_value_subform(JS...` |
| 111 | method | json_eval_get_schema_value_array_subform | (internal) | `FFIResult json_eval_get_schema_value_array_subf...` |
| 112 | method | json_eval_get_schema_value_object_subform | (internal) | `FFIResult json_eval_get_schema_value_object_sub...` |
| 113 | method | json_eval_get_evaluated_schema_without_params_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_withou...` |
| 114 | method | json_eval_get_evaluated_schema_by_path_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 115 | method | json_eval_get_evaluated_schema_by_paths_subform | (internal) | `FFIResult json_eval_get_evaluated_schema_by_pat...` |
| 116 | method | json_eval_get_schema_by_path_subform | (internal) | `FFIResult json_eval_get_schema_by_path_subform(...` |
| 117 | method | json_eval_get_schema_by_paths_subform | (internal) | `FFIResult json_eval_get_schema_by_paths_subform...` |
| 118 | method | json_eval_get_subform_paths | (internal) | `FFIResult json_eval_get_subform_paths(JSONEvalH...` |
| 119 | method | json_eval_has_subform | (internal) | `FFIResult json_eval_has_subform(JSONEvalHandle*...` |
| 120 | method | json_eval_set_timezone_offset | (internal) | `void json_eval_set_timezone_offset(JSONEvalHand...` |
| 122 | method | json_eval_free | (internal) | `void json_eval_free(JSONEvalHandle* handle)` |
| 124 | method | json_eval_cancel | (internal) | `void json_eval_cancel(JSONEvalHandle* handle)` |
| 125 | method | json_eval_free_result | (internal) | `void json_eval_free_result(FFIResult result)` |
| 127 | method | json_eval_free_string | (internal) | `void json_eval_free_string(char* ptr)` |
| 130 | mod | jsoneval | pub | `namespace jsoneval` |
| 328 | method | JsonEvalBridge::compileLogic | pub | `uint64_t JsonEvalBridge::compileLogic(   const ...` |
| 1202 | method | JsonEvalBridge::dispose | pub | `void JsonEvalBridge::dispose(const std::string&...` |
| 1231 | method | JsonEvalBridge::setTimezoneOffset | pub | `void JsonEvalBridge::setTimezoneOffset(   const...` |
| 1239 | method | JsonEvalBridge::cancel | pub | `void JsonEvalBridge::cancel(const std::string& ...` |

## Public API

### `SimpleThreadPool`

```
class SimpleThreadPool
```

**Line:** 12 | **Kind:** class

### `jsoneval`

```
namespace jsoneval
```

**Line:** 130 | **Kind:** mod

### `JsonEvalBridge::compileLogic`

```
uint64_t JsonEvalBridge::compileLogic(
    const std::string& handleId,
    const std::string& logicStr
)
```

**Line:** 328 | **Kind:** method

### `JsonEvalBridge::dispose`

```
void JsonEvalBridge::dispose(const std::string& handleId)
```

**Line:** 1202 | **Kind:** method

### `JsonEvalBridge::setTimezoneOffset`

```
void JsonEvalBridge::setTimezoneOffset(
    const std::string& handleId,
    int32_t offsetMinutes
)
```

**Line:** 1231 | **Kind:** method

### `JsonEvalBridge::cancel`

```
void JsonEvalBridge::cancel(const std::string& handleId)
```

**Line:** 1239 | **Kind:** method

## Memory Markers

### 🟢 `NOTE` (line 135)

> std::mutex is non-movable in NDK libc++, so we keep two parallel maps

