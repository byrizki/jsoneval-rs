#pragma once

#include <string>
#include <functional>
#include <thread>
#include <future>

namespace jsoneval {

/**
 * C++ Bridge for JSON Eval RS
 * Thread-safe wrapper around the Rust FFI
 */
class JsonEvalBridge {
public:
    /**
     * Create a new JSONEval instance
     * @param schema JSON schema string
     * @param context Optional context data
     * @param data Optional initial data
     * @return Handle string or error
     */
    static std::string create(
        const std::string& schema,
        const std::string& context,
        const std::string& data
    );

    /**
     * Create instance from MessagePack
     * @param schemaMsgpack MessagePack-encoded schema bytes
     * @param context Optional context data
     * @param data Optional initial data
     * @return Handle string or error
     */
    static std::string createFromMsgpack(
        const std::vector<uint8_t>& schemaMsgpack,
        const std::string& context,
        const std::string& data
    );

    /**
     * Create instance from ParsedSchemaCache
     * @param cacheKey Cache key to lookup in ParsedSchemaCache
     * @param context Optional context data
     * @param data Optional initial data
     * @return Handle string or error
     */
    static std::string createFromCache(
        const std::string& cacheKey,
        const std::string& context,
        const std::string& data
    );

    /**
     * Evaluate schema with data (async)
     * @param handle Instance handle
     * @param data JSON data string
     * @param context Optional context data
     * @param callback Result callback
     */
    static void evaluateAsync(
        const std::string& handle,
        const std::string& data,
        const std::string& context,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Validate data (async)
     * @param handle Instance handle
     * @param data JSON data string
     * @param context Optional context data
     * @param callback Result callback
     */
    static void validateAsync(
        const std::string& handle,
        const std::string& data,
        const std::string& context,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Evaluate dependents (async) - processes transitively
     * @param handle Instance handle
     * @param changedPathsJson JSON array of field paths that changed
     * @param data Optional updated JSON data string (empty to use existing)
     * @param context Optional context data
     * @param reEvaluate If true, performs full evaluation after processing dependents
     * @param callback Result callback
     */
    static void evaluateDependentsAsync(
        const std::string& handle,
        const std::string& changedPathsJson,
        const std::string& data,
        const std::string& context,
        bool reEvaluate,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get evaluated schema (async)
     * @param handle Instance handle
     * @param skipLayout Whether to skip layout resolution
     * @param callback Result callback
     */
    static void getEvaluatedSchemaAsync(
        const std::string& handle,
        bool skipLayout,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get evaluated schema in MessagePack format (async)
     * @param handle Instance handle
     * @param skipLayout Whether to skip layout resolution
     * @param callback Result callback with MessagePack binary data
     */
    static void getEvaluatedSchemaMsgpackAsync(
        const std::string& handle,
        bool skipLayout,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get schema value (async)
     * @param handle Instance handle
     * @param callback Result callback
     */
    static void getSchemaValueAsync(
        const std::string& handle,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get evaluated schema without $params field (async)
     * @param handle Instance handle
     * @param skipLayout Whether to skip layout resolution
     * @param callback Result callback
     */
    static void getEvaluatedSchemaWithoutParamsAsync(
        const std::string& handle,
        bool skipLayout,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get a value from evaluated schema using dotted path notation (async)
     * @param handle Instance handle
     * @param path Dotted path to the value (e.g., "properties.field.value")
     * @param skipLayout Whether to skip layout resolution
     * @param callback Result callback
     */
    static void getEvaluatedSchemaByPathAsync(
        const std::string& handle,
        const std::string& path,
        bool skipLayout,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get values from evaluated schema using multiple dotted path notations (async)
     * @param handle Instance handle
     * @param pathsJson JSON array of dotted paths to retrieve
     * @param skipLayout Whether to skip layout resolution
     * @param callback Result callback
     */
    static void getEvaluatedSchemaByPathsAsync(
        const std::string& handle,
        const std::string& pathsJson,
        bool skipLayout,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get a value from schema using dotted path notation (async)
     * @param handle Instance handle
     * @param path Dotted path to the value (e.g., "properties.field.value")
     * @param callback Result callback
     */
    static void getSchemaByPathAsync(
        const std::string& handle,
        const std::string& path,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get values from schema using multiple dotted path notations (async)
     * @param handle Instance handle
     * @param pathsJson JSON array of dotted paths to retrieve
     * @param callback Result callback
     */
    static void getSchemaByPathsAsync(
        const std::string& handle,
        const std::string& pathsJson,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Reload schema (async)
     * @param handle Instance handle
     * @param schema New JSON schema string
     * @param context Optional context data
     * @param data Optional initial data
     * @param callback Result callback
     */
    static void reloadSchemaAsync(
        const std::string& handle,
        const std::string& schema,
        const std::string& context,
        const std::string& data,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Reload schema from MessagePack (async)
     * @param handle Instance handle
     * @param schemaMsgpack MessagePack-encoded schema bytes
     * @param context Optional context data
     * @param data Optional initial data
     * @param callback Result callback
     */
    static void reloadSchemaMsgpackAsync(
        const std::string& handle,
        const std::vector<uint8_t>& schemaMsgpack,
        const std::string& context,
        const std::string& data,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Reload schema from ParsedSchemaCache (async)
     * @param handle Instance handle
     * @param cacheKey Cache key to lookup in ParsedSchemaCache
     * @param context Optional context data
     * @param data Optional initial data
     * @param callback Result callback
     */
    static void reloadSchemaFromCacheAsync(
        const std::string& handle,
        const std::string& cacheKey,
        const std::string& context,
        const std::string& data,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get cache stats (async)
     * @param handle Instance handle
     * @param callback Result callback
     */
    static void cacheStatsAsync(
        const std::string& handle,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Clear cache (async)
     * @param handle Instance handle
     * @param callback Result callback
     */
    static void clearCacheAsync(
        const std::string& handle,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get cache length (async)
     * @param handle Instance handle
     * @param callback Result callback
     */
    static void cacheLenAsync(
        const std::string& handle,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Enable evaluation caching (async)
     * @param handle Instance handle
     * @param callback Result callback
     */
    static void enableCacheAsync(
        const std::string& handle,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Disable evaluation caching (async)
     * @param handle Instance handle
     * @param callback Result callback
     */
    static void disableCacheAsync(
        const std::string& handle,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Check if caching is enabled (synchronous)
     * @param handle Instance handle
     * @returns true if caching is enabled, false otherwise
     */
    static bool isCacheEnabled(const std::string& handle);

    /**
     * Resolve layout with optional evaluation (async)
     * @param handle Instance handle
     * @param evaluate If true, runs evaluation before resolving layout
     * @param callback Result callback
     */
    static void resolveLayoutAsync(
        const std::string& handle,
        bool evaluate,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Compile and run JSON logic expression
     * @param handle JSONEval instance handle
     * @param logicStr JSON logic expression
     * @param data Optional JSON data string (empty to use existing data)
     * @param context Optional context data string (empty to use existing context)
     * @param callback Result callback
     */
    static void compileAndRunLogicAsync(
        const std::string& handle,
        const std::string& logicStr,
        const std::string& data,
        const std::string& context,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Compile JSON logic expression and return global logic ID
     * @param handle JSONEval instance handle
     * @param logicStr JSON logic expression
     * @return Compiled logic ID (global cache)
     */
    static uint64_t compileLogic(
        const std::string& handle,
        const std::string& logicStr
    );

    /**
     * Run pre-compiled JSON logic expression by ID
     * @param handle JSONEval instance handle
     * @param logicId Global compiled logic ID
     * @param data Optional JSON data string (empty to use existing data)
     * @param context Optional context data string (empty to use existing context)
     * @param callback Result callback
     */
    static void runLogicAsync(
        const std::string& handle,
        uint64_t logicId,
        const std::string& data,
        const std::string& context,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Validate with paths (async)
     * @param handle Instance handle
     * @param data JSON data string
     * @param context Optional context data
     * @param pathsJson Optional JSON array of paths
     * @param callback Result callback
     */
    static void validatePathsAsync(
        const std::string& handle,
        const std::string& data,
        const std::string& context,
        const std::string& pathsJson,
        std::function<void(const std::string&, const std::string&)> callback
    );

    // ========================================================================
    // Subform Methods
    // ========================================================================

    /**
     * Evaluate a subform with data (async)
     * @param handleId Instance handle
     * @param subformPath Path to the subform (e.g., "#/riders")
     * @param data JSON data string for the subform
     * @param context Optional context data
     * @param callback Result callback
     */
    static void evaluateSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        const std::string& data,
        const std::string& context,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Validate subform data against its schema rules (async)
     * @param handleId Instance handle
     * @param subformPath Path to the subform
     * @param data JSON data string for the subform
     * @param context Optional context data
     * @param callback Result callback
     */
    static void validateSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        const std::string& data,
        const std::string& context,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Evaluate dependents in subform when a field changes (async)
     * @param handleId Instance handle
     * @param subformPath Path to the subform
     * @param changedPath Path of the field that changed
     * @param data Optional updated JSON data string
     * @param context Optional context data
     * @param callback Result callback
     */
    static void evaluateDependentsSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        const std::string& changedPath,
        const std::string& data,
        const std::string& context,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Resolve layout for subform (async)
     * @param handleId Instance handle
     * @param subformPath Path to the subform
     * @param evaluate If true, runs evaluation before resolving layout
     * @param callback Result callback
     */
    static void resolveLayoutSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        bool evaluate,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get evaluated schema from subform (async)
     * @param handleId Instance handle
     * @param subformPath Path to the subform
     * @param resolveLayout Whether to resolve layout
     * @param callback Result callback
     */
    static void getEvaluatedSchemaSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        bool resolveLayout,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get schema value from subform (async)
     * @param handleId Instance handle
     * @param subformPath Path to the subform
     * @param callback Result callback
     */
    static void getSchemaValueSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get evaluated schema without $params from subform (async)
     * @param handleId Instance handle
     * @param subformPath Path to the subform
     * @param resolveLayout Whether to resolve layout
     * @param callback Result callback
     */
    static void getEvaluatedSchemaWithoutParamsSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
     * @param handleId Instance handle
     * @param subformPath Path to the subform
     * @param schemaPath Dotted path to the value within the subform
     * @param skipLayout Whether to skip layout resolution
     * @param callback Result callback
     */
    static void getEvaluatedSchemaByPathSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        const std::string& schemaPath,
        bool skipLayout,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get evaluated schema by multiple paths from subform (async)
     * @param handleId Instance handle
     * @param subformPath Path to the subform
     * @param schemaPathsJson JSON array of dotted paths to retrieve within the subform
     * @param skipLayout Whether to skip layout resolution
     * @param callback Result callback
     */
    static void getEvaluatedSchemaByPathsSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        const std::string& schemaPathsJson,
        bool skipLayout,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get list of available subform paths (async)
     * @param handleId Instance handle
     * @param callback Result callback
     */
    static void getSubformPathsAsync(
        const std::string& handleId,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get schema by specific path from subform (async)
     * @param handleId Instance handle
     * @param subformPath Path to the subform
     * @param schemaPath Dotted path to the value within the subform
     * @param callback Result callback
     */
    static void getSchemaByPathSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        const std::string& schemaPath,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Get schema by multiple paths from subform (async)
     * @param handleId Instance handle
     * @param subformPath Path to the subform
     * @param schemaPathsJson JSON array of dotted paths to retrieve within the subform
     * @param callback Result callback
     */
    static void getSchemaByPathsSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        const std::string& schemaPathsJson,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Check if a subform exists at the given path (async)
     * @param handleId Instance handle
     * @param subformPath Path to check
     * @param callback Result callback
     */
    static void hasSubformAsync(
        const std::string& handleId,
        const std::string& subformPath,
        std::function<void(const std::string&, const std::string&)> callback
    );

    /**
     * Dispose instance
     * @param handle Instance handle
     */
    static void dispose(const std::string& handle);

    /**
     * Get library version
     * @return Version string
     */
    static std::string version();

private:
    // Helper to run async operations
    template<typename Func>
    static void runAsync(Func&& func, std::function<void(const std::string&, const std::string&)> callback);
};

} // namespace jsoneval
