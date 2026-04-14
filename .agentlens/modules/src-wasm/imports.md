# Imports

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

## Dependency Graph

```mermaid
graph TD
    src_wasm[src-wasm] --> JSONEval[JSONEval]
    src_wasm[src-wasm] --> jsoneval[jsoneval]
    src_wasm[src-wasm] --> serde[serde]
    src_wasm[src-wasm] --> super[super]
    src_wasm[src-wasm] --> wasm_bindgen[wasm_bindgen]
```

## Internal Dependencies

Dependencies within this module:

- `core`
- `evaluation`
- `layout`
- `schema`
- `subforms`
- `types`
- `validation`

## External Dependencies

Dependencies from other modules:

- `JSONEval`
- `jsoneval`
- `serde`
- `super`
- `wasm_bindgen`

