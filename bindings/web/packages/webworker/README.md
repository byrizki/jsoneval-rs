# @json-eval-rs/webworker

Run JSON Eval RS WASM in a Web Worker for better performance by keeping heavy computations off the main thread.

## Installation

```bash
npm install @json-eval-rs/webworker @json-eval-rs/bundler
```

## Usage

```typescript
import { JSONEvalWorker } from '@json-eval-rs/webworker';

const evaluator = new JSONEvalWorker({
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

// Initialize the worker
await evaluator.init();

// Validate data
const result = await evaluator.validate({
  data: { name: 'Jo', email: 'invalid' }
});

if (result.has_error) {
  console.log('Validation errors:', result.errors);
  // [{ path: 'name', message: 'Min 3 characters' }, ...]
}

// Clean up when done
await evaluator.free();
```

## API

### `new JSONEvalWorker(options)`

Create a new worker-based evaluator instance.

**Options:**
- `schema` (required) - JSON schema object
- `context` (optional) - Context data object
- `data` (optional) - Initial data object
- `workerUrl` (optional) - Custom worker script URL

### `await evaluator.init()`

Initialize the worker and load WASM. Must be called before other methods.

### `await evaluator.validate({ data, context? })`

Validate data against schema rules (runs in worker).

Returns: `{ has_error: boolean, errors: ValidationError[] }`

### `await evaluator.evaluate({ data, context? })`

Evaluate schema with data (runs in worker).

Returns: Evaluated schema object

### `await evaluator.evaluateDependents({ changedPaths, data, context?, nested? })`

Re-evaluate fields that depend on changed paths (runs in worker).

Returns: Updated evaluated schema

### `await evaluator.free()`

Terminate worker and free resources. Always call this when done.

## Benefits

✅ **Non-blocking** - Computations run off the main thread  
✅ **Better UX** - UI remains responsive during heavy operations  
✅ **Same API** - Similar to `@json-eval-rs/core`  
✅ **Automatic** - Worker lifecycle managed for you  

## When to Use

Use `@json-eval-rs/webworker` when:
- Processing large schemas or datasets
- Running frequent validations (e.g., on every keystroke)
- App performance is critical
- You need to keep the UI responsive

Use `@json-eval-rs/core` when:
- Simple, infrequent operations
- Minimal setup overhead needed
- Not performance-critical

## Example: Next.js

```typescript
'use client';

import { JSONEvalWorker } from '@json-eval-rs/webworker';
import { useEffect, useState } from 'react';

export default function MyForm() {
  const [evaluator, setEvaluator] = useState(null);

  useEffect(() => {
    const worker = new JSONEvalWorker({ schema });
    worker.init().then(() => setEvaluator(worker));

    return () => {
      worker.free();
    };
  }, []);

  const handleValidate = async () => {
    if (!evaluator) return;
    
    const result = await evaluator.validate({ data: formData });
    // Handle result...
  };

  // ...
}
```

## License

MIT
