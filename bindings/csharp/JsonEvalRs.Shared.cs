using System;
using System.Collections.Generic;
using Newtonsoft.Json;

namespace JsonEvalRs
{
    /// <summary>
    /// Validation error for a specific field
    /// </summary>
    public class ValidationError
    {
        [JsonProperty("path")]
        public string Path { get; set; } = string.Empty;

        [JsonProperty("type")]
        public string Type { get; set; } = string.Empty;

        [JsonProperty("message")]
        public string Message { get; set; } = string.Empty;

        [JsonProperty("code")]
        public string? Code { get; set; }

        [JsonProperty("pattern")]
        public string? Pattern { get; set; }

        [JsonProperty("fieldValue")]
        public string? FieldValue { get; set; }

        [JsonProperty("data")]
        public object? Data { get; set; }
        
        // Backwards compatibility - RuleType maps to Type
        [JsonIgnore]
        public string RuleType 
        {
            get => Type;
            set => Type = value;
        }
    }

    /// <summary>
    /// Result of validation operation
    /// </summary>
    public class ValidationResult
    {
        [JsonProperty("hasError")]
        public bool HasError { get; set; }

        [JsonProperty("errors")]
        public List<ValidationError> Errors { get; set; } = new List<ValidationError>();
    }

    /// <summary>
    /// Cache statistics
    /// </summary>
    public class CacheStats
    {
        [JsonProperty("hits")]
        public ulong Hits { get; set; }

        [JsonProperty("misses")]
        public ulong Misses { get; set; }

        [JsonProperty("entries")]
        public ulong Entries { get; set; }
    }

    /// <summary>
    /// Exception thrown when JSON evaluation operations fail
    /// </summary>
    public class JsonEvalException : Exception
    {
        public JsonEvalException(string message) : base(message) { }
        public JsonEvalException(string message, Exception innerException) : base(message, innerException) { }
    }
}
