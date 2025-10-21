using System;
using System.Runtime.InteropServices;

namespace JsonEvalRs
{
    /// <summary>
    /// Common P/Invoke declarations shared across all platforms
    /// </summary>
    internal static partial class Native
    {
        private const string LibName = "json_eval_rs";

        [StructLayout(LayoutKind.Sequential)]
        internal struct FFIResult
        {
            [MarshalAs(UnmanagedType.I1)]
            public bool Success;
            public IntPtr DataPtr;
            public UIntPtr DataLen;
            public IntPtr Error;
            public IntPtr OwnedData;
        }

        // Common FFI functions (no string marshalling differences)
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void json_eval_free_result(FFIResult result);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void json_eval_free_string(IntPtr ptr);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void json_eval_free(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr json_eval_version();

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema(
            IntPtr handle,
            [MarshalAs(UnmanagedType.I1)] bool skipLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_schema_value(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema_without_params(
            IntPtr handle,
            [MarshalAs(UnmanagedType.I1)] bool skipLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema_msgpack(
            IntPtr handle,
            [MarshalAs(UnmanagedType.I1)] bool skipLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_cache_stats(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_clear_cache(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_cache_len(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_subform_paths(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_resolve_layout(
            IntPtr handle,
            [MarshalAs(UnmanagedType.I1)] bool evaluate
        );
    }
}
