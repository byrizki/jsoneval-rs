# JsonEvalRs - C# Bindings

High-performance JSON Logic evaluator with schema validation and dependency tracking for .NET applications.

## Features

- 🚀 **Blazing Fast** - Built on Rust for maximum performance
- ✅ **Schema Validation** - Validate data against JSON schema rules
- 🔄 **Dependency Tracking** - Auto-update dependent fields
- 🎯 **Type Safe** - Full .NET type safety with JSON.NET integration
- 📦 **Cross-Platform** - Works on Windows, Linux, and macOS
- 🔌 **Easy Integration** - Simple API, just a few lines of code

## Installation

```bash
dotnet add package JsonEvalRs
```

Or via NuGet Package Manager:

```
Install-Package JsonEvalRs
```

## Quick Start

```csharp
using JsonEvalRs;
using Newtonsoft.Json.Linq;

// Create evaluator with schema
string schema = @"{
    ""type"": ""object"",
    ""properties"": {
        ""user"": {
            ""type"": ""object"",
            ""properties"": {
                ""name"": {
                    ""type"": ""string"",
                    ""rules"": {
                        ""required"": { ""value"": true, ""message"": ""Name is required"" },
                        ""minLength"": { ""value"": 3, ""message"": ""Min 3 characters"" }
                    }
                },
                ""age"": {
                    ""type"": ""number"",
                    ""rules"": {
                        ""minValue"": { ""value"": 18, ""message"": ""Must be 18+"" }
                    }
                }
            }
        }
    }
}";

using (var eval = new JSONEval(schema))
{
    // Evaluate
    string data = @"{ ""user"": { ""name"": ""John"", ""age"": 25 } }";
    JObject result = eval.Evaluate(data);
    Console.WriteLine($"Evaluated: {result}");

    // Validate
    ValidationResult validation = eval.Validate(data);
    if (validation.HasError)
    {
        foreach (var error in validation.Errors)
        {
            Console.WriteLine($"{error.Path}: {error.Message}");
        }
    }

    // Re-evaluate after changes
    string updatedData = @"{ ""user"": { ""name"": ""John"", ""age"": 30 } }";
    JObject updated = eval.EvaluateDependents(
        new List<string> { "user.age" },
        updatedData,
        nested: true
    );
}
```

## API Reference

### JSONEval Constructor

```csharp
public JSONEval(string schema, string context = null, string data = null)
```

Creates a new evaluator instance.

**Parameters:**
- `schema` (string): JSON schema definition
- `context` (string, optional): Context data for evaluations
- `data` (string, optional): Initial data

### Evaluate Method

```csharp
public JObject Evaluate(string data, string context = null)
```

Evaluates the schema with provided data.

**Parameters:**
- `data` (string): JSON data to evaluate
- `context` (string, optional): Context data

**Returns:** JObject with evaluated schema

### Validate Method

```csharp
public ValidationResult Validate(string data, string context = null)
```

Validates data against schema rules.

**Parameters:**
- `data` (string): JSON data to validate
- `context` (string, optional): Context data

**Returns:** ValidationResult with errors (if any)

### EvaluateDependents Method

```csharp
public JObject EvaluateDependents(
    List<string> changedPaths, 
    string data, 
    string context = null, 
    bool nested = true
)
```

Re-evaluates fields that depend on changed paths.

**Parameters:**
- `changedPaths` (List<string>): Paths that changed
- `data` (string): Updated data
- `context` (string, optional): Context data
- `nested` (bool): Follow dependency chains recursively

**Returns:** JObject with updated evaluated schema

## Validation Rules

Supported validation rules:

- **required** - Field must have a value
- **minLength** / **maxLength** - String/array length validation
- **minValue** / **maxValue** - Numeric range validation
- **pattern** - Regex pattern matching

## Platform Support

- **.NET Standard 2.0+** - Compatible with .NET Framework 4.6.1+
- **.NET 6/7/8** - Latest .NET versions
- **Windows** - x64
- **Linux** - x64
- **macOS** - x64, ARM64

## Performance

Typical performance characteristics:
- Schema parsing: < 5ms
- Evaluation: < 10ms for complex schemas
- Validation: < 5ms

## Error Handling

All methods throw `JsonEvalException` on errors:

```csharp
try
{
    var result = eval.Evaluate(data);
}
catch (JsonEvalException ex)
{
    Console.WriteLine($"Evaluation error: {ex.Message}");
}
```

## Memory Management

`JSONEval` implements `IDisposable`. Always dispose instances:

```csharp
using (var eval = new JSONEval(schema))
{
    // Use eval
} // Automatically disposed

// Or manually
var eval = new JSONEval(schema);
try
{
    // Use eval
}
finally
{
    eval.Dispose();
}
```

## License

MIT License

## Support

- GitHub Issues: https://github.com/yourusername/json-eval-rs/issues
- Documentation: https://github.com/yourusername/json-eval-rs

## Version

```csharp
string version = JSONEval.Version;
Console.WriteLine($"JsonEvalRs version: {version}");
```
