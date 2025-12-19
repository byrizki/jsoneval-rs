---
layout: default
title: Web & NodeJS Usage Guide
---

# Web & NodeJS Usage Guide

The `@json-eval-rs/webcore` package provides a unified high-level API for both Browser (WASM) and NodeJS environments. It handles loading the appropriate backend automatically.

## Installation

### For Web (Bundlers: Vite, Webpack, etc.)

```bash
yarn add @json-eval-rs/webcore @json-eval-rs/bundler
```

-   `@json-eval-rs/webcore`: The high-level TS API.
-   `@json-eval-rs/bundler`: The WASM compilation target for bundlers.

### For NodeJS

```bash
yarn add @json-eval-rs/webcore @json-eval-rs/node
```

-   `@json-eval-rs/node`: The NodeJS bindings (via N-API/WASM).

## Quick Start (Web)

```typescript
import { JSONEval } from '@json-eval-rs/webcore';
import * as wasmModule from '@json-eval-rs/bundler';

// 1. Initialize
// Explicitly passing wasmModule is recommended for bundlers to handle async loading correctly
const evalInstance = new JSONEval({
  schema: mySchema,
  wasmModule
});

// 2. Evaluate
const result = await evalInstance.evaluate({
  data: myData
});

// 3. Selective Evaluation (Optional)
const partialResult = await evalInstance.evaluate({
  data: myData,
  paths: ['user.email', 'preferences']
});

// 4. Cleanup
evalInstance.free();
```

## Quick Start (NodeJS)

NodeJS usage is nearly identical, but the module loading is handled by the specialized package.

```typescript
import { JSONEval } from '@json-eval-rs/webcore';
// Import the node backend
import '@json-eval-rs/node'; 

const run = async () => {
    // The node backend registers itself globally or is auto-detected
    const evalInstance = new JSONEval({ schema });

    try {
        const result = await evalInstance.validate({ data });
        console.log(result.has_error);
    } finally {
        evalInstance.free();
    }
};

run();
```

## API Reference

### `JSONEval` Class

#### Constructor

```typescript
new JSONEval(options: {
  schema: SchemaObject | string;
  context?: object | string;
  data?: object | string;
  wasmModule?: any; // Required for web bundlers
})
```

#### `evaluate`

```typescript
await evaluate(options: {
  data: object | string;
  context?: object | string;
  paths?: string[]; // Selective evaluation paths
})
```

#### `validate`

```typescript
await validate(options: {
  data: object | string;
  context?: object | string;
})
```

Returns:
```typescript
{
  has_error: boolean;
  errors: Array<{
    path: string;
    message: string;
    rule: string;
    // ...
  }>;
}
```

#### `evaluateSubform`

```typescript
await evaluateSubform(options: {
  subformPath: string; // e.g. "#/items"
  data: object | string;
  paths?: string[]; // Selective paths within the subform item
})
```

#### `evaluateDependents`

```typescript
await evaluateDependents(options: {
  changedPaths: string[]; // e.g. ["items.0.qty"]
  data: object | string;
  context?: object | string;
  reEvaluate?: boolean; // Default: false
})
```

## Advanced: Dynamic Imports (Review for Next.js)

For frameworks like Next.js where you might want to load WASM only on the client side:

```typescript
'use client';

import { useEffect, useState } from 'react';

export function MyComponent() {
  const [evaluator, setEvaluator] = useState(null);

  useEffect(() => {
    async function load() {
      const [ { JSONEval }, wasmModule ] = await Promise.all([
        import('@json-eval-rs/webcore'),
        import('@json-eval-rs/bundler')
      ]);

      const instance = new JSONEval({ 
        schema, 
        wasmModule 
      });
      setEvaluator(instance);
    }
    load();
    
    return () => evaluator?.free();
  }, []);
  
  // ...
}
```

## Memory Management

Since the Web implementation relies on WebAssembly (Rust), memory is managed manually to ensure performance and prevent leaks. **You must call `.free()`** when you are done with an instance.

```typescript
// Good practice
try {
  await instance.evaluate(...);
} finally {
  instance.free();
}
```
