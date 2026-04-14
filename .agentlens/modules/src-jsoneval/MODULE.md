# Module: src/jsoneval

[← Back to INDEX](../../INDEX.md)

**Type:** rust | **Files:** 20

**Entry point:** `src/jsoneval/mod.rs`

## Files

| File | Lines | Large |
| ---- | ----- | ----- |
| `src/jsoneval/cancellation.rs` | 62 |  |
| `src/jsoneval/core.rs` | 630 | 📊 |
| `src/jsoneval/dependents.rs` | 1596 | 📊 |
| `src/jsoneval/eval_cache.rs` | 682 | 📊 |
| `src/jsoneval/eval_data.rs` | 351 |  |
| `src/jsoneval/evaluate.rs` | 860 | 📊 |
| `src/jsoneval/getters.rs` | 596 | 📊 |
| `src/jsoneval/json_parser.rs` | 33 |  |
| `src/jsoneval/layout.rs` | 415 |  |
| `src/jsoneval/logic.rs` | 123 |  |
| `src/jsoneval/mod.rs` | 72 |  |
| `src/jsoneval/parsed_schema.rs` | 193 |  |
| `src/jsoneval/parsed_schema_cache.rs` | 243 |  |
| `src/jsoneval/path_utils.rs` | 408 |  |
| `src/jsoneval/static_arrays.rs` | 39 |  |
| `src/jsoneval/subform_methods.rs` | 860 | 📊 |
| `src/jsoneval/table_evaluate.rs` | 568 | 📊 |
| `src/jsoneval/table_metadata.rs` | 83 |  |
| `src/jsoneval/types.rs` | 50 |  |
| `src/jsoneval/validation.rs` | 601 | 📊 |

## Documentation

- [outline.md](outline.md) - Symbol maps for large files
- [imports.md](imports.md) - Dependencies

---

| High 🔴 | Medium 🟡 | Low 🟢 |
| 0 | 0 | 3 |

## 🟢 Low Priority

### `NOTE` (src/jsoneval/eval_data.rs:192)

> Caller must manually increment version after mutation

### `NOTE` (src/jsoneval/evaluate.rs:550)

> bump_params_version / bump_data_version for table results

### `NOTE` (src/jsoneval/logic.rs:114)

> If data is a primitive value, context cannot be merged
