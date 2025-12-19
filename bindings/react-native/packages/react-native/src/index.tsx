import React from 'react';
import { NativeModules, Platform } from 'react-native';

const LINKING_ERROR =
  `The package '@json-eval-rs/react-native' doesn't seem to be linked. Make sure: \n\n` +
  Platform.select({ ios: "- You have run 'pod install'\n", default: '' }) +
  '- You rebuilt the app after installing the package\n' +
  '- You are not using Expo managed workflow\n';

const JsonEvalRs = NativeModules.JsonEvalRs
  ? NativeModules.JsonEvalRs
  : new Proxy(
      {},
      {
        get() {
          throw new Error(LINKING_ERROR);
        },
      }
    );

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
  ruleType: string;
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
 * Result of validation operation
 */
export interface ValidationResult {
  /** Whether any validation errors occurred */
  hasError: boolean;
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
  /** JSON schema string or object */
  schema: string | object;
  /** Optional context data (string or object) */
  context?: string | object;
  /** Optional initial data (string or object) */
  data?: string | object;
}

/**
 * Options for evaluation
 */
export interface EvaluateOptions {
  /** JSON data string or object */
  data: string | object;
  /** Optional context data (string or object) */
  context?: string | object;
  /** Optional array of paths for selective evaluation */
  paths?: string[];
}

/**
 * Options for validation with path filtering
 */
export interface ValidatePathsOptions {
  /** JSON data string or object */
  data: string | object;
  /** Optional context data (string or object) */
  context?: string | object;
  /** Optional array of paths to validate (if not provided, validates all) */
  paths?: string[];
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
 * Options for evaluating dependents
 */
export interface EvaluateDependentsOptions {
  /** Array of field paths that changed */
  changedPaths: string[];
  /** Updated JSON data (string or object) */
  data?: string | object;
  /** Optional context data (string or object) */
  context?: string | object;
  /** If true, performs full evaluation after processing dependents */
  reEvaluate?: boolean;
}

/**
 * Options for evaluating a subform
 */
export interface EvaluateSubformOptions {
  /** Path to the subform (e.g., "#/riders") */
  subformPath: string;
  /** JSON data string or object */
  data: string | object;
  /** Optional context data (string or object) */
  context?: string | object;
}

/**
 * Options for validating a subform
 */
export interface ValidateSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** JSON data string or object */
  data: string | object;
  /** Optional context data (string or object) */
  context?: string | object;
}

/**
 * Options for evaluating dependents in a subform
 */
export interface EvaluateDependentsSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** Array of field paths that changed */
  changedPaths: string[];
  /** Optional updated JSON data (string or object) */
  data?: string | object;
  /** Optional context data (string or object) */
  context?: string | object;
  /** If true, performs full evaluation after processing dependents */
  reEvaluate?: boolean;
}

/**
 * Options for resolving layout in a subform
 */
export interface ResolveLayoutSubformOptions {
  /** Path to the subform */
  subformPath: string;
  /** If true, runs evaluation before resolving layout */
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
  /** Array of dotted paths to the values within the subform */
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
  /** Array of dotted paths to the values within the subform */
  schemaPaths: string[];
  /** Return format (Nested, Flat, or Array) */
  format?: ReturnFormat;
}

/**
 * High-performance JSON Logic evaluator with schema validation for React Native
 * 
 * ## Zero-Copy Architecture
 * 
 * This binding is optimized for minimal memory copies:
 * - **Rust FFI Layer**: Returns raw pointers (zero-copy)
 * - **C++ Bridge**: Uses direct pointer access with single-copy string construction
 * - **Native Platform**: Minimizes intermediate conversions
 * - **JS Bridge**: React Native's architecture requires serialization (unavoidable)
 * 
 * While true zero-copy across JS/Native boundary is not possible due to React Native's
 * architecture, we minimize copies within the native layer to maximize performance.
 * 
 * @example
 * ```typescript
 * import { JSONEval } from '@json-eval-rs/react-native';
 * 
 * const schema = {
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
 * };
 * 
 * const eval = new JSONEval({ schema });
 * 
 * const data = { user: { name: 'John' } };
 * const result = await eval.evaluate({ data });
 * console.log(result);
 * 
 * const validation = await eval.validate({ data });
 * if (validation.hasError) {
 *   console.error('Validation errors:', validation.errors);
 * }
 * 
 * await eval.dispose();
 * ```
 */
export class JSONEval {
  private handle: string;
  private disposed: boolean = false;

  /**
   * Creates a new JSON evaluator instance from a cached ParsedSchema
   * @param cacheKey - Cache key to lookup in the global ParsedSchemaCache
   * @param context - Optional context data
   * @param data - Optional initial data
   * @returns New JSONEval instance
   * @throws {Error} If schema not found in cache or creation fails
   */
  static fromCache(
    cacheKey: string,
    context?: string | object | null,
    data?: string | object | null
  ): JSONEval {
    const contextStr = context ? (typeof context === 'string' ? context : JSON.stringify(context)) : null;
    const dataStr = data ? (typeof data === 'string' ? data : JSON.stringify(data)) : null;
    
    const handle = JsonEvalRs.createFromCache(cacheKey, contextStr, dataStr);
    return new JSONEval({ schema: {}, _handle: handle });
  }

  /**
   * Creates a new JSON evaluator instance
   * @param options - Configuration options with schema, context, and data
   * @throws {Error} If creation fails
   */
  constructor(options: JSONEvalOptions & { _handle?: string }) {
    // If handle is provided (from static factory), use it directly
    if (options._handle) {
      this.handle = options._handle;
      return;
    }
    
    const { schema, context, data } = options;
    
    try {
      const schemaStr = typeof schema === 'string' ? schema : JSON.stringify(schema);
      const contextStr = context ? (typeof context === 'string' ? context : JSON.stringify(context)) : null;
      const dataStr = data ? (typeof data === 'string' ? data : JSON.stringify(data)) : null;
      
      this.handle = JsonEvalRs.create(schemaStr, contextStr, dataStr);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to create JSONEval instance: ${errorMessage}`);
    }
  }

  private throwIfDisposed() {
    if (this.disposed) {
      throw new Error('JSONEval instance has been disposed');
    }
  }

  /**
   * Convert value to JSON string
   * Performance note: If you have a pre-serialized JSON string, pass it directly
   * instead of an object to avoid the JSON.stringify overhead
   */
  private toJsonString(value: string | object): string {
    return typeof value === 'string' ? value : JSON.stringify(value);
  }

  /**
   * Evaluate schema with provided data
   * @param options - Evaluation options
   * @returns Promise resolving to evaluated schema object
   * @throws {Error} If evaluation fails
   */
  async evaluate(options: EvaluateOptions): Promise<any> {
    this.throwIfDisposed();
    
    try {
      const dataStr = this.toJsonString(options.data);
      const contextStr = options.context ? this.toJsonString(options.context) : null;
      const pathsJson = options.paths ? JSON.stringify(options.paths) : null;
      
      const resultStr = await JsonEvalRs.evaluate(this.handle, dataStr, contextStr, pathsJson);
      return JSON.parse(resultStr);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Evaluation failed: ${errorMessage}`);
    }
  }

  /**
   * Validate data against schema rules
   * @param options - Validation options
   * @returns Promise resolving to ValidationResult
   * @throws {Error} If validation operation fails
   */
  async validate(options: EvaluateOptions): Promise<ValidationResult> {
    this.throwIfDisposed();
    
    try {
      const dataStr = this.toJsonString(options.data);
      const contextStr = options.context ? this.toJsonString(options.context) : null;
      
      const resultStr = await JsonEvalRs.validate(this.handle, dataStr, contextStr);
      return JSON.parse(resultStr);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Validation failed: ${errorMessage}`);
    }
  }

  /**
   * Re-evaluate fields that depend on a changed path
   * @param options - Dependent evaluation options
   * @returns Promise resolving to array of dependent field changes
   * @throws {Error} If evaluation fails
   */
  async evaluateDependents(options: EvaluateDependentsOptions): Promise<DependentChange[]> {
    this.throwIfDisposed();
    
    try {
      const { changedPaths, data, context, reEvaluate = false } = options;
      const changedPathsJson = JSON.stringify(changedPaths);
      const dataStr = data ? this.toJsonString(data) : null;
      const contextStr = context ? this.toJsonString(context) : null;
      
      const resultStr = await JsonEvalRs.evaluateDependents(
        this.handle,
        changedPathsJson,
        dataStr,
        contextStr,
        reEvaluate
      );
      return JSON.parse(resultStr);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Dependent evaluation failed: ${errorMessage}`);
    }
  }

  /**
   * Get the evaluated schema with optional layout resolution
   * @param skipLayout - Whether to skip layout resolution (default: false)
   * @returns Promise resolving to evaluated schema object
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchema(skipLayout: boolean = false): Promise<any> {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchema(this.handle, skipLayout);
    return JSON.parse(resultStr);
  }

  /**
   * Get all schema values (evaluations ending with .value)
   * @returns Promise resolving to map of path -> value
   * @throws {Error} If operation fails
   */
  async getSchemaValue(): Promise<Record<string, any>> {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaValue(this.handle);
    return JSON.parse(resultStr);
  }

  /**
   * Get the evaluated schema without $params field
   * @param skipLayout - Whether to skip layout resolution (default: false)
   * @returns Promise resolving to evaluated schema object
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaWithoutParams(skipLayout: boolean = false): Promise<any> {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchemaWithoutParams(this.handle, skipLayout);
    return JSON.parse(resultStr);
  }

  /**
   * Get a value from the evaluated schema using dotted path notation
   * @param path - Dotted path to the value (e.g., "properties.field.value")
   * @param skipLayout - Whether to skip layout resolution
   * @returns Promise resolving to the value at the path, or null if not found
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPath(path: string, skipLayout: boolean = false): Promise<any | null> {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchemaByPath(this.handle, path, skipLayout);
    return resultStr ? JSON.parse(resultStr) : null;
  }

  /**
   * Get values from the evaluated schema using multiple dotted path notations
   * Returns data in the specified format (skips paths that are not found)
   * @param paths - Array of dotted paths to retrieve
   * @param skipLayout - Whether to skip layout resolution
   * @param format - Return format (Nested, Flat, or Array)
   * @returns Promise resolving to data in the specified format
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPaths(paths: string[], skipLayout: boolean = false, format: ReturnFormat = ReturnFormat.Nested): Promise<any> {
    this.throwIfDisposed();
    const pathsJson = JSON.stringify(paths);
    const resultStr = await JsonEvalRs.getEvaluatedSchemaByPaths(this.handle, pathsJson, skipLayout, format);
    return JSON.parse(resultStr);
  }

  /**
   * Get a value from the schema using dotted path notation
   * @param path - Dotted path to the value (e.g., "properties.field.value")
   * @returns Promise resolving to the value at the path, or null if not found
   * @throws {Error} If operation fails
   */
  async getSchemaByPath(path: string): Promise<any | null> {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaByPath(this.handle, path);
    return resultStr ? JSON.parse(resultStr) : null;
  }

  /**
   * Get values from the schema using multiple dotted path notations
   * Returns data in the specified format (skips paths that are not found)
   * @param paths - Array of dotted paths to retrieve
   * @param format - Return format (Nested, Flat, or Array)
   * @returns Promise resolving to data in the specified format
   * @throws {Error} If operation fails
   */
  async getSchemaByPaths(paths: string[], format: ReturnFormat = ReturnFormat.Nested): Promise<any> {
    this.throwIfDisposed();
    const pathsJson = JSON.stringify(paths);
    const resultStr = await JsonEvalRs.getSchemaByPaths(this.handle, pathsJson, format);
    return JSON.parse(resultStr);
  }

  /**
   * Reload schema with new data
   * @param options - Configuration options with new schema, context, and data
   * @throws {Error} If reload fails
   */
  async reloadSchema(options: JSONEvalOptions): Promise<void> {
    this.throwIfDisposed();
    
    try {
      const { schema, context, data } = options;
      const schemaStr = typeof schema === 'string' ? schema : JSON.stringify(schema);
      const contextStr = context ? (typeof context === 'string' ? context : JSON.stringify(context)) : null;
      const dataStr = data ? (typeof data === 'string' ? data : JSON.stringify(data)) : null;
      
      await JsonEvalRs.reloadSchema(this.handle, schemaStr, contextStr, dataStr);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to reload schema: ${errorMessage}`);
    }
  }

  /**
   * Reload schema from MessagePack bytes
   * @param schemaMsgpack - MessagePack-encoded schema bytes (Uint8Array or number array)
   * @param context - Optional context data
   * @param data - Optional initial data
   * @throws {Error} If reload fails
   */
  async reloadSchemaMsgpack(
    schemaMsgpack: Uint8Array | number[],
    context?: string | object | null,
    data?: string | object | null
  ): Promise<void> {
    this.throwIfDisposed();
    
    try {
      // Convert Uint8Array to number array if needed
      const msgpackArray = schemaMsgpack instanceof Uint8Array 
        ? Array.from(schemaMsgpack) 
        : schemaMsgpack;
      
      const contextStr = context ? (typeof context === 'string' ? context : JSON.stringify(context)) : null;
      const dataStr = data ? (typeof data === 'string' ? data : JSON.stringify(data)) : null;
      
      await JsonEvalRs.reloadSchemaMsgpack(this.handle, msgpackArray, contextStr, dataStr);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to reload schema from MessagePack: ${errorMessage}`);
    }
  }

  /**
   * Reload schema from ParsedSchemaCache using a cache key
   * @param cacheKey - Cache key to lookup in the global ParsedSchemaCache
   * @param context - Optional context data
   * @param data - Optional initial data
   * @throws {Error} If reload fails or schema not found in cache
   */
  async reloadSchemaFromCache(
    cacheKey: string,
    context?: string | object | null,
    data?: string | object | null
  ): Promise<void> {
    this.throwIfDisposed();
    
    try {
      const contextStr = context ? (typeof context === 'string' ? context : JSON.stringify(context)) : null;
      const dataStr = data ? (typeof data === 'string' ? data : JSON.stringify(data)) : null;
      
      await JsonEvalRs.reloadSchemaFromCache(this.handle, cacheKey, contextStr, dataStr);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to reload schema from cache: ${errorMessage}`);
    }
  }

  /**
   * Get cache statistics
   * @returns Promise resolving to cache statistics
   * @throws {Error} If operation fails
   */
  async cacheStats(): Promise<CacheStats> {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.cacheStats(this.handle);
    return JSON.parse(resultStr);
  }

  /**
   * Clear the evaluation cache
   * @returns Promise that resolves when cache is cleared
   * @throws {Error} If operation fails
   */
  async clearCache(): Promise<void> {
    this.throwIfDisposed();
    await JsonEvalRs.clearCache(this.handle);
  }

  /**
   * Get the number of cached entries
   * @returns Promise resolving to number of cached entries
   * @throws {Error} If operation fails
   */
  async cacheLen(): Promise<number> {
    this.throwIfDisposed();
    return await JsonEvalRs.cacheLen(this.handle);
  }

  /**
   * Enable evaluation caching
   * Useful for reusing JSONEval instances with different data
   * @returns Promise that resolves when cache is enabled
   * @throws {Error} If operation fails
   */
  async enableCache(): Promise<void> {
    this.throwIfDisposed();
    await JsonEvalRs.enableCache(this.handle);
  }

  /**
   * Disable evaluation caching
   * Useful for web API usage where each request creates a new JSONEval instance
   * Improves performance by skipping cache operations that have no benefit for single-use instances
   * @returns Promise that resolves when cache is disabled
   * @throws {Error} If operation fails
   */
  async disableCache(): Promise<void> {
    this.throwIfDisposed();
    await JsonEvalRs.disableCache(this.handle);
  }

  /**
   * Check if evaluation caching is enabled
   * @returns Boolean indicating if caching is enabled
   * @throws {Error} If operation fails
   */
  isCacheEnabled(): boolean {
    this.throwIfDisposed();
    return JsonEvalRs.isCacheEnabled(this.handle);
  }

  /**
   * Resolve layout with optional evaluation
   * @param evaluate - If true, runs evaluation before resolving layout (default: false)
   * @returns Promise that resolves when layout resolution is complete
   * @throws {Error} If operation fails
   */
  async resolveLayout(evaluate: boolean = false): Promise<void> {
    this.throwIfDisposed();
    await JsonEvalRs.resolveLayout(this.handle, evaluate);
  }

  /**
   * Set timezone offset for datetime operations (TODAY, NOW)
   * @param offsetMinutes - Timezone offset in minutes from UTC (e.g., 420 for UTC+7, -300 for UTC-5)
   *                        Pass null to reset to UTC
   * @returns Promise that resolves when timezone is set
   * @throws {Error} If operation fails
   * 
   * @example
   * ```typescript
   * // Set to UTC+7 (Jakarta, Bangkok)
   * await eval.setTimezoneOffset(420);
   * 
   * // Set to UTC-5 (New York, EST)
   * await eval.setTimezoneOffset(-300);
   * 
   * // Reset to UTC
   * await eval.setTimezoneOffset(null);
   * ```
   */
  async setTimezoneOffset(offsetMinutes: number | null): Promise<void> {
    this.throwIfDisposed();
    await JsonEvalRs.setTimezoneOffset(this.handle, offsetMinutes);
  }

  /**
   * Compile and run JSON logic from a JSON logic string
   * @param logicStr - JSON logic expression as a string or object
   * @param data - Optional JSON data string or object (null to use existing data)
   * @param context - Optional context data string or object (null to use existing context)
   * @returns Promise resolving to the result of the evaluation
   * @throws {Error} If compilation or evaluation fails
   */
  async compileAndRunLogic(logicStr: string | object, data?: string | object, context?: string | object): Promise<any> {
    this.throwIfDisposed();
    
    const logic = this.toJsonString(logicStr);
    const dataStr = data ? this.toJsonString(data) : null;
    const contextStr = context ? this.toJsonString(context) : null;
    
    const resultStr = await JsonEvalRs.compileAndRunLogic(this.handle, logic, dataStr, contextStr);
    return JSON.parse(resultStr);
  }

  /**
   * Compile JSON logic and return a global ID
   * @param logicStr - JSON logic expression as a string or object
   * @returns Promise resolving to the compiled logic ID
   * @throws {Error} If compilation fails
   */
  async compileLogic(logicStr: string | object): Promise<number> {
    this.throwIfDisposed();
    
    const logic = this.toJsonString(logicStr);
    return await JsonEvalRs.compileLogic(this.handle, logic);
  }

  /**
   * Run pre-compiled logic by ID
   * @param logicId - Compiled logic ID from compileLogic
   * @param data - Optional JSON data string or object (null to use existing data)
   * @param context - Optional context data string or object (null to use existing context)
   * @returns Promise resolving to the result of the evaluation
   * @throws {Error} If execution fails
   */
  async runLogic(logicId: number, data?: string | object, context?: string | object): Promise<any> {
    this.throwIfDisposed();
    
    const dataStr = data ? this.toJsonString(data) : null;
    const contextStr = context ? this.toJsonString(context) : null;
    
    const resultStr = await JsonEvalRs.runLogic(this.handle, logicId, dataStr, contextStr);
    return JSON.parse(resultStr);
  }

  /**
   * Validate data against schema rules with optional path filtering
   * @param options - Validation options with optional path filtering
   * @returns Promise resolving to ValidationResult
   * @throws {Error} If validation operation fails
   */
  async validatePaths(options: ValidatePathsOptions): Promise<ValidationResult> {
    this.throwIfDisposed();
    
    const dataStr = this.toJsonString(options.data);
    const contextStr = options.context ? this.toJsonString(options.context) : null;
    const paths = options.paths || null;
    
    const resultStr = await JsonEvalRs.validatePaths(this.handle, dataStr, contextStr, paths);
    return JSON.parse(resultStr);
  }

  // ============================================================================
  // Subform Methods
  // ============================================================================

  /**
   * Evaluate a subform with data
   * @param options - Evaluation options including subform path and data
   * @returns Promise that resolves when evaluation is complete
   * @throws {Error} If evaluation fails
   */
  async evaluateSubform(options: EvaluateSubformOptions): Promise<void> {
    this.throwIfDisposed();
    
    const dataStr = this.toJsonString(options.data);
    const contextStr = options.context ? this.toJsonString(options.context) : null;
    
    return JsonEvalRs.evaluateSubform(this.handle, options.subformPath, dataStr, contextStr);
  }

  /**
   * Validate subform data against its schema rules
   * @param options - Validation options including subform path and data
   * @returns Promise resolving to ValidationResult
   * @throws {Error} If validation fails
   */
  async validateSubform(options: ValidateSubformOptions): Promise<ValidationResult> {
    this.throwIfDisposed();
    
    const dataStr = this.toJsonString(options.data);
    const contextStr = options.context ? this.toJsonString(options.context) : null;
    
    const resultStr = await JsonEvalRs.validateSubform(this.handle, options.subformPath, dataStr, contextStr);
    return JSON.parse(resultStr);
  }

  /**
   * Evaluate dependents in a subform when fields change
   * @param options - Options including subform path, changed paths array, and optional data
   * @returns Promise resolving to dependent evaluation results
   * @throws {Error} If evaluation fails
   */
  async evaluateDependentsSubform(options: EvaluateDependentsSubformOptions): Promise<DependentChange[]> {
    this.throwIfDisposed();
    
    const dataStr = options.data ? this.toJsonString(options.data) : null;
    const contextStr = options.context ? this.toJsonString(options.context) : null;
    
    // For now, pass the first path since native bridge expects single path (wraps internally)
    const changedPath = options.changedPaths[0] || '';
    
    const resultStr = await JsonEvalRs.evaluateDependentsSubform(
      this.handle,
      options.subformPath,
      changedPath,
      dataStr,
      contextStr
    );
    return JSON.parse(resultStr);
  }

  /**
   * Resolve layout for subform
   * @param options - Options including subform path and evaluate flag
   * @returns Promise that resolves when layout is resolved
   * @throws {Error} If layout resolution fails
   */
  async resolveLayoutSubform(options: ResolveLayoutSubformOptions): Promise<void> {
    this.throwIfDisposed();
    
    return JsonEvalRs.resolveLayoutSubform(this.handle, options.subformPath, options.evaluate || false);
  }

  /**
   * Get evaluated schema from subform
   * @param options - Options including subform path and resolveLayout flag
   * @returns Promise resolving to evaluated schema
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any> {
    this.throwIfDisposed();
    
    const resultStr = await JsonEvalRs.getEvaluatedSchemaSubform(
      this.handle,
      options.subformPath,
      options.resolveLayout || false
    );
    return JSON.parse(resultStr);
  }

  /**
   * Get schema value from subform (all .value fields)
   * @param options - Options including subform path
   * @returns Promise resolving to schema values
   * @throws {Error} If operation fails
   */
  async getSchemaValueSubform(options: GetSchemaValueSubformOptions): Promise<any> {
    this.throwIfDisposed();
    
    const resultStr = await JsonEvalRs.getSchemaValueSubform(this.handle, options.subformPath);
    return JSON.parse(resultStr);
  }

  /**
   * Get evaluated schema without $params from subform
   * @param options - Options including subform path and resolveLayout flag
   * @returns Promise resolving to evaluated schema without $params
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaWithoutParamsSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any> {
    this.throwIfDisposed();
    
    const resultStr = await JsonEvalRs.getEvaluatedSchemaWithoutParamsSubform(
      this.handle,
      options.subformPath,
      options.resolveLayout || false
    );
    return JSON.parse(resultStr);
  }

  /**
   * Get evaluated schema by specific path from subform
   * @param options - Options including subform path, schema path, and skipLayout flag
   * @returns Promise resolving to value at path or null if not found
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPathSubform(options: GetEvaluatedSchemaByPathSubformOptions): Promise<any | null> {
    this.throwIfDisposed();
    
    const resultStr = await JsonEvalRs.getEvaluatedSchemaByPathSubform(
      this.handle,
      options.subformPath,
      options.schemaPath,
      options.skipLayout || false
    );
    return resultStr ? JSON.parse(resultStr) : null;
  }

  /**
   * Get evaluated schema by multiple paths from subform
   * Returns data in the specified format (skips paths that are not found)
   * @param options - Options including subform path, array of schema paths, skipLayout flag, and format
   * @returns Promise resolving to data in the specified format
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPathsSubform(options: GetEvaluatedSchemaByPathsSubformOptions): Promise<any> {
    this.throwIfDisposed();
    
    const pathsJson = JSON.stringify(options.schemaPaths);
    const resultStr = await JsonEvalRs.getEvaluatedSchemaByPathsSubform(
      this.handle,
      options.subformPath,
      pathsJson,
      options.skipLayout || false,
      options.format !== undefined ? options.format : ReturnFormat.Nested
    );
    return JSON.parse(resultStr);
  }

  /**
   * Get list of available subform paths
   * @returns Promise resolving to array of subform paths
   * @throws {Error} If operation fails
   */
  async getSubformPaths(): Promise<string[]> {
    this.throwIfDisposed();
    
    const resultStr = await JsonEvalRs.getSubformPaths(this.handle);
    return JSON.parse(resultStr);
  }

  /**
   * Get schema value by specific path from subform
   * @param options - Options including subform path and schema path
   * @returns Promise resolving to value at path or null if not found
   * @throws {Error} If operation fails
   */
  async getSchemaByPathSubform(options: GetSchemaByPathSubformOptions): Promise<any | null> {
    this.throwIfDisposed();
    
    const resultStr = await JsonEvalRs.getSchemaByPathSubform(
      this.handle,
      options.subformPath,
      options.schemaPath
    );
    return resultStr ? JSON.parse(resultStr) : null;
  }

  /**
   * Get schema values by multiple paths from subform
   * Returns data in the specified format (skips paths that are not found)
   * @param options - Options including subform path, array of schema paths, and format
   * @returns Promise resolving to data in the specified format
   * @throws {Error} If operation fails
   */
  async getSchemaByPathsSubform(options: GetSchemaByPathsSubformOptions): Promise<any> {
    this.throwIfDisposed();
    
    const pathsJson = JSON.stringify(options.schemaPaths);
    const resultStr = await JsonEvalRs.getSchemaByPathsSubform(
      this.handle,
      options.subformPath,
      pathsJson,
      options.format !== undefined ? options.format : ReturnFormat.Nested
    );
    return JSON.parse(resultStr);
  }

  /**
   * Check if a subform exists at the given path
   * @param subformPath - Path to check
   * @returns Promise resolving to true if subform exists, false otherwise
   * @throws {Error} If operation fails
   */
  async hasSubform(subformPath: string): Promise<boolean> {
    this.throwIfDisposed();
    
    return JsonEvalRs.hasSubform(this.handle, subformPath);
  }

  /**
   * Dispose of the native resources
   * Must be called when done using the instance
   * @returns Promise that resolves when disposal is complete
   */
  async dispose(): Promise<void> {
    if (this.disposed) return;
    
    await JsonEvalRs.dispose(this.handle);
    this.disposed = true;
  }

  /**
   * Get the library version
   * @returns Promise resolving to version string
   */
  static async version(): Promise<string> {
    return JsonEvalRs.version();
  }
}

/**
 * Hook for using JSONEval in React components with automatic cleanup
 * @param options - Configuration options
 * @returns JSONEval instance or null if not yet initialized
 * 
 * @example
 * ```typescript
 * import { useJSONEval } from '@json-eval-rs/react-native';
 * 
 * function MyComponent() {
 *   const eval = useJSONEval({ schema: mySchema });
 *   
 *   const handleValidate = async () => {
 *     if (!eval) return;
 *     const result = await eval.validate({ data: myData });
 *     console.log(result);
 *   };
 *   
 *   return <Button onPress={handleValidate} title="Validate" />;
 * }
 * ```
 */
export function useJSONEval(options: JSONEvalOptions): JSONEval | null {
  const [evalInstance, setEvalInstance] = React.useState<JSONEval | null>(null);

  React.useEffect(() => {
    const instance = new JSONEval(options);
    setEvalInstance(instance);

    return () => {
      instance.dispose().catch(console.error);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return evalInstance;
}

// Default export
export default JSONEval;

// For backwards compatibility
export const multiply = (a: number, b: number): Promise<number> => {
  return JsonEvalRs.multiply(a, b);
};
