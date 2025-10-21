# Subform Feature

## Overview

The subform feature provides **fully isolated** JSONEval instances for array fields with `items` properties. This allows complex array elements to be evaluated, validated, and managed independently from the parent schema.

## Key Concepts

### What is a Subform?

A subform is created automatically when a field in the schema has:
- `type: "array"`
- An `items` property containing the structure for each array element

### Isolation

Each subform is a **completely isolated** JSONEval instance with:
- Its own schema (constructed from `items` content)
- Copy of parent's `$params` for shared constants/references
- Independent evaluation state
- No direct mutation of parent data

## Schema Structure

### Parent Schema Example

```json
{
  "$params": {
    "constants": {
      "MAX_RIDERS": 5
    }
  },
  "riders": {
    "type": "array",
    "title": "Rider",
    "items": {
      "$layout": {
        "type": "VerticalLayout",
        "elements": [
          { "$ref": "#/riders/properties/name" },
          { "$ref": "#/riders/properties/premium" }
        ]
      },
      "properties": {
        "name": {
          "type": "string",
          "title": "Rider Name",
          "rules": {
            "required": { "value": true }
          }
        },
        "premium": {
          "type": "number",
          "title": "Premium Amount"
        }
      }
    }
  }
}
```

### Generated Subform Schema

```json
{
  "$params": {
    "constants": {
      "MAX_RIDERS": 5
    }
  },
  "riders": {
    "type": "object",
    "$layout": {
      "type": "VerticalLayout",
      "elements": [
        { "$ref": "#/riders/properties/name" },
        { "$ref": "#/riders/properties/premium" }
      ]
    },
    "properties": {
      "name": {
        "type": "string",
        "title": "Rider Name",
        "rules": {
          "required": { "value": true }
        }
      },
      "premium": {
        "type": "number",
        "title": "Premium Amount"
      }
    }
  }
}
```

## API Methods

### 1. `evaluate_subform`

Evaluate a subform with data.

```rust
pub fn evaluate_subform(
    &mut self,
    subform_path: &str,
    data: &str,
    context: Option<&str>,
) -> Result<(), String>
```

**Example:**
```rust
let data = json!({
    "riders": {
        "name": "Life Rider",
        "premium": 1000
    }
});
let data_str = serde_json::to_string(&data).unwrap();

eval.evaluate_subform("#/riders", &data_str, None)?;
```

### 2. `validate_subform`

Validate subform data against its schema rules.

```rust
pub fn validate_subform(
    &self,
    subform_path: &str,
    data: &str,
    context: Option<&str>,
    paths: Option<&[String]>,
) -> Result<ValidationResult, String>
```

**Example:**
```rust
let data = json!({ "riders": { "name": "" } }); // Missing premium
let result = eval.validate_subform("#/riders", &data_str, None, None)?;

if result.has_error {
    for (field, error) in result.errors {
        println!("Field: {}, Rule: {}, Message: {}", 
                 field, error.rule_type, error.message);
    }
}
```

### 3. `evaluate_dependents_subform`

Evaluate dependent fields when a field changes in the subform.

```rust
pub fn evaluate_dependents_subform(
    &mut self,
    subform_path: &str,
    changed_path: &str,
    data: Option<&str>,
    context: Option<&str>,
) -> Result<Value, String>
```

**Example:**
```rust
// When riders.premium changes, recalculate dependent fields
eval.evaluate_dependents_subform(
    "#/riders",
    "#/riders/properties/premium",
    Some(&data_str),
    None
)?;
```

### 4. `resolve_layout_subform`

Resolve layout references (e.g., `$ref`) in the subform's layout.

```rust
pub fn resolve_layout_subform(
    &mut self,
    subform_path: &str,
    evaluate: bool,
) -> Result<(), String>
```

**Example:**
```rust
// Resolve layout without evaluation
eval.resolve_layout_subform("#/riders", false)?;

// Resolve layout with evaluation
eval.resolve_layout_subform("#/riders", true)?;
```

### 5. `get_evaluated_schema_subform`

Get the evaluated schema from the subform.

```rust
pub fn get_evaluated_schema_subform(
    &mut self,
    subform_path: &str,
    resolve_layout: bool,
) -> Value
```

**Example:**
```rust
let schema = eval.get_evaluated_schema_subform("#/riders", true);
println!("{}", serde_json::to_string_pretty(&schema).unwrap());
```

### 6. `get_schema_value_subform`

Get all `.value` fields from the subform schema.

```rust
pub fn get_schema_value_subform(
    &mut self,
    subform_path: &str,
) -> Value
```

**Example:**
```rust
let values = eval.get_schema_value_subform("#/riders");
```

### 7. `get_evaluated_schema_without_params_subform`

Get evaluated schema without `$params`.

```rust
pub fn get_evaluated_schema_without_params_subform(
    &mut self,
    subform_path: &str,
    resolve_layout: bool,
) -> Value
```

**Example:**
```rust
let schema = eval.get_evaluated_schema_without_params_subform("#/riders", false);
```

### 8. `get_evaluated_schema_by_path_subform`

Get evaluated schema at a specific path within the subform.

```rust
pub fn get_evaluated_schema_by_path_subform(
    &mut self,
    subform_path: &str,
    schema_path: &str,
    skip_layout: bool,
) -> Option<Value>
```

**Example:**
```rust
let name_schema = eval.get_evaluated_schema_by_path_subform(
    "#/riders",
    "#/riders/properties/name",
    false
);
```

### Helper Methods

#### `get_subform_paths`

Get list of all available subform paths.

```rust
pub fn get_subform_paths(&self) -> Vec<String>
```

**Example:**
```rust
let paths = eval.get_subform_paths();
for path in paths {
    println!("Subform available at: {}", path);
}
```

#### `has_subform`

Check if a subform exists at a given path.

```rust
pub fn has_subform(&self, subform_path: &str) -> bool
```

**Example:**
```rust
if eval.has_subform("#/riders") {
    // Work with riders subform
}
```

## Usage Flow

### Typical Workflow

1. **Create parent JSONEval** - Subforms are automatically detected and created
2. **Check available subforms** - Use `get_subform_paths()` or `has_subform()`
3. **Evaluate subform** - Call `evaluate_subform()` with array element data
4. **Validate data** - Use `validate_subform()` to check rules
5. **Handle changes** - Use `evaluate_dependents_subform()` for reactive updates
6. **Get results** - Use `get_evaluated_schema_subform()` to retrieve computed values

### Example: Complete Flow

```rust
use json_eval_rs::JSONEval;
use serde_json::json;

// 1. Create parent schema with array field
let schema = json!({
    "$params": {
        "constants": { "TAX_RATE": 0.1 }
    },
    "line_items": {
        "type": "array",
        "items": {
            "properties": {
                "price": { "type": "number" },
                "tax": {
                    "type": "number",
                    "$evaluation": {
                        "*": [
                            { "$ref": "#/line_items/properties/price" },
                            { "$ref": "#/$params/constants/TAX_RATE" }
                        ]
                    }
                }
            }
        }
    }
});

let schema_str = serde_json::to_string(&schema).unwrap();
let mut eval = JSONEval::new(&schema_str, None, None).unwrap();

// 2. Check subform exists
assert!(eval.has_subform("#/line_items"));

// 3. Evaluate subform with data
let item_data = json!({
    "line_items": { "price": 100.0 }
});
let item_data_str = serde_json::to_string(&item_data).unwrap();

eval.evaluate_subform("#/line_items", &item_data_str, None).unwrap();

// 4. Get evaluated schema with calculated tax
let result = eval.get_evaluated_schema_subform("#/line_items", false);
println!("Result: {}", serde_json::to_string_pretty(&result).unwrap());
```

## Benefits

### 1. **Full Isolation**
- Subforms cannot accidentally mutate parent data
- Each subform has its own evaluation context
- Clear boundaries between parent and child schemas

### 2. **Performance**
- Subforms are created once at parse time
- No runtime overhead for detection
- Efficient memory usage with boxed instances

### 3. **Flexibility**
- Work with array elements as independent forms
- Validate individual items before adding to array
- Support complex nested structures

### 4. **Consistency**
- Same API as parent JSONEval
- All features work in subforms (evaluation, validation, dependents, layout)
- Predictable behavior

## Implementation Details

### Parse Time Detection

Subforms are detected during schema parsing in a single pass:

```rust
// In parse_schema.rs
fn collect_subform_fields(value: &Value, path: &str, ...) {
    if type == "array" && has items {
        create_subform(path, field_map, items, ...);
        return; // Don't recurse into items
    }
    // Recurse into other children
}
```

### Schema Construction

Each subform schema is built as:

```rust
{
    "$params": <copied from parent>,
    "<field_name>": {
        "type": "object",
        ...items content...
    }
}
```

### Storage

Subforms are stored in the parent JSONEval:

```rust
pub struct JSONEval {
    ...
    pub subforms: IndexMap<String, Box<JSONEval>>,
    ...
}
```

## Limitations

1. **No Direct Data Mutation** - Subforms work with isolated data, changes don't automatically propagate to parent
2. **Path Format** - Subform paths use schema format (`#/field`) not JSON pointer format
3. **Single Level** - Nested subforms (arrays within arrays) each get their own isolated instance

## Testing

Comprehensive test coverage in `tests/test_subforms.rs`:

- ✅ Subform detection and creation
- ✅ Schema structure validation
- ✅ Evaluation with $evaluation expressions
- ✅ Validation rules
- ✅ Dependent field updates
- ✅ Layout resolution
- ✅ Multiple subforms in one schema
- ✅ Isolation verification
- ✅ Error handling

Run tests:
```bash
cargo test --test test_subforms
```

## Future Enhancements

Possible improvements:

1. **Batch Operations** - Evaluate multiple array items at once
2. **Diff/Patch** - Track changes and apply them back to parent
3. **Nested Array Support** - Better handling of arrays within subform arrays
4. **Performance Metrics** - Track subform evaluation times
5. **Lazy Creation** - Create subforms on-demand rather than at parse time
