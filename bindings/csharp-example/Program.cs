using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using JsonEvalRs;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace JsonEvalBenchmark
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("🚀 JSON Eval RS - C# Benchmark");
            Console.WriteLine($"📦 Library Version: {JSONEval.Version}");
            Console.WriteLine();

            string scenarioFilter = args.Length > 0 ? args[0] : "zcc";
            
            Console.WriteLine($"📋 Running scenario: '{scenarioFilter}'");
            Console.WriteLine();

            try
            {
                RunBenchmark(scenarioFilter);
            }
            catch (Exception ex)
            {
                Console.ForegroundColor = ConsoleColor.Red;
                Console.WriteLine($"❌ Error: {ex.Message}");
                Console.WriteLine($"Stack trace: {ex.StackTrace}");
                Console.ResetColor();
                Environment.Exit(1);
            }
        }

        static void RunBenchmark(string scenario)
        {
            Console.WriteLine("==============================");
            Console.WriteLine($"Scenario: {scenario}");
            Console.WriteLine($"Schema: samples/{scenario}.json");
            Console.WriteLine($"Data: samples/{scenario}-data.json");
            Console.WriteLine();

            // Load files
            Console.WriteLine("Loading files...");
            string schemaPath = $"samples/{scenario}.json";
            string dataPath = $"samples/{scenario}-data.json";
            string comparePath = $"samples/{scenario}-evaluated-compare.json";

            if (!File.Exists(schemaPath))
            {
                throw new FileNotFoundException($"Schema file not found: {schemaPath}");
            }
            if (!File.Exists(dataPath))
            {
                throw new FileNotFoundException($"Data file not found: {dataPath}");
            }

            string schemaJson = File.ReadAllText(schemaPath);
            string dataJson = File.ReadAllText(dataPath);
            
            JObject? compareData = null;
            if (File.Exists(comparePath))
            {
                compareData = JObject.Parse(File.ReadAllText(comparePath));
            }

            Console.WriteLine("Running evaluation...");
            Console.WriteLine();

            var totalStopwatch = Stopwatch.StartNew();

            // Benchmark 1: Schema parsing & compilation (constructor)
            var parsingStopwatch = Stopwatch.StartNew();
            JSONEval eval;
            try
            {
                eval = new JSONEval(schemaJson, context: "{}", data: dataJson);
            }
            catch (Exception ex)
            {
                throw new JsonEvalException($"Failed to create JSONEval instance: {ex.Message}", ex);
            }
            parsingStopwatch.Stop();
            Console.WriteLine($"  Schema parsing & compilation: {parsingStopwatch.Elapsed.TotalMilliseconds:F6}ms");

            // Benchmark 2: Evaluation
            var evalStopwatch = Stopwatch.StartNew();
            try
            {
                eval.Evaluate(dataJson);
            }
            catch (Exception ex)
            {
                throw new JsonEvalException($"Evaluation failed: {ex.Message}", ex);
            }
            evalStopwatch.Stop();
            Console.WriteLine($"  Evaluation: {evalStopwatch.Elapsed.TotalMilliseconds:F6}ms");

            totalStopwatch.Stop();
            Console.ForegroundColor = ConsoleColor.Cyan;
            Console.WriteLine($"⏱️  Total execution time: {totalStopwatch.Elapsed.TotalMilliseconds:F6}ms");
            Console.ResetColor();
            Console.WriteLine();
            
            // Get the result for file output (not included in performance timing)
            JObject result = eval.GetEvaluatedSchema(skipLayout: true);

            // Save results
            string outputDir = "samples";
            Directory.CreateDirectory(outputDir);

            string evaluatedPath = $"{outputDir}/{scenario}-evaluated-schema.json";
            string parsedPath = $"{outputDir}/{scenario}-parsed-schema.json";
            string sortedPath = $"{outputDir}/{scenario}-sorted-evaluations.json";

            // The result from Evaluate already contains the full evaluated schema
            // No need for additional FFI calls to GetEvaluatedSchema() and GetSchemaValue()
            File.WriteAllText(evaluatedPath, result.ToString(Formatting.Indented));
            
            // Extract schema value from result (avoiding extra FFI call)
            var schemaValue = result.SelectToken("$.$params") ?? new JObject();
            File.WriteAllText(parsedPath, schemaValue.ToString(Formatting.Indented));

            // Save the evaluation result
            File.WriteAllText(sortedPath, result.ToString(Formatting.Indented));

            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine("✅ Results saved:");
            Console.ResetColor();
            Console.WriteLine($"  - {evaluatedPath}");
            Console.WriteLine($"  - {parsedPath}");
            Console.WriteLine($"  - {sortedPath}");
            Console.WriteLine();

            // Compare results if comparison file exists
            if (compareData != null)
            {
                CompareResults(result.SelectToken("$.$params.others")?.ToObject<JObject>() ?? new JObject(), compareData, scenario);
            }

            // Performance comparison with Rust
            Console.WriteLine("==================================================");
            Console.ForegroundColor = ConsoleColor.Yellow;
            Console.WriteLine("📊 Performance Comparison");
            Console.ResetColor();
            Console.WriteLine("==================================================");
            Console.WriteLine($"C# Total Time:     {totalStopwatch.Elapsed.TotalMilliseconds:F3}ms");
            Console.WriteLine($"  - Parsing:       {parsingStopwatch.Elapsed.TotalMilliseconds:F3}ms");
            Console.WriteLine($"  - Evaluation:    {evalStopwatch.Elapsed.TotalMilliseconds:F3}ms");
            Console.WriteLine();
            Console.WriteLine("Rust Baseline:     ~518ms (from cargo run)");
            Console.WriteLine("  - Parsing:       ~201ms");
            Console.WriteLine("  - Evaluation:    ~316ms");
            Console.WriteLine();

            double overhead = (totalStopwatch.Elapsed.TotalMilliseconds / 518.0 - 1.0) * 100;
            if (overhead > 0)
            {
                Console.ForegroundColor = ConsoleColor.Yellow;
                Console.WriteLine($"FFI Overhead:      +{overhead:F1}%");
            }
            else
            {
                Console.ForegroundColor = ConsoleColor.Green;
                Console.WriteLine($"Performance:       {overhead:F1}% (faster!)");
            }
            Console.ResetColor();
            Console.WriteLine();

            // Memory info
            Console.WriteLine("💾 Memory Usage:");
            Console.WriteLine($"  Working Set:     {Process.GetCurrentProcess().WorkingSet64 / 1024 / 1024:F2} MB");
            Console.WriteLine($"  Private Memory:  {Process.GetCurrentProcess().PrivateMemorySize64 / 1024 / 1024:F2} MB");
            Console.WriteLine();

            // Dispose
            eval.Dispose();

            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine("✅ Benchmark completed successfully!");
            Console.ResetColor();
        }

        static void CompareResults(JObject actual, JObject expected, string scenario)
        {
            var differences = FindDifferences(actual, expected, "$");
            
            if (differences.Count == 0)
            {
                Console.ForegroundColor = ConsoleColor.Green;
                Console.WriteLine($"✅ Comparison: Results match `samples/{scenario}-evaluated-compare.json`");
                Console.ResetColor();
            }
            else
            {
                Console.ForegroundColor = ConsoleColor.Yellow;
                Console.WriteLine($"⚠️  Comparison: Results differ from `samples/{scenario}-evaluated-compare.json` ({differences.Count} difference(s)):");
                Console.ResetColor();
                foreach (var diff in differences)
                {
                    Console.WriteLine($"  - {diff}");
                }
            }
            Console.WriteLine();
        }

        static List<string> FindDifferences(JToken actual, JToken expected, string path)
        {
            var differences = new List<string>();

            if (actual.Type != expected.Type)
            {
                differences.Add($"{path} type differs: actual={actual.Type} expected={expected.Type}");
                return differences;
            }

            if (actual is JObject actualObj && expected is JObject expectedObj)
            {
                // Check all properties in expected
                foreach (var prop in expectedObj.Properties())
                {
                    var actualProp = actualObj.Property(prop.Name);
                    if (actualProp == null)
                    {
                        differences.Add($"{path}.{prop.Name} missing in actual");
                    }
                    else
                    {
                        differences.AddRange(FindDifferences(actualProp.Value, prop.Value, $"{path}.{prop.Name}"));
                    }
                }

                // Check for extra properties in actual
                foreach (var prop in actualObj.Properties())
                {
                    if (expectedObj.Property(prop.Name) == null)
                    {
                        differences.Add($"{path}.{prop.Name} extra in actual");
                    }
                }
            }
            else if (actual is JArray actualArray && expected is JArray expectedArray)
            {
                if (actualArray.Count != expectedArray.Count)
                {
                    differences.Add($"{path} array length differs: actual={actualArray.Count} expected={expectedArray.Count}");
                }
                else
                {
                    for (int i = 0; i < actualArray.Count; i++)
                    {
                        differences.AddRange(FindDifferences(actualArray[i], expectedArray[i], $"{path}[{i}]"));
                    }
                }
            }
            else if (actual is JValue actualValue && expected is JValue expectedValue)
            {
                if (!JToken.DeepEquals(actualValue, expectedValue))
                {
                    differences.Add($"{path} differs: actual={actualValue} expected={expectedValue}");
                }
            }

            return differences;
        }
    }
}
