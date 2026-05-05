#pragma once

#include <jsi/jsi.h>
#include <string>
#include <unordered_map>
#include <memory>
#include <mutex>

#include "json-eval-bridge.h"

// C-compatible FFI types — full definition needed so callers can access fields
#ifdef __cplusplus
extern "C" {
#endif

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

#ifdef __cplusplus
}
#endif

namespace jsoneval {

namespace jsi = facebook::jsi;

/**
 * JSI HostObject for JSONEval.
 * Installed on the global JS object, provides synchronous access to
 * JSONEval functions — bypassing the RN bridge entirely.
 *
 * Usage from JS:
 *   global.jsonEval.create(schema, context, data) -> handle
 *   global.jsonEval.evaluateOnly(handle, data, context, paths) -> void
 *   global.jsonEval.evaluate(handle, data, context, paths) -> JSON string
 *   global.jsonEval.getSchemaValueObject(handle) -> JSON string
 *   global.jsonEval.dispose(handle) -> void
 */
class JsonEvalJSI : public jsi::HostObject {
public:
  // Install onto a JS runtime. Returns true on success.
  static bool install(jsi::Runtime& runtime);

  // jsi::HostObject interface
  jsi::Value get(jsi::Runtime& runtime, const jsi::PropNameID& name) override;
  void set(jsi::Runtime& runtime, const jsi::PropNameID& name, const jsi::Value& value) override;
  std::vector<jsi::PropNameID> getPropertyNames(jsi::Runtime& runtime) override;

  // Helpers (public so free functions and lambdas can call them)
  static std::string stringFromValue(jsi::Runtime& runtime, const jsi::Value& val);
  static void checkResult(jsi::Runtime& runtime, const FFIResult& result);
  static void checkArgCount(jsi::Runtime& runtime, size_t actual, size_t expected);
};

} // namespace jsoneval
