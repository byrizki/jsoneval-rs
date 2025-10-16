using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Collections.Generic;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace JsonEvalRs
{
    /// <summary>
    /// P/Invoke interop with the native json-eval-rs library
    /// </summary>
    internal static class Native
    {
        private const string LibName = "json_eval_rs";

        static Native()
        {
            // Set up DLL import resolver for .NET 5+ to help find the native library
#if NET5_0_OR_GREATER
            NativeLibrary.SetDllImportResolver(typeof(Native).Assembly, DllImportResolver);
#endif
        }

#if NET5_0_OR_GREATER
        private static IntPtr DllImportResolver(string libraryName, System.Reflection.Assembly assembly, DllImportSearchPath? searchPath)
        {
            if (libraryName != LibName)
                return IntPtr.Zero;

            // Try to load from different possible locations
            string[] possiblePaths = GetPossibleLibraryPaths();
            
            foreach (var path in possiblePaths)
            {
                if (NativeLibrary.TryLoad(path, out IntPtr handle))
                {
                    return handle;
                }
            }

            // Let the default resolver try
            return IntPtr.Zero;
        }

        private static string[] GetPossibleLibraryPaths()
        {
            var paths = new List<string>();
            var assemblyLocation = System.IO.Path.GetDirectoryName(typeof(Native).Assembly.Location) ?? "";
            
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                paths.Add(System.IO.Path.Combine(assemblyLocation, "json_eval_rs.dll"));
                paths.Add(System.IO.Path.Combine(assemblyLocation, "runtimes", "win-x64", "native", "json_eval_rs.dll"));
            }
            else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            {
                paths.Add(System.IO.Path.Combine(assemblyLocation, "libjson_eval_rs.so"));
                paths.Add(System.IO.Path.Combine(assemblyLocation, "runtimes", "linux-x64", "native", "libjson_eval_rs.so"));
            }
            else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            {
                paths.Add(System.IO.Path.Combine(assemblyLocation, "libjson_eval_rs.dylib"));
                paths.Add(System.IO.Path.Combine(assemblyLocation, "runtimes", "osx-x64", "native", "libjson_eval_rs.dylib"));
            }

            return paths.ToArray();
        }
#endif

        [StructLayout(LayoutKind.Sequential)]
        internal struct FFIResult
        {
            [MarshalAs(UnmanagedType.I1)]
            public bool Success;
            public IntPtr Data;
            public IntPtr Error;
        }

#if NETCOREAPP || NET5_0_OR_GREATER
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr json_eval_new(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string schema,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr json_eval_new_with_error(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string schema,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data,
            out IntPtr errorOut
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_evaluate(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string data,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_validate(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string data,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_evaluate_dependents(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string changedPathsJson,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string data,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context,
            [MarshalAs(UnmanagedType.I1)] bool nested
        );
#else
        // .NET Standard 2.0/2.1 - Use byte array marshalling
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr json_eval_new(
            byte[]? schema,
            byte[]? context,
            byte[]? data
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr json_eval_new_with_error(
            byte[]? schema,
            byte[]? context,
            byte[]? data,
            out IntPtr errorOut
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_evaluate(
            IntPtr handle,
            byte[]? data,
            byte[]? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_validate(
            IntPtr handle,
            byte[]? data,
            byte[]? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_evaluate_dependents(
            IntPtr handle,
            byte[]? changedPathsJson,
            byte[]? data,
            byte[]? context,
            [MarshalAs(UnmanagedType.I1)] bool nested
        );

        internal static byte[]? ToUTF8Bytes(string? str)
        {
            if (str == null) return null;
            return Encoding.UTF8.GetBytes(str + "\0"); // Null-terminated
        }

        internal static string? PtrToStringUTF8(IntPtr ptr)
        {
            if (ptr == IntPtr.Zero)
                return null;

            int length = 0;
            while (Marshal.ReadByte(ptr, length) != 0)
                length++;

            byte[] buffer = new byte[length];
            Marshal.Copy(ptr, buffer, 0, length);
            return Encoding.UTF8.GetString(buffer);
        }
#endif

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void json_eval_free_result(FFIResult result);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void json_eval_free_string(IntPtr ptr);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void json_eval_free(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema(
            IntPtr handle,
            [MarshalAs(UnmanagedType.I1)] bool skipLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_schema_value(IntPtr handle);

#if NETCOREAPP || NET5_0_OR_GREATER
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_reload_schema(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string schema,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_validate_paths(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string data,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? pathsJson
        );
#else
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_reload_schema(
            IntPtr handle,
            byte[]? schema,
            byte[]? context,
            byte[]? data
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_validate_paths(
            IntPtr handle,
            byte[]? data,
            byte[]? context,
            byte[]? pathsJson
        );
#endif

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_cache_stats(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_clear_cache(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_cache_len(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr json_eval_version();
    }

    /// <summary>
    /// Validation error for a specific field
    /// </summary>
    public class ValidationError
    {
        [JsonProperty("path")]
        public string Path { get; set; } = string.Empty;

        [JsonProperty("ruleType")]
        public string RuleType { get; set; } = string.Empty;

        [JsonProperty("message")]
        public string Message { get; set; } = string.Empty;
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

    /// <summary>
    /// High-performance JSON Logic evaluator with schema validation
    /// </summary>
    public class JSONEval : IDisposable
    {
        private IntPtr _handle;
        private bool _disposed;

        /// <summary>
        /// Gets the library version
        /// </summary>
        public static string Version
        {
            get
            {
                IntPtr ptr = Native.json_eval_version();
                try
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    return Marshal.PtrToStringUTF8(ptr) ?? "unknown";
#else
                    return Native.PtrToStringUTF8(ptr) ?? "unknown";
#endif
                }
                finally
                {
                    Native.json_eval_free_string(ptr);
                }
            }
        }

        /// <summary>
        /// Creates a new JSON evaluator instance
        /// </summary>
        /// <param name="schema">JSON schema string</param>
        /// <param name="context">Optional context data (can be null)</param>
        /// <param name="data">Optional initial data (can be null)</param>
        public JSONEval(string schema, string? context = null, string? data = null)
        {
            if (string.IsNullOrEmpty(schema))
                throw new ArgumentNullException(nameof(schema));

            // Test if library can be loaded
            try
            {
                var version = Version;
            }
            catch (Exception ex)
            {
                throw new JsonEvalException(
                    $"Failed to load native library 'json_eval_rs'. Make sure the native library is in the correct location. " +
                    $"Platform: {RuntimeInformation.OSDescription}, " +
                    $"Architecture: {RuntimeInformation.ProcessArchitecture}. " +
                    $"Error: {ex.Message}", ex);
            }

            // Use the new error-reporting function
            IntPtr errorPtr;
#if NETCOREAPP || NET5_0_OR_GREATER
            _handle = Native.json_eval_new_with_error(schema, context, data, out errorPtr);
#else
            _handle = Native.json_eval_new_with_error(
                Native.ToUTF8Bytes(schema),
                Native.ToUTF8Bytes(context),
                Native.ToUTF8Bytes(data),
                out errorPtr
            );
#endif
            
            if (_handle == IntPtr.Zero)
            {
                string errorMessage = "Failed to create JSONEval instance";
                if (errorPtr != IntPtr.Zero)
                {
                    try
                    {
#if NETCOREAPP || NET5_0_OR_GREATER
                        errorMessage = Marshal.PtrToStringUTF8(errorPtr) ?? errorMessage;
#else
                        errorMessage = Native.PtrToStringUTF8(errorPtr) ?? errorMessage;
#endif
                    }
                    finally
                    {
                        Native.json_eval_free_string(errorPtr);
                    }
                }
                throw new JsonEvalException(errorMessage);
            }
        }

        /// <summary>
        /// Evaluates the schema with provided data
        /// </summary>
        /// <param name="data">JSON data string</param>
        /// <param name="context">Optional context data</param>
        /// <returns>Evaluated schema as JObject</returns>
        public JObject Evaluate(string data, string? context = null)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(data))
                throw new ArgumentNullException(nameof(data));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_evaluate(_handle, data, context);
#else
            var result = Native.json_eval_evaluate(_handle, Native.ToUTF8Bytes(data), Native.ToUTF8Bytes(context));
#endif
            return ProcessResult(result);
        }

        /// <summary>
        /// Validates data against schema rules
        /// </summary>
        /// <param name="data">JSON data string</param>
        /// <param name="context">Optional context data</param>
        /// <returns>ValidationResult</returns>
        public ValidationResult Validate(string data, string? context = null)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(data))
                throw new ArgumentNullException(nameof(data));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_validate(_handle, data, context);
#else
            var result = Native.json_eval_validate(_handle, Native.ToUTF8Bytes(data), Native.ToUTF8Bytes(context));
#endif
            var jsonResult = ProcessResult(result);
            return jsonResult.ToObject<ValidationResult>();
        }

        /// <summary>
        /// Re-evaluates fields that depend on changed paths
        /// </summary>
        /// <param name="changedPaths">List of field paths that changed</param>
        /// <param name="data">Updated JSON data string</param>
        /// <param name="context">Optional context data</param>
        /// <param name="nested">Whether to recursively follow dependency chains</param>
        /// <returns>Updated evaluated schema as JObject</returns>
        public JObject EvaluateDependents(List<string> changedPaths, string data, 
            string? context = null, bool nested = true)
        {
            ThrowIfDisposed();

            if (changedPaths == null || changedPaths.Count == 0)
                throw new ArgumentException("Changed paths cannot be empty", nameof(changedPaths));

            if (string.IsNullOrEmpty(data))
                throw new ArgumentNullException(nameof(data));

            string pathsJson = JsonConvert.SerializeObject(changedPaths);
#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_evaluate_dependents(_handle, pathsJson, data, context, nested);
#else
            var result = Native.json_eval_evaluate_dependents(_handle, Native.ToUTF8Bytes(pathsJson), Native.ToUTF8Bytes(data), Native.ToUTF8Bytes(context), nested);
#endif
            return ProcessResult(result);
        }

        /// <summary>
        /// Gets the evaluated schema with optional layout resolution
        /// </summary>
        /// <param name="skipLayout">Whether to skip layout resolution</param>
        /// <returns>Evaluated schema as JObject</returns>
        public JObject GetEvaluatedSchema(bool skipLayout = false)
        {
            ThrowIfDisposed();
            var result = Native.json_eval_get_evaluated_schema(_handle, skipLayout);
            return ProcessResult(result);
        }

        /// <summary>
        /// Gets all schema values (evaluations ending with .value)
        /// </summary>
        /// <returns>Dictionary of path -> value</returns>
        public JObject GetSchemaValue()
        {
            ThrowIfDisposed();
            var result = Native.json_eval_get_schema_value(_handle);
            return ProcessResult(result);
        }

        /// <summary>
        /// Reloads the schema with new data
        /// </summary>
        /// <param name="schema">New JSON schema string</param>
        /// <param name="context">Optional context data</param>
        /// <param name="data">Optional initial data</param>
        public void ReloadSchema(string schema, string? context = null, string? data = null)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(schema))
                throw new ArgumentNullException(nameof(schema));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_reload_schema(_handle, schema, context, data);
#else
            var result = Native.json_eval_reload_schema(_handle, Native.ToUTF8Bytes(schema), Native.ToUTF8Bytes(context), Native.ToUTF8Bytes(data));
#endif
            if (!result.Success)
            {
#if NETCOREAPP || NET5_0_OR_GREATER
                string error = result.Error != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(result.Error)
                    : "Unknown error";
#else
                string error = result.Error != IntPtr.Zero
                    ? Native.PtrToStringUTF8(result.Error)
                    : "Unknown error";
#endif
                Native.json_eval_free_result(result);
                throw new JsonEvalException(error);
            }
            Native.json_eval_free_result(result);
        }

        /// <summary>
        /// Gets cache statistics
        /// </summary>
        /// <returns>Cache statistics</returns>
        public CacheStats GetCacheStats()
        {
            ThrowIfDisposed();
            var result = Native.json_eval_cache_stats(_handle);
            var jsonResult = ProcessResult(result);
            return jsonResult.ToObject<CacheStats>();
        }

        /// <summary>
        /// Clears the evaluation cache
        /// </summary>
        public void ClearCache()
        {
            ThrowIfDisposed();
            var result = Native.json_eval_clear_cache(_handle);
            if (!result.Success)
            {
#if NETCOREAPP || NET5_0_OR_GREATER
                string error = result.Error != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(result.Error)
                    : "Unknown error";
#else
                string error = result.Error != IntPtr.Zero
                    ? Native.PtrToStringUTF8(result.Error)
                    : "Unknown error";
#endif
                Native.json_eval_free_result(result);
                throw new JsonEvalException(error);
            }
            Native.json_eval_free_result(result);
        }

        /// <summary>
        /// Gets the number of cached entries
        /// </summary>
        /// <returns>Number of cached entries</returns>
        public int GetCacheLength()
        {
            ThrowIfDisposed();
            var result = Native.json_eval_cache_len(_handle);
            try
            {
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error)
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error)
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }

                if (result.Data == IntPtr.Zero)
                    throw new JsonEvalException("No data returned from native function");

#if NETCOREAPP || NET5_0_OR_GREATER
                string lenStr = Marshal.PtrToStringUTF8(result.Data);
#else
                string lenStr = Native.PtrToStringUTF8(result.Data);
#endif
                return int.Parse(lenStr);
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        /// <summary>
        /// Validates data against schema rules with optional path filtering
        /// </summary>
        /// <param name="data">JSON data string</param>
        /// <param name="context">Optional context data</param>
        /// <param name="paths">Optional list of paths to validate (null for all)</param>
        /// <returns>ValidationResult</returns>
        public ValidationResult ValidatePaths(string data, string? context = null, List<string>? paths = null)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(data))
                throw new ArgumentNullException(nameof(data));

            string pathsJson = paths != null ? JsonConvert.SerializeObject(paths) : null;
#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_validate_paths(_handle, data, context, pathsJson);
#else
            var result = Native.json_eval_validate_paths(_handle, Native.ToUTF8Bytes(data), Native.ToUTF8Bytes(context), Native.ToUTF8Bytes(pathsJson));
#endif
            var jsonResult = ProcessResult(result);
            return jsonResult.ToObject<ValidationResult>();
        }

        private JObject ProcessResult(Native.FFIResult result)
        {
            try
            {
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error)
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error)
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }

                if (result.Data == IntPtr.Zero)
                    throw new JsonEvalException("No data returned from native function");

#if NETCOREAPP || NET5_0_OR_GREATER
                string json = Marshal.PtrToStringUTF8(result.Data);
#else
                string json = Native.PtrToStringUTF8(result.Data);
#endif
                
                if (string.IsNullOrWhiteSpace(json))
                    throw new JsonEvalException("Empty JSON returned from native function");
                
                // Debug: Log the JSON to help diagnose issues
                System.Diagnostics.Debug.WriteLine($"C# received JSON: {json}");
                
                return JObject.Parse(json);
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(JSONEval));
        }

        /// <summary>
        /// Releases native resources
        /// </summary>
        public void Dispose()
        {
            if (_disposed)
                return;

            if (_handle != IntPtr.Zero)
            {
                Native.json_eval_free(_handle);
                _handle = IntPtr.Zero;
            }

            _disposed = true;
            GC.SuppressFinalize(this);
        }

        ~JSONEval()
        {
            Dispose();
        }
    }
}
