# src/rlogic/compiled.rs

[← Back to Module](../modules/src-rlogic/MODULE.md) | [← Back to INDEX](../INDEX.md)

## Overview

- **Lines:** 1868
- **Language:** Rust
- **Symbols:** 38
- **Public symbols:** 13

## Symbol Table

| Line | Kind | Name | Visibility | Signature |
| ---- | ---- | ---- | ---------- | --------- |
| 8 | struct | LogicId | pub | - |
| 12 | enum | CompiledLogic | pub | - |
| 177 | fn | compile | pub | `pub fn compile(logic: &Value) -> Result<Self, S...` |
| 200 | fn | compile_operator | (private) | `fn compile_operator(op: &str, args: &Value) -> ...` |
| 1028 | fn | compile_binary | (private) | `fn compile_binary<F>(args: &Value, f: F) -> Res...` |
| 1049 | fn | preprocess_table_condition | (private) | `fn preprocess_table_condition(value: &Value) ->...` |
| 1127 | fn | is_simple_ref | pub | `pub fn is_simple_ref(&self) -> bool {` |
| 1135 | fn | referenced_vars | pub | `pub fn referenced_vars(&self) -> Vec<String> {` |
| 1144 | fn | flatten_and | (private) | `fn flatten_and(items: Vec<CompiledLogic>) -> Ve...` |
| 1159 | fn | flatten_or | (private) | `fn flatten_or(items: Vec<CompiledLogic>) -> Vec...` |
| 1174 | fn | flatten_add | (private) | `fn flatten_add(items: Vec<CompiledLogic>) -> Ve...` |
| 1189 | fn | flatten_multiply | (private) | `fn flatten_multiply(items: Vec<CompiledLogic>) ...` |
| 1205 | fn | flatten_cat | (private) | `fn flatten_cat(items: Vec<CompiledLogic>) -> Ve...` |
| 1221 | fn | has_forward_reference | pub | `pub fn has_forward_reference(&self) -> bool {` |
| 1226 | fn | check_forward_reference | (private) | `fn check_forward_reference(&self) -> bool {` |
| 1394 | fn | contains_iteration_plus_positive | (private) | `fn contains_iteration_plus_positive(&self) -> b...` |
| 1417 | fn | normalize_ref_path | (private) | `fn normalize_ref_path(path: &str) -> String {` |
| 1445 | fn | collect_vars | pub | `pub fn collect_vars(&self, vars: &mut Vec<Strin...` |
| 1633 | struct | CompiledLogicStore | pub | - |
| 1640 | fn | new | pub | `pub fn new() -> Self {` |
| 1653 | fn | compile | pub | `pub fn compile(&mut self, logic: &Value) -> Res...` |
| 1676 | fn | get | pub | `pub fn get(&self, id: &LogicId) -> Option<&Comp...` |
| 1681 | fn | remove | pub | `pub fn remove(&mut self, id: &LogicId) -> Optio...` |
| 1687 | fn | get_dependencies | pub | `pub fn get_dependencies(&self, id: &LogicId) ->...` |
| 1693 | fn | default | (private) | `fn default() -> Self {` |
| 1703 | fn | is_ok | (private) | `fn is_ok(json_value: serde_json::Value) -> bool {` |
| 1708 | fn | test_compile_literals | (private) | `fn test_compile_literals() {` |
| 1734 | fn | test_compile_variables | (private) | `fn test_compile_variables() {` |
| 1742 | fn | test_compile_logical | (private) | `fn test_compile_logical() {` |
| 1757 | fn | test_compile_comparison | (private) | `fn test_compile_comparison() {` |
| 1772 | fn | test_compile_arithmetic | (private) | `fn test_compile_arithmetic() {` |
| 1785 | fn | test_compile_array_ops | (private) | `fn test_compile_array_ops() {` |
| 1800 | fn | test_compile_string_ops | (private) | `fn test_compile_string_ops() {` |
| 1813 | fn | test_compile_math_ops | (private) | `fn test_compile_math_ops() {` |
| 1826 | fn | test_compile_date_ops | (private) | `fn test_compile_date_ops() {` |
| 1836 | fn | test_compile_table_ops | (private) | `fn test_compile_table_ops() {` |
| 1855 | fn | test_compile_util_ops | (private) | `fn test_compile_util_ops() {` |
| 1864 | fn | test_compile_unknown | (private) | `fn test_compile_unknown() {` |

## Public API

### `compile`

```
pub fn compile(logic: &Value) -> Result<Self, String> {
```

**Line:** 177 | **Kind:** fn

### `is_simple_ref`

```
pub fn is_simple_ref(&self) -> bool {
```

**Line:** 1127 | **Kind:** fn

### `referenced_vars`

```
pub fn referenced_vars(&self) -> Vec<String> {
```

**Line:** 1135 | **Kind:** fn

### `has_forward_reference`

```
pub fn has_forward_reference(&self) -> bool {
```

**Line:** 1221 | **Kind:** fn

### `collect_vars`

```
pub fn collect_vars(&self, vars: &mut Vec<String>) {
```

**Line:** 1445 | **Kind:** fn

### `new`

```
pub fn new() -> Self {
```

**Line:** 1640 | **Kind:** fn

### `compile`

```
pub fn compile(&mut self, logic: &Value) -> Result<LogicId, String> {
```

**Line:** 1653 | **Kind:** fn

### `get`

```
pub fn get(&self, id: &LogicId) -> Option<&CompiledLogic> {
```

**Line:** 1676 | **Kind:** fn

### `remove`

```
pub fn remove(&mut self, id: &LogicId) -> Option<CompiledLogic> {
```

**Line:** 1681 | **Kind:** fn

### `get_dependencies`

```
pub fn get_dependencies(&self, id: &LogicId) -> Option<&[String]> {
```

**Line:** 1687 | **Kind:** fn

