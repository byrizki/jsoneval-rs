# GitHub Actions Workflows

This directory contains automated CI/CD workflows for building, testing, and publishing the json-eval-rs project.

## 📋 Available Workflows

### 1. `build-bindings.yml` - Build All Bindings

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`
- Release creation
- Manual dispatch

**Jobs:**
- ✅ **build-native**: Builds native libraries for Linux, Windows, macOS (x64 + ARM64), iOS (device + simulator)
- ✅ **build-android-jni**: Builds Android JNI libraries for arm64-v8a, armeabi-v7a, x86, x86_64
- ✅ **build-csharp**: Creates C# NuGet package with all native libraries
- ✅ **build-web**: Builds WASM modules for web, bundler, and Node.js
- ✅ **build-react-native**: Creates React Native npm package with Android & iOS libraries
- ✅ **test**: Runs full test suite
- ✅ **create-release**: Packages all artifacts for GitHub release (on release tags)
- ✅ **summary**: Generates build summary

**Artifacts Generated:**

**Desktop:**
- `native-x86_64-unknown-linux-gnu` - Linux x64 library (.so)
- `native-x86_64-pc-windows-msvc` - Windows x64 library (.dll)
- `native-x86_64-apple-darwin` - macOS x64 library (.dylib)
- `native-aarch64-apple-darwin` - macOS ARM64 library (.dylib)

**iOS:**
- `native-aarch64-apple-ios` - iOS device library (.a)
- `native-x86_64-apple-ios` - iOS simulator library (.a)

**Android:**
- `android-jni-arm64-v8a` - Android ARM64 library
- `android-jni-armeabi-v7a` - Android ARMv7 library
- `android-jni-x86` - Android x86 library
- `android-jni-x86_64` - Android x86_64 library

**Packages:**
- `nuget-package` - C# NuGet package (.nupkg)
- `web-package` - Web npm package (.tgz)
- `wasm-modules` - WASM modules for all targets
- `react-native-package` - React Native npm package with all mobile libraries (.tgz)

**Usage:**
```bash
# Automatically runs on push/PR
git push origin main

# Manual trigger
# Go to Actions tab → Build Bindings → Run workflow
```

---

### 2. `publish.yml` - Publish Packages

**Triggers:**
- Push tags matching `v*.*.*` (e.g., v0.0.2)
- Manual dispatch with selective publishing

**Jobs:**
- 🔧 **build-native**: Builds desktop and iOS libraries
- 🤖 **build-android-jni**: Builds Android JNI libraries for all architectures
- 📦 **publish-csharp**: Publishes to NuGet.org
- 📦 **publish-web**: Publishes to npm registry
- 📦 **publish-react-native**: Publishes to npm registry (includes Android & iOS libraries)
- 📦 **publish-crates-io**: Publishes to crates.io
- 🎉 **create-github-release**: Creates GitHub release with all artifacts
- 📊 **publish-summary**: Generates publish summary

**Required Secrets:**
- `NUGET_API_KEY` - NuGet.org API key
- `NPM_TOKEN` - npm authentication token
- `CARGO_REGISTRY_TOKEN` - crates.io API token
- `GITHUB_TOKEN` - Automatically provided by GitHub

**Usage:**

**Publish all packages (recommended):**
```bash
# 1. Update version in all package files
#    - Cargo.toml
#    - bindings/csharp/JsonEvalRs.csproj
#    - bindings/web/package.json
#    - bindings/react-native/package.json

# 2. Create and push tag
git tag v0.0.2
git push origin v0.0.2

# Workflow automatically publishes all packages
```

**Manual selective publishing:**
```bash
# Go to Actions tab → Publish Packages → Run workflow
# Select which packages to publish via checkboxes
```

---

## 🔐 Setting Up Secrets

Before publishing, configure these secrets in your GitHub repository:

### 1. NuGet API Key

1. Create account at [nuget.org](https://www.nuget.org/)
2. Generate API key: Account → API Keys → Create
3. Add to GitHub: Settings → Secrets → Actions → New secret
   - Name: `NUGET_API_KEY`
   - Value: Your NuGet API key

### 2. npm Token

1. Create account at [npmjs.com](https://www.npmjs.com/)
2. Generate token: Account → Access Tokens → Generate New Token → Automation
3. Add to GitHub: Settings → Secrets → Actions → New secret
   - Name: `NPM_TOKEN`
   - Value: Your npm token

### 3. crates.io Token

1. Login to [crates.io](https://crates.io/) with GitHub
2. Get token: Account Settings → API Tokens → New Token
3. Add to GitHub: Settings → Secrets → Actions → New secret
   - Name: `CARGO_REGISTRY_TOKEN`
   - Value: Your crates.io token

---

## 📦 Artifacts and Caching

### Artifacts Retention
- Build artifacts are kept for **90 days**
- Release artifacts are kept permanently (attached to releases)

### Caching Strategy
- **Cargo registry**: Cached per OS and target
- **Cargo build**: Cached per OS, target, and Cargo.lock hash
- **npm modules**: Cached in Node.js setup action

---

## 🚀 Release Process

### Complete Release Checklist

1. **Prepare Release**
   ```bash
   # Update version numbers
   vim Cargo.toml
   vim bindings/csharp/JsonEvalRs.csproj
   vim bindings/web/package.json
   vim bindings/react-native/package.json
   
   # Update CHANGELOG.md
   vim CHANGELOG.md
   
   # Commit changes
   git add -A
   git commit -m "Release v0.0.2"
   git push origin main
   ```

2. **Run Tests Locally**
   ```bash
   cargo test --all-features
   cargo test --release
   ```

3. **Create Release Tag**
   ```bash
   git tag -a v0.0.2 -m "Release version 0.0.2"
   git push origin v0.0.2
   ```

4. **Monitor Workflow**
   - Go to Actions tab
   - Watch "Publish Packages" workflow
   - Verify all jobs succeed

5. **Verify Publications**
   - NuGet: https://www.nuget.org/packages/JsonEvalRs
   - npm (web): https://www.npmjs.com/package/@json-eval-rs/web
   - npm (RN): https://www.npmjs.com/package/@json-eval-rs/react-native
   - crates.io: https://crates.io/crates/json-eval-rs
   - GitHub: https://github.com/YOUR_USERNAME/json-eval-rs/releases

6. **Test Installations**
   ```bash
   # Rust
   cargo install json-eval-rs
   
   # C#
   dotnet new console -n test-csharp
   cd test-csharp
   dotnet add package JsonEvalRs
   
   # Web
   npm install @json-eval-rs/web
   
   # React Native
   npm install @json-eval-rs/react-native
   ```

---

## 🛠️ Troubleshooting

### Build Failures

**Issue: Native library build fails**
```
Solution: Check Rust toolchain and target installation
- Verify target is installed: rustup target list
- Install missing target: rustup target add <target>
```

**Issue: WASM build fails**
```
Solution: Install wasm-pack and wasm32 target
- cargo install wasm-pack
- rustup target add wasm32-unknown-unknown
```

### Publishing Failures

**Issue: NuGet "Package already exists"**
```
Solution: Increment version number
- NuGet doesn't allow overwriting published versions
- Update version in JsonEvalRs.csproj
```

**Issue: npm "Cannot publish over existing version"**
```
Solution: Update version in package.json
- npm doesn't allow republishing same version
- Use npm version patch/minor/major
```

**Issue: crates.io "crate name already taken"**
```
Solution: Choose different name or contact owner
- Check availability: cargo search json-eval-rs
- Contact owner if abandoned
```

**Issue: Authentication failed**
```
Solution: Verify secrets are set correctly
- Check secret names match exactly
- Regenerate tokens if expired
- Ensure tokens have correct permissions
```

### Artifact Issues

**Issue: Artifacts not found in release**
```
Solution: Check workflow completed successfully
- Verify all build jobs succeeded
- Check artifact upload steps didn't fail
- Ensure release was created properly
```

---

## 📊 Workflow Status Badges

Add these badges to your README.md:

```markdown
[![Build Bindings](https://github.com/YOUR_USERNAME/json-eval-rs/actions/workflows/build-bindings.yml/badge.svg)](https://github.com/YOUR_USERNAME/json-eval-rs/actions/workflows/build-bindings.yml)

[![Publish Packages](https://github.com/YOUR_USERNAME/json-eval-rs/actions/workflows/publish.yml/badge.svg)](https://github.com/YOUR_USERNAME/json-eval-rs/actions/workflows/publish.yml)
```

---

## 🔄 Manual Workflow Triggers

### Build Bindings Manually
1. Go to Actions tab
2. Select "Build Bindings"
3. Click "Run workflow"
4. Select branch
5. Click "Run workflow"

### Publish Packages Selectively
1. Go to Actions tab
2. Select "Publish Packages"
3. Click "Run workflow"
4. Select branch
5. Check which packages to publish:
   - ☑️ Publish C# NuGet package
   - ☑️ Publish Web npm package
   - ☑️ Publish React Native npm package
   - ☑️ Publish Rust crate to crates.io
6. Click "Run workflow"

---

## 📈 Performance Optimization

### Cache Hit Rates
- Cargo registry: ~90% hit rate
- Cargo build: ~80% hit rate (varies with code changes)
- Node modules: ~95% hit rate

### Build Times (Approximate)
- Native libraries (per platform): 5-10 minutes
- Android JNI (4 architectures, parallel): 8-12 minutes
- iOS libraries: 6-8 minutes
- WASM modules: 8-12 minutes
- C# package: 2-3 minutes
- npm packages: 1-2 minutes each
- **Total pipeline**: 20-30 minutes

### Optimization Tips
1. Use matrix builds for parallel platform builds
2. Cache cargo registry and build directories
3. Only run expensive jobs when necessary
4. Use conditional job execution
5. Reuse artifacts between jobs

---

## 🔒 Security Best Practices

1. **Never commit secrets** - Always use GitHub Secrets
2. **Rotate tokens periodically** - Update tokens every 6-12 months
3. **Use minimal permissions** - Grant only necessary scopes
4. **Review workflow logs** - Check for exposed sensitive data
5. **Pin action versions** - Use specific versions, not `@main`
6. **Enable Dependabot** - Keep actions up to date
7. **Use OIDC tokens** - Consider GitHub OIDC for cloud deployments

---

## 📞 Support

For issues with workflows:
1. Check workflow logs in Actions tab
2. Review this documentation
3. Open an issue on GitHub
4. Check GitHub Actions documentation

---

## 📝 License

These workflows are part of the json-eval-rs project and share the same license.
