# JSON Eval RS - React Native Example

This is a React Native example demonstrating the `@json-eval-rs/react-native` binding with native Rust-powered validation and evaluation.

## Features

- ✅ **Form Validation** - Native form validation with JSON schema rules
- 🔄 **Dependent Fields** - Automatic calculation of dependent fields
- ⚡ **Native Performance** - Rust-powered, runs on native threads
- 🎨 **Modern UI** - Clean, responsive design with dark mode support
- 📱 **Cross-Platform** - Works on iOS and Android

## Prerequisites

>**Note**: Make sure you have completed the [React Native - Environment Setup](https://reactnative.dev/docs/environment-setup) instructions before proceeding.

## Step 1: Start the Metro Server

First, you will need to start **Metro**, the JavaScript _bundler_ that ships _with_ React Native.

To start Metro, run the following command from the _root_ of your React Native project:

```bash
# using npm
npm start

# OR using Yarn
yarn start
```

## Step 2: Start your Application

Let Metro Bundler run in its _own_ terminal. Open a _new_ terminal from the _root_ of your React Native project. Run the following command to start your _Android_ or _iOS_ app:

### For Android

```bash
# using npm
npm run android

# OR using Yarn
yarn android
```

### For iOS

```bash
# using npm
npm run ios

# OR using Yarn
yarn ios
```

If everything is set up _correctly_, you should see your new app running in your _Android Emulator_ or _iOS Simulator_ shortly provided you have set up your emulator/simulator correctly.

This is one way to run your app — you can also run it directly from within Android Studio and Xcode respectively.

## What's Inside

The example includes two screens demonstrating key features:

### 1. Form Validation Screen
- Validates form inputs using JSON schema rules
- Real-time error display
- Uses native Rust validation for 10-50x better performance

### 2. Dependent Fields Screen
- Automatic calculation of dependent fields (subtotal, tax, total)
- Real-time updates as you type
- Demonstrates dependency tracking with `evaluateDependents`

## Project Structure

```
rncli/
├── App.tsx                          # Main app with tab navigation
├── src/
│   └── screens/
│       ├── FormValidationScreen.tsx # Form validation demo
│       └── DependentFieldsScreen.tsx # Dependent fields demo
├── babel.config.js                  # Configured for local binding
├── metro.config.js                  # Monorepo configuration
└── tsconfig.json                    # TypeScript paths
```

## How It Works

This example uses the local `@json-eval-rs/react-native` binding from the parent directory:

```typescript
import { useJSONEval } from '@json-eval-rs/react-native';

const evalInstance = useJSONEval({ schema });

// Validate
const result = await evalInstance.validate({ data });

// Evaluate dependents
const updated = await evalInstance.evaluateDependents({
  changedPaths: ['field1', 'field2'],
  data,
  nested: true
});
```

## Building Native Bindings

Before running the app, you need to build the native bindings:

### iOS
```bash
cd ../../..  # Navigate to json-eval-rs root
cargo build --release --features ffi --target aarch64-apple-ios
cargo build --release --features ffi --target x86_64-apple-ios
```

### Android
```bash
cd ../../..  # Navigate to json-eval-rs root
cargo install cargo-ndk
cargo ndk -t arm64-v8a -o bindings/react-native/android/src/main/jniLibs build --release --features ffi
```

## Performance

- **Schema parsing**: < 5ms
- **Validation**: < 3ms per validation
- **Dependent field updates**: < 3ms per update

Native performance is 10-50x faster than JavaScript-only solutions.

# Troubleshooting

If you can't get this to work, see the [Troubleshooting](https://reactnative.dev/docs/troubleshooting) page.

# Learn More

## JSON Eval RS Resources

- [json-eval-rs Documentation](../../../README.md) - Main library documentation
- [React Native Bindings Guide](../../README.md) - API reference and usage guide
- [Build Instructions](../../BUILD.md) - How to build native bindings

## React Native Resources

- [React Native Website](https://reactnative.dev) - Learn more about React Native
- [Getting Started](https://reactnative.dev/docs/environment-setup) - Environment setup guide
- [Learn the Basics](https://reactnative.dev/docs/getting-started) - React Native basics tutorial

## License

MIT
