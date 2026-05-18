import os
import re

ROOT = "/mnt/development/jsoneval-rs"

def get_matches(file_paths, pattern, group=1):
    methods = set()
    for path in file_paths:
        full_path = os.path.join(ROOT, path)
        if not os.path.exists(full_path):
            continue
        with open(full_path, "r") as f:
            content = f.read()
            for match in re.finditer(pattern, content, re.MULTILINE):
                methods.add(match.group(group))
    return methods

def snake_to_camel(s):
    parts = s.split('_')
    return parts[0] + ''.join(p.title() for p in parts[1:])

# 1. RUST FFI & WASM
rust_ffi_files = ["src/ffi/schema.rs", "src/ffi/subforms.rs", "src/ffi/evaluation.rs"]
rust_ffi_methods = get_matches(rust_ffi_files, r'pub unsafe extern "C" fn json_eval_([a-z0-9_]+)')
rust_ffi_camel = {snake_to_camel(m) for m in rust_ffi_methods}

rust_wasm_files = ["src/wasm/schema.rs", "src/wasm/subforms.rs", "src/wasm/evaluation.rs", "src/wasm/lib.rs"]
rust_wasm_methods = get_matches(rust_wasm_files, r'#\[wasm_bindgen\(js_name\s*=\s*([a-zA-Z0-9_]+)\)\]')

rust_combined = rust_ffi_camel.union(rust_wasm_methods)

# 2. C# FFI
csharp_files = ["bindings/csharp/JsonEvalRs.Native.NetCore.cs", "bindings/csharp/JsonEvalRs.Native.NetStandard.cs", "bindings/csharp/JsonEvalRs.Native.Common.cs", "bindings/csharp/JsonEvalRs.Main.cs", "bindings/csharp/JsonEvalRs.Subforms.cs"]
csharp_ffi_methods = get_matches(csharp_files, r'internal\s+static\s+extern\s+[a-zA-Z0-9_]+\s+json_eval_([a-z0-9_]+)\(')
csharp_ffi_camel = {snake_to_camel(m) for m in csharp_ffi_methods}

# 3. RN JSI
rn_jsi_files = ["bindings/react-native/packages/react-native/cpp/jsi-bridge.cpp"]
rn_jsi_methods = get_matches(rn_jsi_files, r'prop\s*==\s*"([a-zA-Z0-9_]+)"')

# 4. RN JNI
rn_jni_files = ["bindings/react-native/packages/react-native/android/src/main/cpp/json-eval-rn.cpp"]
rn_jni_raw = get_matches(rn_jni_files, r'Java_com_jsonevalrs_JsonEvalRsModule_native([a-zA-Z0-9_]+)')
rn_jni_methods = {m.replace("Async", "") for m in rn_jni_raw}
rn_jni_camel = {m[0].lower() + m[1:] for m in rn_jni_methods if m}

# 5. RN KOTLIN
rn_kotlin_files = ["bindings/react-native/packages/react-native/android/src/main/java/com/jsonevalrs/JsonEvalRsModule.kt"]
rn_kotlin_methods = get_matches(rn_kotlin_files, r'@ReactMethod.*?\n\s*fun\s+([a-zA-Z0-9_]+)')

# 6. RN IOS
rn_ios_files = ["bindings/react-native/packages/react-native/ios/JsonEvalRs.mm"]
rn_ios_methods1 = get_matches(rn_ios_files, r'RCT_EXPORT_METHOD\(([a-zA-Z0-9_]+)')
rn_ios_methods2 = get_matches(rn_ios_files, r'RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD\(([a-zA-Z0-9_]+)')
rn_ios_methods = rn_ios_methods1.union(rn_ios_methods2)

# 7. RN TS
rn_ts_files = ["bindings/react-native/packages/react-native/src/jsi-bridge.ts", "bindings/react-native/packages/react-native/src/index.tsx"]
rn_ts_methods = get_matches(rn_ts_files, r'^\s+([a-zA-Z0-9_]+)\(', group=1)

# 8. WEB WASM
web_wasm_files = ["bindings/web/packages/vanilla/pkg/json_eval_rs.d.ts"]
web_wasm_methods = get_matches(web_wasm_files, r'^\s*(?:static\s+)?([a-zA-Z0-9_]+)\(.*?\)\s*:', group=1)
web_wasm_clean = {m for m in web_wasm_methods if not m.startswith('__') and not m.startswith('jsonevalwasm') and not m.startswith('validation') and m not in ('free', 'constructor')}

# 9. WEB TS
web_ts_files = ["bindings/web/packages/core/src/index.ts"]
web_ts_methods = get_matches(web_ts_files, r'public\s+(?:async\s+)?([a-zA-Z0-9_]+)\s*\(')

# Compile master list of methods
all_methods = set()
for s in [rust_combined, csharp_ffi_camel, rn_jsi_methods, rn_jni_camel, rn_kotlin_methods, rn_ios_methods, rn_ts_methods, web_wasm_clean, web_ts_methods]:
    all_methods.update(s)

# Filter out internal/irrelevant methods
exclude = {'free', 'new', 'newWithError', 'newFromCache', 'newFromCacheWithError', 'newFromMsgpack', 'freeResult', 'freeString', 'installJSI', 'multiply', 'constructor', 'initSync'}
all_methods = sorted(list(all_methods - exclude))

# Build markdown table
print("| Method | RUST WASM/FFI | C# FFI | RN JSI | RN JNI | RN KOTLIN | RN IOS | RN TS | WEB WASM | WEB TS |")
print("|---|---|---|---|---|---|---|---|---|---|")

def check(method, method_set):
    return "✅" if method in method_set else "❌"

for m in all_methods:
    cols = [
        m,
        check(m, rust_combined),
        check(m, csharp_ffi_camel),
        check(m, rn_jsi_methods),
        check(m, rn_jni_camel),
        check(m, rn_kotlin_methods),
        check(m, rn_ios_methods),
        check(m, rn_ts_methods),
        check(m, web_wasm_clean),
        check(m, web_ts_methods)
    ]
    print("| " + " | ".join(cols) + " |")

