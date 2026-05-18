# DONE

## Summary

- Moved C# benchmark example from old top-level C# example folder to `bindings/csharp/examples/benchmark/` using `git mv`.
- Added `bindings/csharp/examples/README.md` listing available C# examples.
- Updated benchmark project references and docs for new path.
- Updated web benchmark README link to new C# benchmark location.
- Added short role headers to C# binding files:
  - `JsonEvalRs.Main.cs`
  - `JsonEvalRs.Shared.cs`
  - `JsonEvalRs.ParsedCache.cs`
  - `JsonEvalRs.Native.ParsedCache.cs`
  - `JsonEvalRs.Subforms.cs`
  - `JsonEvalRs.Native.Common.cs`
  - `JsonEvalRs.Native.NetCore.cs`
  - `JsonEvalRs.Native.NetStandard.cs`
  - `JsonEvalRs.ReturnFormat.cs`
  - `JsonEvalRs.DependencyInjection.cs`
- Excluded `bindings/csharp/examples/**` from `bindings/csharp/JsonEvalRs.csproj` compile/content globs so moved example source does not get compiled into library package.

## Validation

```bash
dotnet build bindings/csharp/JsonEvalRs.csproj
```

Result: success. `1 projects, 0 errors, 0 warnings`.

```bash
dotnet build bindings/csharp/examples/benchmark/JsonEvalBenchmark.csproj
```

Result: success. `2 projects, 0 errors, 0 warnings`.

Stale old example path scan: no matches. `rg` exits `1` because ripgrep reports no matches.

```bash
git diff --check
```

Result: success. No whitespace errors.

## Commit

Final commit SHA is reported in final response because embedding a commit's own full SHA changes the commit hash.

Message: `refactor: organize csharp binding examples and file roles`

## Integration risks

- Benchmark build now tolerates missing repository sample/native files by copying them only when present. Runtime benchmark still requires scenario files and native library.
- `bindings/csharp/JsonEvalRs.csproj` now excludes nested `examples/**`; needed because SDK-style projects include nested `.cs` files by default after moving example under binding directory.
- No public C# API signatures changed.
