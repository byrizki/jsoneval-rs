using JsonEvalRs;
using Newtonsoft.Json.Linq;
using Xunit;

namespace JsonEvalRs.Tests;

public class LayoutOverlayMergerTests
{
    [Fact]
    public void Merge_stamps_inline_layout_and_property_metadata()
    {
        var schema = JObject.Parse("""
        {
          "illustration": {
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "$layout": {
              "elements": [
                { "$ref": "#/illustration/properties/name" },
                { "type": "TabLayout" }
              ]
            }
          }
        }
        """);
        var overlays = JArray.Parse("""
        [
          {
            "layout_path": "#/illustration/$layout/elements",
            "element_idx": 0,
            "schema_ref_path": "illustration.properties.name",
            "overlay": {
              "$fullpath": "illustration.properties.name",
              "$path": "name",
              "$parentHide": false
            }
          },
          {
            "layout_path": "#/illustration/$layout/elements",
            "element_idx": 1,
            "schema_ref_path": "",
            "overlay": { "$fullpath": "illustration.1", "$path": "1", "$parentHide": false }
          }
        ]
        """);

        var resolved = LayoutOverlayMerger.Merge(schema, overlays);

        Assert.Equal(
            "illustration.$layout.elements.1",
            resolved.SelectToken("illustration.$layout.elements[1].$fullpath")?.Value<string>());
        Assert.Equal(
            "illustration.properties.name",
            resolved.SelectToken("illustration.$layout.elements[0].$fullpath")?.Value<string>());
        Assert.Equal(
            "illustration.properties.name",
            resolved.SelectToken("illustration.properties.name.$fullpath")?.Value<string>());
        Assert.Equal("name", resolved.SelectToken("illustration.properties.name.$path")?.Value<string>());
        Assert.False(resolved.SelectToken("illustration.properties.name.$parentHide")?.Value<bool>() ?? true);
    }

    [Fact]
    public void Merge_preserves_resolved_ref_path_without_overlay_metadata()
    {
        var schema = JObject.Parse("""
        {
          "illustration": {
            "properties": { "name": { "type": "string" } },
            "$layout": { "elements": [{ "$ref": "#/illustration/properties/name" }] }
          }
        }
        """);
        var overlays = JArray.Parse("""
        [
          {
            "layout_path": "#/illustration/$layout/elements",
            "element_idx": 0,
            "schema_ref_path": "illustration.properties.name",
            "overlay": {}
          }
        ]
        """);

        var resolved = LayoutOverlayMerger.Merge(schema, overlays);

        Assert.Equal(
            "illustration.properties.name",
            resolved.SelectToken("illustration.$layout.elements[0].$fullpath")?.Value<string>());
        Assert.Equal(
            "name",
            resolved.SelectToken("illustration.$layout.elements[0].$path")?.Value<string>());
    }

    [Fact]
    public void Merge_preserves_full_non_property_ref_path()
    {
        var schema = JObject.Parse("""
        {
          "illustration": {
            "definition": { "type": "string" },
            "$layout": { "elements": [{ "$ref": "#/illustration/definition" }] }
          }
        }
        """);
        var overlays = JArray.Parse("""
        [
          {
            "layout_path": "#/illustration/$layout/elements",
            "element_idx": 0,
            "schema_ref_path": "illustration.definition",
            "overlay": {}
          }
        ]
        """);

        var resolved = LayoutOverlayMerger.Merge(schema, overlays);

        Assert.Equal(
            "illustration.definition",
            resolved.SelectToken("illustration.$layout.elements[0].$fullpath")?.Value<string>());
    }

    [Fact]
    public void Merge_stamps_properties_when_overlays_are_empty()
    {
        var schema = JObject.Parse("""
        { "profile": { "properties": { "email": { "type": "string" } } } }
        """);

        var resolved = LayoutOverlayMerger.Merge(schema, new JArray());

        Assert.Equal(
            "profile.properties.email",
            resolved.SelectToken("profile.properties.email.$fullpath")?.Value<string>());
        Assert.Equal("email", resolved.SelectToken("profile.properties.email.$path")?.Value<string>());
        Assert.False(resolved.SelectToken("profile.properties.email.$parentHide")?.Value<bool>() ?? true);
    }
}
