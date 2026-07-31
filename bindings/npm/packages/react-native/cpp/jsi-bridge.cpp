#include "jsi-bridge.h"
#include "json-eval-bridge.h"
#include "RustBuffer.h"

#include <atomic>
#include <cctype>

// C FFI function declarations (types defined in jsi-bridge.h)
extern "C" {
    JSONEvalHandle* json_eval_new(const char* schema, const char* context, const char* data);
    JSONEvalHandle* json_eval_new_from_msgpack(const uint8_t* schema_msgpack, size_t schema_len, const char* context, const char* data);
    JSONEvalHandle* json_eval_new_from_cache(const char* cache_key, const char* context, const char* data);
    FFIResult json_eval_evaluate(JSONEvalHandle* handle, const char* data, const char* context, const char* paths_json);
    FFIResult json_eval_get_evaluated_schema(JSONEvalHandle* handle);
    FFIResult json_eval_get_evaluated_schema_msgpack(JSONEvalHandle* handle);
    FFIResult json_eval_get_evaluated_schema_resolved_msgpack(JSONEvalHandle* handle);
    FFIResult json_eval_get_schema_value(JSONEvalHandle* handle);
    FFIResult json_eval_get_schema_value_array(JSONEvalHandle* handle);
    FFIResult json_eval_get_schema_value_object(JSONEvalHandle* handle);
    FFIResult json_eval_validate(JSONEvalHandle* handle, const char* data, const char* context);
    FFIResult json_eval_validate_paths(JSONEvalHandle* handle, const char* data, const char* context, const char* paths_json);
    FFIResult json_eval_evaluate_dependents(JSONEvalHandle* handle, const char* changed_path, const char* data, const char* context, int re_evaluate, int include_subforms);
    FFIResult json_eval_get_evaluated_schema_by_path(JSONEvalHandle* handle, const char* path);
    FFIResult json_eval_get_evaluated_schema_by_paths(JSONEvalHandle* handle, const char* paths_json, uint8_t format);
    FFIResult json_eval_get_schema_by_path(JSONEvalHandle* handle, const char* path);
    FFIResult json_eval_get_schema_by_paths(JSONEvalHandle* handle, const char* paths_json, uint8_t format);
    FFIResult json_eval_get_evaluated_schema_without_params(JSONEvalHandle* handle);
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
    FFIResult json_eval_evaluate_logic_pure(const char* logic_str, const char* data, const char* context);
    FFIResult json_eval_get_field_options(JSONEvalHandle* handle, const char* field_path);
    FFIResult json_eval_get_resolved_layout(JSONEvalHandle* handle);
    FFIResult json_eval_get_resolved_layout_subform(JSONEvalHandle* handle, const char* subform_path);
    FFIResult json_eval_get_evaluated_schema_resolved(JSONEvalHandle* handle);
    FFIResult json_eval_get_evaluated_schema_resolved_subform(JSONEvalHandle* handle, const char* subform_path);
    void json_eval_cancel(JSONEvalHandle* handle);
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
// JSON integer preservation for FFI results.
//
// JSON.parse always materializes numbers as double. Before handing FFI JSON to
// JSI, replace unsafe integer tokens with private strings, parse once natively,
// then replace those exact markers with JSI BigInt values. JS callers receive
// ordinary JSON objects and never need a MessagePack decoder.
// ---------------------------------------------------------------------------
using BigIntMarkers = std::unordered_map<std::string, std::string>;
static std::atomic<uint64_t> s_bigIntMarkerNonce{0};
static constexpr const char* BIGINT_MARKER_PREFIX = "\x1ejson-eval-rs-bigint:";

static bool isUnsafeIntegerToken(const std::string& token) {
    const size_t digitsStart = token[0] == '-' ? 1 : 0;
    if (digitsStart == token.size() ||
        token.find_first_of(".eE") != std::string::npos) {
        return false;
    }

    constexpr const char* MAX_SAFE = "9007199254740991";
    const size_t digits = token.size() - digitsStart;
    return digits > 16 ||
        (digits == 16 && token.compare(digitsStart, 16, MAX_SAFE) > 0);
}

static std::string quoteJsonString(const std::string& value) {
    std::string quoted;
    quoted.reserve(value.size() + 2);
    quoted.push_back('"');
    for (unsigned char c : value) {
        switch (c) {
            case '"': quoted += "\\\""; break;
            case '\\': quoted += "\\\\"; break;
            case '\b': quoted += "\\b"; break;
            case '\f': quoted += "\\f"; break;
            case '\n': quoted += "\\n"; break;
            case '\r': quoted += "\\r"; break;
            case '\t': quoted += "\\t"; break;
            default:
                if (c < 0x20) {
                    static constexpr char HEX[] = "0123456789abcdef";
                    quoted += "\\u00";
                    quoted.push_back(HEX[c >> 4]);
                    quoted.push_back(HEX[c & 0x0f]);
                } else {
                    quoted.push_back(static_cast<char>(c));
                }
        }
    }
    quoted.push_back('"');
    return quoted;
}

static std::string preserveLargeJsonIntegers(
    const std::string& json,
    BigIntMarkers& markers
) {
    const uint64_t nonce = s_bigIntMarkerNonce.fetch_add(1, std::memory_order_relaxed);
    std::string output;
    output.reserve(json.size());

    for (size_t index = 0; index < json.size();) {
        if (json[index] == '"') {
            const size_t start = index++;
            while (index < json.size()) {
                if (json[index] == '\\') {
                    index += 2;
                } else if (json[index++] == '"') {
                    break;
                }
            }
            output.append(json, start, index - start);
            continue;
        }

        const char current = json[index];
        if (current != '-' && !std::isdigit(static_cast<unsigned char>(current))) {
            output.push_back(current);
            ++index;
            continue;
        }

        const size_t start = index++;
        while (index < json.size()) {
            const char tokenChar = json[index];
            if (!std::isdigit(static_cast<unsigned char>(tokenChar)) &&
                tokenChar != '.' && tokenChar != 'e' && tokenChar != 'E' &&
                tokenChar != '+' && tokenChar != '-') {
                break;
            }
            ++index;
        }

        const std::string token = json.substr(start, index - start);
        if (!isUnsafeIntegerToken(token)) {
            output += token;
            continue;
        }

        const std::string marker = std::string(BIGINT_MARKER_PREFIX) +
            std::to_string(nonce) + ":" + std::to_string(markers.size());
        markers.emplace(marker, token);
        output += quoteJsonString(marker);
    }

    return output;
}

static jsi::Value replaceBigIntMarkers(
    jsi::Runtime& runtime,
    jsi::Value value,
    const BigIntMarkers& markers
) {
    if (value.isString()) {
        const std::string marker = value.asString(runtime).utf8(runtime);
        const auto markerIt = markers.find(marker);
        if (markerIt == markers.end()) return value;

        const std::string& token = markerIt->second;
        if (token[0] == '-') {
            return jsi::BigInt::fromInt64(runtime, std::stoll(token));
        }
        return jsi::BigInt::fromUint64(runtime, std::stoull(token));
    }

    if (!value.isObject()) return value;

    jsi::Object object = value.asObject(runtime);
    if (object.isArray(runtime)) {
        jsi::Array array = object.asArray(runtime);
        for (size_t index = 0; index < array.size(runtime); ++index) {
            array.setValueAtIndex(
                runtime,
                index,
                replaceBigIntMarkers(runtime, array.getValueAtIndex(runtime, index), markers)
            );
        }
        return value;
    }

    jsi::Array names = object.getPropertyNames(runtime);
    for (size_t index = 0; index < names.size(runtime); ++index) {
        const jsi::Value name = names.getValueAtIndex(runtime, index);
        if (!name.isString()) continue;
        const jsi::String key = name.asString(runtime);
        object.setProperty(
            runtime,
            key,
            replaceBigIntMarkers(runtime, object.getProperty(runtime, key), markers)
        );
    }
    return value;
}

static jsi::Value ffiResultToJsiValue(
    jsi::Runtime& runtime,
    FFIResult& result,
    const char* fallback = "null"
) {
    JsonEvalJSI::checkResult(runtime, result);
    std::string json = result.data_ptr && result.data_len > 0
        ? std::string(reinterpret_cast<const char*>(result.data_ptr), result.data_len)
        : fallback;
    json_eval_free_result(result);

    BigIntMarkers markers;
    const std::string safeJson = preserveLargeJsonIntegers(json, markers);
    jsi::Value value = jsi::Value::createFromJsonUtf8(
        runtime,
        reinterpret_cast<const uint8_t*>(safeJson.data()),
        safeJson.size()
    );
    if (markers.empty()) return value;
    return replaceBigIntMarkers(runtime, std::move(value), markers);
}

// ---------------------------------------------------------------------------
// Helper: converts FFI result data_ptr+data_len → jsi::ArrayBuffer (Zero-Copy)
// ---------------------------------------------------------------------------
static jsi::Value ffiResultToJsiBuffer(jsi::Runtime& runtime, FFIResult& result) {
    JsonEvalJSI::checkResult(runtime, result);
    auto rustBuffer = std::make_shared<RustBuffer>(result);
    return rustBuffer->toArrayBuffer(runtime);
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
                FFIResult schemaResult = json_eval_get_evaluated_schema(handle);
                return ffiResultToJsiValue(rt, schemaResult);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
            }
        );
    }

    // ---- getEvaluatedSchema ----
    if (prop == "getEvaluatedSchema") {
        return createJsiFn(runtime, "getEvaluatedSchema",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema(handle);
                return ffiResultToJsiValue(rt, result);
            }
        );
    }

    // ---- getEvaluatedSchemaMsgpack ----
    if (prop == "getEvaluatedSchemaMsgpack") {
        return createJsiFn(runtime, "getEvaluatedSchemaMsgpack",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_msgpack(handle);
                return ffiResultToJsiBuffer(rt, result);
            }
        );
    }

    // ---- getEvaluatedSchemaResolvedMsgpack ----
    if (prop == "getEvaluatedSchemaResolvedMsgpack") {
        return createJsiFn(runtime, "getEvaluatedSchemaResolvedMsgpack",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_resolved_msgpack(handle);
                return ffiResultToJsiBuffer(rt, result);
            }
        );
    }

    // ---- getEvaluatedSchemaResolved ----
    if (prop == "getEvaluatedSchemaResolved") {
        return createJsiFn(runtime, "getEvaluatedSchemaResolved",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_resolved(handle);
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_by_path(handle, path.c_str());
                return ffiResultToJsiValue(rt, result);
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
                uint8_t format = count > 2 ? static_cast<uint8_t>(args[2].asNumber()) : 0;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_by_paths(handle, pathsJson.c_str(), format);
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
            }
        );
    }

    // ---- getFieldOptions ----
    if (prop == "getFieldOptions") {
        return createJsiFn(runtime, "getFieldOptions",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto fieldPath = stringFromValue(rt, args[1]);

                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_field_options(handle, fieldPath.c_str());
                // Not found is returned as a failed result — propagate as null to JS
                if (!result.success) {
                    json_eval_free_result(result);
                    return jsi::Value::null();
                }
                return ffiResultToJsiValue(rt, result);
            }
        );
    }

    // ---- getEvaluatedSchemaWithoutParams ----
    if (prop == "getEvaluatedSchemaWithoutParams") {
        return createJsiFn(runtime, "getEvaluatedSchemaWithoutParams",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_without_params(handle);
                return ffiResultToJsiValue(rt, result);
            }
        );
    }

    // ---- getResolvedLayout ----
    if (prop == "getResolvedLayout") {
        return createJsiFn(runtime, "getResolvedLayout",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);

                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_resolved_layout(handle);
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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

    // ---- cancel ----
    if (prop == "cancel") {
        return createJsiFn(runtime, "cancel",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                auto handleId = stringFromValue(rt, args[0]);
                auto [handle, lock] = lockHandleById(handleId);
                json_eval_cancel(handle);
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
                return ffiResultToJsiValue(rt, result);
            }
        );
    }

    // ---- decodeArrayBuffer: convert ArrayBuffer → UTF-8 string (zero-extra-copy)
    // Replaces Hermes TextDecoder with direct JSI-level decode
    if (prop == "decodeArrayBuffer") {
        return createJsiFn(runtime, "decodeArrayBuffer",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 1);
                if (!args[0].isObject()) {
                    throw jsi::JSError(rt, "decodeArrayBuffer: expected ArrayBuffer");
                }
                auto obj = args[0].asObject(rt);
                if (!obj.isArrayBuffer(rt)) {
                    throw jsi::JSError(rt, "decodeArrayBuffer: expected ArrayBuffer");
                }
                auto buf = obj.getArrayBuffer(rt);
                auto* data = buf.data(rt);
                auto length = buf.length(rt);
                return jsi::String::createFromUtf8(rt, data, length);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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

    // ---- getResolvedLayoutSubform ----
    if (prop == "getResolvedLayoutSubform") {
        return createJsiFn(runtime, "getResolvedLayoutSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_resolved_layout_subform(handle, subformPath.c_str());
                return ffiResultToJsiValue(rt, result);
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
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_subform(handle, subformPath.c_str());
                return ffiResultToJsiValue(rt, result);
            }
        );
    }

    // ---- getEvaluatedSchemaResolvedSubform ----
    if (prop == "getEvaluatedSchemaResolvedSubform") {
        return createJsiFn(runtime, "getEvaluatedSchemaResolvedSubform",
            [](jsi::Runtime& rt, const jsi::Value* args, size_t count) -> jsi::Value {
                checkArgCount(rt, count, 2);
                auto handleId = stringFromValue(rt, args[0]);
                auto subformPath = stringFromValue(rt, args[1]);
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_resolved_subform(handle, subformPath.c_str());
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_without_params_subform(handle, subformPath.c_str());
                return ffiResultToJsiValue(rt, result);
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
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_by_path_subform(handle, subformPath.c_str(), schemaPath.c_str());
                return ffiResultToJsiValue(rt, result);
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
                uint8_t format = count > 3 ? static_cast<uint8_t>(args[3].asNumber()) : 0;
                
                auto [handle, lock] = lockHandleById(handleId);
                FFIResult result = json_eval_get_evaluated_schema_by_paths_subform(handle, subformPath.c_str(), schemaPathsJson.c_str(), format);
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
                return ffiResultToJsiValue(rt, result);
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
        "getEvaluatedSchema", "getEvaluatedSchemaMsgpack", "getEvaluatedSchemaResolvedMsgpack", "getEvaluatedSchemaResolved", "getSchemaValue", "getSchemaValueArray", "getSchemaValueObject",
        "getEvaluatedSchemaByPath", "getEvaluatedSchemaByPaths",
        "getSchemaByPath", "getSchemaByPaths",
        "getEvaluatedSchemaWithoutParams",
        "resolveLayout", "getResolvedLayout",
        "compileAndRunLogic", "compileLogic", "runLogic",
        "reloadSchema", "reloadSchemaMsgpack", "reloadSchemaFromCache",
        "setTimezoneOffset",
        "dispose", "cancel", "evaluateLogic", "version", "decodeArrayBuffer",
        // Subform
        "evaluateSubform", "validateSubform", "evaluateDependentsSubform",
        "resolveLayoutSubform",
        "getResolvedLayoutSubform",
        "getEvaluatedSchemaSubform", "getEvaluatedSchemaResolvedSubform",
        "getSchemaValueSubform", "getSchemaValueArraySubform", "getSchemaValueObjectSubform",
        "getEvaluatedSchemaWithoutParamsSubform",
        "getEvaluatedSchemaByPathSubform", "getEvaluatedSchemaByPathsSubform",
        "getSchemaByPathSubform", "getSchemaByPathsSubform",
        "getSubformPaths", "hasSubform",
        "getFieldOptions"
    };
    std::vector<jsi::PropNameID> result;
    for (auto& n : names) {
        result.push_back(jsi::PropNameID::forAscii(runtime, n));
    }
    return result;
}

} // namespace jsoneval
