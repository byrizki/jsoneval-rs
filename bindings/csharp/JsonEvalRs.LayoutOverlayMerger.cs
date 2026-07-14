using System;
using System.Collections.Generic;
using System.Linq;
using Newtonsoft.Json.Linq;

namespace JsonEvalRs
{
    internal static class LayoutOverlayMerger
    {
        internal static JObject Merge(JObject schema, JArray overlayEntries)
        {
            var entries = overlayEntries
                .OfType<JObject>()
                .OrderBy(entry => PointerDepth(entry.Value<string>("layout_path") ?? string.Empty))
                .ThenBy(entry => entry.Value<int?>("element_idx") ?? 0)
                .ToList();
            var resolvedElements = new HashSet<JObject>();

            foreach (var entry in entries)
            {
                var layoutPath = entry.Value<string>("layout_path") ?? string.Empty;
                var elementIndex = entry.Value<int?>("element_idx") ?? -1;
                var elements = GetByPointer(schema, NormalizePointer(layoutPath)) as JArray;
                if (elements == null || elementIndex < 0 || elementIndex >= elements.Count || elements[elementIndex] is not JObject element)
                    continue;

                var reference = element.Value<string>("$ref");
                if (reference != null && reference.Length > 0)
                {
                    var resolvedPointer = ResolveReferencePointer(schema, reference);
                    if (resolvedPointer != null && GetByPointer(schema, resolvedPointer) is JObject resolved)
                    {
                        var merged = FlattenLayout((JObject)resolved.DeepClone());
                        foreach (var property in element.Properties().Where(property => property.Name != "$ref"))
                            merged[property.Name] = property.Value.DeepClone();
                        StampElement(merged, resolvedPointer, layoutPath, elementIndex);
                        resolvedElements.Add(merged);
                        elements[elementIndex] = merged;
                    }
                }

                if (elements[elementIndex] is JObject target && entry["overlay"] is JObject overlay)
                {
                    foreach (var property in overlay.Properties())
                        target[property.Name] = property.Value.DeepClone();
                }
            }

            // Expanded `$ref` items lose `$ref` during merge. Keep their resolved
            // schema-field paths instead of treating them as inline items later.
            StampLayoutMetadata(schema, "#", resolvedElements);
            StampPropertyMetadata(schema, string.Empty, false);
            return schema;
        }

        private static int PointerDepth(string path) => path.Count(character => character == '/');

        private static string NormalizePointer(string path)
        {
            if (string.IsNullOrEmpty(path) || path == "#") return string.Empty;
            if (path.StartsWith("#/", StringComparison.Ordinal)) return path.Substring(1);
            return path.StartsWith("/", StringComparison.Ordinal) ? path : "/" + path;
        }

        private static JToken? GetByPointer(JToken root, string pointer)
        {
            if (string.IsNullOrEmpty(pointer)) return root;
            var current = root;
            foreach (var segment in pointer.TrimStart('/').Split('/'))
            {
                if (string.IsNullOrEmpty(segment)) continue;
                var key = segment.Replace("~1", "/").Replace("~0", "~");
                if (current is JObject obj)
                    current = obj[key];
                else if (current is JArray array && int.TryParse(key, out var index) && index >= 0 && index < array.Count)
                    current = array[index];
                else
                    return null;
                if (current == null) return null;
            }
            return current;
        }

        private static string? ResolveReferencePointer(JObject schema, string reference)
        {
            if (reference.StartsWith("#", StringComparison.Ordinal) || reference.StartsWith("/", StringComparison.Ordinal))
            {
                var pointer = NormalizePointer(reference);
                return GetByPointer(schema, pointer) != null ? pointer : null;
            }
            var dottedPointer = DotNotationToSchemaPointer(reference);
            if (GetByPointer(schema, dottedPointer) != null) return dottedPointer;
            var fallback = "/properties/" + reference.Replace(".", "/properties/");
            return GetByPointer(schema, fallback) != null ? fallback : null;
        }

        private static string DotNotationToSchemaPointer(string path)
        {
            if (path.StartsWith("#", StringComparison.Ordinal) || path.StartsWith("/", StringComparison.Ordinal))
                return NormalizePointer(path);
            if (path.StartsWith("properties.", StringComparison.Ordinal) || path.Contains(".properties."))
                return "/" + path.Replace('.', '/');
            var parts = path.Split('.');
            var result = string.Empty;
            for (var index = 0; index < parts.Length; index++)
            {
                if (parts[index] == "properties") continue;
                if (index > 0 && !path.StartsWith("$", StringComparison.Ordinal)) result += "/properties";
                result += "/" + parts[index];
            }
            return result;
        }

        private static JObject FlattenLayout(JObject resolved)
        {
            if (resolved["$layout"] is not JObject layout) return resolved;
            var result = (JObject)layout.DeepClone();
            foreach (var property in resolved.Properties())
            {
                if (property.Name == "$layout") continue;
                if (property.Name == "type" && result["type"] != null) continue;
                result[property.Name] = property.Value.DeepClone();
            }
            return result;
        }

        private static void StampLayoutMetadata(JToken value, string currentPath, ISet<JObject> resolvedElements)
        {
            if (value is JArray array)
            {
                for (var index = 0; index < array.Count; index++)
                    StampLayoutMetadata(array[index], currentPath + "/" + index, resolvedElements);
                return;
            }
            if (value is not JObject obj) return;
            foreach (var property in obj.Properties().ToList())
            {
                if (property.Name == "$layout" && property.Value is JObject layout && layout["elements"] is JArray elements)
                {
                    StampElements(elements, currentPath + "/$layout/elements", obj.Root as JObject ?? obj, resolvedElements);
                    continue;
                }
                if (property.Name == "elements" && property.Value is JArray nestedElements)
                {
                    StampElements(nestedElements, currentPath + "/elements", obj.Root as JObject ?? obj, resolvedElements);
                    continue;
                }
                StampLayoutMetadata(property.Value, currentPath + "/" + property.Name, resolvedElements);
            }
        }

        private static void StampElements(JArray elements, string layoutPath, JObject schema, ISet<JObject> resolvedElements)
        {
            for (var index = 0; index < elements.Count; index++)
            {
                if (elements[index] is not JObject element) continue;
                var reference = element.Value<string>("$ref");
                var resolvedPointer = reference != null && reference.Length > 0
                    ? ResolveReferencePointer(schema, reference)
                    : null;
                var fullpath = element.Value<string>("$fullpath");
                var needsStamp = reference == null || reference.Length == 0
                    || fullpath == null || fullpath.Length == 0
                    || fullpath.Contains("$layout")
                    || fullpath.Contains("/elements/");
                if (needsStamp && !resolvedElements.Contains(element))
                    StampElement(element, resolvedPointer, layoutPath, index);
                if (element["elements"] is JArray children)
                    StampElements(children, layoutPath.TrimEnd('/') + "/" + index + "/elements", schema, resolvedElements);
            }
        }

        private static void StampElement(JObject element, string? resolvedPointer, string layoutPath, int index)
        {
            string fullpath;
            if (!string.IsNullOrEmpty(resolvedPointer))
            {
                fullpath = resolvedPointer!.TrimStart('/').Replace("/", ".");
            }
            else
            {
                var basePath = layoutPath.TrimStart('#').TrimStart('/').Replace('/', '.');
                fullpath = string.IsNullOrEmpty(basePath) ? index.ToString() : basePath + "." + index;
            }
            element["$fullpath"] = fullpath;
            element["$path"] = fullpath.Split('.').Last();
        }

        private static void StampPropertyMetadata(JToken value, string path, bool parentHidden)
        {
            if (value is not JObject obj) return;
            var hidden = parentHidden || obj["condition"]?["hidden"]?.Value<bool>() == true;
            if (obj["properties"] is JObject properties)
            {
                foreach (var property in properties.Properties().ToList())
                {
                    var propertyPath = string.IsNullOrEmpty(path) ? "properties." + property.Name : path + ".properties." + property.Name;
                    if (property.Value is JObject propertyObject)
                    {
                        propertyObject["$fullpath"] = propertyPath;
                        propertyObject["$path"] = property.Name;
                        propertyObject["$parentHide"] = hidden;
                    }
                    StampPropertyMetadata(property.Value, propertyPath, hidden);
                }
            }
            foreach (var property in obj.Properties().ToList())
            {
                if (property.Name != "properties" && !property.Name.StartsWith("$", StringComparison.Ordinal) && property.Value is JObject)
                {
                    var childPath = string.IsNullOrEmpty(path) ? property.Name : path + "." + property.Name;
                    StampPropertyMetadata(property.Value, childPath, hidden);
                }
            }
        }
    }
}
