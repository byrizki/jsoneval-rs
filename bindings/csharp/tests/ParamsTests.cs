using JsonEvalRs;
using Newtonsoft.Json.Linq;
using Xunit;

namespace JsonEvalRs.Tests;

public class ParamsTests
{
    private const string SampleSchemaWithParams = """
    {
      "$params": {
        "metadata": "form_v1",
        "constants": {
          "TAX_RATE": 0.11
        },
        "references": {
          "SMALL_LIST": ["a", "b", "c"],
          "LARGE_TABLE": [
            {"id": 1, "val": 10},
            {"id": 2, "val": 20},
            {"id": 3, "val": 30},
            {"id": 4, "val": 40},
            {"id": 5, "val": 50},
            {"id": 6, "val": 60},
            {"id": 7, "val": 70},
            {"id": 8, "val": 80},
            {"id": 9, "val": 90},
            {"id": 10, "val": 100},
            {"id": 11, "val": 110},
            {"id": 12, "val": 120}
          ]
        }
      },
      "properties": {
        "rate": {
          "type": "number",
          "value": {
            "$evaluation": {
              "var": "$params.constants.TAX_RATE"
            }
          }
        }
      }
    }
    """;

    private const string SubformSchemaWithParams = """
    {
      "$params": {
        "sub_meta": "sub_v1",
        "list": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
      },
      "benefits": {
        "type": "array",
        "items": {
          "properties": {
            "field": { "type": "string", "value": "test" }
          }
        }
      }
    }
    """;

    [Fact]
    public void GetPlainParams_StripsStaticArrays()
    {
        using var eval = new JSONEval(SampleSchemaWithParams);

        var plain = eval.GetPlainParams();
        Assert.NotNull(plain);
        Assert.Equal("form_v1", plain["metadata"]?.ToString());
        Assert.Equal(0.11, plain["constants"]?["TAX_RATE"]?.Value<double>());

        var references = plain["references"] as JObject;
        Assert.NotNull(references);
        Assert.NotNull(references["SMALL_LIST"]);
        Assert.Equal(3, ((JArray)references["SMALL_LIST"]!).Count);

        // Static array (> 10 items) must be stripped
        Assert.Null(references["LARGE_TABLE"]);
    }

    [Fact]
    public void GetEvaluatedParams_WithoutAndWithStaticArray()
    {
        using var eval = new JSONEval(SampleSchemaWithParams);
        eval.Evaluate("{}");

        var evalWithout = eval.GetEvaluatedParams(withStaticArray: false);
        Assert.NotNull(evalWithout);
        Assert.Equal("form_v1", evalWithout["metadata"]?.ToString());
        Assert.Null(evalWithout["references"]?["LARGE_TABLE"]);

        var evalWith = eval.GetEvaluatedParams(withStaticArray: true);
        Assert.NotNull(evalWith);
        var largeTable = evalWith["references"]?["LARGE_TABLE"] as JArray;
        Assert.NotNull(largeTable);
        Assert.Equal(12, largeTable.Count);
    }

    [Fact]
    public void SubformParams_WorksCorrectly()
    {
        using var eval = new JSONEval(SubformSchemaWithParams);

        var plainSub = eval.GetPlainParamsSubform("#/benefits");
        Assert.NotNull(plainSub);
        Assert.Equal("sub_v1", plainSub["sub_meta"]?.ToString());
        Assert.Null(plainSub["list"]);

        var evalSubWithout = eval.GetEvaluatedParamsSubform("#/benefits", false);
        Assert.NotNull(evalSubWithout);
        Assert.Null(evalSubWithout["list"]);

        var evalSubWith = eval.GetEvaluatedParamsSubform("#/benefits", true);
        Assert.NotNull(evalSubWith);
        var list = evalSubWith["list"] as JArray;
        Assert.NotNull(list);
        Assert.Equal(11, list.Count);
    }

    [Fact]
    public void CamelCaseAliases_MatchPascalCaseResults()
    {
        using var eval = new JSONEval(SampleSchemaWithParams);

        var pascalPlain = eval.GetPlainParams();
        var camelPlain = eval.getPlainParams();
        Assert.NotNull(pascalPlain);
        Assert.NotNull(camelPlain);
        Assert.True(JToken.DeepEquals(pascalPlain, camelPlain));

        eval.Evaluate("{}");

        var pascalEval = eval.GetEvaluatedParams(true);
        var camelEval = eval.getEvaluatedParams(true);
        Assert.NotNull(pascalEval);
        Assert.NotNull(camelEval);
        Assert.True(JToken.DeepEquals(pascalEval, camelEval));
    }

    [Fact]
    public void GetParams_OnSchemaWithoutParams_ReturnsNull()
    {
        const string schemaWithoutParams = """
        {
          "properties": {
            "name": { "type": "string" }
          }
        }
        """;

        using var eval = new JSONEval(schemaWithoutParams);

        Assert.Null(eval.GetPlainParams());
        Assert.Null(eval.GetEvaluatedParams(false));
        Assert.Null(eval.GetEvaluatedParams(true));
        Assert.Null(eval.getPlainParams());
        Assert.Null(eval.getEvaluatedParams());
    }
}
