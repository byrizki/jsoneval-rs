/**
 * @json-eval-rs/common
 *
 * Shared TypeScript types, interfaces, enums, and utilities
 * for @json-eval-rs Web and React Native bindings.
 */

// Re-export all types
export {
  ReturnFormat,
  SchemaValueItem,
  ValidationError,
  ValidationResult,
  DependentChange,
  JSONEvalOptions,
  EvaluateOptions,
  ValidateOptions,
  ValidatePathsOptions,
  EvaluateDependentsOptions,
  GetEvaluatedSchemaOptions,
  GetValueByPathOptions,
  GetValueByPathsOptions,
  GetSchemaByPathOptions,
  GetSchemaByPathsOptions,
  ReloadSchemaOptions,
  CompileAndRunLogicOptions,
  EvaluateSubformOptions,
  ValidateSubformOptions,
  EvaluateDependentsSubformOptions,
  ResolveLayoutSubformOptions,
  GetEvaluatedSchemaSubformOptions,
  GetSchemaValueSubformOptions,
  GetEvaluatedSchemaByPathSubformOptions,
  GetEvaluatedSchemaByPathsSubformOptions,
  GetSchemaByPathSubformOptions,
  GetSchemaByPathsSubformOptions,
  LayoutOverlayEntry,
} from './types.js';

// Re-export utilities
export { stringifyValue, parseValue, stringifyOrNull, extractErrorMessage, mergeLayoutOverlay, resolveEvaluatedLayout } from './utils.js';
