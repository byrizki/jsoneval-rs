# C# Binding - Modular Architecture

## Overview

The C# binding has been successfully modularized from a single 1,495-line file into a clean, maintainable architecture with 6 focused files.

## Architecture Summary

### File Structure

```
bindings/csharp/
├── JsonEvalRs.Shared.cs              (54 lines)   - Common types
├── JsonEvalRs.Native.Common.cs       (78 lines)   - Common FFI
├── JsonEvalRs.Native.NetCore.cs      (227 lines)  - .NET Core/.NET 5+ FFI
├── JsonEvalRs.Native.NetStandard.cs  (191 lines)  - .NET Standard FFI
├── JsonEvalRs.Main.cs                (696 lines)  - Core JSONEval class
├── JsonEvalRs.Subforms.cs            (249 lines)  - Subform methods
├── JsonEvalRs.cs                     (EXCLUDED)   - Original monolithic file
├── JsonEvalRs.csproj                              - Project configuration
├── README.md                                      - Package README
├── README.modular.md                              - Architecture documentation
├── MIGRATION.modular.md                           - Migration guide
└── MODULAR_ARCHITECTURE.md                        - This file
```

**Total: 1,495 lines → 1,495 lines** (same functionality, better organization)

## Component Breakdown

### 1. JsonEvalRs.Shared.cs
**Purpose**: Platform-agnostic shared types

**Contents**:
- `ValidationError` - Validation error details
- `ValidationResult` - Validation result with errors list
- `CacheStats` - Cache performance statistics
- `JsonEvalException` - Custom exception type

**Dependencies**: Newtonsoft.Json (for JSON serialization attributes)

### 2. JsonEvalRs.Native.Common.cs
**Purpose**: Common P/Invoke declarations (no string marshalling differences)

**Contents**:
- `Native` partial class declaration
- `FFIResult` struct
- Platform-independent FFI functions:
  - Version info
  - Resource management (free functions)
  - Schema operations (no string params)
  - Cache operations
  - Layout resolution

**Dependencies**: System.Runtime.InteropServices

### 3. JsonEvalRs.Native.NetCore.cs
**Purpose**: .NET Core/.NET 5+ specific FFI declarations

**Active When**: `NETCOREAPP || NET5_0_OR_GREATER`

**Features**:
- UTF-8 string marshalling (`[MarshalAs(UnmanagedType.LPUTF8Str)]`)
- DLL import resolver for .NET 5+
- Platform-specific library path detection
- More efficient string handling

**Contains**: 15 core FFI methods + 10 subform FFI methods

### 4. JsonEvalRs.Native.NetStandard.cs
**Purpose**: .NET Standard 2.0/2.1 specific FFI declarations

**Active When**: NOT (`NETCOREAPP || NET5_0_OR_GREATER`)

**Features**:
- Byte array marshalling for compatibility
- Helper methods: `ToUTF8Bytes()` and `PtrToStringUTF8()`
- Compatible with older .NET Framework versions

**Contains**: Same 25 FFI methods as NetCore but with byte array signatures

### 5. JsonEvalRs.Main.cs
**Purpose**: Core JSONEval class implementation

**Partial Class**: `public partial class JSONEval : IDisposable`

**Public Methods** (22 total):
- Constructor (2 overloads: JSON string, MessagePack bytes)
- `Evaluate()` - Evaluate schema with data
- `Validate()` - Validate data against rules
- `EvaluateDependents()` - Process dependent fields
- `GetEvaluatedSchema()` - Get evaluated schema
- `GetEvaluatedSchemaMsgpack()` - Get schema as MessagePack
- `GetSchemaValue()` - Get schema values
- `GetEvaluatedSchemaWithoutParams()` - Get schema without $params
- `GetEvaluatedSchemaByPath()` - Get value by path
- `ReloadSchema()` - Reload with new schema
- `GetCacheStats()` - Get cache statistics
- `ClearCache()` - Clear evaluation cache
- `GetCacheLength()` - Get cache entry count
- `ResolveLayout()` - Resolve layout references
- `CompileAndRunLogic()` - Run JSON logic
- `ValidatePaths()` - Validate specific paths
- `Dispose()` - Release native resources
- Static: `Version` property

**Private Methods**: Result processing helpers (4 overloads)

### 6. JsonEvalRs.Subforms.cs
**Purpose**: Subform-specific operations

**Partial Class**: `public partial class JSONEval`

**Public Methods** (10 total):
- `EvaluateSubform()` - Evaluate subform with data
- `ValidateSubform()` - Validate subform data
- `EvaluateDependentsSubform()` - Process subform dependents
- `ResolveLayoutSubform()` - Resolve subform layout
- `GetEvaluatedSchemaSubform()` - Get subform schema
- `GetSchemaValueSubform()` - Get subform values
- `GetEvaluatedSchemaWithoutParamsSubform()` - Get subform schema without $params
- `GetEvaluatedSchemaByPathSubform()` - Get subform value by path
- `GetSubformPaths()` - List available subforms
- `HasSubform()` - Check subform existence

## Design Principles

### 1. Separation of Concerns
Each file has a single, well-defined responsibility:
- **Shared**: Common types used across all files
- **Native.Common**: Platform-independent FFI
- **Native.NetCore/NetStandard**: Platform-specific FFI
- **Main**: Core functionality
- **Subforms**: Subform-specific functionality

### 2. Partial Classes
Uses C# partial classes to split implementation:
```csharp
// JsonEvalRs.Main.cs
public partial class JSONEval : IDisposable { ... }

// JsonEvalRs.Subforms.cs  
public partial class JSONEval { ... }

// Compiled into single class
```

### 3. Conditional Compilation
Platform-specific code uses preprocessor directives:
```csharp
#if NETCOREAPP || NET5_0_OR_GREATER
    // .NET Core/.NET 5+ code
#else
    // .NET Standard code
#endif
```

### 4. Zero-Copy Architecture
Maintains original performance characteristics:
- Direct pointer access to Rust memory
- Single-copy data transfer
- Efficient UTF-8 handling

## Build System Integration

### MSBuild Configuration

```xml
<ItemGroup>
  <!-- All modular files included -->
  <Compile Include="JsonEvalRs.Shared.cs" />
  <Compile Include="JsonEvalRs.Native.Common.cs" />
  <Compile Include="JsonEvalRs.Native.NetCore.cs" />
  <Compile Include="JsonEvalRs.Native.NetStandard.cs" />
  <Compile Include="JsonEvalRs.Main.cs" />
  <Compile Include="JsonEvalRs.Subforms.cs" />
  
  <!-- Old file excluded -->
  <Compile Remove="JsonEvalRs.cs" Condition="Exists('JsonEvalRs.cs')" />
</ItemGroup>
```

### Multi-Targeting

Supports 4 target frameworks:
- netstandard2.0
- netstandard2.1
- net6.0
- net8.0

Each target automatically selects appropriate Native implementation.

## Benefits

### For Maintainers
✅ **Easier Navigation**: Find code quickly by file purpose  
✅ **Reduced Complexity**: No deeply nested #if blocks  
✅ **Clear Dependencies**: Platform requirements are explicit  
✅ **Modular Testing**: Test components independently  
✅ **Better Git Diffs**: Changes isolated to relevant files  

### For Users
✅ **Zero Breaking Changes**: Existing code works unchanged  
✅ **Same Performance**: No overhead from modularization  
✅ **Better Documentation**: Architecture is self-documenting  
✅ **Future-Proof**: Easy to add new platforms  

## Comparison

### Before (Monolithic)
```
JsonEvalRs.cs
├── Lines: 1,495
├── #if blocks: 12+ nested
├── Classes: 5 (mixed together)
└── Maintainability: Low
```

### After (Modular)
```
6 Files
├── Lines: 1,495 (same)
├── #if blocks: Isolated to platform files
├── Classes: Same 5 (clearly separated)
└── Maintainability: High
```

## Testing

All existing tests pass without modification:

```bash
cd bindings/csharp-example
dotnet run
```

Expected output: All benchmarks and tests succeed.

## Future Enhancements

The modular architecture enables:

1. **Platform Expansion**: Add new platforms easily
   - Example: Add `JsonEvalRs.Native.Unity.cs` for Unity3D

2. **Feature Modules**: Separate advanced features
   - Example: `JsonEvalRs.Advanced.cs` for experimental APIs

3. **Testing Modules**: Add test-specific extensions
   - Example: `JsonEvalRs.Testing.cs` with test helpers

4. **Documentation**: Auto-generate from structured files
   - Each file can have focused XML documentation

## Version History

- **v0.0.10+**: Modular architecture
- **v0.0.1-0.0.8**: Monolithic architecture

## References

- **README.modular.md**: Architecture deep-dive
- **MIGRATION.modular.md**: Migration guide for developers
- **JsonEvalRs.csproj**: Build configuration

---

**Architecture Status**: ✅ Production Ready  
**Breaking Changes**: ❌ None  
**Backward Compatibility**: ✅ 100%  
**Code Coverage**: ✅ All original functionality maintained
