/**
 * @json-eval-rs/webcore - TypeScript definitions
 */

/**
 * Get the library version from the WASM module
 * @param wasmModule - WASM module
 * @returns Version string
 */
export function getVersion(wasmModule: any): string;

/**
 * Return format for path-based methods
 */
export enum ReturnFormat {
  /** Nested object preserving the path hierarchy (default) */
  Nested = 0,
  /** Flat object with dotted keys */
  Flat = 1,
  /** Array of values in the order of requested paths */
  Array = 2
}

/**
 * Validation error for a specific field
 */
export interface ValidationError {
  /** Field path with the error */
  path: string;
  /** Type of validation rule that failed (e.g., 'required', 'min', 'max', 'pattern') */
  rule_type: string;
  /** Error message */
  message: string;
  /** Optional error code */
  code?: string;
  /** Optional regex pattern (for pattern validation errors) */
  pattern?: string;
  /** Optional field value that failed validation (as string) */
  field_value?: string;
  /** Optional additional data context for the error */
  data?: any;
}

/**
 * Result of validation operation
 */
export interface ValidationResult {
  /** Whether any validation errors occurred */
  has_error: boolean;
  /** Array of validation errors */
  errors: ValidationError[];
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

/**
 * Options for creating a JSONEval instance
 */
export interface JSONEvalOptions {
  /** JSON schema object */
  schema: any;
  /** Optional context data */
  context?: any;
  /** Optional initial data */
  data?: any;
  /** WASM module instance */
  wasmModule?: any;
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
 * Options for evaluation
 */
export interface EvaluateOptions {
  /** JSON data to evaluate */
  data: any;
  /** Optional context data */
  context?: any;
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
 * Cache statistics
 */
export interface CacheStats {
  /** Number of cache hits */
  hits: number;
  /** Number of cache misses */
  misses: number;
  /** Number of cached entries */
  entries: number;
}

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

export class JSONEval {
  constructor(options: JSONEvalOptions);
  static fromCache(cacheKey: string, context?: any, data?: any): JSONEval;
  init(): Promise<void>;
  validate(options: ValidateOptions): Promise<ValidationResult>;
  evaluate(options: EvaluateOptions): Promise<any>;
  evaluateDependents(options: EvaluateDependentsOptions): Promise<DependentChange[]>;
  compileAndRunLogic(options: CompileAndRunLogicOptions): Promise<any>;
  compileLogic(logicStr: string | object): Promise<number>;
  runLogic(logicId: number, data?: any, context?: any): Promise<any>;
  getEvaluatedSchema(options?: GetEvaluatedSchemaOptions): Promise<any>;
  getSchemaValue(): Promise<any>;
  getEvaluatedSchemaWithoutParams(options?: GetEvaluatedSchemaOptions): Promise<any>;
  getValueByPath(options: GetValueByPathOptions): Promise<any | null>;
  getEvaluatedSchemaByPath(options: GetValueByPathOptions): Promise<any | null>;
  getEvaluatedSchemaByPaths(options: GetValueByPathsOptions): Promise<any>;
  getSchemaByPath(options: GetSchemaByPathOptions): Promise<any | null>;
  getSchemaByPaths(options: GetSchemaByPathsOptions): Promise<any>;
  reloadSchema(options: ReloadSchemaOptions): Promise<void>;
  reloadSchemaMsgpack(schemaMsgpack: Uint8Array, context?: any, data?: any): Promise<void>;
  reloadSchemaFromCache(cacheKey: string, context?: any, data?: any): Promise<void>;
  cacheStats(): Promise<CacheStats>;
  clearCache(): Promise<void>;
  cacheLen(): Promise<number>;
  enableCache(): Promise<void>;
  disableCache(): Promise<void>;
  isCacheEnabled(): boolean;
  
  // Subform methods
  evaluateSubform(options: EvaluateSubformOptions): Promise<void>;
  validateSubform(options: ValidateSubformOptions): Promise<ValidationResult>;
  evaluateDependentsSubform(options: EvaluateDependentsSubformOptions): Promise<DependentChange[]>;
  resolveLayoutSubform(options: ResolveLayoutSubformOptions): Promise<void>;
  getEvaluatedSchemaSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any>;
  getSchemaValueSubform(options: GetSchemaValueSubformOptions): Promise<any>;
  getEvaluatedSchemaWithoutParamsSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any>;
  getEvaluatedSchemaByPathSubform(options: GetEvaluatedSchemaByPathSubformOptions): Promise<any | null>;
  getEvaluatedSchemaByPathsSubform(options: GetEvaluatedSchemaByPathsSubformOptions): Promise<any>;
  getSchemaByPathSubform(options: GetSchemaByPathSubformOptions): Promise<any | null>;
  getSchemaByPathsSubform(options: GetSchemaByPathsSubformOptions): Promise<any>;
  getSubformPaths(): Promise<string[]>;
  hasSubform(subformPath: string): Promise<boolean>;
  
  free(): void;
}

export function version(wasmModule?: any): Promise<string>;

export default JSONEval;
