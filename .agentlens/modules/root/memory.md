# Memory

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

## Summary

| High 🔴 | Medium 🟡 | Low 🟢 |
| 0 | 0 | 14 |

## 🟢 Low Priority

### `NOTE` (bindings/react-native/packages/react-native/android/src/main/cpp/json-eval-rn.cpp:15)

> GetStringUTFChars provides a pinned pointer (minimal copy), but we must

### `NOTE` (bindings/react-native/packages/react-native/android/src/main/cpp/json-eval-rn.cpp:29)

> NewStringUTF copies C string to create Java String object (unavoidable)

### `NOTE` (bindings/react-native/packages/react-native/android/src/main/cpp/json-eval-rn.cpp:74)

> Template functions must have C++ linkage, not C linkage

### `NOTE` (tests/array_tests.rs:359)

> merge flattens one level, so nested maps create nested arrays

### `NOTE` (tests/hidden_filtering_test.rs:261)

> JSONEval uses layout_paths to find root layouts.

### `NOTE` (tests/logical_operator_crosscheck.rs:61)

> In JSON logic, empty array as argument needs to be wrapped in array

### `NOTE` (tests/selective_eval_subforms.rs:81)

> evaluate(paths) evaluates EXACTLY the paths provided. It does not auto-trigger dependents.

### `NOTE` (tests/table_tests.rs:241)

> Multiple conditions are ANDed together automatically

### `NOTE` (tests/test_evaluate_dependents_features.rs:119)

> replace_data_and_context replaces top-level objects, so we must provide the full object or use a different update method.

### `NOTE` (tests/test_evaluate_dependents_features.rs:147)

> implementation details might vary. The key requirement is DATA PRESERVATION.

### `NOTE` (tests/test_evaluate_others.rs:700)

> paths are normalized to JSON pointer format during parsing

### `NOTE` (tests/test_subforms.rs:172)

> The exact value would depend on how the evaluation is stored

### `NOTE` (tests/test_subforms.rs:239)

> Validation behavior depends on schema structure in subform

### `NOTE` (tests/wasm_validation_tests.rs:29)

> JSONEvalWasm::new args: (schema, context, data)

