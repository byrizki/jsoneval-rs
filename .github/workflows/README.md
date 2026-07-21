# GitHub Actions Workflows

Current workflows for `json-eval-rs` release, publish, and docs deployment.

## Workflow map

| Workflow | File | Trigger | Responsibility |
| --- | --- | --- | --- |
| Release and Build | `release.yml` | push to `main`, manual dispatch | Build Rust native libraries, Web/WASM npm packages, React Native packages, C# NuGet package, test Rust, attach artifacts to GitHub Release. |
| Publish Packages | `publish.yml` | manual dispatch | Download pre-built release assets and publish NuGet, npm, and crates.io packages. |
| Deploy docs to GitHub Pages | `deploy-docs.yml` | docs changes on `main`, manual dispatch | Build Nuxt/Docus docs from `docs/` and deploy GitHub Pages. |

## Package layout

```text
bindings/
├── csharp/                         # JsonEvalRs NuGet package
└── npm/                            # Yarn workspace
    ├── packages/common/            # @json-eval-rs/common
    ├── packages/webcore/           # @json-eval-rs/webcore
    ├── packages/bundler/           # @json-eval-rs/bundler
    ├── packages/node/              # @json-eval-rs/node
    ├── packages/vanilla/           # @json-eval-rs/vanilla
    └── packages/react-native/      # @json-eval-rs/react-native
```

## Release and Build

`release.yml` creates release tag `v<version>` from `Cargo.toml`, creates GitHub Release, then builds artifacts. Existing tags/releases fail fast; bump `Cargo.toml` before each release.

Main jobs:

- `create-release` — reads `Cargo.toml`, creates tag/release, extracts changelog.
- `check-rn-tests` — enables RN example validation only when latest changelog or commit message contains `[RN]`.
- `build-native` — builds `ffi` native libraries for Linux, Windows, macOS, iOS targets.
- `build-csharp` — downloads native desktop libraries into `bindings/csharp/runtimes/**/native/`, then runs `dotnet pack`.
- `build-web` — builds WASM output for bundler, Node.js, and vanilla browser packages; packs `common`, `webcore`, `bundler`, `node`, and `vanilla` packages.
- `build-android-jni` — builds Android JNI `.so` files for `arm64-v8a`, `armeabi-v7a`, `x86`, and `x86_64`.
- `build-ios-xcframework` — combines iOS static libs into `JsonEvalRs.xcframework`.
- `build-react-native` — bundles Android JNI libs and iOS XCFramework, builds RN TypeScript output, then packs npm package.
- `test` — runs Rust lib/bin/example tests plus every tracked integration suite, with default features and all features.
- `test-react-native-android` / `test-react-native-ios` — optional RN example validation gated by `[RN]`.
- `upload-to-release` — uploads `.tar.gz`, `.tgz`, and `.nupkg` assets to GitHub Release.

## Publish Packages

`publish.yml` publishes only from release assets. It does not rebuild packages.

Manual dispatch inputs:

- `release_tag` — release to publish from, for example `v0.0.110`.
- `publish_csharp` — NuGet package.
- `publish_common` — `@json-eval-rs/common`.
- `publish_web` — `@json-eval-rs/webcore`, `@json-eval-rs/bundler`, `@json-eval-rs/node`, `@json-eval-rs/vanilla`.
- `publish_react_native` — `@json-eval-rs/react-native`.
- `publish_crates_io` — Rust crate.

Required secrets:

- `NUGET_API_KEY`
- `NPM_TOKEN`
- `CARGO_REGISTRY_TOKEN`

`GITHUB_TOKEN` is provided by GitHub Actions.

## Docs deploy

`deploy-docs.yml` runs in `docs/` with Node 24.x/Corepack/Yarn and uploads `docs/.output/public` to GitHub Pages.

## Local verification before workflow changes

```bash
# YAML syntax
python - <<'PY'
import pathlib, yaml
for path in pathlib.Path('.github/workflows').glob('*.yml'):
    yaml.safe_load(path.read_text())
    print(f'OK {path}')
PY

# Rust checks mirroring workflow test job
cargo test --lib --bins --examples
cargo test --all-features --lib --bins --examples
python - <<'PY'
import pathlib, subprocess
files = subprocess.check_output(['git', 'ls-files', 'tests/*.rs'], text=True).splitlines()
for path in files:
    if path != 'tests/common/mod.rs':
        subprocess.check_call(['cargo', 'test', '--test', pathlib.Path(path).stem])
PY

# Stale workflow references
! grep -R -E 'build-bindings\.yml|pages\.yml' .github/workflows README.md .github/SETUP_GUIDE.md
```
