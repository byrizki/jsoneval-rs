# Publishing Guide for JSON Eval RS Bindings

This guide covers the complete process of publishing all language bindings to their respective package registries.

## Pre-Publishing Checklist

Before publishing any package, ensure:

- [ ] All tests pass (`cargo test`)
- [ ] Documentation is up to date
- [ ] Version numbers are updated consistently across all packages
- [ ] CHANGELOG.md is updated
- [ ] README files are accurate
- [ ] License files are included
- [ ] Build scripts work on all platforms

## Version Management

All packages should maintain the same version number:
- Rust crate: `Cargo.toml`
- C# package: `JsonEvalRs.csproj`
- npm workspace root: `bindings/npm/package.json` (private; never published)
- npm packages: `bindings/npm/packages/*/package.json`

Update all simultaneously when releasing.

## 1. Publishing C# Package to NuGet

### Prerequisites

- .NET SDK 6.0 or later
- NuGet account ([signup](https://www.nuget.org/users/account/LogOn))
- API key from [NuGet.org](https://www.nuget.org/account/apikeys)

### Steps

#### 1.1. Build Native Libraries

```bash
# Linux
cargo build --release --features ffi

# macOS
cargo build --release --features ffi --target x86_64-apple-darwin
cargo build --release --features ffi --target aarch64-apple-darwin

# Windows (from Windows or cross-compile)
cargo build --release --features ffi --target x86_64-pc-windows-msvc
```

#### 1.2. Update Package Metadata

Edit `bindings/csharp/JsonEvalRs.csproj`:

```xml
<Version>0.0.1</Version>
<PackageReleaseNotes>
  - Initial release
  - Schema evaluation support
  - Validation support
  - Dependency tracking
</PackageReleaseNotes>
```

#### 1.3. Build NuGet Package

```bash
cd bindings/csharp
dotnet pack -c Release
```

This creates `bin/Release/JsonEvalRs.0.0.1.nupkg`

#### 1.4. Test Package Locally

```bash
# Create local test project
dotnet new console -n TestJsonEvalRs
cd TestJsonEvalRs
dotnet add package JsonEvalRs --source ../bindings/csharp/bin/Release
```

Test the package works correctly.

#### 1.5. Publish to NuGet

```bash
cd bindings/csharp
dotnet nuget push bin/Release/JsonEvalRs.0.0.1.nupkg \
  --api-key YOUR_API_KEY \
  --source https://api.nuget.org/v3/index.json
```

#### 1.6. Verify Publication

Visit https://www.nuget.org/packages/JsonEvalRs and verify the package appears.

### Symbol Package (Optional)

For debugging support:

```bash
dotnet pack -c Release -p:IncludeSymbols=true -p:SymbolPackageFormat=snupkg
dotnet nuget push bin/Release/JsonEvalRs.0.0.1.snupkg \
  --api-key YOUR_API_KEY \
  --source https://api.nuget.org/v3/index.json
```

## 2. Publishing Web Package to npm

### Prerequisites

- Node.js 14 or later
- npm account ([signup](https://www.npmjs.com/signup))
- wasm-pack installed

### Steps

#### 2.1. Build WASM Module

```bash
# Build for web
wasm-pack build --target web --out-dir bindings/npm/packages/vanilla/pkg --features wasm

# Build for Node.js
wasm-pack build --target nodejs --out-dir bindings/npm/packages/node/pkg --features wasm

# Build for bundlers
wasm-pack build --target bundler --out-dir bindings/npm/packages/bundler/pkg --features wasm
```

#### 2.2. Update Package Metadata

Edit the relevant package metadata under `bindings/npm/packages/<package>/package.json`. `bindings/npm/package.json` is a private workspace root and must not be published.

Examples:
- `bindings/npm/packages/common/package.json`
- `bindings/npm/packages/vanilla/package.json`
- `bindings/npm/packages/node/package.json`
- `bindings/npm/packages/bundler/package.json`

```json
{
  "version": "0.0.1",
  "description": "Updated description",
  "keywords": ["json", "logic", "wasm", "rust"]
}
```

#### 2.3. Test Package Locally

```bash
cd bindings/npm

# Run workspace tests/builds
yarn test
yarn workspace @json-eval-rs/common build
yarn workspace @json-eval-rs/webcore build
yarn workspace @json-eval-rs/bundler build
yarn workspace @json-eval-rs/node build
yarn workspace @json-eval-rs/vanilla build

# Test packaging from the package workspace, not the private root
cd packages/bundler
npm pack

# Install locally in test project
cd /path/to/test/project
yarn install /path/to/json-eval-rs/bindings/npm/packages/bundler/json-eval-rs-bundler-0.0.1.tgz
```

#### 2.4. Login to npm

```bash
npm login
```

Enter your credentials.

#### 2.5. Publish to npm

```bash
# The bindings/npm root is private and must not be published.
# Publish each public package from its workspace directory.
cd bindings/npm/packages/common && npm publish --access public
cd ../webcore && npm publish --access public
cd ../bundler && npm publish --access public
cd ../node && npm publish --access public
cd ../vanilla && npm publish --access public
```

#### 2.6. Verify Publication

Visit https://www.npmjs.com/package/@json-eval-rs/vanilla

### Publishing Different Builds

Publish target-specific packages from their package roots after generating WASM into each package's `pkg/` directory:

```bash
cd bindings/npm/packages/vanilla && npm publish --access public
cd ../node && npm publish --access public
cd ../bundler && npm publish --access public
```

## 3. Publishing React Native Package to npm

### Prerequisites

- Node.js 14 or later
- npm account
- Native libraries built for iOS and Android

### Steps

#### 3.1. Build Native Libraries

```bash
# Android (requires cargo-ndk)
cargo install cargo-ndk
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86 -t x86_64 \
  -o bindings/npm/packages/react-native/android/src/main/jniLibs \
  build --release --features ffi

# iOS (requires macOS)
cargo build --release --features ffi --target aarch64-apple-ios
cargo build --release --features ffi --target x86_64-apple-ios
cargo build --release --features ffi --target aarch64-apple-ios-sim

# Create universal library
lipo -create \
  target/x86_64-apple-ios/release/libjson_eval_rs.a \
  target/aarch64-apple-ios-sim/release/libjson_eval_rs.a \
  -output bindings/npm/packages/react-native/ios/libjson_eval_rs.a
```

#### 3.2. Update Package Metadata

Edit `bindings/npm/packages/react-native/package.json`:

```json
{
  "version": "0.0.1",
  "description": "Updated description"
}
```

#### 3.3. Test Package

```bash
cd bindings/npm

# Install dependencies
yarn install

# Build React Native package
cd packages/react-native
npm run prepare

# Test in example app from workspace root
cd ../..
yarn workspace rncli android
```

#### 3.4. Publish to npm

```bash
# bindings/npm is a private workspace root; publish only the package workspace.
cd bindings/npm/packages/react-native
npm publish --access public
```

#### 3.5. Verify Publication

Visit https://www.npmjs.com/package/@json-eval-rs/react-native

## 4. Publishing Rust Crate to crates.io

### Prerequisites

- Cargo installed
- crates.io account ([signup](https://crates.io/))
- GitHub repository set up

### Steps

#### 4.1. Update Cargo.toml

```toml
[package]
name = "json-eval-rs"
version = "0.0.1"
description = "High-performance JSON Logic evaluator with schema validation"
license = "MIT"
repository = "https://github.com/byrizki/jsoneval-rs"
documentation = "https://docs.rs/json-eval-rs"
keywords = ["json", "logic", "schema", "validation", "evaluation"]
categories = ["parser-implementations", "data-structures"]
```

#### 4.2. Test Package

```bash
# Check for issues
cargo publish --dry-run

# Run tests
cargo test --all-features
```

#### 4.3. Login to crates.io

```bash
cargo login YOUR_API_TOKEN
```

Get your API token from https://crates.io/me

#### 4.4. Publish

```bash
cargo publish
```

#### 4.5. Verify

Visit https://crates.io/crates/json-eval-rs

## Post-Publishing Tasks

After publishing all packages:

### 1. Create GitHub Release

```bash
git tag v0.0.1
git push origin v0.0.1
```

Create release on GitHub with:
- Release notes
- Links to all published packages
- Migration guide (if applicable)

### 2. Update Documentation

- Update main README.md with new version
- Update installation instructions
- Add to CHANGELOG.md

### 3. Announce Release

- Post on relevant forums/communities
- Update project website
- Send notification to users (if mailing list exists)

### 4. Monitor

- Watch for issues on GitHub
- Monitor download stats
- Respond to questions

## Troubleshooting

### NuGet Publishing Issues

**Problem:** "Package already exists"  
**Solution:** Increment version number, NuGet doesn't allow overwriting

**Problem:** "Invalid package"  
**Solution:** Run `dotnet pack` with `-v detailed` to see specific errors

### npm Publishing Issues

**Problem:** "You cannot publish over the previously published version"  
**Solution:** Update version in package.json

**Problem:** "403 Forbidden"  
**Solution:** Ensure you're logged in (`npm whoami`) and have access rights

**Problem:** "Module not found after publishing"  
**Solution:** Check `main`, `module`, and `types` fields in package.json

### crates.io Issues

**Problem:** "crate name already taken"  
**Solution:** Choose a different name or contact current owner

**Problem:** "failed to verify uploaded crate"  
**Solution:** Ensure all dependencies are published and versions match

## Automated Publishing with CI/CD

### GitHub Actions Example

Create `.github/workflows/publish.yml`:

```yaml
name: Publish Packages

on:
  push:
    tags:
      - 'v*'

jobs:
  publish-csharp:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-dotnet@v3
      - run: ./build-bindings.sh csharp
      - run: dotnet nuget push bindings/csharp/bin/Release/*.nupkg \
          --api-key ${{ secrets.NUGET_API_KEY }} \
          --source https://api.nuget.org/v3/index.json

  publish-web:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      - run: ./build-bindings.sh web
      - run: |
          cd bindings/npm
          echo "//registry.npmjs.org/:_authToken=${{ secrets.NPM_TOKEN }}" > .npmrc
          for pkg in packages/common packages/webcore packages/bundler packages/node packages/vanilla; do
            (cd "$pkg" && npm publish --access public)
          done

  publish-react-native:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: ./build-bindings.sh react-native
      - run: |
          cd bindings/npm
          echo "//registry.npmjs.org/:_authToken=${{ secrets.NPM_TOKEN }}" > .npmrc
          cd packages/react-native
          npm publish --access public
```

## Version Release Checklist

- [ ] Update version in all package files
- [ ] Update CHANGELOG.md
- [ ] Run all tests
- [ ] Build all bindings
- [ ] Test packages locally
- [ ] Commit and push changes
- [ ] Create and push Git tag
- [ ] Publish C# to NuGet
- [ ] Publish Web to npm
- [ ] Publish React Native to npm
- [ ] Publish Rust crate to crates.io
- [ ] Create GitHub release
- [ ] Update documentation
- [ ] Announce release

## Support

For publishing issues:
- NuGet: https://docs.microsoft.com/en-us/nuget/
- npm: https://docs.npmjs.com/
- crates.io: https://doc.rust-lang.org/cargo/reference/publishing.html
