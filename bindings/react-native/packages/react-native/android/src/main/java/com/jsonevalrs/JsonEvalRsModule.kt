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
        promise: Promise
    ) {
        nativeEvaluateAsync(handle, data, context ?: "", promise)
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
        changedPaths: ReadableArray,
        data: String,
        context: String?,
        nested: Boolean,
        promise: Promise
    ) {
        val pathsJson = arrayToJsonString(changedPaths)
        nativeEvaluateDependentsAsync(handle, pathsJson, data, context ?: "", nested, promise)
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

    @ReactMethod(isBlockingSynchronousMethod = true)
    fun dispose(
        handle: String
    ) {
        nativeDispose(handle)
    }

    @ReactMethod
    fun version(promise: Promise) {
        try {
            val ver = nativeVersion()
            promise.resolve(ver)
        } catch (e: Exception) {
            promise.reject("VERSION_ERROR", e.message, e)
        }
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
    private external fun nativeEvaluateAsync(handle: String, data: String, context: String, promise: Promise)
    private external fun nativeValidateAsync(handle: String, data: String, context: String, promise: Promise)
    private external fun nativeEvaluateDependentsAsync(
        handle: String,
        changedPathsJson: String,
        data: String,
        context: String,
        nested: Boolean,
        promise: Promise
    )
    private external fun nativeGetEvaluatedSchemaAsync(handle: String, skipLayout: Boolean, promise: Promise)
    private external fun nativeGetSchemaValueAsync(handle: String, promise: Promise)
    private external fun nativeReloadSchemaAsync(handle: String, schema: String, context: String, data: String, promise: Promise)
    private external fun nativeCacheStatsAsync(handle: String, promise: Promise)
    private external fun nativeClearCacheAsync(handle: String, promise: Promise)
    private external fun nativeCacheLenAsync(handle: String, promise: Promise)
    private external fun nativeValidatePathsAsync(handle: String, data: String, context: String, pathsJson: String, promise: Promise)
    private external fun nativeDispose(handle: String)
    private external fun nativeVersion(): String
}
