#include "json-eval-bridge.h"
#include <map>
#include <mutex>
#include <memory>

// Forward declarations for FFI functions
extern "C" {
    typedef struct {
        void* inner;
    } JSONEvalHandle;

    typedef struct {
        bool success;
        const uint8_t* data_ptr;
        size_t data_len;
        char* error;
        void* _owned_data;
    } FFIResult;

    JSONEvalHandle* json_eval_new(const char* schema, const char* context, const char* data);
    FFIResult json_eval_evaluate(JSONEvalHandle* handle, const char* data, const char* context);
    FFIResult json_eval_validate(JSONEvalHandle* handle, const char* data, const char* context);
    FFIResult json_eval_evaluate_dependents(JSONEvalHandle* handle, const char* changed_path, const char* data, const char* context);
    FFIResult json_eval_get_evaluated_schema(JSONEvalHandle* handle, bool skip_layout);
    FFIResult json_eval_get_schema_value(JSONEvalHandle* handle);
    FFIResult json_eval_get_evaluated_schema_without_params(JSONEvalHandle* handle, bool skip_layout);
    FFIResult json_eval_get_value_by_path(JSONEvalHandle* handle, const char* path, bool skip_layout);
    FFIResult json_eval_reload_schema(JSONEvalHandle* handle, const char* schema, const char* context, const char* data);
    FFIResult json_eval_cache_stats(JSONEvalHandle* handle);
    FFIResult json_eval_clear_cache(JSONEvalHandle* handle);
    FFIResult json_eval_cache_len(JSONEvalHandle* handle);
    FFIResult json_eval_validate_paths(JSONEvalHandle* handle, const char* data, const char* context, const char* paths_json);
    void json_eval_free(JSONEvalHandle* handle);
    void json_eval_free_result(FFIResult result);
    const char* json_eval_version();
    void json_eval_free_string(char* ptr);
}

namespace jsoneval {

// Handle storage
static std::map<std::string, JSONEvalHandle*> handles;
static std::mutex handlesMutex;
static int handleCounter = 0;

std::string JsonEvalBridge::create(
    const std::string& schema,
    const std::string& context,
    const std::string& data
) {
    const char* ctx = context.empty() ? nullptr : context.c_str();
    const char* dt = data.empty() ? nullptr : data.c_str();
    
    JSONEvalHandle* handle = json_eval_new(schema.c_str(), ctx, dt);
    
    if (handle == nullptr) {
        throw std::runtime_error("Failed to create JSONEval instance");
    }
    
    std::lock_guard<std::mutex> lock(handlesMutex);
    std::string handleId = "handle_" + std::to_string(handleCounter++);
    handles[handleId] = handle;
    
    return handleId;
}

template<typename Func>
void JsonEvalBridge::runAsync(Func&& func, std::function<void(const std::string&, const std::string&)> callback) {
    std::thread([func = std::forward<Func>(func), callback]() {
        try {
            std::string result = func();
            callback(result, "");
        } catch (const std::exception& e) {
            callback("", e.what());
        }
    }).detach();
}

void JsonEvalBridge::evaluateAsync(
    const std::string& handleId,
    const std::string& data,
    const std::string& context,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, data, context]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        // Step 1: Evaluate (no longer returns data)
        const char* ctx = context.empty() ? nullptr : context.c_str();
        FFIResult evalResult = json_eval_evaluate(it->second, data.c_str(), ctx);
        
        if (!evalResult.success) {
            std::string error = evalResult.error ? evalResult.error : "Unknown error";
            json_eval_free_result(evalResult);
            throw std::runtime_error(error);
        }
        json_eval_free_result(evalResult);
        
        // Step 2: Get the evaluated schema
        FFIResult schemaResult = json_eval_get_evaluated_schema(it->second, true);
        
        if (!schemaResult.success) {
            std::string error = schemaResult.error ? schemaResult.error : "Unknown error";
            json_eval_free_result(schemaResult);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (schemaResult.data_ptr && schemaResult.data_len > 0) {
            resultStr = std::string(reinterpret_cast<const char*>(schemaResult.data_ptr), schemaResult.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(schemaResult);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::validateAsync(
    const std::string& handleId,
    const std::string& data,
    const std::string& context,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, data, context]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* ctx = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_validate(it->second, data.c_str(), ctx);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr = std::string(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::evaluateDependentsAsync(
    const std::string& handleId,
    const std::string& changedPath,
    const std::string& data,
    const std::string& context,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, changedPath, data, context]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* dataPtr = data.empty() ? nullptr : data.c_str();
        const char* ctx = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_evaluate_dependents(
            it->second, 
            changedPath.c_str(), 
            dataPtr, 
            ctx
        );
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr = std::string(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getEvaluatedSchemaAsync(
    const std::string& handleId,
    bool skipLayout,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, skipLayout]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_evaluated_schema(it->second, skipLayout);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr = std::string(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getSchemaValueAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_schema_value(it->second);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr = std::string(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getEvaluatedSchemaWithoutParamsAsync(
    const std::string& handleId,
    bool skipLayout,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, skipLayout]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_evaluated_schema_without_params(it->second, skipLayout);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr = std::string(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getValueByPathAsync(
    const std::string& handleId,
    const std::string& path,
    bool skipLayout,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, path, skipLayout]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_value_by_path(it->second, path.c_str(), skipLayout);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr = std::string(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "null";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::reloadSchemaAsync(
    const std::string& handleId,
    const std::string& schema,
    const std::string& context,
    const std::string& data,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, schema, context, data]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* dt = data.empty() ? nullptr : data.c_str();
        FFIResult result = json_eval_reload_schema(it->second, schema.c_str(), ctx, dt);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        json_eval_free_result(result);
        return "{}";
    }, callback);
}

void JsonEvalBridge::cacheStatsAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_cache_stats(it->second);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr = std::string(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::clearCacheAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_clear_cache(it->second);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        json_eval_free_result(result);
        return "{}";
    }, callback);
}

void JsonEvalBridge::cacheLenAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_cache_len(it->second);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr = std::string(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "0";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::validatePathsAsync(
    const std::string& handleId,
    const std::string& data,
    const std::string& context,
    const std::string& pathsJson,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, data, context, pathsJson]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* paths = pathsJson.empty() ? nullptr : pathsJson.c_str();
        FFIResult result = json_eval_validate_paths(it->second, data.c_str(), ctx, paths);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr = std::string(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::dispose(const std::string& handleId) {
    std::lock_guard<std::mutex> lock(handlesMutex);
    auto it = handles.find(handleId);
    if (it != handles.end()) {
        json_eval_free(it->second);
        handles.erase(it);
    }
}

std::string JsonEvalBridge::version() {
    const char* ver = json_eval_version();
    // Version string is static in Rust, no need to free it
    return ver ? ver : "unknown";
}

} // namespace jsoneval
