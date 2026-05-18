# Build Alignment Verification

This file records how GitHub Actions paths line up with current repository structure.

## Source layout

```text
Cargo.toml                         # Rust crate version and features
bindings/csharp/                   # NuGet package project
bindings/npm/                      # Yarn workspace for npm packages and examples
docs/                              # Nuxt/Docus documentation site
```

## Rust features

- Default feature set is empty.
- `ffi` builds native C ABI libraries used by C#, React Native, desktop, iOS, and Android artifacts.
- `wasm` builds WebAssembly packages with `wasm-bindgen`.

Workflow checks:

```bash
cargo test
cargo test --all-features
cargo build --release --features ffi
wasm-pack build --release --target bundler --out-dir bindings/npm/packages/bundler/pkg --features wasm
wasm-pack build --release --target nodejs --out-dir bindings/npm/packages/node/pkg --features wasm
wasm-pack build --release --target web --out-dir bindings/npm/packages/vanilla/pkg --features wasm
```

## Web/WASM packages

`release.yml` writes wasm-pack output into package-local `pkg/` directories:

```text
bindings/npm/packages/bundler/pkg/
bindings/npm/packages/node/pkg/
bindings/npm/packages/vanilla/pkg/
```

These paths match package `files` declarations:

- `bindings/npm/packages/bundler/package.json`
- `bindings/npm/packages/node/package.json`
- `bindings/npm/packages/vanilla/package.json`

TypeScript package builds run from `bindings/npm` with Yarn workspaces:

```bash
yarn workspace @json-eval-rs/common build
yarn workspace @json-eval-rs/webcore build
yarn workspace @json-eval-rs/bundler build
yarn workspace @json-eval-rs/node build
yarn workspace @json-eval-rs/vanilla build
```

Release assets include npm tarballs from:

```text
bindings/npm/packages/common/*.tgz
bindings/npm/packages/webcore/*.tgz
bindings/npm/packages/bundler/*.tgz
bindings/npm/packages/node/*.tgz
bindings/npm/packages/vanilla/*.tgz
```

## React Native package

Android JNI libraries are copied into:

```text
bindings/npm/packages/react-native/android/src/main/jniLibs/<ABI>/libjson_eval_rs.so
```

Supported ABIs:

- `arm64-v8a`
- `armeabi-v7a`
- `x86`
- `x86_64`

iOS release artifact is copied into:

```text
bindings/npm/packages/react-native/ios/JsonEvalRs.xcframework/
```

This matches `bindings/npm/packages/react-native/json-eval-rs.podspec`:

```ruby
s.vendored_frameworks = 'ios/JsonEvalRs.xcframework'
```

React Native package tarball comes from:

```bash
cd bindings/npm/packages/react-native
npm pack
```

## C# package

Native desktop libraries are copied to NuGet runtime identifiers:

```text
bindings/csharp/runtimes/linux-x64/native/libjson_eval_rs.so
bindings/csharp/runtimes/linux-arm64/native/libjson_eval_rs.so
bindings/csharp/runtimes/win-x64/native/json_eval_rs.dll
bindings/csharp/runtimes/osx-x64/native/libjson_eval_rs.dylib
bindings/csharp/runtimes/osx-arm64/native/libjson_eval_rs.dylib
```

NuGet package command:

```bash
cd bindings/csharp
dotnet pack -c Release
```

Release asset:

```text
bindings/csharp/bin/Release/*.nupkg
```

## Docs

Docs workflow runs with working directory `docs` and uploads:

```text
docs/.output/public
```

Build command:

```bash
NUXT_APP_BASE_URL=/jsoneval-rs/ yarn build
```

## Integrity checks

Before committing workflow changes:

```bash
python - <<'PY'
import pathlib, yaml
for path in pathlib.Path('.github/workflows').glob('*.yml'):
    yaml.safe_load(path.read_text())
    print(f'OK {path}')
PY

! grep -R -E 'build-bindings\.yml|pages\.yml' .github/workflows README.md .github/SETUP_GUIDE.md

git diff --check
```

`bindings/npm` is current repo layout. Do not replace it unless the workspace is actually renamed.
