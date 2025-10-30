# @json-eval-rs/webcore

High-level JavaScript API for JSON Eval RS WASM bindings.

## Installation

```bash
# Install bridge + your target WASM package
yarn install @json-eval-rs/webcore @json-eval-rs/bundler

# Or for direct browser use
yarn install @json-eval-rs/webcore @json-eval-rs/vanilla

# Or for Node.js
yarn install @json-eval-rs/webcore @json-eval-rs/node
```

## Usage

### With Bundler (Webpack, Vite, Next.js, etc.)

```typescript
import { JSONEval } from '@json-eval-rs/webcore';
import * as wasmModule from '@json-eval-rs/bundler';

const evaluator = new JSONEval({
  schema: {
    type: 'object',
    properties: {
      name: {
        type: 'string',
        rules: {
          required: { value: true, message: 'Name is required' },
          minLength: { value: 3, message: 'Min 3 characters' }
        }
      }
    }
  },
  wasmModule // Pass the WASM module explicitly
});

// Validate data
const result = await evaluator.validate({
  data: { name: 'Jo' }
});

if (result.has_error) {
  console.log('Errors:', result.errors);
}

// Don't forget to free resources
evaluator.free();
```

### Dynamic Import (for Next.js client components)

```typescript
'use client';

useEffect(() => {
  Promise.all([
    import('@json-eval-rs/webcore'),
    import('@json-eval-rs/bundler')
  ]).then(([{ JSONEval }, wasmModule]) => {
    const evaluator = new JSONEval({ schema, wasmModule });
    // Use evaluator...
  });
}, []);
```

### API

#### `new JSONEval(options)`

Create a new evaluator instance.

**Options:**
- `schema` (required) - JSON schema object
- `context` (optional) - Context data object
- `data` (optional) - Initial data object
- `wasmModule` (optional) - Pre-loaded WASM module

#### `await evaluator.validate({ data, context? })`

Validate data against schema rules.

Returns: `{ has_error: boolean, errors: ValidationError[] }`

#### `await evaluator.evaluate({ data, context? })`

Evaluate schema with data.

Returns: Evaluated schema object

#### `await evaluator.evaluateDependents({ changedPaths, data, context?, nested? })`

Re-evaluate fields that depend on changed paths.

**Options:**
- `changedPaths` - Array of field paths that changed
- `data` - Current data
- `context` (optional) - Context data
- `nested` (optional, default: true) - Follow dependency chains

Returns: Updated evaluated schema

#### `evaluator.free()`

Free WASM resources. Call this when done.

## Why Use the Core?

The core package provides:

1. **Clean API** - Options objects instead of positional JSON strings
2. **Type Safety** - Full TypeScript support
3. **Auto-detection** - Automatically loads the right WASM target
4. **Flexibility** - Works with bundler/web/node targets

## Direct WASM Usage

If you prefer minimal overhead, you can use WASM packages directly:

```typescript
import { JSONEvalWasm } from '@json-eval-rs/bundler';

const instance = new JSONEvalWasm(
  JSON.stringify(schema),
  JSON.stringify(context),
  JSON.stringify(data)
);

const result = await instance.validate(JSON.stringify(data));
instance.free();
```

## License

MIT
