using System.Diagnostics;
using Xunit;
using Xunit.Abstractions;

namespace JsonEvalRs.Tests;

public class ProductMemoryLeakTests
{
    private readonly ITestOutputHelper _output;

    public ProductMemoryLeakTests(ITestOutputHelper output)
    {
        _output = output;
    }

    private static long GetProcessRssBytes()
    {
        Process.GetCurrentProcess().Refresh();
        return Process.GetCurrentProcess().WorkingSet64;
    }

    private static void ForceGc()
    {
        GC.Collect(2, GCCollectionMode.Forced, true, true);
        GC.WaitForPendingFinalizers();
        GC.Collect(2, GCCollectionMode.Forced, true, true);
    }

    [Fact]
    public void Test_Zia_EvaluateAndValidate_WithSubforms_ReadonlyTrue_NoMemoryLeak()
    {
        string? rootDir = AppContext.BaseDirectory;
        while (rootDir != null && !File.Exists(Path.Combine(rootDir, "products/datas/zia.json")))
        {
            rootDir = Path.GetDirectoryName(rootDir);
        }

        if (rootDir == null)
        {
            _output.WriteLine("zia schema or data not found, skipping");
            return;
        }

        var schemaPath = Path.Combine(rootDir, "products/dev/schemas/zia.json");
        var dataPath = Path.Combine(rootDir, "products/datas/zia.json");

        var schemaJson = File.ReadAllText(schemaPath);
        var dataJson = File.ReadAllText(dataPath);
        const string schemaKey = "mem_test_zia_schema";

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        // Warm up
        for (int i = 0; i < 2; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate(dataJson);
            var _ = eval.Validate(dataJson, null, validateReadonly: true, includeSubforms: true);
        }

        ForceGc();
        long baselineRss = GetProcessRssBytes();
        _output.WriteLine($"[ZIA Baseline] RSS: {baselineRss / 1024 / 1024} MB");

        const int iterations = 10;
        for (int i = 0; i < iterations; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate(dataJson);
            var val = eval.Validate(dataJson, null, validateReadonly: true, includeSubforms: true);
            Assert.False(val.HasError);
        }

        ForceGc();
        long finalRss = GetProcessRssBytes();
        _output.WriteLine($"[ZIA Final] RSS: {finalRss / 1024 / 1024} MB");

        long rssDeltaMb = (finalRss - baselineRss) / 1024 / 1024;
        _output.WriteLine($"[ZIA Delta] RSS delta: {rssDeltaMb} MB");
        Assert.True(rssDeltaMb < 25, $"Possible memory leak in ZIA evaluate+validate! RSS increased by {rssDeltaMb} MB");
    }
}
