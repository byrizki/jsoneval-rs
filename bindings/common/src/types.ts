/**
 * @json-eval-rs/common
 * Shared types and interfaces for Web and React Native bindings.
 *
 * These types match the native Rust output format exactly.
 * Field names use snake_case where Rust serializes snake_case.
 */

// ============================================================================
// Enums
// ============================================================================

/**
 * Return format for path-based methods
 */
export enum ReturnFormat {
  /** Nested object preserving the path hierarchy (default) */
  Nested = 0,
  /** Flat object with dotted keys */
  Flat = 1,
  /** Array of values in the order of requested paths */
  Array = 2,
}

// ============================================================================
// Core data types
// ============================================================================

/**
 * Item for getSchemaValueArray results
 */
export interface SchemaValueItem {
  /** Dotted path (e.g., "field1.field2") */
  path: string;
  /** Value at this path */
  value: any;
}

/**
 * Validation error for a specific field
 */
export interface ValidationError {
  /** Field path with the error */
  path: string;
  /** Type of validation rule that failed (e.g., 'required', 'min', 'max', 'pattern') */
  type: string;
  /** Error message */
  message: string;
  /** Optional error code */
  code?: string;
  /** Optional regex pattern (for pattern validation errors) */
  pattern?: string;
  /** Optional field value that failed validation (as string) */
  fieldValue?: string;
  /** Optional additional data context for the error */
  data?: any;
}

/**
 * Result of a validation operation.
 *
 * NOTE: `has_error` uses snake_case to match the native Rust output.
 */
export interface ValidationResult {
  /** Whether any validation errors occurred */
  has_error: boolean;
  /** Map of validation errors keyed by field path */
  error: Record<string, ValidationError>;
}

/**
 * Dependent field change from evaluateDependents
 */
export interface DependentChange {
  /** Path of the dependent field (in dot notation) */
  $ref: string;
  /** Schema definition of the changed field */
  $field?: any;
  /** Schema definition of the parent field */
  $parentField: any;
  /** Whether this is a transitive dependency */
  transitive: boolean;
  /** If true, the field was cleared */
  clear?: boolean;
  /** New value of the field (if changed) */
  value?: any;
}

// ============================================================================
// Options interfaces
// ============================================================================

/**
 * Options for creating a JSONEval instance
 */
export interface JSONEvalOptions {
  /**
   * JSON schema object or MessagePack binary or cache key string.
   * - If object: Standard JSON Schema
   * - If Uint8Array: MessagePack encoded schema
   * - If string (and fromCache=true): Cache key for pre-parsed schema
   */
  schema: any;
  /**
   * Optional context data accessible via $context in logic.
   */
  context?: any;
  /**
   * Optional initial data object to evaluate against.
   */
  data?: any;
  /**
   * If true, the `schema` parameter is treated as a string cache key.
   * Default: false
   */
  fromCache?: boolean;
}

/**
 * Options for evaluation
 */
export interface EvaluateOptions {
  /** JSON data to evaluate */
  data: any;
  /** Optional context data */
  context?: any;
  /** Optional array of paths for selective evaluation */
  paths?: string[];
}

/**
 * Options for validation
 */
export interface ValidateOptions {
  /** JSON data to validate */
  data: any;
  /** Optional context data */
  context?: any;
}

/**
 * Options for validation with path filtering
 */
export interface ValidatePathsOptions {
  /** JSON data to validate */
  data: any;
  /** Optional context data */
  context?: any;
  /** Optional array of paths to validate */
  paths?: string[];
}

/**
 * Options for evaluating dependents
 */
export interface EvaluateDependentsOptions {
  /** Array of field paths that changed */
  changedPaths: string[];
  /** Updated JSON data */
  data?: any;
  /** Optional context data */
  context?: any;
  /** If true, performs full evaluation after processing dependents */
  reEvaluate?: boolean;
  /** If true, also evaluates dependents in all registered subforms (default: true) */
  includeSubforms?: boolean;
}

/**
 * Options for getting evaluated schema
 */
export interface GetEvaluatedSchemaOptions {
  /** Whether to skip layout resolution */
  skipLayout?: boolean;
}

/**
 * Options for getting a value by path from evaluated schema
 */
export interface GetValueByPathOptions {
  /** Dotted path to the value */
  path: string;
  /** Whether to skip layout resolution */
  skipLayout?: boolean;
}

/**
 * Options for getting values by multiple paths from evaluated schema
 */
export interface GetValueByPathsOptions {
  /** Array of dotted paths to retrieve */
  paths: string[];
  /** Whether to skip layout resolution */
  skipLayout?: boolean;
  /** Return format (Nested, Flat, or Array) */
  format?: ReturnFormat;
}

/**
 * Options for getting a value by path from schema
 */
export interface GetSchemaByPathOptions {
  /** Dotted path to the value */
  path: string;
}

/**
 * Options for getting values by multiple paths from schema
 */
export interface GetSchemaByPathsOptions {
  /** Array of dotted paths to retrieve */
  paths: string[];
  /** Return format (Nested, Flat, or Array) */
  format?: ReturnFormat;
}

/**
 * Options for reloading schema
 */
export interface ReloadSchemaOptions {
  /** New JSON schema */
  schema: any;
  /** Optional new context */
  context?: any;
  /** Optional new data */
  data?: any;
}

/**
 * Options for compiling and running logic
 */
export interface CompileAndRunLogicOptions {
  /** Logic expression as string or object */
  logicStr: string | object;
  /** Optional data context */
  data?: any;
  /** Optional context data */
  context?: any;
}

// ============================================================================
// Subform options interfaces
// ============================================================================

/**
 * Options for evaluating a subform
 */
export interface EvaluateSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** JSON data to evaluate */
  data: any;
  /** Optional context data */
  context?: any;
  /** Optional array of paths to evaluate */
  paths?: string[];
}

/**
 * Options for validating a subform
 */
export interface ValidateSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** JSON data to validate */
  data: any;
  /** Optional context data */
  context?: any;
}

/**
 * Options for evaluating dependents in a subform
 */
export interface EvaluateDependentsSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** Array of field paths that changed */
  changedPaths: string[];
  /** Updated JSON data */
  data?: any;
  /** Optional context data */
  context?: any;
  /** If true, performs full evaluation after processing dependents */
  reEvaluate?: boolean;
  /** If true, also evaluates dependents in sub-subforms (default: true) */
  includeSubforms?: boolean;
}

/**
 * Options for resolving layout in a subform
 */
export interface ResolveLayoutSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** Whether to evaluate after resolving layout */
  evaluate?: boolean;
}

/**
 * Options for getting evaluated schema from a subform
 */
export interface GetEvaluatedSchemaSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** Whether to resolve layout */
  resolveLayout?: boolean;
}

/**
 * Options for getting schema value from a subform
 */
export interface GetSchemaValueSubformOptions {
  /** Path to the subform */
  subformPath: string;
}

/**
 * Options for getting evaluated schema by path from a subform
 */
export interface GetEvaluatedSchemaByPathSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** Dotted path to the value within the subform */
  schemaPath: string;
  /** Whether to skip layout resolution */
  skipLayout?: boolean;
}

/**
 * Options for getting evaluated schema by multiple paths from a subform
 */
export interface GetEvaluatedSchemaByPathsSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** Array of dotted paths to retrieve within the subform */
  schemaPaths: string[];
  /** Whether to skip layout resolution */
  skipLayout?: boolean;
  /** Return format (Nested, Flat, or Array) */
  format?: ReturnFormat;
}

/**
 * Options for getting schema by path from a subform
 */
export interface GetSchemaByPathSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** Dotted path to the value within the subform */
  schemaPath: string;
}

/**
 * Options for getting schema by multiple paths from a subform
 */
export interface GetSchemaByPathsSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** Array of dotted paths to retrieve within the subform */
  schemaPaths: string[];
  /** Return format (Nested, Flat, or Array) */
  format?: ReturnFormat;
}
