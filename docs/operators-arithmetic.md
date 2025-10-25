---
layout: default
title: Arithmetic Operators
---

# Arithmetic Operators

Mathematical operators for numeric calculations.

## `+` - Addition

Adds multiple numbers together.

### Syntax
```json
{"+": [value1, value2, ...]}
```

### Parameters
- **values** (array): Numbers to add

### Return Type
Number - Sum of all values

### Examples

**Basic addition:**
```json
{"+": [1, 2, 3]}           // → 6
{"+": [10, 20]}            // → 30
```

**With variables:**
```json
// Data: {"price": 100, "tax": 10, "shipping": 5}
{"+": [
  {"var": "price"},
  {"var": "tax"},
  {"var": "shipping"}
]}
// → 115
```

**Nested calculations:**
```json
{"+": [
  {"*": [{"var": "quantity"}, {"var": "price"}]},
  {"var": "shipping"}
]}
```

**Single value:**
```json
{"+": [42]}                // → 42
```

**Empty array:**
```json
{"+": []}                  // → 0
```

### Optimization
Nested additions are automatically flattened:
```json
{"+": [{"+": [1, 2]}, 3]}  // Optimized to: {"+": [1, 2, 3]}
```

---

## `-` - Subtraction

Subtracts values from the first value, or negates a single value.

### Syntax
```json
{"-": [value1, value2, ...]}
```

### Parameters
- **values** (array): Numbers to subtract

### Return Type
Number - Result of subtraction

### Examples

**Basic subtraction:**
```json
{"-": [10, 3]}             // → 7
{"-": [100, 20, 5]}        // → 75 (100 - 20 - 5)
```

**Negation (unary minus):**
```json
{"-": [5]}                 // → -5
{"-": [-10]}               // → 10
```

**With variables:**
```json
// Data: {"total": 100, "discount": 15}
{"-": [{"var": "total"}, {"var": "discount"}]}
// → 85
```

**Calculate age:**
```json
{"-": [
  {"year": {"today": null}},
  {"var": "birthYear"}
]}
```

---

## `*` - Multiplication

Multiplies multiple numbers together.

### Syntax
```json
{"*": [value1, value2, ...]}
```

### Parameters
- **values** (array): Numbers to multiply

### Return Type
Number - Product of all values

### Examples

**Basic multiplication:**
```json
{"*": [2, 3, 4]}           // → 24
{"*": [5, 6]}              // → 30
```

**Calculate total price:**
```json
// Data: {"quantity": 5, "price": 10}
{"*": [{"var": "quantity"}, {"var": "price"}]}
// → 50
```

**Apply percentage:**
```json
{"*": [{"var": "amount"}, 0.15]}  // 15% of amount
```

**Empty array:**
```json
{"*": []}                  // → 0 (special case for compatibility)
```

### Optimization
Nested multiplications are automatically flattened:
```json
{"*": [{"*": [2, 3]}, 4]}  // Optimized to: {"*": [2, 3, 4]}
```

---

## `/` - Division

Divides the first value by subsequent values.

### Syntax
```json
{"/": [dividend, divisor, ...]}
```

### Parameters
- **values** (array): Numbers for division

### Return Type
Number - Result of division, or `null` if division by zero

### Examples

**Basic division:**
```json
{"/": [10, 2]}             // → 5
{"/": [100, 5, 2]}         // → 10 (100 / 5 / 2)
```

**Calculate average:**
```json
{"/": [
  {"sum": [{"var": "numbers"}]},
  {"length": {"var": "numbers"}}
]}
```

**Division by zero:**
```json
{"/": [10, 0]}             // → null
```

**Empty array:**
```json
{"/": []}                  // → 0
```

**Percentage calculation:**
```json
// Calculate percentage: (part / total) * 100
{"*": [
  {"/": [{"var": "part"}, {"var": "total"}]},
  100
]}
```

---

## `%` - Modulo

Returns the remainder of division.

### Syntax
```json
{"%": [dividend, divisor]}
```

### Parameters
- **dividend** (number): Value to divide
- **divisor** (number): Value to divide by

### Return Type
Number - Remainder, or `null` if divisor is zero

### Examples

**Basic modulo:**
```json
{"%": [7, 3]}              // → 1
{"%": [10, 5]}             // → 0
{"%": [15, 4]}             // → 3
```

**Even/odd check:**
```json
{"==": [{"%": [{"var": "number"}, 2]}, 0]}
// Returns true if even, false if odd
```

**Cycle through values:**
```json
{"%": [{"var": "counter"}, 10]}
// Returns 0-9 cyclically
```

**With floats:**
```json
{"%": [7.5, 2]}            // → 1.5
```

**Modulo by zero:**
```json
{"%": [5, 0]}              // → null
```

---

## `^` / `pow` - Power

Raises a number to a power.

### Syntax
```json
{"^": [base, exponent]}
{"pow": [base, exponent]}
```

### Parameters
- **base** (number): Base number
- **exponent** (number): Power to raise to

### Return Type
Number - Result of base^exponent

### Examples

**Basic power:**
```json
{"^": [2, 3]}              // → 8 (2³)
{"pow": [10, 2]}           // → 100 (10²)
```

**Square root:**
```json
{"pow": [9, 0.5]}          // → 3 (√9)
{"pow": [16, 0.5]}         // → 4 (√16)
```

**Cube root:**
```json
{"pow": [27, 0.333333]}    // ≈ 3 (∛27)
```

**Negative exponents:**
```json
{"pow": [2, -1]}           // → 0.5 (1/2)
{"pow": [10, -2]}          // → 0.01 (1/100)
```

**Zero power:**
```json
{"pow": [5, 0]}            // → 1 (any number⁰ = 1)
```

**Compound interest:**
```json
// A = P(1 + r)^t
{"*": [
  {"var": "principal"},
  {"pow": [
    {"+": [1, {"var": "rate"}]},
    {"var": "years"}
  ]}
]}
```

---

## Complex Examples

### Calculate Total with Tax
```json
{"+": [
  {"*": [{"var": "subtotal"}, {"+": [1, {"var": "taxRate"}]}]},
  {"var": "shipping"}
]}
```

### Discount Calculation
```json
{"-": [
  {"var": "originalPrice"},
  {"*": [
    {"var": "originalPrice"},
    {"/": [{"var": "discountPercent"}, 100]}
  ]}
]}
```

### Average of Array
```json
{"/": [
  {"reduce": [
    {"var": "numbers"},
    {"+": [{"var": "accumulator"}, {"var": "current"}]},
    0
  ]},
  {"length": {"var": "numbers"}}
]}
```

### Compound Growth Rate
```json
// Final = Initial * (1 + rate)^years
{"*": [
  {"var": "initial"},
  {"pow": [
    {"+": [1, {"var": "growthRate"}]},
    {"var": "years"}
  ]}
]}
```

### Check if Multiple
```json
// Check if x is multiple of y
{"==": [
  {"%": [{"var": "x"}, {"var": "y"}]},
  0
]}
```

### Proportional Distribution
```json
// Distribute amount proportionally
{"*": [
  {"var": "totalAmount"},
  {"/": [
    {"var": "itemValue"},
    {"var": "sumOfAllValues"}
  ]}
]}
```

### Pythagorean Theorem
```json
// c = √(a² + b²)
{"pow": [
  {"+": [
    {"pow": [{"var": "a"}, 2]},
    {"pow": [{"var": "b"}, 2]}
  ]},
  0.5
]}
```

---

## Type Coercion

All arithmetic operators coerce values to numbers:

```json
{"+": ["10", "20"]}        // → 30 (strings to numbers)
{"+": [true, false]}       // → 1 (true=1, false=0)
{"+": [null, 5]}           // → 5 (null=0)
{"*": ["5", 2]}            // → 10
```

---

## Special Cases

### NaN Handling
With `safe_nan_handling` enabled:
```json
{"pow": [-1, 0.5]}         // → 0 (√-1 = NaN → 0)
{"+": [NaN, 5]}            // → 5 (NaN treated as 0)
```

### Infinity
```json
{"/": [1, 0]}              // → null (not Infinity)
{"*": [1e308, 10]}         // → Very large number or Infinity
```

### Precision
Numbers use f64 precision:
```json
{"+": [0.1, 0.2]}          // → 0.30000000000000004 (floating point)
```

For exact decimal precision, consider rounding:
```json
{"round": [{"+": [0.1, 0.2]}, 2]}  // → 0.3
```

---

## Best Practices

1. **Use parentheses** (nesting) for order of operations
   ```json
   {"*": [{"+": [a, b]}, c]}  // (a + b) * c
   ```

2. **Check for division by zero**
   ```json
   {"if": [
     {"!=": [{"var": "divisor"}, 0]},
     {"/": [{"var": "dividend"}, {"var": "divisor"}]},
     null
   ]}
   ```

3. **Round financial calculations**
   ```json
   {"round": [{"*": [price, quantity]}, 2]}
   ```

4. **Use pow for roots**
   ```json
   {"pow": [value, 0.5]}  // Square root
   ```

5. **Leverage automatic flattening**
   ```json
   {"+": [a, b, c]}  // Automatically optimized
   ```

---

## Related Operators

- **[Math Functions](operators-math.md)** - `abs`, `round`, `min`, `max`
- **[Array Operations](operators-array.md)** - `sum`, `reduce` for aggregation
- **[Comparison Operators](operators-comparison.md)** - Compare results

---

## Performance Notes

- **Automatic flattening** optimizes nested operations during compilation
- **Type coercion** happens efficiently at runtime
- **Division by zero** returns `null` without error
- **Fast path optimization** for operations with ≤5 items
