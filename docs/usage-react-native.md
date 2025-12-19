---
layout: default
title: React Native Usage Guide
---

# React Native Usage Guide

The `@json-eval-rs/react-native` package provides high-performance bindings for iOS and Android, using the JSI (JavaScript Interface) for direct C++ communication.

## Installation

```bash
yarn add @json-eval-rs/react-native
# or
npm install @json-eval-rs/react-native
```

### iOS Setup
```bash
cd ios && pod install
```

## Quick Start

The library provides a convenient `useJSONEval` hook for React components.

```typescript
import React, { useState } from 'react';
import { View, Text, TextInput } from 'react-native';
import { useJSONEval } from '@json-eval-rs/react-native';

const schema = {
  type: 'object',
  properties: {
    username: {
      type: 'string',
      rules: {
        required: { value: true, message: 'Username required' },
        minLength: { value: 3, message: 'Min 3 chars' }
      }
    }
  }
};

export default function SignupForm() {
  const [data, setData] = useState({ username: '' });
  const [errors, setErrors] = useState({});
  
  // Initialize evaluator
  const evalInstance = useJSONEval({ schema });

  const handleChange = async (text: string) => {
    const newData = { username: text };
    setData(newData);

    if (evalInstance) {
      // Validate on change
      // Tip: Use selective validation for better performance on large forms
      const validation = await evalInstance.validate({ 
        data: JSON.stringify(newData) 
      });
      
      setErrors(validation.errors || {});
    }
  };

  return (
    <View>
      <TextInput 
        value={data.username}
        onChangeText={handleChange}
        placeholder="Username"
      />
      {errors.username && (
        <Text style={{color: 'red'}}>{errors.username.message}</Text>
      )}
    </View>
  );
}
```

## API Reference

### `JSONEval` Class

You can also use the class directly for non-React contexts (e.g., utility functions, state management stores).

#### Constructor

```typescript
const eval = new JSONEval({ 
  schema: string | object, 
  context?: string | object,
  data?: string | object
});
```

#### `evaluate`

Evaluates the schema with data.

```typescript
await eval.evaluate({ 
  data: string | object,
  paths?: string[] // Optional: Selective evaluation paths
});
```

#### `validate`

Validates data against rules.

```typescript
const result = await eval.validate({ 
  data: string | object 
});

// Result format:
interface ValidationResult {
  has_error: boolean;
  errors: Record<string, ValidationError>;
}
```

#### `evaluateSubform`

Interact with isolated subforms (e.g., line items).

```typescript
await eval.evaluateSubform({
  subformPath: '#/items',
  data: itemData, // Data for the single item
  paths: ['quantity'] // Optional: Selective paths within item
});
```

#### `dispose`

**Crucial**: Native resources must be freed manually when no longer needed. The `useJSONEval` hook handles this automatically, but if you use the class directly:

```typescript
// When done
await eval.dispose();
```

## Subform Usage

Subforms allow efficient evaluation of array items without re-evaluating the entire parent form.

```typescript
// 1. Check if subform exists
if (await eval.hasSubform('#/order_items')) {
  
  // 2. Evaluate a specific item
  await eval.evaluateSubform({
    subformPath: '#/order_items',
    data: { id: '123', qty: 5, price: 10 }
  });

  // 3. Get evaluated result for that item
  const itemResult = await eval.getEvaluatedSchemaSubform({
    subformPath: '#/order_items'
  });
}
```

## Troubleshooting

### "Native module not found"
-   Ensure you ran `pod install` in the `ios` directory.
-   Rebuild the app (`yarn android` or `yarn ios`).
-   Verify you are not using Expo Go (custom native code requires Development Build or bare workflow).

### JSI Errors
-   This library uses JSI. Ensure you have Reanimated installed and configured if you face obscure crashes, though it acts independently.
-   Reloading the Metro bundler completely (r - r) often fixes JSI binding stale references.
