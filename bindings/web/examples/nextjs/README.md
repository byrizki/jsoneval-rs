# JSON Eval RS - Next.js Example

This example demonstrates how to use `@json-eval-rs/bundler` in a Next.js application with three different use cases.

## Features

### 1. Form Validation
- Real-time form validation using JSON schema rules
- Custom validation messages
- Field-level error display
- Powered by WASM for high performance

### 2. Dependent Fields
- Automatic calculation of dependent values
- Real-time updates when dependencies change
- Nested dependency resolution
- Invoice/cart calculation example

### 3. Web Worker
- Off-thread WASM execution using Web Workers
- Non-blocking UI during validation
- Maintains 60fps smooth animations
- Better perceived performance

## Getting Started

### Prerequisites

1. Build the bindings first:
```bash
cd ../..  # Go to repo root
./build-bindings.sh web
```

2. Install dependencies:
```bash
cd examples/nextjs
yarn install
```

### Run Development Server

```bash
yarn dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

## Package Usage

### Main Thread (Default)

```typescript
import { JSONEval } from '@json-eval-rs/bundler';

const evaluator = new JSONEval({
  schema: {
    type: 'object',
    properties: {
      name: {
        type: 'string',
        rules: {
          required: { value: true, message: 'Name is required' }
        }
      }
    }
  }
});

await evaluator.init();
const result = await evaluator.validate({ data: { name: 'John' } });
```

### Web Worker (Off Main Thread)

```typescript
import { JSONEvalWorker } from '@json-eval-rs/bundler/worker-client';

const evaluator = new JSONEvalWorker({ schema });
await evaluator.init();

// Runs in worker - UI stays responsive!
const result = await evaluator.validate({ data });
```

## API Examples

### Validation

```typescript
const result = await evaluator.validate({
  data: { name: 'John', email: 'john@example.com' }
});

if (result.has_error) {
  result.errors.forEach(error => {
    console.log(`${error.path}: ${error.message}`);
  });
}
```

### Dependent Fields

```typescript
const result = await evaluator.evaluateDependents({
  changedPaths: ['quantity', 'price'],
  data: { quantity: 5, price: 10 },
  nested: true  // Follow dependency chains
});

console.log(result.total); // Auto-calculated
```

### Cache Management

```typescript
// Get cache statistics
const stats = await evaluator.cacheStats();
console.log(`Hits: ${stats.hits}, Misses: ${stats.misses}`);

// Clear cache
await evaluator.clearCache();
```

## Architecture

### Client-Side Only

All WASM operations run **client-side only** using Next.js's `'use client'` directive:

```typescript
'use client';

import { useEffect, useState } from 'react';

export default function MyComponent() {
  useEffect(() => {
    // Dynamic import ensures client-side only
    import('@json-eval-rs/bundler').then(({ JSONEval }) => {
      // Use evaluator here
    });
  }, []);
}
```

### Why Client-Side?

- **WASM modules** require browser APIs
- **Web Workers** are browser-only
- **Better performance** - validation happens on the client
- **Reduced server load** - no validation requests

## Performance

### Main Thread
- Small schemas: ~1-5ms
- Medium schemas: ~10-50ms
- Large schemas: ~50-200ms
- **May block UI** during heavy computations

### Web Worker
- Same execution time as main thread
- **Never blocks UI** - runs off main thread
- Smooth 60fps animations maintained
- Better for frequent validations

## File Structure

```
examples/nextjs/
├── components/
│   ├── FormValidator.tsx      # Basic validation
│   ├── DependentFields.tsx    # Dependent field calculation
│   └── WorkerExample.tsx      # Web Worker usage
├── pages/
│   └── index.tsx              # Main page with tabs
└── package.json
```

## Troubleshooting

### TypeScript Errors

If you see TypeScript errors after updating packages:

```bash
# Rebuild bindings
cd ../.. && ./build-bindings.sh web

# Reinstall dependencies
cd examples/nextjs
rm -rf node_modules yarn.lock
yarn install
```

### Worker Not Found

Ensure the bundler includes worker files:

```javascript
// next.config.js
module.exports = {
  webpack: (config) => {
    config.module.rules.push({
      test: /\.wasm$/,
      type: 'webassembly/async'
    });
    return config;
  }
};
```

### WASM Loading Errors

Make sure to use dynamic imports:

```typescript
// ✅ Correct - dynamic import
import('@json-eval-rs/bundler').then(({ JSONEval }) => {
  // Use here
});

// ❌ Wrong - static import may fail in SSR
import { JSONEval } from '@json-eval-rs/bundler';
```

## Learn More

- [Package Documentation](../../bindings/web/README.md)
- [Worker Guide](../../bindings/web/WORKERS.md)
- [Architecture](../../bindings/web/ARCHITECTURE.md)
- [Migration Guide](../../bindings/web/MIGRATION.md)

## License

MIT
