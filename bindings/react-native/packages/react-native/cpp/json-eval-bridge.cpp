#include "json-eval-bridge.h"
#include <map>
#include <mutex>
#include <memory>
#include <queue>
#include <condition_variable>
#include <thread>
#include <functional>
#include <vector>

// Small fixed thread pool -- reuses threads instead of spawn+detach per call
class SimpleThreadPool {
public:
    SimpleThreadPool(size_t numThreads = 4) : stop(false) {
        for (size_t i = 0; i < numThreads; ++i) {
            workers.emplace_back([this] {
                for (;;) {
                    std::function<void()> task;
                    {
                        std::unique_lock<std::mutex> lock(queueMutex);
                        condition.wait(lock, [this] { return stop || !tasks.empty(); });
                        if (stop && tasks.empty()) return;
                        task = std::move(tasks.front());
                        tasks.pop();
                    }
                    task();
                }
            });
        }
    }

    ~SimpleThreadPool() {
        {
            std::unique_lock<std::mutex> lock(queueMutex);
            stop = true;
        }
        condition.notify_all();
        for (std::thread& worker : workers) {
            if (worker.joinable()) worker.join();
        }
    }

    template<class F>
    void enqueue(F&& f) {
        {
            std::unique_lock<std::mutex> lock(queueMutex);
            if (stop) return;
            tasks.emplace(std::forward<F>(f));
        }
        condition.notify_one();
    }

private:
    std::vector<std::thread> workers;
    std::queue<std::function<void()>> tasks;
    std::mutex queueMutex;
    std::condition_variable condition;
    bool stop;
};

// One pool shared across all bridge calls
static SimpleThreadPool gThreadPool;

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
    FFIResult json_eval_get_evaluated_schema_msgpack(JSONEvalHandle* handle);
    FFIResult json_eval_validate(JSONEvalHandle* handle, const char* data, const char* context);
    FFIResult json_eval_evaluate_dependents(JSONEvalHandle* handle, const char* changed_path, const char* data, const char* context, int re_evaluate, int include_subforms);
    FFIResult json_eval_get_evaluated_schema(JSONEvalHandle* handle);
    FFIResult json_eval_get_schema_value(JSONEvalHandle* handle);
    FFIResult json_eval_get_schema_value_array(JSONEvalHandle* handle);
    FFIResult json_eval_get_schema_value_object(JSONEvalHandle* handle);
    FFIResult json_eval_get_evaluated_schema_without_params(JSONEvalHandle* handle);
    FFIResult json_eval_get_evaluated_schema_by_path(JSONEvalHandle* handle, const char* path);
    FFIResult json_eval_get_evaluated_schema_by_paths(JSONEvalHandle* handle, const char* paths_json, uint8_t format);
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
    FFIResult json_eval_validate_paths(JSONEvalHandle* handle, const char* data, const char* context, const char* paths_json);
    FFIResult json_eval_evaluate_logic_pure(const char* logic_str, const char* data, const char* context);
    
    // Subform FFI methods
    FFIResult json_eval_evaluate_subform(JSONEvalHandle* handle, const char* subform_path, const char* data, const char* context, const char* paths_json);
    FFIResult json_eval_validate_subform(JSONEvalHandle* handle, const char* subform_path, const char* data, const char* context);
    FFIResult json_eval_evaluate_dependents_subform(JSONEvalHandle* handle, const char* subform_path, const char* changed_path, const char* data, const char* context, int re_evaluate, int include_subforms);
    FFIResult json_eval_resolve_layout_subform(JSONEvalHandle* handle, const char* subform_path, bool evaluate);
    FFIResult json_eval_get_evaluated_schema_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_get_schema_value_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_get_schema_value_array_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_get_schema_value_object_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_get_evaluated_schema_without_params_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_get_evaluated_schema_by_path_subform(JSONEvalHandle* handle, const char* subform_path, const char* schema_path);
    FFIResult json_eval_get_evaluated_schema_by_paths_subform(JSONEvalHandle* handle, const char* subform_path, const char* schema_paths_json, uint8_t format);
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

// Per-handle storage with per-handle mutex (not global)
// Map access protected by handlesMapMutex (brief lock for find/insert/erase)
// FFI calls on a handle protected by handle's own mutex for true concurrency
// NOTE: std::mutex is non-movable in NDK libc++, so we keep two parallel maps
static std::map<std::string, JSONEvalHandle*> handles;
static std::map<std::string, std::mutex> handleMutexes;
static std::mutex handlesMapMutex;
static int handleCounter = 0;

// ----- Helper: lock a handle for FFI access -----
// Locks per-handle mutex, returns handle pointer.
// Caller must keep the returned unique_lock alive until FFI calls complete.
static std::pair<JSONEvalHandle*, std::unique_lock<std::mutex>> lockHandle(
    const std::string& handleId)
{
    std::lock_guard<std::mutex> mapLock(handlesMapMutex);
    auto it = handles.find(handleId);
    if (it == handles.end()) {
        throw std::runtime_error("Invalid handle");
    }
    // Lock per-handle mutex while still under map lock so the entry can't be erased
    std::unique_lock<std::mutex> handleLock(handleMutexes[handleId]);
    return {it->second, std::move(handleLock)};
}

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
    
    std::lock_guard<std::mutex> lock(handlesMapMutex);
    std::string handleId = "handle_" + std::to_string(handleCounter++);
    handles[handleId] = handle;
    handleMutexes.try_emplace(handleId);  // default-constructs mutex in-place
    
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
    
    std::lock_guard<std::mutex> lock(handlesMapMutex);
    std::string handleId = "handle_" + std::to_string(handleCounter++);
    handles[handleId] = handle;
    handleMutexes.try_emplace(handleId);
    
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
    
    std::lock_guard<std::mutex> lock(handlesMapMutex);
    std::string handleId = "handle_" + std::to_string(handleCounter++);
    handles[handleId] = handle;
    handleMutexes.try_emplace(handleId);
    
    return handleId;
}

template<typename Func>
void JsonEvalBridge::runAsync(Func&& func, std::function<void(const std::string&, const std::string&)> callback) {
    gThreadPool.enqueue([func = std::forward<Func>(func), callback]() {
        try {
            std::string result = func();
            callback(result, "");
        } catch (const std::exception& e) {
            callback("", e.what());
        }
    });
}

// ============================================================================
// Helper: execute FFI on a locked handle inside runAsync
// Handles map lookup + per-handle mutex lock in one step
// ============================================================================

// Wraps a lambda that receives (JSONEvalHandle*) and returns std::string
template<typename Fn>
static void runWithHandle(
    const std::string& handleId,
    Fn&& fn,
    std::function<void(const std::string&, const std::string&)> callback)
{
    gThreadPool.enqueue([handleId, fn = std::forward<Fn>(fn), callback]() {
        try {
            auto [nativeHandle, handleLock] = lockHandle(handleId);
            std::string result = fn(nativeHandle);
            callback(result, "");
        } catch (const std::exception& e) {
            callback("", e.what());
        }
    });
}

void JsonEvalBridge::evaluateAsync(
    const std::string& handleId,
    const std::string& data,
    const std::string& context,
    const std::string& pathsJson,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [data, context, pathsJson](JSONEvalHandle* nativeHandle) -> std::string {
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* pathsPtr = pathsJson.empty() ? nullptr : pathsJson.c_str();
        
        // Step 1: Evaluate
        FFIResult evalResult = json_eval_evaluate(nativeHandle, data.c_str(), ctx, pathsPtr);
        if (!evalResult.success) {
            std::string error = evalResult.error ? evalResult.error : "Unknown error";
            json_eval_free_result(evalResult);
            throw std::runtime_error(error);
        }
        json_eval_free_result(evalResult);
        
        // Step 2: Get evaluated schema
        FFIResult schemaResult = json_eval_get_evaluated_schema(nativeHandle);
        if (!schemaResult.success) {
            std::string error = schemaResult.error ? schemaResult.error : "Unknown error";
            json_eval_free_result(schemaResult);
            throw std::runtime_error(error);
        }
        
        std::string resultStr;
        if (schemaResult.data_ptr && schemaResult.data_len > 0) {
            resultStr.assign(reinterpret_cast<const char*>(schemaResult.data_ptr), schemaResult.data_len);
        } else {
            resultStr = "{}";
        }
        json_eval_free_result(schemaResult);
        return resultStr;
    }, callback);
}

void JsonEvalBridge::evaluateOnlyAsync(
    const std::string& handleId,
    const std::string& data,
    const std::string& context,
    const std::string& pathsJson,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [data, context, pathsJson](JSONEvalHandle* nativeHandle) -> std::string {
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* pathsPtr = pathsJson.empty() ? nullptr : pathsJson.c_str();
        
        FFIResult evalResult = json_eval_evaluate(nativeHandle, data.c_str(), ctx, pathsPtr);
        if (!evalResult.success) {
            std::string error = evalResult.error ? evalResult.error : "Unknown error";
            json_eval_free_result(evalResult);
            throw std::runtime_error(error);
        }
        json_eval_free_result(evalResult);
        return "";
    }, callback);
}

uint64_t JsonEvalBridge::compileLogic(
    const std::string& handleId,
    const std::string& logicStr
) {
    auto [nativeHandle, handleLock] = lockHandle(handleId);
    uint64_t logicId = json_eval_compile_logic(nativeHandle, logicStr.c_str());
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
    runWithHandle(handleId, [logicId, data, context](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_run_logic(
            nativeHandle,
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
    runWithHandle(handleId, [data, context](JSONEvalHandle* nativeHandle) -> std::string {
        const char* ctx = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_validate(nativeHandle, data.c_str(), ctx);
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

void JsonEvalBridge::evaluateLogicAsync(
    const std::string& logicStr,
    const std::string& data,
    const std::string& context,
    std::function<void(const std::string&, const std::string&)> callback
) {
    gThreadPool.enqueue([logicStr, data, context, callback]() {
        try {
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
            callback(resultStr, "");
        } catch (const std::exception& e) {
            callback("", e.what());
        }
    });
}

void JsonEvalBridge::evaluateDependentsAsync(
    const std::string& handleId,
    const std::string& changedPathsJson,
    const std::string& data,
    const std::string& context,
    bool reEvaluate,
    bool includeSubforms,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [changedPathsJson, data, context, reEvaluate, includeSubforms](JSONEvalHandle* nativeHandle) -> std::string {
        const char* dataPtr = data.empty() ? nullptr : data.c_str();
        const char* ctx = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_evaluate_dependents(
            nativeHandle,
            changedPathsJson.c_str(),
            dataPtr,
            ctx,
            reEvaluate ? 1 : 0,
            includeSubforms ? 1 : 0
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

void JsonEvalBridge::getEvaluatedSchemaAsync(
    const std::string& handleId,
    bool skipLayout,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema(nativeHandle);
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

void JsonEvalBridge::getEvaluatedSchemaMsgpackAsync(
    const std::string& handleId,
    bool skipLayout,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema_msgpack(nativeHandle);
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
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
    runWithHandle(handleId, [](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_schema_value(nativeHandle);
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

void JsonEvalBridge::getSchemaValueArrayAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_schema_value_array(nativeHandle);
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

void JsonEvalBridge::getSchemaValueObjectAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_schema_value_object(nativeHandle);
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

void JsonEvalBridge::getEvaluatedSchemaWithoutParamsAsync(
    const std::string& handleId,
    bool skipLayout,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema_without_params(nativeHandle);
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

void JsonEvalBridge::getEvaluatedSchemaByPathAsync(
    const std::string& handleId,
    const std::string& path,
    bool skipLayout,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [path](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema_by_path(nativeHandle, path.c_str());
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

void JsonEvalBridge::getEvaluatedSchemaByPathsAsync(
    const std::string& handleId,
    const std::string& pathsJson,
    bool skipLayout,
    int format,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [pathsJson, format](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema_by_paths(nativeHandle, pathsJson.c_str(), static_cast<uint8_t>(format));
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

void JsonEvalBridge::getSchemaByPathAsync(
    const std::string& handleId,
    const std::string& path,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [path](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_schema_by_path(nativeHandle, path.c_str());
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

void JsonEvalBridge::getSchemaByPathsAsync(
    const std::string& handleId,
    const std::string& pathsJson,
    int format,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [pathsJson, format](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_schema_by_paths(nativeHandle, pathsJson.c_str(), static_cast<uint8_t>(format));
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

void JsonEvalBridge::reloadSchemaAsync(
    const std::string& handleId,
    const std::string& schema,
    const std::string& context,
    const std::string& data,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [schema, context, data](JSONEvalHandle* nativeHandle) -> std::string {
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* dt = data.empty() ? nullptr : data.c_str();
        FFIResult result = json_eval_reload_schema(nativeHandle, schema.c_str(), ctx, dt);
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
    runWithHandle(handleId, [schemaMsgpack, context, data](JSONEvalHandle* nativeHandle) -> std::string {
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* dt = data.empty() ? nullptr : data.c_str();
        FFIResult result = json_eval_reload_schema_msgpack(
            nativeHandle,
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
    runWithHandle(handleId, [cacheKey, context, data](JSONEvalHandle* nativeHandle) -> std::string {
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* dt = data.empty() ? nullptr : data.c_str();
        FFIResult result = json_eval_reload_schema_from_cache(
            nativeHandle,
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

void JsonEvalBridge::resolveLayoutAsync(
    const std::string& handleId,
    bool evaluate,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [evaluate](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_resolve_layout(nativeHandle, evaluate);
        if (!result.success) {
            std::string error = result.error ? result.error : "Unknown error";
            json_eval_free_result(result);
            throw std::runtime_error(error);
        }
        json_eval_free_result(result);
        return "{}";
    }, callback);
}

void JsonEvalBridge::getResolvedLayoutAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_resolved_layout(nativeHandle);
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

void JsonEvalBridge::getEvaluatedSchemaResolvedAsync(
    const std::string& handleId,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema_resolved(nativeHandle);
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

void JsonEvalBridge::getEvaluatedSchemaResolvedSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [subformPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema_resolved_subform(nativeHandle, subformPath.c_str());
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

void JsonEvalBridge::compileAndRunLogicAsync(
    const std::string& handleId,
    const std::string& logicStr,
    const std::string& data,
    const std::string& context,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [logicStr, data, context](JSONEvalHandle* nativeHandle) -> std::string {
        const char* dataPtr = data.empty() ? nullptr : data.c_str();
        const char* contextPtr = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_compile_and_run_logic(nativeHandle, logicStr.c_str(), dataPtr, contextPtr);
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

void JsonEvalBridge::validatePathsAsync(
    const std::string& handleId,
    const std::string& data,
    const std::string& context,
    const std::string& pathsJson,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [data, context, pathsJson](JSONEvalHandle* nativeHandle) -> std::string {
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* paths = pathsJson.empty() ? nullptr : pathsJson.c_str();
        FFIResult result = json_eval_validate_paths(nativeHandle, data.c_str(), ctx, paths);
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
    runWithHandle(handleId, [subformPath, data, context, pathsJson](JSONEvalHandle* nativeHandle) -> std::string {
        const char* ctx = context.empty() ? nullptr : context.c_str();
        const char* pathsPtr = pathsJson.empty() ? nullptr : pathsJson.c_str();
        FFIResult result = json_eval_evaluate_subform(nativeHandle, subformPath.c_str(), data.c_str(), ctx, pathsPtr);
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
    runWithHandle(handleId, [subformPath, data, context](JSONEvalHandle* nativeHandle) -> std::string {
        const char* ctx = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_validate_subform(nativeHandle, subformPath.c_str(), data.c_str(), ctx);
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
    bool includeSubforms,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [subformPath, changedPath, data, context, reEvaluate, includeSubforms](JSONEvalHandle* nativeHandle) -> std::string {
        const char* dt = data.empty() ? nullptr : data.c_str();
        const char* ctx = context.empty() ? nullptr : context.c_str();
        FFIResult result = json_eval_evaluate_dependents_subform(
            nativeHandle, subformPath.c_str(), changedPath.c_str(), dt, ctx,
            reEvaluate ? 1 : 0, includeSubforms ? 1 : 0
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
    runWithHandle(handleId, [subformPath, evaluate](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_resolve_layout_subform(nativeHandle, subformPath.c_str(), evaluate);
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
    runWithHandle(handleId, [subformPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema_subform(nativeHandle, subformPath.c_str());
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
    runWithHandle(handleId, [subformPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_schema_value_subform(nativeHandle, subformPath.c_str());
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

void JsonEvalBridge::getSchemaValueArraySubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [subformPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_schema_value_array_subform(nativeHandle, subformPath.c_str());
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

void JsonEvalBridge::getSchemaValueObjectSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [subformPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_schema_value_object_subform(nativeHandle, subformPath.c_str());
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
    runWithHandle(handleId, [subformPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema_without_params_subform(nativeHandle, subformPath.c_str());
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
    runWithHandle(handleId, [subformPath, schemaPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema_by_path_subform(nativeHandle, subformPath.c_str(), schemaPath.c_str());
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
    runWithHandle(handleId, [subformPath, schemaPathsJson, format](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_evaluated_schema_by_paths_subform(nativeHandle, subformPath.c_str(), schemaPathsJson.c_str(), static_cast<uint8_t>(format));
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
    runWithHandle(handleId, [](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_subform_paths(nativeHandle);
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
    runWithHandle(handleId, [subformPath, schemaPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_schema_by_path_subform(nativeHandle, subformPath.c_str(), schemaPath.c_str());
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
    runWithHandle(handleId, [subformPath, schemaPathsJson, format](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_schema_by_paths_subform(nativeHandle, subformPath.c_str(), schemaPathsJson.c_str(), static_cast<uint8_t>(format));
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

void JsonEvalBridge::getResolvedLayoutSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [subformPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_resolved_layout_subform(nativeHandle, subformPath.c_str());
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

void JsonEvalBridge::getFieldOptionsAsync(
    const std::string& handleId,
    const std::string& fieldPath,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [fieldPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_get_field_options(nativeHandle, fieldPath.c_str());
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

void JsonEvalBridge::hasSubformAsync(
    const std::string& handleId,
    const std::string& subformPath,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [subformPath](JSONEvalHandle* nativeHandle) -> std::string {
        FFIResult result = json_eval_has_subform(nativeHandle, subformPath.c_str());
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
    JSONEvalHandle* nativeHandle = nullptr;
    {
        std::lock_guard<std::mutex> mapLock(handlesMapMutex);
        auto it = handles.find(handleId);
        if (it != handles.end()) {
            // Lock the handle's mutex to wait for any in-flight FFI call to finish
            std::lock_guard<std::mutex> handleLock(handleMutexes[handleId]);
            nativeHandle = it->second;
            handles.erase(it);
            handleMutexes.erase(handleId);
        }
    }
    if (nativeHandle) {
        json_eval_free(nativeHandle);
    }
}

void JsonEvalBridge::setTimezoneOffsetAsync(
    const std::string& handleId,
    int32_t offsetMinutes,
    std::function<void(const std::string&, const std::string&)> callback
) {
    runWithHandle(handleId, [offsetMinutes](JSONEvalHandle* nativeHandle) -> std::string {
        json_eval_set_timezone_offset(nativeHandle, offsetMinutes);
        return "{}";
    }, callback);
}

void JsonEvalBridge::setTimezoneOffset(
    const std::string& handleId,
    int32_t offsetMinutes
) {
    auto [nativeHandle, handleLock] = lockHandle(handleId);
    json_eval_set_timezone_offset(nativeHandle, offsetMinutes);
}

void JsonEvalBridge::cancel(const std::string& handleId) {
    std::lock_guard<std::mutex> mapLock(handlesMapMutex);
    auto it = handles.find(handleId);
    if (it != handles.end()) {
        std::lock_guard<std::mutex> handleLock(handleMutexes[handleId]);
        json_eval_cancel(it->second);
    }
}

std::string JsonEvalBridge::version() {
    const char* ver = json_eval_version();
    return ver ? ver : "unknown";
}

} // namespace jsoneval
