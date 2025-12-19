using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Collections.Generic;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace JsonEvalRs
{
    /// <summary>
    /// Subform methods for JSONEval class
    /// </summary>
    public partial class JSONEval
    {
        // ============================================================================
        // Subform Methods
        // ============================================================================

        /// <summary>
        /// Evaluate a subform with data
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <param name="data">JSON data string for the subform</param>
        /// <param name="context">Optional context data JSON string</param>
        /// <param name="paths">Optional list of paths for selective evaluation</param>
        public void EvaluateSubform(string subformPath, string data, string? context = null, IEnumerable<string> paths = null)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));
            if (string.IsNullOrEmpty(data))
                throw new ArgumentNullException(nameof(data));

            string pathsJson = paths != null ? JsonConvert.SerializeObject(paths) : null;

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_evaluate_subform(_handle, subformPath, data, context, pathsJson);
#else
            var result = Native.json_eval_evaluate_subform(_handle, Native.ToUTF8Bytes(subformPath)!, Native.ToUTF8Bytes(data)!, Native.ToUTF8Bytes(context), Native.ToUTF8Bytes(pathsJson));
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
        /// Validate subform data against its schema rules
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <param name="data">JSON data string for the subform</param>
        /// <param name="context">Optional context data JSON string</param>
        /// <returns>Validation result with errors if any</returns>
        public ValidationResult ValidateSubform(string subformPath, string data, string? context = null)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));
            if (string.IsNullOrEmpty(data))
                throw new ArgumentNullException(nameof(data));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_validate_subform(_handle, subformPath, data, context);
#else
            var result = Native.json_eval_validate_subform(_handle, Native.ToUTF8Bytes(subformPath)!, Native.ToUTF8Bytes(data)!, Native.ToUTF8Bytes(context));
#endif
            
            return ProcessResult<ValidationResult>(result);
        }

        /// <summary>
        /// Evaluate dependents in subform when a field changes
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <param name="changedPath">Path of the field that changed</param>
        /// <param name="data">Optional updated JSON data string</param>
        /// <param name="context">Optional context data JSON string</param>
        /// <returns>Array of dependent change objects</returns>
        public JArray EvaluateDependentsSubform(string subformPath, string changedPath, string? data = null, string? context = null)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));
            if (string.IsNullOrEmpty(changedPath))
                throw new ArgumentNullException(nameof(changedPath));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_evaluate_dependents_subform(_handle, subformPath, changedPath, data, context);
#else
            var result = Native.json_eval_evaluate_dependents_subform(_handle, Native.ToUTF8Bytes(subformPath)!, Native.ToUTF8Bytes(changedPath)!, Native.ToUTF8Bytes(data), Native.ToUTF8Bytes(context));
#endif
            
            return ProcessResultAsArray(result);
        }

        /// <summary>
        /// Resolve layout for subform
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <param name="evaluate">If true, runs evaluation before resolving layout</param>
        public void ResolveLayoutSubform(string subformPath, bool evaluate = false)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_resolve_layout_subform(_handle, subformPath, evaluate);
#else
            var result = Native.json_eval_resolve_layout_subform(_handle, Native.ToUTF8Bytes(subformPath)!, evaluate);
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
        /// Get evaluated schema from subform
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <param name="resolveLayout">Whether to resolve layout</param>
        /// <returns>Evaluated schema as JObject</returns>
        public JObject GetEvaluatedSchemaSubform(string subformPath, bool resolveLayout = false)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_evaluated_schema_subform(_handle, subformPath, resolveLayout);
#else
            var result = Native.json_eval_get_evaluated_schema_subform(_handle, Native.ToUTF8Bytes(subformPath)!, resolveLayout);
#endif
            
            return ProcessResult(result);
        }

        /// <summary>
        /// Get schema value from subform (all .value fields)
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <returns>Modified data as JObject</returns>
        public JObject GetSchemaValueSubform(string subformPath)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_schema_value_subform(_handle, subformPath);
#else
            var result = Native.json_eval_get_schema_value_subform(_handle, Native.ToUTF8Bytes(subformPath)!);
#endif
            
            return ProcessResult(result);
        }

        /// <summary>
        /// Get evaluated schema without $params from subform
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <param name="resolveLayout">Whether to resolve layout</param>
        /// <returns>Evaluated schema as JObject</returns>
        public JObject GetEvaluatedSchemaWithoutParamsSubform(string subformPath, bool resolveLayout = false)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_evaluated_schema_without_params_subform(_handle, subformPath, resolveLayout);
#else
            var result = Native.json_eval_get_evaluated_schema_without_params_subform(_handle, Native.ToUTF8Bytes(subformPath)!, resolveLayout);
#endif
            
            return ProcessResult(result);
        }

        /// <summary>
        /// Get evaluated schema by specific path from subform
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <param name="schemaPath">Dotted path to the value within the subform</param>
        /// <param name="skipLayout">Whether to skip layout resolution</param>
        /// <returns>Value as JObject or null if not found</returns>
        public JObject? GetEvaluatedSchemaByPathSubform(string subformPath, string schemaPath, bool skipLayout = false)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));
            if (string.IsNullOrEmpty(schemaPath))
                throw new ArgumentNullException(nameof(schemaPath));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_evaluated_schema_by_path_subform(_handle, subformPath, schemaPath, skipLayout);
#else
            var result = Native.json_eval_get_evaluated_schema_by_path_subform(_handle, Native.ToUTF8Bytes(subformPath)!, Native.ToUTF8Bytes(schemaPath)!, skipLayout);
#endif
            
            try
            {
                if (!result.Success)
                {
                    // Path not found - return null
                    return null;
                }

                if (result.DataPtr == IntPtr.Zero)
                    return null;

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    return null;

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

        /// <summary>
        /// Gets evaluated schema values by multiple paths from subform
        /// Returns data in the specified format. Skips paths that are not found.
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <param name="schemaPaths">Array of dotted paths to retrieve within the subform</param>
        /// <param name="skipLayout">Whether to skip layout resolution</param>
        /// <param name="format">Return format: Nested (default), Flat, or Array</param>
        /// <returns>Data in the specified format (JObject for Nested/Flat, JArray for Array)</returns>
        public JToken GetEvaluatedSchemaByPathsSubform(string subformPath, string[] schemaPaths, bool skipLayout = false, ReturnFormat format = ReturnFormat.Nested)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));
            if (schemaPaths == null || schemaPaths.Length == 0)
                throw new ArgumentNullException(nameof(schemaPaths));

            string pathsJson = JsonConvert.SerializeObject(schemaPaths);

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_evaluated_schema_by_paths_subform(_handle, subformPath, pathsJson, skipLayout, (byte)format);
#else
            var result = Native.json_eval_get_evaluated_schema_by_paths_subform(_handle, Native.ToUTF8Bytes(subformPath)!, Native.ToUTF8Bytes(pathsJson)!, skipLayout, (byte)format);
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
                throw new InvalidOperationException($"Failed to get evaluated schema by paths from subform: {error}");
            }

            try
            {
                if (result.DataPtr == IntPtr.Zero)
                    return format == ReturnFormat.Array ? (JToken)new JArray() : (JToken)new JObject();

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    return format == ReturnFormat.Array ? (JToken)new JArray() : (JToken)new JObject();

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
        /// Get list of available subform paths
        /// </summary>
        /// <returns>Array of subform paths</returns>
        public List<string> GetSubformPaths()
        {
            ThrowIfDisposed();
            var result = Native.json_eval_get_subform_paths(_handle);
            var array = ProcessResultAsArray(result);
            return array.ToObject<List<string>>() ?? new List<string>();
        }

        /// <summary>
        /// Check if a subform exists at the given path
        /// </summary>
        /// <param name="subformPath">Path to check</param>
        /// <returns>True if subform exists, false otherwise</returns>
        public bool HasSubform(string subformPath)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_has_subform(_handle, subformPath);
#else
            var result = Native.json_eval_has_subform(_handle, Native.ToUTF8Bytes(subformPath)!);
#endif
            
            try
            {
                if (!result.Success)
                    return false;

                if (result.DataPtr == IntPtr.Zero)
                    return false;

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    return false;

                byte[] buffer = new byte[dataLen];
                Marshal.Copy(result.DataPtr, buffer, 0, dataLen);
                string value = Encoding.UTF8.GetString(buffer);
                
                return value == "true";
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        /// <summary>
        /// Gets schema value by specific path from subform
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <param name="schemaPath">Dotted path to the value within the subform</param>
        /// <returns>Value as JObject or null if not found</returns>
        public JObject? GetSchemaByPathSubform(string subformPath, string schemaPath)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));
            if (string.IsNullOrEmpty(schemaPath))
                throw new ArgumentNullException(nameof(schemaPath));

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_schema_by_path_subform(_handle, subformPath, schemaPath);
#else
            var result = Native.json_eval_get_schema_by_path_subform(_handle, Native.ToUTF8Bytes(subformPath)!, Native.ToUTF8Bytes(schemaPath)!);
#endif
            
            try
            {
                if (!result.Success)
                {
                    // Path not found - return null
                    return null;
                }

                if (result.DataPtr == IntPtr.Zero)
                    return null;

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    return null;

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

        /// <summary>
        /// Gets schema values by multiple paths from subform
        /// Returns data in the specified format. Skips paths that are not found.
        /// </summary>
        /// <param name="subformPath">Path to the subform</param>
        /// <param name="schemaPaths">Array of dotted paths to retrieve within the subform</param>
        /// <param name="format">Return format: Nested (default), Flat, or Array</param>
        /// <returns>Data in the specified format (JObject for Nested/Flat, JArray for Array)</returns>
        public JToken GetSchemaByPathsSubform(string subformPath, string[] schemaPaths, ReturnFormat format = ReturnFormat.Nested)
        {
            ThrowIfDisposed();
            if (string.IsNullOrEmpty(subformPath))
                throw new ArgumentNullException(nameof(subformPath));
            if (schemaPaths == null || schemaPaths.Length == 0)
                throw new ArgumentNullException(nameof(schemaPaths));

            string pathsJson = JsonConvert.SerializeObject(schemaPaths);

#if NETCOREAPP || NET5_0_OR_GREATER
            var result = Native.json_eval_get_schema_by_paths_subform(_handle, subformPath, pathsJson, (byte)format);
#else
            var result = Native.json_eval_get_schema_by_paths_subform(_handle, Native.ToUTF8Bytes(subformPath)!, Native.ToUTF8Bytes(pathsJson)!, (byte)format);
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
                throw new InvalidOperationException($"Failed to get schema by paths from subform: {error}");
            }

            try
            {
                if (result.DataPtr == IntPtr.Zero)
                    return format == ReturnFormat.Array ? (JToken)new JArray() : (JToken)new JObject();

                int dataLen = (int)result.DataLen.ToUInt32();
                if (dataLen == 0)
                    return format == ReturnFormat.Array ? (JToken)new JArray() : (JToken)new JObject();

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
    }
}
