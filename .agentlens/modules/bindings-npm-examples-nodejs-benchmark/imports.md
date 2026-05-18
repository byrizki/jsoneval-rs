# Imports

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

## Dependency Graph

```mermaid
graph TD
    bindings_web_examples_nodejs_benchmark[bindings-npm-examples-nodejs-benchmark] --> pkg[pkg]
    bindings_web_examples_nodejs_benchmark[bindings-npm-examples-nodejs-benchmark] --> _json_eval_rs[@json-eval-rs]
    bindings_web_examples_nodejs_benchmark[bindings-npm-examples-nodejs-benchmark] --> fs[fs]
    bindings_web_examples_nodejs_benchmark[bindings-npm-examples-nodejs-benchmark] --> node_fs[node:fs]
    bindings_web_examples_nodejs_benchmark[bindings-npm-examples-nodejs-benchmark] --> node_path[node:path]
    bindings_web_examples_nodejs_benchmark[bindings-npm-examples-nodejs-benchmark] --> node_url[node:url]
    bindings_web_examples_nodejs_benchmark[bindings-npm-examples-nodejs-benchmark] --> path[path]
```

## External Dependencies

Dependencies from other modules:

- `../../packages/node/pkg/json_eval_rs.js`
- `@json-eval-rs/node`
- `fs`
- `node:fs`
- `node:path`
- `node:url`
- `path`

