---
layout: default
title: String Operators
---

# String Operators

Text manipulation and string processing operators.

## Overview

String operators provide comprehensive text manipulation capabilities for formatting, parsing, searching, and transforming string data. These operators are essential for building user-facing messages, parsing input, and working with text-based data formats.

### Common Use Cases

- **Concatenation**: Build messages, labels, and formatted output with `cat`
- **Extraction**: Get substrings, parts of text with `substr`, `left`, `right`, `mid`
- **Parsing**: Split delimited data with `splittext`, `splitvalue`
- **Searching**: Find text within strings using `search`
- **Formatting**: Display numbers as currency or percentages with `stringformat`
- **Validation**: Check string lengths, patterns, and content

### String Operator Categories

1. **Concatenation**: `cat`, `concat` - Combine strings
2. **Extraction**: `substr`, `left`, `right`, `mid` - Extract portions
3. **Search**: `search` - Find text positions
4. **Splitting**: `splittext`, `splitvalue` - Parse delimited data
5. **Formatting**: `stringformat` - Format numbers as strings
6. **Measurement**: `len`, `length` - Get string length

### Excel Compatibility

Operators with uppercase variants (e.g., `CONCAT`, `SEARCH`, `LEFT`) follow Excel conventions for maximum compatibility with spreadsheet formulas.

## `cat` - Concatenate

Concatenates multiple values into a string.

### Syntax
```json
{"cat": [value1, value2, ...]}
```

### Parameters
- **values** (array): Values to concatenate (coerced to strings)

### Return Type
String - Concatenated result

### Examples

**Basic concatenation:**
```json
{"cat": ["Hello", " ", "World"]}  // → "Hello World"
{"cat": ["User: ", "Alice"]}      // → "User: Alice"
```

**With numbers:**
```json
{"cat": ["Total: ", 42]}          // → "Total: 42"
{"cat": ["$", 99.99]}             // → "$99.99"
```

**With variables:**
```json
// Data: {"first": "John", "last": "Doe"}
{"cat": [
  {"var": "first"},
  " ",
  {"var": "last"}
]}
// → "John Doe"
```

**Empty concatenation:**
```json
{"cat": []}                       // → ""
```

### Optimization
Nested `cat` operations are flattened during compilation:
```json
{"cat": [{"cat": ["a", "b"]}, "c"]}  // Optimized to: {"cat": ["a", "b", "c"]}
```

---

## `concat` - Concatenate (Alias)

Excel-style concatenation operator. Identical to `cat`.

### Syntax
```json
{"concat": [value1, value2, ...]}
{"CONCAT": [value1, value2, ...]}
```

### Examples
```json
{"CONCAT": ["First", "Second", "Third"]}  // → "FirstSecondThird"
```

---

## `substr` - Substring

Extracts a substring from a string.

### Syntax
```json
{"substr": [string, start, length]}
{"substr": [string, start]}
```

### Parameters
- **string** (string): Source string
- **start** (number): Starting index (0-based)
- **length** (number, optional): Number of characters to extract

### Return Type
String - Extracted substring

### Examples

**Basic substring:**
```json
{"substr": ["Hello World", 0, 5]}   // → "Hello"
{"substr": ["Hello World", 6, 5]}   // → "World"
```

**From position to end:**
```json
{"substr": ["Hello World", 6]}      // → "World"
```

**Negative indices:**
```json
{"substr": ["Hello World", -5]}     // → "World" (last 5 chars)
{"substr": ["Hello", -1, 1]}        // → "o"
```

**Out of bounds:**
```json
{"substr": ["Hello", 10, 5]}        // → ""
{"substr": ["Hello", 0, 100]}       // → "Hello"
```

---

## `search` / `SEARCH` - Find Substring

Searches for a substring within a string (case-insensitive). Returns 1-based index.

### Syntax
```json
{"search": [find_text, within_text, start_num]}
{"search": [find_text, within_text]}
```

### Parameters
- **find_text** (string): Text to find
- **within_text** (string): Text to search in
- **start_num** (number, optional): Starting position (1-based)

### Return Type
Number - 1-based position of first match, or `null` if not found

### Examples

**Basic search:**
```json
{"search": ["World", "Hello World"]}     // → 7
{"search": ["Hello", "Hello World"]}     // → 1
```

**Case-insensitive:**
```json
{"SEARCH": ["HELLO", "hello world"]}     // → 1
{"search": ["world", "Hello World"]}     // → 7
```

**With start position:**
```json
// Data: {"text": "Hello World, Hello Universe"}
{"search": ["Hello", {"var": "text"}, 8]}  // → 14 (second "Hello")
```

**Not found:**
```json
{"search": ["xyz", "Hello World"]}       // → null
```

**Find in email:**
```json
{"search": ["@", {"var": "email"}]}      // Position of @ symbol
```

---

## `left` / `LEFT` - Left Characters

Extracts characters from the left side of a string.

### Syntax
```json
{"left": [text, num_chars]}
{"LEFT": [text]}
```

### Parameters
- **text** (string): Source string
- **num_chars** (number, optional): Number of characters (default: 1)

### Return Type
String - Left characters

### Examples

**Basic left:**
```json
{"left": ["Hello World", 5]}        // → "Hello"
{"LEFT": ["ABCDEF", 3]}             // → "ABC"
```

**Default (1 char):**
```json
{"left": ["Hello"]}                 // → "H"
```

**Out of bounds:**
```json
{"left": ["Hi", 10]}                // → "Hi"
```

**Get first name:**
```json
{"left": [
  {"var": "fullName"},
  {"search": [" ", {"var": "fullName"}]}
]}
```

---

## `right` / `RIGHT` - Right Characters

Extracts characters from the right side of a string.

### Syntax
```json
{"right": [text, num_chars]}
{"RIGHT": [text]}
```

### Parameters
- **text** (string): Source string
- **num_chars** (number, optional): Number of characters (default: 1)

### Return Type
String - Right characters

### Examples

**Basic right:**
```json
{"right": ["Hello World", 5]}       // → "World"
{"RIGHT": ["ABCDEF", 3]}            // → "DEF"
```

**Default (1 char):**
```json
{"right": ["Hello"]}                // → "o"
```

**Get file extension:**
```json
{"right": [
  {"var": "filename"},
  {"-": [
    {"length": {"var": "filename"}},
    {"search": [".", {"var": "filename"}]}
  ]}
]}
```

---

## `mid` / `MID` - Middle Characters

Extracts characters from the middle of a string.

### Syntax
```json
{"mid": [text, start_num, num_chars]}
{"MID": [text, start_num, num_chars]}
```

### Parameters
- **text** (string): Source string
- **start_num** (number): Starting position (1-based in Excel style, 0-based internally)
- **num_chars** (number): Number of characters

### Return Type
String - Extracted characters

### Examples

**Basic mid:**
```json
{"MID": ["Hello World", 7, 5]}      // → "World"
{"mid": ["ABCDEFGH", 3, 4]}         // → "DEFG"
```

**Extract middle:**
```json
// Data: {"text": "Hello World"}
{"mid": [{"var": "text"}, 6, 5]}    // → " Worl"
```

---

## `len` / `LEN` / `length` - String/Array Length

Returns the length of a string, array, or object.

### Syntax
```json
{"len": value}
{"LEN": value}
{"length": value}
```

### Parameters
- **value** (string/array/object): Value to measure

### Return Type
Number - Length/count

### Examples

**String length:**
```json
{"len": "Hello"}                    // → 5
{"LEN": "Hello World"}              // → 11
```

**Array length:**
```json
{"length": [1, 2, 3, 4, 5]}         // → 5
```

**Object length:**
```json
{"length": {"a": 1, "b": 2, "c": 3}}  // → 3
```

**With variables:**
```json
// Data: {"name": "Alice"}
{"len": {"var": "name"}}            // → 5
```

**Validation:**
```json
{">": [{"len": {"var": "password"}}, 8]}  // Password must be > 8 chars
```

---

## `splittext` / `SPLITTEXT` - Split and Get Index

Splits a string and returns element at specific index.

### Syntax
```json
{"splittext": [text, separator, index]}
```

### Parameters
- **text** (string): String to split
- **separator** (string): Delimiter
- **index** (number): Index to return (0-based)

### Return Type
String - Element at specified index

### Examples

**Split CSV:**
```json
// Data: {"csv": "a,b,c,d,e"}
{"splittext": [{"var": "csv"}, ",", 2]}   // → "c"
```

**Split by space:**
```json
{"SPLITTEXT": ["Hello World Universe", " ", 1]}  // → "World"
```

**Extract domain:**
```json
{"splittext": [{"var": "email"}, "@", 1]}  // Get part after @
```

---

## `splitvalue` / `SPLITVALUE` - Split to Array

Splits a string into an array.

### Syntax
```json
{"splitvalue": [string, separator]}
{"SPLITVALUE": [string, separator]}
```

### Parameters
- **string** (string): String to split
- **separator** (string): Delimiter

### Return Type
Array - Array of string parts

### Examples

**Split CSV:**
```json
{"splitvalue": ["a,b,c", ","]}      // → ["a", "b", "c"]
```

**Split by pipe:**
```json
{"SPLITVALUE": ["one|two|three", "|"]}  // → ["one", "two", "three"]
```

**Split and process:**
```json
{"map": [
  {"splitvalue": [{"var": "tags"}, ","]},
  {"cat": ["#", {"var": ""}]}
]}
```

---

## `stringformat` / `STRINGFORMAT` - Format Number as String

Formats a number as a string with thousands separators, decimals, and affixes.

### Syntax
```json
{"stringformat": [value, decimals, prefix, suffix, thousands_sep]}
```

### Parameters
- **value** (number): Number to format
- **decimals** (number, optional): Decimal places (default: 0)
- **prefix** (string, optional): Text before number
- **suffix** (string, optional): Text after number
- **thousands_sep** (string, optional): Thousands separator (default: ",")

### Return Type
String - Formatted number

### Examples

**Basic formatting:**
```json
{"STRINGFORMAT": [1234.567, 2]}     // → "1,234.57"
{"stringformat": [1000000, 0]}      // → "1,000,000"
```

**With prefix:**
```json
{"STRINGFORMAT": [99.99, 2, "$"]}   // → "$99.99"
```

**With suffix:**
```json
{"stringformat": [75, 0, "", "%"]}  // → "75%"
```

**Full formatting:**
```json
{"STRINGFORMAT": [1234567.89, 2, "$", " USD", ","]}
// → "$1,234,567.89 USD"
```

**Custom separator:**
```json
{"stringformat": [1234567, 0, "", "", "."]}
// → "1.234.567" (European format)
```

**Currency display:**
```json
{"cat": [
  {"STRINGFORMAT": [{"var": "amount"}, 2, "$"]},
  " (Tax: ",
  {"STRINGFORMAT": [{"*": [{"var": "amount"}, 0.1]}, 2, "$"]},
  ")"
]}
```

---

## Complex Examples

### Build Full Name
```json
{"cat": [
  {"var": "firstName"},
  " ",
  {"var": "middleInitial"},
  ". ",
  {"var": "lastName"}
]}
```

### Email Validation Pattern
```json
{"and": [
  {">": [{"search": ["@", {"var": "email"}]}, 0]},
  {">": [{"search": [".", {"var": "email"}]}, {"search": ["@", {"var": "email"}]}]}
]}
```

### Extract File Name Without Extension
```json
{"left": [
  {"var": "filename"},
  {"-": [
    {"length": {"var": "filename"}},
    {"-": [
      {"length": {"var": "filename"}},
      {"search": [".", {"var": "filename"}]}
    ]}
  ]}
]}
```

### Format Phone Number
```json
{"cat": [
  "(",
  {"left": [{"var": "phone"}, 3]},
  ") ",
  {"mid": [{"var": "phone"}, 3, 3]},
  "-",
  {"right": [{"var": "phone"}, 4]}
]}
// "1234567890" → "(123) 456-7890"
```

### Price Display
```json
{"if": [
  {">": [{"var": "discount"}, 0]},
  {"cat": [
    {"STRINGFORMAT": [{"var": "price"}, 2, "$"]},
    " (Save ",
    {"STRINGFORMAT": [{"var": "discount"}, 0, "", "%"]},
    ")"
  ]},
  {"STRINGFORMAT": [{"var": "price"}, 2, "$"]}
]}
```

---

## Troubleshooting

### Issue: Search returns null when text exists

**Problem:** `search` operator returns null even though the text is in the string.

**Common causes:**
1. **Case sensitivity mismatch** - Though `search` is case-insensitive, the implementation might differ
2. **Wrong parameter order** - Search text and within text swapped
3. **Start position past occurrence** - Starting search after the text location

**Solutions:**
```json
// ❌ Wrong parameter order
{"search": ["Hello World", "World"]}  // Wrong order

// ✅ Correct - search text first, within text second
{"search": ["World", "Hello World"]}  // → 7

// ✅ Case insensitive
{"search": ["world", "Hello World"]}  // → 7 (finds "World")
```

### Issue: Substr with negative index doesn't work

**Problem:** Negative indices in `substr` return unexpected results.

**Solution:** Negative indices count from the end. Ensure the logic is correct:

```json
// Get last N characters
{"substr": ["Hello World", -5]}  // → "World" (last 5 chars)

// ✅ To get "all but last N", calculate explicitly
{"substr": [
  {"var": "text"},
  0,
  {"-": [
    {"length": {"var": "text"}},
    5  // Remove last 5
  ]}
]}
```

### Issue: StringFormat shows wrong decimal places

**Problem:** Number formatting displays incorrect precision.

**Common causes:**
1. **Rounding not applied** - Float precision issues
2. **Wrong decimal parameter** - Confusion between parameters

**Solutions:**
```json
// ❌ Precision issue
{"stringformat": [1.005, 2]}  // Might show "1.00" due to float representation

// ✅ Round before formatting
{"stringformat": [
  {"round": [1.005, 2]},
  2
]}  // → "1.01"

// ✅ Ensure decimals parameter is number, not string
{"stringformat": [1234.567, 2]}  // ✓ Correct
{"stringformat": [1234.567, "2"]}  // Might fail
```

### Issue: Split returns wrong element

**Problem:** `splittext` returns unexpected array element.

**Common causes:**
1. **Zero-based indexing** - First element is index 0
2. **Empty segments** - Multiple consecutive separators create empty strings
3. **Separator not found** - Returns entire string

**Solutions:**
```json
// Data: {"csv": "a,b,c,d"}

// ❌ Wrong - expects 1-based indexing
{"splittext": [{"var": "csv"}, ",", 3]}  // → "d" (4th element, 0-based)

// ✅ Correct - use 0-based index
{"splittext": [{"var": "csv"}, ",", 2]}  // → "c" (3rd element)

// ✅ Handle empty segments
// "a,,c" split by "," → ["a", "", "c"]
{"splittext": ["a,,c", ",", 1]}  // → "" (empty string)
```

### Issue: Cat with null produces "null" string

**Problem:** Concatenation includes the word "null" in output.

**Solution:** Use `ifnull` to provide defaults:

```json
// ❌ Results in "Hello null"
// Data: {"name": null}
{"cat": ["Hello ", {"var": "name"}]}

// ✅ Provide default value
{"cat": [
  "Hello ",
  {"ifnull": [{"var": "name"}, "Guest"]}
]}  // → "Hello Guest"
```

### Issue: Search start position produces unexpected results

**Problem:** Using start position in `search` doesn't find expected occurrence.

**Common causes:**
1. **1-based vs 0-based confusion** - Search uses 1-based positions
2. **Inclusive vs exclusive** - Start position is inclusive

**Solutions:**
```json
// Data: {"text": "Hello Hello World"}

// Find second occurrence of "Hello"
{"search": ["Hello", {"var": "text"}, 7]}  // 1-based, starts after first "Hello"
// → 7 (finds second "Hello")

// ✅ Find position AFTER first occurrence
{
  "let": {
    "firstPos": {"search": ["Hello", {"var": "text"}]},
    "secondPos": {"search": [
      "Hello",
      {"var": "text"},
      {"+": [{"var": "firstPos"}, 1]}  // Start after first match
    ]}
  },
  "in": {"var": "secondPos"}
}
```

### Issue: Len returns wrong count for Unicode

**Problem:** String length incorrect for emoji or special characters.

**Note:** `len`/`length` counts UTF-8 characters correctly in Rust implementation. If seeing wrong counts, check:

```json
// Most implementations handle Unicode correctly
{"len": "Hello 👋"}  // → 7 (counts emoji as 1 char)

// If seeing wrong counts, ensure proper UTF-8 encoding in input data
```

---

## Best Practices

1. **Use cat for readability**
   ```json
   {"cat": [a, " ", b]}  // ✓ Clear
   ```

2. **Check for empty strings**
   ```json
   {"if": [
     {">": [{"len": {"var": "text"}}, 0]},
     process,
     default
   ]}
   ```

3. **Use splitvalue for parsing**
   ```json
   {"splitvalue": [csv, ","]}
   ```

4. **Format currency consistently**
   ```json
   {"STRINGFORMAT": [amount, 2, "$"]}
   ```

5. **Handle search not found**
   ```json
   {"ifnull": [
     {"search": ["needle", "haystack"]},
     -1
   ]}
   ```

---

## Related Operators

- **[Comparison Operators](operators-comparison.md)** - Compare strings
- **[Array Operators](operators-array.md)** - Process split results
- **[Math Functions](operators-math.md)** - `round` for number formatting

---

## Performance Notes

- **Automatic flattening** optimizes nested `cat` operations
- **Zero-copy** string operations where possible
- **Case-insensitive search** has minimal overhead
