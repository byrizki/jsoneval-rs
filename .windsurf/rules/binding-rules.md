---
trigger: always_on
---

When modifying Rust core codebase, ensture to all binding code must be updated and aligned with the Core codebase

FFI
- src/ffi/*

C#
- bindings/csharp/JsonEvalRs.*

React Native
- bindings/react-native/packages/react-native/android/src/main/cpp/json-eval-rn.cpp
- bindings/react-native/packages/react-native/android/src/main/java/com/jsonevalrs/*
- bindings/react-native/packages/react-native/cpp/*
- bindings/react-native/packages/react-native/ios/*
- bindings/react-native/packages/react-native/src/index.tsx

WASM
- src/wasm/*

Web Core
- bindings/web/packages/core/*