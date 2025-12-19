---
layout: default
title: Comparison Operators
---

# Comparison Operators

Value comparison operators that return boolean results.

## Overview

Comparison operators evaluate relationships between values and return `true` or `false`. They are fundamental for building conditional logic, validation rules, and flow control in JSON evaluations.

### When to Use Comparison Operators

- **Validation**: Check if user input meets requirements
- **Business Rules**: Implement age restrictions, price limits, date ranges
- **Conditional Logic**: Branch execution based on value relationships
- **Filtering**: Select data matching specific criteria
- **Access Control**: Verify permissions and authorization levels

### Type Coercion vs Strict Comparison

JSON-EvalRS provides two comparison modes:

1. **Loose Equality** (`==`, `!=`): Performs type coercion before comparison
2. **Strict Equality** (`===`, `!==`): No type coercion, types must match

**When to use each:**
- Use **strict** (`===`) when type safety is important (e.g., distinguishing `0` from `false`)
- Use **loose** (`==`) when comparing values that might have different representations (e.g., string "1" vs number 1)

---

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
{"==": [0, false]}       // → true (boolean coerced to number)
{"==": [null, null]}     // → true
{"==": ["", false]}      // → true (both falsy)
```

**With variables:**
```json
// Data: {"age": 30, "minAge": 30}
{"==": [{"var": "age"}, {"var": "minAge"}]}
// → true
```

**Comparing calculated values:**
```json
// Data: {"price": 100, "discount": 20}
{"==": [
  {"-": [{"var": "price"}, {"var": "discount"}]},
  80
]}
// → true
```

### Type Coercion Rules

- Numbers and strings: String converted to number
- Booleans and numbers: `true` → 1, `false` → 0
- `null` equals only `null` (and `""` in loose mode)
- Objects and arrays: Reference comparison (same instance only)

---

## `===` - Strict Equal

Checks if two values are equal without type coercion.

### Syntax
```json
{"===": [value1, value2]}
```

### Parameters
Same as `==`

### Return Type
Boolean - `true` if values are equal and of same type

### Examples

**Strict equality:**
```json
{"===": [1, 1]}           // → true
{"===": [1, "1"]}         // → false (different types)
{"===": [0, false]}       // → false (number vs boolean)
{"===": [null, null]}     // → true
{"===": ["", ""]}         // → true
```

**When types matter:**
```json
// API returns string "0" for inactive, number 0 for error
// Data: {"status":0}
{"===": [{"var": "status"}, "0"]}    // → false
{"===": [{"var": "status"}, 0]}      // → true
```

**Validating exact matches:**
```json
// Ensure role is exactly the string "admin"
{"===": [{"var": "user.role"}, "admin"]}
```

---

## `!=` - Not Equal

Checks if two values are not equal with type coercion.

### Syntax
```json
{"!=": [value1, value2]}
```

### Parameters
Same as `==`

### Return Type
Boolean - `true` if values are not equal (after type coercion)

### Examples

**Basic inequality:**
```json
{"!=": [1, 2]}            // → true
{"!=": [1, "1"]}          // → false (coerced to same)
{"!=": ["a", "b"]}        // → true
```

**Validation:**
```json
// Ensure password and confirm don't match would be an error
{"!=": [{"var": "password"}, {"var": "confirmPassword"}]}
// → true means passwords don't match (validation fails)
```

**Check for non-zero:**
```json
{"!=": [{"var": "count"}, 0]}
// → true if count has any value except 0 or "0"
```

---

## `!==` - Strict Not Equal

Checks if two values are not equal without type coercion.

### Syntax
```json
{"!==": [value1, value2]}
```

### Parameters
Same as `==`

### Return Type
Boolean - `true` if values differ in value or type

### Examples

**Strict inequality:**
```json
{"!==": [1, "1"]}         // → true (different types)
{"!==": [1, 1]}           // → false (same value and type)
{"!==": [0, false]}       // → true (different types)
```

**Distinguish between null and empty:**
```json
{"!==": [{"var": "optional"}, null]}
// Returns true if field exists (even if empty string)
```

**Type safety:**
```json
// Ensure value is number, not string
{"and": [
  {"!==": [{"var": "amount"}, null]},
  {"!==": [{"var": "amount"}, ""]}
]}
```

---

## `<` - Less Than

Compares if first value is less than the second.

### Syntax
```json
{"<": [value1, value2]}
```

### Parameters
- **value1** (number/string): First value
- **value2** (number/string): Second value

### Return Type
Boolean - `true` if value1 < value2

### Examples

**Numeric comparison:**
```json
{"<": [5, 10]}            // → true
{"<": [10, 5]}            // → false
{"<": [5, 5]}             // → false
```

**String comparison (lexicographic):**
```json
{"<": ["a", "b"]}         // → true
{"<": ["apple", "banana"]} // → true (alphabetical)
{"<": ["10", "2"]}        // → true (string comparison, not numeric)
```

**Age validation:**
```json
// Data: {"age": 16}
{"<": [{"var": "age"}, 18]}
// → true (minor)
```

**Date comparison:**
```json
{"<": [
  {"var": "startDate"},
  {"today": null}
]}
// → true if start date is in the past
```

---

## `<=` - Less Than or Equal

Compares if first value is less than or equal to the second.

### Syntax
```json
{"<=": [value1, value2]}
```

### Parameters
Same as `<`

### Return Type
Boolean - `true` if value1 ≤ value2

### Examples

**Basic comparison:**
```json
{"<=": [5, 10]}           // → true
{"<=": [10, 10]}          // → true (equal counts)
{"<=": [11, 10]}          // → false
```

**Range validation:**
```json
{"<=": [{"var": "score"}, 100]}
// Ensure score doesn't exceed maximum
```

**Quantity check:**
```json
// Data: {"ordered": 50, "available": 100}
{"<=": [{"var": "ordered"}, {"var": "available"}]}
// → true (enough stock)
```

---

## `>` - Greater Than

Compares if first value is greater than the second.

### Syntax
```json
{">": [value1, value2]}
```

### Parameters
Same as `<`

### Return Type
Boolean - `true` if value1 > value2

### Examples

**Numeric comparison:**
```json
{">": [10, 5]}            // → true
{">": [5, 10]}            // → false
{">": [10, 10]}           // → false
```

**Age check:**
```json
{">": [{"var": "age"}, 18]}
// → true if age > 18 (adult)
```

**Non-empty array:**
```json
{">": [{"length": {"var": "items"}}, 0]}
// → true if array has elements
```

**Price validation:**
```json
{">": [{"var": "bidAmount"}, {"var": "minimumBid"}]}
```

---

## `>=` - Greater Than or Equal

Compares if first value is greater than or equal to the second.

### Syntax
```json
{">=": [value1, value2]}
```

### Parameters
Same as `<`

### Return Type
Boolean - `true` if value1 ≥ value2

### Examples

**Basic comparison:**
```json
{">=": [10, 5]}           // → true
{">=": [10, 10]}          // → true (equal counts)
{">=": [5, 10]}           // → false
```

**Minimum value validation:**
```json
{">=": [{"var": "quantity"}, 1]}
// Ensure at least one item ordered
```

**Permission level:**
```json
// Data: {"userLevel": 5, "requiredLevel": 3}
{">=": [{"var": "userLevel"}, {"var": "requiredLevel"}]}
// → true (sufficient permissions)
```

---

## Complex Examples

### Range Validation (Between Min and Max)

```json
{"and": [
  {">=": [{"var": "value"}, 10]},
  {"<=": [{"var": "value"}, 100]}
]}
// Returns true if 10 ≤ value ≤ 100
```

**Alternative using multiple conditions:**
```json
{
  "let": {
    "val": {"var": "value"},
    "inRange": {"and": [
      {">=": [{"var": "val"}, {"var": "min"}]},
      {"<=": [{"var": "val"}, {"var": "max"}]}
    ]}
  },
  "in": {"var": "inRange"}
}
```

### Grade Calculator

```json
{"if": [
  {">=": [{"var": "score"}, 90]}, "A",
  {"if": [
    {">=": [{"var": "score"}, 80]}, "B",
    {"if": [
      {">=": [{"var": "score"}, 70]}, "C",
      {"if": [
        {">=": [{"var": "score"}, 60]}, "D",
        "F"
      ]}
    ]}
  ]}
]}
```

**Cleaner with let:**
```json
{
  "let": {
    "s": {"var": "score"}
  },
  "in": {
    "if": [
      {">=": [{"var": "s"}, 90]}, "A",
      {">=": [{"var": "s"}, 80]}, "B",
      {">=": [{"var": "s"}, 70]}, "C",
      {">=": [{"var": "s"}, 60]}, "D",
      "F"
    ]
  }
}
```

### Password Strength Validation

```json
{"and": [
  {">=": [{"length": {"var": "password"}}, 8]},
  {"<=": [{"length": {"var": "password"}}, 128]},
  {"!==": [{"var": "password"}, {"var": "username"}]}
]}
// Must be 8-128 chars and not equal to username
```

### Price Comparison with Discount

```json
{
  "let": {
    "originalPrice": {"var": "price"},
    "discountedPrice": {"*": [
      {"var": "price"},
      {"-": [1, {"/": [{"var": "discountPercent"}, 100]}]}
    ]}
  },
  "in": {
    "<": [{"var": "discountedPrice"}, {"var": "originalPrice"}]
  }
}
// Verifies discount actually reduces price
```

### Age-Based Access Control

```json
{
  "let": {
    "age": {"DATEDIF": [
      {"var": "birthDate"},
      {"today": null},
      "Y"
    ]},
    "isChild": {"<": [{"var": "age"}, 13]},
    "isTeen": {"and": [
      {">=": [{"var": "age"}, 13]},
      {"<": [{"var": "age"}, 18]}
    ]},
    "isAdult": {">=": [{"var": "age"}, 18]}
  },
  "in": {
    "if": [
      {"var": "isChild"}, "RESTRICTED",
      {"var": "isTeen"}, "PARENTAL_CONSENT_REQUIRED",
      "FULL_ACCESS"
    ]
  }
}
```

### Tiered Pricing

```json
{
  "let": {
    "qty": {"var": "quantity"}
  },
  "in": {
    "*": [
      {"var": "qty"},
      {"if": [
        {">=": [{"var": "qty"}, 100]}, 8.00,  // Bulk price
        {">=": [{"var": "qty"}, 50]}, 9.00,   // Medium volume
        {">=": [{"var": "qty"}, 10]}, 10.00,  // Small volume
        12.00                                  // Retail price
      ]}
    ]
  }
}
```

### Date Range Validation

```json
{"and": [
  {">=": [{"var": "eventDate"}, {"today": null}]},
  {"<=": [
    {"var": "eventDate"},
    {"DATEADD": [{"today": null}, 90, "days"]}
  ]}
]}
// Event must be within next 90 days
```

---

## Best Practices

### 1. Use Strict Equality for Type Safety

When type matters, always use strict comparison:

```json
// ✅ Good - ensures exact type match
{"===": [{"var": "status"}, "active"]}

// ⚠️ Risky - might match number 0 or false
{"==": [{"var": "status"}, "active"]}
```

### 2. Combine Comparisons for Ranges

Use `and` to validate ranges:

```json
// ✅ Clear range check
{"and": [
  {">=": [{"var": "age"}, 18]},
  {"<=": [{"var": "age"}, 65]}
]}

// ❌ Don't compare both ends separately
```

### 3. Order Matters in Conditional Chains

Put most restrictive conditions first:

```json
// ✅ Efficient - checks highest tier first
{"if": [
  {">=": [score, 90]}, "A",
  {">=": [score, 80]}, "B",
  "F"
]}

// ❌ Inefficient - would never reach lower tiers
{"if": [
  {">=": [score, 60]}, "Pass",
  {">=": [score, 90]}, "A"  // Never evaluated
]}
```

### 4. Use Let for Repeated Comparisons

Cache values used multiple times:

```json
// ✅ Efficient - calculates once
{
  "let": {
    "totalPrice": {"+": [{"var": "price"}, {"var": "tax"}]}
  },
  "in": {
    "and": [
      {">": [{"var": "totalPrice"}, 0]},
      {"<=": [{"var": "totalPrice"}, {"var": "budget"}]}
    ]
  }
}

// ❌ Inefficient - calculates twice
{"and": [
  {">": [{"+": [{"var": "price"}, {"var": "tax"}]}, 0]},
  {"<=": [{"+": [{"var": "price"}, {"var": "tax"}]}, {"var": "budget"}]}
]}
```

### 5. Handle Edge Cases

Always consider null, undefined, and boundary values:

```json
// ✅ Safe - handles missing values
{"and": [
  {"!==": [{"var": "age"}, null]},
  {">": [{"var": "age"}, 0]},
  {"<": [{"var": "age"}, 150]}
]}
```

### 6. Use Descriptive Variable Names

Make comparisons self-documenting:

```json
// ✅ Clear intent
{
  "let": {
    "isEligibleAge": {">=": [{"var": "age"}, 18]},
    "hasValidID": {"!==": [{"var": "idNumber"}, null]}
  },
  "in": {
    "and": [{"var": "isEligibleAge"}, {"var": "hasValidID"}]
  }
}
```

### 7. Consider Floating Point Precision

For decimal comparisons, use rounding:

```json
// ⚠️ Might fail due to floating point precision
{"==": [{"+": [0.1, 0.2]}, 0.3]}

// ✅ Better - round before comparing
{"==": [
  {"round": [{"+": [0.1, 0.2]}, 2]},
  0.3
]}
```

---

## Troubleshooting

### Issue: Unexpected false with type coercion

**Problem:** Loose equality returns unexpected results.

**Solution:** Use strict equality or explicitly convert types:

```json
// ❌ Unexpected: "10" == 10 is true
{"==": [{"var": "stringValue"}, 10]}

// ✅ Explicit type checking
{"===": [{"ToNumber": [{"var": "stringValue"}]}, 10]}
```

### Issue: String vs number comparison

**Problem:** Comparing strings numerically.

**Solution:** Convert strings to numbers first:

```json
// ❌ Wrong: Lexicographic comparison
{"<": ["10", "2"]}  // true but not numerically correct

// ✅ Correct: Numeric comparison
{"<": [
  {"ToNumber": ["10"]},
  {"ToNumber": ["2"]}
]}  // false
```

### Issue: Null comparisons failing

**Problem:** Null checks not working as expected.

**Solution:** Use explicit null checks:

```json
// ✅ Correct null check
{"===": [{"var": "optional"}, null]}

// ✅ Check if not null
{"!==": [{"var": "optional"}, null]}

// ⚠️ Might not work as expected (null is falsy)
{"!": [{"var": "optional"}]}
```

---

## Related Operators

- **[Logical Operators](operators-logical.md)** - Combine comparisons with `and`, `or`, `not`
- **[if](operators-logical.md#if)** - Branch based on comparison results
- **[Math Functions](operators-math.md)** - `round`, `floor`, `ceil` for numeric comparisons
- **[Type Operators](operators-core.md)** - Convert types before comparing

---

## Performance Notes

- **Short-circuit evaluation** in compound comparisons
- **Type coercion** adds minimal overhead
- **Strict comparisons** slightly faster (no type conversion)
- **Compiled optimizations** for constant comparisons
