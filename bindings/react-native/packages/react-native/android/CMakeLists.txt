cmake_minimum_required(VERSION 3.9.0)

project(json-eval-rn)

set(CMAKE_VERBOSE_MAKEFILE ON)
set(CMAKE_CXX_STANDARD 17)

# Find React Native
find_package(ReactAndroid REQUIRED CONFIG)

# Pre-built Rust library (bundled with npm package)
# The library is located in src/main/jniLibs/[abi]/libjson_eval_rs.so
set(RUST_PREBUILT_DIR ${CMAKE_SOURCE_DIR}/src/main/jniLibs/${ANDROID_ABI})
set(RUST_LIB_PATH ${RUST_PREBUILT_DIR}/libjson_eval_rs.so)

message(STATUS "Looking for pre-built Rust library at: ${RUST_LIB_PATH}")

# Check if pre-built library exists
if(NOT EXISTS ${RUST_LIB_PATH})
    message(FATAL_ERROR 
        "Pre-built Rust library not found at: ${RUST_LIB_PATH}\n"
        "\nThis package requires pre-built native libraries.\n"
        "If you're developing this package, build the libraries first:\n"
        "  cd ${CMAKE_SOURCE_DIR}/../..\n"
        "  ./build-android.sh all\n"
        "\nIf you're a user and seeing this error, the package may be corrupted.\n"
        "Try reinstalling: npm install @json-eval-rs/react-native\n"
        "\nCurrent ANDROID_ABI: ${ANDROID_ABI}")
endif()

message(STATUS "Using pre-built Rust library: ${RUST_LIB_PATH}")

# Add library search path for the Rust library
link_directories(${RUST_PREBUILT_DIR})

# C++ Bridge files
add_library(
  json_eval_rn
  SHARED
  src/main/cpp/json-eval-rn.cpp
  ../cpp/json-eval-bridge.cpp
)

# Include directories
target_include_directories(
  json_eval_rn
  PRIVATE
  ../cpp
  src/main/cpp
)

# Link libraries
target_link_libraries(
  json_eval_rn
  ReactAndroid::jsi
  ReactAndroid::reactnativejni
  json_eval_rs
  android
  log
)
