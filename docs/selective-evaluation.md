---
layout: default
title: Selective Evaluation Guide
---

# Selective Evaluation Guide

Selective evaluation allows you to re-evaluate only specific fields in your schema. This drastically improves performance for partial updates, interactive forms, and real-time validation by skipping unchanged parts of the schema logic.

## Concept

When you update data in a large form or configuration, typically only a few fields change. A full re-evaluation is often unnecessary waste.

**Selective Evaluation** solves this by:
1.  Accepting a list of changed field paths.
2.  Identifying which fields depend on those changes.
3.  Re-evaluating **only** the necessary fields.
4.  Keeping the cached results for everything else.

## API Usage

### Rust

```rust
use json_eval_rs::JSONEval;

// Initial full evaluation
let mut eval = JSONEval::new(schema, None, Some(data))?;
eval.evaluate(data, None, None)?;

// ... data changes ...

// Selective re-evaluation
let paths = vec![
    "user.email".to_string(),
    "preferences.theme".to_string()
];
eval.evaluate(new_data, None, Some(&paths))?;
```

### C#

```csharp
using JsonEvalRs;

// Initial evaluation
var eval = new JSONEval(schema);
eval.Evaluate(data);

// ... data changes ...

// Selective re-evaluation
var paths = new[] { "user.email", "preferences.theme" };
eval.Evaluate(newData, paths: paths);
```

### Web / React Native (TypeScript)

```typescript
import { JSONEval } from "@json-eval-rs/web";

const eval = new JSONEval({ schema });
await eval.evaluateJS({ data });

// ... data changes ...

// Selective re-evaluation
await eval.evaluateJS({
  data: newData,
  paths: ["user.email", "preferences.theme"]
});
```

## How It Works

1.  **Dependency Graph**: During schema parsing, `json-eval-rs` builds a directed graph of field dependencies. It knows that `fieldC` depends on `fieldB`, which depends on `fieldA`.
2.  **Path Resolution**: When you pass `paths: ["fieldA"]`, the engine traverses the graph to find all "dependents" (e.g., `fieldB`, `fieldC`).
3.  **Selective Cache Purge**: The cache entries for these specific fields are invalidated. All other cached values (validations, computed properties) remain intact.
4.  **Targeted Execution**: The evaluation engine executes logic only for the invalidated fields.

## Subform Selective Evaluation

Subforms (isolated array item forms) also support selective evaluation. This is critical for performance when editing a single field within a complex list item.

### API

Use `evaluate_subform` with the optional `paths` parameter.

### Rust Example

```rust
// Re-evaluate only the 'quantity' field logic within a specific line item context
let subform_path = "#/line_items";
let item_data = r#"{ "quantity": 5, "price": 100 }"#; // Updated data for the item
let changed_paths = vec!["quantity".to_string()];

eval.evaluate_subform(
    subform_path,
    item_data,
    None,
    Some(&changed_paths)
)?;
```

### C# Example

```csharp
var subformPath = "#/line_items";
var itemData = "{\"quantity\": 5, \"price\": 100}";
var paths = new[] { "quantity" };

eval.EvaluateSubform(
    subformPath,
    itemData,
    paths: paths
);
```

### Web / React Native Example

```typescript
const subformPath = "#/line_items";
const itemData = JSON.stringify({ quantity: 5, price: 100 });
const paths = ["quantity"];

await eval.evaluateSubform({
    subformPath,
    data: itemData,
    paths
});
```

## Best Practices

-   **Use Dotted Notation**: Pass paths like `"user.name"` or `"items.0.value"`. The engine automatically normalizes them.
-   **Granularity**: Be precise. If only `user.firstName` changed, don't pass `user`. Passing parent objects will trigger re-evaluation of all children.
-   **Interactive Inputs**: For "type-as-you-validate" features, always use selective evaluation keyed to the active input field.
