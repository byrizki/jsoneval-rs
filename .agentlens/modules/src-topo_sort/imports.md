# Imports

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

## Dependency Graph

```mermaid
graph TD
    src_topo_sort[src-topo_sort] --> JSONEval[JSONEval]
    src_topo_sort[src-topo_sort] --> ParsedSchema[ParsedSchema]
    src_topo_sort[src-topo_sort] --> indexmap[indexmap]
    src_topo_sort[src-topo_sort] --> jsoneval[jsoneval]
```

## Internal Dependencies

Dependencies within this module:

- `common`
- `legacy`
- `parsed`
- `topo_sort`

## External Dependencies

Dependencies from other modules:

- `JSONEval`
- `ParsedSchema`
- `indexmap`
- `jsoneval`

