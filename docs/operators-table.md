---
layout: default
title: Table/Lookup Operators
---

# Table/Lookup Operators

Advanced table operations for data lookups, indexing, and queries.

## Overview

Table operators work with **arrays of objects** (referred to as "tables") to perform lookups, searches, and queries. These operators are essential for working with structured data like rate tables, configuration maps, and relational-style data within JSON evaluations.

### What is a Table?

In JSON-Eval-RS, a "table" is an array of objects where each object represents a row:

```json
[
  {"id": 1, "name": "Alice", "age": 30},
  {"id": 2, "name": "Bob", "age": 25},
  {"id": 3, "name": "Charlie", "age": 35}
]
```

Each object has the same structure (fields/columns), similar to a database table or spreadsheet.

### Common Use Cases

- **Rate Tables**: Look up insurance rates, tax brackets, or pricing tiers based on ranges
- **Configuration Maps**: Find settings or parameters based on keys
- **Data Joins**: Retrieve related data from multiple tables
- **Filtering & Searching**: Find rows matching specific criteria
- **Range-Based Lookups**: Find values that fall within min/max boundaries

### Operator Categories

1. **Direct Access**: `VALUEAT`, `MAXAT` - Get values by position
2. **Search/Find**: `INDEXAT`, `MATCH`, `MATCHRANGE`, `CHOOSE` - Find row indices
3. **Complex Queries**: `FINDINDEX` - Advanced conditional searching

Most workflows combine these: use a search operator to find the row index, then use `VALUEAT` to retrieve the desired value.

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

Finds the index of an element in a table by matching a field value.

### Syntax
```json
{"INDEXAT": [lookup_value, table, field]}
{"INDEXAT": [lookup_value, table, field, range]}
```

### Parameters
- **lookup_value** (any): Value to find
- **table** (array): Array of objects to search
- **field** (string): Field name to compare against
- **range** (boolean, optional): 
  - `false` or omitted: **Exact match** - finds row where `field == lookup_value`
  - `true`: **Range search** - finds **first** row where `field <= lookup_value` (assumes table is sorted by field in ascending order)

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

**Range search (assumes sorted):**
```json
// Data: {"brackets": [
//   {"threshold": 0, "rate": 0.1},
//   {"threshold": 10000, "rate": 0.15},
//   {"threshold": 50000, "rate": 0.25}
// ]}

// Find the first bracket where threshold <= 25000
{"INDEXAT": [25000, {"var": "brackets"}, "threshold", true]}
// → 1 (threshold 10000 <= 25000)

// Find the first bracket where threshold <= 75000
{"INDEXAT": [75000, {"var": "brackets"}, "threshold", true]}
// → 2 (threshold 50000 <= 75000)
```

**Important**: Range mode expects the table to be **sorted** by the field in ascending order. It returns the **first** (leftmost) row where the field value is less than or equal to the lookup value.

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

Finds the index of any (typically first) row matching the condition. Similar to `MATCH` but with less predictable ordering.

### Syntax
```json
{"CHOOSE": [table, value, field, ...]}
```

### Parameters
- **table** (array): Array of objects
- **value, field** (pairs): One or more value-field pairs to match (all must match)

### Return Type
Number - Index of a matching row, or `-1` if not found

### When to Use CHOOSE vs MATCH

- **MATCH**: Guarantees return of **first** matching row. Use for predictable results.
- **CHOOSE**: Returns **any** matching row (implementation-dependent). Use when order doesn't matter or for performance optimization in specific scenarios.

⚠️ **Note**: In most cases, prefer `MATCH` for predictable behavior. `CHOOSE` exists for compatibility and specific performance scenarios where the first match doesn't matter.

### Examples

**Find any match:**
```json
// Data: {"products": [
//   {"name": "Widget", "category": "A", "price": 10},
//   {"name": "Gadget", "category": "B", "price": 20},
//   {"name": "Tool", "category": "A", "price": 15}
// ]}

{"CHOOSE": [{"var": "products"}, "A", "category"]}
// → Could return 0 or 2 (any item in category A)
// Actual result depends on implementation
```

**Get matched product:**
```json
{"VALUEAT": [
  {"var": "products"},
  {"CHOOSE": [{"var": "products"}, "A", "category"]},
  "name"
]}
// → "Widget" or "Tool" (whichever match is returned)
```

**Multiple conditions:**
```json
// Find any product where category="A" AND price=15
{"CHOOSE": [{"var": "products"}, "A", "category", 15, "price"]}
// → 2 (only one match, so deterministic in this case)
```

**Practical scenario - any available resource:**
```json
// Get any available server (order doesn't matter)
{"VALUEAT": [
  {"var": "servers"},
  {"CHOOSE": [{"var": "servers"}, true, "available"]},
  "hostname"
]}
```

---

## `FINDINDEX` - Find Index with Complex Conditions

Finds the index of first row matching complex conditions with ergonomic syntax. This is the most flexible search operator, supporting multiple condition formats.

### Syntax
```json
{"FINDINDEX": [table, condition1, condition2, ...]}
```

### Conditions Format

`FINDINDEX` supports **three formats** for conditions, which can be mixed and matched:

#### 1. String Literals - Field Truthiness Check
Simplest format: checks if a field exists and is truthy.

```json
"fieldName"  // Checks if row.fieldName is truthy (not null, false, 0, or empty string)
```

**Example:**
```json
// Find first row where "active" field is truthy
{"FINDINDEX": [{"var": "users"}, "active"]}
```

#### 2. Array Triplets - Comparison Operations
Format: `[operator, value, field]` - Compares row's field against a value.

```json
[">", 18, "age"]          // row.age > 18
["==", "active", "status"] // row.status == "active"
["<=", 100, "price"]       // row.price <= 100
["!=", null, "email"]      // row.email != null
```

**Supported operators:** `==`, `!=`, `>`, `>=`, `<`, `<=`

**Example:**
```json
// Find first user where age > 18
{"FINDINDEX": [{"var": "users"}, [">", 18, "age"]]}
```

#### 3. Object Expressions - Full Logic Power
Use any JSON-Eval-RS expression for maximum flexibility.

```json
{">": [{"var": "score"}, 70]}
{"and": [
  {">": [{"var": "age"}, 18]},
  {"==": [{"var": "status"}, "active"]}
]}
```

**Example:**
```json
// Find first user where score > 70
{"FINDINDEX": [
  {"var": "users"},
  {">": [{"var": "score"}, 70]}
]}
```

### Combining Conditions (AND Logic)

All conditions provided must be true (AND logic). You can mix different formats:

```json
{"FINDINDEX": [
  {"var": "users"},
  "verified",                          // String: verified is truthy
  [">", 18, "age"],                    // Array: age > 18
  {"!=": [{"var": "email"}, null]}     // Object: email is not null
]}
// Finds first user where: verified AND age > 18 AND email is not null
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
// → 1 (first item where active is truthy)
```

**With comparison:**
```json
{"FINDINDEX": [
  {"var": "items"},
  [">", 15, "value"]
]}
// → 1 (first item where value > 15, which is 20)
```

**Multiple conditions (AND):**
```json
{"FINDINDEX": [
  {"var": "items"},
  "active",
  [">", 15, "value"]
]}
// → 2 (first item where active=true AND value > 15)
// Item at index 1 has value=20 but we need both conditions
```

**Complex logic with object expression:**
```json
{"FINDINDEX": [
  {"var": "users"},
  {">": [{"var": "score"}, 70]},
  {"==": [{"var": "status"}, "verified"]}
]}
// Find first verified user with score > 70
```

**String matching:**
```json
{"FINDINDEX": [
  {"var": "products"},
  ["==", "electronics", "category"],
  [">", 50, "price"]
]}
// Find first electronics product over $50
```

**Checking for null/undefined:**
```json
{"FINDINDEX": [
  {"var": "records"},
  ["!=", null, "email"]  // Has email
]}
// Find first record with non-null email
```

### Which Format to Use?

- **String literal** (`"field"`) - When you just need to check if a field exists/is truthy
- **Array triplet** (`["op", value, "field"]`) - For simple comparisons. Clean and readable.
- **Object expression** (`{">": [...]}`) - For complex logic, nested operations, or when you need OR conditions

**Pro tip**: Use array triplets for better readability when possible. They're more compact than object expressions for simple comparisons.

---

## Complex Examples

These examples demonstrate real-world patterns combining multiple table operators.

### Lookup with Fallback

**Use case:** Get a value from a table, but provide a default if not found.

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

**How it works:**
1. `INDEXAT` searches for the row where `id` matches `key`
2. `VALUEAT` retrieves the `value` field from that row
3. `ifnull` returns "default" if step 2 returns `null` (when index is `-1`)

### Multi-Table Lookup

**Use case:** Find a row that matches multiple criteria across different fields.

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

**How it works:**
1. `MATCH` finds the first row where both:
   - `category` field equals the value in `{"var": "category"}`
   - `type` field equals the value in `{"var": "type"}`
2. `VALUEAT` extracts the `multiplier` from that matched row

### Range-Based Calculation

**Use case:** Apply different rates based on income brackets (like tax calculation).

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

**How it works:**
1. `MATCHRANGE` finds the bracket where `min <= income <= max`
2. `VALUEAT` gets the `rate` from that bracket
3. Multiply `amount` by the found `rate`

**Example data:**
```json
{
  "income": 35000,
  "amount": 1000,
  "brackets": [
    {"min": 0, "max": 25000, "rate": 0.10},
    {"min": 25001, "max": 50000, "rate": 0.15},
    {"min": 50001, "max": 999999, "rate": 0.25}
  ]
}
// Result: 1000 * 0.15 = 150
```

### Find and Update Pattern

**Use case:** Safely retrieve an item if it exists, return null otherwise.

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

**How it works:**
1. Find the index of the first selected item
2. Store whether an item was found (`found` = index > -1)
3. If found, return the full item object; otherwise return `null`

**Why use this pattern:** Separates search from retrieval, making the logic clearer and allowing reuse of the index.

### Nested Table Lookup

**Use case:** Relational-style lookup across master-detail tables.

```json
{"VALUEAT": [
  {"VALUEAT": [
    {"var": "masterTable"},
    {"INDEXAT": [{"var": "masterId"}, {"var": "masterTable"}, "id"]},
    "detailTable"
  ]},
  {"INDEXAT": [{"var": "detailId"}, {"var": "detailTable"}, "id"]},
  "value"
]}
```

**How it works:**
1. Find the master record by `masterId`
2. Extract the `detailTable` array from that master record
3. Within that detail table, find the record matching `detailId`
4. Extract the `value` from that detail record

**Example data structure:**
```json
{
  "masterId": 1,
  "detailId": 101,
  "masterTable": [
    {
      "id": 1,
      "name": "Master A",
      "detailTable": [
        {"id": 101, "value": "Detail Value A"},
        {"id": 102, "value": "Detail Value B"}
      ]
    }
  ]
}
// Result: "Detail Value A"
```

### Age-Based Rate with Range

**Use case:** Insurance/actuarial calculations using mortality tables.

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

**How it works:**
1. Reference a mortality table from parameters
2. Find the age bracket containing `current_age`
3. Return the mortality rate for that bracket

**Important:** This uses `$ref` to reference external parameters, useful when table data is large or shared across multiple calculations.

### Conditional Lookup Based on Multiple Tables

**Use case:** Choose between different rate tables based on a condition, then perform lookup.

```json
{
  "let": {
    "selectedTable": {
      "if": [
        {"var": "isPremium"},
        {"var": "premiumRates"},
        {"var": "standardRates"}
      ]
    },
    "rateIndex": {"INDEXAT": [
      {"var": "age"},
      {"var": "selectedTable"},
      "minAge"
    ]}
  },
  "in": {
    "VALUEAT": [
      {"var": "selectedTable"},
      {"var": "rateIndex"},
      "rate"
    ]
  }
}
```

**How it works:**
1. Select which table to use based on `isPremium` flag
2. Find the appropriate rate row in the selected table
3. Extract the rate value
4. By using `let`, we avoid repeating the table selection logic

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

### 1. Combine Operators for Lookups

Table operators are designed to work together. Most patterns use a search operator to find an index, then `VALUEAT` to retrieve data.

```json
// ✅ Standard pattern
{"VALUEAT": [table, {"INDEXAT": [value, table, field]}, column]}
```

**Why:** This separation of concerns makes logic clearer and allows for null checks on the index.

### 2. Always Check for Not Found (-1)

Search operators return `-1` when no match is found. Always handle this case.

```json
// ✅ Safe approach
{
  "let": {
    "idx": {"INDEXAT": [value, table, field]}
  },
  "in": {
    "if": [
      {">": [{"var": "idx"}, -1]},
      {"VALUEAT": [table, {"var": "idx"}, column]},
      "DEFAULT_VALUE"  // Handle not found
    ]
  }
}

// ❌ Unsafe - may return null unexpectedly
{"VALUEAT": [table, {"INDEXAT": [value, table, field]}, column]}
```

### 3. Use MATCHRANGE for Bracketed Data

When your data represents ranges (age brackets, tax tiers, price ranges), use `MATCHRANGE`.

```json
// ✅ Correct for range-based lookups
{"MATCHRANGE": [brackets, "min", "max", value]}

// ❌ Don't use exact match for ranges
{"INDEXAT": [value, brackets, "min"]}  // Won't work correctly
```

**Important:** Ensure your range table:
- Is sorted (ascending by min/max)
- Has no overlapping ranges
- Covers all expected input values

### 4. Cache Lookups with `let`

If you use the same index or value multiple times, store it in a variable.

```json
// ✅ Efficient - search once, use many times
{
  "let": {
    "userIdx": {"INDEXAT": [{"var": "userId"}, {"var": "users"}, "id"]}
  },
  "in": {
    "name": {"VALUEAT": [{"var": "users"}, {"var": "userIdx"}, "name"]},
    "email": {"VALUEAT": [{"var": "users"}, {"var": "userIdx"}, "email"]},
    "phone": {"VALUEAT": [{"var": "users"}, {"var": "userIdx"}, "phone"]}
  }
}

// ❌ Inefficient - searches three times
{
  "name": {"VALUEAT": [
    {"var": "users"},
    {"INDEXAT": [{"var": "userId"}, {"var": "users"}, "id"]},
    "name"
  ]},
  "email": {"VALUEAT": [
    {"var": "users"},
    {"INDEXAT": [{"var": "userId"}, {"var": "users"}, "id"]},
    "email"
  ]}
  // ... repeats search
}
```

### 5. Validate Table Structure Before Use

Prevent runtime errors by checking table validity.

```json
// ✅ Safe table access
{
  "if": [
    {"and": [
      {"isarray": {"var": "table"}},
      {">": [{"length": {"var": "table"}}, 0]}
    ]},
    // Table is valid - perform lookup
    {"VALUEAT": [{"var": "table"}, 0, "field"]},
    // Table invalid - use fallback
    null
  ]
}
```

### 6. Choose the Right Operator

Different operators have different use cases:

| Situation | Operator | Reason |
|-----------|----------|--------|
| Single field match | `INDEXAT` | Simplest and clearest |
| Multiple field match (AND) | `MATCH` | Designed for multi-criteria |
| Range-based lookup | `MATCHRANGE` | Handles min/max efficiently |
| Complex conditions | `FINDINDEX` | Flexible condition formats |
| Don't care which match | `CHOOSE` | Rare - usually use `MATCH` |

### 7. Use Descriptive Variable Names in `let`

Make your logic self-documenting:

```json
// ✅ Clear intent
{
  "let": {
    "userIndex": {"INDEXAT": [{"var": "userId"}, {"var": "users"}, "id"]},
    "userFound": {">": [{"var": "userIndex"}, -1]}
  },
  "in": { /* ... */ }
}

// ❌ Unclear
{
  "let": {
    "i": {"INDEXAT": [{"var": "userId"}, {"var": "users"}, "id"]},
    "f": {">": [{"var": "i"}, -1]}
  },
  "in": { /* ... */ }
}
```

### 8. Prefer `MATCH` Over `CHOOSE`

Unless you have a specific reason, use `MATCH` for predictable results.

```json
// ✅ Predictable - always returns first match
{"MATCH": [table, value, field]}

// ⚠️ Unpredictable - returns any match
{"CHOOSE": [table, value, field]}
```

### 9. Handle Edge Cases

Consider what happens with:
- Empty tables
- Missing fields in rows
- Null values
- Out-of-range values

```json
// Comprehensive error handling
{
  "let": {
    "table": {"var": "rateTable"},
    "isValid": {"and": [
      {"isarray": {"var": "table"}},
      {">": [{"length": {"var": "table"}}, 0]}
    ]},
    "index": {
      "if": [
        {"var": "isValid"},
        {"INDEXAT": [{"var": "searchKey"}, {"var": "table"}, "id"]},
        -1
      ]
    }
  },
  "in": {
    "if": [
      {"and": [
        {"var": "isValid"},
        {">=": [{"var": "index"}, 0]}
      ]},
      {"VALUEAT": [{"var": "table"}, {"var": "index"}, "value"]},
      {"error": "Data not found", "key": {"var": "searchKey"}}
    ]
  }
}
```

### 10. Document Complex Lookups

For multi-step lookups, add comments (in your code, not JSON) explaining the logic:

```javascript
// Example in JavaScript where you build the JSON
const rateCalculation = {
  // Step 1: Find the age bracket (0-25, 26-40, etc.)
  let: {
    bracketIndex: {
      MATCHRANGE: [
        {var: "ageBrackets"},
        "minAge",
        "maxAge",
        {var: "currentAge"}
      ]
    }
  },
  // Step 2: Extract the rate for that bracket
  in: {
    VALUEAT: [
      {var: "ageBrackets"},
      {var: "bracketIndex"},
      "rate"
    ]
  }
};
```

---

## Common Patterns

### Progressive Lookup (Fallback Chain)

**When to use:** When you have multiple data sources and want to check them in priority order.

```json
{"ifnull": [
  {"VALUEAT": [primary, index, field]},
  {"ifnull": [
    {"VALUEAT": [secondary, index, field]},
    default
  ]}
]}
```

**Example:** Check premium rates first, fall back to standard rates, then use default:
```json
{
  "let": {
    "index": {"INDEXAT": [{"var": "age"}, {"var": "premiumRates"}, "minAge"]}
  },
  "in": {
    "ifnull": [
      {"VALUEAT": [{"var": "premiumRates"}, {"var": "index"}, "rate"]},
      {"ifnull": [
        {"VALUEAT": [{"var": "standardRates"}, {"var": "index"}, "rate"]},
        0.05  // Default rate
      ]}
    ]
  }
}
```

### Tiered Rate Table

**When to use:** Pricing, insurance, or tax calculations with range-based rates.

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

**Real-world example - Progressive tax:**
```json
// Data: income = 45000
// Brackets: 0-20k: 10%, 20k-50k: 15%, 50k+: 25%
{
  "let": {
    "taxBracketIndex": {"MATCHRANGE": [
      {"var": "taxBrackets"},
      "minIncome",
      "maxIncome",
      {"var": "income"}
    ]}
  },
  "in": {
    "VALUEAT": [
      {"var": "taxBrackets"},
      {"var": "taxBracketIndex"},
      "rate"
    ]
  }
}
// Returns: 0.15 (15% rate for 20k-50k bracket)
```

### Conditional Table Selection

**When to use:** Different lookup tables based on user type, region, or configuration.

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

**Better pattern using `let` for clarity:**
```json
{
  "let": {
    "activeTable": {
      "if": [
        {"==": [{"var": "region"}, "US"]},
        {"var": "usRates"},
        {
          "if": [
            {"==": [{"var": "region"}, "EU"]},
            {"var": "euRates"},
            {"var": "defaultRates"}
          ]
        }
      ]
    }
  },
  "in": {
    "VALUEAT": [
      {"var": "activeTable"},
      {"INDEXAT": [{"var": "criteria"}, {"var": "activeTable"}, "key"]},
      "value"
    ]
  }
}
```

### Lookup with Validation

**When to use:** Ensure data exists before using it, preventing null errors downstream.

```json
{
  "let": {
    "index": {"INDEXAT": [{"var": "searchKey"}, {"var": "table"}, "id"]}
  },
  "in": {
    "if": [
      {">=": [{"var": "index"}, 0]},
      {
        // Data found - safe to use
        "result": {"VALUEAT": [{"var": "table"}, {"var": "index"}, "value"]},
        "status": "found"
      },
      {
        // Not found - handle gracefully
        "result": null,
        "status": "not_found",
        "searchKey": {"var": "searchKey"}
      }
    ]
  }
}
```

### Multi-Criteria Lookup

**When to use:** Need to match multiple fields simultaneously (AND logic).

```json
{
  "let": {
    "matchIndex": {"MATCH": [
      {"var": "products"},
      {"var": "category"}, "category",
      {"var": "brand"}, "brand",
      {"var": "size"}, "size"
    ]}
  },
  "in": {
    "if": [
      {">=": [{"var": "matchIndex"}, 0]},
      {"VALUEAT": [{"var": "products"}, {"var": "matchIndex"}]},
      {"error": "Product not found with specified criteria"}
    ]
  }
}
```

### Cached Index Pattern

**When to use:** Need to use the same index multiple times efficiently.

```json
{
  "let": {
    "userIndex": {"INDEXAT": [{"var": "userId"}, {"var": "users"}, "id"]}
  },
  "in": {
    "if": [
      {">=": [{"var": "userIndex"}, 0]},
      {
        "name": {"VALUEAT": [{"var": "users"}, {"var": "userIndex"}, "name"]},
        "email": {"VALUEAT": [{"var": "users"}, {"var": "userIndex"}, "email"]},
        "role": {"VALUEAT": [{"var": "users"}, {"var": "userIndex"}, "role"]},
        "age": {"VALUEAT": [{"var": "users"}, {"var": "userIndex"}, "age"]}
      },
      null
    ]
  }
}
```

**Why this is better:** The index search happens once, stored in `userIndex`, then reused four times. Without `let`, the search would happen four times.

---

## Related Operators

- **[Array Operators](operators-array.md)** - `filter`, `map`, `find` alternatives
- **[Comparison Operators](operators-comparison.md)** - Build conditions
- **[Logical Operators](operators-logical.md)** - Combine conditions
- **[Utility Operators](operators-utility.md)** - UI helpers like `MAPOPTIONS` for tables

---

## Performance Notes

- **Linear search** for MATCH, FINDINDEX, CHOOSE
- **Indexed access** for VALUEAT is O(1)
- **Early termination** on first match
- **Table caching** recommended for repeated lookups

---

## Troubleshooting

### Issue: Getting `null` instead of expected value

**Problem:** Your lookup returns `null` even though you expect data.

**Common causes:**
1. **Index not found** - The search operator returned `-1`
2. **Column name typo** - Field name doesn't match exactly
3. **Wrong row index** - Off-by-one error or incorrect calculation

**Solution - Add defensive checks:**
```json
{
  "let": {
    "index": {"INDEXAT": [{"var": "searchKey"}, {"var": "table"}, "id"]}
  },
  "in": {
    "if": [
      {">=": [{"var": "index"}, 0]},
      {"VALUEAT": [{"var": "table"}, {"var": "index"}, "value"]},
      "NOT_FOUND"  // Fallback value
    ]
  }
}
```

### Issue: MATCHRANGE not finding expected row

**Problem:** Range lookup returns `-1` or wrong row.

**Common causes:**
1. **Table not sorted** - Range operators assume sorted data
2. **Inclusive/exclusive confusion** - MATCHRANGE uses `min <= value <= max` (inclusive on both ends)
3. **Overlapping ranges** - Returns first match only

**Solution - Verify your data:**
```json
// Ensure table is sorted and ranges don't overlap
// Example correct range table:
[
  {"min": 0, "max": 25, "bracket": "low"},
  {"min": 26, "max": 50, "bracket": "medium"},  // No overlap
  {"min": 51, "max": 100, "bracket": "high"}
]
```

### Issue: FINDINDEX returns wrong row

**Problem:** FINDINDEX returns unexpected index.

**Common causes:**
1. **Condition format error** - Array triplet order is `[operator, value, field]` not `[field, operator, value]`
2. **AND vs OR confusion** - Multiple conditions are AND, not OR
3. **Type mismatch** - String `"10"` vs number `10`

**Solution - Check condition format:**
```json
// ✅ Correct
{"FINDINDEX": [table, [">", 18, "age"]]}  // age > 18

// ❌ Wrong order
{"FINDINDEX": [table, ["age", ">", 18]]}  // Won't work

// For OR logic, use object expressions
{"FINDINDEX": [
  table,
  {"or": [
    {">": [{"var": "age"}, 18]},
    {"==": [{"var": "status"}, "verified"]}
  ]}
]}
```

### Issue: Performance is slow with large tables

**Problem:** Lookups are taking too long.

**Common causes:**
1. **Repeated searches** - Searching same table multiple times
2. **Nested lookups** - Lookup inside a loop
3. **Large table** - Thousands of rows with linear search

**Solutions:**
```json
// 1. Cache index results using `let`
{
  "let": {
    "userIndex": {"INDEXAT": [{"var": "userId"}, {"var": "users"}, "id"]}
  },
  "in": {
    // Use userIndex multiple times without re-searching
  }
}

// 2. Consider filtering table first
{
  "let": {
    "filtered": {"filter": [
      {"var": "largeTable"},
      {">": [{"var": "date"}, {"var": "cutoffDate"}]}
    ]}
  },
  "in": {
    "VALUEAT": [{"var": "filtered"}, 0]  // Work with smaller subset
  }
}
```

### Issue: Table has wrong structure

**Problem:** Operators fail  or return unexpected results.

**Solution - Validate table structure:**
```json
// Check if table is valid before using
{
  "if": [
    {"and": [
      {"isarray": {"var": "table"}},  // Is an array
      {">": [{"length": {"var": "table"}}, 0]}  // Has rows
    ]},
    {"VALUEAT": [{"var": "table"}, 0, "field"]},
    null  // Fallback for invalid table
  ]
}
```
