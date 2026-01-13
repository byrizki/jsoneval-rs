package com.jsonevalrs

import com.facebook.react.bridge.*
import com.facebook.react.module.annotations.ReactModule

@ReactModule(name = JsonEvalRsModule.NAME)
class JsonEvalRsModule(reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext) {

    override fun getName(): String {
        return NAME
    }

    companion object {
        const val NAME = "JsonEvalRs"

        init {
            try {
                System.loadLibrary("json_eval_rs")
                System.loadLibrary("json_eval_rn")
            } catch (e: UnsatisfiedLinkError) {
                e.printStackTrace()
            }
        }
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun create(
        schema: String,
        context: String?,
        data: String?
    ): String {
        return nativeCreate(schema, context ?: "", data ?: "")
    }

    @ReactMethod
    fun evaluate(
        handle: String,
        data: String,
        context: String?,
        pathsJson: String?,
        promise: Promise
    ) {
        nativeEvaluateAsync(handle, data, context ?: "", pathsJson ?: "", promise)
    }

    @ReactMethod
    fun validate(
        handle: String,
        data: String,
        context: String?,
        promise: Promise
    ) {
        nativeValidateAsync(handle, data, context ?: "", promise)
    }

    @ReactMethod
    fun evaluateDependents(
        handle: String,
        changedPathsJson: String,
        data: String?,
        context: String?,
        reEvaluate: Boolean,
        promise: Promise
    ) {
        nativeEvaluateDependentsAsync(handle, changedPathsJson, data ?: "", context ?: "", reEvaluate, promise)
    }

    @ReactMethod
    fun getEvaluatedSchema(
        handle: String,
        skipLayout: Boolean,
        promise: Promise
    ) {
        nativeGetEvaluatedSchemaAsync(handle, skipLayout, promise)
    }

    @ReactMethod
    fun getSchemaValue(
        handle: String,
        promise: Promise
    ) {
        nativeGetSchemaValueAsync(handle, promise)
    }

    @ReactMethod
    fun getEvaluatedSchemaWithoutParams(
        handle: String,
        skipLayout: Boolean,
        promise: Promise
    ) {
        nativeGetEvaluatedSchemaWithoutParamsAsync(handle, skipLayout, promise)
    }

    @ReactMethod
    fun getEvaluatedSchemaByPath(
        handle: String,
        path: String,
        skipLayout: Boolean,
        promise: Promise
    ) {
        nativeGetEvaluatedSchemaByPathAsync(handle, path, skipLayout, promise)
    }

    @ReactMethod
    fun getEvaluatedSchemaByPaths(
        handle: String,
        pathsJson: String,
        skipLayout: Boolean,
        format: Int,
        promise: Promise
    ) {
        nativeGetEvaluatedSchemaByPathsAsync(handle, pathsJson, skipLayout, format, promise)
    }

    @ReactMethod
    fun getSchemaByPath(
        handle: String,
        path: String,
        promise: Promise
    ) {
        nativeGetSchemaByPathAsync(handle, path, promise)
    }

    @ReactMethod
    fun getSchemaByPaths(
        handle: String,
        pathsJson: String,
        format: Int,
        promise: Promise
    ) {
        nativeGetSchemaByPathsAsync(handle, pathsJson, format, promise)
    }

    @ReactMethod
    fun reloadSchema(
        handle: String,
        schema: String,
        context: String?,
        data: String?,
        promise: Promise
    ) {
        nativeReloadSchemaAsync(handle, schema, context ?: "", data ?: "", promise)
    }

    @ReactMethod
    fun reloadSchemaMsgpack(
        handle: String,
        schemaMsgpack: ReadableArray,
        context: String?,
        data: String?,
        promise: Promise
    ) {
        // Convert ReadableArray to ByteArray
        val byteArray = ByteArray(schemaMsgpack.size())
        for (i in 0 until schemaMsgpack.size()) {
            byteArray[i] = schemaMsgpack.getInt(i).toByte()
        }
        nativeReloadSchemaMsgpackAsync(handle, byteArray, context ?: "", data ?: "", promise)
    }

    @ReactMethod
    fun reloadSchemaFromCache(
        handle: String,
        cacheKey: String,
        context: String?,
        data: String?,
        promise: Promise
    ) {
        nativeReloadSchemaFromCacheAsync(handle, cacheKey, context ?: "", data ?: "", promise)
    }

    @ReactMethod
    fun createFromMsgpack(
        schemaMsgpack: ReadableArray,
        context: String?,
        data: String?,
        promise: Promise
    ) {
        try {
            // Convert ReadableArray to ByteArray
            val byteArray = ByteArray(schemaMsgpack.size())
            for (i in 0 until schemaMsgpack.size()) {
                byteArray[i] = schemaMsgpack.getInt(i).toByte()
            }
            val handle = nativeCreateFromMsgpack(byteArray, context ?: "", data ?: "")
            promise.resolve(handle)
        } catch (e: Exception) {
            promise.reject("CREATE_FROM_MSGPACK_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun createFromCache(
        cacheKey: String,
        context: String?,
        data: String?,
        promise: Promise
    ) {
        try {
            val handle = nativeCreateFromCache(cacheKey, context ?: "", data ?: "")
            promise.resolve(handle)
        } catch (e: Exception) {
            promise.reject("CREATE_FROM_CACHE_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun cacheStats(
        handle: String,
        promise: Promise
    ) {
        nativeCacheStatsAsync(handle, promise)
    }

    @ReactMethod
    fun clearCache(
        handle: String,
        promise: Promise
    ) {
        nativeClearCacheAsync(handle, promise)
    }

    @ReactMethod
    fun cacheLen(
        handle: String,
        promise: Promise
    ) {
        nativeCacheLenAsync(handle, promise)
    }

    @ReactMethod
    fun enableCache(
        handle: String,
        promise: Promise
    ) {
        nativeEnableCacheAsync(handle, promise)
    }

    @ReactMethod
    fun disableCache(
        handle: String,
        promise: Promise
    ) {
        nativeDisableCacheAsync(handle, promise)
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun isCacheEnabled(
        handle: String
    ): Boolean {
        return nativeIsCacheEnabled(handle)
    }

    @ReactMethod
    fun setTimezoneOffset(
        handle: String,
        offsetMinutes: Int?,
        promise: Promise
    ) {
        try {
            // Use Int.MIN_VALUE as sentinel for null
            val offset = offsetMinutes ?: Int.MIN_VALUE
            nativeSetTimezoneOffset(handle, offset)
            promise.resolve(null)
        } catch (e: Exception) {
            promise.reject("SET_TIMEZONE_OFFSET_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun validatePaths(
        handle: String,
        data: String,
        context: String?,
        paths: ReadableArray?,
        promise: Promise
    ) {
        val pathsJson = if (paths != null) arrayToJsonString(paths) else ""
        nativeValidatePathsAsync(handle, data, context ?: "", pathsJson, promise)
    }

    @ReactMethod
    fun resolveLayout(
        handle: String,
        evaluate: Boolean,
        promise: Promise
    ) {
        nativeResolveLayoutAsync(handle, evaluate, promise)
    }

    @ReactMethod
    fun compileAndRunLogic(
        handle: String,
        logicStr: String,
        data: String?,
        context: String?,
        promise: Promise
    ) {
        nativeCompileAndRunLogicAsync(handle, logicStr, data ?: "", context ?: "", promise)
    }

    @ReactMethod
    fun compileLogic(
        handle: String,
        logicStr: String,
        promise: Promise
    ) {
        try {
            val logicId = nativeCompileLogic(handle, logicStr)
            if (logicId == 0.0) {
                promise.reject("COMPILE_LOGIC_ERROR", "Failed to compile logic")
            } else {
                promise.resolve(logicId)
            }
        } catch (e: Exception) {
            promise.reject("COMPILE_LOGIC_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun runLogic(
        handle: String,
        logicId: Double,
        data: String?,
        context: String?,
        promise: Promise
    ) {
        nativeRunLogicAsync(handle, logicId, data ?: "", context ?: "", promise)
    }

    // ========================================================================
    // Subform Methods
    // ========================================================================


    @ReactMethod
    fun validateSubform(
        handle: String,
        subformPath: String,
        data: String,
        context: String?,
        promise: Promise
    ) {
        nativeValidateSubformAsync(handle, subformPath, data, context ?: "", promise)
    }

    @ReactMethod
    fun evaluateDependentsSubform(
        handle: String,
        subformPath: String,
        changedPath: String,
        data: String?,
        context: String?,
        reEvaluate: Boolean,
        promise: Promise
    ) {
        nativeEvaluateDependentsSubformAsync(handle, subformPath, changedPath, data ?: "", context ?: "", reEvaluate, promise)
    }

    @ReactMethod
    fun resolveLayoutSubform(
        handle: String,
        subformPath: String,
        evaluate: Boolean,
        promise: Promise
    ) {
        nativeResolveLayoutSubformAsync(handle, subformPath, evaluate, promise)
    }

    @ReactMethod
    fun getEvaluatedSchemaSubform(
        handle: String,
        subformPath: String,
        resolveLayout: Boolean,
        promise: Promise
    ) {
        nativeGetEvaluatedSchemaSubformAsync(handle, subformPath, resolveLayout, promise)
    }

    @ReactMethod
    fun getSchemaValueSubform(
        handle: String,
        subformPath: String,
        promise: Promise
    ) {
        nativeGetSchemaValueSubformAsync(handle, subformPath, promise)
    }

    @ReactMethod
    fun getEvaluatedSchemaWithoutParamsSubform(
        handle: String,
        subformPath: String,
        resolveLayout: Boolean,
        promise: Promise
    ) {
        nativeGetEvaluatedSchemaWithoutParamsSubformAsync(handle, subformPath, resolveLayout, promise)
    }

    @ReactMethod
    fun getEvaluatedSchemaByPathSubform(
        handle: String,
        subformPath: String,
        schemaPath: String,
        skipLayout: Boolean,
        promise: Promise
    ) {
        nativeGetEvaluatedSchemaByPathSubformAsync(handle, subformPath, schemaPath, skipLayout, promise)
    }

    @ReactMethod
    fun getEvaluatedSchemaByPathsSubform(
        handle: String,
        subformPath: String,
        schemaPathsJson: String,
        skipLayout: Boolean,
        format: Int,
        promise: Promise
    ) {
        nativeGetEvaluatedSchemaByPathsSubformAsync(
            handle,
            subformPath,
            schemaPathsJson,
            skipLayout,
            format,
            promise
        )
    }

    @ReactMethod
    fun getSchemaByPathSubform(
        handle: String,
        subformPath: String,
        schemaPath: String,
        promise: Promise
    ) {
        nativeGetSchemaByPathSubformAsync(handle, subformPath, schemaPath, promise)
    }

    @ReactMethod
    fun getSchemaByPathsSubform(
        handle: String,
        subformPath: String,
        schemaPathsJson: String,
        format: Int,
        promise: Promise
    ) {
        nativeGetSchemaByPathsSubformAsync(
            handle,
            subformPath,
            schemaPathsJson,
            format,
            promise
        )
    }

    @ReactMethod
    fun getSubformPaths(
        handle: String,
        promise: Promise
    ) {
        nativeGetSubformPathsAsync(handle, promise)
    }

    @ReactMethod
    fun hasSubform(
        handle: String,
        subformPath: String,
        promise: Promise
    ) {
        nativeHasSubformAsync(handle, subformPath, promise)
    }

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun dispose(
        handle: String
    ): Boolean {
        nativeDispose(handle)
        return true
    }


    @ReactMethod
    fun cancel(handle: String) {
        nativeCancel(handle)
    }

    @ReactMethod
    fun multiply(a: Double, b: Double, promise: Promise) {
        promise.resolve(a * b)
    }

    private fun arrayToJsonString(array: ReadableArray): String {
        val items = mutableListOf<String>()
        for (i in 0 until array.size()) {
            items.add("\"${array.getString(i)}\"")
        }
        return "[${items.joinToString(",")}]"
    }

    // Native methods
    private external fun nativeCreate(schema: String, context: String, data: String): String
    private external fun nativeCreateFromMsgpack(schemaMsgpack: ByteArray, context: String, data: String): String
    private external fun nativeCreateFromCache(cacheKey: String, context: String, data: String): String
    private external fun nativeEvaluateAsync(handle: String, data: String, context: String, pathsJson: String, promise: Promise)
    private external fun nativeValidateAsync(handle: String, data: String, context: String, promise: Promise)
    private external fun nativeEvaluateDependentsAsync(
        handle: String,
        changedPathsJson: String,
        data: String,
        context: String,
        reEvaluate: Boolean,
        promise: Promise
    )
    private external fun nativeGetEvaluatedSchemaAsync(handle: String, skipLayout: Boolean, promise: Promise)
    private external fun nativeGetSchemaValueAsync(handle: String, promise: Promise)
    private external fun nativeGetEvaluatedSchemaWithoutParamsAsync(handle: String, skipLayout: Boolean, promise: Promise)
    private external fun nativeGetEvaluatedSchemaByPathAsync(handle: String, path: String, skipLayout: Boolean, promise: Promise)
    private external fun nativeGetEvaluatedSchemaByPathsAsync(handle: String, pathsJson: String, skipLayout: Boolean, format: Int, promise: Promise)
    private external fun nativeGetSchemaByPathAsync(handle: String, path: String, promise: Promise)
    private external fun nativeGetSchemaByPathsAsync(handle: String, pathsJson: String, format: Int, promise: Promise)
    private external fun nativeReloadSchemaAsync(handle: String, schema: String, context: String, data: String, promise: Promise)
    private external fun nativeReloadSchemaMsgpackAsync(handle: String, schemaMsgpack: ByteArray, context: String, data: String, promise: Promise)
    private external fun nativeReloadSchemaFromCacheAsync(handle: String, cacheKey: String, context: String, data: String, promise: Promise)
    private external fun nativeCacheStatsAsync(handle: String, promise: Promise)
    private external fun nativeClearCacheAsync(handle: String, promise: Promise)
    private external fun nativeCacheLenAsync(handle: String, promise: Promise)
    private external fun nativeEnableCacheAsync(handle: String, promise: Promise)
    private external fun nativeDisableCacheAsync(handle: String, promise: Promise)
    private external fun nativeIsCacheEnabled(handle: String): Boolean
    private external fun nativeValidatePathsAsync(handle: String, data: String, context: String, pathsJson: String, promise: Promise)
    private external fun nativeResolveLayoutAsync(handle: String, evaluate: Boolean, promise: Promise)
    private external fun nativeCompileAndRunLogicAsync(handle: String, logicStr: String, data: String, context: String, promise: Promise)
    private external fun nativeCompileLogic(handle: String, logicStr: String): Double
    private external fun nativeRunLogicAsync(handle: String, logicId: Double, data: String, context: String, promise: Promise)
    private external fun nativeSetTimezoneOffset(handle: String, offsetMinutes: Int)
    
    // Subform native methods
    @ReactMethod
    fun evaluateSubform(handle: String, subformPath: String, data: String, context: String?, paths: ReadableArray?, promise: Promise) {
        // Convert ReadableArray to JSON string for passing to JNI?
        // Or native method accepts array?
        // Based on implementation plan, we'll convert to JSON string here or in C++.
        // Let's pass the array to JNI directly if possible, or convert to List/JSON string.
        // For simplicity and consistency with C#, let's serialize to JSON string if paths is not null.

        // Actually, readableArray handling in JNI usually involves specialized helpers.
        // Simpler: convert to JSON string in Kotlin.
        // But Wait, C++ side `nativeEvaluateSubform` needs paths_json string.

        var pathsJson: String? = null
        if (paths != null) {
            try {
                val list = ArrayList<String>()
                for (i in 0 until paths.size()) {
                    list.add(paths.getString(i) ?: "")
                }
                // Simple manual JSON array construction or use Gson if available?
                // Avoiding dependency: assume valid strings
                val sb = StringBuilder("[")
                for (i in list.indices) {
                    if (i > 0) sb.append(",")
                    sb.append("\"").append(list[i].replace("\"", "\\\"")).append("\"")
                }
                sb.append("]")
                pathsJson = sb.toString()
            } catch (e: Exception) {
                 promise.reject("E_INVALID_PATHS", "Invalid paths array")
                 return
            }
        }

        try {
            nativeEvaluateSubform(handle, subformPath, data, context ?: "", pathsJson ?: "", promise)
        } catch (e: Exception) {
            promise.reject("E_EVAL_ERROR", e.message)
        }
    }

    private external fun nativeEvaluateSubform(handle: String, subformPath: String, data: String, context: String, pathsJson: String, promise: Promise)
    private external fun nativeValidateSubformAsync(handle: String, subformPath: String, data: String, context: String, promise: Promise)
    private external fun nativeEvaluateDependentsSubformAsync(handle: String, subformPath: String, changedPath: String, data: String, context: String, reEvaluate: Boolean, promise: Promise)
    private external fun nativeResolveLayoutSubformAsync(handle: String, subformPath: String, evaluate: Boolean, promise: Promise)
    private external fun nativeGetEvaluatedSchemaSubformAsync(handle: String, subformPath: String, resolveLayout: Boolean, promise: Promise)
    private external fun nativeGetSchemaValueSubformAsync(handle: String, subformPath: String, promise: Promise)
    private external fun nativeGetEvaluatedSchemaWithoutParamsSubformAsync(handle: String, subformPath: String, resolveLayout: Boolean, promise: Promise)
    private external fun nativeGetEvaluatedSchemaByPathSubformAsync(handle: String, subformPath: String, schemaPath: String, skipLayout: Boolean, promise: Promise)
    private external fun nativeGetEvaluatedSchemaByPathsSubformAsync(handle: String, subformPath: String, schemaPathsJson: String, skipLayout: Boolean, format: Int, promise: Promise)
    private external fun nativeGetSchemaByPathSubformAsync(handle: String, subformPath: String, schemaPath: String, promise: Promise)
    private external fun nativeGetSchemaByPathsSubformAsync(handle: String, subformPath: String, schemaPathsJson: String, format: Int, promise: Promise)
    private external fun nativeGetSubformPathsAsync(handle: String, promise: Promise)
    private external fun nativeHasSubformAsync(handle: String, subformPath: String, promise: Promise)
    
    private external fun nativeDispose(handle: String)
    private external fun nativeCancel(handle: String)
    private external fun nativeVersion(): String
}
