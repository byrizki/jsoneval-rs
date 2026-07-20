# Dependency Graph

## Most Imported Files (change these carefully)

- `examples/common.rs` — imported by **3** files
- `bindings/npm/packages/node/pkg/json_eval_rs.js` — imported by **2** files
- `bindings/npm/examples/rncli/App.tsx` — imported by **2** files
- `bindings/npm/examples/rncli/src/screens/FormValidationScreen.tsx` — imported by **1** files
- `bindings/npm/examples/rncli/src/screens/DependentFieldsScreen.tsx` — imported by **1** files
- `bindings/npm/packages/bundler/pkg/json_eval_rs_bg.js` — imported by **1** files
- `bindings/npm/packages/bundler/pkg/json_eval_rs.js` — imported by **1** files
- `bindings/npm/packages/common/src/utils.ts` — imported by **1** files
- `bindings/npm/packages/common/src/types.ts` — imported by **1** files
- `bindings/npm/packages/react-native/lib/commonjs/jsi-bridge.js` — imported by **1** files
- `bindings/npm/packages/react-native/lib/module/jsi-bridge.js` — imported by **1** files
- `bindings/npm/packages/react-native/src/index.tsx` — imported by **1** files
- `bindings/npm/packages/react-native/src/jsi-bridge.ts` — imported by **1** files
- `bindings/npm/packages/vanilla/pkg/json_eval_rs.js` — imported by **1** files
- `tests/common.rs` — imported by **1** files

## Import Map (who imports what)

- `examples/common.rs` ← `examples/basic.rs`, `examples/basic_parsed.rs`, `examples/benchmark.rs`
- `bindings/npm/packages/node/pkg/json_eval_rs.js` ← `bindings/npm/examples/nodejs-benchmark/simulate_cache_miss.js`, `bindings/npm/packages/node/src/index.ts`
- `bindings/npm/examples/rncli/App.tsx` ← `bindings/npm/examples/rncli/__tests__/App.test.tsx`, `bindings/npm/examples/rncli/index.js`
- `bindings/npm/examples/rncli/src/screens/FormValidationScreen.tsx` ← `bindings/npm/examples/rncli/App.tsx`
- `bindings/npm/examples/rncli/src/screens/DependentFieldsScreen.tsx` ← `bindings/npm/examples/rncli/App.tsx`
- `bindings/npm/packages/bundler/pkg/json_eval_rs_bg.js` ← `bindings/npm/packages/bundler/pkg/json_eval_rs.js`
- `bindings/npm/packages/bundler/pkg/json_eval_rs.js` ← `bindings/npm/packages/bundler/src/index.ts`
- `bindings/npm/packages/common/src/utils.ts` ← `bindings/npm/packages/common/src/index.ts`
- `bindings/npm/packages/common/src/types.ts` ← `bindings/npm/packages/common/src/utils.ts`
- `bindings/npm/packages/react-native/lib/commonjs/jsi-bridge.js` ← `bindings/npm/packages/react-native/lib/commonjs/index.js`
