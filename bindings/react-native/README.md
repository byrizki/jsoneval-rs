# JSON Eval RS - React Native Bindings (Monorepo)

High-performance JSON Logic evaluator with schema validation for React Native.

This is a Yarn workspace monorepo containing the React Native bindings and examples.

## Features

- 🚀 **Native Performance** - Built with Rust for iOS and Android
- ✅ **Schema Validation** - Validate data against JSON schema rules
- 🔄 **Dependency Tracking** - Auto-update dependent fields
- 🎯 **Type Safe** - Full TypeScript support
- ⚛️ **React Hooks** - Built-in `useJSONEval` hook
- 📱 **Cross-Platform** - Works on iOS and Android
- 🔥 **Fast** - Native performance, not JavaScript

## Monorepo Structure

```
bindings/react-native/
├── packages/
│   └── react-native/         # Main React Native package
│       ├── src/              # TypeScript source
│       ├── android/          # Android native code (Kotlin + JNI)
│       ├── ios/              # iOS native code (Objective-C++)
│       ├── cpp/              # C++ bridge code
│       └── lib/              # Built output
└── examples/
    └── rncli/                # React Native CLI example app
```

## Packages

### [@json-eval-rs/react-native](./packages/react-native)

The main React Native package with native iOS and Android bindings.

**[📖 Full Documentation](./packages/react-native/README.md)**

## Examples

### [React Native CLI Example](./examples/rncli)

A complete example app demonstrating form validation, dependent fields, and more.

## Getting Started

### Development Setup

1. Install dependencies:
```bash
yarn install
```

2. Build the package:
```bash
yarn build
```

3. Run the example app:
```bash
# iOS
yarn rncli ios

# Android
yarn rncli android
```

## Installation (For End Users)

```bash
yarn install @json-eval-rs/react-native
# or
yarn add @json-eval-rs/react-native
```

See the [package README](./packages/react-native/README.md) for detailed usage instructions.

## Quick Example

```typescript
import { JSONEval } from '@json-eval-rs/react-native';

const schema = {
  type: 'object',
  properties: {
    user: {
      type: 'object',
      properties: {
        name: {
          type: 'string',
          rules: {
            required: { value: true, message: 'Name is required' }
          }
        }
      }
    }
  }
};

const eval = new JSONEval({ schema });
const data = { user: { name: 'John' } };
const result = await eval.evaluate({ data });
await eval.dispose();
```

## Scripts

- `yarn build` - Build the React Native package
- `yarn clean` - Clean build artifacts
- `yarn test` - Run tests in all workspaces
- `yarn lint` - Lint code in all workspaces
- `yarn rncli [command]` - Run commands in the example app

## License

MIT

## Support

- GitHub Issues: https://github.com/byrizki/json-eval-rs/issues
- Documentation: https://github.com/byrizki/json-eval-rs

