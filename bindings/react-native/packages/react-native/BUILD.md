# Building React Native Bindings for JSON Eval RS

This guide explains how to build the React Native native module with proper Android and iOS bindings.

## Architecture

The React Native binding uses a three-layer architecture:

1. **Rust FFI Layer** - Core library with C-compatible FFI functions
2. **C++ Bridge Layer** - Thread-safe wrapper that manages handles and async execution
3. **Native Platform Layer**:
   - **Android**: Kotlin + JNI bindings
   - **iOS**: Objective-C++ bindings

All operations run asynchronously on separate threads to avoid blocking the JavaScript thread.

## Prerequisites

### General
- Rust toolchain (1.70+)
- Node.js (16+)
- Yarn or npm

### Android
- Android Studio (2023.1+)
- Android NDK (25+)
- CMake (3.18+)
- Java/Kotlin development tools

### iOS
- Xcode (14+)
- CocoaPods (1.12+)
- macOS (for iOS development)

## Building the Rust Library

### 1. Install Rust Targets

```bash
# Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android

# iOS targets
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim
```

### 2. Install cargo-ndk (for Android)

```bash
cargo install cargo-ndk
```

### 3. Build for Android

```bash
# From the root of the repository
cd /path/to/json-eval-rs

# Set Android NDK path
export ANDROID_NDK_HOME=/path/to/android-ndk

# Build for all Android architectures
cargo ndk --platform 21 --target aarch64-linux-android build --release --features ffi
cargo ndk --platform 21 --target armv7-linux-androideabi build --release --features ffi
cargo ndk --platform 21 --target x86_64-linux-android build --release --features ffi
cargo ndk --platform 21 --target i686-linux-android build --release --features ffi

# Copy libraries to React Native jniLibs
mkdir -p bindings/react-native/android/src/main/jniLibs/arm64-v8a
mkdir -p bindings/react-native/android/src/main/jniLibs/armeabi-v7a
mkdir -p bindings/react-native/android/src/main/jniLibs/x86_64
mkdir -p bindings/react-native/android/src/main/jniLibs/x86

cp target/aarch64-linux-android/release/libjson_eval_rs.so \
   bindings/react-native/android/src/main/jniLibs/arm64-v8a/

cp target/armv7-linux-androideabi/release/libjson_eval_rs.so \
   bindings/react-native/android/src/main/jniLibs/armeabi-v7a/

cp target/x86_64-linux-android/release/libjson_eval_rs.so \
   bindings/react-native/android/src/main/jniLibs/x86_64/

cp target/i686-linux-android/release/libjson_eval_rs.so \
   bindings/react-native/android/src/main/jniLibs/x86/
```

### 4. Build for iOS

```bash
# Build for iOS device (arm64)
cargo build --release --target aarch64-apple-ios --features ffi

# Build for iOS simulator (arm64 Mac)
cargo build --release --target aarch64-apple-ios-sim --features ffi

# Build for iOS simulator (x86_64 Intel Mac)
cargo build --release --target x86_64-apple-ios --features ffi

# Create universal library for simulator
lipo -create \
  target/aarch64-apple-ios-sim/release/libjson_eval_rs.a \
  target/x86_64-apple-ios/release/libjson_eval_rs.a \
  -output target/universal-ios-sim/libjson_eval_rs.a
```

## Building the React Native Module

### 1. Install Dependencies

```bash
cd bindings/react-native
yarn install
```

### 2. Build TypeScript

```bash
yarn prepare
```

### 3. Android Build

The Android build uses CMake and is integrated with Gradle:

```bash
cd android
./gradlew assembleRelease
```

The CMake configuration will:
- Compile the C++ bridge (`cpp/json-eval-bridge.cpp`)
- Compile the JNI bindings (`android/src/main/cpp/json-eval-rn.cpp`)
- Link against the Rust library (`libjson_eval_rs.so`)
- Create the final `libjson-eval-rn.so`

### 4. iOS Build

```bash
cd ios
pod install
```

The Podspec will:
- Include C++ bridge files
- Include Objective-C++ implementation
- Link against the Rust static library
- Configure C++17 standard

## Testing in a React Native App

### Android

1. Add to your `package.json`:
```json
{
  "dependencies": {
    "@json-eval-rs/react-native": "file:../path/to/json-eval-rs/bindings/react-native"
  }
}
```

2. Run:
```bash
yarn install
yarn android
```

### iOS

1. Add to your `package.json` (same as above)

2. Run:
```bash
yarn install
cd ios && pod install && cd ..
yarn ios
```

## Build Scripts

For convenience, you can use the provided build script:

```bash
#!/bin/bash
# build-native.sh

set -e

echo "Building Rust library for all platforms..."

# Android
export ANDROID_NDK_HOME=${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk-bundle}

for TARGET in aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
do
  echo "Building for Android $TARGET..."
  cargo ndk --platform 21 --target $TARGET build --release --features ffi
done

# iOS
for TARGET in aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
do
  echo "Building for iOS $TARGET..."
  cargo build --release --target $TARGET --features ffi
done

echo "Copying Android libraries..."
mkdir -p bindings/react-native/android/src/main/jniLibs/{arm64-v8a,armeabi-v7a,x86_64,x86}

cp target/aarch64-linux-android/release/libjson_eval_rs.so \
   bindings/react-native/android/src/main/jniLibs/arm64-v8a/

cp target/armv7-linux-androideabi/release/libjson_eval_rs.so \
   bindings/react-native/android/src/main/jniLibs/armeabi-v7a/

cp target/x86_64-linux-android/release/libjson_eval_rs.so \
   bindings/react-native/android/src/main/jniLibs/x86_64/

cp target/i686-linux-android/release/libjson_eval_rs.so \
   bindings/react-native/android/src/main/jniLibs/x86/

echo "Build complete!"
```

## Troubleshooting

### Android

**CMake can't find Rust library:**
- Ensure you've built the Rust library first
- Check that `libjson_eval_rs.so` exists in `target/[arch]/release/`
- Verify `ANDROID_NDK_HOME` is set correctly

**JNI method not found:**
- Clean and rebuild: `cd android && ./gradlew clean && ./gradlew assembleRelease`
- Check that C++ method signatures match Kotlin native declarations

### iOS

**Linker errors about missing symbols:**
- Ensure you've built the Rust library for the correct architecture
- For simulator, use `aarch64-apple-ios-sim` on Apple Silicon or `x86_64-apple-ios` on Intel
- For device, use `aarch64-apple-ios`

**Pod install fails:**
- Update CocoaPods: `gem install cocoapods`
- Clear cache: `pod cache clean --all`
- Try: `cd ios && pod deintegrate && pod install`

## Performance Considerations

- All operations run asynchronously on background threads via `std::thread::detach()`
- The C++ bridge manages a thread-safe handle registry using `std::mutex`
- JNI/iOS callbacks properly attach/detach threads to avoid memory leaks
- Results are passed back to JavaScript via Promise callbacks

## Next Steps

1. See [README.md](./README.md) for API documentation
2. Check [examples/](./examples/) for usage examples
3. Read the main repository documentation for Rust API details
