# Web Bindings Architecture

## Package Structure

The web bindings use a **core abstraction pattern** where each WASM target exports a pre-configured version of the core API.

```
bindings/web/packages/
├── core/         # Internal abstraction layer (private, not published)
├── bundler/      # For Webpack, Vite, Next.js (exports core + bundler WASM)
├── vanilla/      # For direct browser usage (exports core + vanilla WASM)
└── node/         # For Node.js/SSR (exports core + node WASM)
```

## How It Works

### 1. Core Package (Internal)

`@json-eval-rs/webcore` is a **private package** that provides the high-level JavaScript API:

- `JSONEvalCore` class with all methods
- Accepts WASM module as first constructor argument
- Not published to npm (internal use only)

### 2. WASM Packages (Public)

Each WASM package (`bundler`, `vanilla`, `node`) extends core with its own WASM:

```javascript
// packages/bundler/index.js
import { JSONEvalCore } from '@json-eval-rs/webcore';
import * as wasm from './pkg/json_eval_rs.js';

export class JSONEval extends JSONEvalCore {
  constructor(options) {
    super(wasm, options); // Auto-inject WASM
  }
}
```

## Usage

### Before (Old API - Required Manual WASM Passing)

```typescript
import { JSONEval } from '@json-eval-rs/webcore';
import * as wasmModule from '@json-eval-rs/bundler';

const evaluator = new JSONEval({ schema, wasmModule });
```

### After (New API - WASM Auto-Injected)

```typescript
import { JSONEval } from '@json-eval-rs/bundler';

const evaluator = new JSONEval({ schema });
```

## Benefits

✅ **Simpler API** - No need to import and pass WASM module  
✅ **Type-safe** - Each package exports correct types  
✅ **Tree-shakeable** - Only import what you use  
✅ **Zero duplication** - Core logic in one place  
✅ **Target-specific** - Each package optimized for its environment  

## Build Process

1. **Build WASM** - `wasm-pack build` creates `pkg/` in each target
2. **Core unchanged** - No build needed (pure JS wrapper)
3. **Packages export** - Each package's `index.js` re-exports core with WASM

## Publishing

Only the WASM packages are published:
- ✅ `@json-eval-rs/bundler`
- ✅ `@json-eval-rs/vanilla`
- ✅ `@json-eval-rs/node`
- ❌ `@json-eval-rs/webcore` (private, internal only)

## Development

For monorepo development, link packages locally:

```bash
cd bindings/web
yarn install
```

This creates symlinks between packages so changes are immediately reflected.
