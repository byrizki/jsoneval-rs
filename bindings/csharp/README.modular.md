# JsonEvalRs C# Binding - Modular Structure

The C# binding has been modularized into separate files for better maintainability and organization.

## File Structure

### Shared Files (All Platforms)
- **JsonEvalRs.Shared.cs** - Common types and exceptions
  - `ValidationError` - Validation error information
  - `ValidationResult` - Validation result
  - `CacheStats` - Cache statistics
  - `JsonEvalException` - Custom exception type

- **JsonEvalRs.Native.Common.cs** - Platform-independent FFI declarations
  - Common P/Invoke functions that don't require string marshalling
  - Struct definitions (FFIResult)

- **JsonEvalRs.cs** - Main JSONEval class (kept for backward compatibility)
  - Platform-agnostic public API
  - Uses partial classes to incorporate platform-specific implementations

### Platform-Specific Files

#### .NET Core / .NET 5+ (.NET Standard 2.1+)
- **JsonEvalRs.Native.NetCore.cs** (active when `NETCOREAPP || NET5_0_OR_GREATER`)
  - Uses `[MarshalAs(UnmanagedType.LPUTF8Str)]` for better UTF-8 string handling
  - Includes DLL import resolver for .NET 5+
  - More efficient string marshalling

#### .NET Standard 2.0
- **JsonEvalRs.Native.NetStandard.cs** (active when NOT `NETCOREAPP` and NOT `NET5_0_OR_GREATER`)
  - Uses byte array marshalling for compatibility
  - Helper methods: `ToUTF8Bytes()` and `PtrToStringUTF8()`
  - Compatible with older .NET Framework versions

## Benefits

1. **Separation of Concerns** - Platform-specific code is isolated
2. **Maintainability** - Easier to update platform-specific implementations
3. **Readability** - No more nested `#if` blocks in large files
4. **Modularity** - Each file has a single responsibility
5. **Type Safety** - Partial classes ensure compile-time type checking across files

## Build Configuration

The build system uses conditional compilation directives:
- `NETCOREAPP || NET5_0_OR_GREATER` - Activates .NET Core/5+ specific files
- Default (absence of above) - Activates .NET Standard specific files

## Usage

No changes required for consumers of this library. The public API remains the same:

```csharp
using JsonEvalRs;

var eval = new JSONEval(schema, context, data);
var result = eval.Evaluate(data, context);
```

All platform-specific implementation details are handled internally by the build system.
