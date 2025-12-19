using System;
using System.Runtime.InteropServices;
using System.Text;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace JsonEvalRs
{
    /// <summary>
    /// High-performance JSON Logic evaluator with schema validation
    /// 
    /// This file contains the platform-agnostic JSONEval class implementation.
    /// Platform-specific Native FFI declarations are in:
    /// - JsonEvalRs.Native.Common.cs (shared)
    /// - JsonEvalRs.Native.NetCore.cs (.NET Core/.NET 5+)
    /// - JsonEvalRs.Native.NetStandard.cs (.NET Standard 2.0)
    /// 
    /// Subform methods are in JsonEvalRs.Subforms.cs
    /// </summary>
    public partial class JSONEval : IDisposable
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
                // Version string is static in Rust, no need to free it
#if NETCOREAPP || NET5_0_OR_GREATER
                return Marshal.PtrToStringUTF8(ptr) ?? "unknown";
#else
                return Native.PtrToStringUTF8(ptr) ?? "unknown";
#endif
            }
        }

        /// <summary>
        /// Creates a new JSON evaluator instance from a cached ParsedSchema
        /// </summary>
        /// <param name="cacheKey">Cache key to lookup in the global ParsedSchemaCache</param>
        /// <param name="context">Optional context data (can be null)</param>
        /// <param name="data">Optional initial data (can be null)</param>
        /// <returns>New JSONEval instance using the cached schema</returns>
        /// <exception cref="ArgumentNullException">If cacheKey is null or empty</exception>
        /// <exception cref="JsonEvalException">If schema not found in cache or creation fails</exception>
        public static JSONEval FromCache(string cacheKey, string? context = null, string? data = null)
        {
            if (string.IsNullOrEmpty(cacheKey))
                throw new ArgumentNullException(nameof(cacheKey));

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
            IntPtr handle;
#if NETCOREAPP || NET5_0_OR_GREATER
            handle = Native.json_eval_new_from_cache_with_error(cacheKey, context, data, out errorPtr);
#else
            handle = Native.json_eval_new_from_cache_with_error(
                Native.ToUTF8Bytes(cacheKey)!,
                Native.ToUTF8Bytes(context),
                Native.ToUTF8Bytes(data),
                out errorPtr
            );
#endif
            
            if (handle == IntPtr.Zero)
            {
                string errorMessage = $"Failed to create JSONEval from cache key '{cacheKey}'";
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

            // Create instance with the handle
            return new JSONEval(handle);
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
        /// Creates a new JSON evaluator instance from MessagePack-encoded schema
        /// </summary>
        /// <param name="schemaMsgpack">MessagePack-encoded schema bytes</param>
        /// <param name="context">Optional context data (can be null)</param>
        /// <param name="data">Optional initial data (can be null)</param>
        public JSONEval(byte[] schemaMsgpack, string? context = null, string? data = null)
        {
            if (schemaMsgpack == null || schemaMsgpack.Length == 0)
                throw new ArgumentNullException(nameof(schemaMsgpack));

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

            // Pin the byte array and get a pointer to it
            GCHandle handle = GCHandle.Alloc(schemaMsgpack, GCHandleType.Pinned);
            try
            {
                IntPtr schemaPtr = handle.AddrOfPinnedObject();
#if NETCOREAPP || NET5_0_OR_GREATER
                _handle = Native.json_eval_new_from_msgpack(
                    schemaPtr,
                    (UIntPtr)schemaMsgpack.Length,
                    context,
                    data
                );
#else
                _handle = Native.json_eval_new_from_msgpack(
                    schemaPtr,
                    (UIntPtr)schemaMsgpack.Length,
                    Native.ToUTF8Bytes(context),
                    Native.ToUTF8Bytes(data)
                );
#endif
            }
            finally
            {
                handle.Free();
            }
            
            if (_handle == IntPtr.Zero)
            {
                throw new JsonEvalException("Failed to create JSONEval instance from MessagePack schema");
            }
        }

        /// <summary>
        /// Private constructor that wraps an existing handle
        /// Used by static factory methods like FromCache()
        /// </summary>
        private JSONEval(IntPtr handle)
        {
            _handle = handle;
        }

        /// <summary>
        /// Evaluates the schema with provided data
        /// </summary>
        /// <param name="data">JSON data string</param>
        /// <param name="context">Optional context data</param>
        public void Evaluate(string data, string? context = null)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(data))
                throw new ArgumentNullException(nameof(data));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_evaluate(_handle, data, context, null);
#else
            var result = Native.json_eval_evaluate(_handle, Native.ToUTF8Bytes(data), Native.ToUTF8Bytes(context), null);
#endif
            
            // Check for errors but don't return data (massive performance optimization)
            if (!result.Success)
            {
#if NETCOREAPP || NET5_0_OR_GREATER
                string error = result.Error != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#else
                string error = result.Error != IntPtr.Zero
                    ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#endif
                Native.json_eval_free_result(result);
                throw new JsonEvalException(error);
            }
            Native.json_eval_free_result(result);
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
            return ProcessResult<ValidationResult>(result);
        }

        /// <summary>
        /// Re-evaluates fields that depend on the changed paths (processes transitively)
        /// </summary>
        /// <param name="changedPaths">Array of field paths that changed (e.g., ["#/properties/field1", "field2"])</param>
        /// <param name="data">Optional updated JSON data string (null to use existing data)</param>
        /// <param name="context">Optional context data</param>
        /// <param name="reEvaluate">If true, performs full evaluation after processing dependents</param>
        /// <returns>Array of dependent change objects as JArray</returns>
        public JArray EvaluateDependents(string[] changedPaths, string? data = null, 
            string? context = null, bool reEvaluate = false)
        {
            ThrowIfDisposed();

            if (changedPaths == null || changedPaths.Length == 0)
                throw new ArgumentNullException(nameof(changedPaths));

            var changedPathsJson = JsonConvert.SerializeObject(changedPaths);

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_evaluate_dependents(_handle, changedPathsJson, data, context, reEvaluate ? 1 : 0);
#else
            var result = Native.json_eval_evaluate_dependents(_handle, Native.ToUTF8Bytes(changedPathsJson)!, Native.ToUTF8Bytes(data), Native.ToUTF8Bytes(context), reEvaluate ? 1 : 0);
#endif
            return ProcessResultAsArray(result);
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
        /// Gets the evaluated schema as MessagePack binary data
        /// </summary>
        /// <param name="skipLayout">Whether to skip layout resolution</param>
        /// <returns>Evaluated schema as MessagePack bytes</returns>
        public byte[] GetEvaluatedSchemaMsgpack(bool skipLayout = false)
        {
            ThrowIfDisposed();
            var result = Native.json_eval_get_evaluated_schema_msgpack(_handle, skipLayout);
            return ProcessResultAsBytes(result);
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
        /// Gets the evaluated schema without $params field
        /// </summary>
        /// <param name="skipLayout">Whether to skip layout resolution</param>
        /// <returns>Evaluated schema as JObject</returns>
        public JObject GetEvaluatedSchemaWithoutParams(bool skipLayout = false)
        {
            ThrowIfDisposed();
            var result = Native.json_eval_get_evaluated_schema_without_params(_handle, skipLayout);
            return ProcessResult(result);
        }

        /// <summary>
        /// Gets a value from the evaluated schema using dotted path notation
        /// </summary>
        /// <param name="path">Dotted path to the value (e.g., "properties.field.value")</param>
        /// <param name="skipLayout">Whether to skip layout resolution</param>
        /// <returns>Value as JToken, or null if not found</returns>
        public JToken? GetEvaluatedSchemaByPath(string path, bool skipLayout = false)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(path))
                throw new ArgumentNullException(nameof(path));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_evaluated_schema_by_path(_handle, path, skipLayout);
#else
            var result = Native.json_eval_get_evaluated_schema_by_path(_handle, Native.ToUTF8Bytes(path), skipLayout);
#endif
            
            if (!result.Success)
            {
                // Path not found - return null instead of throwing
                Native.json_eval_free_result(result);
                return null;
            }

            try
            {
                if (result.DataPtr == IntPtr.Zero)
                    return null;

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    return null;

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                string json = Encoding.UTF8.GetString(buffer);
                return JToken.Parse(json);
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        /// <summary>
        /// Gets values from the evaluated schema using multiple dotted path notations.
        /// Returns data in the specified format. Skips paths that are not found.
        /// </summary>
        /// <param name="paths">Array of dotted paths to retrieve (e.g., ["properties.field1", "properties.field2"])</param>
        /// <param name="skipLayout">Whether to skip layout resolution</param>
        /// <param name="format">Return format: Nested (default), Flat, or Array</param>
        /// <returns>Data in the specified format (JObject for Nested/Flat, JArray for Array)</returns>
        public JToken GetEvaluatedSchemaByPaths(string[] paths, bool skipLayout = false, ReturnFormat format = ReturnFormat.Nested)
        {
            ThrowIfDisposed();

            if (paths == null || paths.Length == 0)
                throw new ArgumentNullException(nameof(paths));

            string pathsJson = JsonConvert.SerializeObject(paths);

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_evaluated_schema_by_paths(_handle, pathsJson, skipLayout, (byte)format);
#else
            var result = Native.json_eval_get_evaluated_schema_by_paths(_handle, Native.ToUTF8Bytes(pathsJson)!, skipLayout, (byte)format);
#endif
            
            if (!result.Success)
            {
#if NETCOREAPP || NET5_0_OR_GREATER
                string error = result.Error != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#else
                string error = result.Error != IntPtr.Zero
                    ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#endif
                Native.json_eval_free_result(result);
                throw new InvalidOperationException($"Failed to get evaluated schema by paths: {error}");
            }

            try
            {
                if (result.DataPtr == IntPtr.Zero)
                    return format == ReturnFormat.Array ? (JToken)new JArray() : (JToken)new JObject();

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    return format == ReturnFormat.Array ? (JToken)new JArray() : (JToken)new JObject();

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                string json = Encoding.UTF8.GetString(buffer);
                return format == ReturnFormat.Array ? (JToken)JArray.Parse(json) : (JToken)JObject.Parse(json);
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        /// <summary>
        /// Gets a value from the schema using dotted path notation
        /// </summary>
        /// <param name="path">Dotted path to the value (e.g., "properties.field.value")</param>
        /// <returns>Value as JToken, or null if not found</returns>
        public JToken? GetSchemaByPath(string path)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(path))
                throw new ArgumentNullException(nameof(path));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_schema_by_path(_handle, path);
#else
            var result = Native.json_eval_get_schema_by_path(_handle, Native.ToUTF8Bytes(path));
#endif
            
            if (!result.Success)
            {
                // Path not found - return null instead of throwing
                Native.json_eval_free_result(result);
                return null;
            }

            try
            {
                if (result.DataPtr == IntPtr.Zero)
                    return null;

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    return null;

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                string json = Encoding.UTF8.GetString(buffer);
                return JToken.Parse(json);
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        /// <summary>
        /// Gets values from the schema using multiple dotted path notations.
        /// Returns data in the specified format. Skips paths that are not found.
        /// </summary>
        /// <param name="paths">Array of dotted paths to retrieve (e.g., ["properties.field1", "properties.field2"])</param>
        /// <param name="format">Return format: Nested (default), Flat, or Array</param>
        /// <returns>Data in the specified format (JObject for Nested/Flat, JArray for Array)</returns>
        public JToken GetSchemaByPaths(string[] paths, ReturnFormat format = ReturnFormat.Nested)
        {
            ThrowIfDisposed();

            if (paths == null || paths.Length == 0)
                throw new ArgumentNullException(nameof(paths));

            string pathsJson = JsonConvert.SerializeObject(paths);

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_schema_by_paths(_handle, pathsJson, (byte)format);
#else
            var result = Native.json_eval_get_schema_by_paths(_handle, Native.ToUTF8Bytes(pathsJson)!, (byte)format);
#endif
            
            if (!result.Success)
            {
#if NETCOREAPP || NET5_0_OR_GREATER
                string error = result.Error != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#else
                string error = result.Error != IntPtr.Zero
                    ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#endif
                Native.json_eval_free_result(result);
                throw new InvalidOperationException($"Failed to get schema by paths: {error}");
            }

            try
            {
                if (result.DataPtr == IntPtr.Zero)
                    return format == ReturnFormat.Array ? (JToken)new JArray() : (JToken)new JObject();

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    return format == ReturnFormat.Array ? (JToken)new JArray() : (JToken)new JObject();

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                string json = Encoding.UTF8.GetString(buffer);
                return format == ReturnFormat.Array ? (JToken)JArray.Parse(json) : (JToken)JObject.Parse(json);
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
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
                    ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#else
                string error = result.Error != IntPtr.Zero
                    ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#endif
                Native.json_eval_free_result(result);
                throw new JsonEvalException(error);
            }
            Native.json_eval_free_result(result);
        }

        /// <summary>
        /// Reloads the schema from MessagePack-encoded bytes
        /// </summary>
        /// <param name="schemaMsgpack">MessagePack-encoded schema bytes</param>
        /// <param name="context">Optional context data</param>
        /// <param name="data">Optional initial data</param>
        public void ReloadSchemaMsgpack(byte[] schemaMsgpack, string? context = null, string? data = null)
        {
            ThrowIfDisposed();

            if (schemaMsgpack == null || schemaMsgpack.Length == 0)
                throw new ArgumentNullException(nameof(schemaMsgpack));

            // Pin the array and get pointer
            var handle = GCHandle.Alloc(schemaMsgpack, GCHandleType.Pinned);
            try
            {
                IntPtr ptr = handle.AddrOfPinnedObject();
                
#if NETCOREAPP || NET5_0_OR_GREATER
                var result = Native.json_eval_reload_schema_msgpack(_handle, ptr, (UIntPtr)schemaMsgpack.Length, context, data);
#else
                var result = Native.json_eval_reload_schema_msgpack(_handle, ptr, (UIntPtr)schemaMsgpack.Length, Native.ToUTF8Bytes(context), Native.ToUTF8Bytes(data));
#endif
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#endif
                    Native.json_eval_free_result(result);
                    throw new JsonEvalException(error);
                }
                Native.json_eval_free_result(result);
            }
            finally
            {
                handle.Free();
            }
        }

        /// <summary>
        /// Reloads the schema from ParsedSchemaCache using a cache key
        /// </summary>
        /// <param name="cacheKey">Cache key to lookup in ParsedSchemaCache</param>
        /// <param name="context">Optional context data</param>
        /// <param name="data">Optional initial data</param>
        public void ReloadSchemaFromCache(string cacheKey, string? context = null, string? data = null)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(cacheKey))
                throw new ArgumentNullException(nameof(cacheKey));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_reload_schema_from_cache(_handle, cacheKey, context, data);
#else
            var result = Native.json_eval_reload_schema_from_cache(_handle, Native.ToUTF8Bytes(cacheKey)!, Native.ToUTF8Bytes(context), Native.ToUTF8Bytes(data));
#endif
            if (!result.Success)
            {
#if NETCOREAPP || NET5_0_OR_GREATER
                string error = result.Error != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#else
                string error = result.Error != IntPtr.Zero
                    ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
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
            return jsonResult.ToObject<CacheStats>() ?? new CacheStats();
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
                    ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#else
                string error = result.Error != IntPtr.Zero
                    ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
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
                        ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }

                if (result.DataPtr == IntPtr.Zero)
                    throw new JsonEvalException("No data returned from native function");

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    throw new JsonEvalException("Invalid cache length returned from native function");

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                string lenStr = Encoding.UTF8.GetString(buffer);
                return int.Parse(lenStr);
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        /// <summary>
        /// Enable evaluation caching.
        /// Useful for reusing JSONEval instances with different data.
        /// </summary>
        public void EnableCache()
        {
            ThrowIfDisposed();
            var result = Native.json_eval_enable_cache(_handle);
            try
            {
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        /// <summary>
        /// Disable evaluation caching.
        /// Useful for web API usage where each request creates a new JSONEval instance.
        /// Improves performance by skipping cache operations that have no benefit for single-use instances.
        /// </summary>
        public void DisableCache()
        {
            ThrowIfDisposed();
            var result = Native.json_eval_disable_cache(_handle);
            try
            {
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        /// <summary>
        /// Check if evaluation caching is enabled.
        /// </summary>
        /// <returns>True if caching is enabled, false otherwise</returns>
        public bool IsCacheEnabled()
        {
            ThrowIfDisposed();
            return Native.json_eval_is_cache_enabled(_handle) != 0;
        }

        /// <summary>
        /// Resolves layout with optional evaluation
        /// </summary>
        /// <param name="evaluate">If true, runs evaluation before resolving layout</param>
        /// <throws>JsonEvalException if resolve fails</throws>
        public void ResolveLayout(bool evaluate = false)
        {
            ThrowIfDisposed();
            var result = Native.json_eval_resolve_layout(_handle, evaluate);
            if (!result.Success)
            {
#if NETCOREAPP || NET5_0_OR_GREATER
                string error = result.Error != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#else
                string error = result.Error != IntPtr.Zero
                    ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Unknown error";
#endif
                Native.json_eval_free_result(result);
                throw new JsonEvalException(error);
            }
            Native.json_eval_free_result(result);
        }

        /// <summary>
        /// Compiles and runs JSON logic from a JSON logic string
        /// </summary>
        /// <param name="logicStr">JSON logic expression as a string</param>
        /// <param name="data">Optional JSON data string (null to use existing data)</param>
        /// <param name="context">Optional context data string (null to use existing context)</param>
        /// <returns>Result as JToken</returns>
        /// <throws>JsonEvalException if compilation or evaluation fails</throws>
        public JToken CompileAndRunLogic(string logicStr, string? data = null, string? context = null)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(logicStr))
                throw new ArgumentNullException(nameof(logicStr));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_compile_and_run_logic(_handle, logicStr, data, context);
#else
            var logicBytes = Native.ToUTF8Bytes(logicStr)
                ?? throw new ArgumentNullException(nameof(logicStr));
            var result = Native.json_eval_compile_and_run_logic(
                _handle,
                logicBytes,
                Native.ToUTF8Bytes(data),
                Native.ToUTF8Bytes(context));
#endif
            
            try
            {
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }

                if (result.DataPtr == IntPtr.Zero)
                    throw new JsonEvalException("No data returned from native function");

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    throw new JsonEvalException("Empty result returned from native function");

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                string json = Encoding.UTF8.GetString(buffer);
                return JToken.Parse(json);
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        /// <summary>
        /// Compiles JSON logic and returns a global ID
        /// </summary>
        /// <param name="logicStr">JSON logic expression as a string</param>
        /// <returns>Compiled logic ID</returns>
        /// <throws>JsonEvalException if compilation fails or returns 0 (error)</throws>
        public ulong CompileLogic(string logicStr)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(logicStr))
                throw new ArgumentNullException(nameof(logicStr));

#if NETCOREAPP || NET5_0_OR_GREATER
            ulong logicId = Native.json_eval_compile_logic(_handle, logicStr);
#else
            var logicBytes = Native.ToUTF8Bytes(logicStr)
                ?? throw new ArgumentNullException(nameof(logicStr));
            ulong logicId = Native.json_eval_compile_logic(_handle, logicBytes);
#endif

            if (logicId == 0)
            {
                throw new JsonEvalException("Failed to compile logic (returned ID 0)");
            }

            return logicId;
        }

        /// <summary>
        /// Runs pre-compiled logic by ID
        /// </summary>
        /// <param name="logicId">Compiled logic ID from CompileLogic</param>
        /// <param name="data">Optional JSON data string (null to use existing data)</param>
        /// <param name="context">Optional context data string (null to use existing context)</param>
        /// <returns>Result as JToken</returns>
        /// <throws>JsonEvalException if execution fails</throws>
        public JToken RunLogic(ulong logicId, string? data = null, string? context = null)
        {
            ThrowIfDisposed();

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_run_logic(_handle, logicId, data, context);
#else
            var result = Native.json_eval_run_logic(_handle, logicId, Native.ToUTF8Bytes(data), Native.ToUTF8Bytes(context));
#endif
            
            try
            {
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }

                if (result.DataPtr == IntPtr.Zero)
                    throw new JsonEvalException("No data returned from native function");

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    throw new JsonEvalException("Empty result returned from native function");

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                string json = Encoding.UTF8.GetString(buffer);
                return JToken.Parse(json);
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
        public ValidationResult ValidatePaths(string data, string? context = null, System.Collections.Generic.List<string>? paths = null)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(data))
                throw new ArgumentNullException(nameof(data));

            string? pathsJson = paths != null ? JsonConvert.SerializeObject(paths) : null;
#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_validate_paths(_handle, data, context, pathsJson);
#else
            var result = Native.json_eval_validate_paths(_handle, Native.ToUTF8Bytes(data), Native.ToUTF8Bytes(context), Native.ToUTF8Bytes(pathsJson));
#endif
            return ProcessResult<ValidationResult>(result);
        }

        // Helper methods for processing results
        private JObject ProcessResult(Native.FFIResult result)
        {
            try
            {
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }

                if (result.DataPtr == IntPtr.Zero)
                    throw new JsonEvalException("No data returned from native function");

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    throw new JsonEvalException("Empty JSON returned from native function");

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                string json = Encoding.UTF8.GetString(buffer);
                return JObject.Parse(json);
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        private JArray ProcessResultAsArray(Native.FFIResult result)
        {
            try
            {
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }

                if (result.DataPtr == IntPtr.Zero)
                    throw new JsonEvalException("No data returned from native function");

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    throw new JsonEvalException("Empty JSON returned from native function");

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                string json = Encoding.UTF8.GetString(buffer);
                return JArray.Parse(json);
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        private T ProcessResult<T>(Native.FFIResult result) where T : class
        {
            try
            {
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }

                if (result.DataPtr == IntPtr.Zero)
                    throw new JsonEvalException("No data returned from native function");

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    throw new JsonEvalException("Empty JSON returned from native function");

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                string json = Encoding.UTF8.GetString(buffer);
                var obj = JObject.Parse(json);
                return obj.ToObject<T>() ?? throw new JsonEvalException("Failed to deserialize result");
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        private byte[] ProcessResultAsBytes(Native.FFIResult result)
        {
            try
            {
                if (!result.Success)
                {
#if NETCOREAPP || NET5_0_OR_GREATER
                    string error = result.Error != IntPtr.Zero
                        ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#else
                    string error = result.Error != IntPtr.Zero
                        ? Native.PtrToStringUTF8(result.Error) ?? "Unknown error"
                        : "Unknown error";
#endif
                    throw new JsonEvalException(error);
                }

                if (result.DataPtr == IntPtr.Zero)
                    throw new JsonEvalException("No data returned from native function");

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    throw new JsonEvalException("Empty data returned from native function");

                // Zero-copy: read directly from Rust-owned memory
                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                
                return buffer;
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
