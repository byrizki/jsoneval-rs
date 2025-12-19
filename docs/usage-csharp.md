---
layout: default
title: C# Usage Guide
---

# C# Usage Guide

The `JsonEvalRs` NuGet package provides high-performance bindings for .NET applications. It wraps the Rust core engine with a user-friendly, type-safe C# API.

## Installation

```bash
dotnet add package JsonEvalRs
```

Or via NuGet Package Manager:
```bash
Install-Package JsonEvalRs
```

## Quick Start

```csharp
using JsonEvalRs;
using Newtonsoft.Json.Linq;

// Define schema with validation rules and layout
string schema = @"{
    ""type"": ""object"",
    ""properties"": {
        ""email"": {
            ""type"": ""string"",
            ""rules"": {
                ""required"": { ""value"": true, ""message"": ""Email is required"" },
                ""pattern"": { 
                    ""value"": ""^[^@]+@[^@]+\\.[^@]+$"", 
                    ""message"": ""Invalid email format"" 
                }
            }
        }
    }
}";

// Use 'using' block for automatic disposal
using (var eval = new JSONEval(schema))
{
    // 1. Evaluate
    string data = @"{ ""email"": ""invalid-email"" }";
    eval.Evaluate(data);

    // 2. Validate
    ValidationResult validation = eval.Validate(data);
    
    if (validation.HasError)
    {
        foreach (var error in validation.Errors)
        {
            Console.WriteLine($"Field: {error.Path}, Error: {error.Message}");
        }
    }

    // 3. Get Evaluated Schema (includes computed properties)
    JObject result = eval.GetEvaluatedSchema();
}
```

## API Reference

### `JSONEval` Class

The main entry point for evaluation. Implements `IDisposable` to manage native resources.

#### Constructor

```csharp
public JSONEval(string schema, string? context = null, string? data = null)
```

-   **schema**: JSON schema definition string.
-   **context**: Optional external context data (e.g., user info, environment vars).
-   **data**: Optional initial data.

#### `Evaluate`

Evaluates the schema against provided data.

```csharp
public void Evaluate(string data, string? context = null, string[]? paths = null)
```

-   **data**: JSON data string.
-   **paths**: **[New]** Optional array of field paths for [Selective Evaluation](selective-evaluation).

#### `Validate`

Validates data against rules defined in the schema.

```csharp
public ValidationResult Validate(string data, string? context = null)
```

Returns `ValidationResult` containing:
-   `HasError` (bool)
-   `Errors` (List of `ValidationError`)

#### `EvaluateDependents`

Efficiently re-evaluates fields that depend on specific changes.

```csharp
public JArray EvaluateDependents(
    string[] changedPaths, 
    string? data = null, 
    string? context = null, 
    bool reEvaluate = false
)
```

-   **changedPaths**: List of paths that have changed (e.g., `["user.email"]`).
-   **reEvaluate**: If true, triggers a full `Evaluate` call after determining dependencies.

### Subform Methods

For managing isolated array items (Subforms).

#### `EvaluateSubform`

```csharp
public void EvaluateSubform(
    string subformPath, 
    string data, 
    string? context = null, 
    IEnumerable<string>? paths = null
)
```

-   **subformPath**: Schema reference to the array field (e.g., `#/line_items`).
-   **data**: Data for the single item being evaluated.
-   **paths**: Optional paths for selective evaluation within the subform.

#### `EvaluateDependentsSubform`

```csharp
public JArray EvaluateDependentsSubform(
    string subformPath, 
    string changedPath, 
    string? data = null, 
    string? context = null
)
```

## Platform Support

-   **.NET Standard 2.0+**
-   **.NET 6/7/8+**
-   **Platforms**: Windows (x64), Linux (x64), macOS (x64, ARM64)

## Performance & Concurrency

The library is thread-safe for **read interactions** but requires exclusive access for **write/evaluate** operations on the same instance. For high-concurrency server scenarios:

1.  **Parse Once, Reuse Many**: Parse the schema once and create a new `JSONEval` instance for each request using `JSONEval.FromCache`.
2.  **Dispose**: Always call `Dispose()` or use `using` blocks to prevent memory leaks.

```csharp
// Global initialization
var schemaKey = "main_schema";
// Assumes you have a way to pre-cache the schema (advanced usage)

// Per-request
using (var eval = JSONEval.FromCache(schemaKey)) 
{
    eval.Evaluate(requestData);
}
```
