#include <jni.h>
#include <string>
#include "json-eval-bridge.h"

using namespace jsoneval;

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

// Helper to resolve promise with string result
// Zero-copy optimization: Pass result directly without intermediate conversions
void resolvePromise(JNIEnv* env, jobject promise, const std::string& result) {
    jclass promiseClass = env->GetObjectClass(promise);
    jmethodID resolveMethod = env->GetMethodID(promiseClass, "resolve", "(Ljava/lang/Object;)V");
    jstring jresult = stringToJstring(env, result);
    env->CallVoidMethod(promise, resolveMethod, jresult);
    env->DeleteLocalRef(jresult);
    env->DeleteLocalRef(promiseClass);
}

void rejectPromise(JNIEnv* env, jobject promise, const std::string& code, const std::string& message) {
    jclass promiseClass = env->GetObjectClass(promise);
    jmethodID rejectMethod = env->GetMethodID(promiseClass, "reject", "(Ljava/lang/String;Ljava/lang/String;)V");
    jstring jcode = stringToJstring(env, code);
    jstring jmsg = stringToJstring(env, message);
    env->CallVoidMethod(promise, rejectMethod, jcode, jmsg);
    env->DeleteLocalRef(jcode);
    env->DeleteLocalRef(jmsg);
    env->DeleteLocalRef(promiseClass);
}

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

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeEvaluateAsync(
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
    
    // Keep global reference to promise for callback
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::evaluateAsync(handleStr, dataStr, contextStr,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "EVALUATE_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
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
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::validateAsync(handleStr, dataStr, contextStr,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "VALIDATE_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeEvaluateDependentsAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jstring changedPath,
    jstring data,
    jstring context,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    std::string pathStr = jstringToString(env, changedPath);
    std::string dataStr = jstringToString(env, data);
    std::string contextStr = jstringToString(env, context);
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::evaluateDependentsAsync(handleStr, pathStr, dataStr, contextStr,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "EVALUATE_DEPENDENTS_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
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
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::getEvaluatedSchemaAsync(handleStr, skipLayoutBool,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "GET_SCHEMA_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeGetSchemaValueAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::getSchemaValueAsync(handleStr,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "GET_VALUE_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
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
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::getEvaluatedSchemaWithoutParamsAsync(handleStr, skipLayout,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "GET_SCHEMA_WITHOUT_PARAMS_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
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
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::getEvaluatedSchemaByPathAsync(handleStr, pathStr, skipLayout,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "GET_EVALUATED_SCHEMA_BY_PATH_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
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
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::reloadSchemaAsync(handleStr, schemaStr, contextStr, dataStr,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "RELOAD_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeCacheStatsAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::cacheStatsAsync(handleStr,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "CACHE_STATS_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
}

JNIEXPORT void JNICALL
Java_com_jsonevalrs_JsonEvalRsModule_nativeClearCacheAsync(
    JNIEnv* env,
    jobject /* this */,
    jstring handle,
    jobject promise
) {
    std::string handleStr = jstringToString(env, handle);
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::clearCacheAsync(handleStr,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "CLEAR_CACHE_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
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
                // Parse result as integer
                jclass integerClass = env->FindClass("java/lang/Integer");
                jmethodID parseIntMethod = env->GetStaticMethodID(integerClass, "parseInt", "(Ljava/lang/String;)I");
                jstring jresultStr = stringToJstring(env, result);
                jint intValue = env->CallStaticIntMethod(integerClass, parseIntMethod, jresultStr);
                
                jclass promiseClass = env->GetObjectClass(globalPromise);
                jmethodID resolveMethod = env->GetMethodID(promiseClass, "resolve", "(Ljava/lang/Object;)V");
                
                jmethodID valueOfMethod = env->GetStaticMethodID(integerClass, "valueOf", "(I)Ljava/lang/Integer;");
                jobject integerObj = env->CallStaticObjectMethod(integerClass, valueOfMethod, intValue);
                
                env->CallVoidMethod(globalPromise, resolveMethod, integerObj);
                
                env->DeleteLocalRef(jresultStr);
                env->DeleteLocalRef(integerObj);
                env->DeleteLocalRef(integerClass);
                env->DeleteLocalRef(promiseClass);
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
    
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    JsonEvalBridge::validatePathsAsync(handleStr, dataStr, contextStr, pathsStr,
        [jvm, globalPromise](const std::string& result, const std::string& error) {
            JNIEnv* env = nullptr;
            jvm->AttachCurrentThread(&env, nullptr);
            
            if (error.empty()) {
                resolvePromise(env, globalPromise, result);
            } else {
                rejectPromise(env, globalPromise, "VALIDATE_PATHS_ERROR", error);
            }
            
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
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

} // extern "C"
