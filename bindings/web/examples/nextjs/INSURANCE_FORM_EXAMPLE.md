# Insurance Form Example - Next.js

This example demonstrates the new `evaluate_dependents` API features using the `minimal_form.json` schema with WebAssembly.

## Features Demonstrated

### 1. **Dot Notation Paths** 🎯
Instead of verbose JSON Schema pointers, you can now use simple dot notation:

```typescript
// Old way (still works):
await evalInstance.evaluateDependents({
  changedPath: "#/illustration/properties/insured/properties/date_of_birth",
  data
})

// New way (recommended):
await evalInstance.evaluateDependents({
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

// Get reference tables
const occupationTable = await evalInstance.getValueByPath({ 
  path: '$params.references.OCCUPATION_TABLE',
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
// Result includes 'transitive' flag to identify chained updates
result.forEach(change => {
  console.log(`${change.transitive ? '🔗' : '✅'} ${change.$ref}: ${change.value}`);
});
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

// Check the result:
result.forEach(change => {
  if (change.clear) {
    console.log(`Cleared: ${change.$ref}`);
  } else if (change.value) {
    console.log(`Updated: ${change.$ref} = ${change.value}`);
  }
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

## Dependency Chain Visualization

The example includes a live dependency execution log:

```
📅 Date changed → Age calculated
  ✅ Age: 34

🚬 Smoker status changed → Clear occupation, update risk
  ✅ Occupation cleared
  ✅ Risk: High

💼 Occupation → Class → Risk (transitive chain)
  ✅ Class: 1
  🔗 Risk: Low
```

## Schema Structure

From `tests/fixtures/minimal_form.json`:

```json
{
  "$params": {
    "type": "illustration",
    "productCode": "MIN001",
    "productName": "Minimal Insurance Product",
    "constants": {
      "MAX_AGE": 100,
      "MIN_AGE": 1
    },
    "references": {
      "OCCUPATION_TABLE": [
        { "occupation": "OFFICE", "class": "1", "risk": "Low" },
        { "occupation": "PROFESSIONAL", "class": "1", "risk": "Low" },
        { "occupation": "MANUAL", "class": "2", "risk": "Medium" },
        { "occupation": "HIGH_RISK", "class": "3", "risk": "High" }
      ]
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
                "value": {
                  "$evaluation": {
                    "DATEDIF": [
                      { "$ref": "$value" },
                      { "NOW": [] },
                      "Y"
                    ]
                  }
                }
              }]
            },
            "is_smoker": {
              "dependents": [
                {
                  "$ref": "#/illustration/properties/insured/properties/occupation",
                  "clear": { "$evaluation": true }
                },
                {
                  "$ref": "#/illustration/properties/insured/properties/risk_category",
                  "value": {
                    "$evaluation": {
                      "if": [
                        { "$ref": "$value" },
                        "High",
                        "Standard"
                      ]
                    }
                  }
                }
              ]
            },
            "occupation": {
              "dependents": [{
                "$ref": "#/illustration/properties/insured/properties/occupation_class",
                "value": { "$evaluation": { "if": [...] } }
              }]
            },
            "occupation_class": {
              "dependents": [{
                "$ref": "#/illustration/properties/insured/properties/risk_category",
                "value": { "$evaluation": { "if": [...] } }
              }]
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
cd bindings/web/examples/nextjs
npm install
npm run dev
```

Open http://localhost:3000 and navigate to the "Insurance Form 🎯" tab.

## WebAssembly Integration

The example uses dynamic imports for optimal performance:

```typescript
import('@json-eval-rs/bundler').then(async ({ JSONEval }) => {
  const instance = new JSONEval({ schema });
  
  // Initialize with data
  await instance.evaluate({ data });
  
  // Get params separately
  const params = await instance.getValueByPath({ 
    path: '$params', 
    skipLayout: true 
  });
  
  // Get schema without params
  const cleanSchema = await instance.getEvaluatedSchemaWithoutParams({ 
    skipLayout: true 
  });
});
```

## API Migration

### Before (Old API):
```typescript
const result = await evalInstance.evaluateDependents({
  changedPaths: [
    "#/illustration/properties/insured/properties/name",
    "#/illustration/properties/insured/properties/age"
  ],
  data,
  context,
  nested: true
});
```

### After (New API):
```typescript
// Single field change with dot notation
const result = await evalInstance.evaluateDependents({
  changedPath: "illustration.insured.name",  // Dot notation!
  data,  // Optional now
  context
});
// nested parameter removed - always transitive
// changedPaths array → single changedPath string
```

## Key Benefits

1. ✅ **Simpler paths**: Dot notation is more intuitive than JSON Schema pointers
2. ✅ **Cleaner schema access**: Separate $params from evaluated schema
3. ✅ **Flexible value access**: Get any schema value by path
4. ✅ **Automatic transitive processing**: No manual configuration needed
5. ✅ **Better debugging**: Clear dependency execution logs with transitive flags
6. ✅ **Type-safe**: Full TypeScript support with proper interfaces
7. ✅ **WebAssembly performance**: Blazing fast evaluation in the browser
8. ✅ **SSR-compatible**: Works with Next.js server-side rendering

## Component Features

The `InsuranceForm.tsx` component demonstrates:

- ✅ Dot notation for all field paths
- ✅ `getEvaluatedSchemaWithoutParams()` for clean UI schema
- ✅ `getValueByPath()` for accessing $params and calculated values
- ✅ Real-time dependency execution log with transitive indicators
- ✅ Interactive UI showing all dependency chains
- ✅ Dark mode support
- ✅ Responsive design with Tailwind CSS

## Browser Compatibility

Works in all modern browsers supporting WebAssembly:
- Chrome/Edge 57+
- Firefox 52+
- Safari 11+
- Opera 44+

## Performance

The WebAssembly implementation provides:
- **Fast evaluation**: Near-native performance
- **Small bundle size**: ~150KB gzipped WASM
- **Zero JavaScript runtime**: Pure Rust logic compiled to WASM
- **Efficient caching**: Built-in evaluation cache with content-based hashing
