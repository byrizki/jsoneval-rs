DONE_WITH_CONCERNS

Summary:
- Clarified Rust crate root docs and module map in `src/lib.rs`.
- Added focused module docs to `src/jsoneval/mod.rs`, `src/parse_schema/mod.rs`, `src/rlogic/mod.rs`, `src/topo_sort/mod.rs`, and `src/utils/mod.rs`.
- Cleaned stale commented module notes in `src/lib.rs` without changing public re-exports.
- Marked `examples/common/mod.rs` as shared Rust example setup/plumbing only.
- Rewrote `examples/README.md` as Rust-only example guide, removed stale `basic_msgpack` references, documented current Rust examples, and pointed binding examples to planned canonical `bindings/npm/examples/` and `bindings/csharp/examples/` locations.

Changed files:
- `examples/README.md`
- `examples/common/mod.rs`
- `src/jsoneval/mod.rs`
- `src/lib.rs`
- `src/parse_schema/mod.rs`
- `src/rlogic/mod.rs`
- `src/topo_sort/mod.rs`
- `src/utils/mod.rs`

Validation:
- `cargo fmt` passed.
- `git diff --check` passed.
- `cargo test --examples` passed: `cargo test: 0 passed (5 suites, 0.00s)`.
- `cargo test --quiet` failed with baseline FFI parity compile errors:

```text
cargo test: 14 errors, 0 warnings (0 crates)
═══════════════════════════════════════
error[E0425]: cannot find function `json_eval_new` in this scope
  --> tests/test_ffi_parity.rs:45:22
   |
45 |         let handle = json_eval_new(schema_str.as_ptr(), std::ptr::null(), std::ptr::null());
   |                      ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_evaluate` in this scope
  --> tests/test_ffi_parity.rs:53:22
   |
53 |         let result = json_eval_evaluate(
   |                      ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_free_result` in this scope
  --> tests/test_ffi_parity.rs:68:9
   |
68 |         json_eval_free_result(result);
   |         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_get_evaluated_schema_resolved` in this scope
  --> tests/test_ffi_parity.rs:71:22
   |
71 |         let result = json_eval_get_evaluated_schema_resolved(handle);
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_free_result` in this scope
  --> tests/test_ffi_parity.rs:92:9
   |
92 |         json_eval_free_result(result);
   |         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_get_resolved_layout` in this scope
  --> tests/test_ffi_parity.rs:95:22
   |
95 |         let result = json_eval_get_resolved_layout(handle);
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_free_result` in this scope
  --> tests/test_ffi_parity.rs:98:9
   |
98 |         json_eval_free_result(result);
   |         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_get_field_options` in this scope
   --> tests/test_ffi_parity.rs:102:22
    |
102 |         let result = json_eval_get_field_options(handle, field_path.as_ptr());
    |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_free_result` in this scope
   --> tests/test_ffi_parity.rs:104:9
    |
104 |         json_eval_free_result(result);
    |         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_get_resolved_layout_subform` in this scope
   --> tests/test_ffi_parity.rs:110:22
    |
110 |         let result = json_eval_get_resolved_layout_subform(handle, subform_path.as_ptr());
    |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_free_result` in this scope
   --> tests/test_ffi_parity.rs:112:9
    |
112 |         json_eval_free_result(result);
    |         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_get_evaluated_schema_resolved_subform` in this scope
   --> tests/test_ffi_parity.rs:115:22
    |
115 |         let result = json_eval_get_evaluated_schema_resolved_subform(handle, subform_path.as_ptr());
    |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_free_result` in this scope
   --> tests/test_ffi_parity.rs:132:9
    |
132 |         json_eval_free_result(result);
    |         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `json_eval_free` in this scope
   --> tests/test_ffi_parity.rs:135:9
    |
135 |         json_eval_free(handle);
    |         ^^^^^^^^^^^^^^ not found in this scope
```

Commit:
- `3818a501d42057ce5cbfae9e6c868a33ebf22ebd` (`refactor: clarify rust modules and examples`)

Integration risks:
- Full suite still blocked by pre-existing FFI parity missing-function failures.
- `examples/README.md` references planned canonical binding paths (`bindings/npm/examples/`, `bindings/csharp/examples/`) that do not exist in this worktree yet by supervisor direction.
