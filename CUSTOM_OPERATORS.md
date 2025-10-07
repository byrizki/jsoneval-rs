# Custom JSON Logic Operators

This document lists all custom operators implemented in the RLogic library, extending the standard JSON Logic specification.

## Math Operators

### `abs` - Absolute Value
Returns the absolute value of a number.
```json
{"abs": -5} // Returns 5
{"abs": {"var": "value"}} // Returns |value|
```

### `max` - Maximum
Returns the maximum value from an array of numbers.
```json
{"max": [1, 5, 3, 9, 2]} // Returns 9
```

### `min` - Minimum
Returns the minimum value from an array of numbers.
```json
{"min": [1, 5, 3, 9, 2]} // Returns 1
```

### `pow` or `**` - Power
Raises a number to a power.
```json
{"pow": [2, 3]} // Returns 8 (2^3)
{"**": [2, 3]} // Same as pow
```

### `round`, `ROUND` - Round
Rounds a number to the nearest integer.
```json
{"round": 3.7} // Returns 4
{"ROUND": 3.2} // Returns 3
```

### `roundup`, `ROUNDUP` - Round Up (Ceiling)
Rounds a number up to the next integer.
```json
{"ROUNDUP": 3.2} // Returns 4
```

### `rounddown`, `ROUNDDOWN` - Round Down (Floor)
Rounds a number down to the previous integer.
```json
{"ROUNDDOWN": 3.9} // Returns 3
```

## String Operators

### `length` - Get Length
Returns the length of a string, array, or object.
```json
{"length": {"var": "items"}} // Returns item count
```

### `len`, `LEN` - String Length
Returns the character count of a string.
```json
{"LEN": "Hello"} // Returns 5
```

### `search`, `SEARCH` - Search Text
Finds text within text (case-insensitive, 1-indexed).
```json
{"SEARCH": ["world", "Hello World"]} // Returns 7
{"SEARCH": ["text", "content", 3]} // Start from position 3
```

### `left`, `LEFT` - Left Substring
Returns the leftmost N characters.
```json
{"LEFT": ["Hello World", 5]} // Returns "Hello"
{"LEFT": ["Text", 2]} // Returns "Te"
```

### `right`, `RIGHT` - Right Substring
Returns the rightmost N characters.
```json
{"RIGHT": ["Hello World", 5]} // Returns "World"
```

### `mid`, `MID` - Middle Substring
Extracts a substring from a specific position.
```json
{"MID": ["Hello World", 7, 5]} // Returns "World" (from position 7, 5 chars)
```

### `splittext`, `SPLITTEXT` - Split and Extract
Splits a string and returns element at index.
```json
{"SPLITTEXT": ["a,b,c", ",", 1]} // Returns "b"
```

### `concat`, `CONCAT` - Concatenate
Joins multiple values into a string.
```json
{"CONCAT": ["Hello", " ", "World"]} // Returns "Hello World"
```

### `splitvalue`, `SPLITVALUE` - Split to Array
Splits a string into an array.
```json
{"SPLITVALUE": ["a,b,c", ","]} // Returns ["a", "b", "c"]
```

## Logical Operators

### `xor` - Exclusive OR
Returns true if exactly one operand is truthy.
```json
{"xor": [true, false]} // Returns true
{"xor": [true, true]} // Returns false
```

### `ifnull`, `IFNULL` - If Null Coalesce
Returns alternative if value is null or empty.
```json
{"IFNULL": [{"var": "optional"}, "default"]} // Returns "default" if optional is null
```

### `isempty`, `ISEMPTY` - Check Empty
Returns true if value is null or empty string.
```json
{"ISEMPTY": ""} // Returns true
{"ISEMPTY": "text"} // Returns false
```

### `empty`, `EMPTY` - Empty String
Returns an empty string.
```json
{"EMPTY": []} // Returns ""
```

## Date Operators

### `today`, `TODAY` - Current Date
Returns today's date at midnight (ISO format).
```json
{"TODAY": []} // Returns "2024-01-01T00:00:00.000Z"
```

### `now`, `NOW` - Current DateTime
Returns current date and time (ISO format).
```json
{"NOW": []} // Returns "2024-01-01T12:34:56.789Z"
```

### `days`, `DAYS` - Days Between
Returns the number of days between two dates.
```json
{"DAYS": ["2024-01-10", "2024-01-01"]} // Returns 9
```

### `year`, `YEAR` - Extract Year
Extracts the year from a date string.
```json
{"YEAR": "2024-03-15"} // Returns 2024
```

### `month`, `MONTH` - Extract Month
Extracts the month from a date string (1-12).
```json
{"MONTH": "2024-03-15"} // Returns 3
```

### `day`, `DAY` - Extract Day
Extracts the day from a date string.
```json
{"DAY": "2024-03-15"} // Returns 15
```

### `date`, `DATE` - Create Date
Creates a date from year, month, day.
```json
{"DATE": [2024, 3, 15]} // Returns "2024-03-15T00:00:00.000Z"
```

## Array/Table Operators

### `sum`, `SUM` - Sum Values
Sums numeric values in an array or field.
```json
{"SUM": [1, 2, 3, 4, 5]} // Returns 15
{"SUM": [{"var": "items"}, "price"]} // Sums the "price" field
```

## Performance Features

All custom operators benefit from:
- **Pre-compilation**: Logic is compiled once and reused
- **Automatic caching**: Results are cached and invalidated on data changes
- **Mutation tracking**: Data changes trigger cache invalidation automatically
- **Zero-copy where possible**: Efficient memory usage

## Example Usage

```rust
use json_eval_rs::{RLogic, TrackedData};
use serde_json::json;

let mut engine = RLogic::new();

// Compile complex logic with custom operators
let logic_id = engine.compile(&json!({
    "CONCAT": [
        {"LEFT": [{"var": "name"}, 10]},
        " - Score: ",
        {"ROUND": {"*": [{"var": "score"}, 100]}}
    ]
})).unwrap();

// Evaluate with tracked data
let data = TrackedData::new(json!({
    "name": "John Doe",
    "score": 0.856
}));

let result = engine.evaluate(&logic_id, &data).unwrap();
// Returns: "John Doe - Score: 86"

// Cache stats
let stats = engine.cache_stats();
println!("Cache hit rate: {:.2}%", stats.hit_rate * 100.0);
```

## Running Benchmarks

```bash
cargo run --release --bin rlogic_bench
```

## Running Tests

```bash
cargo test --lib
```
