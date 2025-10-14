/**
 * High-performance JSON Logic evaluator with schema validation
 * @packageDocumentation
 */

/**
 * Validation error for a specific field
 */
export interface ValidationError {
  /** Field path with the error */
  readonly path: string;
  /** Type of validation rule that failed */
  readonly rule_type: string;
  /** Error message */
  readonly message: string;
}

/**
 * Result of validation operation
 */
export interface ValidationResult {
  /** Whether any validation errors occurred */
  readonly has_error: boolean;
  /** Array of validation errors */
  readonly errors: ValidationError[];
  /** Convert to JSON object */
  toJSON(): object;
}

/**
 * Options for creating a JSONEval instance
 */
export interface JSONEvalOptions {
  /** JSON schema string */
  schema: string;
  /** Optional context data (JSON string) */
  context?: string;
  /** Optional initial data (JSON string) */
  data?: string;
}

/**
 * Options for evaluation
 */
export interface EvaluateOptions {
  /** JSON data string */
  data: string;
  /** Optional context data (JSON string) */
  context?: string;
}

/**
 * Options for validation with path filtering
 */
export interface ValidatePathsOptions {
  /** JSON data string */
  data: string;
  /** Optional context data (JSON string) */
  context?: string;
  /** Optional array of paths to validate (if not provided, validates all) */
  paths?: string[];
}

/**
 * Cache statistics
 */
export interface CacheStats {
  /** Number of cache hits */
  readonly hits: number;
  /** Number of cache misses */
  readonly misses: number;
  /** Number of cached entries */
  readonly entries: number;
}

/**
 * Options for evaluating dependents
 */
export interface EvaluateDependentsOptions {
  /** Array of field paths that changed */
  changedPaths: string[];
  /** Updated JSON data string */
  data: string;
  /** Optional context data (JSON string) */
  context?: string;
  /** Whether to recursively follow dependency chains (default: true) */
  nested?: boolean;
}

/**
 * High-performance JSON Logic evaluator with schema validation
 * 
 * @example
 * ```typescript
 * import { JSONEval } from '@json-eval-rs/web';
 * 
 * const schema = JSON.stringify({
 *   type: 'object',
 *   properties: {
 *     user: {
 *       type: 'object',
 *       properties: {
 *         name: {
 *           type: 'string',
 *           rules: {
 *             required: { value: true, message: 'Name is required' }
 *           }
 *         }
 *       }
 *     }
 *   }
 * });
 * 
 * const eval = new JSONEval({ schema });
 * 
 * const data = JSON.stringify({ user: { name: 'John' } });
 * const result = eval.evaluateJS({ data });
 * console.log(result);
 * 
 * const validation = eval.validate({ data });
 * if (validation.has_error) {
 *   console.error('Validation errors:', validation.errors);
 * }
 * 
 * eval.free();
 * ```
 */
export class JSONEval {
  /**
   * Create a new JSONEval instance
   * @param options - Configuration options
   * @throws {Error} If schema is invalid
   */
  constructor(options: JSONEvalOptions);
  
  /**
   * Evaluate schema with provided data
   * Returns JSON string
   * @param options - Evaluation options
   * @returns Evaluated schema as JSON string
   * @throws {Error} If evaluation fails
   */
  evaluate(options: EvaluateOptions): string;
  
  /**
   * Evaluate schema with provided data
   * Returns JavaScript object directly
   * @param options - Evaluation options
   * @returns Evaluated schema as JavaScript object
   * @throws {Error} If evaluation fails
   */
  evaluateJS(options: EvaluateOptions): any;
  
  /**
   * Validate data against schema rules
   * @param options - Validation options
   * @returns ValidationResult with any errors
   * @throws {Error} If validation operation fails
   */
  validate(options: EvaluateOptions): ValidationResult;
  
  /**
   * Re-evaluate fields that depend on changed paths
   * Returns JSON string
   * @param options - Dependent evaluation options
   * @returns Updated evaluated schema as JSON string
   * @throws {Error} If evaluation fails
   */
  evaluateDependents(options: EvaluateDependentsOptions): string;
  
  /**
   * Re-evaluate fields that depend on changed paths
   * Returns JavaScript object directly
   * @param options - Dependent evaluation options
   * @returns Updated evaluated schema as JavaScript object
   * @throws {Error} If evaluation fails
   */
  evaluateDependentsJS(options: EvaluateDependentsOptions): any;
  
  /**
   * Get the evaluated schema with optional layout resolution
   * Returns JSON string
   * @param options - Options for schema retrieval
   * @returns Evaluated schema as JSON string
   */
  getEvaluatedSchema(options?: { skipLayout?: boolean }): string;
  
  /**
   * Get the evaluated schema with optional layout resolution
   * Returns JavaScript object directly
   * @param options - Options for schema retrieval
   * @returns Evaluated schema as JavaScript object
   */
  getEvaluatedSchemaJS(options?: { skipLayout?: boolean }): any;
  
  /**
   * Get all schema values (evaluations ending with .value)
   * @returns Map of path -> value as JavaScript object
   */
  getSchemaValue(): Record<string, any>;
  
  /**
   * Reload schema with new data
   * @param options - Reload options with new schema, context, and data
   * @throws {Error} If reload fails
   */
  reloadSchema(options: JSONEvalOptions): void;
  
  /**
   * Get cache statistics
   * @returns Cache statistics with hits, misses, and entries
   */
  cacheStats(): CacheStats;
  
  /**
   * Clear the evaluation cache
   */
  clearCache(): void;
  
  /**
   * Get the number of cached entries
   * @returns Number of cached entries
   */
  cacheLen(): number;
  
  /**
   * Validate data against schema rules with optional path filtering
   * @param options - Validation options with optional path filtering
   * @returns ValidationResult with any errors
   * @throws {Error} If validation operation fails
   */
  validatePaths(options: ValidatePathsOptions): ValidationResult;
  
  /**
   * Free the underlying WebAssembly resources
   * Must be called when done using the instance
   */
  free(): void;
}

/**
 * Get the library version
 * @returns Version string
 */
export function version(): string;

/**
 * Initialize the library
 * Called automatically on import
 */
export function init(): void;

/**
 * Default export for convenience
 */
export default JSONEval;
