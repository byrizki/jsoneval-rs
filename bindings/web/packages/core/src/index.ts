/**
 * @json-eval-rs/webcore
 * High-level JavaScript API for JSON Eval RS WASM bindings
 *
 * This package provides a clean, ergonomic API that works with any WASM target:
 */

/**
 * Get the library version from the WASM module
 * @param wasmModule - WASM module
 * @returns Version string
 */
export function getVersion(wasmModule: any): string {
  if (wasmModule && typeof wasmModule.getVersion === "function") {
    return wasmModule.getVersion();
  }
  return "unknown";
}

/**
 * Item for get schema value array results
 */
export interface SchemaValueItem {
  /** Dotted path (e.g., "field1.field2") */
  path: string;
  /** Value at this path */
  value: any;
}

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
   * Useful for user sessions, environment variables, etc.
   */
  context?: any;
  /**
   * Optional initial data object to evaluate against.
   * Can be updated later with reloadSchema.
   */
  data?: any;
  /**
   * If true, the `schema` parameter is treated as a string cache key
   * to lookup a pre-parsed schema from the global cache.
   * Default: false
   */
  fromCache?: boolean;
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
  /** Optional array of paths for selective evaluation */
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

/**
 * JSONEval - High-level JavaScript API for JSON Eval RS
 *
 * This is an internal abstraction layer. Use specific packages instead:
 * - @json-eval-rs/bundler (for bundlers like Webpack, Vite, Next.js)
 * - @json-eval-rs/vanilla (for direct browser usage)
 * - @json-eval-rs/node (for Node.js/SSR)
 *
 * @example
 * ```js
 * import { JSONEval } from '@json-eval-rs/webcore';
 *
 * const evaluator = new JSONEval({
 *   schema: { type: 'object', properties: { ... } }
 * });
 *
 * await evaluator.init();
 * const result = await evaluator.validate({ data: { name: 'John' } });
 * ```
 */
export class JSONEvalCore {
  /** Internal storage for the schema (JSON, MsgPack, or Cache Key) */
  private _schema: any;
  /** Reference to the loaded WASM module */
  private _wasmModule: any;
  /** Current context data */
  private _context: any;
  /** Current evaluation data */
  private _data: any;
  /** The underlying WASM JSONEval instance */
  private _instance: any = null;
  /** Initialization state flag */
  private _ready: boolean = false;
  /** Flag indicating if schema provided is binary MessagePack */
  private _isMsgpackSchema: boolean;
  /** Flag indicating if schema is a cache key reference */
  private _isFromCache: boolean;

  /**
   * Create a new JSONEval Core instance.
   * Does not initialize WASM immediately; wait for `init()` or call async methods.
   *
   * @param wasmModule - The loaded WASM module (provided by wrapper packages like @json-eval-rs/node or /vanilla)
   * @param options - Configuration options containing schema, data, and context
   */
  constructor(
    wasmModule: any,
    { schema, context, data, fromCache = false }: JSONEvalOptions
  ) {
    this._schema = schema;
    this._wasmModule = wasmModule;
    this._context = context;
    this._data = data;
    this._instance = null;
    this._ready = false;
    this._isMsgpackSchema = schema instanceof Uint8Array;
    this._isFromCache = fromCache;
  }

  /**
   * Initialize the WASM instance
   * Call this before using other methods, or use the async methods which call it automatically
   */
  async init(): Promise<void> {
    if (this._ready) return;

    // If WASM module not provided, throw error - user must provide it or install peer dependency
    if (!this._wasmModule) {
      throw new Error(
        "No WASM module provided. Please either:\n" +
          '1. Pass wasmModule in constructor: new JSONEval({ schema, wasmModule: await import("@json-eval-rs/bundler") })\n' +
          "2. Or install a peer dependency: yarn install @json-eval-rs/bundler (or @json-eval-rs/vanilla or @json-eval-rs/node)"
      );
    }

    try {
      const { JSONEvalWasm } = this._wasmModule;

      // Create instance from cache, MessagePack, or JSON
      if (this._isFromCache) {
        this._instance = JSONEvalWasm.newFromCache(
          this._schema, // cache key
          this._context ? JSON.stringify(this._context) : null,
          this._data ? JSON.stringify(this._data) : null
        );
      } else if (this._isMsgpackSchema) {
        this._instance = JSONEvalWasm.newFromMsgpack(
          this._schema,
          this._context ? JSON.stringify(this._context) : null,
          this._data ? JSON.stringify(this._data) : null
        );
      } else {
        this._instance = new JSONEvalWasm(
          JSON.stringify(this._schema),
          this._context ? JSON.stringify(this._context) : null,
          this._data ? JSON.stringify(this._data) : null
        );
      }
      this._ready = true;
    } catch (error: any) {
      throw new Error(
        `Failed to create JSONEval instance: ${error.message || error}`
      );
    }
  }

  /**
   * Create a new JSONEval instance from a cached ParsedSchema
   * Static factory method for convenience
   *
   * @param wasmModule - WASM module
   * @param cacheKey - Cache key to lookup in ParsedSchemaCache
   * @param context - Optional context data
   * @param data - Optional initial data
   * @returns New instance
   */
  static fromCache(
    wasmModule: any,
    cacheKey: string,
    context?: any,
    data?: any
  ): JSONEvalCore {
    return new JSONEvalCore(wasmModule, {
      schema: cacheKey,
      context,
      data,
      fromCache: true,
    });
  }

  /**
   * Evaluate logic expression without creating an instance
   *
   * @param wasmModule - WASM module
   * @param logicStr - JSON Logic expression (string or object)
   * @param data - Optional data (string or object)
   * @param context - Optional context (string or object)
   * @returns Evaluation result
   */
  static evaluateLogic(
    wasmModule: any,
    logicStr: string | object,
    data?: any,
    context?: any
  ): any {
    if (!wasmModule) {
      throw new Error("No WASM module provided.");
    }
    const { JSONEvalWasm } = wasmModule;
    if (!JSONEvalWasm || typeof JSONEvalWasm.evaluateLogic !== "function") {
      throw new Error("WASM module does not support evaluateLogic.");
    }

    const logic =
      typeof logicStr === "string" ? logicStr : JSON.stringify(logicStr);
    const dataStr = data
      ? typeof data === "string"
        ? data
        : JSON.stringify(data)
      : null;
    const contextStr = context
      ? typeof context === "string"
        ? context
        : JSON.stringify(context)
      : null;

    return JSONEvalWasm.evaluateLogic(logic, dataStr, contextStr);
  }

  /**
   * Validate data against schema (returns parsed JavaScript object)
   * Uses validateJS for Worker-safe serialization
   */
  async validate({
    data,
    context,
  }: ValidateOptions): Promise<ValidationResult> {
    await this.init();
    try {
      // Use validateJS for proper serialization (Worker-safe)
      return this._instance.validateJS(
        JSON.stringify(data),
        context ? JSON.stringify(context) : null
      );
    } catch (error: any) {
      throw new Error(`Validation failed: ${error.message || error}`);
    }
  }

  /**
   * Evaluate schema with data (returns parsed JavaScript object)
   */
  async evaluate({ data, context, paths }: EvaluateOptions): Promise<any> {
    await this.init();
    try {
      return this._instance.evaluateJS(
        JSON.stringify(data),
        context ? JSON.stringify(context) : null,
        paths || null
      );
    } catch (error: any) {
      throw new Error(`Evaluation failed: ${error.message || error}`);
    }
  }

  /**
   * Evaluate dependent fields (returns parsed JavaScript object, processes transitively)
   */
  async evaluateDependents({
    changedPaths,
    data,
    context,
    reEvaluate = true,
  }: EvaluateDependentsOptions): Promise<DependentChange[]> {
    await this.init();
    try {
      // Ensure paths is an array for WASM
      const paths = Array.isArray(changedPaths) ? changedPaths : [changedPaths];

      return this._instance.evaluateDependentsJS(
        JSON.stringify(paths),
        data ? JSON.stringify(data) : null,
        context ? JSON.stringify(context) : null,
        reEvaluate
      );
    } catch (error: any) {
      throw new Error(`Dependent evaluation failed: ${error.message || error}`);
    }
  }

  /**
   * Get evaluated schema
   */
  async getEvaluatedSchema({
    skipLayout = false,
  }: GetEvaluatedSchemaOptions = {}): Promise<any> {
    await this.init();
    return this._instance.getEvaluatedSchemaJS(skipLayout);
  }

  /**
   * Get evaluated schema as MessagePack binary data
   */
  async getEvaluatedSchemaMsgpack({
    skipLayout = false,
  }: GetEvaluatedSchemaOptions = {}): Promise<Uint8Array> {
    await this.init();
    return this._instance.getEvaluatedSchemaMsgpack(skipLayout);
  }

  /**
   * Get schema values (evaluations ending with .value)
   */
  async getSchemaValue(): Promise<any> {
    await this.init();
    return this._instance.getSchemaValue();
  }

  /**
   * Get all schema values as array of path-value pairs
   * Returns [{path: "", value: ""}, ...]
   */
  async getSchemaValueArray(): Promise<SchemaValueItem[]> {
    await this.init();
    return this._instance.getSchemaValueArray();
  }

  /**
   * Get all schema values as object with dotted path keys
   * Returns {path: value, ...}
   */
  async getSchemaValueObject(): Promise<Record<string, any>> {
    await this.init();
    return this._instance.getSchemaValueObject();
  }

  /**
   * Get evaluated schema without $params field
   */
  async getEvaluatedSchemaWithoutParams({
    skipLayout = false,
  }: GetEvaluatedSchemaOptions = {}): Promise<any> {
    await this.init();
    return this._instance.getEvaluatedSchemaWithoutParamsJS(skipLayout);
  }

  /**
   * Get a value from the evaluated schema using dotted path notation
   */
  async getEvaluatedSchemaByPath({
    path,
    skipLayout = false,
  }: GetValueByPathOptions): Promise<any | null> {
    await this.init();
    return this._instance.getEvaluatedSchemaByPathJS(path, skipLayout);
  }

  /**
   * Get values from the evaluated schema using multiple dotted path notations
   * Returns data in the specified format (skips paths that are not found)
   */
  async getEvaluatedSchemaByPaths({
    paths,
    skipLayout = false,
    format = 0,
  }: GetValueByPathsOptions): Promise<any> {
    await this.init();
    return this._instance.getEvaluatedSchemaByPathsJS(
      JSON.stringify(paths),
      skipLayout,
      format
    );
  }

  /**
   * Get a value from the schema using dotted path notation
   */
  async getSchemaByPath({ path }: GetSchemaByPathOptions): Promise<any | null> {
    await this.init();
    return this._instance.getSchemaByPathJS(path);
  }

  /**
   * Get values from the schema using multiple dotted path notations
   * Returns data in the specified format (skips paths that are not found)
   */
  async getSchemaByPaths({
    paths,
    format = 0,
  }: GetSchemaByPathsOptions): Promise<any> {
    await this.init();
    return this._instance.getSchemaByPathsJS(JSON.stringify(paths), format);
  }

  /**
   * Reload schema with new data
   */
  async reloadSchema({
    schema,
    context,
    data,
  }: ReloadSchemaOptions): Promise<void> {
    if (!this._instance) {
      throw new Error("Instance not initialized. Call init() first.");
    }

    try {
      await this._instance.reloadSchema(
        JSON.stringify(schema),
        context ? JSON.stringify(context) : null,
        data ? JSON.stringify(data) : null
      );

      // Update internal state
      this._schema = schema;
      this._context = context;
      this._data = data;
    } catch (error: any) {
      throw new Error(`Failed to reload schema: ${error.message || error}`);
    }
  }

  /**
   * Reload schema from MessagePack bytes
   */
  async reloadSchemaMsgpack(
    schemaMsgpack: Uint8Array,
    context?: any,
    data?: any
  ): Promise<void> {
    if (!this._instance) {
      throw new Error("Instance not initialized. Call init() first.");
    }

    if (!(schemaMsgpack instanceof Uint8Array)) {
      throw new Error("schemaMsgpack must be a Uint8Array");
    }

    try {
      await this._instance.reloadSchemaMsgpack(
        schemaMsgpack,
        context ? JSON.stringify(context) : null,
        data ? JSON.stringify(data) : null
      );

      // Update internal state
      this._schema = schemaMsgpack;
      this._context = context;
      this._data = data;
      this._isMsgpackSchema = true;
    } catch (error: any) {
      throw new Error(
        `Failed to reload schema from MessagePack: ${error.message || error}`
      );
    }
  }

  /**
   * Reload schema from ParsedSchemaCache using a cache key
   */
  async reloadSchemaFromCache(
    cacheKey: string,
    context?: any,
    data?: any
  ): Promise<void> {
    if (!this._instance) {
      throw new Error("Instance not initialized. Call init() first.");
    }

    if (typeof cacheKey !== "string" || !cacheKey) {
      throw new Error("cacheKey must be a non-empty string");
    }

    try {
      await this._instance.reloadSchemaFromCache(
        cacheKey,
        context ? JSON.stringify(context) : null,
        data ? JSON.stringify(data) : null
      );

      // Update internal state
      this._context = context;
      this._data = data;
      // Note: schema is not updated as we don't have access to it from the cache key
    } catch (error: any) {
      throw new Error(
        `Failed to reload schema from cache: ${error.message || error}`
      );
    }
  }

  /**
   * Get cache statistics
   */
  async cacheStats(): Promise<CacheStats> {
    await this.init();
    return this._instance.cacheStats();
  }

  /**
   * Clear the evaluation cache
   */
  async clearCache(): Promise<void> {
    await this.init();
    this._instance.clearCache();
  }

  /**
   * Get the number of cached entries
   */
  async cacheLen(): Promise<number> {
    await this.init();
    return this._instance.cacheLen();
  }

  /**
   * Enable evaluation caching
   */
  async enableCache(): Promise<void> {
    await this.init();
    this._instance.enableCache();
  }

  /**
   * Disable evaluation caching
   */
  async disableCache(): Promise<void> {
    await this.init();
    this._instance.disableCache();
  }

  /**
   * Check if evaluation caching is enabled
   */
  isCacheEnabled(): boolean {
    if (!this._instance) return true; // Default is enabled
    return this._instance.isCacheEnabled();
  }

  /**
   * Resolve layout with optional evaluation
   */
  async resolveLayout(options: { evaluate?: boolean } = {}): Promise<void> {
    const { evaluate = false } = options;
    await this.init();
    return this._instance.resolveLayout(evaluate);
  }

  /**
   * Set timezone offset for datetime operations (TODAY, NOW)
   * @param offsetMinutes - Timezone offset in minutes from UTC
   *                        (e.g., 420 for UTC+7, -300 for UTC-5)
   *                        Pass null or undefined to reset to UTC
   */
  setTimezoneOffset(offsetMinutes: number | null | undefined): void {
    if (!this._instance) {
      throw new Error("Instance not initialized. Call init() first.");
    }
    this._instance.setTimezoneOffset(offsetMinutes);
  }

  /**
   * Compile and run JSON logic from a JSON logic string
   */
  async compileAndRunLogic({
    logicStr,
    data,
    context,
  }: CompileAndRunLogicOptions): Promise<any> {
    await this.init();
    const logic =
      typeof logicStr === "string" ? logicStr : JSON.stringify(logicStr);
    const result = await this._instance.compileAndRunLogic(
      logic,
      data ? JSON.stringify(data) : null,
      context ? JSON.stringify(context) : null
    );
    return result;
  }

  /**
   * Compile JSON logic and return a global ID
   */
  async compileLogic(logicStr: string | object): Promise<number> {
    await this.init();
    const logic =
      typeof logicStr === "string" ? logicStr : JSON.stringify(logicStr);
    return this._instance.compileLogic(logic);
  }

  /**
   * Run pre-compiled logic by ID
   */
  async runLogic(logicId: number, data?: any, context?: any): Promise<any> {
    await this.init();
    const result = await this._instance.runLogic(
      logicId,
      data ? JSON.stringify(data) : null,
      context ? JSON.stringify(context) : null
    );
    return result;
  }

  /**
   * Validate data against schema rules with optional path filtering
   */
  async validatePaths({
    data,
    context,
    paths,
  }: EvaluateOptions): Promise<ValidationResult> {
    await this.init();
    try {
      // Use validatePathsJS for proper serialization (Worker-safe)
      return this._instance.validatePathsJS(
        JSON.stringify(data),
        context ? JSON.stringify(context) : null,
        paths || null
      );
    } catch (error: any) {
      throw new Error(`Validation failed: ${error.message || error}`);
    }
  }

  /**
   * Cancel any running evaluation
   */
  async cancel(): Promise<void> {
    if (this._ready && this._instance) {
      this._instance.cancel();
    }
  }

  // ============================================================================
  // Subform Methods
  // ============================================================================

  /**
   * Evaluate a subform with data
   */
  async evaluateSubform({
    subformPath,
    data,
    context,
    paths,
  }: EvaluateSubformOptions): Promise<void> {
    await this.init();
    return this._instance.evaluateSubform(
      subformPath,
      JSON.stringify(data),
      context ? JSON.stringify(context) : null,
      paths || null
    );
  }

  /**
   * Validate subform data against its schema rules
   */
  async validateSubform({
    subformPath,
    data,
    context,
  }: ValidateSubformOptions): Promise<ValidationResult> {
    await this.init();
    return this._instance.validateSubform(
      subformPath,
      JSON.stringify(data),
      context ? JSON.stringify(context) : null
    );
  }

  /**
   * Evaluate dependent fields in subform
   */
  async evaluateDependentsSubform({
    subformPath,
    changedPaths,
    data,
    context,
    reEvaluate = true,
  }: EvaluateDependentsSubformOptions): Promise<DependentChange[]> {
    await this.init();

    // For backward compatibility, accept single changedPath too (though types say array)
    const paths = Array.isArray(changedPaths) ? changedPaths : [changedPaths];

    return this._instance.evaluateDependentsSubformJS(
      subformPath,
      JSON.stringify(paths),
      data ? JSON.stringify(data) : null,
      context ? JSON.stringify(context) : null,
      reEvaluate
    );
  }

  /**
   * Resolve layout for subform
   */
  async resolveLayoutSubform({
    subformPath,
    evaluate = false,
  }: ResolveLayoutSubformOptions): Promise<void> {
    await this.init();
    return this._instance.resolveLayoutSubform(subformPath, evaluate);
  }

  /**
   * Get evaluated schema from subform
   */
  async getEvaluatedSchemaSubform({
    subformPath,
    resolveLayout = false,
  }: GetEvaluatedSchemaSubformOptions): Promise<any> {
    await this.init();
    return this._instance.getEvaluatedSchemaSubformJS(
      subformPath,
      resolveLayout
    );
  }

  /**
   * Get schema value from subform in nested object format (all .value fields).
   * Returns a hierarchical object structure mimicking the schema.
   */
  async getSchemaValueSubform({
    subformPath,
  }: GetSchemaValueSubformOptions): Promise<any> {
    await this.init();
    return this._instance.getSchemaValueSubform(subformPath);
  }

  /**
   * Get schema values from subform as a flat array of path-value pairs.
   * Returns an array like `[{path: "field.sub", value: 123}, ...]`.
   */
  async getSchemaValueArraySubform({
    subformPath,
  }: GetSchemaValueSubformOptions): Promise<SchemaValueItem[]> {
    await this.init();
    return this._instance.getSchemaValueArraySubform(subformPath);
  }

  /**
   * Get schema values from subform as a flat object with dotted path keys.
   * Returns an object like `{"field.sub": 123, ...}`.
   */
  async getSchemaValueObjectSubform({
    subformPath,
  }: GetSchemaValueSubformOptions): Promise<Record<string, any>> {
    await this.init();
    return this._instance.getSchemaValueObjectSubform(subformPath);
  }

  /**
   * Get evaluated schema without $params from subform
   */
  async getEvaluatedSchemaWithoutParamsSubform({
    subformPath,
    resolveLayout = false,
  }: GetEvaluatedSchemaSubformOptions): Promise<any> {
    await this.init();
    return this._instance.getEvaluatedSchemaWithoutParamsSubformJS(
      subformPath,
      resolveLayout
    );
  }

  /**
   * Get evaluated schema by specific path from subform
   */
  async getEvaluatedSchemaByPathSubform({
    subformPath,
    schemaPath,
    skipLayout = false,
  }: GetEvaluatedSchemaByPathSubformOptions): Promise<any | null> {
    await this.init();
    return this._instance.getEvaluatedSchemaByPathSubformJS(
      subformPath,
      schemaPath,
      skipLayout
    );
  }

  /**
   * Get evaluated schema by multiple paths from subform
   * Returns data in the specified format (skips paths that are not found)
   */
  async getEvaluatedSchemaByPathsSubform({
    subformPath,
    schemaPaths,
    skipLayout = false,
    format = 0,
  }: GetEvaluatedSchemaByPathsSubformOptions): Promise<any> {
    await this.init();
    return this._instance.getEvaluatedSchemaByPathsSubformJS(
      subformPath,
      JSON.stringify(schemaPaths),
      skipLayout,
      format
    );
  }

  /**
   * Get list of available subform paths
   */
  async getSubformPaths(): Promise<string[]> {
    await this.init();
    return this._instance.getSubformPaths();
  }

  /**
   * Get schema by specific path from subform
   */
  async getSchemaByPathSubform({
    subformPath,
    schemaPath,
  }: GetSchemaByPathSubformOptions): Promise<any | null> {
    await this.init();
    return this._instance.getSchemaByPathSubformJS(subformPath, schemaPath);
  }

  /**
   * Get schema by multiple paths from subform
   * Returns data in the specified format (skips paths that are not found)
   */
  async getSchemaByPathsSubform({
    subformPath,
    schemaPaths,
    format = 0,
  }: GetSchemaByPathsSubformOptions): Promise<any> {
    await this.init();
    return this._instance.getSchemaByPathsSubformJS(
      subformPath,
      JSON.stringify(schemaPaths),
      format
    );
  }

  /**
   * Check if a subform exists at the given path
   */
  async hasSubform(subformPath: string): Promise<boolean> {
    await this.init();
    return this._instance.hasSubform(subformPath);
  }

  /**
   * Free WASM resources
   */
  free(): void {
    if (this._instance) {
      this._instance.free();
      this._instance = null;
      this._ready = false;
    }
  }
}

export default JSONEvalCore;
