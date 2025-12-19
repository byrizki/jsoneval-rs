---
layout: default
title: Operators Quick Reference
---

# Operators Quick Reference

Complete alphabetical reference of all operators in JSONEval-Rs.

## Introduction

This document provides a quick reference to all **80+ operators** available in JSON-Eval-RS. Each operator is documented in detail in its respective category page with comprehensive examples, troubleshooting guides, and best practices.

### How to Use This Guide

1. **Quick Lookup**: Find operators by category in the tables below
2. **Detailed Learning**: Click through to category pages for comprehensive documentation
3. **Pattern Library**: See common patterns section for real-world examples
4. **Troubleshooting**: Each category page includes troubleshooting sections for common issues

### Documentation Structure

Each operator category includes:
- **Overview**: Context and use cases for the operator category
- **Operator Descriptions**: Syntax, parameters, return types, and examples
- **Complex Examples**: Real-world scenarios with step-by-step breakdowns
- **Troubleshooting**: Common issues and their solutions
- **Best Practices**: Guidelines for effective operator usage
- **Performance Notes**: Optimization tips and considerations

### What is JSON-Eval-RS?

JSON-Eval-RS is an extended implementation of JSON Logic that adds powerful operators for:
- **Excel-style functions**: Rounding, date operations, string manipulation
- **Table operations**: Data lookups, searches, and queries
- **Advanced arrays**: Functional programming with map, filter, reduce
- **Type safety**: Strict and loose comparison modes
- **Financial calculations**: Precise decimal handling

### Quick Start Examples

**Calculate age from birthdate:**
```json
{"DATEDIF": [{"var": "birthdate"}, {"today": null}, "Y"]}
```

**Filter and transform data:**
```json
{"map": [
  {"filter": [{"var": "users"}, {"var": "active"}]},
  {"var": "email"}
]}
```

**Lookup value from table:**
```json
{"VALUEAT": [
  {"var": "rateTable"},
  {"INDEXAT": [{"var": "age"}, {"var": "rateTable"}, "minAge"]},
  "rate"
]}
```

**Format currency:**
```json
{"STRINGFORMAT": [
  {"round": [{"*": [{"var": "price"}, 1.1]}, 2]},
  2,
  "$"
]}
```

## Core & Variable Access

| Operator | Description | Example |
|----------|-------------|---------|
| `var` | Access variable from data | `{"var": "user.name"}` |
| `$ref` / `ref` | JSON Schema reference | `{"$ref": "path"}` |
| Literals | null, bool, number, string, array | `42`, `"text"`, `[1,2,3]` |

## Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `and` | Logical AND | `{"and": [cond1, cond2]}` |
| `or` | Logical OR | `{"or": [val1, val2]}` |
| `not` / `!` | Logical NOT | `{"!": value}` |
| `if` | Conditional | `{"if": [cond, then, else]}` |
| `xor` | Exclusive OR | `{"xor": [a, b]}` |
| `ifnull` | Null coalescing | `{"ifnull": [val, default]}` |
| `isempty` | Check if empty | `{"isempty": value}` |
| `empty` | Empty string literal | `{"empty": null}` |

## Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal (loose) | `{"==": [a, b]}` |
| `===` | Strict equal | `{"===": [a, b]}` |
| `!=` | Not equal (loose) | `{"!=": [a, b]}` |
| `!==` | Strict not equal | `{"!==": [a, b]}` |
| `<` | Less than | `{"<": [a, b]}` |
| `<=` | Less than or equal | `{"<=": [a, b]}` |
| `>` | Greater than | `{">": [a, b]}` |
| `>=` | Greater than or equal | `{">=": [a, b]}` |

## Arithmetic Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `{"+": [1, 2, 3]}` |
| `-` | Subtraction / Negation | `{"-": [10, 3]}` |
| `*` | Multiplication | `{"*": [2, 3, 4]}` |
| `/` | Division | `{"/": [10, 2]}` |
| `%` | Modulo | `{"%": [7, 3]}` |
| `^` / `pow` | Power | `{"^": [2, 3]}` |

## Math Functions

| Operator | Description | Example |
|----------|-------------|---------|
| `abs` | Absolute value | `{"abs": -5}` |
| `max` | Maximum value | `{"max": [1, 5, 3]}` |
| `min` | Minimum value | `{"min": [1, 5, 3]}` |
| `round` | Round to decimals | `{"ROUND": [3.14159, 2]}` |
| `roundup` | Round up | `{"ROUNDUP": [3.1, 0]}` |
| `rounddown` | Round down | `{"ROUNDDOWN": [3.9, 0]}` |
| `ceiling` | Ceiling to multiple | `{"CEILING": [4.3, 0.5]}` |
| `floor` | Floor to multiple | `{"FLOOR": [4.7, 0.5]}` |
| `trunc` | Truncate decimals | `{"TRUNC": [8.9, 2]}` |
| `mround` | Round to multiple | `{"MROUND": [10, 3]}` |

## String Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `cat` | Concatenate | `{"cat": ["Hello", " ", "World"]}` |
| `concat` | Concatenate (alias) | `{"CONCAT": [a, b, c]}` |
| `substr` | Substring | `{"substr": [text, 0, 5]}` |
| `search` | Find substring | `{"SEARCH": ["find", "text"]}` |
| `left` | Left characters | `{"LEFT": [text, 5]}` |
| `right` | Right characters | `{"RIGHT": [text, 5]}` |
| `mid` | Middle characters | `{"MID": [text, 1, 5]}` |
| `len` / `length` | String/array length | `{"len": text}` |
| `splittext` | Split and get index | `{"splittext": [text, ",", 0]}` |
| `splitvalue` | Split to array | `{"splitvalue": [text, ","]}` |
| `stringformat` | Format number | `{"STRINGFORMAT": [1234.5, 2, "$"]}` |

## Date Functions

| Operator | Description | Example |
|----------|-------------|---------|
| `today` | Current date | `{"today": null}` |
| `now` | Current datetime | `{"now": null}` |
| `year` | Extract year | `{"YEAR": date}` |
| `month` | Extract month | `{"MONTH": date}` |
| `day` | Extract day | `{"DAY": date}` |
| `date` | Construct date | `{"DATE": [2024, 1, 15]}` |
| `dateformat` | Format date | `{"DATEFORMAT": [date, "short"]}` |
| `days` | Days between | `{"DAYS": [end, start]}` |
| `yearfrac` | Year fraction | `{"YEARFRAC": [start, end]}` |
| `datedif` | Date difference | `{"DATEDIF": [start, end, "Y"]}` |

## Array Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `map` | Transform array | `{"map": [array, logic]}` |
| `filter` | Filter array | `{"filter": [array, logic]}` |
| `reduce` | Reduce array | `{"reduce": [array, logic, init]}` |
| `all` | All match | `{"all": [array, logic]}` |
| `some` | Some match | `{"some": [array, logic]}` |
| `none` | None match | `{"none": [array, logic]}` |
| `merge` | Merge arrays | `{"merge": [arr1, arr2]}` |
| `in` | Contains value | `{"in": [val, array]}` |
| `sum` | Sum values | `{"SUM": [array]}` |
| `for` | Loop and build | `{"FOR": [1, 5, logic]}` |
| `multiplies` | Flatten and multiply | `{"MULTIPLIES": [vals]}` |
| `divides` | Flatten and divide | `{"DIVIDES": [vals]}` |

## Table/Lookup Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `VALUEAT` | Get table value | `{"VALUEAT": [table, idx, col]}` |
| `MAXAT` | Last row value | `{"MAXAT": [table, col]}` |
| `INDEXAT` | Find index | `{"INDEXAT": [val, table, field]}` |
| `MATCH` | Match conditions | `{"MATCH": [table, val, field]}` |
| `MATCHRANGE` | Match in range | `{"MATCHRANGE": [table, min, max, val]}` |
| `CHOOSE` | Choose match | `{"CHOOSE": [table, val, field]}` |
| `FINDINDEX` | Complex find | `{"FINDINDEX": [table, conditions]}` |

## Utility Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `missing` | Check missing keys | `{"missing": ["key1", "key2"]}` |
| `missing_some` | Check minimum present | `{"missing_some": [1, keys]}` |
| `return` | Return raw value | `{"return": value}` |
| `RANGEOPTIONS` | Generate range options | `{"RANGEOPTIONS": [1, 100]}` |
| `MAPOPTIONS` | Map to options | `{"MAPOPTIONS": [table, lbl, val]}` |
| `MAPOPTIONSIF` | Conditional map | `{"MAPOPTIONSIF": [table, lbl, val, cond]}` |

## Operator Aliases

Many operators have multiple names for compatibility:

| Primary | Aliases | Style |
|---------|---------|-------|
| `var` | - | Standard |
| `$ref` | `ref` | JSON Schema |
| `not` | `!` | JavaScript |
| `pow` | `^`, `**` | Math |
| `len` | `length`, `LEN` | Excel/JS |
| `cat` | `concat`, `CONCAT` | JS/Excel |

**Case Sensitivity:**
- Lowercase: JavaScript-style (`round`, `sum`, `concat`)
- Uppercase: Excel-style (`ROUND`, `SUM`, `CONCAT`)
- Both work identically

## Special Variables

Available in specific contexts:

| Variable | Context | Description |
|----------|---------|-------------|
| `{"var": ""}` | map, filter, all, some, none | Current element |
| `{"var": "accumulator"}` | reduce | Accumulated value |
| `{"var": "current"}` | reduce | Current element |
| `{"var": "$iteration"}` | FOR | Loop index |

## Common Patterns

### Validation
```json
{"if": [
  {"==": [{"length": {"missing": ["email", "name"]}}, 0]},
  "valid",
  "invalid"
]}
```

### Calculation
```json
{"round": [
  {"*": [{"var": "price"}, {"+": [1, {"var": "taxRate"}]}]},
  2
]}
```

### Array Processing
```json
{"map": [
  {"filter": [{"var": "users"}, {"var": "active"}]},
  {"var": "name"}
]}
```

### Table Lookup
```json
{"VALUEAT": [
  {"var": "table"},
  {"INDEXAT": [{"var": "key"}, {"var": "table"}, "id"]},
  "value"
]}
```

### Date Calculation
```json
{"-": [
  {"year": {"today": null}},
  {"year": {"var": "birthdate"}}
]}
```

## Type Coercion

### To Number
- `"123"` → `123`
- `true` → `1`
- `false` → `0`
- `null` → `0`
- `""` → `0`

### To Boolean (Truthiness)
- Falsy: `false`, `null`, `0`, `""`
- Truthy: Everything else

### To String
- All values can be converted via `cat`

## Configuration Options

Operators that respect configuration:

| Option | Affects | Description |
|--------|---------|-------------|
| `safe_nan_handling` | Math ops | Converts NaN/Infinity to 0 or null |
| `recursion_limit` | All | Max nesting depth (default: 100) |

## Performance Tips

1. **Compile once, run many times**
2. **Use automatic optimizations** (flattening, short-circuit)
3. **Leverage fast paths** for simple operations
4. **Cache expensive lookups**
5. **Minimize nested operations** when possible

## Documentation Links

- **[Main Documentation](README.md)** - Overview and navigation
- **[Core Operators](operators-core.md)** - Variables and literals
- **[Logical Operators](operators-logical.md)** - Boolean logic
- **[Comparison Operators](operators-comparison.md)** - Value comparisons
- **[Arithmetic Operators](operators-arithmetic.md)** - Math operations
- **[String Operators](operators-string.md)** - Text manipulation
- **[Math Functions](operators-math.md)** - Advanced math
- **[Date Functions](operators-date.md)** - Date/time operations
- **[Array Operators](operators-array.md)** - Array transformations
- **[Table Operators](operators-table.md)** - Data lookups
- **[Utility Operators](operators-utility.md)** - Helper functions

---

**Total Operators: 80+**

This implementation extends standard JSON Logic with custom operators for Excel-like functions, table operations, and advanced date/string manipulation.
