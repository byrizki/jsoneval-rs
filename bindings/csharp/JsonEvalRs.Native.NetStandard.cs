#if !NETCOREAPP && !NET5_0_OR_GREATER
using System;
using System.Runtime.InteropServices;
using System.Text;

namespace JsonEvalRs
{
    /// <summary>
    /// .NET Standard 2.0/2.1 specific P/Invoke declarations
    /// Uses byte array marshalling for compatibility
    /// </summary>
    internal static partial class Native
    {
        // Helper methods for byte array marshalling
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

        // Core FFI functions with byte array marshalling
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
        internal static extern IntPtr json_eval_new_from_msgpack(
            IntPtr schemaMsgpack,
            UIntPtr schemaLen,
            byte[]? context,
            byte[]? data
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
            byte[]? changedPath,
            byte[]? data,
            byte[]? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema_by_path(
            IntPtr handle,
            byte[]? path,
            [MarshalAs(UnmanagedType.I1)] bool skipLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_reload_schema(
            IntPtr handle,
            byte[]? schema,
            byte[]? context,
            byte[]? data
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_reload_schema_msgpack(
            IntPtr handle,
            IntPtr schemaMsgpack,
            UIntPtr schemaLen,
            byte[]? context,
            byte[]? data
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_reload_schema_from_cache(
            IntPtr handle,
            byte[]? cacheKey,
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

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_compile_and_run_logic(
            IntPtr handle,
            byte[]? logicStr,
            byte[]? data
        );

        // Subform methods
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_evaluate_subform(
            IntPtr handle,
            byte[] subformPath,
            byte[] data,
            byte[]? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_validate_subform(
            IntPtr handle,
            byte[] subformPath,
            byte[] data,
            byte[]? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_evaluate_dependents_subform(
            IntPtr handle,
            byte[] subformPath,
            byte[] changedPath,
            byte[]? data,
            byte[]? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_resolve_layout_subform(
            IntPtr handle,
            byte[] subformPath,
            [MarshalAs(UnmanagedType.I1)] bool evaluate
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema_subform(
            IntPtr handle,
            byte[] subformPath,
            [MarshalAs(UnmanagedType.I1)] bool resolveLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_schema_value_subform(
            IntPtr handle,
            byte[] subformPath
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema_without_params_subform(
            IntPtr handle,
            byte[] subformPath,
            [MarshalAs(UnmanagedType.I1)] bool resolveLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema_by_path_subform(
            IntPtr handle,
            byte[] subformPath,
            byte[] schemaPath,
            [MarshalAs(UnmanagedType.I1)] bool skipLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_has_subform(
            IntPtr handle,
            byte[] subformPath
        );
    }
}
#endif
