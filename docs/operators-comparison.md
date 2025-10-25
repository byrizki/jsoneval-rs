---
layout: default
title: Comparison Operators
---

# Comparison Operators

Value comparison operators that return boolean results.

## `==` - Equal (Loose Equality)

Checks if two values are equal with type coercion.

### Syntax
```json
{"==": [value1, value2]}
```

### Parameters
- **value1** (any): First value
- **value2** (any): Second value

### Return Type
Boolean - `true` if values are equal (after type coercion)

### Examples

**Basic equality:**
```json
{"==": [1, 1]}           // → true
{"==": ["hello", "hello"]} // → true
{"==": [true, true]}     // → true
```

**Type coercion:**
```json
{"==": [1, "1"]}         // → true (string coerced to number)
{"==": [0, false]}       // → true
{"==": [null, null]}     // → true
```

**With variables:**
```json
// Data: {"age": 30, "minAge": 30}
{"==": [{"var": "age"}, {"var": "minAge"}]}
// → true
```

---

## `===` - Strict Equal

Checks if two values are equal without type coercion.

### Syntax
```json
{"===": [value1, value2]}
```

### Examples

**Strict equality:**
```json
{"===": [1, 1]}           // → true
{"===": [1, "1"]}         // → false (different types)
{"===": [0, false]}       // → false (different types)
```

---

## `!=` - Not Equal

Checks if two values are not equal with type coercion.

### Syntax
```json
{"!=": [value1, value2]}
```

### Examples
```json
{"!=": [1, 2]}            // → true
{"!=": [1, "1"]}          // → false (coerced to same)
```

---

## `!==` - Strict Not Equal

Checks if two values are not equal without type coercion.

### Syntax
```json
{"!==": [value1, value2]}
```

### Examples
```json
{"!==": [1, "1"]}         // → true (different types)
{"!==": [1, 1]}           // → false
```

---

## `<` - Less Than

### Syntax
```json
{"<": [value1, value2]}
```

### Examples
```json
{"<": [5, 10]}            // → true
{"<": ["a", "b"]}         // → true (lexicographic)
```

---

## `<=` - Less Than or Equal

### Syntax
```json
{"<=": [value1, value2]}
```

### Examples
```json
{"<=": [5, 10]}           // → true
{"<=": [10, 10]}          // → true
```

---

## `>` - Greater Than

### Syntax
```json
{">": [value1, value2]}
```

### Examples
```json
{">": [10, 5]}            // → true
{">": [{var": "age"}, 18]} // Age check
```

---

## `>=` - Greater Than or Equal

### Syntax
```json
{">=": [value1, value2]}
```

### Examples
```json
{">=": [10, 5]}           // → true
{">=": [10, 10]}          // → true
```

---

## Complex Examples

### Range Validation
```json
{"and": [
  {">=": [{"var": "value"}, 10]},
  {"<=": [{"var": "value"}, 100]}
]}
```

### Grade Calculator
```json
{"if": [
  {">=": [{"var": "score"}, 90]}, "A",
  {"if": [
    {">=": [{"var": "score"}, 80]}, "B",
    {"if": [
      {">=": [{"var": "score"}, 70]}, "C",
      "F"
    ]}
  ]}
]}
```

---

## Best Practices

1. **Use strict equality** when type matters
2. **Combine comparisons** for range checks
3. **Order matters** in chained conditions

---

## Related Operators

- **[Logical Operators](operators-logical.md)** - Combine comparisons
- **[if](operators-logical.md#if)** - Branch based on comparisons
