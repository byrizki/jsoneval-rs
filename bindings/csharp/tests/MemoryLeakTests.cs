using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using JsonEvalRs;
using Newtonsoft.Json.Linq;
using Xunit;
using Xunit.Abstractions;

namespace JsonEvalRs.Tests;

public class MemoryLeakTests
{
    private readonly ITestOutputHelper _output;

    public MemoryLeakTests(ITestOutputHelper output)
    {
        _output = output;
    }

    private static long GetProcessRssBytes()
    {
        try
        {
            if (File.Exists("/proc/self/status"))
            {
                foreach (var line in File.ReadAllLines("/proc/self/status"))
                {
                    if (line.StartsWith("VmRSS:"))
                    {
                        var parts = line.Split(new[] { ' ', '\t' }, StringSplitOptions.RemoveEmptyEntries);
                        if (parts.Length >= 2 && long.TryParse(parts[1], out long kb))
                        {
                            return kb * 1024;
                        }
                    }
                }
            }
        }
        catch { }

        return Process.GetCurrentProcess().WorkingSet64;
    }

    private static void ForceGc()
    {
        GC.Collect();
        GC.WaitForPendingFinalizers();
        GC.Collect();
    }

    [Fact]
    public void Test_RepeatedInstanceCreationAndDisposal_FromCache_NoMemoryLeak()
    {
        const string schemaKey = "mem_test_cache_schema";
        const string schemaJson = """
        {
          "type": "object",
          "properties": {
            "a": { "type": "number" },
            "b": { "type": "number" },
            "total": {
              "type": "number",
              "$evaluation": { "logic": { "+": [{ "var": "a" }, { "var": "b" }] } }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        for (int i = 0; i < 500; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate("{\"a\": 1, \"b\": 2}");
            var _ = eval.GetEvaluatedSchema();
        }

        ForceGc();
        long baselineRss = GetProcessRssBytes();
        long baselineGc = GC.GetTotalMemory(true);
        _output.WriteLine($"[Baseline] RSS: {baselineRss / 1024 / 1024} MB, Managed GC: {baselineGc / 1024} KB");

        const int iterations = 10000;
        for (int i = 0; i < iterations; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate($"{{\"a\": {i}, \"b\": {i * 2}}}");
            var schema = eval.GetEvaluatedSchema();
            int total = schema.SelectToken("properties.total")?.Value<int>() ?? 0;
            Assert.Equal(i * 3, total);

            if ((i + 1) % 2500 == 0)
            {
                ForceGc();
                long currentRss = GetProcessRssBytes();
                long currentGc = GC.GetTotalMemory(true);
                _output.WriteLine($"[Iter {i + 1}] RSS: {currentRss / 1024 / 1024} MB, Managed GC: {currentGc / 1024} KB");
            }
        }

        ForceGc();
        long finalRss = GetProcessRssBytes();
        long finalGc = GC.GetTotalMemory(true);
        _output.WriteLine($"[Final] RSS: {finalRss / 1024 / 1024} MB, Managed GC: {finalGc / 1024} KB");

        long rssDeltaMb = (finalRss - baselineRss) / 1024 / 1024;
        _output.WriteLine($"[Delta] RSS delta: {rssDeltaMb} MB");
        Assert.True(rssDeltaMb < 25, $"Possible memory leak! RSS increased by {rssDeltaMb} MB over {iterations} iterations");
    }

    [Fact]
    public void Test_TableCalculation_HeavyIterations_NoMemoryLeak()
    {
        const string schemaKey = "mem_test_table_calc_schema";
        const string schemaJson = """
        {
          "$params": {
            "references": {
              "CALC_TABLE": {
                "$table": [
                  {
                    "$repeat": [
                      0,
                      8,
                      {
                        "STEP": { "$evaluation": { "$ref": "$iteration" } },
                        "VAL": {
                          "$evaluation": {
                            "*": [
                              { "$ref": "$STEP" },
                              { "var": "multiplier" }
                            ]
                          }
                        }
                      }
                    ]
                  }
                ]
              }
            }
          },
          "type": "object",
          "properties": {
            "multiplier": { "type": "number" },
            "result": {
              "type": "number",
              "$evaluation": {
                "logic": {
                  "VALUEAT": [
                    { "$ref": "#/$params/references/CALC_TABLE" },
                    8,
                    "VAL"
                  ]
                }
              }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        // Warm up
        for (int i = 0; i < 200; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate("{\"multiplier\": 2}");
            var _ = eval.GetEvaluatedSchema();
        }

        ForceGc();
        long baselineRss = GetProcessRssBytes();
        _output.WriteLine($"[Table Baseline] RSS: {baselineRss / 1024 / 1024} MB");

        const int iterations = 5000;
        for (int i = 0; i < iterations; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate($"{{\"multiplier\": {i}}}");
            var schema = eval.GetEvaluatedSchema();
            Assert.Equal(8 * i, schema.SelectToken("properties.result")?.Value<int>());
        }

        ForceGc();
        long finalRss = GetProcessRssBytes();
        _output.WriteLine($"[Table Final] RSS: {finalRss / 1024 / 1024} MB");

        long rssDeltaMb = (finalRss - baselineRss) / 1024 / 1024;
        _output.WriteLine($"[Table Delta] RSS delta: {rssDeltaMb} MB");
        Assert.True(rssDeltaMb < 25, $"Possible memory leak in table calc! RSS increased by {rssDeltaMb} MB");
    }

    [Fact]
    public void Test_Subforms_HeavyIterations_NoMemoryLeak()
    {
        const string schemaKey = "mem_test_subform_schema";
        const string schemaJson = """
        {
          "type": "object",
          "properties": {
            "dept": { "type": "string" },
            "staff": {
              "type": "array",
              "items": {
                "properties": {
                  "name": {
                    "type": "string",
                    "rules": {
                      "required": { "value": true, "message": "Name is required" }
                    }
                  },
                  "hours": { "type": "number" },
                  "rate": { "type": "number" },
                  "pay": {
                    "type": "number",
                    "$evaluation": { "logic": { "*": [{ "var": "hours" }, { "var": "rate" }] } }
                  }
                }
              }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        for (int i = 0; i < 200; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate("{\"dept\": \"IT\", \"staff\": [{\"name\": \"Alice\", \"hours\": 40, \"rate\": 25}]}");
            var _ = eval.Validate("{\"dept\": \"IT\", \"staff\": [{\"name\": \"Alice\", \"hours\": 40, \"rate\": 25}]}", null, false, true);
        }

        ForceGc();
        long baselineRss = GetProcessRssBytes();
        _output.WriteLine($"[Subform Baseline] RSS: {baselineRss / 1024 / 1024} MB");

        const int iterations = 5000;
        for (int i = 0; i < iterations; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            var data = $"{{\"dept\": \"D_{i}\", \"staff\": [{{\"name\": \"N_{i}\", \"hours\": {i}, \"rate\": 10}}]}}";
            eval.Evaluate(data);
            var val = eval.Validate(data, null, false, true);
            Assert.False(val.HasError);
        }

        ForceGc();
        long finalRss = GetProcessRssBytes();
        _output.WriteLine($"[Subform Final] RSS: {finalRss / 1024 / 1024} MB");

        long rssDeltaMb = (finalRss - baselineRss) / 1024 / 1024;
        _output.WriteLine($"[Subform Delta] RSS delta: {rssDeltaMb} MB");
        Assert.True(rssDeltaMb < 25, $"Possible memory leak in subforms! RSS increased by {rssDeltaMb} MB");
    }

    [Fact]
    public void Test_MessagePack_ZeroCopy_NoMemoryLeak()
    {
        const string schemaKey = "mem_test_msgpack_schema";
        const string schemaJson = """
        {
          "type": "object",
          "properties": {
            "val": { "type": "number" },
            "doubled": {
              "type": "number",
              "$evaluation": { "logic": { "*": [{ "var": "val" }, 2] } }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        for (int i = 0; i < 200; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate("{\"val\": 10}");
            var _ = eval.GetEvaluatedSchemaMsgpack();
            var __ = eval.GetEvaluatedSchemaResolvedMsgpack();
        }

        ForceGc();
        long baselineRss = GetProcessRssBytes();
        _output.WriteLine($"[MsgPack Baseline] RSS: {baselineRss / 1024 / 1024} MB");

        const int iterations = 5000;
        for (int i = 0; i < iterations; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate($"{{\"val\": {i}}}");
            byte[] bytes1 = eval.GetEvaluatedSchemaMsgpack();
            byte[] bytes2 = eval.GetEvaluatedSchemaResolvedMsgpack();
            Assert.NotEmpty(bytes1);
            Assert.NotEmpty(bytes2);
        }

        ForceGc();
        long finalRss = GetProcessRssBytes();
        _output.WriteLine($"[MsgPack Final] RSS: {finalRss / 1024 / 1024} MB");

        long rssDeltaMb = (finalRss - baselineRss) / 1024 / 1024;
        _output.WriteLine($"[MsgPack Delta] RSS delta: {rssDeltaMb} MB");
        Assert.True(rssDeltaMb < 25, $"Possible memory leak in MessagePack! RSS increased by {rssDeltaMb} MB");
    }

    [Fact]
    public void Test_FieldDependents_HeavyIterations_NoMemoryLeak()
    {
        const string schemaKey = "mem_test_dependents_schema";
        const string schemaJson = """
        {
          "type": "object",
          "properties": {
            "tier": {
              "type": "string",
              "dependents": [
                {
                  "$ref": "#/properties/rate",
                  "value": {
                    "$evaluation": {
                      "if": [
                        { "==": [{ "$ref": "$value" }, "PREMIUM"] },
                        50,
                        10
                      ]
                    }
                  }
                }
              ]
            },
            "rate": { "type": "number" }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        for (int i = 0; i < 200; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate("{\"tier\": \"NONE\", \"rate\": 0}");
            var _ = eval.EvaluateDependents(new[] { "#/properties/tier" }, "{\"tier\": \"PREMIUM\", \"rate\": 0}");
        }

        ForceGc();
        long baselineRss = GetProcessRssBytes();
        _output.WriteLine($"[Dependents Baseline] RSS: {baselineRss / 1024 / 1024} MB");

        const int iterations = 5000;
        for (int i = 0; i < iterations; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate("{\"tier\": \"NONE\", \"rate\": 0}");
            var changes = eval.EvaluateDependents(
                new[] { "#/properties/tier" },
                $"{{\"tier\": \"{(i % 2 == 0 ? "PREMIUM" : "STANDARD")}\", \"rate\": 0}}"
            );
            Assert.NotEmpty(changes);
        }

        ForceGc();
        long finalRss = GetProcessRssBytes();
        _output.WriteLine($"[Dependents Final] RSS: {finalRss / 1024 / 1024} MB");

        long rssDeltaMb = (finalRss - baselineRss) / 1024 / 1024;
        _output.WriteLine($"[Dependents Delta] RSS delta: {rssDeltaMb} MB");
        Assert.True(rssDeltaMb < 25, $"Possible memory leak in field dependents! RSS increased by {rssDeltaMb} MB");
    }

    [Fact]
    public void Test_WithoutDispose_FinalizerCleansUpNativeMemory()
    {
        const string schemaKey = "mem_test_finalizer_schema";
        const string schemaJson = """
        {
          "type": "object",
          "properties": {
            "x": { "type": "number" }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        ForceGc();
        long baselineRss = GetProcessRssBytes();
        _output.WriteLine($"[Finalizer Baseline] RSS: {baselineRss / 1024 / 1024} MB");

        // Intentionally create instances WITHOUT calling Dispose(), relying on finalizer
        void CreateInstancesWithoutDispose(int count)
        {
            for (int i = 0; i < count; i++)
            {
                var eval = JSONEval.FromCache(schemaKey);
                eval.Evaluate($"{{\"x\": {i}}}");
                // Do not call eval.Dispose()
            }
        }

        CreateInstancesWithoutDispose(5000);

        ForceGc();
        long finalRss = GetProcessRssBytes();
        _output.WriteLine($"[Finalizer Final] RSS: {finalRss / 1024 / 1024} MB");

        long rssDeltaMb = (finalRss - baselineRss) / 1024 / 1024;
        _output.WriteLine($"[Finalizer Delta] RSS delta: {rssDeltaMb} MB");
        Assert.True(rssDeltaMb < 30, $"Finalizer failed to reclaim native memory! RSS increased by {rssDeltaMb} MB");
    }

    [Fact]
    public void Test_RealWorld_ZccSchema_RepeatedEvaluations_MemoryProfiling()
    {
        string schemaPath = "/mnt/development/jsoneval-rs/products/dev/schemas/zcc.json";
        string dataPath = "/mnt/development/jsoneval-rs/products/datas/zcc.json";

        if (!File.Exists(schemaPath) || !File.Exists(dataPath))
        {
            _output.WriteLine("zcc schema or data not found, skipping real-world test");
            return;
        }

        string schemaJson = File.ReadAllText(schemaPath);
        string dataJson = File.ReadAllText(dataPath);

        const string schemaKey = "mem_test_realworld_zcc";
        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        // Warm up
        for (int i = 0; i < 5; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate(dataJson);
            var _ = eval.GetEvaluatedSchema();
        }

        ForceGc();
        long baselineRss = GetProcessRssBytes();
        long baselineGc = GC.GetTotalMemory(true);
        _output.WriteLine($"[ZCC Baseline] RSS: {baselineRss / 1024 / 1024} MB, Managed GC: {baselineGc / 1024} KB");

        const int iterations = 100;
        for (int i = 0; i < iterations; i++)
        {
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate(dataJson);
            var schema = eval.GetEvaluatedSchema();
            Assert.NotNull(schema);

            if ((i + 1) % 25 == 0)
            {
                ForceGc();
                long currentRss = GetProcessRssBytes();
                long currentGc = GC.GetTotalMemory(true);
                _output.WriteLine($"[ZCC GetEvaluatedSchema Iter {i + 1}] RSS: {currentRss / 1024 / 1024} MB, Managed GC: {currentGc / 1024} KB");
            }
        }

        ForceGc();
        long finalRss = GetProcessRssBytes();
        long finalGc = GC.GetTotalMemory(true);
        _output.WriteLine($"[ZCC GetEvaluatedSchema Final] RSS: {finalRss / 1024 / 1024} MB, Managed GC: {finalGc / 1024} KB");

        long rssDeltaMb = (finalRss - baselineRss) / 1024 / 1024;
        _output.WriteLine($"[ZCC GetEvaluatedSchema Delta] RSS delta: {rssDeltaMb} MB");
        // 64 MB delta is the one-time managed GC heap commit for the 180 MB JObject DOM tree
        Assert.True(rssDeltaMb < 100, $"RSS increased by {rssDeltaMb} MB over {iterations} iterations");
    }
}

