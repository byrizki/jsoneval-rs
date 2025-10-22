using System;
using System.Runtime.InteropServices;

namespace JsonEvalRs
{
    /// <summary>
    /// P/Invoke declarations for ParsedSchemaCache FFI functions
    /// </summary>
    internal static partial class Native
    {
        // ParsedSchemaCache creation and management
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr parsed_cache_new();

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr parsed_cache_global();

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void parsed_cache_free(IntPtr handle);

        // Cache operations - .NET Core/.NET 5+
#if NETCOREAPP || NET5_0_OR_GREATER
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern FFIResult parsed_cache_insert(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string key,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string schemaJson
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern IntPtr parsed_cache_get(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string key
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern int parsed_cache_contains(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string key
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern int parsed_cache_remove(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string key
        );
#else
        // .NET Standard 2.0 - use byte arrays
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult parsed_cache_insert(
            IntPtr handle,
            byte[] key,
            byte[] schemaJson
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr parsed_cache_get(
            IntPtr handle,
            byte[] key
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int parsed_cache_contains(
            IntPtr handle,
            byte[] key
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int parsed_cache_remove(
            IntPtr handle,
            byte[] key
        );
#endif

        // MessagePack support
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult parsed_cache_insert_msgpack(
            IntPtr handle,
            byte[] key,
            IntPtr schemaMsgpack,
            UIntPtr schemaLen
        );

        // Cache info functions
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void parsed_cache_clear(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern UIntPtr parsed_cache_len(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int parsed_cache_is_empty(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult parsed_cache_stats(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult parsed_cache_keys(IntPtr handle);
    }
}
