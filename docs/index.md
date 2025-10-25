---
layout: home
title: Home
---

# JSON-Eval-RS Operator Documentation

Comprehensive documentation for all operators available in json-eval-rs.

## 📚 Documentation Structure

This documentation is organized by operator category for easy navigation:

### Core Operators
- **[Core Operators](operators-core.md)** - Variables, references, and literal values
  - `var`, `$ref`, literals (null, bool, number, string, array)

### Logical & Control Flow
- **[Logical Operators](operators-logical.md)** - Boolean logic and conditional execution
  - `and`, `or`, `not`, `if`, `xor`, `ifnull`, `isempty`, `empty`

### Comparison Operators
- **[Comparison Operators](operators-comparison.md)** - Value comparisons
  - `==`, `===`, `!=`, `!==`, `<`, `<=`, `>`, `>=`

### Arithmetic Operators
- **[Arithmetic Operators](operators-arithmetic.md)** - Mathematical operations
  - `+`, `-`, `*`, `/`, `%`, `^`, `pow`

### String Operators
- **[String Operators](operators-string.md)** - Text manipulation
  - `cat`, `concat`, `substr`, `search`, `left`, `right`, `mid`, `len`, `length`
  - `splittext`, `splitvalue`, `stringformat`

### Math Functions
- **[Math Functions](operators-math.md)** - Advanced mathematical operations
  - `abs`, `max`, `min`, `pow`, `round`, `roundup`, `rounddown`
  - `ceiling`, `floor`, `trunc`, `mround`

### Date Functions
- **[Date Functions](operators-date.md)** - Date and time operations
  - `today`, `now`, `year`, `month`, `day`, `date`, `dateformat`
  - `days`, `yearfrac`, `datedif`

### Array Operators
- **[Array Operators](operators-array.md)** - Array transformations and operations
  - `map`, `filter`, `reduce`, `all`, `some`, `none`
  - `merge`, `in`, `sum`, `for`, `multiplies`, `divides`

### Table Operators
- **[Table/Lookup Operators](operators-table.md)** - Data table operations
  - `VALUEAT`, `MAXAT`, `INDEXAT`, `MATCH`, `MATCHRANGE`
  - `CHOOSE`, `FINDINDEX`

### Utility Operators
- **[Utility Operators](operators-utility.md)** - Helper functions and UI operations
  - `missing`, `missing_some`, `return`
  - `RANGEOPTIONS`, `MAPOPTIONS`, `MAPOPTIONSIF`

## 🚀 Quick Start

### Basic Variable Access
```json
{"var": "user.name"}
```

### Conditional Logic
```json
{"if": [
  {">": [{"var": "age"}, 18]},
  "Adult",
  "Minor"
]}
```

### Array Transformation
```json
{"map": [
  {"var": "numbers"},
  {"*": [{"var": ""}, 2]}
]}
```

### Date Operations
```json
{"days": [{"today": null}, {"var": "birthdate"}]}
```

## 📖 How to Use This Documentation

Each operator documentation page includes:

- **Syntax** - How to write the operator
- **Parameters** - What arguments it accepts
- **Return Type** - What value it returns
- **Examples** - Practical usage examples
- **Edge Cases** - Special behaviors and gotchas
- **Related Operators** - Similar or complementary operators

## 💡 Key Concepts

### Path Notation
json-eval-rs supports multiple path notations:
- **Dot notation**: `"user.profile.name"`
- **JSON Pointer**: `"/user/profile/name"`
- **Array indices**: `"items.0"` or `"items[0]"`

### Context Variables
Special variables available in certain contexts:
- **`$iteration`** - Current iteration index in FOR loops
- **`$loopIteration`** - Loop counter
- **`current`** - Current element in array operations
- **`accumulator`** - Accumulated value in reduce operations
- **Empty string `""`** - Refers to current context in map/filter

### Type Coercion
json-eval-rs follows JavaScript-like type coercion rules:
- Numbers: `"123"` → `123`
- Booleans: `0`, `null`, `""` → `false`; others → `true`
- Strings: All values can be converted to strings

### Operator Aliases
Many operators have multiple names for compatibility:
- Excel-style (uppercase): `ROUND`, `SUM`, `CONCAT`
- JavaScript-style (lowercase): `round`, `sum`, `concat`

## 🔧 Configuration

Some operators respect configuration options:
- **`safe_nan_handling`** - Converts NaN/Infinity to 0 or null
- **`recursion_limit`** - Maximum nesting depth (default: 100)

## 📝 Conventions Used

Throughout this documentation:
- `{...}` - Object/operator notation
- `[...]` - Array notation
- `{"var": "..."}` - Variable access
- `// comment` - Explanatory comments (not valid in JSON)

## 🎯 Finding Operators

**By Use Case:**
- **Data validation**: `missing`, `missing_some`, `isempty`
- **Calculations**: Arithmetic and Math operators
- **Text processing**: String operators
- **Date calculations**: Date operators
- **Array filtering**: `filter`, `all`, `some`, `none`
- **Table lookups**: Table operators

**By Name:**
Navigate to the appropriate category page or use your browser's search function.

## 🌟 Best Practices

1. **Use compiled logic** - Compile once, run many times for better performance
2. **Normalize paths** - Paths are automatically normalized during compilation
3. **Handle missing data** - Use defaults with `var` or `ifnull` operators
4. **Leverage array operations** - Use `map`, `filter`, `reduce` instead of loops
5. **Cache table lookups** - Reuse VALUEAT/INDEXAT results when possible

## 📚 Additional Resources

- **[Main README](../README.md)** - Project overview and installation
- **[Examples](../examples/)** - Practical code examples
- **[Tests](../tests/)** - Comprehensive test suite showing all features

---

**Note**: This implementation extends standard JSON Logic with custom operators for Excel-like functions, table operations, and advanced date/string manipulation.
