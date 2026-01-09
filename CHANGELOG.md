# Changelog

## [0.0.48] - 2026-01-09

### Fixed

- [core] fix eval depends

## [0.0.47] - 2026-01-09

### Fixed

- [wasm] fix return value to JS Object
- [core] fix options url templation

## [0.0.46] - 2026-01-08

### Fixed

- [wasm] fix return value to JS Object

## [0.0.45] - 2026-01-08

### Added

- [web] Vanilla package
- [web] Migrate to TypeScript

## [0.0.44] - 2026-01-07

### Fixed

- [core] Re: fix layout sync issue

## [0.0.43] - 2025-01-07

### Changed

- [RN] **Android 15 Compatibility**: Update Android native builds to support 16KB page sizes (required for Android 15+)

## [0.0.42] - 2025-01-07

### Fixed

- [core] **Layout sync cache bug**: Fixed cache purging for dependent evaluations when data changes

## [0.0.41] - 2025-12-29

### Fixed

- [core] Fix array operations with parallel feature

## [0.0.40] - 2025-12-24

### Fixed

- [core] Fix cache purging function

## [0.0.39] - 2025-12-24

### Fixed

- [core] Fix get schema values to exclude $params

## [0.0.38] - 2025-12-24

### Fixed

- [core] Fix schema value evaluations

## [0.0.37] - 2025-12-23

### Fixed

- [RN] Fix RN Android Build

## [0.0.36] - 2025-12-22

### Fixed

- [core] Fix table calculation dependency on optional deps

## [0.0.35] - 2025-12-19

### Added

- [Feature] Add selective paths evaluation for subforms

### Fixed

- [core] Fix selective paths evaluation

## [0.0.34] - 2025-12-19

### Fixed

- [core] Fix C# binding evaluate for selective eval

## [0.0.33] - 2025-12-19

### Fixed

- [RN] Fix Android JSI binding

### Added

- [Feature] Add set timezone offset for date operations
- [Feature] Add selective paths evaluation

## [0.0.32] - 2025-12-12

### Fixed

- [RN] Fix Android build - moved template function outside `extern "C"` block (templates must have C++ linkage)

## [0.0.31] - 2025-11-14

### Fixed

- [validation] Fix validation for disabled field

## [0.0.30] - 2025-11-12

### Fixed

- [core] Fix non mandatory table dependency check

## [0.0.29] - 2025-10-30

### Added

- [core] Implement `format` option on `getSchemaByPaths`, `getEvaluatedSchemaByPaths`

## [0.0.28] - 2025-10-30

### Added

- [core] Implement `getSchemaByPath`, `getSchemaPaths`, `getEvaluatedSchemaByPath`, `getEvaluatedSchemaPaths`
- [bindings] Implement `getSchemaByPath`, `getSchemaPaths`, `getEvaluatedSchemaByPath`, `getEvaluatedSchemaPaths` for C# and FFI
- [bindings] Implement `getEvaluatedSchemaByPath` and `getEvaluatedSchemaPaths` for Web and React Native
- [react-native] Enhance RN build

## [0.0.27] - 2025-10-27

### Changed

- [core] Implement micro optimization

### Fixed

- [react-native] Fix Android build - moved template function outside `extern "C"` block (templates must have C++ linkage)

## [0.0.26] - 2025-10-25

### Changed

- [core] Optimize parallel evaluation

## [0.0.25] - 2025-10-25

### Added

- Comprehensive operator documentation under `docs/`, including quick reference and per-category guides.
- GitHub Pages configuration and workflow to publish documentation from the `docs/` directory.
- Automated release workflow that tags, publishes releases, and triggers binding builds.

### Changed

- README now links directly to the documentation hub.
- Binding build workflow now responds to auto-release completion when uploading artifacts.

## [0.0.24] - 2025-10-24

### Fixed

- [bindings] fix C# & RN FFI binding

## [0.0.23] - 2025-10-24

### Changed

- [core] toggle on/off evaluation cache
- [core] enhance evaluation cache to be smart, purging caches per-changed data key only

## [0.0.22] - 2025-10-24

### Changed

- [core] optimize schema to be immutable and lightly clone, expose getSchemaByPath

### Fixed

- [core] Logic evaluate dependents

## [0.0.21] - 2025-10-24

### Changed

- [core] Refactored `evalDepends` to accept multiple evaluation paths and an option to trigger evaluation automatically from latest data
- [core] Fixed date operations and `evalDepends` path
- [android] Fixed JNI calls in `evaluateDependents`

## [0.0.20] - 2025-10-23

### Added

- [validation] Support for custom evaluation rules with `$evaluation` expressions
  - Rules can have `value.$evaluation` that dynamically evaluates validation logic
  - Supports `message.$evaluation` for dynamic error messages
  - Supports `data[key].$evaluation` for computed error data
  - Supports array format: `"evaluation": [{ "code": "...", "message": "...", "$evaluation": {...} }]`
  - Falsy evaluation results (false, 0, null, empty) trigger validation errors with type "evaluation"

### Changed

- [validation] Enhanced ValidationError structure to match JS version with additional fields:
  - `type` (renamed from `ruleType`, with backwards compatibility in C#)
  - `code` - error code (defaults to `{path}.{ruleName}` if not specified)
  - `pattern` - regex pattern (for pattern rule only)
  - `fieldValue` - actual field value (for pattern rule only)
  - `data` - additional data (for evaluation rules)
- [bindings] Updated C# and FFI bindings to expose new validation error fields

### Performance

- [validation] Major validation optimization: moved heavy operations to schema parse time
  - Fields with rules are collected during schema parsing (one-time cost)
  - Rules with `$evaluation` are evaluated during `evaluate()` so values are available in `evaluated_schema`
  - No tree walking during validation - uses pre-parsed field list
  - Validates only fields that have rules: O(fields_with_rules) vs O(all_fields)
  - Removed 40+ lines of runtime tree-walking code
  - Significant performance improvement for large schemas with few validated fields

## [0.0.19] - 2025-10-23

### Fixed

- [validation] Fix minValue/maxValue validation for schemas without root properties wrapper

## [0.0.18] - 2025-10-23

### Fixed

- [layout] Flag $path, $fullpath on direct layout mapping

## [0.0.17] - 2025-10-23

### Fixed

- [layout] Flag $path, $fullpath, hideLayout on direct layout mapping
- [android] Fix multiple duplicate so files

## [0.0.16] - 2025-10-23

### Fixed

- [layout] Flag $parentHide, $path with dot annotation

## [0.0.15] - 2025-10-23

### Fixed

- Enable Parallel on Native

## [0.0.14] - 2025-10-22

### Fixed

- Fix FFI compile and run logic with context data.

## [0.0.13] - 2025-10-22

### Fixed

- Fix FFI compile and run logic from string.

## [0.0.12] - 2025-10-22

### Added

- Introduced parsed schema cache instantiation path for `JSONEval`, enabling reuse of precompiled logic.
- Added dependency-injected parsed cache support across the C# bindings and reload flows for C#, React Native, and WASM targets.

### Changed

- Refactored FFI and WASM layers to integrate the parsed schema cache pipeline.

### Fixed

- Resolved doctest failures, build warnings, and packaging issues across C#, iOS, Android, and React Native targets.

## [0.0.11] - 2025-10-22

### Added

- Introduced parsed schema cache storage and reload workflows across `JSONEval`.
- Enabled dependency-injected parsed cache support within the C# bindings.
- Added cross-platform cache reload integration for React Native and WASM targets.

### Changed

- Refactored FFI and WASM layers to support cache hydration while reducing duplication.
- Enhanced release pipelines with Linux ARM artifacts and faster packaging steps.

### Fixed

- Resolved packaging and build issues across C#, iOS (XCFramework), Android, and React Native targets.

## [0.0.10] - 2025-10-21

### Added

- Implemented the subform evaluation pipeline.
- Added template string support for options definitions.
- Exposed `get_evaluated_schema_by_path()` and layout resolution helpers.

### Fixed

- Patched C# binding regressions impacting packaging consistency.

## [0.0.9] - 2025-10-20

### Added

- Added MessagePack serialization support for schema and data interchange.

### Fixed

- Corrected sum operator threshold handling and topological sort edge cases.

## [0.0.8] - 2025-10-18

### Changed

- Reverted the C# serializer swap to maintain compatibility with existing bindings.

## [0.0.7] - 2025-10-18

### Added

- Optimized the React Native binding with zero-copy data paths.

### Changed

- Migrated C# bindings to `System.Text.Json` for serialization.

### Fixed

- Stabilized cross-platform build outputs.

## [0.0.6] - 2025-10-18

### Changed

- Improved FFI performance and introduced dedicated C# benchmarks.

## [0.0.5] - 2025-10-17

### Added

- Enabled retrieving schemas without `$params` and accessing evaluated values by path.
- Exposed library version metadata via FFI and C# bindings.
- Added a dedicated C# benchmark suite.

### Fixed

- Resolved evaluation dependency propagation issues and .NET packaging problems.
- Improved comparison tooling and dependency collection accuracy.

## [0.0.3] - 2025-10-16

### Changed

- Updated build pipelines and removed prebuilt artifacts to simplify releases.

### Fixed

- Addressed C# nullable reference warnings, exception constructors, and React Native TypeScript configuration.
- Stabilized FFI builds across targets.

## [0.0.2] - 2025-10-16

### Added

- Added .NET Standard support and Android JNI fixes.

### Fixed

- Patched web binding behaviors and return-operator handling.
- Streamlined CI pipeline and binding packaging flows.

## [0.0.1] - 2025-10-XX

### Added

- Initial release with core evaluation engine
- Multi-platform bindings (Rust, C#, Web, React Native)
- Advanced caching and parallel processing
- Schema validation with detailed error reporting
- CLI tool for testing and benchmarking
- Comprehensive documentation and examples
