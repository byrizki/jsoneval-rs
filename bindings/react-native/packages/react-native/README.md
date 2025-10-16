# @json-eval-rs/react-native

High-performance JSON Logic evaluator with schema validation for React Native.

Built with Rust for maximum performance, with native Android (Kotlin + JNI) and iOS (Objective-C++) bindings. All operations run asynchronously on background threads to keep your UI responsive.

## Features

- 🚀 **Native Performance** - Built with Rust for iOS and Android
- ✅ **Schema Validation** - Validate data against JSON schema rules
- 🔄 **Dependency Tracking** - Auto-update dependent fields
- 🎯 **Type Safe** - Full TypeScript support
- ⚛️ **React Hooks** - Built-in `useJSONEval` hook
- 📱 **Cross-Platform** - Works on iOS and Android
- 🔥 **Fast** - Native performance, not JavaScript

## Installation

```bash
npm install @json-eval-rs/react-native
```

Or with Yarn:

```bash
yarn add @json-eval-rs/react-native
```

### iOS

```bash
cd ios && pod install
```

### Android

No additional steps required. The library uses autolinking.

## Quick Start

### Basic Usage

```typescript
import { JSONEval } from '@json-eval-rs/react-native';

const schema = {
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
};

// Create evaluator
const eval = new JSONEval({ schema });

// Evaluate
const data = { user: { name: 'John', age: 25 } };
const result = await eval.evaluate({ data });
console.log('Evaluated:', result);

// Validate
const validation = await eval.validate({ data });
if (validation.hasError) {
  validation.errors.forEach(error => {
    console.error(`${error.path}: ${error.message}`);
  });
}

// Clean up
await eval.dispose();
```

### Using React Hook

```typescript
import React, { useState } from 'react';
import { View, TextInput, Button, Text } from 'react-native';
import { useJSONEval } from '@json-eval-rs/react-native';

const schema = {
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
        }
      }
    }
  }
};

function MyForm() {
  const eval = useJSONEval({ schema });
  const [name, setName] = useState('');
  const [errors, setErrors] = useState<string[]>([]);

  const handleValidate = async () => {
    if (!eval) return;
    
    const data = { user: { name } };
    const validation = await eval.validate({ data });
    
    if (validation.hasError) {
      setErrors(validation.errors.map(e => e.message));
    } else {
      setErrors([]);
      console.log('Valid!');
    }
  };

  return (
    <View>
      <TextInput
        value={name}
        onChangeText={setName}
        placeholder="Enter name"
      />
      <Button title="Validate" onPress={handleValidate} />
      
      {errors.map((error, i) => (
        <Text key={i} style={{ color: 'red' }}>{error}</Text>
      ))}
    </View>
  );
}
```

### Advanced: Dependent Fields

```typescript
import React, { useState, useEffect } from 'react';
import { View, TextInput, Text } from 'react-native';
import { useJSONEval } from '@json-eval-rs/react-native';

const schema = {
  type: 'object',
  properties: {
    quantity: { type: 'number' },
    price: { type: 'number' },
    total: {
      type: 'number',
      $evaluation: {
        '*': [{ var: 'quantity' }, { var: 'price' }]
      }
    }
  }
};

function Calculator() {
  const eval = useJSONEval({ schema });
  const [quantity, setQuantity] = useState(1);
  const [price, setPrice] = useState(10);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    if (!eval) return;
    
    const updateTotal = async () => {
      const data = { quantity, price };
      const result = await eval.evaluateDependents({
        changedPaths: ['quantity', 'price'],
        data,
        nested: true
      });
      
      setTotal(result.total);
    };
    
    updateTotal();
  }, [eval, quantity, price]);

  return (
    <View>
      <TextInput
        value={String(quantity)}
        onChangeText={(val) => setQuantity(Number(val))}
        keyboardType="numeric"
      />
      <TextInput
        value={String(price)}
        onChangeText={(val) => setPrice(Number(val))}
        keyboardType="numeric"
      />
      <Text>Total: {total}</Text>
    </View>
  );
}
```

## API Reference

### JSONEval Class

#### Constructor

```typescript
constructor(options: {
  schema: string | object;
  context?: string | object;
  data?: string | object;
})
```

Creates a new evaluator instance.

#### Methods

##### evaluate(options)

Evaluates the schema with provided data.

```typescript
async evaluate(options: {
  data: string | object;
  context?: string | object;
}): Promise<any>
```

##### validate(options)

Validates data against schema rules.

```typescript
async validate(options: {
  data: string | object;
  context?: string | object;
}): Promise<ValidationResult>
```

Returns:
```typescript
interface ValidationResult {
  hasError: boolean;
  errors: ValidationError[];
}

interface ValidationError {
  path: string;
  ruleType: string;
  message: string;
}
```

##### evaluateDependents(options)

Re-evaluates fields that depend on changed paths.

```typescript
async evaluateDependents(options: {
  changedPaths: string[];
  data: string | object;
  context?: string | object;
  nested?: boolean;
}): Promise<any>
```

##### dispose()

Frees native resources. Must be called when done.

```typescript
async dispose(): Promise<void>
```

##### static version()

Gets the library version.

```typescript
static async version(): Promise<string>
```

### useJSONEval Hook

React hook for automatic lifecycle management.

```typescript
function useJSONEval(options: JSONEvalOptions): JSONEval | null
```

Returns `null` until initialized, then returns the `JSONEval` instance.
Automatically disposes on unmount.

## Validation Rules

Supported validation rules:

- **required** - Field must have a value
- **minLength** / **maxLength** - String/array length validation
- **minValue** / **maxValue** - Numeric range validation
- **pattern** - Regex pattern matching

## Platform Support

- **iOS**: 11.0+
- **Android**: API 21+ (Android 5.0)
- **React Native**: 0.64+

## Performance

Typical performance on modern devices:
- Schema parsing: < 5ms
- Evaluation: < 10ms for complex schemas
- Validation: < 5ms

Native performance beats JavaScript-only solutions by 10-50x.

### Sequential Processing

This library uses **sequential processing** by default, which is optimal for mobile devices. The Rust core supports an optional `parallel` feature using Rayon, but:
- Mobile devices have limited cores and power constraints
- Sequential processing is faster for typical mobile use cases (small to medium datasets)
- Parallel overhead exceeds benefits for arrays < 1000 items
- Battery life is better with sequential processing

The default configuration is optimized for mobile performance and battery efficiency.

## Error Handling

All async methods can throw errors:

```typescript
try {
  const result = await eval.evaluate({ data });
} catch (error) {
  console.error('Evaluation error:', error.message);
}
```

## Memory Management

Always dispose of instances when done:

```typescript
const eval = new JSONEval({ schema });
try {
  // Use eval
} finally {
  await eval.dispose(); // Important!
}
```

Or use the hook for automatic management:

```typescript
function MyComponent() {
  const eval = useJSONEval({ schema }); // Auto-disposed on unmount
  // Use eval
}
```

## TypeScript

Full TypeScript support included. All types are exported:

```typescript
import type {
  JSONEval,
  ValidationError,
  ValidationResult,
  JSONEvalOptions,
  EvaluateOptions,
  EvaluateDependentsOptions
} from '@json-eval-rs/react-native';
```

## Troubleshooting

### iOS Build Errors

If you encounter build errors on iOS:

```bash
cd ios
rm -rf Pods Podfile.lock
pod install --repo-update
```

### Android Build Errors

If you encounter build errors on Android:

```bash
cd android
./gradlew clean
cd ..
```

Then rebuild your app.

### "Module not found" Error

Make sure you've:
1. Installed the package
2. Run `pod install` on iOS
3. Rebuilt the app completely

## Contributing

See the [contributing guide](CONTRIBUTING.md) to learn how to contribute to the repository and the development workflow.

## License

MIT

## Support

- GitHub Issues: https://github.com/byrizki/json-eval-rs/issues
- Documentation: https://github.com/byrizki/json-eval-rs
