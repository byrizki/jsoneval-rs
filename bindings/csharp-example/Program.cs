using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.RegularExpressions;
using JsonEvalRs;

namespace JsonEvalBenchmark
{
    class Program
    {
        private static string? _projectRoot;
        
        static void Main(string[] args)
        {
            Console.WriteLine("🚀 JSON Eval RS - Benchmark Suite");
            Console.WriteLine();

            string scenario = args.Length > 0 ? args[0] : "zcc";
            
            Console.WriteLine($"📋 Scenario: '{scenario}'");
            Console.WriteLine();

            try
            {
                // Find project root
                _projectRoot = FindProjectRoot();
                if (_projectRoot == null)
                {
                    Console.ForegroundColor = ConsoleColor.Red;
                    Console.WriteLine("❌ Error: Could not find project root (Cargo.toml not found)");
                    Console.ResetColor();
                    Environment.Exit(1);
                }
                
                Console.WriteLine($"📁 Project Root: {_projectRoot}");
                Console.WriteLine();

                // Step 1: Build
                if (!BuildRelease())
                {
                    Console.ForegroundColor = ConsoleColor.Red;
                    Console.WriteLine("❌ Build failed. Aborting benchmark.");
                    Console.ResetColor();
                    Environment.Exit(1);
                }

                // Step 2: Run Rust benchmark
                var rustResult = RunRustBenchmark(scenario);

                // Step 3: Run C# benchmark
                var csharpResult = RunCSharpBenchmark(scenario);

                // Step 4: Compare results
                PrintComparisonResults(rustResult, csharpResult);

                Console.ForegroundColor = ConsoleColor.Green;
                Console.WriteLine("✅ Benchmark suite completed successfully!");
                Console.ResetColor();
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

        private static string? FindProjectRoot()
        {
            string currentDir = Directory.GetCurrentDirectory();
            string projectRoot = currentDir;
            
            // Navigate up to find Cargo.toml
            while (!File.Exists(Path.Combine(projectRoot, "Cargo.toml")) && Directory.GetParent(projectRoot) != null)
            {
                projectRoot = Directory.GetParent(projectRoot)!.FullName;
            }

            if (!File.Exists(Path.Combine(projectRoot, "Cargo.toml")))
            {
                return null;
            }

            return projectRoot;
        }

        private static bool BuildRelease()
        {
            Console.WriteLine("==================================================");
            Console.ForegroundColor = ConsoleColor.Cyan;
            Console.WriteLine("🔨 Step 1: Building Release");
            Console.ResetColor();
            Console.WriteLine("==================================================");

            // Build both FFI library and CLI binary together to prevent clearing DLL
            Console.WriteLine("🧠 Building Rust library and CLI binary (--release --features ffi)...");
            if (!RunCommand("cargo", "build --release --features ffi --bins", _projectRoot!))
            {
                return false;
            }

            // Ensure DLL is in the correct location for C# to find it
            Console.WriteLine("📋 Ensuring DLL is accessible...");
            string dllSource = Path.Combine(_projectRoot!, "target", "release", "json_eval_rs.dll");
            string dllDest = Path.Combine(_projectRoot!, "bindings", "csharp-example", "bin", "Release", "net8.0", "json_eval_rs.dll");
            
            if (File.Exists(dllSource))
            {
                File.Copy(dllSource, dllDest, overwrite: true);
                Console.WriteLine($"  Copied DLL to {dllDest}");
            }
            else
            {
                Console.ForegroundColor = ConsoleColor.Yellow;
                Console.WriteLine($"  ⚠️  DLL not found at {dllSource}");
                Console.ResetColor();
            }

            Console.WriteLine("✅ Build completed successfully!");
            Console.WriteLine();
            return true;
        }

        private static bool RunCommand(string fileName, string arguments, string workingDirectory)
        {
            var startInfo = new ProcessStartInfo
            {
                FileName = fileName,
                Arguments = arguments,
                WorkingDirectory = workingDirectory,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true
            };

            using var process = new Process { StartInfo = startInfo };
            process.Start();
            
            string output = process.StandardOutput.ReadToEnd();
            string error = process.StandardError.ReadToEnd();
            process.WaitForExit();

            if (process.ExitCode != 0)
            {
                Console.ForegroundColor = ConsoleColor.Red;
                Console.WriteLine($"❌ Command failed with exit code {process.ExitCode}");
                Console.WriteLine($"Command: {fileName} {arguments}");
                if (!string.IsNullOrEmpty(output))
                {
                    Console.WriteLine($"Output: {output}");
                }
                if (!string.IsNullOrEmpty(error))
                {
                    Console.WriteLine($"Error: {error}");
                }
                Console.ResetColor();
                return false;
            }

            // Show relevant output lines
            if (!string.IsNullOrEmpty(output))
            {
                var lines = output.Split('\n');
                foreach (var line in lines)
                {
                    if (line.Contains("Compiling") || line.Contains("Finished") || line.Contains("error"))
                    {
                        Console.WriteLine($"  {line.Trim()}");
                    }
                }
            }

            return true;
        }

        private class BenchmarkResult
        {
            public bool Success { get; set; }
            public double TotalMs { get; set; }
            public double ParsingMs { get; set; }
            public double EvaluationMs { get; set; }
            public double OutputMs { get; set; }
            public string Scenario { get; set; } = string.Empty;
            public JsonObject? EvaluatedSchema { get; set; }
            public int DifferenceCount { get; set; }
        }

        private static BenchmarkResult RunCSharpBenchmark(string scenario)
        {
            Console.WriteLine("==================================================");
            Console.ForegroundColor = ConsoleColor.Cyan;
            Console.WriteLine("🎯 Step 3: Running C# Benchmark");
            Console.ResetColor();
            Console.WriteLine("==================================================");
            Console.WriteLine($"Scenario: {scenario}");
            Console.WriteLine($"Schema: samples/{scenario}.json");
            Console.WriteLine($"Data: samples/{scenario}-data.json");
            Console.WriteLine();

            Console.WriteLine("📂 Loading files...");
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
            
            JsonObject? compareData = null;
            if (File.Exists(comparePath))
            {
                compareData = JsonNode.Parse(File.ReadAllText(comparePath))?.AsObject();
            }

            Console.WriteLine("⏱️ Running evaluation...");
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
            Console.WriteLine($"  🔧 Schema parsing & compilation: {parsingStopwatch.Elapsed.TotalMilliseconds:F3}ms");

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
            Console.WriteLine($"  ⚡ Evaluation: {evalStopwatch.Elapsed.TotalMilliseconds:F3}ms");

            // Benchmark 3: Constructing output
            var outputStopwatch = Stopwatch.StartNew();
            JsonObject result = eval.GetEvaluatedSchema(skipLayout: true);
            var schemaValue = eval.GetSchemaValue();
            outputStopwatch.Stop();
            Console.WriteLine($"  📦 Constructing output: {outputStopwatch.Elapsed.TotalMilliseconds:F3}ms");

            totalStopwatch.Stop();
            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine($"  ✅ Total execution time: {totalStopwatch.Elapsed.TotalMilliseconds:F3}ms");
            Console.ResetColor();
            Console.WriteLine();

            // Save results
            Console.WriteLine("💾 Saving results...");
            string outputDir = "samples";
            Directory.CreateDirectory(outputDir);

            string evaluatedPath = $"{outputDir}/{scenario}-evaluated-schema.json";
            string parsedPath = $"{outputDir}/{scenario}-parsed-schema.json";
            string sortedPath = $"{outputDir}/{scenario}-sorted-evaluations.json";

            // Save the evaluated schema and parsed schema
            var options = new JsonSerializerOptions { WriteIndented = true };
            File.WriteAllText(evaluatedPath, result.ToJsonString(options));
            File.WriteAllText(parsedPath, schemaValue.ToJsonString(options));

            // Save the evaluation result
            File.WriteAllText(sortedPath, result.ToJsonString(options));

            Console.WriteLine("✅ Results saved:");
            Console.WriteLine($"  - {evaluatedPath}");
            Console.WriteLine($"  - {parsedPath}");
            Console.WriteLine($"  - {sortedPath}");
            Console.WriteLine();

            // Compare results if comparison file exists
            int differenceCount = 0;
            List<string> differences = new List<string>();
            if (compareData != null)
            {
                var resultOthers = result["$params"]?["others"]?.AsObject() ?? new JsonObject();
                var compareOthers = compareData["others"]?.AsObject() ?? new JsonObject();
                differences = FindDifferences(resultOthers, compareOthers, "$");
                differenceCount = differences.Count;
                
                if (differenceCount > 0)
                {
                    Console.ForegroundColor = ConsoleColor.Yellow;
                    Console.WriteLine($"⚠️  Comparison: Results differ from baseline ({differenceCount} difference(s)):");
                    Console.ResetColor();
                    foreach (var diff in differences)
                    {
                        Console.WriteLine($"  - {diff}");
                    }
                    Console.WriteLine();
                }
                else
                {
                    Console.ForegroundColor = ConsoleColor.Green;
                    Console.WriteLine("✅ Comparison: Results match baseline");
                    Console.ResetColor();
                    Console.WriteLine();
                }
            }

            // Dispose
            eval.Dispose();

            return new BenchmarkResult
            {
                Success = true,
                TotalMs = totalStopwatch.Elapsed.TotalMilliseconds,
                ParsingMs = parsingStopwatch.Elapsed.TotalMilliseconds,
                EvaluationMs = evalStopwatch.Elapsed.TotalMilliseconds,
                OutputMs = outputStopwatch.Elapsed.TotalMilliseconds,
                Scenario = scenario,
                EvaluatedSchema = result,
                DifferenceCount = differenceCount
            };
        }

        private static BenchmarkResult RunRustBenchmark(string scenario)
        {
            Console.WriteLine("==================================================");
            Console.ForegroundColor = ConsoleColor.Cyan;
            Console.WriteLine("🦀 Step 2: Running Rust Benchmark");
            Console.ResetColor();
            Console.WriteLine("==================================================");

            // Run the pre-built binary directly to avoid cargo rebuilding and clearing the FFI DLL
            string binaryPath = Path.Combine(_projectRoot!, "target", "release", "json-eval-cli.exe");
            
            if (!File.Exists(binaryPath))
            {
                Console.ForegroundColor = ConsoleColor.Yellow;
                Console.WriteLine($"⚠️  Binary not found at {binaryPath}");
                Console.WriteLine("    Building CLI binary...");
                Console.ResetColor();
                
                if (!RunCommand("cargo", "build --release --bin json-eval-cli", _projectRoot!))
                {
                    return new BenchmarkResult { Success = false, Scenario = scenario };
                }
            }

            var startInfo = new ProcessStartInfo
            {
                FileName = binaryPath,
                Arguments = scenario,
                WorkingDirectory = _projectRoot!,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true
            };

            using var process = new Process { StartInfo = startInfo };
            process.Start();
            
            string output = process.StandardOutput.ReadToEnd();
            string error = process.StandardError.ReadToEnd();
            process.WaitForExit();

            if (process.ExitCode != 0)
            {
                Console.ForegroundColor = ConsoleColor.Yellow;
                Console.WriteLine($"⚠️  Rust benchmark failed with exit code {process.ExitCode}");
                if (!string.IsNullOrEmpty(error))
                {
                    Console.WriteLine($"Error: {error}");
                }
                Console.ResetColor();
                Console.WriteLine();
                return new BenchmarkResult { Success = false, Scenario = scenario };
            }

            // Parse output to extract timings
            // Rust Duration debug format: "123.456789ms" or "1.234567s" or "123.456µs" (no space between number and unit)
            var parsingMatch = Regex.Match(output, @"Schema parsing & compilation:\s*([0-9.]+)(s|ms|µs|ns)", RegexOptions.IgnoreCase);
            var evalMatch = Regex.Match(output, @"Evaluation:\s*([0-9.]+)(s|ms|µs|ns)", RegexOptions.IgnoreCase);
            var totalMatch = Regex.Match(output, @"Execution time:\s*([0-9.]+)(s|ms|µs|ns)", RegexOptions.IgnoreCase);

            double parsing = ParseDuration(parsingMatch);
            double evaluation = ParseDuration(evalMatch);
            double total = ParseDuration(totalMatch);

            // Fallback: if total not found, calculate from parts
            if (total == 0 && parsing > 0 && evaluation > 0)
            {
                total = parsing + evaluation;
            }

            if (total == 0)
            {
                Console.ForegroundColor = ConsoleColor.Yellow;
                Console.WriteLine("⚠️  Could not parse Rust benchmark output");
                Console.ResetColor();
                Console.WriteLine();
                return new BenchmarkResult { Success = false, Scenario = scenario };
            }

            Console.WriteLine($"  🔧 Schema parsing & compilation: {parsing:F3}ms");
            Console.WriteLine($"  ⚡ Evaluation: {evaluation:F3}ms");
            Console.ForegroundColor = ConsoleColor.Green;
            Console.WriteLine($"  ✅ Total execution time: {total:F3}ms");
            Console.ResetColor();
            Console.WriteLine();

            return new BenchmarkResult
            {
                Success = true,
                TotalMs = total,
                ParsingMs = parsing,
                EvaluationMs = evaluation,
                Scenario = scenario
            };
        }

        private static void PrintComparisonResults(BenchmarkResult rustResult, BenchmarkResult csharpResult)
        {
            Console.WriteLine("==================================================");
            Console.ForegroundColor = ConsoleColor.Yellow;
            Console.WriteLine("📊 Step 4: Performance Comparison");
            Console.ResetColor();
            Console.WriteLine("==================================================");

            // Rust results
            Console.WriteLine("🦀 Rust (Native):");
            if (rustResult.Success)
            {
                Console.WriteLine($"  Total:       {rustResult.TotalMs:F3}ms");
                Console.WriteLine($"  - Parsing:   {rustResult.ParsingMs:F3}ms");
                Console.WriteLine($"  - Evaluation: {rustResult.EvaluationMs:F3}ms");
            }
            else
            {
                Console.ForegroundColor = ConsoleColor.Yellow;
                Console.WriteLine("  ⚠️  Benchmark unavailable");
                Console.ResetColor();
            }
            Console.WriteLine();

            // C# results
            Console.WriteLine("🎯 C# (FFI):");
            Console.WriteLine($"  Total:       {csharpResult.TotalMs:F3}ms");
            Console.WriteLine($"  - Parsing:   {csharpResult.ParsingMs:F3}ms");
            Console.WriteLine($"  - Evaluation: {csharpResult.EvaluationMs:F3}ms");
            Console.WriteLine($"  - Output:    {csharpResult.OutputMs:F3}ms");
            
            if (csharpResult.DifferenceCount > 0)
            {
                Console.ForegroundColor = ConsoleColor.Yellow;
                Console.WriteLine($"  ⚠️  {csharpResult.DifferenceCount} differences from baseline");
                Console.ResetColor();
            }
            Console.WriteLine();

            // Calculate overhead
            if (rustResult.Success)
            {
                double overhead = (csharpResult.TotalMs / rustResult.TotalMs - 1.0) * 100;
                Console.WriteLine("📈 FFI Overhead:");
                
                if (overhead > 0)
                {
                    Console.ForegroundColor = ConsoleColor.Yellow;
                    Console.WriteLine($"  Total:       +{overhead:F1}%");
                }
                else
                {
                    Console.ForegroundColor = ConsoleColor.Green;
                    Console.WriteLine($"  Total:       {overhead:F1}% (faster!)");
                }

                double parsingOverhead = (csharpResult.ParsingMs / rustResult.ParsingMs - 1.0) * 100;
                double evalOverhead = (csharpResult.EvaluationMs / rustResult.EvaluationMs - 1.0) * 100;
                
                Console.WriteLine($"  - Parsing:   {(parsingOverhead >= 0 ? "+" : "")}{parsingOverhead:F1}%");
                Console.WriteLine($"  - Evaluation: {(evalOverhead >= 0 ? "+" : "")}{evalOverhead:F1}%");
                Console.ResetColor();
                Console.WriteLine();
            }

            // Memory info
            Console.WriteLine("💾 Memory Usage (C#):");
            Console.WriteLine($"  Working Set:  {Process.GetCurrentProcess().WorkingSet64 / 1024 / 1024:F2} MB");
            Console.WriteLine($"  Private:      {Process.GetCurrentProcess().PrivateMemorySize64 / 1024 / 1024:F2} MB");
            Console.WriteLine();
        }

        private static double ParseDuration(Match match)
        {
            if (!match.Success)
                return 0;

            double value = double.Parse(match.Groups[1].Value, System.Globalization.CultureInfo.InvariantCulture);
            string unit = match.Groups[2].Value.ToLower();

            // Convert all to milliseconds
            return unit switch
            {
                "s" => value * 1000.0,
                "ms" => value,
                "µs" or "us" => value / 1000.0,
                "ns" => value / 1000000.0,
                _ => value
            };
        }

        private static List<string> FindDifferences(JsonNode? actual, JsonNode? expected, string path)
        {
            var differences = new List<string>();

            if (actual == null && expected == null)
                return differences;

            if (actual == null)
            {
                differences.Add($"{path} is null in actual but not in expected");
                return differences;
            }

            if (expected == null)
            {
                differences.Add($"{path} is null in expected but not in actual");
                return differences;
            }

            if (actual is JsonObject actualObj && expected is JsonObject expectedObj)
            {
                // Check all properties in expected
                foreach (var prop in expectedObj)
                {
                    if (!actualObj.ContainsKey(prop.Key))
                    {
                        differences.Add($"{path}.{prop.Key} missing in actual");
                    }
                    else
                    {
                        differences.AddRange(FindDifferences(actualObj[prop.Key], prop.Value, $"{path}.{prop.Key}"));
                    }
                }

                // Check for extra properties in actual
                foreach (var prop in actualObj)
                {
                    if (!expectedObj.ContainsKey(prop.Key))
                    {
                        differences.Add($"{path}.{prop.Key} extra in actual");
                    }
                }
            }
            else if (actual is JsonArray actualArray && expected is JsonArray expectedArray)
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
            else if (actual is JsonValue actualValue && expected is JsonValue expectedValue)
            {
                if (!JsonNode.DeepEquals(actualValue, expectedValue))
                {
                    differences.Add($"{path} differs: actual={actualValue.ToJsonString()} expected={expectedValue.ToJsonString()}");
                }
            }

            return differences;
        }
    }
}
