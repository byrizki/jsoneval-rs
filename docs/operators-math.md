---
layout: default
title: Math Functions
---

# Math Functions

Advanced mathematical functions and rounding operations.

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
