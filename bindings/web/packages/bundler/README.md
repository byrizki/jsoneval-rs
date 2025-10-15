# @json-eval-rs/bundler

JSON Eval RS for modern bundlers (Webpack, Vite, Rollup, Next.js, etc.) with ergonomic API.

## Installation

```bash
npm install @json-eval-rs/bundler
# or
yarn add @json-eval-rs/bundler
```

## Usage

```typescript
import { JSONEval } from '@json-eval-rs/bundler';

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
      },
      email: {
        type: 'string',
        rules: {
          required: { value: true, message: 'Email is required' },
          email: { value: true, message: 'Invalid email' }
        }
      }
    }
  }
});

// Initialize (loads WASM)
await evaluator.init();

// Validate data
const result = await evaluator.validate({
  data: { name: 'Jo', email: 'invalid' }
});

if (result.has_error) {
  console.log('Validation errors:', result.errors);
  // [{ path: 'name', rule_type: 'minLength', message: 'Min 3 characters' }, ...]
}

// Evaluate schema with data
const evaluated = await evaluator.evaluate({
  data: { name: 'John', email: 'john@example.com' }
});

// Get schema values
const values = await evaluator.getSchemaValue();

// Clean up when done
evaluator.free();
```

## API

### `new JSONEval(options)`

Create a new evaluator instance.

**Options:**
- `schema` (required) - JSON schema object
- `context` (optional) - Context data object
- `data` (optional) - Initial data object

### `await evaluator.init()`

Initialize the WASM module. Must be called before other methods.

### `await evaluator.validate({ data, context? })`

Validate data against schema rules.

Returns: `{ has_error: boolean, errors: ValidationError[] }`

### `await evaluator.evaluate({ data, context? })`

Evaluate schema with data and return evaluated schema.

### `await evaluator.evaluateDependents({ changedPaths, data, context?, nested? })`

Re-evaluate fields that depend on changed paths.

### `await evaluator.getEvaluatedSchema({ skipLayout? })`

Get the evaluated schema with optional layout resolution.

### `await evaluator.getSchemaValue()`

Get all schema values (evaluations ending with `.value`).

### `await evaluator.reloadSchema({ schema, context?, data? })`

Reload schema with new data.

### `await evaluator.cacheStats()`

Get cache statistics: `{ hits, misses, entries }`.

### `await evaluator.clearCache()`

Clear the evaluation cache.

### `await evaluator.cacheLen()`

Get number of cached entries.

### `evaluator.free()`

Free WASM resources. Always call when done.

### `version()`

Get library version string.

## Example: Next.js

```typescript
'use client';

import { JSONEval } from '@json-eval-rs/bundler';
import { useEffect, useState } from 'react';

export default function MyForm() {
  const [evaluator, setEvaluator] = useState(null);

  useEffect(() => {
    // Dynamically import (client-side only)
    import('@json-eval-rs/bundler').then(({ JSONEval }) => {
      const instance = new JSONEval({ schema });
      setEvaluator(instance);
    });

    return () => {
      if (evaluator) evaluator.free();
    };
  }, []);

  // ... use evaluator
}
```

## License

MIT
