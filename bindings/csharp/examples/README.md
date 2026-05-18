# C# Examples

Available examples for the JsonEvalRs C# binding.

## Benchmark

Path: [`benchmark/`](benchmark/)

Benchmarks the local C# binding against the Rust baseline using repository sample schemas.

```bash
cd bindings/csharp/examples/benchmark
dotnet build --configuration Release
dotnet run --configuration Release
```

See [`benchmark/README.md`](benchmark/README.md) for requirements, options, and troubleshooting.
