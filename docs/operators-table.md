---
layout: page
title: Table/Lookup Operators
permalink: /operators-table/
---

# Table/Lookup Operators

Advanced table operations for data lookups, indexing, and queries.

## `VALUEAT` - Get Value from Table

Retrieves a value from a table (array of objects) by row index and optional column name.

### Syntax
```json
{"VALUEAT": [table, row_index]}
{"VALUEAT": [table, row_index, column_name]}
```

### Parameters
- **table** (array): Array of objects
- **row_index** (number): Row index (0-based)
- **column_name** (string, optional): Column/field name

### Return Type
Any - Entire row object or specific column value, `null` if not found

### Examples

**Get entire row:**
```json
// Data: {"table": [
//   {"name": "Alice", "age": 30},
//   {"name": "Bob", "age": 25},
//   {"name": "Charlie", "age": 35}
// ]}
{"VALUEAT": [{"var": "table"}, 1]}
// → {"name": "Bob", "age": 25}
```

**Get specific column:**
```json
{"VALUEAT": [{"var": "table"}, 1, "name"]}  // → "Bob"
{"VALUEAT": [{"var": "table"}, 2, "age"]}   // → 35
```

**Out of bounds:**
```json
{"VALUEAT": [{"var": "table"}, -1]}   // → null
{"VALUEAT": [{"var": "table"}, 999]}  // → null
```

**Dynamic index:**
```json
{"VALUEAT": [
  {"var": "rates"},
  {"INDEXAT": [{"var": "age"}, {"var": "rates"}, "minAge"]},
  "rate"
]}
```

**With FOR loop:**
```json
{"FOR": [
  0,
  {"-": [{"length": {"var": "table"}}, 1]},
  {"VALUEAT": [{"var": "table"}, {"var": "$iteration"}, "value"]}
]}
```

---

## `MAXAT` - Get Last Row Value

Returns a column value from the last row of a table.

### Syntax
```json
{"MAXAT": [table, column_name]}
```

### Parameters
- **table** (array): Array of objects
- **column_name** (string): Field name to extract

### Return Type
Any - Value from last row's column, `null` if table is empty

### Examples

**Get last value:**
```json
// Data: {"table": [
//   {"name": "Alice", "score": 85},
//   {"name": "Bob", "score": 92},
//   {"name": "Charlie", "score": 78}
// ]}
{"MAXAT": [{"var": "table"}, "score"]}  // → 78 (Charlie's score)
```

**Assumes sorted data:**
```json
// If table is sorted by age ascending, get maximum age
{"MAXAT": [{"var": "sortedByAge"}, "age"]}
```

**Get latest entry:**
```json
{"MAXAT": [{"var": "history"}, "timestamp"]}
```

---

## `INDEXAT` - Find Index

Finds the index of an element in a table by field value.

### Syntax
```json
{"INDEXAT": [lookup_value, table, field]}
{"INDEXAT": [lookup_value, table, field, range]}
```

### Parameters
- **lookup_value** (any): Value to find
- **table** (array): Array of objects to search
- **field** (string): Field name to compare
- **range** (boolean, optional): If `true`, finds first where field <= value

### Return Type
Number - Index (0-based) of match, or `-1` if not found

### Examples

**Exact match:**
```json
// Data: {"table": [
//   {"id": 100, "name": "Alice"},
//   {"id": 200, "name": "Bob"},
//   {"id": 300, "name": "Charlie"}
// ]}
{"INDEXAT": [200, {"var": "table"}, "id"]}  // → 1
```

**Not found:**
```json
{"INDEXAT": [999, {"var": "table"}, "id"]}  // → -1
```

**Range search (first where field <= value):**
```json
{"INDEXAT": [250, {"var": "table"}, "id", true]}  // → 0 (first id <= 250)
```

**Lookup and retrieve:**
```json
{"VALUEAT": [
  {"var": "table"},
  {"INDEXAT": [{"var": "searchId"}, {"var": "table"}, "id"]},
  "name"
]}
```

---

## `MATCH` - Match Row by Conditions

Finds the index of first row matching all specified field-value pairs.

### Syntax
```json
{"MATCH": [table, value1, field1, value2, field2, ...]}
```

### Parameters
- **table** (array): Array of objects
- **value, field** (pairs): Value-field pairs to match (all must match)

### Return Type
Number - Index of first matching row, or `-1` if not found

### Examples

**Single condition:**
```json
// Data: {"table": [
//   {"name": "Alice", "age": 30, "city": "NYC"},
//   {"name": "Bob", "age": 25, "city": "LA"},
//   {"name": "Charlie", "age": 35, "city": "NYC"}
// ]}
{"MATCH": [{"var": "table"}, "Alice", "name"]}  // → 0
```

**Multiple conditions (AND):**
```json
{"MATCH": [{"var": "table"}, "Alice", "name", "NYC", "city"]}
// → 0 (Alice in NYC)

{"MATCH": [{"var": "table"}, "Bob", "name", "NYC", "city"]}
// → -1 (Bob not in NYC)
```

**Not found:**
```json
{"MATCH": [{"var": "table"}, "David", "name"]}  // → -1
```

**Retrieve matched row:**
```json
{"VALUEAT": [
  {"var": "table"},
  {"MATCH": [{"var": "table"}, "Alice", "name"]}
]}
```

---

## `MATCHRANGE` - Match Value in Range

Finds the index of first row where value falls between min and max fields.

### Syntax
```json
{"MATCHRANGE": [table, min_field, max_field, value]}
```

### Parameters
- **table** (array): Array of objects with range fields
- **min_field** (string): Field name for minimum value
- **max_field** (string): Field name for maximum value
- **value** (number): Value to check

### Return Type
Number - Index of first row where min <= value <= max, or `-1` if not found

### Examples

**Rate table lookup:**
```json
// Data: {"rates": [
//   {"min_age": 0, "max_age": 25, "rate": 0.05},
//   {"min_age": 26, "max_age": 40, "rate": 0.07},
//   {"min_age": 41, "max_age": 60, "rate": 0.09}
// ]}

{"MATCHRANGE": [{"var": "rates"}, "min_age", "max_age", 30]}
// → 1 (30 falls in 26-40 range)

{"MATCHRANGE": [{"var": "rates"}, "min_age", "max_age", 50]}
// → 2 (50 falls in 41-60 range)
```

**Get rate for age:**
```json
{"VALUEAT": [
  {"var": "rates"},
  {"MATCHRANGE": [{"var": "rates"}, "min_age", "max_age", {"var": "age"}]},
  "rate"
]}
```

**Tax bracket:**
```json
{"MATCHRANGE": [
  {"var": "taxBrackets"},
  "min_income",
  "max_income",
  {"var": "income"}
]}
```

---

## `CHOOSE` - Choose Random/First Match

Finds the index of any (typically first) row matching the condition.

### Syntax
```json
{"CHOOSE": [table, value, field, ...]}
```

### Parameters
- **table** (array): Array of objects
- **value, field** (pairs): Condition pairs

### Return Type
Number - Index of a matching row, or `-1` if not found

### Examples

**Find any match:**
```json
// Data: {"products": [
//   {"name": "Widget", "category": "A", "price": 10},
//   {"name": "Gadget", "category": "B", "price": 20},
//   {"name": "Tool", "category": "A", "price": 15}
// ]}

{"CHOOSE": [{"var": "products"}, "A", "category"]}
// → 0 or 2 (any item in category A)
```

**Get matched product:**
```json
{"VALUEAT": [
  {"var": "products"},
  {"CHOOSE": [{"var": "products"}, "A", "category"]},
  "name"
]}
```

---

## `FINDINDEX` - Find Index with Complex Conditions

Finds the index of first row matching complex conditions with ergonomic syntax.

### Syntax
```json
{"FINDINDEX": [table, condition1, condition2, ...]}
```

### Conditions Format

**String literals** - Treated as field name checks (truthy):
```json
"fieldName"  // row.fieldName is truthy
```

**Array format** - Comparison triplets `[op, value, field]`:
```json
[">", 15, "age"]  // row.age > 15
["==", "active", "status"]  // row.status == "active"
```

**Object format** - Standard logic expressions:
```json
{">": [{"var": "age"}, 18]}
```

### Return Type
Number - Index of first matching row, or `-1` if not found

### Examples

**Simple field check:**
```json
// Data: {"items": [
//   {"value": 10, "active": false},
//   {"value": 20, "active": true},
//   {"value": 30, "active": true}
// ]}

{"FINDINDEX": [{"var": "items"}, "active"]}
// → 1 (first active item)
```

**With comparison:**
```json
{"FINDINDEX": [
  {"var": "items"},
  [">", 15, "value"]
]}
// → 1 (first item where value > 15)
```

**Multiple conditions (AND):**
```json
{"FINDINDEX": [
  {"var": "items"},
  "active",
  [">", 15, "value"]
]}
// → 2 (first item where active=true AND value > 15)
```

**Complex logic:**
```json
{"FINDINDEX": [
  {"var": "users"},
  {">": [{"var": "score"}, 70]},
  {"==": [{"var": "status"}, "verified"]}
]}
```

---

## Complex Examples

### Lookup with Fallback
```json
{"ifnull": [
  {"VALUEAT": [
    {"var": "table"},
    {"INDEXAT": [{"var": "key"}, {"var": "table"}, "id"]},
    "value"
  ]},
  "default"
]}
```

### Multi-Table Lookup
```json
{"VALUEAT": [
  {"var": "rates"},
  {"MATCH": [
    {"var": "rates"},
    {"var": "category"},
    "category",
    {"var": "type"},
    "type"
  ]},
  "multiplier"
]}
```

### Range-Based Calculation
```json
{"*": [
  {"var": "amount"},
  {"VALUEAT": [
    {"var": "brackets"},
    {"MATCHRANGE": [
      {"var": "brackets"},
      "min",
      "max",
      {"var": "income"}
    ]},
    "rate"
  ]}
]}
```

### Find and Update Pattern
```json
{
  "let": {
    "index": {"FINDINDEX": [{"var": "items"}, "selected"]},
    "found": {">": [{"var": "index"}, -1]}
  },
  "in": {
    "if": [
      {"var": "found"},
      {"VALUEAT": [{"var": "items"}, {"var": "index"}]},
      null
    ]
  }
}
```

### Nested Table Lookup
```json
{"VALUEAT": [
  {"VALUEAT": [
    {"var": "masterTable"},
    {"INDEXAT": [{"var": "masterId"}, {"var": "masterTable"}, "id"]},
    "detailTable"
  ]},
  {"INDEXAT": [{"var": "detailId"}, detailTable, "id"]},
  "value"
]}
```

### Age-Based Rate with Range
```json
{"VALUEAT": [
  {"$ref": "$params.mortality_table"},
  {"MATCHRANGE": [
    {"$ref": "$params.mortality_table"},
    "min_age",
    "max_age",
    {"var": "current_age"}
  ]},
  "rate"
]}
```

---

## Comparison Table

| Operator | Use Case | Match Type | Return |
|----------|----------|------------|--------|
| **VALUEAT** | Direct access by index | Index-based | Value or row |
| **MAXAT** | Get last row value | Last element | Column value |
| **INDEXAT** | Find by exact match | Exact or range | Index |
| **MATCH** | Find by multiple fields | Exact (AND) | Index |
| **MATCHRANGE** | Find by value in range | Range | Index |
| **CHOOSE** | Find any match | Exact | Index |
| **FINDINDEX** | Complex conditions | Custom logic | Index |

---

## Best Practices

1. **Combine operators** for lookups
   ```json
   {"VALUEAT": [table, {"INDEXAT": [value, table, field]}, column]}
   ```

2. **Check for not found** (-1)
   ```json
   {"if": [
     {">": [{"INDEXAT": [value, table, field]}, -1]},
     foundLogic,
     notFoundLogic
   ]}
   ```

3. **Use MATCHRANGE** for bracketed data
   ```json
   {"MATCHRANGE": [brackets, "min", "max", value]}
   ```

4. **Cache lookups** when used multiple times
   ```json
   // Store index result if used repeatedly
   ```

5. **Validate table structure** before lookup
   ```json
   {"if": [
     {">": [{"length": {"var": "table"}}, 0]},
     lookup,
     default
   ]}
   ```

---

## Common Patterns

### Progressive Lookup
```json
{"ifnull": [
  {"VALUEAT": [primary, index, field]},
  {"ifnull": [
    {"VALUEAT": [secondary, index, field]},
    default
  ]}
]}
```

### Tiered Rate Table
```json
{"VALUEAT": [
  {"var": "rateTables"},
  {"MATCHRANGE": [
    {"var": "rateTables"},
    "minValue",
    "maxValue",
    {"var": "input"}
  ]},
  "rate"
]}
```

### Conditional Table Selection
```json
{"VALUEAT": [
  {"if": [
    {"var": "usePremium"},
    {"var": "premiumRates"},
    {"var": "standardRates"}
  ]},
  {"INDEXAT": [{"var": "age"}, table, "minAge"]},
  "rate"
]}
```

---

## Related Operators

- **[Array Operators](operators-array.md)** - `filter`, `map`, `find` alternatives
- **[Comparison Operators](operators-comparison.md)** - Build conditions
- **[Logical Operators](operators-logical.md)** - Combine conditions

---

## Performance Notes

- **Linear search** for MATCH, FINDINDEX, CHOOSE
- **Indexed access** for VALUEAT is O(1)
- **Early termination** on first match
- **Table caching** recommended for repeated lookups
