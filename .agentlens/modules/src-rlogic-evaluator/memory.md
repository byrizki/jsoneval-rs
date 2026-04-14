# Memory

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

## Summary

| High 🔴 | Medium 🟡 | Low 🟢 |
| 13 | 0 | 0 |

## 🔴 High Priority

### `SAFETY` (src/rlogic/evaluator/array_lookup.rs:23)

> single-threaded (eval_lock), UnsafeCell access

### `SAFETY` (src/rlogic/evaluator/array_lookup.rs:27)

> local_rows outlives this call (table_evaluate_inner scope)

### `SAFETY` (src/rlogic/evaluator/helpers.rs:107)

> single-threaded (eval_lock held during this scope), UnsafeCell

### `SAFETY` (src/rlogic/evaluator/helpers.rs:112)

> local_rows outlives this evaluation frame

### `SAFETY` (src/rlogic/evaluator/mod.rs:27)

> /// `rows` is a raw pointer to `local_rows` on the stack of `evaluate_table_inner`.

### `SAFETY` (src/rlogic/evaluator/mod.rs:40)

> table evaluation is protected by eval_lock (single-threaded access).

### `SAFETY` (src/rlogic/evaluator/mod.rs:54)

> single-threaded (eval_lock), no concurrent access

### `SAFETY` (src/rlogic/evaluator/mod.rs:95)

> /// `rows` must outlive the returned guard. The guard MUST be dropped before

### `SAFETY` (src/rlogic/evaluator/mod.rs:103)

> single-threaded (eval_lock held by caller)

### `SAFETY` (src/rlogic/evaluator/mod.rs:116)

> single-threaded (eval_lock held by caller)

### `SAFETY` (src/rlogic/evaluator/mod.rs:126)

> single-threaded (eval_lock held by caller)

### `SAFETY` (src/rlogic/evaluator/mod.rs:849)

> single-threaded (eval_lock), UnsafeCell

### `SAFETY` (src/rlogic/evaluator/mod.rs:853)

> local_rows outlives this evaluation frame

