#pragma once

#include <jsi/jsi.h>
#include "jsi-bridge.h"

namespace jsoneval {

namespace jsi = facebook::jsi;

// Forward declaration of free function from Rust
extern "C" void json_eval_free_result(FFIResult result);

/**
 * HostObject that wraps an FFIResult and ensures its memory is freed 
 * when the JS object is garbage collected.
 */
class RustBuffer : public jsi::HostObject {
public:
    RustBuffer(FFIResult result) : result_(result) {}

    ~RustBuffer() {
        if (result_._owned_data) {
            json_eval_free_result(result_);
        }
    }

    /**
     * Create a JSI ArrayBuffer that points directly to the Rust memory.
     * The ArrayBuffer will keep this HostObject alive via its private data
     * or by being attached to it, ensuring zero-copy access.
     */
    jsi::Value toArrayBuffer(jsi::Runtime& runtime) {
        if (!result_.data_ptr || result_.data_len == 0) {
            return jsi::ArrayBuffer(runtime, 0);
        }

        // We use a HostObject as the "owner" of the data.
        // We create an ArrayBuffer that points to result_.data_ptr.
        // To ensure the memory isn't freed too early, we need to ensure 
        // this RustBuffer stays alive as long as the ArrayBuffer does.
        
        // In JSI, we can achieve this by using the constructor that takes a MutableBuffer.
        // If the JSI version is too old, we might need a different trick.
        
        struct RustMutableBuffer : public jsi::MutableBuffer {
            RustMutableBuffer(FFIResult result) : result_(result) {}
            ~RustMutableBuffer() {
                if (result_._owned_data) {
                    json_eval_free_result(result_);
                }
            }
            uint8_t* data() override { return const_cast<uint8_t*>(result_.data_ptr); }
            size_t size() const override { return result_.data_len; }
        private:
            FFIResult result_;
        };

        auto buffer = std::make_shared<RustMutableBuffer>(result_);
        // Clear result_ so destructor doesn't free it twice
        result_._owned_data = nullptr; 
        
        return jsi::ArrayBuffer(runtime, std::move(buffer));
    }

    // jsi::HostObject implementation
    jsi::Value get(jsi::Runtime& runtime, const jsi::PropNameID& name) override {
        auto prop = name.utf8(runtime);
        if (prop == "byteLength") {
            return jsi::Value(static_cast<double>(result_.data_len));
        }
        if (prop == "buffer") {
            return toArrayBuffer(runtime);
        }
        return jsi::Value::undefined();
    }

    std::vector<jsi::PropNameID> getPropertyNames(jsi::Runtime& runtime) override {
        return jsi::PropNameID::names(runtime, "byteLength", "buffer");
    }

private:
    FFIResult result_;
};

} // namespace jsoneval
