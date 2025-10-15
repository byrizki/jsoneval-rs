# Build Alignment Verification

## Summary of Fixes Applied

All build scripts and workflows have been aligned with the actual code structure.

---

## Web Bindings

### **FIXED**: Output Directory Paths

**Previous (WRONG):**
```bash
# build-bindings.sh & workflows
wasm-pack build --target web --out-dir bindings/web/pkg
wasm-pack build --target bundler --out-dir bindings/web/pkg-bundler
wasm-pack build --target nodejs --out-dir bindings/web/pkg-node
```

**Current (CORRECT):**
```bash
# build-bindings.sh & workflows
wasm-pack build --target bundler --out-dir bindings/web/packages/bundler/pkg
wasm-pack build --target nodejs --out-dir bindings/web/packages/node/pkg
```

### Verification
```javascript
// packages/bundler/index.js imports:
import * as wasm from './pkg/json_eval_rs.js';  // ✅ Matches build output

// packages/node/index.js imports:
import * as wasm from './pkg/json_eval_rs.js';  // ✅ Matches build output
```

### Package Structure
```
bindings/web/
├── packages/
│   ├── core/          # Internal API wrapper (no WASM build needed)
│   ├── bundler/       # ✅ WASM built to pkg/
│   │   └── pkg/       # ← wasm-pack output
│   └── node/          # ✅ WASM built to pkg/
│       └── pkg/       # ← wasm-pack output
```

---

## React Native Bindings

### Android JNI Libraries

**Build Command:**
```bash
cargo ndk -t ${ARCH} -o bindings/react-native/android/src/main/jniLibs build --release --features ffi
```

**Output Locations:**
1. **CMake linking** (build-time): `/target/${ANDROID_ABI}/release/libjson_eval_rs.so`
   - Used by CMakeLists.txt line 13-16
2. **APK packaging** (runtime): `bindings/react-native/android/src/main/jniLibs/${ANDROID_ABI}/libjson_eval_rs.so`
   - Used by build.gradle line 59

**Architectures:**
- ✅ arm64-v8a (ARM 64-bit)
- ✅ armeabi-v7a (ARM 32-bit)
- ✅ x86 (32-bit emulator)
- ✅ x86_64 (64-bit emulator)

### iOS Libraries

**Build Command:**
```bash
cargo build --release --features ffi --target aarch64-apple-ios
cargo build --release --features ffi --target x86_64-apple-ios
```

**Output Locations:**
- Device: `/target/aarch64-apple-ios/release/libjson_eval_rs.a`
- Simulator: `/target/x86_64-apple-ios/release/libjson_eval_rs.a`

**Verification:**
```ruby
# json-eval-rs.podspec line 29:
s.vendored_libraries = "../../target/aarch64-apple-ios/release/libjson_eval_rs.a"
# ✅ Matches build output
```

---

## Build Script Alignment

### `build-bindings.sh`

**Changes Made:**
1. ✅ Web output directories corrected to `packages/*/pkg/`
2. ✅ Android builds all 4 architectures (was only arm64-v8a)
3. ✅ Removed unused "web" target (no vanilla package exists)
4. ✅ Added web dependencies installation

### `.github/workflows/build-bindings.yml`

**Changes Made:**
1. ✅ Web output directories corrected to `packages/*/pkg/`
2. ✅ Removed unused "web" target build
3. ✅ Added example dependencies installation step
4. ✅ WASM artifacts upload paths corrected

### `.github/workflows/publish.yml`

**Changes Made:**
1. ✅ Web output directories corrected to `packages/*/pkg/`
2. ✅ Removed unused "web" target build

---

## Verification Checklist

### Web Bindings
- [x] Build output matches import paths in code
- [x] No orphaned pkg directories at root level
- [x] Dependencies install correctly
- [x] Workflow artifacts reference correct paths

### React Native - Android
- [x] JNI libs built to correct locations for CMake
- [x] JNI libs copied to correct locations for APK
- [x] All 4 architectures supported
- [x] NDK setup configured in workflow

### React Native - iOS  
- [x] Static libs built to locations referenced by podspec
- [x] Both device and simulator targets built
- [x] Podspec paths are correct

### Examples
- [x] rncli example dependencies installation added
- [x] Example can find the binding package

---

## Testing Commands

### Local Testing
```bash
# Test web build
./build-bindings.sh web
ls -la bindings/web/packages/bundler/pkg/
ls -la bindings/web/packages/node/pkg/

# Test React Native build (requires cargo-ndk and NDK)
./build-bindings.sh react-native
ls -la bindings/react-native/android/src/main/jniLibs/
ls -la target/aarch64-apple-ios/release/

# Test all
./build-bindings.sh all
```

### Workflow Testing
Push to a branch and check Actions tab to verify:
1. Web WASM artifacts contain `packages/bundler/pkg/` and `packages/node/pkg/`
2. Android JNI artifacts contain all 4 architectures
3. React Native package contains all native libraries
4. No "file not found" errors in any job

---

## Breaking Changes

**None** - These are corrections to match existing code structure, not code changes.

---

## Notes

1. **Web "vanilla" target**: The ARCHITECTURE.md mentions a "vanilla" package but it doesn't exist in the codebase. Only bundler and node packages exist.

2. **iOS XCFramework**: Currently building separate static libs. Could be enhanced to build XCFramework for better distribution.

3. **Android**: cargo-ndk handles both CMake linking and APK packaging automatically.

4. **Package publishing**: Web bindings are published as a monorepo root package that includes all sub-packages.

---

## Related Files Modified

- ✅ `/build-bindings.sh`
- ✅ `/.github/workflows/build-bindings.yml`
- ✅ `/.github/workflows/publish.yml`

---

*Last updated: 2025-10-15*
