# React Native Native Module Architecture

## Overview

This React Native binding implements a proper native module with full C++ bridging for both Android and iOS platforms. All operations are executed asynchronously on separate threads to ensure the JavaScript thread remains responsive.

## Architecture Layers

```
┌─────────────────────────────────────┐
│   JavaScript/TypeScript Layer       │
│   (src/index.tsx)                   │
└───────────────┬─────────────────────┘
                │
                │ React Native Bridge
                │
┌───────────────▼─────────────────────┐
│   Platform Native Layer             │
│                                     │
│  ┌──────────────┬─────────────────┐ │
│  │   Android    │      iOS        │ │
│  │ (Kotlin+JNI) │ (Objective-C++) │ │
│  └──────┬───────┴────────┬────────┘ │
└─────────┼────────────────┼──────────┘
          │                │
          │ JNI            │ Objective-C++
          │                │
┌─────────▼────────────────▼──────────┐
│   C++ Bridge Layer                  │
│   (cpp/json-eval-bridge.{h,cpp})    │
│                                     │
│   - Thread-safe handle management   │
│   - Async execution (std::thread)   │
│   - Callback marshalling            │
└───────────────┬─────────────────────┘
                │
                │ C FFI
                │
┌───────────────▼─────────────────────┐
│   Rust Core Library                 │
│   (src/ffi.rs)                      │
│                                     │
│   - JSON evaluation                 │
│   - Schema validation               │
│   - Dependency tracking             │
│   - Cache management                │
└─────────────────────────────────────┘
```

## Component Details

### 1. JavaScript/TypeScript Layer (`src/index.tsx`)

**Purpose**: Provides the React Native API that JavaScript code interacts with.

**Key Components**:
- `JSONEval` class - Main API wrapper
- `useJSONEval` hook - React hook for component integration
- Type definitions for all interfaces

**Threading**: All calls are async and return Promises.

### 2. Android Native Layer

#### Kotlin Module (`android/src/main/java/com/jsonevalrs/JsonEvalRsModule.kt`)

**Purpose**: React Native module that exposes native methods to JavaScript.

**Key Features**:
- Loads native libraries: `libjson_eval_rs.so` and `libjson_eval_rn.so`
- Converts JavaScript arrays to JSON strings
- Uses Promise-based API for async operations
- Declares external native methods for JNI

#### JNI C++ Bridge (`android/src/main/cpp/json-eval-rn.cpp`)

**Purpose**: Bridges Kotlin and C++, handles thread attachment/detachment.

**Key Features**:
- Converts between `jstring` and `std::string`
- Manages JavaVM for thread attachment
- Creates global references for async callbacks
- Properly attaches/detaches threads for callback execution

**Threading Model**:
```cpp
// Main thread (from Kotlin)
Java_com_jsonevalrs_JsonEvalRsModule_nativeEvaluateAsync() {
    // Get JavaVM and create global promise reference
    JavaVM* jvm;
    env->GetJavaVM(&jvm);
    jobject globalPromise = env->NewGlobalRef(promise);
    
    // Spawn background thread via C++ bridge
    JsonEvalBridge::evaluateAsync(...,
        [jvm, globalPromise](...) {
            // Background thread
            // Attach to JVM
            JNIEnv* env;
            jvm->AttachCurrentThread(&env, nullptr);
            
            // Resolve/reject promise
            resolvePromise(env, globalPromise, result);
            
            // Cleanup and detach
            env->DeleteGlobalRef(globalPromise);
            jvm->DetachCurrentThread();
        }
    );
}
```

### 3. iOS Native Layer

#### Objective-C++ Module (`ios/JsonEvalRs.mm`)

**Purpose**: React Native module for iOS using Objective-C++.

**Key Features**:
- Direct C++ integration (`.mm` extension)
- Automatic thread management via lambdas
- Type conversion between NSString and std::string
- Promise-based async callbacks

**Threading Model**:
```objc
RCT_EXPORT_METHOD(evaluate:...) {
    // Main thread (from React Native)
    std::string handleStr = [self stdStringFromNSString:handle];
    
    // Spawn background thread via C++ bridge
    JsonEvalBridge::evaluateAsync(handleStr, ...,
        [resolve, reject](const std::string& result, ...) {
            // Background thread - callbacks handled by RN
            if (error.empty()) {
                resolve([NSString stringWithUTF8String:result.c_str()]);
            } else {
                reject(@"ERROR", [...], nil);
            }
        }
    );
}
```

### 4. C++ Bridge Layer (`cpp/json-eval-bridge.{h,cpp}`)

**Purpose**: Platform-agnostic async execution layer with thread-safe handle management.

**Key Features**:

#### Handle Management
```cpp
static std::map<std::string, JSONEvalHandle*> handles;
static std::mutex handlesMutex;
static int handleCounter = 0;

std::string create(...) {
    JSONEvalHandle* handle = json_eval_new(...);
    std::lock_guard<std::mutex> lock(handlesMutex);
    std::string handleId = "handle_" + std::to_string(handleCounter++);
    handles[handleId] = handle;
    return handleId;
}
```

#### Async Execution
```cpp
template<typename Func>
void runAsync(Func&& func, std::function<void(...)> callback) {
    std::thread([func = std::forward<Func>(func), callback]() {
        try {
            std::string result = func();
            callback(result, "");
        } catch (const std::exception& e) {
            callback("", e.what());
        }
    }).detach();  // Thread runs independently
}
```

#### Thread Safety
- All handle access protected by `std::mutex`
- Each operation creates a detached thread
- No thread pooling (simple but effective)
- Callbacks execute on background thread

### 5. Rust FFI Layer (`src/ffi.rs`)

**Purpose**: C-compatible interface to Rust core library.

**Key Features**:
- C-compatible types (`*const c_char`, `FFIResult`)
- Manual memory management with explicit free functions
- Thread-safe (Rust guarantees)
- Zero-cost abstractions

**Memory Management**:
```rust
#[no_mangle]
pub unsafe extern "C" fn json_eval_evaluate(...) -> FFIResult {
    // Allocate result
    let result_str = serde_json::to_string_pretty(&result).unwrap();
    FFIResult {
        success: true,
        data: CString::new(result_str).unwrap().into_raw(),
        error: ptr::null_mut(),
    }
}

// Caller must free
#[no_mangle]
pub unsafe extern "C" fn json_eval_free_result(result: FFIResult) {
    if !result.data.is_null() {
        drop(CString::from_raw(result.data));
    }
    // ...
}
```

## Build System

### Android Build (CMake + Gradle)

**CMakeLists.txt**:
```cmake
# Find Rust library
find_library(RUST_LIB json_eval_rs ...)

# Build C++ bridge
add_library(json-eval-rn SHARED
  src/main/cpp/json-eval-rn.cpp
  ../cpp/json-eval-bridge.cpp
)

# Link everything
target_link_libraries(json-eval-rn
  ReactAndroid::jsi
  ReactAndroid::reactnativejni
  ${RUST_LIB}
)
```

**build.gradle**:
- Configures CMake
- Sets up JNI library paths
- Handles multiple Android architectures

### iOS Build (CocoaPods)

**Podspec**:
- Includes C++ and Objective-C++ sources
- Links against Rust static library (`.a`)
- Configures C++17 standard
- Uses `-force_load` to include all Rust symbols

## Thread Safety Guarantees

### Rust Level
- All operations are inherently thread-safe
- Mutex-protected internal state
- No global mutable state

### C++ Bridge Level
- `std::mutex` protects handle map
- Each async operation runs in isolated thread
- No shared state between operations

### Platform Level
- **Android**: Proper JVM attachment/detachment
- **iOS**: React Native handles callback thread safety
- Promises ensure single callback execution

## Performance Characteristics

### Async Overhead
- Thread creation: ~1-2ms per operation
- Callback marshalling: ~0.5ms
- Total overhead: ~2-3ms

### Benefits
- Non-blocking JavaScript thread
- True parallelism for CPU-bound operations
- No impact on UI responsiveness

### Memory Usage
- Each handle: ~few KB
- Per-operation thread stack: ~1MB (temporary)
- Cleanup: Automatic via `detach()` and RAII

## Error Handling

### Error Flow
```
Rust Error → C++ Exception → Platform Callback → Promise Rejection → JavaScript Error
```

### Error Types
1. **Creation Errors**: Invalid schema, parse errors
2. **Evaluation Errors**: Logic execution failures
3. **Validation Errors**: Schema rule violations
4. **System Errors**: Out of memory, thread failures

### Error Propagation
- Rust: `Result<T, String>`
- C++: `try/catch` with `std::exception`
- Platform: Promise rejection with error code and message
- JavaScript: `Promise.catch()` or `try/await/catch`

## Testing Strategy

### Unit Tests
- Rust core (via `cargo test`)
- C++ bridge (via Google Test)
- TypeScript types (via Jest)

### Integration Tests
- End-to-end via React Native test app
- Platform-specific tests

## Future Enhancements

### Short Term
- Thread pool for better resource management
- Configurable thread priority
- Operation cancellation support

### Long Term
- JSI (JavaScript Interface) direct integration
- TurboModules support
- Synchronous API for simple operations
- SharedArrayBuffer for zero-copy transfers

## References

- [React Native Native Modules](https://reactnative.dev/docs/native-modules-intro)
- [Android JNI Guide](https://developer.android.com/training/articles/perf-jni)
- [iOS Objective-C++ Guide](https://developer.apple.com/documentation/objectivec)
- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)
