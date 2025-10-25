---
layout: page
title: Core Operators
permalink: /operators-core/
---

# Core Operators

Core operators for accessing data and defining literal values.

## `var` - Variable Access

Access data from the input context.

### Syntax
```json
{"var": "path"}
{"var": ["path", default_value]}
```

### Parameters
- **path** (string): Path to the variable (dot notation, JSON pointer, or array indices)
- **default_value** (optional, any): Value to return if variable is missing or null

### Return Type
Any - Returns the value at the specified path, or the default value if not found

### Examples

**Basic access:**
```json
// Data: {"name": "Alice", "age": 30}
{"var": "name"}  // → "Alice"
{"var": "age"}   // → 30
```

**Nested object access:**
```json
// Data: {"user": {"profile": {"city": "NYC"}}}
{"var": "user.profile.city"}  // → "NYC"
```

**Array access:**
```json
// Data: {"items": [1, 2, 3, 4, 5]}
{"var": "items.0"}  // → 1
{"var": "items.2"}  // → 3
```

**With default value:**
```json
// Data: {"name": "Bob"}
{"var": ["age", 25]}        // → 25 (age missing, returns default)
{"var": ["name", "Guest"]}  // → "Bob" (name exists, returns actual value)
```

**Root context reference:**
```json
// Data: [1, 2, 3]
{"var": ""}  // → [1, 2, 3] (returns entire data)
```

**Current context in array operations:**
```json
// In map/filter, empty string refers to current element
{"map": [[1, 2, 3], {"var": ""}]}  // → [1, 2, 3]
```

### Path Formats

All formats are normalized internally:

**Dot notation:**
```json
{"var": "user.profile.name"}
```

**JSON Pointer:**
```json
{"var": "/user/profile/name"}
```

**Array indices:**
```json
{"var": "items.0"}
{"var": "items[0]"}  // Also supported
```

### Edge Cases

**Missing variables:**
```json
// Data: {"name": "Alice"}
{"var": "missing"}  // → null
```

**Null values:**
```json
// Data: {"value": null}
{"var": "value"}           // → null
{"var": ["value", "default"]}  // → "default" (null treated as missing)
```

**Empty path:**
```json
{"var": ""}  // Returns root context
```

---

## `$ref` / `ref` - Reference Access

Access data using JSON Schema-style references. Behaves identically to `var` but uses `$ref` syntax.

### Syntax
```json
{"$ref": "path"}
{"ref": "path"}
{"$ref": ["path", default_value]}
```

### Parameters
Same as `var`

### Return Type
Any - Returns the value at the specified path

### Examples

**Basic reference:**
```json
// Data: {"$params": {"rate": 0.05}}
{"$ref": "$params.rate"}  // → 0.05
```

**JSON Schema style:**
```json
{"$ref": "#/properties/user/properties/name"}
```

**With default:**
```json
{"$ref": ["missing.field", "default"]}  // → "default"
```

### Notes
- Automatically normalizes JSON Schema paths (removes `/properties/` segments)
- Both `$ref` and `ref` are supported
- Paths are converted to JSON pointer format internally

---

## Literal Values

Define constant values directly in logic expressions.

### `null` - Null Value

```json
null  // → null
```

### `bool` - Boolean Value

```json
true   // → true
false  // → false
```

### `number` - Numeric Value

```json
42        // → 42
3.14159   // → 3.14159
-17.5     // → -17.5
0         // → 0
```

**Features:**
- Arbitrary precision support via `serde_json`
- Stored as strings internally to preserve exact precision
- Converted to f64 for arithmetic operations

### `string` - String Value

```json
"hello"         // → "hello"
"Hello World"   // → "Hello World"
""              // → "" (empty string)
```

### `array` - Array Literal

```json
[1, 2, 3]                    // → [1, 2, 3]
["a", "b", "c"]              // → ["a", "b", "c"]
[1, "hello", true, null]     // → [1, "hello", true, null] (mixed types)
[]                           // → [] (empty array)
```

**Nested arrays:**
```json
[[1, 2], [3, 4], [5, 6]]     // → [[1, 2], [3, 4], [5, 6]]
```

**With expressions:**
```json
[{"var": "x"}, {"var": "y"}, 10]  // Evaluates expressions inside array
```

---

## Examples

### Combining Core Operators

**Access nested data with fallback:**
```json
{
  "if": [
    {"var": "user.profile.displayName"},
    {"var": "user.profile.displayName"},
    {"var": ["user.name", "Guest"]}
  ]
}
```

**Building complex structures:**
```json
{
  "cat": [
    "User: ",
    {"var": "name"},
    " (Age: ",
    {"var": ["age", "Unknown"]},
    ")"
  ]
}
```

**Array manipulation:**
```json
{
  "map": [
    {"var": "users"},
    {"var": "name"}
  ]
}
// Extracts name from each user object
```

---

## Best Practices

1. **Use dot notation** for clearer, more readable paths
   ```json
   {"var": "user.profile.name"}  // ✓ Clear
   {"var": "/user/profile/name"} // ✓ Also works
   ```

2. **Provide defaults** for optional fields
   ```json
   {"var": ["settings.theme", "light"]}
   ```

3. **Use empty string** for current context in array operations
   ```json
   {"map": [array, {"*": [{"var": ""}, 2]}]}
   ```

4. **Normalize paths consistently** (handled automatically)
   - `"user.profile"` and `"/user/profile"` are equivalent
   - Both are converted to JSON pointer format internally

5. **Handle missing data gracefully**
   ```json
   {"ifnull": [{"var": "optional"}, "default"]}
   ```

---

## Related Operators

- **[if](operators-logical.md#if)** - Conditional logic based on variable values
- **[missing](operators-utility.md#missing)** - Check for missing variables
- **[ifnull](operators-logical.md#ifnull)** - Provide fallback for null values
- **[Array Operators](operators-array.md)** - Transform arrays using `var` with empty string

---

## Performance Notes

- **Path normalization** happens during compilation, not evaluation
- **Variable lookups** use zero-copy references for maximum performance
- **Compiled logic** caches path parsing for repeated evaluations
