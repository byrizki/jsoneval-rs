using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using JsonEvalRs;
using Newtonsoft.Json.Linq;
using Xunit;

namespace JsonEvalRs.Tests;

public class ConcurrencyTests
{
    [Fact]
    public async Task Evaluate_SameParsedSchema_ConcurrentDifferentData_ProducesCorrectIsolatedResults()
    {
        const string schemaKey = "concurrency_eval_calc_schema";
        const string schemaJson = """
        {
          "type": "object",
          "properties": {
            "quantity": { "type": "number" },
            "unit_price": { "type": "number" },
            "total": {
              "type": "number",
              "$evaluation": { "logic": { "*": [{ "var": "quantity" }, { "var": "unit_price" }] } }
            },
            "discount": {
              "type": "number",
              "$evaluation": {
                "logic": {
                  "if": [
                    { ">": [{ "var": "quantity" }, 50] },
                    50,
                    0
                  ]
                }
              }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        const int taskCount = 100;
        var tasks = Enumerable.Range(1, taskCount).Select(i => Task.Run(() =>
        {
            int quantity = i;
            const int unitPrice = 10;
            int expectedTotal = quantity * unitPrice;
            int expectedDiscount = quantity > 50 ? 50 : 0;

            var data = new JObject
            {
                ["quantity"] = quantity,
                ["unit_price"] = unitPrice
            }.ToString();

            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate(data);

            var evaluatedSchema = eval.GetEvaluatedSchema();
            int actualTotal = evaluatedSchema.SelectToken("properties.total")?.Value<int>() ?? -1;
            int actualDiscount = evaluatedSchema.SelectToken("properties.discount")?.Value<int>() ?? -1;

            Assert.Equal(expectedTotal, actualTotal);
            Assert.Equal(expectedDiscount, actualDiscount);
        }));

        await Task.WhenAll(tasks);
    }

    [Fact]
    public async Task Validate_SameParsedSchema_ConcurrentDifferentData_ProducesIsolatedValidationErrors()
    {
        const string schemaKey = "concurrency_validation_schema";
        const string schemaJson = """
        {
          "type": "object",
          "properties": {
            "username": {
              "type": "string",
              "rules": {
                "required": { "value": true, "message": "Username is required" }
              }
            },
            "age": {
              "type": "number",
              "rules": {
                "required": { "value": true, "message": "Age is required" }
              }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        const int taskCount = 120;
        var tasks = Enumerable.Range(0, taskCount).Select(i => Task.Run(() =>
        {
            int scenario = i % 3;
            string data;

            switch (scenario)
            {
                case 0:
                    // Valid
                    data = $"{{\"username\": \"user_{i}\", \"age\": 25}}";
                    break;
                case 1:
                    // Missing username
                    data = "{\"username\": \"\", \"age\": 25}";
                    break;
                default:
                    // Missing age
                    data = $"{{\"username\": \"user_{i}\"}}";
                    break;
            }

            using var eval = JSONEval.FromCache(schemaKey);
            var validation = eval.Validate(data, null, validateReadonly: false, includeSubforms: false);

            if (scenario == 0)
            {
                Assert.False(validation.HasError);
                Assert.Empty(validation.Errors);
            }
            else if (scenario == 1)
            {
                Assert.True(validation.HasError);
                Assert.True(validation.Errors.ContainsKey("username"));
                Assert.False(validation.Errors.ContainsKey("age"));
                Assert.Equal("Username is required", validation.Errors["username"].Message);
            }
            else
            {
                Assert.True(validation.HasError);
                Assert.True(validation.Errors.ContainsKey("age"));
                Assert.False(validation.Errors.ContainsKey("username"));
                Assert.Equal("Age is required", validation.Errors["age"].Message);
            }
        }));

        await Task.WhenAll(tasks);
    }

    [Fact]
    public async Task Subforms_SameParsedSchema_ConcurrentDifferentArrayItems_IsolatesSubformEvaluations()
    {
        const string schemaKey = "concurrency_subform_schema";
        const string schemaJson = """
        {
          "type": "object",
          "properties": {
            "department": { "type": "string" },
            "employees": {
              "type": "array",
              "items": {
                "properties": {
                  "name": {
                    "type": "string",
                    "rules": {
                      "required": { "value": true, "message": "Name is required" }
                    }
                  },
                  "email": {
                    "type": "string",
                    "rules": {
                      "required": { "value": true, "message": "Email is required" }
                    }
                  }
                }
              }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        const int taskCount = 60;
        var tasks = Enumerable.Range(0, taskCount).Select(i => Task.Run(() =>
        {
            int itemCount = (i % 5) + 1;
            bool shouldFail = (i % 2) == 1;
            int failingIndex = shouldFail ? (i % itemCount) : -1;

            var employeesArray = new JArray();
            for (int idx = 0; idx < itemCount; idx++)
            {
                var emp = new JObject
                {
                    ["name"] = $"Emp_{i}_{idx}",
                    ["email"] = (idx == failingIndex) ? "" : $"emp_{i}_{idx}@company.com"
                };
                employeesArray.Add(emp);
            }

            var rootObj = new JObject
            {
                ["department"] = $"Dept_{i}",
                ["employees"] = employeesArray
            };

            string data = rootObj.ToString();

            using var eval = JSONEval.FromCache(schemaKey);
            var validation = eval.Validate(data, null, validateReadonly: false, includeSubforms: true);

            if (!shouldFail)
            {
                Assert.False(validation.HasError);
                Assert.Empty(validation.Errors);
            }
            else
            {
                Assert.True(validation.HasError);
                string expectedErrorKey = $"employees.{failingIndex}.email";
                Assert.True(validation.Errors.ContainsKey(expectedErrorKey));
                Assert.Equal("Email is required", validation.Errors[expectedErrorKey].Message);
            }
        }));

        await Task.WhenAll(tasks);
    }

    [Fact]
    public async Task StressTest_MultiSchema_ConcurrentEvaluators_NoMemoryCorruptionOrRaceConditions()
    {
        const string schemaKey1 = "stress_calc_schema";
        const string schemaKey2 = "stress_valid_schema";

        const string schema1Json = """
        {
          "type": "object",
          "properties": {
            "a": { "type": "number" },
            "b": { "type": "number" },
            "sum": {
              "type": "number",
              "$evaluation": { "logic": { "+": [{ "var": "a" }, { "var": "b" }] } }
            }
          }
        }
        """;

        const string schema2Json = """
        {
          "type": "object",
          "properties": {
            "code": {
              "type": "string",
              "rules": {
                "required": { "value": true, "message": "Code is required" }
              }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey1, schema1Json);
        ParsedSchemaCache.Global.Insert(schemaKey2, schema2Json);

        const int taskCount = 200;
        var tasks = Enumerable.Range(0, taskCount).Select(i => Task.Run(() =>
        {
            if (i % 2 == 0)
            {
                int a = i;
                int b = i * 2;
                int expectedSum = a + b;

                var data = $"{{\"a\": {a}, \"b\": {b}}}";
                using var eval = JSONEval.FromCache(schemaKey1);
                eval.Evaluate(data);

                var evaluated = eval.GetEvaluatedSchema();
                int actualSum = evaluated.SelectToken("properties.sum")?.Value<int>() ?? -1;
                Assert.Equal(expectedSum, actualSum);
            }
            else
            {
                bool isValid = (i % 4) == 1;
                string data = isValid ? $"{{\"code\": \"CODE_{i}\"}}" : "{\"code\": \"\"}";

                using var eval = JSONEval.FromCache(schemaKey2);
                var validation = eval.Validate(data, null, validateReadonly: false, includeSubforms: false);

                if (isValid)
                {
                    Assert.False(validation.HasError);
                }
                else
                {
                    Assert.True(validation.HasError);
                    Assert.True(validation.Errors.ContainsKey("code"));
                }
            }
        }));

        await Task.WhenAll(tasks);
    }

    [Fact]
    public async Task ReloadSchemaFromCache_ConcurrentExecution_RemainsThreadSafe()
    {
        const string schemaKey = "concurrency_reload_schema";
        const string schemaJson = """
        {
          "type": "object",
          "properties": {
            "multiplier": { "type": "number" },
            "val": { "type": "number" },
            "computed": {
              "type": "number",
              "$evaluation": { "logic": { "*": [{ "var": "multiplier" }, { "var": "val" }] } }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        const int taskCount = 50;
        var tasks = Enumerable.Range(1, taskCount).Select(i => Task.Run(() =>
        {
            using var eval = JSONEval.FromCache(schemaKey);

            // Phase 1 evaluation
            string data1 = $"{{\"multiplier\": 2, \"val\": {i}}}";
            eval.Evaluate(data1);
            var result1 = eval.GetEvaluatedSchema();
            Assert.Equal(2 * i, result1.SelectToken("properties.computed")?.Value<int>());

            // Phase 2 reload schema and evaluate with different data
            eval.ReloadSchemaFromCache(schemaKey);
            string data2 = $"{{\"multiplier\": 5, \"val\": {i}}}";
            eval.Evaluate(data2);
            var result2 = eval.GetEvaluatedSchema();
            Assert.Equal(5 * i, result2.SelectToken("properties.computed")?.Value<int>());
        }));

        await Task.WhenAll(tasks);
    }

    [Fact]
    public async Task ParsedSchemaCache_ConcurrentReadsAndWrites_ThreadSafe()
    {
        using var localCache = new ParsedSchemaCache();
        const int taskCount = 100;

        const string sampleSchema = """
        {
          "type": "object",
          "properties": {
            "val": { "type": "number" }
          }
        }
        """;

        var tasks = Enumerable.Range(0, taskCount).Select(i => Task.Run(() =>
        {
            string key = $"schema_{i % 10}";

            // Concurrently insert schemas
            localCache.Insert(key, sampleSchema);

            // Concurrently query cache state
            Assert.True(localCache.Contains(key));
            Assert.False(localCache.IsEmpty);
            Assert.True(localCache.Count > 0);

            var stats = localCache.GetStats();
            Assert.NotNull(stats);
            Assert.True(stats.EntryCount > 0);

            var keys = localCache.GetKeys();
            Assert.NotNull(keys);
            Assert.NotEmpty(keys);
        }));

        await Task.WhenAll(tasks);
        Assert.Equal(10, localCache.Count);
    }

    [Fact]
    public async Task TableCalculation_SameParsedSchema_ConcurrentDifferentData_ProducesIsolatedTables()
    {
        const string schemaKey = "concurrency_table_calc_schema";
        const string schemaJson = """
        {
          "$params": {
            "references": {
              "CALC_TABLE": {
                "$table": [
                  {
                    "$repeat": [
                      0,
                      4,
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
                    4,
                    "VAL"
                  ]
                }
              }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        const int taskCount = 100;
        var tasks = Enumerable.Range(1, taskCount).Select(i => Task.Run(() =>
        {
            int multiplier = i;
            int expectedResult = 4 * multiplier;

            var data = $"{{\"multiplier\": {multiplier}}}";
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate(data);

            var evaluated = eval.GetEvaluatedSchema();
            int actualResult = evaluated.SelectToken("properties.result")?.Value<int>() ?? -1;
            Assert.Equal(expectedResult, actualResult);

            var cell = eval.GetEvaluatedSchemaByPath("$params.references.CALC_TABLE.3.VAL");
            Assert.NotNull(cell);
            Assert.Equal(3 * multiplier, cell.Value<int>());
        }));

        await Task.WhenAll(tasks);
    }

    [Fact]
    public async Task FieldDependents_SameParsedSchema_ConcurrentChanges_IsolatesDependentEvaluations()
    {
        const string schemaKey = "concurrency_field_dependents_schema";
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
                        {
                          "if": [
                            { "==": [{ "$ref": "$value" }, "STANDARD"] },
                            20,
                            10
                          ]
                        }
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

        const int taskCount = 120;
        var tasks = Enumerable.Range(0, taskCount).Select(i => Task.Run(() =>
        {
            int variant = i % 3;
            string tier;
            int expectedRate;

            switch (variant)
            {
                case 0:
                    tier = "PREMIUM";
                    expectedRate = 50;
                    break;
                case 1:
                    tier = "STANDARD";
                    expectedRate = 20;
                    break;
                default:
                    tier = "BASIC";
                    expectedRate = 10;
                    break;
            }

            // Initial evaluator with default data
            var initialData = "{\"tier\": \"NONE\", \"rate\": 0}";
            using var eval = JSONEval.FromCache(schemaKey);
            eval.Evaluate(initialData);

            // Trigger field dependent change with new tier
            var updatedData = $"{{\"tier\": \"{tier}\", \"rate\": 0}}";
            var changes = eval.EvaluateDependents(
                new[] { "#/properties/tier" },
                updatedData,
                context: null,
                reEvaluate: true,
                includeSubforms: false
            );

            Assert.NotNull(changes);
            Assert.NotEmpty(changes);

            // Verify rate in returned dependent changes
            var rateChange = changes.FirstOrDefault(c => c["$ref"]?.ToString() == "rate" || c["$ref"]?.ToString() == "#/properties/rate");
            Assert.NotNull(rateChange);
            int actualRate = rateChange["value"]?.Value<int>() ?? -1;
            Assert.Equal(expectedRate, actualRate);
        }));

        await Task.WhenAll(tasks);
    }

    [Fact]
    public async Task ValidationCache_SameParsedSchema_ConcurrentValidations_HitAndInvalidateSafely()
    {
        const string schemaKey = "concurrency_validation_cache_schema";
        const string schemaJson = """
        {
          "type": "object",
          "properties": {
            "name": {
              "type": "string",
              "rules": {
                "required": { "value": true, "message": "Name is required" }
              }
            },
            "score": {
              "type": "number",
              "rules": {
                "minValue": { "value": 0, "message": "Score must be non-negative" }
              }
            }
          }
        }
        """;

        ParsedSchemaCache.Global.Insert(schemaKey, schemaJson);

        const int taskCount = 100;
        var tasks = Enumerable.Range(0, taskCount).Select(i => Task.Run(() =>
        {
            using var eval = JSONEval.FromCache(schemaKey);

            // Round 1: Valid data (populates validation cache)
            string validData = $"{{\"name\": \"User_{i}\", \"score\": 100}}";
            var res1 = eval.Validate(validData, null, validateReadonly: false, includeSubforms: false);
            Assert.False(res1.HasError);
            Assert.Empty(res1.Errors);

            // Round 2: Repeated identical validation (hits ValidationCache full-result fast path)
            var res2 = eval.Validate(validData, null, validateReadonly: false, includeSubforms: false);
            Assert.False(res2.HasError);
            Assert.Empty(res2.Errors);

            // Round 3: Modified data with error (triggers cache miss / invalidation)
            string invalidData = $"{{\"name\": \"\", \"score\": 100}}";
            var res3 = eval.Validate(invalidData, null, validateReadonly: false, includeSubforms: false);
            Assert.True(res3.HasError);
            Assert.True(res3.Errors.ContainsKey("name"));
            Assert.Equal("Name is required", res3.Errors["name"].Message);

            // Round 4: Repeated invalid validation (hits ValidationCache with cached error)
            var res4 = eval.Validate(invalidData, null, validateReadonly: false, includeSubforms: false);
            Assert.True(res4.HasError);
            Assert.True(res4.Errors.ContainsKey("name"));
            Assert.Equal("Name is required", res4.Errors["name"].Message);
        }));

        await Task.WhenAll(tasks);
    }
}
