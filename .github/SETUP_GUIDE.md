# GitHub CI/CD Setup Guide

Complete guide to setting up automated builds, tests, and publishing for json-eval-rs.

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Repository Configuration](#repository-configuration)
3. [Setting Up Secrets](#setting-up-secrets)
4. [Workflow Configuration](#workflow-configuration)
5. [Testing the Setup](#testing-the-setup)
6. [First Release](#first-release)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Accounts

1. **GitHub Account** - Repository hosting
   - Create at: https://github.com/join

2. **NuGet Account** - For C# package publishing
   - Create at: https://www.nuget.org/users/account/LogOn

3. **npm Account** - For Web and React Native packages
   - Create at: https://www.npmjs.com/signup

4. **crates.io Account** - For Rust crate publishing
   - Login at: https://crates.io/ (uses GitHub authentication)

### Required Tools (for local testing)

- Rust 1.70+ (`rustup`, `cargo`)
- .NET SDK 6.0+
- Node.js 18+
- wasm-pack (for WASM builds)

---

## Repository Configuration

### 1. Enable GitHub Actions

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Actions** → **General**
3. Under "Actions permissions", select:
   - ✅ **Allow all actions and reusable workflows**
4. Under "Workflow permissions", select:
   - ✅ **Read and write permissions**
   - ✅ **Allow GitHub Actions to create and approve pull requests**
5. Click **Save**

### 2. Configure Branch Protection (Optional but Recommended)

1. Go to **Settings** → **Branches**
2. Click **Add rule**
3. Branch name pattern: `main`
4. Enable:
   - ✅ **Require a pull request before merging**
   - ✅ **Require status checks to pass before merging**
   - Select status checks: `CI`, `Test`, `Format Check`, `Clippy Lints`
   - ✅ **Require branches to be up to date before merging**
5. Click **Create**

### 3. Enable Issue Templates

Create `.github/ISSUE_TEMPLATE/bug_report.md`:

```yaml
---
name: Bug report
about: Create a report to help us improve
title: '[BUG] '
labels: bug
assignees: ''
---

**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior.

**Expected behavior**
What you expected to happen.

**Environment:**
- OS: [e.g., Ubuntu 22.04, Windows 11, macOS 13]
- Version: [e.g., 0.0.1]
- Binding: [e.g., Rust, C#, Web, React Native]
```

---

## Setting Up Secrets

### 1. NuGet API Key

**Step 1: Generate API Key**
1. Login to [NuGet.org](https://www.nuget.org/)
2. Click your username → **API Keys**
3. Click **Create**
   - Key name: `GitHub Actions`
   - Select scopes: **Push new packages and package versions**
   - Select packages: **All packages**
   - Glob pattern: `JsonEvalRs*`
   - Expiration: 365 days (recommended)
4. Click **Create**
5. **Copy the key** (you won't see it again!)

**Step 2: Add to GitHub**
1. Go to your GitHub repository
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Name: `NUGET_API_KEY`
5. Secret: Paste your NuGet API key
6. Click **Add secret**

### 2. npm Authentication Token

**Step 1: Generate Token**
1. Login to [npmjs.com](https://www.npmjs.com/)
2. Click your profile icon → **Access Tokens**
3. Click **Generate New Token** → **Automation**
4. Token name: `GitHub Actions`
5. Click **Generate Token**
6. **Copy the token**

**Step 2: Add to GitHub**
1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Click **New repository secret**
3. Name: `NPM_TOKEN`
4. Secret: Paste your npm token
5. Click **Add secret**

### 3. crates.io API Token

**Step 1: Generate Token**
1. Login to [crates.io](https://crates.io/) with GitHub
2. Go to **Account Settings**
3. Scroll to **API Tokens**
4. Click **New Token**
5. Token name: `GitHub Actions`
6. Click **Create**
7. **Copy the token**

**Step 2: Add to GitHub**
1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Click **New repository secret**
3. Name: `CARGO_REGISTRY_TOKEN`
4. Secret: Paste your crates.io token
5. Click **Add secret**

### 4. Verify All Secrets

After adding all secrets, you should see:

```
✅ NUGET_API_KEY
✅ NPM_TOKEN
✅ CARGO_REGISTRY_TOKEN
```

Note: `GITHUB_TOKEN` is automatically provided by GitHub Actions.

---

## Workflow Configuration

### 1. Verify Workflow Files

Ensure these files exist in `.github/workflows/`:

```
.github/workflows/
├── ci.yml                  # Continuous Integration
├── build-bindings.yml      # Build all bindings
├── publish.yml             # Publish to registries
└── README.md              # Workflow documentation
```

### 2. Update Package Metadata

Before first release, update these files:

**Cargo.toml:**
```toml
[package]
name = "json-eval-rs"
version = "0.0.1"
authors = ["Muhamad Rizki <hello@byrizki.com>"]
edition = "2021"
description = "High-performance JSON Logic evaluator with schema validation"
license = "MIT"
repository = "https://github.com/byrizki/json-eval-rs"
documentation = "https://docs.rs/json-eval-rs"
homepage = "https://github.com/byrizki/json-eval-rs"
keywords = ["json", "logic", "schema", "validation", "evaluation"]
categories = ["parser-implementations", "data-structures"]
```

**bindings/csharp/JsonEvalRs.csproj:**
```xml
<PropertyGroup>
  <Version>0.0.1</Version>
  <Authors>Muhamad Rizki</Authors>
  <Company>Quadrant Synergy International</Company>
  <PackageProjectUrl>https://github.com/byrizki/json-eval-rs</PackageProjectUrl>
  <RepositoryUrl>https://github.com/byrizki/json-eval-rs</RepositoryUrl>
</PropertyGroup>
```

**bindings/web/package.json:**
```json
{
  "name": "@json-eval-rs/web",
  "version": "0.0.1",
  "author": "Muhamad Rizki <hello@byrizki.com>",
  "repository": {
    "type": "git",
    "url": "https://github.com/byrizki/json-eval-rs"
  }
}
```

**bindings/react-native/package.json:**
```json
{
  "name": "@json-eval-rs/react-native",
  "version": "0.0.1",
  "author": "Muhamad Rizki <hello@byrizki.com>",
  "repository": {
    "type": "git",
    "url": "https://github.com/byrizki/json-eval-rs"
  }
}
```

### 3. Add README Badges

Update your main `README.md`:

```markdown
# json-eval-rs

[![CI](https://github.com/byrizki/json-eval-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/byrizki/json-eval-rs/actions/workflows/ci.yml)
[![Build Bindings](https://github.com/byrizki/json-eval-rs/actions/workflows/build-bindings.yml/badge.svg)](https://github.com/byrizki/json-eval-rs/actions/workflows/build-bindings.yml)
[![Crates.io](https://img.shields.io/crates/v/json-eval-rs.svg)](https://crates.io/crates/json-eval-rs)
[![NuGet](https://img.shields.io/nuget/v/JsonEvalRs.svg)](https://www.nuget.org/packages/JsonEvalRs)
[![npm](https://img.shields.io/npm/v/@json-eval-rs/web.svg)](https://www.npmjs.com/package/@json-eval-rs/web)
```

---

## Testing the Setup

### 1. Test CI Workflow

**Push to main branch:**
```bash
git add .
git commit -m "Setup GitHub Actions"
git push origin main
```

**Expected:**
- Go to **Actions** tab
- See "CI" workflow running
- All checks should pass: ✅

### 2. Test Build Workflow

**Create a pull request:**
```bash
git checkout -b test-workflows
git push origin test-workflows
```

Create PR on GitHub.

**Expected:**
- See "Build Bindings" workflow running
- All artifacts should be created
- Download and test artifacts

### 3. Test Manual Build

1. Go to **Actions** tab
2. Select **Build Bindings**
3. Click **Run workflow**
4. Select branch: `main`
5. Click **Run workflow**

**Expected:**
- Workflow runs successfully
- All artifacts available for download

---

## First Release

### Pre-Release Checklist

- [ ] All tests passing locally
- [ ] Documentation updated
- [ ] CHANGELOG.md created
- [ ] Version numbers consistent
- [ ] All secrets configured
- [ ] Package metadata updated

### Step-by-Step Release Process

**1. Update version numbers:**
```bash
# Update all these files to version 0.0.1
vim Cargo.toml
vim bindings/csharp/JsonEvalRs.csproj
vim bindings/web/package.json
vim bindings/react-native/package.json
```

**2. Create CHANGELOG.md:**
```markdown
# Changelog

## [0.0.1] - 2024-01-XX

### Added
- Initial release
- Core JSON Logic evaluation engine
- Schema validation support
- C# NuGet package
- Web WASM package
- React Native package
- Comprehensive test suite
```

**3. Commit and push:**
```bash
git add -A
git commit -m "Release v0.0.1"
git push origin main
```

**4. Create and push tag:**
```bash
git tag -a v0.0.1 -m "Release version 0.0.1"
git push origin v0.0.1
```

**5. Monitor workflow:**
- Go to **Actions** tab
- Watch "Publish Packages" workflow
- Verify all jobs complete successfully

**6. Verify publications:**
- ✅ Check [crates.io](https://crates.io/crates/json-eval-rs)
- ✅ Check [NuGet](https://www.nuget.org/packages/JsonEvalRs)
- ✅ Check [npm web](https://www.npmjs.com/package/@json-eval-rs/web)
- ✅ Check [npm RN](https://www.npmjs.com/package/@json-eval-rs/react-native)
- ✅ Check GitHub Releases

**7. Test installations:**
```bash
# Test each package
cargo add json-eval-rs
dotnet add package JsonEvalRs
npm install @json-eval-rs/web
npm install @json-eval-rs/react-native
```

---

## Troubleshooting

### Workflow Not Running

**Problem:** Pushed code but no workflow appears

**Solutions:**
1. Check if Actions are enabled (Settings → Actions)
2. Verify workflow file is in `.github/workflows/`
3. Check YAML syntax with: `yamllint .github/workflows/*.yml`
4. Check workflow trigger conditions match your push

### Secret Not Working

**Problem:** "Invalid credentials" or "Unauthorized"

**Solutions:**
1. Verify secret name matches exactly (case-sensitive)
2. Check secret value doesn't have extra spaces
3. Regenerate token and update secret
4. Verify token hasn't expired
5. Check token permissions/scopes

### Build Failing

**Problem:** Native library build fails

**Solutions:**
1. Check Rust version in workflow matches local
2. Verify all features are specified correctly
3. Check for platform-specific code issues
4. Review error logs in Actions tab

### Publishing Fails

**Problem:** "Package already exists"

**Solutions:**
1. Increment version number in ALL package files
2. Ensure versions are consistent
3. Check if version was already published

**Problem:** "Cannot authenticate"

**Solutions:**
1. Verify secrets are set correctly
2. Check token hasn't expired
3. Regenerate tokens if needed
4. Verify repository access for tokens

### Artifact Not Found

**Problem:** Artifacts not available in release

**Solutions:**
1. Check all build jobs completed successfully
2. Verify artifact upload steps succeeded
3. Check artifact names match download steps
4. Review workflow logs for errors

---

## Advanced Configuration

### Enable Dependabot

Create `.github/dependabot.yml`:

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
  
  - package-ecosystem: "npm"
    directory: "/bindings/web"
    schedule:
      interval: "weekly"
  
  - package-ecosystem: "npm"
    directory: "/bindings/react-native"
    schedule:
      interval: "weekly"
  
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
```

### Enable Code Scanning

1. Go to **Security** → **Code scanning**
2. Click **Set up code scanning**
3. Choose **CodeQL Analysis**
4. Commit the workflow file

---

## Support

**Issues with setup:**
- Open an issue: https://github.com/byrizki/json-eval-rs/issues
- Check workflow logs in Actions tab
- Review GitHub Actions documentation

**Package-specific issues:**
- NuGet: https://docs.microsoft.com/en-us/nuget/
- npm: https://docs.npmjs.com/
- crates.io: https://doc.rust-lang.org/cargo/

---

## Next Steps

After successful setup:

1. ✅ Configure branch protection rules
2. ✅ Set up Dependabot
3. ✅ Enable code scanning
4. ✅ Add status badges to README
5. ✅ Create contribution guidelines
6. ✅ Set up issue templates
7. ✅ Configure release automation
8. ✅ Monitor download statistics

---

**🎉 Congratulations! Your CI/CD pipeline is now fully configured!**
