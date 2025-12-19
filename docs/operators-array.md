---
layout: default
title: Array Operators
---

# Array Operators

Array transformation, filtering, and aggregation operators.

## Overview

Array operators provide powerful functional programming capabilities for working with collections of data. These operators enable you to transform, filter, aggregate, and query arrays without writing imperative loops.

### Common Use Cases

- **Data Transformation**: Convert raw data into desired formats using `map`
- **Filtering**: Select subsets of data matching criteria with `filter`
- **Aggregation**: Calculate totals, averages, or other aggregate values with `reduce` and `sum`
- **Validation**: Check if all/some/no elements meet conditions using `all`, `some`, `none`
- **Data Combination**: Merge multiple arrays with `merge`
- **Generation**: Create sequences or ranges using `FOR`

### Array Operator Categories

1. **Transformation**: `map` - Transform each element
2. **Filtering**: `filter` - Select matching elements
3. **Aggregation**: `reduce`, `sum`, `multiplies`, `divides` - Combine elements
4. **Quantifiers**: `all`, `some`, `none` - Test conditions
5. **Utilities**: `merge`, `in`, `FOR` - Helper operations

### The Current Element Pattern

Most array operators use `{"var": ""}` (empty string) to reference the current element being processed. This is a key pattern to understand:

```json
// In map/filter/all/some/none:
{"map": [array, {"*": [{"var": ""}, 2]}]}
//                      ^^^^ current element

// In reduce, use "accumulator" and "current":
{"reduce": [array, 
  {"+": [{"var": "accumulator"}, {"var": "current"}]},
  0
]}
```

## `map` - Transform Array

Transforms each element in an array using a logic expression.

### Syntax
```json
{"map": [array, logic]}
```

### Parameters
- **array** (array): Array to transform
- **logic** (expression): Logic to apply to each element (use `{"var": ""}` for current element)

### Return Type
Array - Transformed array

### Examples

**Double each number:**
```json
// Data: {"numbers": [1, 2, 3, 4, 5]}
{"map": [
  {"var": "numbers"},
  {"*": [{"var": ""}, 2]}
]}
// → [2, 4, 6, 8, 10]
```

**Extract property:**
```json
// Data: {"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}
{"map": [
  {"var": "users"},
  {"var": "name"}
]}
// → ["Alice", "Bob"]
```

**Format strings:**
```json
{"map": [
  [1, 2, 3],
  {"cat": ["Item ", {"var": ""}]}
]}
// → ["Item 1", "Item 2", "Item 3"]
```

**Empty array:**
```json
{"map": [[], {"*": [{"var": ""}, 2]}]}  // → []
```

---

## `filter` - Filter Array

Filters array elements based on a condition.

### Syntax
```json
{"filter": [array, logic]}
```

### Parameters
- **array** (array): Array to filter
- **logic** (expression): Condition to test each element

### Return Type
Array - Filtered array containing only elements where condition is truthy

### Examples

**Filter even numbers:**
```json
// Data: {"numbers": [1, 2, 3, 4, 5, 6]}
{"filter": [
  {"var": "numbers"},
  {"==": [{"%": [{"var": ""}, 2]}, 0]}
]}
// → [2, 4, 6]
```

**Filter by property:**
```json
// Data: {"users": [
//   {"name": "Alice", "active": true},
//   {"name": "Bob", "active": false},
//   {"name": "Charlie", "active": true}
// ]}
{"filter": [
  {"var": "users"},
  {"var": "active"}
]}
// → [{"name": "Alice", "active": true}, {"name": "Charlie", "active": true}]
```

**Filter by comparison:**
```json
{"filter": [
  {"var": "scores"},
  {">": [{"var": ""}, 70]}
]}
```

---

## `reduce` - Reduce Array

Reduces an array to a single value using an accumulator.

### Syntax
```json
{"reduce": [array, logic, initial]}
```

### Parameters
- **array** (array): Array to reduce
- **logic** (expression): Logic with `accumulator` and `current` variables
- **initial** (any): Initial accumulator value

### Return Type
Any - Final accumulated value

### Examples

**Sum all numbers:**
```json
// Data: {"numbers": [1, 2, 3, 4, 5]}
{"reduce": [
  {"var": "numbers"},
  {"+": [{"var": "accumulator"}, {"var": "current"}]},
  0
]}
// → 15
```

**Find maximum:**
```json
{"reduce": [
  {"var": "numbers"},
  {"if": [
    {">": [{"var": "current"}, {"var": "accumulator"}]},
    {"var": "current"},
    {"var": "accumulator"}
  ]},
  0
]}
```

**Concatenate strings:**
```json
{"reduce": [
  ["a", "b", "c"],
  {"cat": [{"var": "accumulator"}, ",", {"var": "current"}]},
  ""
]}
// → ",a,b,c" (note leading comma)
```

**Count occurrences:**
```json
{"reduce": [
  {"var": "items"},
  {"+": [
    {"var": "accumulator"},
    {"if": [{"==": [{"var": "current"}, "target"]}, 1, 0]}
  ]},
  0
]}
```

---

## `all` - All Elements Match

Tests if all elements satisfy a condition.

### Syntax
```json
{"all": [array, logic]}
```

### Parameters
- **array** (array): Array to test
- **logic** (expression): Condition to test

### Return Type
Boolean - `true` if all elements match, `false` otherwise

### Examples

**All positive:**
```json
{"all": [
  [1, 2, 3, 4, 5],
  {">": [{"var": ""}, 0]}
]}
// → true
```

**All even:**
```json
{"all": [
  [2, 4, 6, 8],
  {"==": [{"%": [{"var": ""}, 2]}, 0]}
]}
// → true
```

**Empty array:**
```json
{"all": [[], {">": [{"var": ""}, 0]}]}  // → true (vacuously true)
```

**All users active:**
```json
{"all": [
  {"var": "users"},
  {"var": "active"}
]}
```

---

## `some` - Some Elements Match

Tests if at least one element satisfies a condition.

### Syntax
```json
{"some": [array, logic]}
```

### Parameters
- **array** (array): Array to test
- **logic** (expression): Condition to test

### Return Type
Boolean - `true` if any element matches, `false` otherwise

### Examples

**Has negative:**
```json
{"some": [
  [1, -2, 3, 4],
  {"<": [{"var": ""}, 0]}
]}
// → true
```

**Has active user:**
```json
{"some": [
  {"var": "users"},
  {"var": "active"}
]}
```

**Empty array:**
```json
{"some": [[], {">": [{"var": ""}, 0]}]}  // → false
```

---

## `none` - No Elements Match

Tests if no elements satisfy a condition.

### Syntax
```json
{"none": [array, logic]}
```

### Parameters
- **array** (array): Array to test
- **logic** (expression): Condition to test

### Return Type
Boolean - `true` if no elements match, `false` otherwise

### Examples

**No negatives:**
```json
{"none": [
  [1, 2, 3, 4, 5],
  {"<": [{"var": ""}, 0]}
]}
// → true
```

**No inactive users:**
```json
{"none": [
  {"var": "users"},
  {"!": {"var": "active"}}
]}
```

**Empty array:**
```json
{"none": [[], {">": [{"var": ""}, 0]}]}  // → true
```

---

## `merge` - Merge Arrays

Flattens and merges multiple arrays into one.

### Syntax
```json
{"merge": [array1, array2, ...]}
```

### Parameters
- **arrays** (array): Arrays to merge

### Return Type
Array - Single merged array

### Examples

**Basic merge:**
```json
{"merge": [[1, 2], [3, 4], [5]]}  // → [1, 2, 3, 4, 5]
```

**Mixed types:**
```json
{"merge": [["a", "b"], [1, 2], [true, null]]}
// → ["a", "b", 1, 2, true, null]
```

**Combine data sources:**
```json
{"merge": [
  {"var": "localItems"},
  {"var": "remoteItems"}
]}
```

---

## `in` - Contains Value

Checks if a value exists in an array or substring in a string.

### Syntax
```json
{"in": [value, array_or_string]}
```

### Parameters
- **value** (any): Value to search for
- **array_or_string** (array|string): Array or string to search in

### Return Type
Boolean - `true` if found, `false` otherwise

### Examples

**Value in array:**
```json
{"in": [3, [1, 2, 3, 4, 5]]}  // → true
{"in": [6, [1, 2, 3, 4, 5]]}  // → false
```

**Substring in string:**
```json
{"in": ["world", "hello world"]}  // → true
{"in": ["foo", "hello world"]}    // → false
```

**Check membership:**
```json
{"in": [
  {"var": "userRole"},
  ["admin", "moderator", "editor"]
]}
```

---

## `sum` / `SUM` - Sum Array

Sums numeric values in an array, with optional field extraction and threshold.

### Syntax
```json
{"sum": [array]}
{"sum": [array, field]}
{"sum": [array, field, threshold]}
{"SUM": [array]}
```

### Parameters
- **array** (array): Array to sum (or single number)
- **field** (string, optional): Field name to extract from objects
- **threshold** (number, optional): Index threshold for stopping

### Return Type
Number - Sum of values

### Examples

**Sum numbers:**
```json
{"sum": [[1, 2, 3, 4, 5]]}  // → 15
{"SUM": [[10, 20, 30]]}     // → 60
```

**Sum object field:**
```json
// Data: {"items": [{"price": 10}, {"price": 20}, {"price": 30}]}
{"sum": [{"var": "items"}, "price"]}  // → 60
```

**With threshold:**
```json
// Sum first 3 elements only
{"sum": [[1, 2, 3, 4, 5], null, 3]}  // → 6
```

**Sum variable:**
```json
{"sum": [{"var": "values"}]}
```

---

## `for` / `FOR` - Loop and Build Array

Creates an array by iterating from start to end.

### Syntax
```json
{"FOR": [start, end, logic]}
```

### Parameters
- **start** (number): Starting value (inclusive)
- **end** (number): Ending value (inclusive)
- **logic** (expression): Logic for each iteration (use `{"var": "$iteration"}`)

### Return Type
Array - Array of results

### Examples

**Generate range:**
```json
{"FOR": [1, 5, {"var": "$iteration"}]}
// → [1, 2, 3, 4, 5]
```

**Generate squares:**
```json
{"FOR": [
  1,
  5,
  {"*": [{"var": "$iteration"}, {"var": "$iteration"}]}
]}
// → [1, 4, 9, 16, 25]
```

**Generate labels:**
```json
{"FOR": [
  0,
  3,
  {"cat": ["Item ", {"var": "$iteration"}]}
]}
// → ["Item 0", "Item 1", "Item 2", "Item 3"]
```

**Build table rows:**
```json
{"FOR": [
  1,
  10,
  {"*": [{"var": "basePrice"}, {"var": "$iteration"}]}
]}
```

---

## `multiplies` / `MULTIPLIES` - Flatten and Multiply

Flattens nested arrays and multiplies all numeric values.

### Syntax
```json
{"multiplies": [value1, value2, ...]}
{"MULTIPLIES": [value1, value2, ...]}
```

### Parameters
- **values** (array): Values or arrays to flatten and multiply

### Return Type
Number - Product of all flattened values

### Examples

**Multiply flat values:**
```json
{"multiplies": [2, 3, 4]}  // → 24
```

**Multiply nested arrays:**
```json
{"MULTIPLIES": [[2, 3], [4, 5]]}  // → 120 (2*3*4*5)
```

---

## `divides` / `DIVIDES` - Flatten and Divide

Flattens nested arrays and divides values sequentially.

### Syntax
```json
{"divides": [value1, value2, ...]}
{"DIVIDES": [value1, value2, ...]}
```

### Parameters
- **values** (array): Values or arrays to flatten and divide

### Return Type
Number - Result of sequential division

### Examples

**Divide values:**
```json
{"divides": [100, 2, 5]}  // → 10 (100/2/5)
```

**Divide nested:**
```json
{"DIVIDES": [[100], [2, 5]]}  // → 10
```

---

## Complex Examples

### Calculate Average
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

### Filter and Transform
```json
{"map": [
  {"filter": [
    {"var": "users"},
    {"var": "active"}
  ]},
  {"var": "name"}
]}
// Get names of active users
```

### Group By Count
```json
{"reduce": [
  {"var": "items"},
  {"+": [
    {"var": "accumulator"},
    {"if": [
      {"==": [{"var": "current.category"}, "A"]},
      1,
      0
    ]}
  ]},
  0
]}
```

### Find First Match
```json
{"reduce": [
  {"var": "users"},
  {"if": [
    {"!=": [{"var": "accumulator"}, null]},
    {"var": "accumulator"},
    {"if": [
      {"==": [{"var": "current.id"}, {"var": "searchId"}]},
      {"var": "current"},
      null
    ]}
  ]},
  null
]}
```

### Validate All Required Fields
```json
{"all": [
  ["name", "email", "age"],
  {"!": {"isempty": {"var": {"var": ""}}}}
]}
```

### Collect Unique Values
```json
{"reduce": [
  {"var": "items"},
  {"if": [
    {"in": [{"var": "current"}, {"var": "accumulator"}]},
    {"var": "accumulator"},
    {"merge": [{"var": "accumulator"}, [{"var": "current"}]]}
  ]},
  []
]}
```

---

## Troubleshooting

### Issue: Empty array returns from map/filter

**Problem:** Operations return empty arrays when you expect results.

**Common causes:**
1. **Wrong variable reference** - Using `{"var": "fieldName"}` instead of `{"var": ""}`
2. **Condition always false** - Logic error in filter predicate
3. **Source array is actually empty** - Input data issue

**Solutions:**
```json
// ❌ Wrong - references non-existent field
{"map": [
  [1, 2, 3],
  {"*": [{"var": "value"}, 2]}  // Looking for "value" field on numbers
]}

// ✅ Correct - uses current element
{"map": [
  [1, 2, 3],
  {"*": [{"var": ""}, 2]}  // Uses current number
]}

// ✅ Check if source is empty first
{"if": [
  {">": [{"length": {"var": "items"}}, 0]},
  {"map": [{"var": "items"}, transform]},
  []
]}
```

### Issue: Reduce returns wrong accumulator type

**Problem:** Reduce returns unexpected type or structure.

**Solution:** Ensure initial value matches expected type:

```json
// ❌ Wrong - initial is number but building array
{"reduce": [
  array,
  {"merge": [{"var": "accumulator"}, [{"var": "current"}]]},
  0  // Should be [] for array
]}

// ✅ Correct - initial matches accumulator type
{"reduce": [
  array,
  {"merge": [{"var": "accumulator"}, [{"var": "current"}]]},
  []  // Correct initial for array accumulator
]}
```

### Issue: Performance slow with large arrays

**Problem:** Operations timeout or are very slow.

**Solutions:**
1. **Avoid nested loops** - Don't use map/filter inside map/filter
2. **Use quantifiers** - `all`/`some`/`none` short-circuit
3. **Filter before map** - Reduce data size early

```json
// ❌ Inefficient - nested operations
{"map": [
  largeArray1,
  {"filter": [largeArray2, condition]}  // O(n²)
]}

// ✅ Better - single pass
{"filter": [
  combinedData,
  condition
]}
```

### Issue: Reduce with string concatenation has extra separators

**Problem:** String concatenation adds separator at the beginning.

**Solution:** Handle first element specially or trim result:

```json
// ❌ Problem - adds comma before first item
{"reduce": [
  ["a", "b", "c"],
  {"cat": [{"var": "accumulator"}, ",", {"var": "current"}]},
  ""
]}
// Result: ",a,b,c"

// ✅ Solution 1 - Check if accumulator is empty
{"reduce": [
  ["a", "b", "c"],
  {"cat": [
    {"var": "accumulator"},
    {"if": [
      {">": [{"length": {"var": "accumulator"}}, 0]},
      ",",
      ""
    ]},
    {"var": "current"}
  ]},
  ""
]}
// Result: "a,b,c"

// ✅ Solution 2 - Use array join pattern
{"reduce": [
  {"filter": [  // Remove empty strings
    {"map": [
      ["a", "b", "c"],
      {"var": ""}
    ]},
    {">": [{"length": {"var": ""}}, 0]}
  ]},
  {"cat": [{"var": "accumulator"}, ",", {"var": "current"}]},
  ""
]}
```

### Issue: Cannot access nested properties in map

**Problem:** Trying to extract nested fields returns null.

**Solution:** Use proper path notation:

```json
// Data: {"users": [{"profile": {"name": "Alice"}}, {"profile": {"name": "Bob"}}]}

// ✅ Correct - use dot notation from current element
{"map": [
  {"var": "users"},
  {"var": "profile.name"}  // Relative to current element
]}
// Result: ["Alice", "Bob"]
```

### Issue: All/some/none returning unexpected results

**Problem:** Quantifier returns true/false opposite of expected.

**Common causes:**
1. **Empty array** - `all([])` is `true`, `some([])` and `none([])` are `false`
2. **Truthy/falsy confusion** - `0`, `""`, `null`, `false` are falsy

**Solutions:**
```json
// Empty array edge cases
{"all": [[], condition]}   // → true (vacuously true)
{"some": [[], condition]}  // → false
{"none": [[], condition]}  // → true

// ✅ Handle empty arrays explicitly
{"if": [
  {">": [{"length": array}, 0]},
  {"all": [array, condition]},
  defaultValue
]}
```

---

## Best Practices

1. **Use empty string** for current element
   ```json
   {"map": [array, {"var": ""}]}
   ```

2. **Chain operations** for complex transforms
   ```json
   {"map": [
     {"filter": [data, condition]},
     transform
   ]}
   ```

3. **Use quantifiers** instead of reduce when possible
   ```json
   {"all": [array, condition]}  // ✓ Clear
   ```

4. **Provide initial value** for reduce
   ```json
   {"reduce": [array, logic, 0]}  // Always specify initial
   ```

5. **Handle empty arrays** gracefully
   ```json
   {"if": [
     {">": [{"length": array}, 0]},
     {"map": [array, logic]},
     []
   ]}
   ```

---

## Context Variables

Special variables available in array operations:

- **`{"var": ""}`** - Current element (map, filter, all, some, none)
- **`{"var": "accumulator"}`** - Accumulated value (reduce)
- **`{"var": "current"}`** - Current element (reduce)
- **`{"var": "$iteration"}`** - Current iteration index (FOR)

---

## Related Operators

- **[Comparison Operators](operators-comparison.md)** - For filtering conditions
- **[Logical Operators](operators-logical.md)** - Combine conditions
- **[Arithmetic Operators](operators-arithmetic.md)** - Transform values
- **[Table Operators](operators-table.md)** - Advanced array queries

---

## Performance Notes

- **Short-circuit evaluation** in `all`, `some`, `none`
- **Zero-copy** context switching for current element
- **Empty array handling** optimized
- **Nested array flattening** efficient in `merge`, `multiplies`, `divides`
