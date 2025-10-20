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
     * Create a new JSONEval instance from MessagePack-encoded schema
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
     * @param changedPath Single field path that changed
     * @param data Optional updated JSON data string (empty to use existing)
     * @param context Optional context data
     * @param callback Result callback
     */
    static void evaluateDependentsAsync(
        const std::string& handle,
        const std::string& changedPath,
        const std::string& data,
        const std::string& context,
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
     * Compile and run JSON logic from a JSON logic string (async)
     * @param handle Instance handle
     * @param logicStr JSON logic expression as a string
     * @param data Optional JSON data string (empty to use existing data)
     * @param callback Result callback
     */
    static void compileAndRunLogicAsync(
        const std::string& handle,
        const std::string& logicStr,
        const std::string& data,
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
