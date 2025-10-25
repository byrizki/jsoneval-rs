---
layout: default
title: Utility Operators
---

# Utility Operators

Helper functions and UI utility operators.

## `missing` - Check Missing Keys

Returns an array of keys that are missing or null in the data.

### Syntax
```json
{"missing": ["key1", "key2", ...]}
{"missing": "single_key"}
```

### Parameters
- **keys** (array|string): Keys to check

### Return Type
Array - Array of missing key names (empty array if all present)

### Examples

**Check multiple fields:**
```json
// Data: {"name": "Alice", "age": 30}
{"missing": ["name", "age", "email"]}
// → ["email"]
```

**All present:**
```json
// Data: {"name": "Alice", "age": 30}
{"missing": ["name", "age"]}
// → []
```

**Single key:**
```json
// Data: {"name": "Alice"}
{"missing": "email"}
// → ["email"]
```

**Validation:**
```json
{"if": [
  {"==": [{"length": {"missing": ["name", "email"]}}, 0]},
  "Valid",
  "Missing required fields"
]}
```

**Get missing fields:**
```json
{
  "cat": [
    "Missing: ",
    {"reduce": [
      {"missing": ["name", "email", "age"]},
      {"cat": [{"var": "accumulator"}, ", ", {"var": "current"}]},
      ""
    ]}
  ]
}
```

### Notes
- Checks for both missing keys and null values
- Empty strings are not considered missing
- Nested paths supported: `"user.profile.name"`

---

## `missing_some` - Check Minimum Present

Returns missing keys only if fewer than the minimum number of keys are present.

### Syntax
```json
{"missing_some": [minimum, ["key1", "key2", ...]]}
```

### Parameters
- **minimum** (number): Minimum number of required keys
- **keys** (array): Keys to check

### Return Type
Array - Missing keys if requirement not met, empty array otherwise

### Examples

**At least one required:**
```json
// Data: {"phone": "555-1234"}
{"missing_some": [1, ["email", "phone"]]}
// → [] (at least 1 present)
```

**Requirement not met:**
```json
// Data: {}
{"missing_some": [1, ["email", "phone"]]}
// → ["email", "phone"] (need at least 1)
```

**At least two required:**
```json
// Data: {"email": "a@b.com"}
{"missing_some": [2, ["email", "phone", "address"]]}
// → ["phone", "address"] (only 1 present, need 2)
```

**Validation message:**
```json
{"if": [
  {"==": [
    {"length": {"missing_some": [1, ["phone", "email"]]}},
    0
  ]},
  "Valid",
  "Provide at least phone or email"
]}
```

---

## `return` - Return Raw Value

Returns a raw JSON value without evaluation. Useful for returning complex structures as-is.

### Syntax
```json
{"return": any_value}
```

### Parameters
- **any_value** (any): Value to return as-is

### Return Type
Any - The value exactly as specified

### Examples

**Return object:**
```json
{"return": {"status": "ok", "code": 200}}
// → {"status": "ok", "code": 200}
```

**Return array:**
```json
{"return": [1, 2, 3, 4, 5]}
// → [1, 2, 3, 4, 5]
```

**Return complex structure:**
```json
{"return": {
  "user": {"name": "Alice", "role": "admin"},
  "permissions": ["read", "write", "delete"]
}}
```

**Use in conditional:**
```json
{"if": [
  {"var": "error"},
  {"return": {"error": true, "message": "Failed"}},
  {"return": {"success": true, "data": []}}
]}
```

---

## `RANGEOPTIONS` - Generate Range Options

Generates an array of options for UI selects/dropdowns from a numeric range.

### Syntax
```json
{"RANGEOPTIONS": [min, max]}
```

### Parameters
- **min** (number): Minimum value (inclusive)
- **max** (number): Maximum value (inclusive)

### Return Type
Array - Array of objects with `label` and `value` properties

### Examples

**Age range:**
```json
{"RANGEOPTIONS": [18, 65]}
// → [
//   {"label": "18", "value": "18"},
//   {"label": "19", "value": "19"},
//   ...
//   {"label": "65", "value": "65"}
// ]
```

**Year range:**
```json
{"RANGEOPTIONS": [2020, 2024]}
// → [
//   {"label": "2020", "value": "2020"},
//   {"label": "2021", "value": "2021"},
//   {"label": "2022", "value": "2022"},
//   {"label": "2023", "value": "2023"},
//   {"label": "2024", "value": "2024"}
// ]
```

**Invalid range:**
```json
{"RANGEOPTIONS": [10, 5]}  // → [] (min > max)
```

**Dynamic range:**
```json
{"RANGEOPTIONS": [
  {"var": "minAge"},
  {"var": "maxAge"}
]}
```

---

## `MAPOPTIONS` - Map Table to Options

Transforms a table (array of objects) into UI options by extracting label and value fields.

### Syntax
```json
{"MAPOPTIONS": [table, label_field, value_field]}
```

### Parameters
- **table** (array): Array of objects
- **label_field** (string): Field name for option label
- **value_field** (string): Field name for option value

### Return Type
Array - Array of option objects with `label` and `value` properties

### Examples

**Basic mapping:**
```json
// Data: {"countries": [
//   {"code": "US", "name": "United States"},
//   {"code": "CA", "name": "Canada"},
//   {"code": "UK", "name": "United Kingdom"}
// ]}

{"MAPOPTIONS": [{"var": "countries"}, "name", "code"]}
// → [
//   {"label": "United States", "value": "US"},
//   {"label": "Canada", "value": "CA"},
//   {"label": "United Kingdom", "value": "UK"}
// ]
```

**Product options:**
```json
{"MAPOPTIONS": [
  {"var": "products"},
  "displayName",
  "productId"
]}
```

**Same field for label and value:**
```json
{"MAPOPTIONS": [
  {"var": "categories"},
  "name",
  "name"
]}
```

---

## `MAPOPTIONSIF` - Conditional Map to Options

Maps table to options with filtering conditions.

### Syntax
```json
{"MAPOPTIONSIF": [table, label_field, value_field, condition1, condition2, ...]}
```

### Conditions Format

**Triplet format** `[value, operator, field]`:
```json
[value, "==", "fieldName"]  // row.fieldName == value
```

**Standard logic expressions:**
```json
{">": [{"var": "price"}, 100]}
```

### Parameters
- **table** (array): Array of objects
- **label_field** (string): Field for option label
- **value_field** (string): Field for option value
- **conditions** (...): Conditions to filter rows (all must match)

### Return Type
Array - Filtered option objects

### Examples

**Filter active items:**
```json
// Data: {"products": [
//   {"name": "Widget", "id": "W1", "active": true, "price": 10},
//   {"name": "Gadget", "id": "G1", "active": false, "price": 20},
//   {"name": "Tool", "id": "T1", "active": true, "price": 15}
// ]}

{"MAPOPTIONSIF": [
  {"var": "products"},
  "name",
  "id",
  true, "==", "active"
]}
// → [
//   {"label": "Widget", "value": "W1"},
//   {"label": "Tool", "value": "T1"}
// ]
```

**Price range filter:**
```json
{"MAPOPTIONSIF": [
  {"var": "products"},
  "name",
  "id",
  10, ">=", "price",
  20, "<=", "price"
]}
// Products with price between 10 and 20
```

**Category filter:**
```json
{"MAPOPTIONSIF": [
  {"var": "items"},
  "title",
  "itemId",
  {"var": "selectedCategory"}, "==", "category"
]}
```

**Multiple conditions:**
```json
{"MAPOPTIONSIF": [
  {"var": "users"},
  "name",
  "userId",
  true, "==", "active",
  "admin", "==", "role"
]}
// Active admin users only
```

---

## Complex Examples

### Form Validation
```json
{
  "if": [
    {">": [{"length": {"missing": ["name", "email", "phone"]}}, 0]},
    {
      "cat": [
        "Please provide: ",
        {"reduce": [
          {"missing": ["name", "email", "phone"]},
          {"cat": [{"var": "accumulator"}, ", ", {"var": "current"}]},
          ""
        ]}
      ]
    },
    "Form is valid"
  ]
}
```

### Flexible Contact Validation
```json
{
  "if": [
    {">": [
      {"length": {"missing_some": [1, ["email", "phone", "address"]]}},
      0
    ]},
    "Provide at least one contact method",
    "Valid"
  ]
}
```

### Dynamic Dropdown
```json
{
  "if": [
    {"var": "useRange"},
    {"RANGEOPTIONS": [1, 100]},
    {"MAPOPTIONS": [
      {"var": "customOptions"},
      "display",
      "value"
    ]}
  ]
}
```

### Filtered Product Dropdown
```json
{"MAPOPTIONSIF": [
  {"var": "products"},
  "name",
  "productId",
  {"var": "selectedCategory"}, "==", "category",
  true, "==", "inStock",
  0, ">", "quantity"
]}
```

### Conditional Return Structure
```json
{
  "if": [
    {"var": "success"},
    {"return": {
      "status": "success",
      "data": {"var": "result"},
      "timestamp": {"now": null}
    }},
    {"return": {
      "status": "error",
      "error": {"var": "errorMessage"},
      "code": 400
    }}
  ]
}
```

### Multi-Level Options
```json
{
  "merge": [
    {"RANGEOPTIONS": [1, 10]},
    {"MAPOPTIONS": [
      {"var": "specialValues"},
      "label",
      "value"
    ]}
  ]
}
```

---

## Best Practices

1. **Use missing for validation**
   ```json
   {"missing": ["required", "fields"]}
   ```

2. **Use missing_some for flexible requirements**
   ```json
   {"missing_some": [2, ["option1", "option2", "option3"]]}
   ```

3. **Cache option generation** if used multiple times
   ```json
   // Generate options once, store in variable
   ```

4. **Validate before MAPOPTIONS**
   ```json
   {"if": [
     {">": [{"length": {"var": "table"}}, 0]},
     {"MAPOPTIONS": [table, label, value]},
     []
   ]}
   ```

5. **Use MAPOPTIONSIF** instead of filter + map
   ```json
   {"MAPOPTIONSIF": [table, label, value, conditions]}  // ✓
   {"MAPOPTIONS": [{"filter": [...]}, label, value]}    // ✗ Verbose
   ```

---

## UI Integration Patterns

### Select Dropdown Data
```json
{
  "options": {"RANGEOPTIONS": [18, 100]},
  "placeholder": "Select age",
  "required": true
}
```

### Cascading Dropdowns
```json
{
  "categories": {"MAPOPTIONS": [categories, "name", "id"]},
  "products": {"MAPOPTIONSIF": [
    products,
    "name",
    "id",
    {"var": "selectedCategory"}, "==", "categoryId"
  ]}
}
```

### Dynamic Form Fields
```json
{
  "if": [
    {"==": [{"length": {"missing": ["email", "phone"]}}, 2]},
    {"return": {"showContactWarning": true}},
    {"return": {"showContactWarning": false}}
  ]
}
```

---

## Related Operators

- **[Logical Operators](operators-logical.md)** - `if`, `and`, `or` for conditions
- **[Array Operators](operators-array.md)** - `filter`, `map` alternatives
- **[Table Operators](operators-table.md)** - Advanced filtering

---

## Performance Notes

- **missing** checks are optimized for common cases
- **RANGEOPTIONS** efficient for reasonable ranges (<1000 items)
- **MAPOPTIONS** uses zero-copy field extraction
- **MAPOPTIONSIF** short-circuits on condition failure
