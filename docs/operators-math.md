---
layout: default
title: Math Functions
---

# Math Functions

Advanced mathematical functions and rounding operations.

## Overview

Math functions provide advanced numerical operations beyond basic arithmetic, including rounding, absolute values, min/max comparisons, and power calculations. These functions are essential for financial calculations, statistical operations, and precise numeric manipulation.

### Common Use Cases

- **Financial Calculations**: Precise currency rounding with `round`, `roundup`, `rounddown`
- **Range Operations**: Find minimum/maximum values with `min`, `max`
- **Statistical Analysis**: Calculate absolute deviations with `abs`
- **Scientific Computing**: Power and exponential calculations with `pow`
- **Data Binning**: Round to specific multiples with `mround`, `ceiling`, `floor`
- **Truncation**: Remove decimal places without rounding using `trunc`

### Math Function Categories

1. **Rounding**: `round`, `roundup`, `rounddown`, `trunc` - Control decimal precision
2. **Multiple-based**: `ceiling`, `floor`, `mround` - Round to specific multiples
3. **Comparison**: `min`, `max` - Find extreme values
4. **Transformation**: `abs`, `pow` - Absolute values and exponentiation

### Rounding Modes Explained

Different rounding modes serve different purposes:

- **`round`**: Standard rounding (banker's rounding) - most common use
- **`roundup`**: Always rounds away from zero - for conservative estimates, page counts
- **`rounddown`**: Always rounds toward zero - for discounts, floor prices
- **`ceiling`**: Rounds up to next multiple - for pricing tiers
- **`floor`**: Rounds down to previous multiple - for quantity discounts
- **`trunc`**: Removes decimals without rounding - for integer extraction
- **`mround`**: Rounds to nearest multiple - for time intervals, unit quantities

### Excel Compatibility

All uppercase function names (`ROUND`, `CEILING`, `FLOOR`, etc.) maintain Excel compatibility for seamless formula migration.

## `abs` - Absolute Value

Returns the absolute value of a number.

### Syntax
```json
{"abs": value}
```

### Parameters
- **value** (number): Number to get absolute value of

### Return Type
Number - Absolute value (always non-negative)

### Examples

**Basic absolute:**
```json
{"abs": -5}                // → 5
{"abs": 10}                // → 10
{"abs": 0}                 // → 0
```

**With variables:**
```json
// Data: {"temperature": -15}
{"abs": {"var": "temperature"}}  // → 15
```

**Distance calculation:**
```json
{"abs": {"-": [{"var": "x1"}, {"var": "x2"}]}}
```

---

## `max` - Maximum Value

Returns the largest value from a list.

### Syntax
```json
{"max": [value1, value2, ...]}
```

### Parameters
- **values** (array): Numbers to compare

### Return Type
Number - Largest value, or `null` if empty array

### Examples

**Basic max:**
```json
{"max": [1, 5, 3, 9, 2]}         // → 9
{"max": [-10, -5, -20]}          // → -5
```

**Two values:**
```json
{"max": [{"var": "a"}, {"var": "b"}]}
```

**Empty array:**
```json
{"max": []}                      // → null
```

**Find highest score:**
```json
// Data: {"scores": [85, 92, 78, 95, 88]}
{"reduce": [
  {"var": "scores"},
  {"max": [{"var": "accumulator"}, {"var": "current"}]},
  0
]}
```

---

## `min` - Minimum Value

Returns the smallest value from a list.

### Syntax
```json
{"min": [value1, value2, ...]}
```

### Parameters
- **values** (array): Numbers to compare

### Return Type
Number - Smallest value, or `null` if empty array

### Examples

**Basic min:**
```json
{"min": [1, 5, 3, 9, 2]}         // → 1
{"min": [-10, -5, -20]}          // → -20
```

**Clamp value:**
```json
{"min": [{"var": "input"}, 100]}  // Max value of 100
```

---

## `pow` / `**` - Power

Raises a number to a power. Alternative to `^` operator.

### Syntax
```json
{"pow": [base, exponent]}
{"**": [base, exponent]}
```

### Parameters
- **base** (number): Base number
- **exponent** (number): Power to raise to

### Return Type
Number - base^exponent

### Examples

**Basic power:**
```json
{"pow": [2, 3]}                  // → 8
{"pow": [10, 2]}                 // → 100
```

**Square root:**
```json
{"pow": [16, 0.5]}               // → 4
```

**Cube:**
```json
{"pow": [3, 3]}                  // → 27
```

---

## `round` / `ROUND` - Round Number

Rounds a number to specified decimal places.

### Syntax
```json
{"round": [value, decimals]}
{"ROUND": [value]}
```

### Parameters
- **value** (number): Number to round
- **decimals** (number, optional): Decimal places (default: 0)
  - Positive: decimal places
  - Zero: round to integer
  - Negative: round to left of decimal

### Return Type
Number - Rounded value

### Examples

**Round to integer:**
```json
{"round": [3.7]}                 // → 4
{"ROUND": [3.2]}                 // → 3
{"round": [3.5]}                 // → 4
```

**Round to decimals:**
```json
{"ROUND": [3.14159, 2]}          // → 3.14
{"round": [2.71828, 3]}          // → 2.718
```

**Round to tens:**
```json
{"ROUND": [1234, -1]}            // → 1230
{"round": [1567, -2]}            // → 1600
```

**Financial rounding:**
```json
{"ROUND": [
  {"*": [{"var": "price"}, {"var": "quantity"}]},
  2
]}
```

### Notes
Uses "round half to even" (banker's rounding) by default.

---

## `roundup` / `ROUNDUP` - Round Up

Always rounds away from zero.

### Syntax
```json
{"roundup": [value, decimals]}
{"ROUNDUP": [value, decimals]}
```

### Parameters
- **value** (number): Number to round up
- **decimals** (number, optional): Decimal places (default: 0)

### Return Type
Number - Rounded up value

### Examples

**Round up to integer:**
```json
{"roundup": [3.1]}               // → 4
{"ROUNDUP": [3.9]}               // → 4
{"roundup": [-3.1]}              // → -4 (away from zero)
```

**Round up to decimals:**
```json
{"ROUNDUP": [3.14159, 2]}        // → 3.15
{"roundup": [2.001, 2]}          // → 2.01
```

**Round up to tens:**
```json
{"ROUNDUP": [1234, -1]}          // → 1240
{"roundup": [1001, -2]}          // → 1100
```

**Calculate pages:**
```json
// Total pages needed
{"ROUNDUP": [
  {"/": [{"var": "items"}, {"var": "perPage"}]},
  0
]}
```

---

## `rounddown` / `ROUNDDOWN` - Round Down

Always rounds toward zero.

### Syntax
```json
{"rounddown": [value, decimals]}
{"ROUNDDOWN": [value, decimals]}
```

### Parameters
- **value** (number): Number to round down
- **decimals** (number, optional): Decimal places (default: 0)

### Return Type
Number - Rounded down value

### Examples

**Round down to integer:**
```json
{"rounddown": [3.9]}             // → 3
{"ROUNDDOWN": [3.1]}             // → 3
{"rounddown": [-3.9]}            // → -3 (toward zero)
```

**Round down to decimals:**
```json
{"ROUNDDOWN": [3.14159, 2]}      // → 3.14
{"rounddown": [2.999, 2]}        // → 2.99
```

---

## `ceiling` / `CEILING` - Ceiling

Rounds up to nearest multiple of significance.

### Syntax
```json
{"ceiling": [value, significance]}
{"CEILING": [value]}
```

### Parameters
- **value** (number): Number to round
- **significance** (number, optional): Multiple to round to (default: 1)

### Return Type
Number - Rounded up to nearest multiple

### Examples

**Round up to integer:**
```json
{"ceiling": [4.3]}               // → 5
{"CEILING": [4.1]}               // → 5
```

**Round to specific multiple:**
```json
{"CEILING": [4.3, 0.5]}          // → 4.5
{"ceiling": [123, 10]}           // → 130
{"ceiling": [7, 3]}              // → 9 (next multiple of 3)
```

**Price rounding:**
```json
{"CEILING": [{"var": "price"}, 0.99]}  // Round to .99
```

---

## `floor` / `FLOOR` - Floor

Rounds down to nearest multiple of significance.

### Syntax
```json
{"floor": [value, significance]}
{"FLOOR": [value]}
```

### Parameters
- **value** (number): Number to round
- **significance** (number, optional): Multiple to round to (default: 1)

### Return Type
Number - Rounded down to nearest multiple

### Examples

**Round down to integer:**
```json
{"floor": [4.9]}                 // → 4
{"FLOOR": [4.1]}                 // → 4
```

**Round to specific multiple:**
```json
{"FLOOR": [4.7, 0.5]}            // → 4.5
{"floor": [123, 10]}             // → 120
{"floor": [8, 3]}                // → 6 (previous multiple of 3)
```

---

## `trunc` / `TRUNC` - Truncate

Truncates a number to specified decimal places (removes fractional part).

### Syntax
```json
{"trunc": [value, decimals]}
{"TRUNC": [value]}
```

### Parameters
- **value** (number): Number to truncate
- **decimals** (number, optional): Decimal places (default: 0)

### Return Type
Number - Truncated value

### Examples

**Truncate to integer:**
```json
{"trunc": [8.9]}                 // → 8
{"TRUNC": [-8.9]}                // → -8
```

**Truncate to decimals:**
```json
{"TRUNC": [8.9876, 2]}           // → 8.98
{"trunc": [-3.789, 1]}           // → -3.7
```

**Truncate to tens:**
```json
{"TRUNC": [123.456, -1]}         // → 120
```

**Get integer part:**
```json
{"trunc": [{"var": "decimal"}]}  // Remove fractional part
```

---

## `mround` / `MROUND` - Round to Multiple

Rounds to the nearest multiple of a specified value.

### Syntax
```json
{"mround": [value, multiple]}
{"MROUND": [value, multiple]}
```

### Parameters
- **value** (number): Number to round
- **multiple** (number): Multiple to round to

### Return Type
Number - Nearest multiple

### Examples

**Round to nearest multiple:**
```json
{"mround": [10, 3]}              // → 9
{"MROUND": [11, 3]}              // → 12
{"mround": [7.5, 2]}             // → 8
```

**Round to nearest 5:**
```json
{"MROUND": [13, 5]}              // → 15
{"mround": [12, 5]}              // → 10
```

**Round to decimal multiple:**
```json
{"mround": [1.23, 0.1]}          // → 1.2
{"MROUND": [0.567, 0.05]}        // → 0.55
```

**Time rounding (15 minutes):**
```json
{"MROUND": [{"var": "minutes"}, 15]}
```

---

## Complex Examples

### Percentage Calculation
```json
{"round": [
  {"*": [
    {"/": [{"var": "part"}, {"var": "total"}]},
    100
  ]},
  1
]}
// Result: percentage with 1 decimal place
```

### Calculate Tax
```json
{"round": [
  {"*": [{"var": "amount"}, {"var": "taxRate"}]},
  2
]}
```

### Clamp Value to Range
```json
{"max": [
  {"min": [{"var": "value"}, {"var": "max"}]},
  {"var": "min"}
]}
// Ensures value is between min and max
```

### Convert to Display Units
```json
{
  "if": [
    {">": [{"var": "bytes"}, 1073741824]},
    {"cat": [
      {"round": [{"/": [{"var": "bytes"}, 1073741824]}, 2]},
      " GB"
    ]},
    {"cat": [
      {"round": [{"/": [{"var": "bytes"}, 1048576]}, 2]},
      " MB"
    ]}
  ]
}
```

### Calculate Compound Interest
```json
{"round": [
  {"*": [
    {"var": "principal"},
    {"-": [
      {"pow": [{"+": [1, {"var": "rate"}]}, {"var": "years"}]},
      1
    ]}
  ]},
  2
]}
```

### Average with Rounding
```json
{"round": [
  {"/": [
    {"sum": [{"var": "values"}]},
    {"length": {"var": "values"}}
  ]},
  2
]}
```

### Price Tier Rounding
```json
{"if": [
  {"<": [{"var": "price"}, 10]},
  {"ceiling": [{"var": "price"}, 0.99]},
  {"ceiling": [{"var": "price"}, 5]}
]}
```

---

## Best Practices

1. **Always round financial calculations**
   ```json
   {"round": [amount, 2]}  // 2 decimals for currency
   ```

2. **Use appropriate rounding method**
   - `round` - General purpose
   - `roundup` - Always round up (pages, inventory)
   - `rounddown` - Always round down (discounts)
   - `trunc` - Remove decimals without rounding

3. **Round early and often** in multi-step calculations
   ```json
   {"round": [
     {"+": [
       {"round": [step1, 2]},
       {"round": [step2, 2]}
     ]},
     2
   ]}
   ```

4. **Use ceiling/floor for discretization**
   ```json
   {"ceiling": [value, binSize]}
   ```

5. **Combine min/max for clamping**
   ```json
   {"max": [minValue, {"min": [value, maxValue]}]}
   ```

---

## Troubleshooting

### Issue: Floating point precision errors

**Problem:** Decimal calculations show unexpected results like `0.30000000000000004`.

**Common causes:**
1. **Binary floating point representation** - 0.1 + 0.2 ≠ 0.3 exactly
2. **Accumulated rounding errors** - Multiple operations compound errors

**Solutions:**
```json
// ❌ Precision issue
{"+": [0.1, 0.2]}  // → 0.30000000000000004

// ✅ Round to expected precision
{"round": [{"+": [0.1, 0.2]}, 2]}  // → 0.3

// ✅ Always round financial calculations
{"round": [
  {"*": [{"var": "price"}, {"var": "quantity"}]},
  2
]}
```

### Issue: Round/roundup/rounddown giving same result

**Problem:** Different rounding functions produce identical output.

**Common causes:**
1. **Already at desired precision** - Number has fewer decimals than requested
2. **Result happens to be same** - Value at midpoint might round same way

**Solutions:**
```json
// These will differ:
{"round": [2.5]}     // → 2 (banker's rounding, rounds to even)
{"roundup": [2.5]}   // → 3 (always rounds away from zero)
{"rounddown": [2.5]} // → 2 (always rounds toward zero)

// ✅ Test with values that show difference
{"round": [2.1]}     // → 2
{"roundup": [2.1]}   // → 3 (rounds up)
{"rounddown": [2.1]} // → 2
```

### Issue: Negative number rounding confusion

**Problem:** Rounding negative numbers produces unexpected results.

**Explanation:** Different rounding functions treat negatives differently:

```json
// -3.5 rounding:
{"round": [-3.5]}     // → -4 (banker's rounding)
{"roundup": [-3.5]}   // → -4 (away from zero = more negative)
{"rounddown": [-3.5]} // → -3 (toward zero = less negative)
{"ceiling": [-3.5]}   // → -3 (toward positive infinity)
{"floor": [-3.5]}     // → -4 (toward negative infinity)

// ✅ For consistent "round up" behavior regardless of sign:
{"ceiling": [value]}  // Always toward positive infinity

// ✅ For "round toward zero" (truncate):
{"trunc": [value]}    // Removes decimals
```

### Issue: Min/Max returns null

**Problem:** `min` or `max` returns null instead of expected value.

**Common causes:**
1. **Empty array** - No values to compare
2. **All null values** - No valid numbers in array
3. **Wrong nesting** - Array of arrays instead of flat array

**Solutions:**
```json
// ❌ Empty array
{"max": []}  // → null

// ✅ Provide default for empty arrays
{"ifnull": [
  {"max": [{"var": "values"}]},
  0  // Default if empty
]}

// ❌ Wrong structure
{"max": [[1, 2], [3, 4]]}  // → null (array of arrays)

// ✅ Flatten first
{"max": [{"merge": [[1, 2], [3, 4]]}]}  // → 4
```

### Issue: Ceiling/Floor with negative significance

**Problem:** Using negative significance produces unexpected results.

**Explanation:** Significance sign affects rounding direction:

```json
// Positive significance
{"ceiling": [123, 10]}   // → 130 (rounds up)
{"floor": [123, 10]}     // → 120 (rounds down)

// Negative significance behavior
{"ceiling": [123, -10]}  // → 120 (!)
{"floor": [123, -10]}    // → 130 (!)

// ✅ Use absolute value for significance
{"ceiling": [value, {"abs": [significance]}]}
```

### Issue: Pow with large exponents crashes or returns Infinity

**Problem:** Power calculations overflow or return Infinity.

**Solutions:**
```json
// ❌ Overflow
{"pow": [10, 1000]}  // → Infinity

// ✅ Check for reasonable bounds
{"if": [
  {">": [exponent, 100]},
  {"return": "ERROR: Exponent too large"},
  {"pow": [base, exponent]}
]}

// ✅ Use logarithms for very large powers
// Instead of a^b, calculate using log: e^(b * ln(a))
```

### Issue: Mround returns incorrect multiples

**Problem:** `mround` doesn't round to expected multiple.

**Common causes:**
1. **Floating point precision** - Rounding errors in multiple
2. **Wrong multiple value** - Using wrong unit

**Solutions:**
```json
// ❌ Precision issue with decimal multiples
{"mround": [1.27, 0.1]}  // Might not be exact

// ✅ Round the multiple too
{"mround": [
  {"round": [value, 2]},
  {"round": [0.1, 2]}
]}

// ✅ For time rounding (15-minute intervals):
{"mround": [{"var": "minutes"}, 15]}  // → 0, 15, 30, 45, 60...
```

### Issue: Abs returns negative number

**Problem:** This should never happen. If it does:

**Causes:**
1. **Not using abs correctly** - Applied to wrong value
2. **Type coercion issue** - String "-5" might not be converted

**Solutions:**
```json
// ✅ Ensure value is a number
{"abs": [{"ToNumber": [{"var": "value"}]}]}

// ✅ Double-check the operation
{"abs": {"-": [a, b]}}  // Not {"abs": value}
```

---

## Excel Compatibility

All operators marked with uppercase names (ROUND, CEILING, etc.) are Excel-compatible:

- **ROUND** - Banker's rounding (round half to even)
- **ROUNDUP** - Always away from zero
- **ROUNDDOWN** - Always toward zero
- **CEILING** - Round up to multiple
- **FLOOR** - Round down to multiple
- **TRUNC** - Truncate decimals
- **MROUND** - Round to nearest multiple

---

## Related Operators

- **[Arithmetic Operators](operators-arithmetic.md)** - Basic math
- **[Comparison Operators](operators-comparison.md)** - Compare results
- **[Array Operators](operators-array.md)** - `sum`, `reduce` for aggregations

---

## Performance Notes

- **Rounding** uses native Rust implementations
- **Special case handling** for NaN/Infinity with `safe_nan_handling`
- **Integer operations** optimized when decimals=0
