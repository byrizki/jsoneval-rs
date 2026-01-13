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
    JSONEvalHandle* json_eval_new_from_msgpack(const uint8_t* schema_msgpack, size_t schema_len, const char* context, const char* data);
    FFIResult json_eval_evaluate(JSONEvalHandle* handle, const char* data, const char* context, const char* paths_json);
    FFIResult json_eval_get_evaluated_schema_msgpack(JSONEvalHandle* handle, bool skip_layout);
    FFIResult json_eval_validate(JSONEvalHandle* handle, const char* data, const char* context);
    FFIResult json_eval_evaluate_dependents(JSONEvalHandle* handle, const char* changed_path, const char* data, const char* context, int re_evaluate);
    FFIResult json_eval_get_evaluated_schema(JSONEvalHandle* handle, bool skip_layout);
    FFIResult json_eval_get_schema_value(JSONEvalHandle* handle);
    FFIResult json_eval_get_evaluated_schema_without_params(JSONEvalHandle* handle, bool skip_layout);
    FFIResult json_eval_get_evaluated_schema_by_path(JSONEvalHandle* handle, const char* path, bool skip_layout);
    FFIResult json_eval_get_evaluated_schema_by_paths(JSONEvalHandle* handle, const char* paths_json, bool skip_layout, uint8_t format);
    FFIResult json_eval_get_schema_by_path(JSONEvalHandle* handle, const char* path);
    FFIResult json_eval_get_schema_by_paths(JSONEvalHandle* handle, const char* paths_json, uint8_t format);
    FFIResult json_eval_resolve_layout(JSONEvalHandle* handle, bool evaluate);
    FFIResult json_eval_compile_and_run_logic(JSONEvalHandle* handle, const char* logic_str, const char* data, const char* context);
    uint64_t json_eval_compile_logic(JSONEvalHandle* handle, const char* logic_str);
    FFIResult json_eval_run_logic(JSONEvalHandle* handle, uint64_t logic_id, const char* data, const char* context);
    FFIResult json_eval_reload_schema(JSONEvalHandle* handle, const char* schema, const char* context, const char* data);
    FFIResult json_eval_reload_schema_msgpack(JSONEvalHandle* handle, const uint8_t* schema_msgpack, size_t schema_len, const char* context, const char* data);
    FFIResult json_eval_reload_schema_from_cache(JSONEvalHandle* handle, const char* cache_key, const char* context, const char* data);
    JSONEvalHandle* json_eval_new_from_cache(const char* cache_key, const char* context, const char* data);
    FFIResult json_eval_cache_stats(JSONEvalHandle* handle);
    FFIResult json_eval_clear_cache(JSONEvalHandle* handle);
    FFIResult json_eval_cache_len(JSONEvalHandle* handle);
    FFIResult json_eval_enable_cache(JSONEvalHandle* handle);
    FFIResult json_eval_disable_cache(JSONEvalHandle* handle);
    int json_eval_is_cache_enabled(JSONEvalHandle* handle);
    FFIResult json_eval_validate_paths(JSONEvalHandle* handle, const char* data, const char* context, const char* paths_json);
    FFIResult json_eval_evaluate_logic_pure(const char* logic_str, const char* data, const char* context);
    
    // Subform FFI methods
    FFIResult json_eval_evaluate_subform(JSONEvalHandle* handle, const char* subform_path, const char* data, const char* context, const char* paths_json);
    FFIResult json_eval_validate_subform(JSONEvalHandle* handle, const char* subform_path, const char* data, const char* context);
    FFIResult json_eval_evaluate_dependents_subform(JSONEvalHandle* handle, const char* subform_path, const char* changed_path, const char* data, const char* context, int re_evaluate);
    FFIResult json_eval_resolve_layout_subform(JSONEvalHandle* handle, const char* subform_path, bool evaluate);
    FFIResult json_eval_get_evaluated_schema_subform(JSONEvalHandle* handle, const char* subform_path, bool resolve_layout);
    FFIResult json_eval_get_schema_value_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_get_evaluated_schema_without_params_subform(JSONEvalHandle* handle, const char* subform_path, bool resolve_layout);
    FFIResult json_eval_get_evaluated_schema_by_path_subform(JSONEvalHandle* handle, const char* subform_path, const char* schema_path, bool skip_layout);
    FFIResult json_eval_get_evaluated_schema_by_paths_subform(JSONEvalHandle* handle, const char* subform_path, const char* schema_paths_json, bool skip_layout, uint8_t format);
    FFIResult json_eval_get_schema_by_path_subform(JSONEvalHandle* handle, const char* subform_path, const char* schema_path);
    FFIResult json_eval_get_schema_by_paths_subform(JSONEvalHandle* handle, const char* subform_path, const char* schema_paths_json, uint8_t format);
    FFIResult json_eval_get_subform_paths(JSONEvalHandle* handle);
    FFIResult json_eval_has_subform(JSONEvalHandle* handle, const char* subform_path);
    
    void json_eval_set_timezone_offset(JSONEvalHandle* handle, int32_t offset_minutes);
    
    void json_eval_free(JSONEvalHandle* handle);
    void json_eval_cancel(JSONEvalHandle* handle);
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

std::string JsonEvalBridge::createFromMsgpack(
    const std::vector<uint8_t>& schemaMsgpack,
    const std::string& context,
    const std::string& data
) {
    const char* ctx = context.empty() ? nullptr : context.c_str();
    const char* dt = data.empty() ? nullptr : data.c_str();
    
    JSONEvalHandle* handle = json_eval_new_from_msgpack(
        schemaMsgpack.data(),
        schemaMsgpack.size(),
        ctx,
        dt
    );
    
    if (handle == nullptr) {
        throw std::runtime_error("Failed to create JSONEval instance from MessagePack");
    }
    
    std::lock_guard<std::mutex> lock(handlesMutex);
    std::string handleId = "handle_" + std::to_string(handleCounter++);
    handles[handleId] = handle;
    
    return handleId;
}

std::string JsonEvalBridge::createFromCache(
    const std::string& cacheKey,
    const std::string& context,
    const std::string& data
) {
    const char* ctx = context.empty() ? nullptr : context.c_str();
    const char* dt = data.empty() ? nullptr : data.c_str();
    
    JSONEvalHandle* handle = json_eval_new_from_cache(
        cacheKey.c_str(),
        ctx,
        dt
    );
    
    if (handle == nullptr) {
        throw std::runtime_error("Failed to create JSONEval instance from cache");
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
    const std::string& pathsJson,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, data, context, pathsJson]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        // Step 1: Evaluate with pathsJson parameter
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* pathsPtr = pathsJson.empty() ? nullptr : pathsJson.c_str();
        FFIResult evalResult = json_eval_evaluate(it->second, data.c_str(), ctx, pathsPtr);
        
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
        
        // Zero-copy: construct string directly from raw pointer without intermediate copy
        std::string resultStr;
        if (schemaResult.data_ptr && schemaResult.data_len > 0) {
            // Use string constructor that takes pointer + length (still copies, but single copy)
            resultStr.assign(reinterpret_cast<const char*>(schemaResult.data_ptr), schemaResult.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(schemaResult);
        return resultStr;
    }, callback);
}

uint64_t JsonEvalBridge::compileLogic(
    const std::string& handleId,
    const std::string& logicStr
) {
    std::lock_guard<std::mutex> lock(handlesMutex);
    auto it = handles.find(handleId);
    if (it == handles.end()) {
        throw std::runtime_error("Invalid handle");
    }

    uint64_t logicId = json_eval_compile_logic(it->second, logicStr.c_str());
    if (logicId == 0) {
        throw std::runtime_error("Failed to compile logic (received ID 0)");
    }

    return logicId;
}

void JsonEvalBridge::runLogicAsync(
    const std::string& handleId,
    uint64_t logicId,
    const std::string& data,
    const std::string& context,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, logicId, data, context]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }

        FFIResult result = json_eval_run_logic(
            it->second,
            logicId,
            data.empty() ? nullptr : data.c_str(),
            context.empty() ? nullptr : context.c_str()
        );

        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }

        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
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
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::evaluateLogicAsync(
    const std::string& logicStr,
    const std::string& data,
    const std::string& context,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([logicStr, data, context]() -> std::string {
        const char* dt = data.empty() ? nullptr : data.c_str();
        const char* ctx = context.empty() ? nullptr : context.c_str();
        
        FFIResult result = json_eval_evaluate_logic_pure(logicStr.c_str(), dt, ctx);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "null";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::evaluateDependentsAsync(
    const std::string& handleId,
    const std::string& changedPathsJson,
    const std::string& data,
    const std::string& context,
    bool reEvaluate,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, changedPathsJson, data, context, reEvaluate]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* dataPtr = data.empty() ? nullptr : data.c_str();
        const char* ctx = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_evaluate_dependents(
            it->second, 
            changedPathsJson.c_str(), 
            dataPtr, 
            ctx,
            reEvaluate ? 1 : 0
        );
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
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
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getEvaluatedSchemaMsgpackAsync(
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
        
        FFIResult result = json_eval_get_evaluated_schema_msgpack(it->second, skipLayout);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        // Zero-copy: construct string directly from raw pointer (binary data)
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "";
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
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
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
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getEvaluatedSchemaByPathAsync(
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
        
        FFIResult result = json_eval_get_evaluated_schema_by_path(it->second, path.c_str(), skipLayout);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "null";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getEvaluatedSchemaByPathsAsync(
    const std::string& handleId,
    const std::string& pathsJson,
    bool skipLayout,
    int format,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, pathsJson, skipLayout, format]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_evaluated_schema_by_paths(it->second, pathsJson.c_str(), skipLayout, static_cast<uint8_t>(format));
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getSchemaByPathAsync(
    const std::string& handleId,
    const std::string& path,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, path]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_schema_by_path(it->second, path.c_str());
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "null";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getSchemaByPathsAsync(
    const std::string& handleId,
    const std::string& pathsJson,
    int format,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, pathsJson, format]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_schema_by_paths(it->second, pathsJson.c_str(), static_cast<uint8_t>(format));
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
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

void JsonEvalBridge::reloadSchemaMsgpackAsync(
    const std::string& handleId,
    const std::vector<uint8_t>& schemaMsgpack,
    const std::string& context,
    const std::string& data,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, schemaMsgpack, context, data]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* dt = data.empty() ? nullptr : data.c_str();
        FFIResult result = json_eval_reload_schema_msgpack(
            it->second,
            schemaMsgpack.data(),
            schemaMsgpack.size(),
            ctx,
            dt
        );
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        json_eval_free_result(result);
        return "{}";
    }, callback);
}

void JsonEvalBridge::reloadSchemaFromCacheAsync(
    const std::string& handleId,
    const std::string& cacheKey,
    const std::string& context,
    const std::string& data,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, cacheKey, context, data]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* dt = data.empty() ? nullptr : data.c_str();
        FFIResult result = json_eval_reload_schema_from_cache(
            it->second,
            cacheKey.c_str(),
            ctx,
            dt
        );
        
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
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
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
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "0";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::enableCacheAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_enable_cache(it->second);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        json_eval_free_result(result);
        return "";
    }, callback);
}

void JsonEvalBridge::disableCacheAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_disable_cache(it->second);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        json_eval_free_result(result);
        return "";
    }, callback);
}

bool JsonEvalBridge::isCacheEnabled(const std::string& handleId) {
    std::lock_guard<std::mutex> lock(handlesMutex);
    auto it = handles.find(handleId);
    if (it == handles.end()) {
        return false;
    }
    
    int result = json_eval_is_cache_enabled(it->second);
    return result != 0;
}

void JsonEvalBridge::resolveLayoutAsync(
    const std::string& handleId,
    bool evaluate,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, evaluate]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_resolve_layout(it->second, evaluate);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        json_eval_free_result(result);
        return "{}";
    }, callback);
}

void JsonEvalBridge::compileAndRunLogicAsync(
    const std::string& handleId,
    const std::string& logicStr,
    const std::string& data,
    const std::string& context,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, logicStr, data, context]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* dataPtr = data.empty() ? nullptr : data.c_str();
        const char* contextPtr = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_compile_and_run_logic(it->second, logicStr.c_str(), dataPtr, contextPtr);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "null";
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
        
        // Zero-copy: construct string directly from raw pointer
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

// ============================================================================
// Subform Methods
// ============================================================================

void JsonEvalBridge::evaluateSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    const std::string& data,
    const std::string& context,
    const std::string& pathsJson,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath, data, context, pathsJson]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* pathsPtr = pathsJson.empty() ? nullptr : pathsJson.c_str();
        FFIResult result = json_eval_evaluate_subform(
            it->second, 
            subformPath.c_str(), 
            data.c_str(), 
            ctx,
            pathsPtr
        );
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        json_eval_free_result(result);
        return "{}";
    }, callback);
}

void JsonEvalBridge::validateSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    const std::string& data,
    const std::string& context,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath, data, context]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* ctx = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_validate_subform(it->second, subformPath.c_str(), data.c_str(), ctx);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{\"hasError\":false,\"errors\":[]}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::evaluateDependentsSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    const std::string& changedPath,
    const std::string& data,
    const std::string& context,
    bool reEvaluate,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath, changedPath, data, context, reEvaluate]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        const char* dt = data.empty() ? nullptr : data.c_str();
        const char* ctx = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_evaluate_dependents_subform(it->second, subformPath.c_str(), changedPath.c_str(), dt, ctx, reEvaluate ? 1 : 0);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "[]";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::resolveLayoutSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    bool evaluate,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath, evaluate]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_resolve_layout_subform(it->second, subformPath.c_str(), evaluate);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        json_eval_free_result(result);
        return "{}";
    }, callback);
}

void JsonEvalBridge::getEvaluatedSchemaSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    bool resolveLayout,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath, resolveLayout]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_evaluated_schema_subform(it->second, subformPath.c_str(), resolveLayout);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getSchemaValueSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_schema_value_subform(it->second, subformPath.c_str());
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getEvaluatedSchemaWithoutParamsSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    bool resolveLayout,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath, resolveLayout]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_evaluated_schema_without_params_subform(it->second, subformPath.c_str(), resolveLayout);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getEvaluatedSchemaByPathSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    const std::string& schemaPath,
    bool skipLayout,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath, schemaPath, skipLayout]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_evaluated_schema_by_path_subform(it->second, subformPath.c_str(), schemaPath.c_str(), skipLayout);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "null";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getEvaluatedSchemaByPathsSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    const std::string& schemaPathsJson,
    bool skipLayout,
    int format,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath, schemaPathsJson, skipLayout, format]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_evaluated_schema_by_paths_subform(it->second, subformPath.c_str(), schemaPathsJson.c_str(), skipLayout, static_cast<uint8_t>(format));
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getSubformPathsAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_subform_paths(it->second);
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "[]";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getSchemaByPathSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    const std::string& schemaPath,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath, schemaPath]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_schema_by_path_subform(it->second, subformPath.c_str(), schemaPath.c_str());
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "null";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::getSchemaByPathsSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    const std::string& schemaPathsJson,
    int format,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath, schemaPathsJson, format]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_get_schema_by_paths_subform(it->second, subformPath.c_str(), schemaPathsJson.c_str(), static_cast<uint8_t>(format));
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(result);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::hasSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, subformPath]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        FFIResult result = json_eval_has_subform(it->second, subformPath.c_str());
        
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (result.data_ptr && result.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
        } else {
            resultStr = "false";
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

void JsonEvalBridge::setTimezoneOffsetAsync(
    const std::string& handleId,
    int32_t offsetMinutes,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runAsync([handleId, offsetMinutes]() -> std::string {
        std::lock_guard<std::mutex> lock(handlesMutex);
        auto it = handles.find(handleId);
        if (it == handles.end()) {
            throw std::runtime_error("Invalid handle");
        }
        
        json_eval_set_timezone_offset(it->second, offsetMinutes);
        return "{}";
    }, callback);
}

void JsonEvalBridge::setTimezoneOffset(
    const std::string& handleId,
    int32_t offsetMinutes
) {
    std::lock_guard<std::mutex> lock(handlesMutex);
    auto it = handles.find(handleId);
    if (it == handles.end()) {
        throw std::runtime_error("Invalid handle");
    }

    json_eval_set_timezone_offset(it->second, offsetMinutes);
}



void JsonEvalBridge::cancel(const std::string& handle) {
    std::lock_guard<std::mutex> lock(handlesMutex);
    auto it = handles.find(handle);
    if (it != handles.end()) {
        json_eval_cancel(it->second);
    }
}

std::string JsonEvalBridge::version() {
    const char* ver = json_eval_version();
    // Version string is static in Rust, no need to free it
    return ver ? ver : "unknown";
}

} // namespace jsoneval
