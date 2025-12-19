---
layout: default
title: Common Mistakes and How to Avoid Them
---

# Common Mistakes and How to Avoid Them

A quick reference guide to common pitfalls when working with JSON-Eval-RS operators and how to avoid them.

## Table of Contents

- [Variable Access Mistakes](#variable-access-mistakes)
- [Array Operation Mistakes](#array-operation-mistakes)
- [Arithmetic Mistakes](#arithmetic-mistakes)
- [String Operation Mistakes](#string-operation-mistakes)
- [Date Operation Mistakes](#date-operation-mistakes)
- [Comparison Mistakes](#comparison-mistakes)
- [Logic Mistakes](#logic-mistakes)

---

## Variable Access Mistakes

### ❌ Using wrong variable reference in array operations

```json
// WRONG - referencing non-existent field
{"map": [
  [1, 2, 3],
  {"*": [{"var": "value"}, 2]}  // Looking for "value" field
]}

// CORRECT - use empty string for current element
{"map": [
  [1, 2, 3],
  {"*": [{"var": ""}, 2]}
]}
```

**Why:** In `map`, `filter`, `all`, `some`, `none`, the current element is accessed with `{"var": ""}`.

---

### ❌ Forgetting to provide default values

```json
// WRONG - might return null unexpectedly
{"var": "optional.field"}

// CORRECT - provide default
{"var": ["optional.field", "default value"]}

// OR use ifnull
{"ifnull": [{"var": "optional.field"}, "default"]}
```

**Why:** Missing fields return `null` which might cause issues in calculations.

---

## Array Operation Mistakes

### ❌ Wrong reduce initial value type

```json
// WRONG - initial value doesn't match accumulator type
{"reduce": [
  array,
  {"merge": [{"var": "accumulator"}, [{"var": "current"}]]},
  0  // Should be [] for array accumulator!
]}

// CORRECT - match types
{"reduce": [
  array,
  {"merge": [{"var": "accumulator"}, [{"var": "current"}]]},
  []  // Array initial value
]}
```

**Why:** The initial value type must match what you're building.

---

### ❌ Adding separator before first item in reduce

```json
// WRONG - results in ",a,b,c"
{"reduce": [
  ["a", "b", "c"],
  {"cat": [{"var": "accumulator"}, ",", {"var": "current"}]},
  ""
]}

// CORRECT - conditional separator
{"reduce": [
  ["a", "b", "c"],
  {"cat": [
    {"var": "accumulator"},
    {"if": [{">": [{"length": {"var": "accumulator"}}, 0]}, ",", ""]},
    {"var": "current"}
  ]},
  ""
]}
```

**Why:** Separator is added before checking if accumulator is empty.

---

### ❌ Confusing all/some/none with empty arrays

```json
{"all": [[], condition]}   // → true (vacuously true - no counterexample)
{"some": [[], condition]}  // → false (no element matches)
{"none": [[], condition]}  // → true (no element matches)
```

**Best Practice:** Always check if array is non-empty first when logic requires it:

```json
{"if": [
  {">": [{"length": array}, 0]},
  {"all": [array, condition]},
  defaultValue
]}
```

---

## Arithmetic Mistakes

### ❌ Floating point precision in financial calculations

```json
// WRONG - precision error
{"+": [0.1, 0.2]}  // → 0.30000000000000004

// CORRECT - always round
{"round": [{"+": [0.1, 0.2]}, 2]}  // → 0.3
```

**Why:** Binary floating point can't represent all decimals exactly.

---

### ❌ Not handling division by zero

```json
// WRONG - returns null unexpectedly
{"/": [{"var": "total"}, {"var": "count"}]}  // null if count=0

// CORRECT - check first
{"if": [
  {"!=": [{"var": "count"}, 0]},
  {"/": [{"var": "total"}, {"var": "count"}]},
  0
]}

// OR use ifnull
{"ifnull": [
  {"/": [{"var": "total"}, {"var": "count"}]},
  0
]}
```

**Why:** Division by zero returns `null` instead of throwing an error.

---

### ❌ Percentage as whole number instead of decimal

```json
// WRONG - using 10 for 10%
{"*": [100, 10]}  // → 1000 (expected 10)

// CORRECT - use decimal
{"*": [100, 0.1]}  // → 10

// OR divide by 100
{"*": [100, {"/": [10, 100]}]}  // → 10
```

**Why:** 10% means 0.1, not 10.

---

### ❌ Using + for string concatenation

```json
// WRONG - coerces strings to numbers
{"+": ["Hello", " World"]}  // → NaN or 0

// CORRECT - use cat
{"cat": ["Hello", " World"]}  // → "Hello World"
```

**Why:** Arithmetic operators always coerce to numbers.

---

## String Operation Mistakes

### ❌ Wrong parameter order in search

```json
// WRONG - parameters swapped
{"search": ["Hello World", "World"]}  // Wrong order

// CORRECT - search text first, within text second
{"search": ["World", "Hello World"]}  // → 7
```

**Why:** First parameter is what to find, second is where to search.

---

### ❌ Confusing 0-based vs 1-based indexing

```json
// splittext uses 0-based indexing
{"splittext": ["a,b,c", ",", 0]}  // → "a" (first element)
{"splittext": ["a,b,c", ",", 2]}  // → "c" (third element)

// search returns 1-based positions
{"search": ["World", "Hello World"]}  // → 7 (position 7, not 6)
```

**Why:** Different operators follow different conventions.

---

### ❌ Not handling null in concatenation

```json
// WRONG - produces "Hello null"
{"cat": ["Hello ", {"var": "name"}]}  // If name is null

// CORRECT - provide default
{"cat": [
  "Hello ",
  {"ifnull": [{"var": "name"}, "Guest"]}
]}
```

**Why:** `cat` converts null to the string "null".

---

## Date Operation Mistakes

### ❌ Simple year subtraction for age

```json
// WRONG - doesn't account for birthday
{"-": [
  {"year": {"today": null}},
  {"year": {"var": "birthdate"}}
]}

// CORRECT - use DATEDIF
{"DATEDIF": [{"var": "birthdate"}, {"today": null}, "Y"]}
```

**Why:** Age depends on whether birthday has occurred this year.

---

### ❌ Wrong parameter order in days

```json
// WRONG - returns negative
{"days": [{"var": "startDate"}, {"var": "endDate"}]}  // → -14

// CORRECT - end date first
{"days": [{"var": "endDate"}, {"var": "startDate"}]}  // → 14
```

**Why:** `days([end, start])` returns `end - start` in days.

---

### ❌ Using non-ISO date formats

```json
// WRONG - ambiguous formats
{"year": "01/15/2024"}  // MM/DD or DD/MM?
{"year": "15-01-2024"}  // Wrong separator

// CORRECT - use ISO 8601
{"year": "2024-01-15"}  // → 2024
```

**Why:** Only ISO 8601 format is guaranteed to parse correctly.

---

### ❌ Confusing DATEDIF unit codes

```json
// "M" vs "YM" confusion
{"DATEDIF": ["1990-03-15", "2023-07-10", "M"]}   // → 399 (total months)
{"DATEDIF": ["1990-03-15", "2023-07-10", "YM"]}  // → 3 (months remainder)

// For "33 years, 3 months" display:
{"cat": [
  {"DATEDIF": [start, end, "Y"]}, " years, ",
  {"DATEDIF": [start, end, "YM"]}, " months"
]}
```

**Why:** "M" = total months, "YM" = months ignoring years.

---

## Comparison Mistakes

### ❌ Using == when types matter

```json
// WRONG - type coercion
{"==": [0, false]}     // → true (unexpected)
{"==": [1, "1"]}       // → true (unexpected)

// CORRECT - use strict equality
{"===": [0, false]}    // → false
{"===": [1, "1"]}      // → false
```

**Why:** `==` performs type coercion, `===` requires exact type match.

---

### ❌ Comparing strings as numbers

```json
// WRONG - lexicographic comparison
{"<": ["10", "2"]}  // → true (string comparison: "1" < "2")

// CORRECT - convert to numbers
{"<": [
  {"ToNumber": ["10"]},
  {"ToNumber": ["2"]}
]}  // → false (10 > 2)
```

**Why:** Comparison operators don't automatically convert strings to numbers for ordering.

---

### ❌ Not checking for null before comparison

```json
// WRONG - null comparisons might fail
{">": [{"var": "age"}, 18]}  // Fails if age is null

// CORRECT - validate first
{"and": [
  {"!==": [{"var": "age"}, null]},
  {">": [{"var": "age"}, 18]}
]}
```

**Why:** Null comparisons can produce unexpected results.

---

## Logic Mistakes

### ❌ Not leveraging short-circuit evaluation

```json
// INEFFICIENT - both conditions always evaluated
{"if": [
  {"and": [expensive_check, cheap_check]},
  ...
]}

// BETTER - cheap check first
{"if": [
  {"and": [cheap_check, expensive_check]},
  ...
]}
```

**Why:** `and` short-circuits on first falsy value.

---

### ❌ Nested if instead of else-if pattern

```json
// WRONG - deeply nested
{"if": [
  cond1, val1,
  {"if": [
    cond2, val2,
    {"if": [cond3, val3, default]}
  ]}
]}

// BETTER - flat structure
{"if": [
  cond1, val1,
  cond2, val2,
  cond3, val3,
  default
]}
```

**Why:** `if` supports else-if chaining naturally.

---

### ❌ Using nested if for null coalescing

```json
// WRONG - verbose
{"if": [
  {"!==": [{"var": "value"}, null]},
  {"var": "value"},
  "default"
]}

// BETTER - use ifnull
{"ifnull": [{"var": "value"}, "default"]}
```

**Why:** `ifnull` is designed for this exact pattern.

---

## Quick Reference: Most Common Mistakes

1. **Using `{"var": "field"}` instead of `{"var": ""}` in array operations**
2. **Forgetting to round financial calculations**
3. **Not checking for division by zero**
4. **Using `==` when types matter (use `===` instead)**
5. **Type coercion surprises (`+` coerces to number, use `cat` for strings)**
6. **Using wrong date format (always use ISO 8601: `"2024-01-15"`)**
7. **Not providing defaults for nullable fields**
8. **Wrong parameter order (`search`, `days`, etc.)**
9. **Floating point precision issues (always `round` decimals)**
10. **Empty array edge cases in `all`/`some`/`none`**

---

## Related Documentation

- **[Operators Summary](OPERATORS_SUMMARY.md)** - Quick reference for all operators
- **[Core Operators](operators-core.md)** - Variable access patterns
- **[Array Operators](operators-array.md)** - Array operation troubleshooting
- **[Arithmetic Operators](operators-arithmetic.md)** - Math operation issues
- **[Comparison Operators](operators-comparison.md)** - Type coercion details
- **[Date Functions](operators-date.md)** - Date handling best practices

---

**Remember:** Most issues come from:
1. Not reading the documentation for parameter order
2. Assuming JavaScript/Excel behavior without checking
3. Not handling null/edge cases
4. Type coercion surprises

When in doubt, check the relevant operator documentation for examples and troubleshooting!
