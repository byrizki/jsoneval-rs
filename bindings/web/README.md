# @json-eval-rs/web

High-performance JSON Logic evaluator with schema validation for web browsers and Node.js. Built with WebAssembly for native-like performance.

## Features

- 🚀 **Blazing Fast** - WebAssembly powered by Rust
- ✅ **Schema Validation** - Validate data against JSON schema rules
- 🔄 **Dependency Tracking** - Auto-update dependent fields
- 🎯 **Type Safe** - Full TypeScript support
- 📦 **Zero Dependencies** - Pure WASM, no external deps
- 🌐 **Universal** - Works in browsers and Node.js
- 📊 **Small Bundle** - < 500KB gzipped

## Installation

```bash
npm install @json-eval-rs/web
```

Or with Yarn:

```bash
yarn add @json-eval-rs/web
```

## Quick Start

### ES Modules (Browser/Modern Node.js)

```javascript
import { JSONEval } from '@json-eval-rs/web';

const schema = JSON.stringify({
  type: 'object',
  properties: {
    user: {
      type: 'object',
      properties: {
        name: {
          type: 'string',
          rules: {
            required: { value: true, message: 'Name is required' },
            minLength: { value: 3, message: 'Min 3 characters' }
          }
        },
        age: {
          type: 'number',
          rules: {
            minValue: { value: 18, message: 'Must be 18+' }
          }
        }
      }
    }
  }
});

// Create evaluator
const eval = new JSONEval({ schema });

// Evaluate
const data = JSON.stringify({ user: { name: 'John', age: 25 } });
const result = await eval.evaluateJS({ data });
console.log('Evaluated:', result);

// Validate
const validation = await eval.validate({ data });
if (validation.has_error) {
  validation.errors.forEach(error => {
    console.error(`${error.path}: ${error.message}`);
  });
}

// Clean up
eval.free();
```

### TypeScript

```typescript
import { JSONEval, ValidationResult } from '@json-eval-rs/web';

interface Schema {
  type: string;
  properties: Record<string, any>;
}

const schema: Schema = {
  type: 'object',
  properties: {
    user: {
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
  }
};

const eval = new JSONEval({ 
  schema: JSON.stringify(schema) 
});

const data = JSON.stringify({ user: { name: 'Alice' } });
const result: any = await eval.evaluateJS({ data });

const validation: ValidationResult = await eval.validate({ data });
console.log('Valid:', !validation.has_error);

eval.free();
```

## API Reference

### Constructor

```typescript
new JSONEval(options: {
  schema: string;
  context?: string;
  data?: string;
})
```

Creates a new evaluator instance.

### Methods

#### evaluate(options)

Evaluates the schema with provided data, returns JSON string.

```typescript
await eval.evaluate({ 
  data: string,
  context?: string 
}): Promise<string>
```

#### evaluateJS(options)

Evaluates the schema with provided data, returns JavaScript object.

```typescript
await eval.evaluateJS({ 
  data: string,
  context?: string 
}): Promise<any>
```

#### validate(options)

Validates data against schema rules.

```typescript
await eval.validate({ 
  data: string,
  context?: string 
}): Promise<ValidationResult>
```

#### evaluateDependents(options)

Re-evaluates fields that depend on changed paths, returns JSON string.

```typescript
await eval.evaluateDependents({
  changedPaths: string[],
  data: string,
  context?: string,
  nested?: boolean
}): Promise<string>
```

#### evaluateDependentsJS(options)

Re-evaluates fields that depend on changed paths, returns JavaScript object.

```typescript
await eval.evaluateDependentsJS({
  changedPaths: string[],
  data: string,
  context?: string,
  nested?: boolean
}): Promise<any>
```

#### free()

Frees WebAssembly resources. Must be called when done.

```typescript
eval.free(): void
```

### Validation Rules

Supported validation rules:

- **required** - Field must have a value
- **minLength** / **maxLength** - String/array length validation
- **minValue** / **maxValue** - Numeric range validation
- **pattern** - Regex pattern matching

## React Example

```jsx
import { useEffect, useState } from 'react';
import { JSONEval } from '@json-eval-rs/web';

function MyForm() {
  const [eval, setEval] = useState(null);
  const [data, setData] = useState({ user: { name: '', age: 0 } });
  const [validation, setValidation] = useState(null);

  useEffect(() => {
    const instance = new JSONEval({ schema: mySchema });
    setEval(instance);
    
    return () => instance.free();
  }, []);

  const handleValidate = async () => {
    if (!eval) return;
    
    const result = await eval.validate({ 
      data: JSON.stringify(data) 
    });
    setValidation(result);
  };

  return (
    <div>
      <input 
        value={data.user.name}
        onChange={(e) => setData({
          ...data,
          user: { ...data.user, name: e.target.value }
        })}
      />
      <button onClick={handleValidate}>Validate</button>
      
      {validation?.has_error && (
        <ul>
          {validation.errors.map((err, i) => (
            <li key={i}>{err.message}</li>
          ))}
        </ul>
      )}
    </div>
  );
}
```

## Vue Example

```vue
<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { JSONEval } from '@json-eval-rs/web';

const eval = ref(null);
const data = ref({ user: { name: '', age: 0 } });
const validation = ref(null);

onMounted(async () => {
  eval.value = new JSONEval({ schema: mySchema });
});

onUnmounted(() => {
  eval.value?.free();
});

async function validate() {
  if (!eval.value) return;
  validation.value = await eval.value.validate({
    data: JSON.stringify(data.value)
  });
}
</script>

<template>
  <div>
    <input v-model="data.user.name" />
    <button @click="validate">Validate</button>
    
    <ul v-if="validation?.has_error">
      <li v-for="(err, i) in validation.errors" :key="i">
        {{ err.message }}
      </li>
    </ul>
  </div>
</template>
```

## Performance

Typical performance (1000 iterations):
- Schema parsing: ~3ms
- Evaluation: ~5ms
- Validation: ~2ms

WASM module size: ~450KB gzipped

## Browser Support

- Chrome/Edge 57+
- Firefox 52+
- Safari 11+
- Node.js 12+

## Error Handling

All async methods can throw errors:

```javascript
try {
  const result = await eval.evaluate({ data });
} catch (error) {
  console.error('Evaluation error:', error.message);
}
```

## Memory Management

Always call `free()` when done:

```javascript
const eval = new JSONEval({ schema });
try {
  // Use eval
} finally {
  eval.free(); // Important!
}
```

## Building from Source

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build for web
npm run build

# Build for Node.js
npm run build:node

# Build for bundlers (Webpack, etc.)
npm run build:bundler
```

## License

MIT

## Support

- GitHub Issues: https://github.com/yourusername/json-eval-rs/issues
- Documentation: https://github.com/yourusername/json-eval-rs

## Version

```javascript
import { version } from '@json-eval-rs/web';
console.log(await version()); // "0.1.0"
```
