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
export declare enum ReturnFormat {
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
    /** Optional array of paths for selective evaluation */
    paths?: string[];
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
export declare class JSONEval {
    private handle;
    private disposed;
    /**
     * Creates a new JSON evaluator instance from a cached ParsedSchema
     * @param cacheKey - Cache key to lookup in the global ParsedSchemaCache
     * @param context - Optional context data
     * @param data - Optional initial data
     * @returns New JSONEval instance
     * @throws {Error} If schema not found in cache or creation fails
     */
    static fromCache(cacheKey: string, context?: string | object | null, data?: string | object | null): JSONEval;
    /**
     * Evaluates logic expression without creating an instance
     * @param logicStr - JSON Logic expression as string or object
     * @param data - Optional data as string or object
     * @param context - Optional context as string or object
     * @returns Promise resolving to evaluation result
     */
    static evaluateLogic(logicStr: string | object, data?: string | object | null, context?: string | object | null): Promise<any>;
    /**
     * Creates a new JSON evaluator instance
     * @param options - Configuration options with schema, context, and data
     * @throws {Error} If creation fails
     */
    constructor(options: JSONEvalOptions & {
        _handle?: string;
    });
    private throwIfDisposed;
    /**
     * Convert value to JSON string
     * Performance note: If you have a pre-serialized JSON string, pass it directly
     * instead of an object to avoid the JSONStringify overhead
     */
    private toJsonString;
    /**
     * Cancel any running evaluation
     * The generic auto-cancellation on new evaluation will still work,
     * but this allows manual cancellation.
     */
    cancel(): Promise<void>;
    /**
     * Evaluate schema with provided data
     * @param options - Evaluation options
     * @returns Promise resolving to evaluated schema object
     * @throws {Error} If evaluation fails
     */
    evaluate(options: EvaluateOptions): Promise<any>;
    /**
     * Validate data against schema rules
     * @param options - Validation options
     * @returns Promise resolving to ValidationResult
     * @throws {Error} If validation operation fails
     */
    validate(options: EvaluateOptions): Promise<ValidationResult>;
    /**
     * Re-evaluate fields that depend on a changed path
     * @param options - Dependent evaluation options
     * @returns Promise resolving to array of dependent field changes
     * @throws {Error} If evaluation fails
     */
    evaluateDependents(options: EvaluateDependentsOptions): Promise<DependentChange[]>;
    /**
     * Get the evaluated schema with optional layout resolution
     * @param skipLayout - Whether to skip layout resolution (default: false)
     * @returns Promise resolving to evaluated schema object
     * @throws {Error} If operation fails
     */
    getEvaluatedSchema(skipLayout?: boolean): Promise<any>;
    /**
     * Get all schema values (evaluations ending with .value)
     * @returns Promise resolving to map of path -> value
     * @throws {Error} If operation fails
     */
    getSchemaValue(): Promise<Record<string, any>>;
    /**
     * Get all schema values as array of path-value pairs
     * Returns [{path: "", value: ""}, ...]
     * @returns Promise resolving to array of SchemaValueItem objects
     * @throws {Error} If operation fails
     */
    getSchemaValueArray(): Promise<SchemaValueItem[]>;
    /**
     * Get all schema values as object with dotted path keys
     * Returns {path: value, ...}
     * @returns Promise resolving to flat object with dotted paths as keys
     * @throws {Error} If operation fails
     */
    getSchemaValueObject(): Promise<Record<string, any>>;
    /**
     * Get the evaluated schema without $params field
     * @param skipLayout - Whether to skip layout resolution (default: false)
     * @returns Promise resolving to evaluated schema object
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaWithoutParams(skipLayout?: boolean): Promise<any>;
    /**
     * Get a value from the evaluated schema using dotted path notation
     * @param path - Dotted path to the value (e.g., "properties.field.value")
     * @param skipLayout - Whether to skip layout resolution
     * @returns Promise resolving to the value at the path, or null if not found
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaByPath(path: string, skipLayout?: boolean): Promise<any | null>;
    /**
     * Get values from the evaluated schema using multiple dotted path notations
     * Returns data in the specified format (skips paths that are not found)
     * @param paths - Array of dotted paths to retrieve
     * @param skipLayout - Whether to skip layout resolution
     * @param format - Return format (Nested, Flat, or Array)
     * @returns Promise resolving to data in the specified format
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaByPaths(paths: string[], skipLayout?: boolean, format?: ReturnFormat): Promise<any>;
    /**
     * Get a value from the schema using dotted path notation
     * @param path - Dotted path to the value (e.g., "properties.field.value")
     * @returns Promise resolving to the value at the path, or null if not found
     * @throws {Error} If operation fails
     */
    getSchemaByPath(path: string): Promise<any | null>;
    /**
     * Get values from the schema using multiple dotted path notations
     * Returns data in the specified format (skips paths that are not found)
     * @param paths - Array of dotted paths to retrieve
     * @param format - Return format (Nested, Flat, or Array)
     * @returns Promise resolving to data in the specified format
     * @throws {Error} If operation fails
     */
    getSchemaByPaths(paths: string[], format?: ReturnFormat): Promise<any>;
    /**
     * Reload schema with new data
     * @param options - Configuration options with new schema, context, and data
     * @throws {Error} If reload fails
     */
    reloadSchema(options: JSONEvalOptions): Promise<void>;
    /**
     * Reload schema from MessagePack bytes
     * @param schemaMsgpack - MessagePack-encoded schema bytes (Uint8Array or number array)
     * @param context - Optional context data
     * @param data - Optional initial data
     * @throws {Error} If reload fails
     */
    reloadSchemaMsgpack(schemaMsgpack: Uint8Array | number[], context?: string | object | null, data?: string | object | null): Promise<void>;
    /**
     * Reload schema from ParsedSchemaCache using a cache key
     * @param cacheKey - Cache key to lookup in the global ParsedSchemaCache
     * @param context - Optional context data
     * @param data - Optional initial data
     * @throws {Error} If reload fails or schema not found in cache
     */
    reloadSchemaFromCache(cacheKey: string, context?: string | object | null, data?: string | object | null): Promise<void>;
    /**
     * Get cache statistics
     * @returns Promise resolving to cache statistics
     * @throws {Error} If operation fails
     */
    cacheStats(): Promise<CacheStats>;
    /**
     * Clear the evaluation cache
     * @returns Promise that resolves when cache is cleared
     * @throws {Error} If operation fails
     */
    clearCache(): Promise<void>;
    /**
     * Get the number of cached entries
     * @returns Promise resolving to number of cached entries
     * @throws {Error} If operation fails
     */
    cacheLen(): Promise<number>;
    /**
     * Enable evaluation caching
     * Useful for reusing JSONEval instances with different data
     * @returns Promise that resolves when cache is enabled
     * @throws {Error} If operation fails
     */
    enableCache(): Promise<void>;
    /**
     * Disable evaluation caching
     * Useful for web API usage where each request creates a new JSONEval instance
     * Improves performance by skipping cache operations that have no benefit for single-use instances
     * @returns Promise that resolves when cache is disabled
     * @throws {Error} If operation fails
     */
    disableCache(): Promise<void>;
    /**
     * Check if evaluation caching is enabled
     * @returns Boolean indicating if caching is enabled
     * @throws {Error} If operation fails
     */
    isCacheEnabled(): boolean;
    /**
     * Resolve layout with optional evaluation
     * @param evaluate - If true, runs evaluation before resolving layout (default: false)
     * @returns Promise that resolves when layout resolution is complete
     * @throws {Error} If operation fails
     */
    resolveLayout(evaluate?: boolean): Promise<void>;
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
    setTimezoneOffset(offsetMinutes: number | null): Promise<void>;
    /**
     * Compile and run JSON logic from a JSON logic string
     * @param logicStr - JSON logic expression as a string or object
     * @param data - Optional JSON data string or object (null to use existing data)
     * @param context - Optional context data string or object (null to use existing context)
     * @returns Promise resolving to the result of the evaluation
     * @throws {Error} If compilation or evaluation fails
     */
    compileAndRunLogic(logicStr: string | object, data?: string | object, context?: string | object): Promise<any>;
    /**
     * Compile JSON logic and return a global ID
     * @param logicStr - JSON logic expression as a string or object
     * @returns Promise resolving to the compiled logic ID
     * @throws {Error} If compilation fails
     */
    compileLogic(logicStr: string | object): Promise<number>;
    /**
     * Run pre-compiled logic by ID
     * @param logicId - Compiled logic ID from compileLogic
     * @param data - Optional JSON data string or object (null to use existing data)
     * @param context - Optional context data string or object (null to use existing context)
     * @returns Promise resolving to the result of the evaluation
     * @throws {Error} If execution fails
     */
    runLogic(logicId: number, data?: string | object, context?: string | object): Promise<any>;
    /**
     * Validate data against schema rules with optional path filtering
     * @param options - Validation options with optional path filtering
     * @returns Promise resolving to ValidationResult
     * @throws {Error} If validation operation fails
     */
    validatePaths(options: ValidatePathsOptions): Promise<ValidationResult>;
    /**
     * Evaluate a subform with data
     * @param options - Evaluation options including subform path and data
     * @returns Promise that resolves when evaluation is complete
     * @throws {Error} If evaluation fails
     */
    evaluateSubform(options: EvaluateSubformOptions): Promise<void>;
    /**
     * Validate subform data against its schema rules
     * @param options - Validation options including subform path and data
     * @returns Promise resolving to ValidationResult
     * @throws {Error} If validation fails
     */
    validateSubform(options: ValidateSubformOptions): Promise<ValidationResult>;
    /**
     * Evaluate dependents in a subform when fields change
     * @param options - Options including subform path, changed paths array, and optional data
     * @returns Promise resolving to dependent evaluation results
     * @throws {Error} If evaluation fails
     */
    evaluateDependentsSubform(options: EvaluateDependentsSubformOptions): Promise<DependentChange[]>;
    /**
     * Resolve layout for subform
     * @param options - Options including subform path and evaluate flag
     * @returns Promise that resolves when layout is resolved
     * @throws {Error} If layout resolution fails
     */
    resolveLayoutSubform(options: ResolveLayoutSubformOptions): Promise<void>;
    /**
     * Get evaluated schema from subform
     * @param options - Options including subform path and resolveLayout flag
     * @returns Promise resolving to evaluated schema
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any>;
    /**
     * Get schema value from subform (all .value fields)
     * @param options - Options including subform path
     * @returns Promise resolving to schema values
     * @throws {Error} If operation fails
     */
    getSchemaValueSubform(options: GetSchemaValueSubformOptions): Promise<any>;
    /**
     * Get schema values from subform as a flat array of path-value pairs.
     * Returns an array like `[{path: "field.sub", value: 123}, ...]`.
     * @param options - Options including subform path
     * @returns Promise resolving to array of SchemaValueItem objects
     * @throws {Error} If operation fails
     */
    getSchemaValueArraySubform(options: GetSchemaValueSubformOptions): Promise<SchemaValueItem[]>;
    /**
     * Get schema values from subform as a flat object with dotted path keys.
     * Returns an object like `{"field.sub": 123, ...}`.
     * @param options - Options including subform path
     * @returns Promise resolving to flat object with dotted paths
     * @throws {Error} If operation fails
     */
    getSchemaValueObjectSubform(options: GetSchemaValueSubformOptions): Promise<Record<string, any>>;
    /**
     * Get evaluated schema without $params from subform
     * @param options - Options including subform path and resolveLayout flag
     * @returns Promise resolving to evaluated schema without $params
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaWithoutParamsSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any>;
    /**
     * Get evaluated schema by specific path from subform
     * @param options - Options including subform path, schema path, and skipLayout flag
     * @returns Promise resolving to value at path or null if not found
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaByPathSubform(options: GetEvaluatedSchemaByPathSubformOptions): Promise<any | null>;
    /**
     * Get evaluated schema by multiple paths from subform
     * Returns data in the specified format (skips paths that are not found)
     * @param options - Options including subform path, array of schema paths, skipLayout flag, and format
     * @returns Promise resolving to data in the specified format
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaByPathsSubform(options: GetEvaluatedSchemaByPathsSubformOptions): Promise<any>;
    /**
     * Get list of available subform paths
     * @returns Promise resolving to array of subform paths
     * @throws {Error} If operation fails
     */
    getSubformPaths(): Promise<string[]>;
    /**
     * Get schema value by specific path from subform
     * @param options - Options including subform path and schema path
     * @returns Promise resolving to value at path or null if not found
     * @throws {Error} If operation fails
     */
    getSchemaByPathSubform(options: GetSchemaByPathSubformOptions): Promise<any | null>;
    /**
     * Get schema values by multiple paths from subform
     * Returns data in the specified format (skips paths that are not found)
     * @param options - Options including subform path, array of schema paths, and format
     * @returns Promise resolving to data in the specified format
     * @throws {Error} If operation fails
     */
    getSchemaByPathsSubform(options: GetSchemaByPathsSubformOptions): Promise<any>;
    /**
     * Check if a subform exists at the given path
     * @param subformPath - Path to check
     * @returns Promise resolving to true if subform exists, false otherwise
     * @throws {Error} If operation fails
     */
    hasSubform(subformPath: string): Promise<boolean>;
    /**
     * Dispose of the native resources
     * Must be called when done using the instance
     * @returns Promise that resolves when disposal is complete
     */
    dispose(): Promise<void>;
    /**
     * Get the library version
     * @returns Promise resolving to version string
     */
    static version(): Promise<string>;
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
export declare function useJSONEval(options: JSONEvalOptions): JSONEval | null;
export default JSONEval;
export declare const multiply: (a: number, b: number) => Promise<number>;
//# sourceMappingURL=index.d.ts.map