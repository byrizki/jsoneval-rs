using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace JsonEvalRs
{
    /// <summary>
    /// Thread-safe cache for pre-parsed schemas
    /// 
    /// ParsedSchemaCache stores compiled schema instances that can be reused across multiple
    /// JSONEval instances, significantly improving performance for applications that use
    /// the same schemas repeatedly.
    /// </summary>
    public class ParsedSchemaCache : IDisposable
    {
        private IntPtr _handle;
        private bool _disposed;
        private readonly bool _isGlobal;

        /// <summary>
        /// Creates a new local cache instance
        /// </summary>
        public ParsedSchemaCache()
        {
            _handle = Native.parsed_cache_new();
            _isGlobal = false;
        }

        /// <summary>
        /// Gets the global singleton cache instance
        /// </summary>
        public static ParsedSchemaCache Global { get; } = new ParsedSchemaCache(true);

        private ParsedSchemaCache(bool isGlobal)
        {
            _handle = Native.parsed_cache_global();
            _isGlobal = isGlobal;
        }

        /// <summary>
        /// Parse and insert a schema into the cache
        /// </summary>
        /// <param name="key">Unique key for the schema</param>
        /// <param name="schemaJson">JSON schema string</param>
        public void Insert(string key, string schemaJson)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(key))
                throw new ArgumentNullException(nameof(key));
            if (string.IsNullOrEmpty(schemaJson))
                throw new ArgumentNullException(nameof(schemaJson));

            Native.FFIResult result;
#if NETCOREAPP || NET5_0_OR_GREATER
            result = Native.parsed_cache_insert(_handle, key, schemaJson);
#else
            result = Native.parsed_cache_insert(
                _handle,
                Native.ToUTF8Bytes(key)!,
                Native.ToUTF8Bytes(schemaJson)!
            );
#endif

            if (!result.Success)
            {
                string error = "Failed to insert schema into cache";
                if (result.Error != IntPtr.Zero)
                {
                    try
                    {
#if NETCOREAPP || NET5_0_OR_GREATER
                        error = Marshal.PtrToStringUTF8(result.Error) ?? error;
#else
                        error = Native.PtrToStringUTF8(result.Error) ?? error;
#endif
                    }
                    finally
                    {
                        Native.json_eval_free_string(result.Error);
                    }
                }
                throw new JsonEvalException(error);
            }

            Native.json_eval_free_result(result);
        }

        /// <summary>
        /// Parse and insert a schema from MessagePack into the cache
        /// </summary>
        /// <param name="key">Unique key for the schema</param>
        /// <param name="schemaMsgpack">MessagePack-encoded schema bytes</param>
        public void InsertMsgpack(string key, byte[] schemaMsgpack)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(key))
                throw new ArgumentNullException(nameof(key));
            if (schemaMsgpack == null || schemaMsgpack.Length == 0)
                throw new ArgumentNullException(nameof(schemaMsgpack));

            var keyBytes = Encoding.UTF8.GetBytes(key + "\0");

            // Pin the array and get pointer
            var handle = GCHandle.Alloc(schemaMsgpack, GCHandleType.Pinned);
            try
            {
                IntPtr ptr = handle.AddrOfPinnedObject();
                var result = Native.parsed_cache_insert_msgpack(
                    _handle,
                    keyBytes,
                    ptr,
                    (UIntPtr)schemaMsgpack.Length
                );

                if (!result.Success)
                {
                    string error = "Failed to insert MessagePack schema into cache";
                    if (result.Error != IntPtr.Zero)
                    {
                        try
                        {
#if NETCOREAPP || NET5_0_OR_GREATER
                            error = Marshal.PtrToStringUTF8(result.Error) ?? error;
#else
                            error = Native.PtrToStringUTF8(result.Error) ?? error;
#endif
                        }
                        finally
                        {
                            Native.json_eval_free_string(result.Error);
                        }
                    }
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
        /// Check if a key exists in the cache
        /// </summary>
        /// <param name="key">Key to check</param>
        /// <returns>True if the key exists, false otherwise</returns>
        public bool Contains(string key)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(key))
                return false;

#if NETCOREAPP || NET5_0_OR_GREATER
            return Native.parsed_cache_contains(_handle, key) != 0;
#else
            return Native.parsed_cache_contains(_handle, Native.ToUTF8Bytes(key)!) != 0;
#endif
        }

        /// <summary>
        /// Remove a schema from the cache
        /// </summary>
        /// <param name="key">Key to remove</param>
        /// <returns>True if removed, false if key not found</returns>
        public bool Remove(string key)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(key))
                return false;

#if NETCOREAPP || NET5_0_OR_GREATER
            return Native.parsed_cache_remove(_handle, key) != 0;
#else
            return Native.parsed_cache_remove(_handle, Native.ToUTF8Bytes(key)!) != 0;
#endif
        }

        /// <summary>
        /// Clear all entries from the cache
        /// </summary>
        public void Clear()
        {
            ThrowIfDisposed();
            Native.parsed_cache_clear(_handle);
        }

        /// <summary>
        /// Get the number of entries in the cache
        /// </summary>
        public int Count
        {
            get
            {
                ThrowIfDisposed();
                return (int)Native.parsed_cache_len(_handle);
            }
        }

        /// <summary>
        /// Check if the cache is empty
        /// </summary>
        public bool IsEmpty
        {
            get
            {
                ThrowIfDisposed();
                return Native.parsed_cache_is_empty(_handle) != 0;
            }
        }

        /// <summary>
        /// Get cache statistics
        /// </summary>
        /// <returns>Cache statistics object</returns>
        public ParsedCacheStats GetStats()
        {
            ThrowIfDisposed();

            var result = Native.parsed_cache_stats(_handle);
            try
            {
                if (!result.Success)
                {
                    throw new JsonEvalException("Failed to get cache stats");
                }

                if (result.DataPtr == IntPtr.Zero || result.DataLen == UIntPtr.Zero)
                {
                    return new ParsedCacheStats { EntryCount = 0, Keys = new List<string>() };
                }

                byte[] data = new byte[(int)result.DataLen];
                Marshal.Copy(result.DataPtr, data, 0, data.Length);
                string json = Encoding.UTF8.GetString(data);

                var jobj = JObject.Parse(json);
                return new ParsedCacheStats
                {
                    EntryCount = jobj["entry_count"]?.Value<int>() ?? 0,
                    Keys = jobj["keys"]?.ToObject<List<string>>() ?? new List<string>()
                };
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        /// <summary>
        /// Get all keys in the cache
        /// </summary>
        /// <returns>List of all keys</returns>
        public List<string> GetKeys()
        {
            ThrowIfDisposed();

            var result = Native.parsed_cache_keys(_handle);
            try
            {
                if (!result.Success)
                {
                    throw new JsonEvalException("Failed to get cache keys");
                }

                if (result.DataPtr == IntPtr.Zero || result.DataLen == UIntPtr.Zero)
                {
                    return new List<string>();
                }

                byte[] data = new byte[(int)result.DataLen];
                Marshal.Copy(result.DataPtr, data, 0, data.Length);
                string json = Encoding.UTF8.GetString(data);

                return JsonConvert.DeserializeObject<List<string>>(json) ?? new List<string>();
            }
            finally
            {
                Native.json_eval_free_result(result);
            }
        }

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(ParsedSchemaCache));
        }

        /// <summary>
        /// Dispose of the cache instance
        /// Note: Do not dispose the Global instance!
        /// </summary>
        public void Dispose()
        {
            if (_disposed)
                return;

            _disposed = true;

            // Only free non-global instances
            if (!_isGlobal && _handle != IntPtr.Zero)
            {
                Native.parsed_cache_free(_handle);
                _handle = IntPtr.Zero;
            }

            GC.SuppressFinalize(this);
        }

        ~ParsedSchemaCache()
        {
            Dispose();
        }
    }

    /// <summary>
    /// Statistics about the ParsedSchemaCache state
    /// </summary>
    public class ParsedCacheStats
    {
        /// <summary>
        /// Number of entries in the cache
        /// </summary>
        public int EntryCount { get; set; }

        /// <summary>
        /// List of all keys in the cache
        /// </summary>
        public List<string> Keys { get; set; } = new List<string>();

        public override string ToString()
        {
            return $"ParsedSchemaCache: {EntryCount} entries" +
                   (Keys.Count > 0 ? $" (keys: {string.Join(", ", Keys)})" : "");
        }
    }
}
