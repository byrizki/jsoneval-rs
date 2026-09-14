using JsonEvalRs;
using Newtonsoft.Json.Linq;
using Xunit;

namespace JsonEvalRs.Tests;

public class ValidationTests
{
    [Fact]
    public void Validate_With_IncludeSubforms_Flag()
    {
        var schema = """
        {
          "type": "object",
          "properties": {
            "title": {
              "type": "string",
              "rules": {
                "required": { "value": true, "message": "Title is required" }
              }
            },
            "contacts": {
              "type": "array",
              "items": {
                "properties": {
                  "name": {
                    "type": "string",
                    "rules": {
                      "required": { "value": true, "message": "Name is required" }
                    }
                  },
                  "phone": {
                    "type": "string",
                    "rules": {
                      "required": { "value": true, "message": "Phone is required" }
                    }
                  }
                }
              }
            }
          }
        }
        """;

        var data = """
        {
          "title": "",
          "contacts": [
            { "name": "Alice", "phone": "" },
            { "name": "", "phone": "12345" }
          ]
        }
        """;

        using var eval = new JSONEval(schema);

        // 1. includeSubforms = false: only root field error
        var resWithout = eval.Validate(data, null, validateReadonly: false, includeSubforms: false);
        Assert.True(resWithout.HasError);
        Assert.Single(resWithout.Errors);
        Assert.True(resWithout.Errors.ContainsKey("title"));
        Assert.False(resWithout.Errors.ContainsKey("contacts.0.phone"));

        // 2. includeSubforms = true: root field + subform array item errors
        var resWith = eval.Validate(data, null, validateReadonly: false, includeSubforms: true);
        Assert.True(resWith.HasError);
        Assert.Equal(3, resWith.Errors.Count);
        Assert.True(resWith.Errors.ContainsKey("title"));
        Assert.True(resWith.Errors.ContainsKey("contacts.0.phone"));
        Assert.True(resWith.Errors.ContainsKey("contacts.1.name"));

        Assert.Equal("Phone is required", resWith.Errors["contacts.0.phone"].Message);
        Assert.Equal("contacts.0.phone.required", resWith.Errors["contacts.0.phone"].Code);

        // 3. ValidatePaths with includeSubforms = true
        var paths = new System.Collections.Generic.List<string> { "contacts.0.phone" };
        var resPaths = eval.ValidatePaths(data, null, paths, validateReadonly: false, includeSubforms: true);
        Assert.True(resPaths.HasError);
        Assert.Single(resPaths.Errors);
        Assert.True(resPaths.Errors.ContainsKey("contacts.0.phone"));
    }

    [Fact]
    public void Validate_ErrorData_Contains_Title_Description_And_Constraints()
    {
        var schema = """
        {
          "type": "object",
          "properties": {
            "age": {
              "type": "number",
              "title": "Age",
              "description": "Applicant age",
              "rules": {
                "minValue": { "value": 18, "message": "Min age 18" },
                "maxValue": { "value": 65, "message": "Max age 65" }
              }
            },
            "name": {
              "type": "string",
              "title": "Name",
              "rules": {
                "required": { "value": true, "message": "Name is required" }
              }
            }
          }
        }
        """;

        var data = """{"age": 15, "name": ""}""";

        using var eval = new JSONEval(schema);
        var res = eval.Validate(data);
        Assert.True(res.HasError);
        Assert.Equal(2, res.Errors.Count);

        var ageErr = res.Errors["age"];
        Assert.NotNull(ageErr.Data);
        var ageData = Assert.IsType<JObject>(ageErr.Data);
        Assert.Equal("Age", (string?)ageData["title"]);
        Assert.Equal("Applicant age", (string?)ageData["description"]);
        Assert.Equal(18, (int?)ageData["minValue"]);
        Assert.Equal(65, (int?)ageData["maxValue"]);
        Assert.Null(ageData["min"]);
        Assert.Null(ageData["max"]);

        var nameErr = res.Errors["name"];
        Assert.NotNull(nameErr.Data);
        var nameData = Assert.IsType<JObject>(nameErr.Data);
        Assert.Equal("Name", (string?)nameData["title"]);
        Assert.Equal(true, (bool?)nameData["required"]);
    }
}
