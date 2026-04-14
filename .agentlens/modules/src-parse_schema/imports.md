# Imports

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

## Dependency Graph

```mermaid
graph TD
    src_parse_schema[src-parse_schema] --> ParsedSchema[ParsedSchema]
    src_parse_schema[src-parse_schema] --> crate[crate]
    src_parse_schema[src-parse_schema] --> indexmap[indexmap]
    src_parse_schema[src-parse_schema] --> jsoneval[jsoneval]
    src_parse_schema[src-parse_schema] --> serde_json[serde_json]
    src_parse_schema[src-parse_schema] --> std[std]
    src_parse_schema[src-parse_schema] --> topo_sort[topo_sort]
```

## Internal Dependencies

Dependencies within this module:

- `common`
- `legacy`
- `parsed`

## External Dependencies

Dependencies from other modules:

- `ParsedSchema`
- `crate`
- `indexmap`
- `jsoneval`
- `serde_json`
- `std`
- `topo_sort`

