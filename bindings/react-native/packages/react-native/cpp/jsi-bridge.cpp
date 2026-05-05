#include "jsi-bridge.h"
#include "json-eval-bridge.h"

// C FFI function declarations (types defined in jsi-bridge.h)
extern "C" {
    JSONEvalHandle* json_eval_new(const char* schema, const char* context, const char* data);
    JSONEvalHandle* json_eval_new_from_msgpack(const uint8_t* schema_msgpack, size_t schema_len, const char* context, const char* data);
    JSONEvalHandle* json_eval_new_from_cache(const char* cache_key, const char* context, const char* data);
    FFIResult json_eval_evaluate(JSONEvalHandle* handle, const char* data, const char* context, const char* paths_json);
    FFIResult json_eval_get_evaluated_schema(JSONEvalHandle* handle, bool skip_layout);
    FFIResult json_eval_get_schema_value(JSONEvalHandle* handle);
    FFIResult json_eval_get_schema_value_array(JSONEvalHandle* handle);
    FFIResult json_eval_get_schema_value_object(JSONEvalHandle* handle);
    FFIResult json_eval_validate(JSONEvalHandle* handle, const char* data, const char* context);
    FFIResult json_eval_validate_paths(JSONEvalHandle* handle, const char* data, const char* context, const char* paths_json);
    FFIResult json_eval_evaluate_dependents(JSONEvalHandle* handle, const char* changed_path, const char* data, const char* context, int re_evaluate, int include_subforms);
    FFIResult json_eval_get_evaluated_schema_by_path(JSONEvalHandle* handle, const char* path, bool skip_layout);
    FFIResult json_eval_get_evaluated_schema_by_paths(JSONEvalHandle* handle, const char* paths_json, bool skip_layout, uint8_t format);
    FFIResult json_eval_get_schema_by_path(JSONEvalHandle* handle, const char* path);
    FFIResult json_eval_get_schema_by_paths(JSONEvalHandle* handle, const char* paths_json, uint8_t format);
    FFIResult json_eval_get_evaluated_schema_without_params(JSONEvalHandle* handle, bool skip_layout);
    FFIResult json_eval_resolve_layout(JSONEvalHandle* handle, bool evaluate);
    FFIResult json_eval_compile_and_run_logic(JSONEvalHandle* handle, const char* logic_str, const char* data, const char* context);
    uint64_t json_eval_compile_logic(JSONEvalHandle* handle, const char* logic_str);
    FFIResult json_eval_run_logic(JSONEvalHandle* handle, uint64_t logic_id, const char* data, const char* context);
    FFIResult json_eval_reload_schema(JSONEvalHandle* handle, const char* schema, const char* context, const char* data);
    FFIResult json_eval_reload_schema_msgpack(JSONEvalHandle* handle, const uint8_t* schema_msgpack, size_t schema_len, const char* context, const char* data);
    FFIResult json_eval_reload_schema_from_cache(JSONEvalHandle* handle, const char* cache_key, const char* context, const char* data);
    void json_eval_set_timezone_offset(JSONEvalHandle* handle, int32_t offset_minutes);
    void json_eval_free(JSONEvalHandle* handle);
    void json_eval_free_result(FFIResult result);
    const char* json_eval_version();
    void json_eval_free_string(char* ptr);

    // Subform FFI methods
    FFIResult json_eval_evaluate_subform(JSONEvalHandle* handle, const char* subform_path, const char* data, const char* context, const char* paths_json);
    FFIResult json_eval_validate_subform(JSONEvalHandle* handle, const char* subform_path, const char* data, const char* context);
    FFIResult json_eval_evaluate_dependents_subform(JSONEvalHandle* handle, const char* subform_path, const char* changed_path, const char* data, const char* context, int re_evaluate, int include_subforms);
    FFIResult json_eval_resolve_layout_subform(JSONEvalHandle* handle, const char* subform_path, bool evaluate);
    FFIResult json_eval_get_evaluated_schema_subform(JSONEvalHandle* handle, const char* subform_path, bool resolve_layout);
    FFIResult json_eval_get_schema_value_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_get_schema_value_array_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_get_schema_value_object_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_get_evaluated_schema_without_params_subform(JSONEvalHandle* handle, const char* subform_path, bool resolve_layout);
    FFIResult json_eval_get_evaluated_schema_by_path_subform(JSONEvalHandle* handle, const char* subform_path, const char* schema_path, bool skip_layout);
    FFIResult json_eval_get_evaluated_schema_by_paths_subform(JSONEvalHandle* handle, const char* subform_path, const char* schema_paths_json, bool skip_layout, uint8_t format);
    FFIResult json_eval_get_schema_by_path_subform(JSONEvalHandle* handle, const char* subform_path, const char* schema_path);
    FFIResult json_eval_get_schema_by_paths_subform(JSONEvalHandle* handle, const char* subform_path, const char* schema_paths_json, uint8_t format);
    FFIResult json_eval_get_subform_paths(JSONEvalHandle* handle);
    FFIResult json_eval_has_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_evaluate_logic_pure(const char* logic_str, const char* data, const char* context);
}

namespace jsoneval {

// ---------------------------------------------------------------------------
// Handle storage — separate from json-eval-bridge.cpp to avoid cross-module
// static variable conflicts when both are linked. JSI uses its own map.
// ---------------------------------------------------------------------------
static std::unordered_map<std::string, JSONEvalHandle*> s_handles;
static std::unordered_map<std::string, std::mutex> s_handleMutexes;
static std::mutex s_mapMutex;
static int s_handleCounter = 0;

static std::string createHandleId() {
    std::lock_guard<std::mutex> lock(s_mapMutex);
    return "jsi_handle_" + std::to_string(s_handleCounter++);
}

static void storeHandle(const std::string& id, JSONEvalHandle* handle) {
    std::lock_guard<std::mutex> lock(s_mapMutex);
    s_handles[id] = handle;
    s_handleMutexes.try_emplace(id);
}

static std::pair<JSONEvalHandle*, std::unique_lock<std::mutex>> lockHandleById(const std::string& id) {
    std::unique_lock<std::mutex> mapLock(s_mapMutex);
    auto it = s_handles.find(id);
    if (it == s_handles.end()) {
        throw std::runtime_error("Invalid JSI handle: " + id);
    }
    std::unique_lock<std::mutex> handleLock(s_handleMutexes[id]);
    mapLock.unlock();
    return {it->second, std::move(handleLock)};
}

// ---------------------------------------------------------------------------
// HostObject property implementations
// ---------------------------------------------------------------------------

bool JsonEvalJSI::install(jsi::Runtime& runtime) {
    auto hostObject = std::make_shared<JsonEvalJSI>();
    auto obj = jsi::Object::createFromHostObject(runtime, hostObject);
    runtime.global().setProperty(runtime, "jsonEval", obj);
    return true;
}

std::string JsonEvalJSI::stringFromValue(jsi::Runtime& runtime, const jsi::Value& val) {
    if (val.isNull() || val.isUndefined()) return "";
    if (val.isString()) return val.asString(runtime).utf8(runtime);
    if (val.isNumber()) return std::to_string(val.asNumber());
    if (val.isBool()) return val.asBool() ? "true" : "false";
    // Object or other — try JSON stringify via JSI JSON
    // We just return empty since caller should pass strings
    return "";
}

void JsonEvalJSI::checkResult(jsi::Runtime& runtime, const FFIResult& result) {
    if (!result.success) {
        std::string err = result.error ? result.error : "Unknown error";
        json_eval_free_result(const_cast<FFIResult&>(result));
        throw jsi::JSError(runtime, err);
    }
}

void JsonEvalJSI::checkArgCount(jsi::Runtime& runtime, size_t actual, size_t expected) {
    if (actual < expected) {
        throw jsi::JSError(runtime, "Expected " + std::to_string(expected) +
            " arguments but got " + std::to_string(actual));
    }
}

// ---------------------------------------------------------------------------
// Helper: converts FFI result data_ptr+data_len → jsi::String (JSON)
// ---------------------------------------------------------------------------
static jsi::Value ffiResultToJsiString(jsi::Runtime& runtime, FFIResult& result, const char* fallback) {
    JsonEvalJSI::checkResult(runtime, result);
    std::string str;
    if (result.data_ptr && result.data_len > 0) {
        str.assign(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
    } else {
        str = fallback;
    }
    json_eval_free_result(result);
    return jsi::String::createFromUtf8(runtime, str);
}

// ---------------------------------------------------------------------------
// Helper: create JSI function with a lambda wrapping a Fn(const string& → string)
// ---------------------------------------------------------------------------
template<typename Fn>
static jsi::Value createJsiFn(jsi::Runtime& runtime, const char* name, Fn&& fn) {
    return jsi::Function::createFromHostFunction(
        runtime,
        jsi::PropNameID::forAscii(runtime, name),
        0,
        [fn = std::forward<Fn>(fn), name](jsi::Runtime& rt, const jsi::Value&, const jsi::Value* args, size_t count) -> jsi::Value {
            try {
                return fn(rt, args, count);
            } catch (const jsi::JSError&) {
                throw;
            } catch (const std::exception& e) {
                throw jsi::JSError(rt, std::string("jsonEval.") + name + ": " + e.what());
            }
        }
    );
}

// ---------------------------------------------------------------------------
// get() — dispatched by property name
// ---------------------------------------------------------------------------
jsi::Value JsonEvalJSI::get(jsi::Runtime& runtime, const jsi::PropNameID& name) {
    auto prop = name.utf8(runtime);

    // ---- Create ----
    if (prop == "create") {
        return createJsiFn(runtime, "create",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto schema = stringFromValue(rt, args[0]);
                auto ctx = count > 1 ? stringFromValue(rt, args[1]) : "";
                auto data = count > 2 ? stringFromValue(rt, args[2]) : "";
                JSONEvalHandle* handle = json_eval_new(
                    schema.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    data.empty() ? nullptr : data.c_str()
                );
                if (!handle) throw jsi::JSError(rt, "Failed to create JSONEval instance");
                auto id = createHandleId();
                storeHandle(id, handle);
                return jsi::String::createFromUtf8(rt, id);
            }
        );
    }

    // ---- createFromMsgpack ----
    if (prop == "createFromMsgpack") {
        return createJsiFn(runtime, "createFromMsgpack",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                // First arg: JSI Array of uint8 bytes or ArrayBuffer
                std::vector<uint8_t> msgpackBytes;
                if (args[0].isObject()) {
                    auto obj = args[0].asObject(rt);
                    if (obj.isArray(rt)) {
                        auto arr = obj.asArray(rt);
                        size_t len = arr.size(rt);
                        msgpackBytes.reserve(len);
                        for (size_t i = 0; i < len; i++) {
                            msgpackBytes.push_back(static_cast<uint8_t>(
                                arr.getValueAtIndex(rt, i).asNumber()));
                        }
                    } else if (obj.isArrayBuffer(rt)) {
                        auto buf = obj.getArrayBuffer(rt);
                        auto* data = buf.data(rt);
                        msgpackBytes.assign(data, data + buf.length(rt));
                    }
                }
                auto ctx = count > 1 ? stringFromValue(rt, args[1]) : "";
                auto data = count > 2 ? stringFromValue(rt, args[2]) : "";
                JSONEvalHandle* handle = json_eval_new_from_msgpack(
                    msgpackBytes.data(), msgpackBytes.size(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    data.empty() ? nullptr : data.c_str()
                );
                if (!handle) throw jsi::JSError(rt, "Failed to create JSONEval from msgpack");
                auto id = createHandleId();
                storeHandle(id, handle);
                return jsi::String::createFromUtf8(rt, id);
            }
        );
    }

    // ---- createFromCache ----
    if (prop == "createFromCache") {
        return createJsiFn(runtime, "createFromCache",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto cacheKey = stringFromValue(rt, args[0]);
                auto ctx = count > 1 ? stringFromValue(rt, args[1]) : "";
                auto data = count > 2 ? stringFromValue(rt, args[2]) : "";
                JSONEvalHandle* handle = json_eval_new_from_cache(
                    cacheKey.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    data.empty() ? nullptr : data.c_str()
                );
                if (!handle) throw jsi::JSError(rt, "Failed to create JSONEval from cache");
                auto id = createHandleId();
                storeHandle(id, handle);
                return jsi::String::createFromUtf8(rt, id);
            }
        );
    }

    // ---- evaluateOnly (void return, no serialization) ----
    if (prop == "evaluateOnly") {
        return createJsiFn(runtime, "evaluateOnly",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto data = stringFromValue(rt, args[1]);
                auto ctx = count > 2 ? stringFromValue(rt, args[2]) : "";
                auto paths = count > 3 ? stringFromValue(rt, args[3]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_evaluate(
                    handle,
                    data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    paths.empty() ? nullptr : paths.c_str()
                );
                checkResult(rt, result);
                json_eval_free_result(result);
                // Return undefined — no schema serialization
                return jsi::Value::undefined();
            }
        );
    }

    // ---- evaluate (returns evaluated schema JSON string) ----
    if (prop == "evaluate") {
        return createJsiFn(runtime, "evaluate",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto data = stringFromValue(rt, args[1]);
                auto ctx = count > 2 ? stringFromValue(rt, args[2]) : "";
                auto paths = count > 3 ? stringFromValue(rt, args[3]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                
                // Step 1: Evaluate
                FFIResult evalResult = json_eval_evaluate(
                    handle,
                    data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    paths.empty() ? nullptr : paths.c_str()
                );
                checkResult(rt, evalResult);
                json_eval_free_result(evalResult);
                
                // Step 2: Get evaluated schema
                FFIResult schemaResult = json_eval_get_evaluated_schema(handle, true);
                checkResult(rt, schemaResult);
                
                std::string str;
                if (schemaResult.data_ptr && schemaResult.data_len > 0) {
                    str.assign(reinterpret_cast<const char*>(schemaResult.data_ptr), schemaResult.data_len);
                } else {
                    str = "{}";
                }
                json_eval_free_result(schemaResult);
                return jsi::String::createFromUtf8(rt, str);
            }
        );
    }

    // ---- validate ----
    if (prop == "validate") {
        return createJsiFn(runtime, "validate",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto data = stringFromValue(rt, args[1]);
                auto ctx = count > 2 ? stringFromValue(rt, args[2]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_validate(
                    handle,
                    data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str()
                );
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- validatePaths ----
    if (prop == "validatePaths") {
        return createJsiFn(runtime, "validatePaths",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto data = stringFromValue(rt, args[1]);
                auto ctx = count > 2 ? stringFromValue(rt, args[2]) : "";
                auto paths = count > 3 ? stringFromValue(rt, args[3]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_validate_paths(
                    handle,
                    data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    paths.empty() ? nullptr : paths.c_str()
                );
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- evaluateDependents ----
    if (prop == "evaluateDependents") {
        return createJsiFn(runtime, "evaluateDependents",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto changedPaths = stringFromValue(rt, args[1]);
                auto data = count > 2 ? stringFromValue(rt, args[2]) : "";
                auto ctx = count > 3 ? stringFromValue(rt, args[3]) : "";
                bool reEvaluate = count > 4 ? args[4].asBool() : true;
                bool includeSubforms = count > 5 ? args[5].asBool() : true;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_evaluate_dependents(
                    handle,
                    changedPaths.c_str(),
                    data.empty() ? nullptr : data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    reEvaluate ? 1 : 0,
                    includeSubforms ? 1 : 0
                );
                return ffiResultToJsiString(rt, result, "[]");
            }
        );
    }

    // ---- getEvaluatedSchema ----
    if (prop == "getEvaluatedSchema") {
        return createJsiFn(runtime, "getEvaluatedSchema",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                bool skipLayout = count > 1 ? args[1].asBool() : false;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema(handle, skipLayout);
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getSchemaValue ----
    if (prop == "getSchemaValue") {
        return createJsiFn(runtime, "getSchemaValue",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_schema_value(handle);
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getSchemaValueArray ----
    if (prop == "getSchemaValueArray") {
        return createJsiFn(runtime, "getSchemaValueArray",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_schema_value_array(handle);
                return ffiResultToJsiString(rt, result, "[]");
            }
        );
    }

    // ---- getSchemaValueObject ----
    if (prop == "getSchemaValueObject") {
        return createJsiFn(runtime, "getSchemaValueObject",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_schema_value_object(handle);
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getEvaluatedSchemaByPath ----
    if (prop == "getEvaluatedSchemaByPath") {
        return createJsiFn(runtime, "getEvaluatedSchemaByPath",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto path = stringFromValue(rt, args[1]);
                bool skipLayout = count > 2 ? args[2].asBool() : false;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_by_path(handle, path.c_str(), skipLayout);
                return ffiResultToJsiString(rt, result, "null");
            }
        );
    }

    // ---- getEvaluatedSchemaByPaths ----
    if (prop == "getEvaluatedSchemaByPaths") {
        return createJsiFn(runtime, "getEvaluatedSchemaByPaths",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto pathsJson = stringFromValue(rt, args[1]);
                bool skipLayout = count > 2 ? args[2].asBool() : false;
                uint8_t format = count > 3 ? static_cast<uint8_t>(args[3].asNumber()) : 0;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_by_paths(handle, pathsJson.c_str(), skipLayout, format);
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getSchemaByPath ----
    if (prop == "getSchemaByPath") {
        return createJsiFn(runtime, "getSchemaByPath",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto path = stringFromValue(rt, args[1]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_schema_by_path(handle, path.c_str());
                return ffiResultToJsiString(rt, result, "null");
            }
        );
    }

    // ---- getSchemaByPaths ----
    if (prop == "getSchemaByPaths") {
        return createJsiFn(runtime, "getSchemaByPaths",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto pathsJson = stringFromValue(rt, args[1]);
                uint8_t format = count > 2 ? static_cast<uint8_t>(args[2].asNumber()) : 0;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_schema_by_paths(handle, pathsJson.c_str(), format);
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getEvaluatedSchemaWithoutParams ----
    if (prop == "getEvaluatedSchemaWithoutParams") {
        return createJsiFn(runtime, "getEvaluatedSchemaWithoutParams",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                bool skipLayout = count > 1 ? args[1].asBool() : false;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_without_params(handle, skipLayout);
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- resolveLayout ----
    if (prop == "resolveLayout") {
        return createJsiFn(runtime, "resolveLayout",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                bool evaluate = count > 1 ? args[1].asBool() : false;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_resolve_layout(handle, evaluate);
                checkResult(rt, result);
                json_eval_free_result(result);
                return jsi::Value::undefined();
            }
        );
    }

    // ---- compileAndRunLogic ----
    if (prop == "compileAndRunLogic") {
        return createJsiFn(runtime, "compileAndRunLogic",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto logicStr = stringFromValue(rt, args[1]);
                auto data = count > 2 ? stringFromValue(rt, args[2]) : "";
                auto ctx = count > 3 ? stringFromValue(rt, args[3]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_compile_and_run_logic(
                    handle,
                    logicStr.c_str(),
                    data.empty() ? nullptr : data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str()
                );
                return ffiResultToJsiString(rt, result, "null");
            }
        );
    }

    // ---- compileLogic ----
    if (prop == "compileLogic") {
        return createJsiFn(runtime, "compileLogic",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto logicStr = stringFromValue(rt, args[1]);
                
                auto [handle, lock] = lockHandleById(handleId);
                uint64_t logicId = json_eval_compile_logic(handle, logicStr.c_str());
                if (logicId == 0) {
                    throw jsi::JSError(rt, "Failed to compile logic");
                }
                return jsi::Value(static_cast<double>(logicId));
            }
        );
    }

    // ---- runLogic ----
    if (prop == "runLogic") {
        return createJsiFn(runtime, "runLogic",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                uint64_t logicId = static_cast<uint64_t>(args[1].asNumber());
                auto data = count > 2 ? stringFromValue(rt, args[2]) : "";
                auto ctx = count > 3 ? stringFromValue(rt, args[3]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_run_logic(
                    handle,
                    logicId,
                    data.empty() ? nullptr : data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str()
                );
                return ffiResultToJsiString(rt, result, "null");
            }
        );
    }

    // ---- reloadSchemaMsgpack ----
    if (prop == "reloadSchemaMsgpack") {
        return createJsiFn(runtime, "reloadSchemaMsgpack",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 4);
                auto handleId = stringFromValue(rt, args[0]);
                std::vector<uint8_t> msgpackBytes;
                if (args[1].isObject()) {
                    auto obj = args[1].asObject(rt);
                    if (obj.isArray(rt)) {
                        auto arr = obj.asArray(rt);
                        size_t len = arr.size(rt);
                        msgpackBytes.reserve(len);
                        for (size_t i = 0; i < len; i++) {
                            msgpackBytes.push_back(static_cast<uint8_t>(
                                arr.getValueAtIndex(rt, i).asNumber()));
                        }
                    } else if (obj.isArrayBuffer(rt)) {
                        auto buf = obj.getArrayBuffer(rt);
                        auto* data = buf.data(rt);
                        msgpackBytes.assign(data, data + buf.length(rt));
                    }
                }
                auto ctx = count > 2 ? stringFromValue(rt, args[2]) : "";
                auto data = count > 3 ? stringFromValue(rt, args[3]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_reload_schema_msgpack(
                    handle,
                    msgpackBytes.data(), msgpackBytes.size(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    data.empty() ? nullptr : data.c_str()
                );
                checkResult(rt, result);
                json_eval_free_result(result);
                return jsi::Value::undefined();
            }
        );
    }

    // ---- reloadSchema ----
    if (prop == "reloadSchema") {
        return createJsiFn(runtime, "reloadSchema",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto schema = stringFromValue(rt, args[1]);
                auto ctx = count > 2 ? stringFromValue(rt, args[2]) : "";
                auto data = count > 3 ? stringFromValue(rt, args[3]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_reload_schema(
                    handle,
                    schema.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    data.empty() ? nullptr : data.c_str()
                );
                checkResult(rt, result);
                json_eval_free_result(result);
                return jsi::Value::undefined();
            }
        );
    }

    // ---- reloadSchemaFromCache ----
    if (prop == "reloadSchemaFromCache") {
        return createJsiFn(runtime, "reloadSchemaFromCache",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto cacheKey = stringFromValue(rt, args[1]);
                auto ctx = count > 2 ? stringFromValue(rt, args[2]) : "";
                auto data = count > 3 ? stringFromValue(rt, args[3]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_reload_schema_from_cache(
                    handle,
                    cacheKey.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    data.empty() ? nullptr : data.c_str()
                );
                checkResult(rt, result);
                json_eval_free_result(result);
                return jsi::Value::undefined();
            }
        );
    }

    // ---- setTimezoneOffset ----
    if (prop == "setTimezoneOffset") {
        return createJsiFn(runtime, "setTimezoneOffset",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                int32_t offset = args[1].isNumber() ? static_cast<int32_t>(args[1].asNumber()) : INT32_MIN;
                
                auto [handle, lock] = lockHandleById(handleId);
                json_eval_set_timezone_offset(handle, offset);
                return jsi::Value::undefined();
            }
        );
    }

    // ---- dispose ----
    if (prop == "dispose") {
        return createJsiFn(runtime, "dispose",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                
                JSONEvalHandle* nativeHandle = nullptr;
                {
                    std::lock_guard<std::mutex> mapLock(s_mapMutex);
                    auto it = s_handles.find(handleId);
                    if (it != s_handles.end()) {
                        std::lock_guard<std::mutex> handleLock(s_handleMutexes[handleId]);
                        nativeHandle = it->second;
                        s_handles.erase(it);
                        s_handleMutexes.erase(handleId);
                    }
                }
                if (nativeHandle) {
                    json_eval_free(nativeHandle);
                }
                return jsi::Value::undefined();
            }
        );
    }

    // ---- evaluateLogic (static, no handle) ----
    if (prop == "evaluateLogic") {
        return createJsiFn(runtime, "evaluateLogic",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto logic = stringFromValue(rt, args[0]);
                auto data = count > 1 ? stringFromValue(rt, args[1]) : "";
                auto ctx = count > 2 ? stringFromValue(rt, args[2]) : "";
                FFIResult result = json_eval_evaluate_logic_pure(
                    logic.c_str(),
                    data.empty() ? nullptr : data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str()
                );
                return ffiResultToJsiString(rt, result, "null");
            }
        );
    }

    // ---- version ----
    if (prop == "version") {
        return createJsiFn(runtime, "version",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                const char* ver = json_eval_version();
                return jsi::String::createFromUtf8(rt, ver ? ver : "unknown");
            }
        );
    }

    // ---- Subform Methods ----

    // ---- evaluateSubform ----
    if (prop == "evaluateSubform") {
        return createJsiFn(runtime, "evaluateSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 3);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                auto data = stringFromValue(rt, args[2]);
                auto ctx = count > 3 ? stringFromValue(rt, args[3]) : "";
                auto paths = count > 4 ? stringFromValue(rt, args[4]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_evaluate_subform(
                    handle,
                    subformPath.c_str(),
                    data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    paths.empty() ? nullptr : paths.c_str()
                );
                checkResult(rt, result);
                json_eval_free_result(result);
                return jsi::Value::undefined();
            }
        );
    }

    // ---- validateSubform ----
    if (prop == "validateSubform") {
        return createJsiFn(runtime, "validateSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 3);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                auto data = stringFromValue(rt, args[2]);
                auto ctx = count > 3 ? stringFromValue(rt, args[3]) : "";
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_validate_subform(
                    handle,
                    subformPath.c_str(),
                    data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str()
                );
                return ffiResultToJsiString(rt, result, "{\"hasError\":false,\"errors\":[]}");
            }
        );
    }

    // ---- evaluateDependentsSubform ----
    if (prop == "evaluateDependentsSubform") {
        return createJsiFn(runtime, "evaluateDependentsSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 3);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                auto changedPath = stringFromValue(rt, args[2]);
                auto data = count > 3 ? stringFromValue(rt, args[3]) : "";
                auto ctx = count > 4 ? stringFromValue(rt, args[4]) : "";
                bool reEvaluate = count > 5 ? args[5].asBool() : true;
                bool includeSubforms = count > 6 ? args[6].asBool() : true;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_evaluate_dependents_subform(
                    handle, subformPath.c_str(), changedPath.c_str(),
                    data.empty() ? nullptr : data.c_str(),
                    ctx.empty() ? nullptr : ctx.c_str(),
                    reEvaluate ? 1 : 0, includeSubforms ? 1 : 0
                );
                return ffiResultToJsiString(rt, result, "[]");
            }
        );
    }

    // ---- resolveLayoutSubform ----
    if (prop == "resolveLayoutSubform") {
        return createJsiFn(runtime, "resolveLayoutSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                bool evaluate = count > 2 ? args[2].asBool() : false;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_resolve_layout_subform(handle, subformPath.c_str(), evaluate);
                checkResult(rt, result);
                json_eval_free_result(result);
                return jsi::Value::undefined();
            }
        );
    }

    // ---- getEvaluatedSchemaSubform ----
    if (prop == "getEvaluatedSchemaSubform") {
        return createJsiFn(runtime, "getEvaluatedSchemaSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                bool resolveLayout = count > 2 ? args[2].asBool() : false;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_subform(handle, subformPath.c_str(), resolveLayout);
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getSchemaValueSubform ----
    if (prop == "getSchemaValueSubform") {
        return createJsiFn(runtime, "getSchemaValueSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_schema_value_subform(handle, subformPath.c_str());
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getSchemaValueArraySubform ----
    if (prop == "getSchemaValueArraySubform") {
        return createJsiFn(runtime, "getSchemaValueArraySubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_schema_value_array_subform(handle, subformPath.c_str());
                return ffiResultToJsiString(rt, result, "[]");
            }
        );
    }

    // ---- getSchemaValueObjectSubform ----
    if (prop == "getSchemaValueObjectSubform") {
        return createJsiFn(runtime, "getSchemaValueObjectSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_schema_value_object_subform(handle, subformPath.c_str());
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getEvaluatedSchemaWithoutParamsSubform ----
    if (prop == "getEvaluatedSchemaWithoutParamsSubform") {
        return createJsiFn(runtime, "getEvaluatedSchemaWithoutParamsSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                bool resolveLayout = count > 2 ? args[2].asBool() : false;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_without_params_subform(handle, subformPath.c_str(), resolveLayout);
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getEvaluatedSchemaByPathSubform ----
    if (prop == "getEvaluatedSchemaByPathSubform") {
        return createJsiFn(runtime, "getEvaluatedSchemaByPathSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 3);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                auto schemaPath = stringFromValue(rt, args[2]);
                bool skipLayout = count > 3 ? args[3].asBool() : false;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_by_path_subform(handle, subformPath.c_str(), schemaPath.c_str(), skipLayout);
                return ffiResultToJsiString(rt, result, "null");
            }
        );
    }

    // ---- getEvaluatedSchemaByPathsSubform ----
    if (prop == "getEvaluatedSchemaByPathsSubform") {
        return createJsiFn(runtime, "getEvaluatedSchemaByPathsSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 3);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                auto schemaPathsJson = stringFromValue(rt, args[2]);
                bool skipLayout = count > 3 ? args[3].asBool() : false;
                uint8_t format = count > 4 ? static_cast<uint8_t>(args[4].asNumber()) : 0;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_by_paths_subform(handle, subformPath.c_str(), schemaPathsJson.c_str(), skipLayout, format);
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getSchemaByPathSubform ----
    if (prop == "getSchemaByPathSubform") {
        return createJsiFn(runtime, "getSchemaByPathSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 3);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                auto schemaPath = stringFromValue(rt, args[2]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_schema_by_path_subform(handle, subformPath.c_str(), schemaPath.c_str());
                return ffiResultToJsiString(rt, result, "null");
            }
        );
    }

    // ---- getSchemaByPathsSubform ----
    if (prop == "getSchemaByPathsSubform") {
        return createJsiFn(runtime, "getSchemaByPathsSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 3);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                auto schemaPathsJson = stringFromValue(rt, args[2]);
                uint8_t format = count > 3 ? static_cast<uint8_t>(args[3].asNumber()) : 0;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_schema_by_paths_subform(handle, subformPath.c_str(), schemaPathsJson.c_str(), format);
                return ffiResultToJsiString(rt, result, "{}");
            }
        );
    }

    // ---- getSubformPaths ----
    if (prop == "getSubformPaths") {
        return createJsiFn(runtime, "getSubformPaths",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_subform_paths(handle);
                return ffiResultToJsiString(rt, result, "[]");
            }
        );
    }

    // ---- hasSubform ----
    if (prop == "hasSubform") {
        return createJsiFn(runtime, "hasSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_has_subform(handle, subformPath.c_str());
                checkResult(rt, result);
                bool found = false;
                if (result.data_ptr && result.data_len > 0) {
                    std::string s(reinterpret_cast<const char*>(result.data_ptr), result.data_len);
                    found = (s == "true");
                }
                json_eval_free_result(result);
                return jsi::Value(found);
            }
        );
    }

    return jsi::Value::undefined();
}

void JsonEvalJSI::set(jsi::Runtime& runtime, const jsi::PropNameID& name, const jsi::Value& value) {
    // Read-only host object — ignore sets
}

std::vector<jsi::PropNameID> JsonEvalJSI::getPropertyNames(jsi::Runtime& runtime) {
    std::vector<const char*> names = {
        "create", "createFromMsgpack", "createFromCache",
        "evaluateOnly", "evaluate",
        "validate", "validatePaths",
        "evaluateDependents",
        "getEvaluatedSchema", "getSchemaValue", "getSchemaValueArray", "getSchemaValueObject",
        "getEvaluatedSchemaByPath", "getEvaluatedSchemaByPaths",
        "getSchemaByPath", "getSchemaByPaths",
        "getEvaluatedSchemaWithoutParams",
        "resolveLayout",
        "compileAndRunLogic", "compileLogic", "runLogic",
        "reloadSchema", "reloadSchemaMsgpack", "reloadSchemaFromCache",
        "setTimezoneOffset",
        "dispose", "evaluateLogic", "version",
        // Subform
        "evaluateSubform", "validateSubform", "evaluateDependentsSubform",
        "resolveLayoutSubform",
        "getEvaluatedSchemaSubform",
        "getSchemaValueSubform", "getSchemaValueArraySubform", "getSchemaValueObjectSubform",
        "getEvaluatedSchemaWithoutParamsSubform",
        "getEvaluatedSchemaByPathSubform", "getEvaluatedSchemaByPathsSubform",
        "getSchemaByPathSubform", "getSchemaByPathsSubform",
        "getSubformPaths", "hasSubform"
    };
    std::vector<jsi::PropNameID> result;
    for (auto& n : names) {
        result.push_back(jsi::PropNameID::forAscii(runtime, n));
    }
    return result;
}

} // namespace jsoneval
