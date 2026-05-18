# JSON Eval RS npm workspace

Yarn 1 workspace for JavaScript, web, Node.js, and React Native packages.

## Layout

```text
bindings/npm/
├── package.json
├── tsconfig.base.json
├── packages/
│   ├── common/        # @json-eval-rs/common
│   ├── webcore/       # @json-eval-rs/webcore
│   ├── node/          # @json-eval-rs/node
│   ├── bundler/       # @json-eval-rs/bundler
│   ├── vanilla/       # @json-eval-rs/vanilla
│   └── react-native/  # @json-eval-rs/react-native
└── examples/
    ├── nextjs/
    ├── nodejs/
    ├── nodejs-benchmark/
    ├── web-benchmark/
    └── rncli/
```

## Install

```bash
cd bindings/npm
yarn install
```

## Build

```bash
yarn build:common
yarn build:web
yarn build:react-native
```

`yarn build` runs all package builds.

## Examples

```bash
yarn example:web:nextjs
yarn example:react-native:start
yarn example:react-native:android
yarn example:react-native:ios
yarn example:react-native:pods
```

## Docs

- React Native package: [`packages/react-native/README.md`](./packages/react-native/README.md)
- React Native build guide: [`docs/react-native-BUILD.md`](./docs/react-native-BUILD.md)
- Web architecture: [`docs/web-ARCHITECTURE.md`](./docs/web-ARCHITECTURE.md)
