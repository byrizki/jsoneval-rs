# Migration Guide: Modular C# Binding Structure

## Overview

The C# binding has been refactored from a single monolithic file (`JsonEvalRs.cs` - 1495 lines) into a modular architecture with separate files for better maintainability.

## What Changed

### Old Structure (Single File)
```
JsonEvalRs.cs (1495 lines)
├── Native class with #if directives
├── Validation types
├── Exception types  
├── JSONEval class
└── Subform methods
```

### New Structure (Modular)
```
JsonEvalRs.Shared.cs (54 lines)
├── ValidationError
├── ValidationResult
├── CacheStats
└── JsonEvalException

JsonEvalRs.Native.Common.cs (78 lines)
├── Native partial class (common FFI)
└── Platform-independent declarations

JsonEvalRs.Native.NetCore.cs (227 lines)
├── Native partial class (.NET Core/.NET 5+)
├── DLL resolver
└── UTF-8 string marshalling

JsonEvalRs.Native.NetStandard.cs (191 lines)
├── Native partial class (.NET Standard 2.0)
├── Byte array marshalling helpers
└── Compatibility implementations

JsonEvalRs.Main.cs (696 lines)
├── JSONEval partial class
└── Core evaluation methods

JsonEvalRs.Subforms.cs (249 lines)
├── JSONEval partial class
└── Subform-specific methods
```

## Benefits

1. **Separation of Concerns**: Each file has a single, clear responsibility
2. **Easier Maintenance**: Platform-specific code is isolated
3. **Better Readability**: No more deeply nested #if blocks
4. **Modular Testing**: Individual components can be tested separately
5. **Clear Dependencies**: Platform requirements are explicit

## For Library Users

### ✅ No Code Changes Required

Your existing code continues to work without any modifications:

```csharp
using JsonEvalRs;

// All existing code works exactly the same
var eval = new JSONEval(schema);
var result = eval.Evaluate(data);
var validation = eval.Validate(data);
eval.EvaluateSubform(subformPath, data);
```

### Package Contents

The NuGet package now includes:
- All modular source files (compiled into the same assembly)
- Documentation in `/docs/README.modular.md`
- Same native libraries as before

## For Library Developers

### Building the Library

```bash
# Clean build
dotnet clean
dotnet build

# The build system automatically:
# 1. Compiles all modular files
# 2. Excludes the old JsonEvalRs.cs
# 3. Uses conditional compilation for platform-specific code
```

### File Responsibilities

| File | Purpose | Dependencies |
|------|---------|-------------|
| **JsonEvalRs.Shared.cs** | Common types/exceptions | Newtonsoft.Json |
| **JsonEvalRs.Native.Common.cs** | Platform-agnostic FFI | System.Runtime.InteropServices |
| **JsonEvalRs.Native.NetCore.cs** | .NET Core/.NET 5+ FFI | NETCOREAPP \|\| NET5_0_OR_GREATER |
| **JsonEvalRs.Native.NetStandard.cs** | .NET Standard FFI | !(NETCOREAPP \|\| NET5_0_OR_GREATER) |
| **JsonEvalRs.Main.cs** | Core JSONEval methods | All Native files |
| **JsonEvalRs.Subforms.cs** | Subform methods | All Native files |

### Adding New Methods

1. **Platform-agnostic method**: Add to `JsonEvalRs.Main.cs` or `JsonEvalRs.Subforms.cs`
2. **FFI declaration**: 
   - Common: Add to `JsonEvalRs.Native.Common.cs`
   - .NET Core: Add to `JsonEvalRs.Native.NetCore.cs`
   - .NET Standard: Add to `JsonEvalRs.Native.NetStandard.cs`

### Example: Adding a New Method

```csharp
// Step 1: Add to JsonEvalRs.Native.Common.cs (if no string marshalling)
[DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
internal static extern FFIResult json_eval_new_method(IntPtr handle);

// OR Step 1: Add to both platform-specific files (if string marshalling needed)

// JsonEvalRs.Native.NetCore.cs
[DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
internal static extern FFIResult json_eval_new_method(
    IntPtr handle,
    [MarshalAs(UnmanagedType.LPUTF8Str)] string param
);

// JsonEvalRs.Native.NetStandard.cs
[DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
internal static extern FFIResult json_eval_new_method(
    IntPtr handle,
    byte[]? param
);

// Step 2: Add public method to JsonEvalRs.Main.cs
public void NewMethod(string param)
{
    ThrowIfDisposed();
#if NETCOREAPP || NET5_0_OR_GREATER
    var result = Native.json_eval_new_method(_handle, param);
#else
    var result = Native.json_eval_new_method(_handle, Native.ToUTF8Bytes(param));
#endif
    // Process result...
}
```

## Backward Compatibility

The old `JsonEvalRs.cs` file is **excluded** from compilation but kept in the repository for reference:

```xml
<!-- In JsonEvalRs.csproj -->
<Compile Remove="JsonEvalRs.cs" Condition="Exists('JsonEvalRs.cs')" />
```

You can safely delete it after verifying the modular version works for your use case.

## Testing

Run the existing test suite to verify compatibility:

```bash
cd bindings/csharp-example
dotnet run
```

All tests should pass without modification.

## Questions or Issues?

- Check `README.modular.md` for architecture details
- Review individual files for specific implementations
- File an issue if you find any discrepancies

## Summary

✅ **Zero breaking changes** for library users  
✅ **Better maintainability** for developers  
✅ **Same performance** and functionality  
✅ **Clearer code organization**  

The modular structure is a refactoring that improves code quality without changing behavior.
