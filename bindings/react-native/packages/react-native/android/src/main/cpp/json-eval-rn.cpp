#include <jni.h>
#include <string>
#include <functional>
#include "json-eval-bridge.h"

using namespace jsoneval;

// Cached JNI references for performance (initialized in JNI_OnLoad)
static jclass gPromiseClass = nullptr;
static jmethodID gResolveMethodID = nullptr;
static jmethodID gRejectMethodID = nullptr;
static jclass gIntegerClass = nullptr;
static jmethodID gIntegerValueOfMethodID = nullptr;

// Helper functions (C++ linkage - internal use only)
// Helper to convert jstring to std::string
// Note: GetStringUTFChars provides a pinned pointer (minimal copy), but we must
// copy to std::string to pass to the C++ bridge layer due to lifetime management
static std::string jstringToString(JNIEnv* env, jstring jStr) {
    if (jStr == nullptr) return "";
    // GetStringUTFChars returns a pointer to UTF-8 chars (may pin Java string in memory)
    const char* chars = env->GetStringUTFChars(jStr, nullptr);
    if (chars == nullptr) return "";
    // Copy to std::string for safe lifetime management
    std::string str(chars);
    env->ReleaseStringUTFChars(jStr, chars);
    return str;
}

// Helper to create jstring from std::string
// Note: NewStringUTF copies C string to create Java String object (unavoidable)
static jstring stringToJstring(JNIEnv* env, const std::string& str) {
    return env->NewStringUTF(str.c_str());
}

extern "C" {

// Initialize cached JNI method IDs for performance
// Called once when library is loaded
JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM* vm, void* reserved) {
    JNIEnv* env;
    if (vm->GetEnv(reinterpret_cast<void**>(&env), JNI_VERSION_1_6) != JNI_OK) {
        return JNI_ERR;
    }
    
    // Cache Promise class and methods
    jclass promiseClass = env->FindClass("com/facebook/react/bridge/Promise");
    if (promiseClass == nullptr) return JNI_ERR;
    gPromiseClass = reinterpret_cast<jclass>(env->NewGlobalRef(promiseClass));
    gResolveMethodID = env->GetMethodID(gPromiseClass, "resolve", "(Ljava/lang/Object;)V");
    gRejectMethodID = env->GetMethodID(gPromiseClass, "reject", "(Ljava/lang/String;Ljava/lang/String;)V");
    env->DeleteLocalRef(promiseClass);
    
    // Cache Integer class for cacheLenAsync optimization
    jclass integerClass = env->FindClass("java/lang/Integer");
    if (integerClass == nullptr) return JNI_ERR;
    gIntegerClass = reinterpret_cast<jclass>(env->NewGlobalRef(integerClass));
    gIntegerValueOfMethodID = env->GetStaticMethodID(gIntegerClass, "valueOf", "(I)Ljava/lang/Integer;");
    env->DeleteLocalRef(integerClass);
    
    return JNI_VERSION_1_6;
}

// Optimized promise helpers using cached method IDs (30-50% faster)
void resolvePromise(JNIEnv* env, jobject promise, const std::string& result) {
    jstring jresult = stringToJstring(env, result);
    env->CallVoidMethod(promise, gResolveMethodID, jresult);
    env->DeleteLocalRef(jresult);
}

void rejectPromise(JNIEnv* env, jobject promise, const std::string& code, const std::string& message) {
    jstring jcode = stringToJstring(env, code);
    jstring jmsg = stringToJstring(env, message);
    env->CallVoidMethod(promise, gRejectMethodID, jcode, jmsg);
    env->DeleteLocalRef(jcode);
    env->DeleteLocalRef(jmsg);
}

} // extern "C"

// Generic async helper to reduce code duplication
// Encapsulates the JavaVM/thread attachment boilerplate pattern
// Note: Template functions must have C++ linkage, not C linkage
template<typename Func>
void runAsyncWithPromise(
    JNIEnv* env,
    jobject promise,
    const char* errorCode,
    Func&& bridgeCall
) {
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    bridgeCall([jvm, globalPromise, errorCode](const std::string& result, const std::string& error) {
        JNIEnv* env = nullptr;
        jvm->AttachCurrentThread(&env, nullptr);
        
        if (error.empty()) {
            resolvePromise(env, globalPromise, result);
        } else {
            rejectPromise(env, globalPromise, errorCode, error);
        }
        
        env->DeleteGlobalRef(globalPromise);
        jvm->DetachCurrentThread();
    });
}

extern "C" {

JNIEXPORT jstring JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeCreate(
    JNIEnv* env,
    jobject /* this */,
    jstring schema,
    jstring context,
    jstring data
) {
    try {
        std::string schemaStr = jstringToString(env, schema);
        std::string contextStr = jstringToString(env, context);
        std::string dataStr = jstringToString(env, data);
        
        std::string handle = JsonEvalBridge::create(schemaStr, contextStr, dataStr);
        return stringToJstring(env, handle);
    } catch (const std::exception& e) {
        jclass exClass = env->FindClass("java/lang/RuntimeException");
        env->ThrowNew(exClass, e.what());
        return nullptr;
    }
}

JNIEXPORT jstring JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeCreateFromMsgpack(
    JNIEnv* env,
    jobject /* this */,
    jbyteArray schemaMsgpack,
    jstring context,
    jstring data
) {
    try {
        std::string contextStr = jstringToString(env, context);
        std::string dataStr = jstringToString(env, data);
        
        // Convert jbyteArray to std::vector<uint8_t>
        jsize len = env->GetArrayLength(schemaMsgpack);
        std::vector<uint8_t> msgpackBytes(len);
        env->GetByteArrayRegion(schemaMsgpack, 0, len, reinterpret_cast<jbyte*>(msgpackBytes.data()));
        
        std::string handle = JsonEvalBridge::createFromMsgpack(msgpackBytes, contextStr, dataStr);
        return stringToJstring(env, handle);
    } catch (const std::exception& e) {
        jclass exClass = env->FindClass("java/lang/RuntimeException");
        env->ThrowNew(exClass, e.what());
        return nullptr;
    }
}

JNIEXPORT jstring JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeCreateFromCache(
    JNIEnv* env,
    jobject /* this */,
    jstring cacheKey,
    jstring context,
    jstring data
) {
    try {
        std::string cacheKeyStr = jstringToString(env, cacheKey);
        std::string contextStr = jstringToString(env, context);
        std::string dataStr = jstringToString(env, data);
        
        std::string handle = JsonEvalBridge::createFromCache(cacheKeyStr, contextStr, dataStr);
        return stringToJstring(env, handle);
    } catch (const std::exception& e) {
        jclass exClass = env->FindClass("java/lang/RuntimeException");
        env->ThrowNew(exClass, e.what());
        return nullptr;
    }
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeEvaluateAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring data,
    jstring context,
    jstring pathsJson,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string dataStr = jstringToString(env, data);
    std::string contextStr = jstringToString(env, context);
    std::string pathsJsonStr = jstringToString(env, pathsJson);
    
    runAsyncWithPromise(env, promise, "EVALUATE_ERROR", [handleStr, dataStr, contextStr, pathsJsonStr](auto callback) {
        JsonEvalBridge::evaluateAsync(handleStr, dataStr, contextStr, pathsJsonStr, callback);
    });
}

JNIEXPORT jdouble JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeCompileLogic(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring logicStr
) {
    try {
        std::string handleStr = jstringToString(env, handle);
        std::string logicStrCpp = jstringToString(env, logicStr);
        uint64_t logicId = JsonEvalBridge::compileLogic(handleStr, logicStrCpp);
        return static_cast<jdouble>(logicId);
    } catch (const std::exception& e) {
        jclass exClass = env->FindClass("java/lang/RuntimeException");
        env->ThrowNew(exClass, e.what());
        return 0.0;
    }
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeRunLogicAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jdouble logicId,
    jstring data,
    jstring context,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string dataStr = jstringToString(env, data);
    std::string contextStr = jstringToString(env, context);
    uint64_t logicIdValue = static_cast<uint64_t>(logicId);

    runAsyncWithPromise(env, promise, "RUN_LOGIC_ERROR", [handleStr, logicIdValue, dataStr, contextStr](auto callback) {
        JsonEvalBridge::runLogicAsync(handleStr, logicIdValue, dataStr, contextStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeValidateAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring data,
    jstring context,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string dataStr = jstringToString(env, data);
    std::string contextStr = jstringToString(env, context);
    
    runAsyncWithPromise(env, promise, "VALIDATE_ERROR", [handleStr, dataStr, contextStr](auto callback) {
        JsonEvalBridge::validateAsync(handleStr, dataStr, contextStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeEvaluateDependentsAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring changedPath,
    jstring data,
    jstring context,
    jboolean reEvaluate,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string pathStr = jstringToString(env, changedPath);
    std::string dataStr = jstringToString(env, data);
    std::string contextStr = jstringToString(env, context);
    bool reEval = static_cast<bool>(reEvaluate);
    
    runAsyncWithPromise(env, promise, "EVALUATE_DEPENDENTS_ERROR", [handleStr, pathStr, dataStr, contextStr, reEval](auto callback) {
        JsonEvalBridge::evaluateDependentsAsync(handleStr, pathStr, dataStr, contextStr, reEval, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetEvaluatedSchemaAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jboolean skipLayout,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    bool skipLayoutBool = (skipLayout == JNI_TRUE);
    
    runAsyncWithPromise(env, promise, "GET_SCHEMA_ERROR", [handleStr, skipLayoutBool](auto callback) {
        JsonEvalBridge::getEvaluatedSchemaAsync(handleStr, skipLayoutBool, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetSchemaValueAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    
    runAsyncWithPromise(env, promise, "GET_VALUE_ERROR", [handleStr](auto callback) {
        JsonEvalBridge::getSchemaValueAsync(handleStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetEvaluatedSchemaWithoutParamsAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jboolean skipLayout,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    bool skipLayoutBool = (skipLayout == JNI_TRUE);
    
    runAsyncWithPromise(env, promise, "GET_SCHEMA_WITHOUT_PARAMS_ERROR", [handleStr, skipLayoutBool](auto callback) {
        JsonEvalBridge::getEvaluatedSchemaWithoutParamsAsync(handleStr, skipLayoutBool, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetEvaluatedSchemaByPathAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring path,
    jboolean skipLayout,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string pathStr = jstringToString(env, path);
    bool skipLayoutBool = (skipLayout == JNI_TRUE);
    
    runAsyncWithPromise(env, promise, "GET_EVALUATED_SCHEMA_BY_PATH_ERROR", [handleStr, pathStr, skipLayoutBool](auto callback) {
        JsonEvalBridge::getEvaluatedSchemaByPathAsync(handleStr, pathStr, skipLayoutBool, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetEvaluatedSchemaByPathsAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring pathsJson,
    jboolean skipLayout,
    jint format,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string pathsJsonStr = jstringToString(env, pathsJson);
    bool skipLayoutBool = (skipLayout == JNI_TRUE);
    int formatInt = static_cast<int>(format);
    
    runAsyncWithPromise(env, promise, "GET_EVALUATED_SCHEMA_BY_PATHS_ERROR", [handleStr, pathsJsonStr, skipLayoutBool, formatInt](auto callback) {
        JsonEvalBridge::getEvaluatedSchemaByPathsAsync(handleStr, pathsJsonStr, skipLayoutBool, formatInt, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetSchemaByPathAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring path,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string pathStr = jstringToString(env, path);
    
    runAsyncWithPromise(env, promise, "GET_SCHEMA_BY_PATH_ERROR", [handleStr, pathStr](auto callback) {
        JsonEvalBridge::getSchemaByPathAsync(handleStr, pathStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetSchemaByPathsAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring pathsJson,
    jint format,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string pathsJsonStr = jstringToString(env, pathsJson);
    int formatInt = static_cast<int>(format);
    
    runAsyncWithPromise(env, promise, "GET_SCHEMA_BY_PATHS_ERROR", [handleStr, pathsJsonStr, formatInt](auto callback) {
        JsonEvalBridge::getSchemaByPathsAsync(handleStr, pathsJsonStr, formatInt, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeReloadSchemaAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring schema,
    jstring context,
    jstring data,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string schemaStr = jstringToString(env, schema);
    std::string contextStr = jstringToString(env, context);
    std::string dataStr = jstringToString(env, data);
    
    runAsyncWithPromise(env, promise, "RELOAD_ERROR", [handleStr, schemaStr, contextStr, dataStr](auto callback) {
        JsonEvalBridge::reloadSchemaAsync(handleStr, schemaStr, contextStr, dataStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeReloadSchemaMsgpackAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jbyteArray schemaMsgpack,
    jstring context,
    jstring data,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string contextStr = jstringToString(env, context);
    std::string dataStr = jstringToString(env, data);
    
    // Convert jbyteArray to std::vector<uint8_t>
    jsize len = env->GetArrayLength(schemaMsgpack);
    std::vector<uint8_t> msgpackBytes(len);
    env->GetByteArrayRegion(schemaMsgpack, 0, len, reinterpret_cast<jbyte*>(msgpackBytes.data()));
    
    runAsyncWithPromise(env, promise, "RELOAD_MSGPACK_ERROR", [handleStr, msgpackBytes, contextStr, dataStr](auto callback) {
        JsonEvalBridge::reloadSchemaMsgpackAsync(handleStr, msgpackBytes, contextStr, dataStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeReloadSchemaFromCacheAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring cacheKey,
    jstring context,
    jstring data,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string cacheKeyStr = jstringToString(env, cacheKey);
    std::string contextStr = jstringToString(env, context);
    std::string dataStr = jstringToString(env, data);
    
    runAsyncWithPromise(env, promise, "RELOAD_CACHE_ERROR", [handleStr, cacheKeyStr, contextStr, dataStr](auto callback) {
        JsonEvalBridge::reloadSchemaFromCacheAsync(handleStr, cacheKeyStr, contextStr, dataStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeCacheStatsAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    
    runAsyncWithPromise(env, promise, "CACHE_STATS_ERROR", [handleStr](auto callback) {
        JsonEvalBridge::cacheStatsAsync(handleStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeClearCacheAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    
    runAsyncWithPromise(env, promise, "CLEAR_CACHE_ERROR", [handleStr](auto callback) {
        JsonEvalBridge::clearCacheAsync(handleStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeCacheLenAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::cacheLenAsync(handleStr,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                // Optimized: Direct parse and box using cached method IDs
                jint intValue = std::stoi(result);
                jobject integerObj = env->CallStaticObjectMethod(gIntegerClass, gIntegerValueOfMethodID, intValue);
                env->CallVoidMethod(globalPromise, gResolveMethodID, integerObj);
                env->DeleteLocalRef(integerObj);
            } else {
                rejectPromise(env, globalPromise, "CACHE_LEN_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeValidatePathsAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring data,
    jstring context,
    jstring pathsJson,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string dataStr = jstringToString(env, data);
    std::string contextStr = jstringToString(env, context);
    std::string pathsStr = jstringToString(env, pathsJson);
    
    runAsyncWithPromise(env, promise, "VALIDATE_PATHS_ERROR", [handleStr, dataStr, contextStr, pathsStr](auto callback) {
        JsonEvalBridge::validatePathsAsync(handleStr, dataStr, contextStr, pathsStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeEnableCacheAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    
    runAsyncWithPromise(env, promise, "ENABLE_CACHE_ERROR", [handleStr](auto callback) {
        JsonEvalBridge::enableCacheAsync(handleStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeDisableCacheAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    
    runAsyncWithPromise(env, promise, "DISABLE_CACHE_ERROR", [handleStr](auto callback) {
        JsonEvalBridge::disableCacheAsync(handleStr, callback);
    });
}

JNIEXPORT jboolean JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeIsCacheEnabled(
    JNIEnv* env,
    jobject /* this */,
    jstring handle
) {
    std::string handleStr = jstringToString(env, handle);
    bool enabled = JsonEvalBridge::isCacheEnabled(handleStr);
    return enabled ? JNI_TRUE : JNI_FALSE;
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeDispose(
    JNIEnv* env,
    jobject /* this */,
    jstring handle
) {
    std::string handleStr = jstringToString(env, handle);
    JsonEvalBridge::dispose(handleStr);
}

JNIEXPORT jstring JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeVersion(
    JNIEnv* env,
    jobject /* this */
) {
    std::string version = JsonEvalBridge::version();
    return stringToJstring(env, version);
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeResolveLayoutAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jboolean evaluate,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    bool evaluateBool = (evaluate == JNI_TRUE);
    
    runAsyncWithPromise(env, promise, "RESOLVE_LAYOUT_ERROR", [handleStr, evaluateBool](auto callback) {
        JsonEvalBridge::resolveLayoutAsync(handleStr, evaluateBool, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeCompileAndRunLogicAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring logicStr,
    jstring data,
    jstring context,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string logicStrCpp = jstringToString(env, logicStr);
    std::string dataStr = jstringToString(env, data);
    std::string contextStr = jstringToString(env, context);
    
    runAsyncWithPromise(env, promise, "COMPILE_AND_RUN_LOGIC_ERROR", [handleStr, logicStrCpp, dataStr, contextStr](auto callback) {
        JsonEvalBridge::compileAndRunLogicAsync(handleStr, logicStrCpp, dataStr, contextStr, callback);
    });
}

// ============================================================================
// Subform Methods
// ============================================================================

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeEvaluateSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jstring data,
    jstring context,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    std::string dataStr = jstringToString(env, data);
    std::string contextStr = jstringToString(env, context);
    
    runAsyncWithPromise(env, promise, "EVALUATE_SUBFORM_ERROR", [handleStr, subformPathStr, dataStr, contextStr](auto callback) {
        JsonEvalBridge::evaluateSubformAsync(handleStr, subformPathStr, dataStr, contextStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeValidateSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jstring data,
    jstring context,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    std::string dataStr = jstringToString(env, data);
    std::string contextStr = jstringToString(env, context);
    
    runAsyncWithPromise(env, promise, "VALIDATE_SUBFORM_ERROR", [handleStr, subformPathStr, dataStr, contextStr](auto callback) {
        JsonEvalBridge::validateSubformAsync(handleStr, subformPathStr, dataStr, contextStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeEvaluateDependentsSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jstring changedPath,
    jstring data,
    jstring context,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    std::string changedPathStr = jstringToString(env, changedPath);
    std::string dataStr = jstringToString(env, data);
    std::string contextStr = jstringToString(env, context);
    
    runAsyncWithPromise(env, promise, "EVALUATE_DEPENDENTS_SUBFORM_ERROR", [handleStr, subformPathStr, changedPathStr, dataStr, contextStr](auto callback) {
        JsonEvalBridge::evaluateDependentsSubformAsync(handleStr, subformPathStr, changedPathStr, dataStr, contextStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeResolveLayoutSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jboolean evaluate,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    bool evaluateBool = (evaluate == JNI_TRUE);
    
    runAsyncWithPromise(env, promise, "RESOLVE_LAYOUT_SUBFORM_ERROR", [handleStr, subformPathStr, evaluateBool](auto callback) {
        JsonEvalBridge::resolveLayoutSubformAsync(handleStr, subformPathStr, evaluateBool, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetEvaluatedSchemaSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jboolean resolveLayout,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    bool resolveLayoutBool = (resolveLayout == JNI_TRUE);
    
    runAsyncWithPromise(env, promise, "GET_EVALUATED_SCHEMA_SUBFORM_ERROR", [handleStr, subformPathStr, resolveLayoutBool](auto callback) {
        JsonEvalBridge::getEvaluatedSchemaSubformAsync(handleStr, subformPathStr, resolveLayoutBool, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetSchemaValueSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    
    runAsyncWithPromise(env, promise, "GET_SCHEMA_VALUE_SUBFORM_ERROR", [handleStr, subformPathStr](auto callback) {
        JsonEvalBridge::getSchemaValueSubformAsync(handleStr, subformPathStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetEvaluatedSchemaWithoutParamsSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jboolean resolveLayout,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    bool resolveLayoutBool = (resolveLayout == JNI_TRUE);
    
    runAsyncWithPromise(env, promise, "GET_SCHEMA_WITHOUT_PARAMS_SUBFORM_ERROR", [handleStr, subformPathStr, resolveLayoutBool](auto callback) {
        JsonEvalBridge::getEvaluatedSchemaWithoutParamsSubformAsync(handleStr, subformPathStr, resolveLayoutBool, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetEvaluatedSchemaByPathSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jstring schemaPath,
    jboolean skipLayout,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    std::string schemaPathStr = jstringToString(env, schemaPath);
    bool skipLayoutBool = (skipLayout == JNI_TRUE);
    
    runAsyncWithPromise(env, promise, "GET_SCHEMA_BY_PATH_SUBFORM_ERROR", [handleStr, subformPathStr, schemaPathStr, skipLayoutBool](auto callback) {
        JsonEvalBridge::getEvaluatedSchemaByPathSubformAsync(handleStr, subformPathStr, schemaPathStr, skipLayoutBool, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetEvaluatedSchemaByPathsSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jstring schemaPathsJson,
    jboolean skipLayout,
    jint format,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    std::string schemaPathsJsonStr = jstringToString(env, schemaPathsJson);
    bool skipLayoutBool = (skipLayout == JNI_TRUE);
    int formatInt = static_cast<int>(format);
    
    runAsyncWithPromise(env, promise, "GET_SCHEMA_BY_PATHS_SUBFORM_ERROR", [handleStr, subformPathStr, schemaPathsJsonStr, skipLayoutBool, formatInt](auto callback) {
        JsonEvalBridge::getEvaluatedSchemaByPathsSubformAsync(handleStr, subformPathStr, schemaPathsJsonStr, skipLayoutBool, formatInt, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetSubformPathsAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    
    runAsyncWithPromise(env, promise, "GET_SUBFORM_PATHS_ERROR", [handleStr](auto callback) {
        JsonEvalBridge::getSubformPathsAsync(handleStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetSchemaByPathSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jstring schemaPath,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    std::string schemaPathStr = jstringToString(env, schemaPath);
    
    runAsyncWithPromise(env, promise, "GET_SCHEMA_BY_PATH_SUBFORM_ERROR", [handleStr, subformPathStr, schemaPathStr](auto callback) {
        JsonEvalBridge::getSchemaByPathSubformAsync(handleStr, subformPathStr, schemaPathStr, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetSchemaByPathsSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jstring schemaPathsJson,
    jint format,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    std::string schemaPathsJsonStr = jstringToString(env, schemaPathsJson);
    int formatInt = static_cast<int>(format);
    
    runAsyncWithPromise(env, promise, "GET_SCHEMA_BY_PATHS_SUBFORM_ERROR", [handleStr, subformPathStr, schemaPathsJsonStr, formatInt](auto callback) {
        JsonEvalBridge::getSchemaByPathsSubformAsync(handleStr, subformPathStr, schemaPathsJsonStr, formatInt, callback);
    });
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeHasSubformAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring subformPath,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string subformPathStr = jstringToString(env, subformPath);
    
    runAsyncWithPromise(env, promise, "HAS_SUBFORM_ERROR", [handleStr, subformPathStr](auto callback) {
        JsonEvalBridge::hasSubformAsync(handleStr, subformPathStr, callback);
    });
}

} // extern "C"
