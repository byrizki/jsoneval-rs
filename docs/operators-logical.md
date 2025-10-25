---
layout: page
title: Logical Operators
permalink: /operators-logical/
---

# Logical Operators

Boolean logic and conditional execution operators.

## `and` - Logical AND

Returns true only if all conditions are truthy. Short-circuits (stops evaluating) on first falsy value.

### Syntax
```json
{"and": [condition1, condition2, ...]}
```

### Parameters
- **conditions** (array): One or more expressions to evaluate

### Return Type
Any - Returns the first falsy value, or the last value if all are truthy

### Examples

**Basic AND:**
```json
// Data: {"age": 25, "verified": true}
{"and": [
  {">": [{"var": "age"}, 18]},
  {"var": "verified"}
]}
// → true (both conditions met)
```

**Short-circuit evaluation:**
```json
{"and": [false, {"var": "never.evaluated"}]}
// → false (stops at first false, doesn't error on missing path)
```

**Multiple conditions:**
```json
{"and": [
  {">": [{"var": "score"}, 70]},
  {"<": [{"var": "score"}, 100]},
  {"==": [{"var": "status"}, "active"]}
]}
// → true only if all three conditions are true
```

### Truthiness Rules
- **Falsy values**: `false`, `null`, `0`, `""` (empty string)
- **Truthy values**: Everything else

### Optimization
Nested `and` operations are automatically flattened during compilation:
```json
{"and": [{"and": [a, b]}, c]}  // Optimized to: {"and": [a, b, c]}
```

---

## `or` - Logical OR

Returns true if any condition is truthy. Short-circuits on first truthy value.

### Syntax
```json
{"or": [condition1, condition2, ...]}
```

### Parameters
- **conditions** (array): One or more expressions to evaluate

### Return Type
Any - Returns the first truthy value, or the last value if all are falsy

### Examples

**Basic OR:**
```json
// Data: {"premium": false, "trial": true}
{"or": [
  {"var": "premium"},
  {"var": "trial"}
]}
// → true (trial is truthy)
```

**Default values with OR:**
```json
{"or": [
  {"var": "username"},
  {"var": "email"},
  "Guest"
]}
// Returns first available value or "Guest"
```

**Multiple fallbacks:**
```json
{"or": [
  {"var": "primary.value"},
  {"var": "secondary.value"},
  {"var": "tertiary.value"},
  0
]}
```

### Optimization
Nested `or` operations are automatically flattened:
```json
{"or": [{"or": [a, b]}, c]}  // Optimized to: {"or": [a, b, c]}
```

---

## `not` / `!` - Logical NOT

Inverts the truthiness of a value.

### Syntax
```json
{"not": value}
{"!": value}
{"!": [value]}
```

### Parameters
- **value** (any): Expression to negate

### Return Type
Boolean - Always returns `true` or `false`

### Examples

**Basic negation:**
```json
{"!": true}   // → false
{"!": false}  // → true
{"!": 0}      // → true (0 is falsy)
{"!": 1}      // → false (1 is truthy)
```

**Negate comparisons:**
```json
{"!": {">": [{"var": "age"}, 65]}}
// Same as: {"<=": [{"var": "age"}, 65]}
```

**Check for missing:**
```json
{"!": {"var": "optional.field"}}
// Returns true if field is missing or falsy
```

### Optimization
Double negation is automatically eliminated:
```json
{"!": {"!": value}}  // Optimized to: value
```

---

## `if` - Conditional Expression

Evaluates condition and returns one of two branches.

### Syntax
```json
{"if": [condition, then_value, else_value]}
```

### Parameters
- **condition** (any): Expression to evaluate for truthiness
- **then_value** (any): Returned if condition is truthy
- **else_value** (any): Returned if condition is falsy

### Return Type
Any - Returns either then_value or else_value

### Examples

**Basic conditional:**
```json
// Data: {"age": 25}
{"if": [
  {">": [{"var": "age"}, 18]},
  "Adult",
  "Minor"
]}
// → "Adult"
```

**Nested conditions:**
```json
{"if": [
  {"<": [{"var": "age"}, 13]},
  "Child",
  {"if": [
    {"<": [{"var": "age"}, 18]},
    "Teen",
    "Adult"
  ]}
]}
```

**With calculations:**
```json
{"if": [
  {"var": "premium"},
  {"*": [{"var": "price"}, 0.8]},  // 20% discount
  {"var": "price"}                 // Regular price
]}
```

**Conditional array access:**
```json
{"if": [
  {">": [{"length": {"var": "items"}}, 0]},
  {"var": "items.0"},
  null
]}
```

### Notes
- Only evaluates the branch that is taken (lazy evaluation)
- Can be chained for multiple conditions
- Similar to ternary operator in JavaScript: `condition ? then : else`

---

## `xor` - Exclusive OR

Returns true if exactly one of two conditions is truthy (not both, not neither).

### Syntax
```json
{"xor": [value1, value2]}
```

### Parameters
- **value1** (any): First expression
- **value2** (any): Second expression

### Return Type
Boolean - `true` if exactly one value is truthy

### Examples

**Basic XOR:**
```json
{"xor": [true, false]}   // → true
{"xor": [true, true]}    // → false
{"xor": [false, false]}  // → false
```

**Mutual exclusivity check:**
```json
// Data: {"cash": true, "credit": false}
{"xor": [{"var": "cash"}, {"var": "credit"}]}
// → true (exactly one payment method selected)
```

**Toggle logic:**
```json
{"xor": [
  {"var": "settings.darkMode"},
  {"var": "settings.systemDefault"}
]}
```

---

## `ifnull` - Null Coalescing

Returns the first value if it's not null-like, otherwise returns the alternative.

### Syntax
```json
{"ifnull": [value, alternative]}
```

### Parameters
- **value** (any): Expression to check
- **alternative** (any): Fallback value

### Return Type
Any - Either value or alternative

### Examples

**Basic null check:**
```json
// Data: {"name": null}
{"ifnull": [{"var": "name"}, "Unknown"]}
// → "Unknown"
```

**Chain multiple fallbacks:**
```json
{"ifnull": [
  {"var": "primary"},
  {"ifnull": [
    {"var": "secondary"},
    "default"
  ]}
]}
```

**With calculations:**
```json
{"ifnull": [
  {"var": "customRate"},
  0.05  // Default rate
]}
```

### Null-like Values
Treated as null:
- `null`
- `""` (empty string)
- `undefined` (missing variables)

Not treated as null:
- `0` (number zero)
- `false` (boolean)
- `[]` (empty array)

---

## `isempty` - Empty Check

Checks if a value is null or an empty string.

### Syntax
```json
{"isempty": value}
```

### Parameters
- **value** (any): Expression to check

### Return Type
Boolean - `true` if value is null or empty string

### Examples

**Check for empty:**
```json
{"isempty": ""}      // → true
{"isempty": null}    // → true
{"isempty": "text"}  // → false
{"isempty": 0}       // → false
{"isempty": []}      // → false
```

**Validate required field:**
```json
{"if": [
  {"isempty": {"var": "username"}},
  "Username is required",
  "Valid"
]}
```

**Combined with NOT:**
```json
{"!": {"isempty": {"var": "optional.field"}}}
// Returns true if field has a value
```

---

## `empty` - Empty String Literal

Returns an empty string constant.

### Syntax
```json
{"empty": null}
```

### Return Type
String - `""`

### Examples

**Return empty string:**
```json
{"empty": null}  // → ""
```

**Conditional empty:**
```json
{"if": [
  {"var": "show"},
  {"var": "message"},
  {"empty": null}
]}
```

**String building:**
```json
{"cat": [
  {"var": "prefix"},
  {"empty": null},
  {"var": "suffix"}
]}
```

---

## Complex Examples

### Access Control
```json
{"and": [
  {"var": "user.authenticated"},
  {"or": [
    {"==": [{"var": "user.role"}, "admin"]},
    {"==": [{"var": "resource.owner"}, {"var": "user.id"}]}
  ]}
]}
```

### Tiered Pricing
```json
{"if": [
  {">": [{"var": "quantity"}, 100]},
  {"*": [{"var": "price"}, 0.7]},
  {"if": [
    {">": [{"var": "quantity"}, 50]},
    {"*": [{"var": "price"}, 0.85]},
    {"var": "price"}
  ]}
]}
```

### Validation with Multiple Rules
```json
{"and": [
  {"!": {"isempty": {"var": "email"}}},
  {">": [{"length": {"var": "password"}}, 8]},
  {"var": "terms_accepted"},
  {"xor": [
    {"var": "phone"},
    {"var": "backup_email"}
  ]}
]}
```

---

## Best Practices

1. **Use short-circuit evaluation** to avoid unnecessary work
   ```json
   {"and": [{"var": "enabled"}, expensiveCalculation]}
   ```

2. **Prefer ifnull over nested if** for null checks
   ```json
   {"ifnull": [{"var": "value"}, "default"]}  // ✓ Clear
   {"if": [{"var": "value"}, {"var": "value"}, "default"]}  // ✗ Verbose
   ```

3. **Chain conditions logically**
   ```json
   {"and": [condition1, condition2, condition3]}  // ✓
   {"and": [condition1, {"and": [condition2, condition3]}]}  // ✗ Unnecessary nesting
   ```

4. **Use isempty for string validation**
   ```json
   {"!": {"isempty": {"var": "required.field"}}}
   ```

---

## Related Operators

- **[Comparison Operators](operators-comparison.md)** - Generate boolean conditions
- **[missing](operators-utility.md#missing)** - Check for missing keys
- **[Array Quantifiers](operators-array.md#quantifiers)** - `all`, `some`, `none`

---

## Performance Notes

- **Short-circuit evaluation** prevents unnecessary computation
- **Automatic flattening** optimizes nested `and`/`or` chains
- **Double negation elimination** removes redundant `not` operations
- **Lazy evaluation** in `if` only evaluates the taken branch
