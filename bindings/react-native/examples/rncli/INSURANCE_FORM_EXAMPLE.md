# Insurance Form Example - React Native

This example demonstrates the new `evaluate_dependents` API features using the `minimal_form.json` schema.

## Features Demonstrated

### 1. **Dot Notation Paths** 🎯
Instead of verbose JSON Schema pointers, you can now use simple dot notation:

```typescript
// Old way (still works):
evaluateDependents({
  changedPath: "#/illustration/properties/insured/properties/date_of_birth",
  data
})

// New way (recommended):
evaluateDependents({
  changedPath: "illustration.insured.date_of_birth",  // Much simpler!
  data
})
```

### 2. **Get Evaluated Schema Without $params**
Keep sensitive metadata separate from the evaluated schema:

```typescript
// Get schema without $params (clean for UI rendering)
const schema = await evalInstance.getEvaluatedSchemaWithoutParams({ skipLayout: true });

// Access $params separately using dot notation
const params = await evalInstance.getValueByPath({ 
  path: '$params', 
  skipLayout: true 
});

console.log(params.productName);  // "Minimal Insurance Product"
console.log(params.productCode);  // "MIN001"
console.log(params.constants.MAX_AGE);  // 100
```

### 3. **Get Values by Path (Dot Notation)**
Access any schema value using simple paths:

```typescript
// Get calculated age value
const age = await evalInstance.getValueByPath({ 
  path: 'illustration.properties.insured.properties.age.value',
  skipLayout: true 
});

// Get product constants
const maxAge = await evalInstance.getValueByPath({ 
  path: '$params.constants.MAX_AGE',
  skipLayout: true 
});
```

### 4. **Transitive Dependencies (Automatic)**
The dependency chain is automatically processed:

```typescript
// Changing occupation triggers a chain reaction:
// occupation → occupation_class → risk_category
await evalInstance.evaluateDependents({
  changedPath: "illustration.insured.occupation",
  data
});

// All dependent fields are automatically updated transitively!
// No need for manual "nested" parameter anymore
```

### 5. **Clear and Value Dependents**
Two types of dependent actions:

```typescript
// When is_smoker changes:
// 1. Clears occupation field
// 2. Updates risk_category based on smoker status
await evalInstance.evaluateDependents({
  changedPath: "illustration.insured.is_smoker",
  data
});
```

### 6. **Real-time Field Calculations**
Uses DATEDIF function for age calculation:

```typescript
// When date_of_birth changes, age is automatically calculated
await evalInstance.evaluateDependents({
  changedPath: "illustration.insured.date_of_birth",
  data
});
```

## Dependency Chain Example

```
is_smoker changes
  ↓
  Clears occupation
  ↓
  Updates risk_category (based on smoker)

occupation changes
  ↓
  Calculates occupation_class (1, 2, or 3)
  ↓
  Calculates risk_category (Low, Medium, High) [transitive]
```

## Schema Structure

From `tests/fixtures/minimal_form.json`:

```json
{
  "$params": {
    "productCode": "MIN001",
    "constants": { "MAX_AGE": 100, "MIN_AGE": 1 },
    "references": {
      "OCCUPATION_TABLE": [...]
    }
  },
  "properties": {
    "illustration": {
      "properties": {
        "insured": {
          "properties": {
            "date_of_birth": {
              "dependents": [{
                "$ref": "#/illustration/properties/insured/properties/age",
                "value": { "$evaluation": { "DATEDIF": [...] } }
              }]
            },
            "is_smoker": {
              "dependents": [
                { "$ref": "...", "clear": true },
                { "$ref": "...", "value": { "$evaluation": {...} } }
              ]
            }
          }
        }
      }
    }
  }
}
```

## Running the Example

```bash
cd bindings/react-native/examples/rncli
yarn install
npm run android  # or npm run ios
```

Navigate to the "Insurance Form" screen to see all features in action.

## API Migration

### Before (Old API):
```typescript
const result = await evalInstance.evaluateDependents({
  changedPaths: ["#/illustration/properties/insured/properties/name"],
  data,
  context,
  nested: true
});
```

### After (New API):
```typescript
const result = await evalInstance.evaluateDependents({
  changedPath: "illustration.insured.name",  // Single path, dot notation
  data,  // Optional now
  context
});
// nested parameter removed - always transitive now
```

## Key Benefits

1. ✅ **Simpler paths**: Dot notation is more intuitive
2. ✅ **Cleaner schema access**: Separate $params from evaluated schema
3. ✅ **Flexible value access**: Get any schema value by path
4. ✅ **Automatic transitive processing**: No manual configuration needed
5. ✅ **Better debugging**: Clear dependency execution logs
6. ✅ **Type-safe**: Full TypeScript support
