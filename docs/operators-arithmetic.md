---
layout: default
title: Arithmetic Operators
---

# Arithmetic Operators

Mathematical operators for numeric calculations.

## Overview

Arithmetic operators form the foundation of mathematical calculations in JSON-Eval-RS. These operators handle basic mathematical operations with support for multiple operands, automatic type coercion, and compile-time optimizations.

### Common Use Cases

- **Financial Calculations**: Calculate totals, taxes, discounts, and pricing
- **Data Aggregation**: Sum, average, or compute statistical values
- **Unit Conversions**: Convert between different measurement units
- **Percentage Calculations**: Calculate percentages, rates, and ratios
- **Scientific Computing**: Perform basic mathematical formulas
- **Score Calculations**: Compute grades, ratings, or weighted averages

### Arithmetic Operator Categories

1. **Basic Operations**: `+`, `-`, `*`, `/` - Fundamental arithmetic
2. **Advanced**: `%` (modulo), `^` (power) - Specialized calculations

### Key Features

**Multi-operand Support**: Most operators accept multiple values:
```json
{"+": [1, 2, 3, 4, 5]}  // → 15
{"*": [2, 3, 4]}        // → 24
```

**Automatic Flattening**: Nested operations of the same type are optimized:
```json
{"+": [{"+": [1, 2]}, 3]}  // Optimized to: {"+": [1, 2, 3]}
```

**Type Coercion**: Values are automatically converted to numbers:
```json
{"+": ["10", 5]}      // → 15 (string "10" → number 10)
{"+": [true, false]}  // → 1 (true → 1, false → 0)
```

**Safe NaN Handling**: Controlled behavior for invalid operations with `safe_nan_handling` configuration option.

### Operator Precedence

When combining operators, use explicit nesting to ensure correct order:

```json
// Clear precedence through nesting
{"*": [
  {"+": [2, 3]},  // 2 + 3 = 5
  4               // 5 * 4 = 20
]}

// Multiple operations
{"+": [
  {"*": [{"var": "quantity"}, {"var": "price"}]},
  {"/": [{"var": "shipping"}, 2]}
]}
```

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

## Troubleshooting

### Issue: Unexpected floating point precision errors

**Problem:** Decimal arithmetic produces results like `0.30000000000000004` instead of `0.3`.

**Common causes:**
1. **Binary floating point representation** - Inherent limitation of IEEE 754
2. **Accumulated errors** - Multiple operations compound precision issues

**Solutions:**
```json
// ❌ Precision issue
{"+": [0.1, 0.2]}  // → 0.30000000000000004

// ✅ Round to expected precision
{"round": [{"+": [0.1, 0.2]}, 2]}  // → 0.3

// ✅ For financial calculations, always round
{"round": [
  {"*": [
    {"+": [{"var": "price"}, {"var": "tax"}]},
    {"var": "quantity"}
  ]},
  2
]}

// ✅ Use integer arithmetic when possible (cents instead of dollars)
{"/": [
  {"*": [{"var": "priceInCents"}, {"var": "quantity"}]},
  100
]}
```

### Issue: Division by zero returns null

**Problem:** Division by zero doesn't throw error, returns `null` unexpectedly.

**Explanation:** This is by design to prevent runtime errors. Handle explicitly:

```json
// Division by zero behavior
{"/": [10, 0]}  // → null

// ✅ Check before dividing
{"if": [
  {"!=": [{"var": "divisor"}, 0]},
  {"/": [{"var": "dividend"}, {"var": "divisor"}]},
  0  // or appropriate default
]}

// ✅ Use ifnull for default value
{"ifnull": [
  {"/": [{"var": "total"}, {"var": "count"}]},
  0
]}

// ✅ For averages, check array length
{"if": [
  {">": [{"length": {"var": "values"}}, 0]},
  {"/": [
    {"sum": [{"var": "values"}]},
    {"length": {"var": "values"}}
  ]},
  0
]}
```

### Issue: Modulo with negative numbers produces unexpected results

**Problem:** Modulo operator behavior with negatives seems inconsistent.

**Explanation:** Modulo follows standard mathematical conventions:

```json
// Positive modulo
{"%": [7, 3]}    // → 1 (7 mod 3)
{"%": [10, 4]}   // → 2

// Negative dividend
{"%": [-7, 3]}   // → -1 (sign of dividend)
{"%": [-10, 4]}  // → -2

// Negative divisor
{"%": [7, -3]}   // → 1
{"%": [-7, -3]}  // → -1

// ✅ For always-positive modulo (useful for array indices):
{"%": [
  {"+": [
    {"%": [value, length]},
    length
  ]},
  length
]}
```

### Issue: Power operation returns NaN or Infinity

**Problem:** `pow` or `^` returns NaN or Infinity.

**Common causes:**
1. **Negative base with fractional exponent** - √-1 = NaN
2. **Very large exponents** - Overflow to Infinity
3. **0^0 edge case** - Mathematically undefined

**Solutions:**
```json
// ❌ Invalid operations
{"pow": [-1, 0.5]}    // → NaN (√-1)
{"pow": [10, 1000]}   // → Infinity (overflow)
{"pow": [0, 0]}       // → 1 (by convention)

// ✅ With safe_nan_handling enabled
{"pow": [-1, 0.5]}    // → 0 (NaN converted to 0)

// ✅ Validate before calculation
{"if": [
  {"and": [
    {">=": [base, 0]},
    {"<=": [exponent, 100]}
  ]},
  {"pow": [base, exponent]},
  {"return": "Invalid calculation"}
]}

// ✅ Use absolute value for even roots
{"pow": [{"abs": [value]}, 0.5]}  // Always valid
```

### Issue: Subtraction with single value returns negative

**Problem:** `{"-": [5]}` returns `-5` instead of error.

**Explanation:** This is the **negation** (unary minus) operator - intentional behavior:

```json
// Negation (single operand)
{"-": [5]}      // → -5
{"-": [-10]}    // → 10

// ✅ Use this for sign inversion
{"-": [{"var": "value"}]}  // Negate variable

// ✅ For absolute difference, use abs
{"abs": [{"-": [a, b]}]}
```

### Issue: Type coercion produces unexpected sums

**Problem:** String concatenation expected but got addition.

**Explanation:** Arithmetic operators always coerce to numbers. Use `cat` for strings:

```json
// ❌ Wrong - expects "1020" but gets 30
{"+": ["10", "20"]}  // → 30 (strings coerced to numbers)

// ✅ Use cat for string concatenation
{"cat": ["10", "20"]}  // → "1020"

// If you want number addition from string inputs:
{"+": [
  {"ToNumber": [{"var": "input1"}]},
  {"ToNumber": [{"var": "input2"}]}
]}

// ✅ Validation before coercion
{"if": [
  {"and": [
    {"!==": [{"var": "a"}, ""]},
    {"!==": [{"var": "b"}, ""]}
  ]},
  {"+": [{"var": "a"}, {"var": "b"}]},
  0
]}
```

### Issue: Multiplication result much larger than expected

**Problem:** Multiplying percentages gives wrong result.

**Common causes:**
1. **Percentage as whole number** - Using 10 instead of 0.1 for 10%
2. **Multiple percentage applications** - Compounding when shouldn't

**Solutions:**
```json
// ❌ Wrong - using percentage as whole number
{"*": [100, 10]}  // → 1000 (expected 10)

// ✅ Correct - divide by 100
{"*": [100, {"/": [10, 100]}]}  // → 10

// ✅ Or use decimal directly
{"*": [100, 0.1]}  // → 10

// ✅ Calculate price with tax
{"*": [
  {"var": "price"},
  {"+": [1, {"/": [{"var": "taxPercent"}, 100]}]}
]}
// For price=$100, tax=10%: 100 * 1.1 = $110

// ❌ Wrong compounding
{"*": [
  {"*": [price, {"+": [1, tax1]}]},
  {"+": [1, tax2]}
]}
// This compounds: price * (1+tax1) * (1+tax2)

// ✅ Correct for additive taxes
{"*": [
  price,
  {"+": [1, tax1, tax2]}
]}
```

### Issue: Empty array operations return unexpected defaults

**Problem:** Operations on empty arrays return 0 or other values.

**Explanation:** Each operator has a defined identity element:

```json
// Identity elements (what empty arrays return)
{"+": []}   // → 0 (additive identity)
{"*": []}   // → 1 (multiplicative identity)
{"-": []}   // → 0
{"/": []}   // → null

// ✅ Handle empty cases explicitly
{"if": [
  {">": [{"length": {"var": "values"}}, 0]},
  {"+": [{"var": "values"}]},
  null  // Explicit default instead of 0
]}
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
