---
layout: default
title: Operator Cross-Reference Guide
---

# Operator Cross-Reference Guide

Quick lookup for finding the right operator based on what you want to accomplish.

## "I want to..."

### Work with Variables

| Task | Operator | Example |
|------|----------|---------|
| Access a variable | `var` | `{"var": "user.name"}` |
| Provide default for missing variable | `var` with default | `{"var": ["age", 0]}` |
| Check if fields are missing | `missing` | `{"missing": ["email", "phone"]}` |
| Return raw value without evaluation | `return` | `{"return": "literal value"}` |
| Reference JSON Schema definition | `$ref` | `{"$ref": "#/definitions/User"}` |

**See:** [Core Operators](operators-core.md)

---

### Compare Values

| Task | Operator | Example |
|------|----------|---------|
| Check equality (loose) | `==` | `{"==": [a, b]}` |
| Check equality (strict, type-safe) | `===` | `{"===": [a, b]}` |
| Check inequality | `!=` or `!==` | `{"!=": [a, b]}` |
| Less than | `<` | `{"<": [age, 18]}` |
| Greater than | `>` | `{">": [score, 100]}` |
| Range check | `and` + comparisons | `{"and": [{">=": [val, min]}, {"<=": [val, max]}]}` |
| Check if value exists | `!==` with null | `{"!==": [value, null]}` |

**See:** [Comparison Operators](operators-comparison.md)

---

### Logic and Conditionals

| Task | Operator | Example |
|------|----------|---------|
| Conditional (if-then-else) | `if` | `{"if": [condition, then, else]}` |
| Multiple conditions (AND) | `and` | `{"and": [cond1, cond2]}` |
| Alternative conditions (OR) | `or` | `{"or": [val1, val2]}` |
| Negate condition | `not` or `!` | `{"!": condition}` |
| Exclusive or | `xor` | `{"xor": [a, b]}` |
| Null coalescing / default value | `ifnull` | `{"ifnull": [value, "default"]}` |
| Check if empty | `isempty` | `{"isempty": value}` |
| Else-if chain | `if` with multiple conditions | `{"if": [c1, v1, c2, v2, default]}` |

**See:** [Logical Operators](operators-logical.md)

---

### Math Operations

| Task | Operator | Example |
|------|----------|---------|
| Add numbers | `+` | `{"+": [1, 2, 3]}` |
| Subtract | `-` | `{"-": [10, 3]}` |
| Multiply | `*` | `{"*": [price, quantity]}` |
| Divide | `/` | `{"/": [total, count]}` |
| Remainder (modulo) | `%` | `{"%": [7, 3]}` |
| Power | `^` or `pow` | `{"pow": [2, 3]}` |
| Absolute value | `abs` | `{"abs": -5}` |
| Maximum value | `max` | `{"max": [1, 5, 3]}` |
| Minimum value | `min` | `{"min": [1, 5, 3]}` |
| Round to decimals | `round` | `{"round": [3.14159, 2]}` |
| Round up | `roundup` or `ceiling` | `{"roundup": [3.1]}` |
| Round down | `rounddown` or `floor` | `{"rounddown": [3.9]}` |
| Truncate decimals | `trunc` | `{"trunc": [3.7]}` |
| Round to multiple | `mround` | `{"mround": [10, 3]}` |
| Square root | `pow` with 0.5 | `{"pow": [16, 0.5]}` |

**See:** [Arithmetic Operators](operators-arithmetic.md), [Math Functions](operators-math.md)

---

### String Operations

| Task | Operator | Example |
|------|----------|---------|
| Concatenate strings | `cat` | `{"cat": ["Hello", " ", "World"]}` |
| Get substring | `substr` | `{"substr": [text, 0, 5]}` |
| Find text in string | `search` | `{"search": ["find", "in this text"]}` |
| Get first N characters | `left` | `{"left": [text, 5]}` |
| Get last N characters | `right` | `{"right": [text, 5]}` |
| Get middle characters | `mid` | `{"mid": [text, start, length]}` |
| String length | `len` or `length` | `{"len": text}` |
| Split and get element | `splittext` | `{"splittext": ["a,b,c", ",", 0]}` |
| Split to array | `splitvalue` | `{"splitvalue": ["a,b,c", ","]}` |
| Format number as string | `stringformat` | `{"stringformat": [1234.5, 2, "$"]}` |
| Check if string contains | `search` then check null | `{"!==": [{"search": ["find", text]}, null]}` |

**See:** [String Operators](operators-string.md)

---

### Date and Time

| Task | Operator | Example |
|------|----------|---------|
| Get current date | `today` | `{"today": null}` |
| Get current datetime | `now` | `{"now": null}` |
| Extract year from date | `year` | `{"year": date}` |
| Extract month from date | `month` | `{"month": date}` |
| Extract day from date | `day` | `{"day": date}` |
| Create date from parts | `date` | `{"date": [2024, 1, 15]}` |
| Format date for display | `dateformat` | `{"dateformat": [date, "short"]}` |
| Days between dates | `days` | `{"days": [end, start]}` |
| Calculate age | `DATEDIF` with "Y" | `{"DATEDIF": [birthdate, today, "Y"]}` |
| Date difference (various units) | `datedif` | `{"datedif": [start, end, "M"]}` |
| Fractional years | `yearfrac` | `{"yearfrac": [start, end]}` |
| Check if date is in range | Comparisons | `{"and": [{">=": [date, start]}, {"<=": [date, end]}]}` |

**See:** [Date Functions](operators-date.md)

---

### Array Operations

| Task | Operator | Example |
|------|----------|---------|
| Transform each element | `map` | `{"map": [array, logic]}` |
| Filter array by condition | `filter` | `{"filter": [array, condition]}` |
| Reduce to single value | `reduce` | `{"reduce": [array, logic, initial]}` |
| Check if all match | `all` | `{"all": [array, condition]}` |
| Check if any match | `some` | `{"some": [array, condition]}` |
| Check if none match | `none` | `{"none": [array, condition]}` |
| Combine arrays | `merge` | `{"merge": [arr1, arr2]}` |
| Check if value in array | `in` | `{"in": [value, array]}` |
| Sum array values | `sum` | `{"sum": [array]}` |
| Generate sequence | `for` or `FOR` | `{"FOR": [1, 10, logic]}` |
| Array length | `length` | `{"length": array}` |
| Get element at index | Array with `var` | `{"var": "items.0"}` |
| Find element | `filter` then first | `{"var": [{"filter": [...]}, 0]}` |

**See:** [Array Operators](operators-array.md)

---

### Table Lookups

| Task | Operator | Example |
|------|----------|---------|
| Get value from table by index | `VALUEAT` | `{"VALUEAT": [table, idx, "field"]}` |
| Get last row value | `MAXAT` | `{"MAXAT": [table, "field"]}` |
| Find row index by value | `INDEXAT` | `{"INDEXAT": [val, table, "field"]}` |
| Match and return index | `MATCH` | `{"MATCH": [table, val, "field"]}` |
| Match within range | `MATCHRANGE` | `{"MATCHRANGE": [table, min, max, val]}` |
| Choose first match | `CHOOSE` | `{"CHOOSE": [table, val, "field"]}` |
| Find with complex condition | `FINDINDEX` | `{"FINDINDEX": [table, conditions]}` |
| Lookup and return value | `VALUEAT` + `INDEXAT` | See table lookup pattern |

**See:** [Table Operators](operators-table.md)

---

### Utilities

| Task | Operator | Example |
|------|----------|---------|
| Check missing fields | `missing` | `{"missing": ["email", "name"]}` |
| Check minimum fields present | `missing_some` | `{"missing_some": [2, ["a","b","c"]]}` |
| Return uninterpreted value | `return` | `{"return": rawValue}` |
| Generate range of options | `RANGEOPTIONS` | `{"RANGEOPTIONS": [1, 100]}` |
| Map table to options | `MAPOPTIONS` | `{"MAPOPTIONS": [table, "label", "value"]}` |
| Conditional option mapping | `MAPOPTIONSIF` | `{"MAPOPTIONSIF": [table, "l", "v", cond]}` |

**See:** [Utility Operators](operators-utility.md)

---

## Common Patterns by Use Case

### Validation

```json
// Check all required fields present
{"==": [{"length": {"missing": ["email", "name", "age"]}}, 0]}

// Validate email format (basic)
{"and": [
  {">": [{"search": ["@", {"var": "email"}]}, 0]},
  {">": [{"search": [".", {"var": "email"}]}, {"search": ["@", {"var": "email"}]}]}
]}

// Range validation
{"and": [
  {">=": [{"var": "age"}, 18]},
  {"<=": [{"var": "age"}, 120]}
]}
```

---

### Financial Calculations

```json
// Calculate total with tax
{"round": [
  {"*": [
    {"var": "price"},
    {"+": [1, {"/": [{"var": "taxRate"}, 100]}]}
  ]},
  2
]}

// Apply discount
{"round": [
  {"*": [
    {"var": "price"},
    {"-": [1, {"/": [{"var": "discountPercent"}, 100]}]}
  ]},
  2
]}

// Calculate percentage
{"round": [
  {"*": [
    {"/": [{"var": "part"}, {"var": "total"}]},
    100
  ]},
  1
]}
```

---

### Data Transformation

```json
// Extract names from active users
{"map": [
  {"filter": [{"var": "users"}, {"var": "active"}]},
  {"var": "name"}
]}

// Build full names
{"map": [
  {"var": "users"},
  {"cat": [{"var": "firstName"}, " ", {"var": "lastName"}]}
]}

// Calculate averages
{"/": [
  {"sum": [{"var": "scores"}]},
  {"length": {"var": "scores"}}
]}
```

---

### Conditional Logic

```json
// Tiered pricing
{"if": [
  {">=": [{"var": "qty"}, 100]}, 8.00,
  {">=": [{"var": "qty"}, 50]}, 9.00,
  {">=": [{"var": "qty"}, 10]}, 10.00,
  12.00
]}

// Age-based access
{"if": [
  {"<": [{"var": "age"}, 13]}, "CHILD",
  {"<": [{"var": "age"}, 18]}, "TEEN",
  "ADULT"
]}
```

---

### Table Lookups

```json
// Standard lookup pattern
{"VALUEAT": [
  {"var": "rateTable"},
  {"INDEXAT": [{"var": "age"}, {"var": "rateTable"}, "minAge"]},
  "rate"
]}

// Range lookup
{"VALUEAT": [
  {"var": "table"},
  {"MATCHRANGE": [
    {"var": "table"},
    "minValue",
    "maxValue",
    {"var": "lookupValue"}
  ]},
  "result"
]}
```

---

## Operator Selection Guide

### When to Use What

| Scenario | Use This | Not This | Why |
|----------|----------|----------|-----|
| String concatenation | `cat` | `+` | `+` coerces to numbers |
| Type-safe comparison | `===` | `==` | Avoid type coercion surprises |
| Null defaults | `ifnull` | nested `if` | Cleaner and explicit |
| Array membership | `in` | `some` with `==` | More efficient |
| Calculate age | `DATEDIF` | Year subtraction | Accounts for birthday |
| Round currency | `round` with 2 decimals | `floor` or `ceiling` | Standard rounding |
| Check all conditions | `all` | `reduce` with logic | Short-circuits, clearer |
| Loop through numbers | `FOR` | `reduce` tricks | Purpose-built |

---

## Quick Syntax Reference

### Common Structures

**If-Then-Else:**
```json
{"if": [condition, thenValue, elseValue]}
```

**If-ElseIf-Else:**
```json
{"if": [
  condition1, value1,
  condition2, value2,
  condition3, value3,
  defaultValue
]}
```

**Map (transform):**
```json
{"map": [array, {"operation": {"var": ""}}]}
```

**Filter:**
```json
{"filter": [array, condition]}
```

**Reduce:**
```json
{"reduce": [
  array,
  {"operation": [{"var": "accumulator"}, {"var": "current"}]},
  initialValue
]}
```

---

## Related Documentation

- **[Operators Summary](OPERATORS_SUMMARY.md)** - Complete operator reference
- **[Common Mistakes](COMMON_MISTAKES.md)** - Pitfalls to avoid
- Individual operator pages for detailed documentation

---

**Tip:** When in doubt, check the specific operator documentation for comprehensive examples and troubleshooting!
