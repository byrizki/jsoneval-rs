#if NETCOREAPP || NET5_0_OR_GREATER
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace JsonEvalRs
{
    /// <summary>
    /// .NET Core / .NET 5+ specific P/Invoke declarations
    /// Uses UTF-8 string marshalling for better performance
    /// </summary>
    internal static partial class Native
    {
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

        // Core FFI functions with UTF-8 string marshalling
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
            out IntPtr errorPtr
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr json_eval_new_from_msgpack(
            IntPtr schemaMsgpack,
            UIntPtr schemaLen,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr json_eval_new_from_cache(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string cacheKey,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr json_eval_new_from_cache_with_error(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string cacheKey,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data,
            out IntPtr errorPtr
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
            [MarshalAs(UnmanagedType.LPUTF8Str)] string changedPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema_by_path(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
            [MarshalAs(UnmanagedType.I1)] bool skipLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_reload_schema(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string schema,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_reload_schema_msgpack(
            IntPtr handle,
            IntPtr schemaMsgpack,
            UIntPtr schemaLen,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_reload_schema_from_cache(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string cacheKey,
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

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_compile_and_run_logic(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string logicStr,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context
        );

        // Subform methods
        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_evaluate_subform(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string subformPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string data,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_validate_subform(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string subformPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string data,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_evaluate_dependents_subform(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string subformPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string changedPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? data,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string? context
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_resolve_layout_subform(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string subformPath,
            [MarshalAs(UnmanagedType.I1)] bool evaluate
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema_subform(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string subformPath,
            [MarshalAs(UnmanagedType.I1)] bool resolveLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_schema_value_subform(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string subformPath
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema_without_params_subform(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string subformPath,
            [MarshalAs(UnmanagedType.I1)] bool resolveLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_get_evaluated_schema_by_path_subform(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string subformPath,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string schemaPath,
            [MarshalAs(UnmanagedType.I1)] bool skipLayout
        );

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern FFIResult json_eval_has_subform(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string subformPath
        );
    }
}
#endif
