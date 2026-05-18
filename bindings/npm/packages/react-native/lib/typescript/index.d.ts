import { type JSONEvalOptions, type EvaluateOptions, type EvaluateDependentsOptions, type LayoutOverlayEntry, type EvaluateSubformOptions, type ValidateSubformOptions, type EvaluateDependentsSubformOptions, type ResolveLayoutSubformOptions, type GetEvaluatedSchemaSubformOptions, type GetSchemaValueSubformOptions, type GetEvaluatedSchemaByPathSubformOptions, type GetEvaluatedSchemaByPathsSubformOptions, type GetSchemaByPathSubformOptions, type GetSchemaByPathsSubformOptions, type ValidationResult, type DependentChange, type SchemaValueItem, type ValidatePathsOptions, ReturnFormat } from '@json-eval-rs/common';
export { ReturnFormat } from '@json-eval-rs/common';
export type { LayoutOverlayEntry, SchemaValueItem, ValidationResult, DependentChange, ValidationError, JSONEvalOptions, EvaluateOptions, EvaluateDependentsOptions, EvaluateSubformOptions, ValidateSubformOptions, EvaluateDependentsSubformOptions, ResolveLayoutSubformOptions, GetEvaluatedSchemaSubformOptions, GetSchemaValueSubformOptions, GetEvaluatedSchemaByPathSubformOptions, GetEvaluatedSchemaByPathsSubformOptions, GetSchemaByPathSubformOptions, GetSchemaByPathsSubformOptions, } from '@json-eval-rs/common';
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
 * if (validation.has_error) {
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
     * Creates a new JSON evaluator instance from a MessagePack-encoded schema
     * @param schemaMsgpack - MessagePack-encoded schema bytes (Uint8Array or number array)
     * @param context - Optional context data
     * @param data - Optional initial data
     * @returns New JSONEval instance
     * @throws {Error} If creation fails
     */
    static fromMsgpack(schemaMsgpack: Uint8Array | number[], context?: string | object | null, data?: string | object | null): JSONEval;
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
     * Internal helper to call native methods with JSI fallback.
     * Handles synchronous JSI calls and asynchronous bridge calls.
     */
    private _callNative;
    /**
     * Internal helper to call native methods and parse JSON result.
     */
    private _callNativeJson;
    /**
     * Internal helper to call native methods and parse JSON result, or return null if empty.
     */
    private _callNativeJsonOrNull;
    /**
     * Cancel any running evaluation
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
     * Evaluate schema with provided data (only updates internal state)
     * @param options - Evaluation options
     * @returns Promise that resolves when evaluation is complete
     * @throws {Error} If evaluation fails
     */
    evaluateOnly(options: EvaluateOptions): Promise<void>;
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
     * Re-evaluate fields that depend on a changed path (returns JSON string)
     * @param options - Dependent evaluation options
     * @returns Promise resolving to JSON string of dependent field changes
     * @throws {Error} If evaluation fails
     */
    evaluateDependentsString(options: EvaluateDependentsOptions): Promise<string>;
    /**
     * Get the evaluated schema (compact, without $layout resolution)
     * @returns Promise resolving to evaluated schema object
     * @throws {Error} If operation fails
     */
    getEvaluatedSchema(): Promise<any>;
    /**
     * Get resolved layout overlay entries
     * @returns Promise resolving to array of overlay entries
     * @throws {Error} If operation fails
     */
    getResolvedLayout(): Promise<LayoutOverlayEntry[]>;
    /**
     * Get evaluated schema with layout fully resolved
     * @returns Promise resolving to evaluated schema with layout applied
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaResolved(): Promise<any>;
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
     * Get the evaluated schema without $params field (compact)
     * @returns Promise resolving to evaluated schema object
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaWithoutParams(): Promise<any>;
    /**
     * Get a value from the evaluated schema using dotted path notation (compact)
     * @param path - Dotted path to the value (e.g., "properties.field.value")
     * @returns Promise resolving to the value at the path, or null if not found
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaByPath(path: string): Promise<any | null>;
    /**
     * Get values from the evaluated schema using multiple dotted path notations (compact)
     * Returns data in the specified format (skips paths that are not found)
     * @param paths - Array of dotted paths to retrieve
     * @param format - Return format (Nested, Flat, or Array)
     * @returns Promise resolving to data in the specified format
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaByPaths(paths: string[], format?: ReturnFormat): Promise<any>;
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
     * Evaluate and return the options for a specific field on demand.
     *
     * The field is identified by `fieldPath`, which can be:
     * - Dotted notation: `"form.occupation"`
     * - JSON pointer: `"/properties/form/properties/occupation"`
     * - Schema ref: `"#/properties/form/properties/occupation"`
     *
     * @param fieldPath - Field path identifying which field's options to retrieve
     * @returns Promise resolving to the resolved options (array or URL string), or null if none
     * @throws {Error} If operation fails
     */
    getFieldOptions(fieldPath: string): Promise<any | null>;
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
     * Resolve layout with optional evaluation
     * @param evaluate - If true, runs evaluation before resolving layout (default: false)
     * @returns Promise resolving to array of layout overlay entries
     * @throws {Error} If operation fails
     */
    resolveLayout(evaluate?: boolean): Promise<LayoutOverlayEntry[]>;
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
     * Validate data against schema rules with optional path filtering
     * (alias for validatePaths in RN)
     * @param options - Validation options with optional path filtering
     * @returns Promise resolving to ValidationResult
     * @throws {Error} If validation operation fails
     */
    validatePathsOnly(options: ValidatePathsOptions): Promise<any>;
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
     * Evaluate dependents in a subform when fields change (returns JSON string)
     * @param options - Options including subform path, changed paths array, and optional data
     * @returns Promise resolving to JSON string of dependent evaluation results
     * @throws {Error} If evaluation fails
     */
    evaluateDependentsSubformString(options: EvaluateDependentsSubformOptions): Promise<string>;
    /**
     * Resolve layout for subform
     * @param options - Options including subform path and evaluate flag
     * @returns Promise resolving to array of layout overlay entries
     * @throws {Error} If layout resolution fails
     */
    resolveLayoutSubform(options: ResolveLayoutSubformOptions): Promise<LayoutOverlayEntry[]>;
    /**
     * Get resolved layout overlay entries for subform
     * @param options - Options including subform path
     * @returns Promise resolving to array of layout overlay entries
     * @throws {Error} If operation fails
     */
    getResolvedLayoutSubform(options: GetEvaluatedSchemaSubformOptions): Promise<LayoutOverlayEntry[]>;
    /**
     * Get evaluated schema with layout fully resolved for subform
     * @param options - Options including subform path
     * @returns Promise resolving to evaluated schema with layout applied
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaResolvedSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any>;
    /**
     * Get evaluated schema from subform (compact, without $layout resolution)
     * @param options - Options including subform path
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
     * Get evaluated schema without $params from subform (compact)
     * @param options - Options including subform path
     * @returns Promise resolving to evaluated schema without $params
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaWithoutParamsSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any>;
    /**
     * Get evaluated schema by specific path from subform (compact)
     * @param options - Options including subform path and schema path
     * @returns Promise resolving to value at path or null if not found
     * @throws {Error} If operation fails
     */
    getEvaluatedSchemaByPathSubform(options: GetEvaluatedSchemaByPathSubformOptions): Promise<any | null>;
    /**
     * Get evaluated schema by multiple paths from subform (compact)
     * Returns data in the specified format (skips paths that are not found)
     * @param options - Options including subform path, array of schema paths, and format
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