# GitHub CI/CD Setup Guide

Setup guide for release, publish, and docs workflows in this repository.

## Prerequisites

Accounts:

- GitHub repository with Actions enabled.
- NuGet account for `JsonEvalRs`.
- npm account for `@json-eval-rs/*` packages.
- crates.io account for `json-eval-rs`.

Local tools for pre-release checks:

- Rust stable toolchain.
- .NET SDK 8.0+.
- Node.js 24.x+ with Corepack.
- wasm-pack for Web/WASM package checks.
- Android NDK and Xcode only when validating mobile artifacts locally.

## Repository settings

1. Enable Actions: repository Settings → Actions → General → allow actions and reusable workflows.
2. Workflow permissions: enable read/write permissions so release workflow can create tags/releases and deploy Pages.
3. Optional branch protection for `main`: require pull request, required checks, and up-to-date branches.
4. Enable GitHub Pages with GitHub Actions as deployment source.

## Required secrets

Add these under Settings → Secrets and variables → Actions:

| Secret | Used by | Purpose |
| --- | --- | --- |
| `NUGET_API_KEY` | `publish.yml` | Push `JsonEvalRs` NuGet package. |
| `NPM_TOKEN` | `publish.yml` | Publish `@json-eval-rs/*` packages. |
| `CARGO_REGISTRY_TOKEN` | `publish.yml` | Publish `json-eval-rs` to crates.io. |

`GITHUB_TOKEN` is automatic.

## Current workflows

```text
.github/workflows/
├── release.yml       # Build release artifacts and attach them to GitHub Release
├── publish.yml       # Publish pre-built release artifacts to registries
├── deploy-docs.yml   # Build docs and deploy GitHub Pages
└── README.md         # Workflow reference
```

## Package metadata to keep in sync

Update versions before release:

```text
Cargo.toml
bindings/csharp/JsonEvalRs.csproj
bindings/npm/package.json
bindings/npm/packages/common/package.json
bindings/npm/packages/webcore/package.json
bindings/npm/packages/bundler/package.json
bindings/npm/packages/node/package.json
bindings/npm/packages/vanilla/package.json
bindings/npm/packages/react-native/package.json
```

## Release flow

1. Update package versions and `CHANGELOG.md`.
2. Run local checks:

```bash
cargo test --lib --bins --examples
cargo test --all-features --lib --bins --examples
```

3. Commit version/changelog changes and push to `main`.
4. `release.yml` creates tag `v<version>` based on `Cargo.toml` and uploads release assets. Existing tags/releases fail; bump `Cargo.toml` before each release.
5. Run `publish.yml` manually after verifying release assets. Use package toggles for selective publishing.

## Manual publish

Use Actions → Publish Packages → Run workflow.

Inputs:

- `release_tag`: release asset source, for example `v0.0.106`.
- package toggles: C#, common npm, web npm, React Native npm, crates.io.

Publish workflow downloads `.nupkg` and `.tgz` assets from the GitHub Release. It does not rebuild packages.

## Docs deploy

Docs deploy when files under `docs/**` or `.github/workflows/deploy-docs.yml` change on `main`.

Local docs smoke check:

```bash
cd docs
corepack enable
yarn install --immutable
yarn nuxt build --extends docus --preset github_pages
```

## Troubleshooting

### Release workflow did not publish packages

Expected. `release.yml` only builds and uploads release assets. Run `publish.yml` manually after checking the release assets.

### Missing release asset

Open the GitHub Release for `v<version>` and confirm expected assets exist:

- `*.nupkg` for NuGet.
- `json-eval-rs-common-*.tgz` for common package.
- Web package `.tgz` files: webcore, bundler, node, vanilla.
- `*react-native*.tgz` for React Native.
- Native archives and `wasm-modules.tar.gz` for distribution/debugging.

### npm duplicate publish

npm rejects duplicate versions. Bump every npm package version before release, or disable npm jobs during manual dispatch.

### crates.io duplicate publish

crates.io rejects duplicate versions. Bump `Cargo.toml` before release, or disable crates.io during manual dispatch.

### React Native validation skipped

RN example validation runs only when latest changelog section or latest commit message contains `[RN]`. Core React Native package build still runs for release artifacts.
