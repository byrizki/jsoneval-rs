"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.multiply = exports.default = exports.ReturnFormat = exports.JSONEval = void 0;
exports.useJSONEval = useJSONEval;
var _react = _interopRequireDefault(require("react"));
var _reactNative = require("react-native");
var _jsonWithBigint = require("json-with-bigint");
function _interopRequireDefault(e) { return e && e.__esModule ? e : { default: e }; }
const LINKING_ERROR = `The package '@json-eval-rs/react-native' doesn't seem to be linked. Make sure: \n\n` + _reactNative.Platform.select({
  ios: "- You have run 'pod install'\n",
  default: ''
}) + '- You rebuilt the app after installing the package\n' + '- You are not using Expo managed workflow\n';
const JsonEvalRs = _reactNative.NativeModules.JsonEvalRs ? _reactNative.NativeModules.JsonEvalRs : new Proxy({}, {
  get() {
    throw new Error(LINKING_ERROR);
  }
});

/**
 * Item for get schema value array results
 */
/**
 * Return format for path-based methods
 */
let ReturnFormat = exports.ReturnFormat = /*#__PURE__*/function (ReturnFormat) {
  /** Nested object preserving the path hierarchy (default) */
  ReturnFormat[ReturnFormat["Nested"] = 0] = "Nested";
  /** Flat object with dotted keys */
  ReturnFormat[ReturnFormat["Flat"] = 1] = "Flat";
  /** Array of values in the order of requested paths */
  ReturnFormat[ReturnFormat["Array"] = 2] = "Array";
  return ReturnFormat;
}({});
/**
 * Validation error for a specific field
 */
/**
 * Result of validation operation
 */
/**
 * Dependent field change from evaluateDependents
 */
/**
 * Options for creating a JSONEval instance
 */
/**
 * Options for evaluation
 */
/**
 * Options for validation with path filtering
 */
/**
 * Cache statistics
 */
/**
 * Options for evaluating dependents
 */
/**
 * Options for evaluating a subform
 */
/**
 * Options for validating a subform
 */
/**
 * Options for evaluating dependents in a subform
 */
/**
 * Options for resolving layout in a subform
 */
/**
 * Options for getting evaluated schema from a subform
 */
/**
 * Options for getting schema value from a subform
 */
/**
 * Options for getting evaluated schema by path from a subform
 */
/**
 * Options for getting evaluated schema by multiple paths from a subform
 */
/**
 * Options for getting schema by path from a subform
 */
/**
 * Options for getting schema by multiple paths from a subform
 */
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
class JSONEval {
  disposed = false;

  /**
   * Creates a new JSON evaluator instance from a cached ParsedSchema
   * @param cacheKey - Cache key to lookup in the global ParsedSchemaCache
   * @param context - Optional context data
   * @param data - Optional initial data
   * @returns New JSONEval instance
   * @throws {Error} If schema not found in cache or creation fails
   */
  static fromCache(cacheKey, context, data) {
    const contextStr = context ? typeof context === 'string' ? context : (0, _jsonWithBigint.JSONStringify)(context) : null;
    const dataStr = data ? typeof data === 'string' ? data : (0, _jsonWithBigint.JSONStringify)(data) : null;
    const handle = JsonEvalRs.createFromCache(cacheKey, contextStr, dataStr);
    return new JSONEval({
      schema: {},
      _handle: handle
    });
  }

  /**
   * Evaluates logic expression without creating an instance
   * @param logicStr - JSON Logic expression as string or object
   * @param data - Optional data as string or object
   * @param context - Optional context as string or object
   * @returns Promise resolving to evaluation result
   */
  static async evaluateLogic(logicStr, data, context) {
    const logic = typeof logicStr === 'string' ? logicStr : (0, _jsonWithBigint.JSONStringify)(logicStr);
    const dataStr = data ? typeof data === 'string' ? data : (0, _jsonWithBigint.JSONStringify)(data) : null;
    const contextStr = context ? typeof context === 'string' ? context : (0, _jsonWithBigint.JSONStringify)(context) : null;
    const resultStr = await JsonEvalRs.evaluateLogic(logic, dataStr, contextStr);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Creates a new JSON evaluator instance
   * @param options - Configuration options with schema, context, and data
   * @throws {Error} If creation fails
   */
  constructor(options) {
    // If handle is provided (from static factory), use it directly
    if (options._handle) {
      this.handle = options._handle;
      return;
    }
    const {
      schema,
      context,
      data
    } = options;
    try {
      const schemaStr = typeof schema === 'string' ? schema : (0, _jsonWithBigint.JSONStringify)(schema);
      const contextStr = context ? typeof context === 'string' ? context : (0, _jsonWithBigint.JSONStringify)(context) : null;
      const dataStr = data ? typeof data === 'string' ? data : (0, _jsonWithBigint.JSONStringify)(data) : null;
      this.handle = JsonEvalRs.create(schemaStr, contextStr, dataStr);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to create JSONEval instance: ${errorMessage}`);
    }
  }
  throwIfDisposed() {
    if (this.disposed) {
      throw new Error('JSONEval instance has been disposed');
    }
  }

  /**
   * Convert value to JSON string
   * Performance note: If you have a pre-serialized JSON string, pass it directly
   * instead of an object to avoid the JSONStringify overhead
   */
  toJsonString(value) {
    return typeof value === 'string' ? value : (0, _jsonWithBigint.JSONStringify)(value);
  }

  /**
   * Cancel any running evaluation
   * The generic auto-cancellation on new evaluation will still work,
   * but this allows manual cancellation.
   */
  async cancel() {
    this.throwIfDisposed();
    JsonEvalRs.cancel(this.handle);
  }

  /**
   * Evaluate schema with provided data
   * @param options - Evaluation options
   * @returns Promise resolving to evaluated schema object
   * @throws {Error} If evaluation fails
   */
  async evaluate(options) {
    this.throwIfDisposed();
    try {
      const dataStr = this.toJsonString(options.data);
      const contextStr = options.context ? this.toJsonString(options.context) : null;
      const pathsJson = options.paths ? typeof options.paths === 'string' ? options.paths : (0, _jsonWithBigint.JSONStringify)(options.paths) : null;
      const resultStr = await JsonEvalRs.evaluate(this.handle, dataStr, contextStr, pathsJson);
      return (0, _jsonWithBigint.JSONParse)(resultStr);
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
  async validate(options) {
    this.throwIfDisposed();
    try {
      const dataStr = this.toJsonString(options.data);
      const contextStr = options.context ? this.toJsonString(options.context) : null;
      const resultStr = await JsonEvalRs.validate(this.handle, dataStr, contextStr);
      return (0, _jsonWithBigint.JSONParse)(resultStr);
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
  async evaluateDependents(options) {
    this.throwIfDisposed();
    try {
      const {
        changedPaths,
        data,
        context,
        reEvaluate = true
      } = options;
      const changedPathsJson = typeof changedPaths === 'string' ? changedPaths : (0, _jsonWithBigint.JSONStringify)(changedPaths);
      const dataStr = data ? this.toJsonString(data) : null;
      const contextStr = context ? this.toJsonString(context) : null;
      const resultStr = await JsonEvalRs.evaluateDependents(this.handle, changedPathsJson, dataStr, contextStr, reEvaluate);
      return (0, _jsonWithBigint.JSONParse)(resultStr);
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
  async getEvaluatedSchema(skipLayout = false) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchema(this.handle, skipLayout);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get all schema values (evaluations ending with .value)
   * @returns Promise resolving to map of path -> value
   * @throws {Error} If operation fails
   */
  async getSchemaValue() {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaValue(this.handle);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get all schema values as array of path-value pairs
   * Returns [{path: "", value: ""}, ...]
   * @returns Promise resolving to array of SchemaValueItem objects
   * @throws {Error} If operation fails
   */
  async getSchemaValueArray() {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaValueArray(this.handle);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get all schema values as object with dotted path keys
   * Returns {path: value, ...}
   * @returns Promise resolving to flat object with dotted paths as keys
   * @throws {Error} If operation fails
   */
  async getSchemaValueObject() {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaValueObject(this.handle);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get the evaluated schema without $params field
   * @param skipLayout - Whether to skip layout resolution (default: false)
   * @returns Promise resolving to evaluated schema object
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaWithoutParams(skipLayout = false) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchemaWithoutParams(this.handle, skipLayout);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get a value from the evaluated schema using dotted path notation
   * @param path - Dotted path to the value (e.g., "properties.field.value")
   * @param skipLayout - Whether to skip layout resolution
   * @returns Promise resolving to the value at the path, or null if not found
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPath(path, skipLayout = false) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchemaByPath(this.handle, path, skipLayout);
    return resultStr ? (0, _jsonWithBigint.JSONParse)(resultStr) : null;
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
  async getEvaluatedSchemaByPaths(paths, skipLayout = false, format = ReturnFormat.Nested) {
    this.throwIfDisposed();
    const pathsJson = typeof paths === 'string' ? paths : (0, _jsonWithBigint.JSONStringify)(paths);
    const resultStr = await JsonEvalRs.getEvaluatedSchemaByPaths(this.handle, pathsJson, skipLayout, format);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get a value from the schema using dotted path notation
   * @param path - Dotted path to the value (e.g., "properties.field.value")
   * @returns Promise resolving to the value at the path, or null if not found
   * @throws {Error} If operation fails
   */
  async getSchemaByPath(path) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaByPath(this.handle, path);
    return resultStr ? (0, _jsonWithBigint.JSONParse)(resultStr) : null;
  }

  /**
   * Get values from the schema using multiple dotted path notations
   * Returns data in the specified format (skips paths that are not found)
   * @param paths - Array of dotted paths to retrieve
   * @param format - Return format (Nested, Flat, or Array)
   * @returns Promise resolving to data in the specified format
   * @throws {Error} If operation fails
   */
  async getSchemaByPaths(paths, format = ReturnFormat.Nested) {
    this.throwIfDisposed();
    const pathsJson = typeof paths === 'string' ? paths : (0, _jsonWithBigint.JSONStringify)(paths);
    const resultStr = await JsonEvalRs.getSchemaByPaths(this.handle, pathsJson, format);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Reload schema with new data
   * @param options - Configuration options with new schema, context, and data
   * @throws {Error} If reload fails
   */
  async reloadSchema(options) {
    this.throwIfDisposed();
    try {
      const {
        schema,
        context,
        data
      } = options;
      const schemaStr = typeof schema === 'string' ? schema : (0, _jsonWithBigint.JSONStringify)(schema);
      const contextStr = context ? typeof context === 'string' ? context : (0, _jsonWithBigint.JSONStringify)(context) : null;
      const dataStr = data ? typeof data === 'string' ? data : (0, _jsonWithBigint.JSONStringify)(data) : null;
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
  async reloadSchemaMsgpack(schemaMsgpack, context, data) {
    this.throwIfDisposed();
    try {
      // Convert Uint8Array to number array if needed
      const msgpackArray = schemaMsgpack instanceof Uint8Array ? Array.from(schemaMsgpack) : schemaMsgpack;
      const contextStr = context ? typeof context === 'string' ? context : (0, _jsonWithBigint.JSONStringify)(context) : null;
      const dataStr = data ? typeof data === 'string' ? data : (0, _jsonWithBigint.JSONStringify)(data) : null;
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
  async reloadSchemaFromCache(cacheKey, context, data) {
    this.throwIfDisposed();
    try {
      const contextStr = context ? typeof context === 'string' ? context : (0, _jsonWithBigint.JSONStringify)(context) : null;
      const dataStr = data ? typeof data === 'string' ? data : (0, _jsonWithBigint.JSONStringify)(data) : null;
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
  async cacheStats() {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.cacheStats(this.handle);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Clear the evaluation cache
   * @returns Promise that resolves when cache is cleared
   * @throws {Error} If operation fails
   */
  async clearCache() {
    this.throwIfDisposed();
    await JsonEvalRs.clearCache(this.handle);
  }

  /**
   * Get the number of cached entries
   * @returns Promise resolving to number of cached entries
   * @throws {Error} If operation fails
   */
  async cacheLen() {
    this.throwIfDisposed();
    return await JsonEvalRs.cacheLen(this.handle);
  }

  /**
   * Enable evaluation caching
   * Useful for reusing JSONEval instances with different data
   * @returns Promise that resolves when cache is enabled
   * @throws {Error} If operation fails
   */
  async enableCache() {
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
  async disableCache() {
    this.throwIfDisposed();
    await JsonEvalRs.disableCache(this.handle);
  }

  /**
   * Check if evaluation caching is enabled
   * @returns Boolean indicating if caching is enabled
   * @throws {Error} If operation fails
   */
  isCacheEnabled() {
    this.throwIfDisposed();
    return JsonEvalRs.isCacheEnabled(this.handle);
  }

  /**
   * Resolve layout with optional evaluation
   * @param evaluate - If true, runs evaluation before resolving layout (default: false)
   * @returns Promise that resolves when layout resolution is complete
   * @throws {Error} If operation fails
   */
  async resolveLayout(evaluate = false) {
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
  async setTimezoneOffset(offsetMinutes) {
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
  async compileAndRunLogic(logicStr, data, context) {
    this.throwIfDisposed();
    const logic = this.toJsonString(logicStr);
    const dataStr = data ? this.toJsonString(data) : null;
    const contextStr = context ? this.toJsonString(context) : null;
    const resultStr = await JsonEvalRs.compileAndRunLogic(this.handle, logic, dataStr, contextStr);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Compile JSON logic and return a global ID
   * @param logicStr - JSON logic expression as a string or object
   * @returns Promise resolving to the compiled logic ID
   * @throws {Error} If compilation fails
   */
  async compileLogic(logicStr) {
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
  async runLogic(logicId, data, context) {
    this.throwIfDisposed();
    const dataStr = data ? this.toJsonString(data) : null;
    const contextStr = context ? this.toJsonString(context) : null;
    const resultStr = await JsonEvalRs.runLogic(this.handle, logicId, dataStr, contextStr);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Validate data against schema rules with optional path filtering
   * @param options - Validation options with optional path filtering
   * @returns Promise resolving to ValidationResult
   * @throws {Error} If validation operation fails
   */
  async validatePaths(options) {
    this.throwIfDisposed();
    const dataStr = this.toJsonString(options.data);
    const contextStr = options.context ? this.toJsonString(options.context) : null;
    const paths = options.paths || null;
    const resultStr = await JsonEvalRs.validatePaths(this.handle, dataStr, contextStr, paths);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
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
  async evaluateSubform(options) {
    this.throwIfDisposed();
    const dataStr = this.toJsonString(options.data);
    const contextStr = options.context ? this.toJsonString(options.context) : null;
    return JsonEvalRs.evaluateSubform(this.handle, options.subformPath, dataStr, contextStr, options.paths);
  }

  /**
   * Validate subform data against its schema rules
   * @param options - Validation options including subform path and data
   * @returns Promise resolving to ValidationResult
   * @throws {Error} If validation fails
   */
  async validateSubform(options) {
    this.throwIfDisposed();
    const dataStr = this.toJsonString(options.data);
    const contextStr = options.context ? this.toJsonString(options.context) : null;
    const resultStr = await JsonEvalRs.validateSubform(this.handle, options.subformPath, dataStr, contextStr);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Evaluate dependents in a subform when fields change
   * @param options - Options including subform path, changed paths array, and optional data
   * @returns Promise resolving to dependent evaluation results
   * @throws {Error} If evaluation fails
   */
  async evaluateDependentsSubform(options) {
    this.throwIfDisposed();
    const dataStr = options.data ? this.toJsonString(options.data) : null;
    const contextStr = options.context ? this.toJsonString(options.context) : null;

    // For now, pass the first path since native bridge expects single path (wraps internally)
    const changedPath = options.changedPaths[0] || '';
    const resultStr = await JsonEvalRs.evaluateDependentsSubform(this.handle, options.subformPath, changedPath, dataStr, contextStr, options.reEvaluate ?? true);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Resolve layout for subform
   * @param options - Options including subform path and evaluate flag
   * @returns Promise that resolves when layout is resolved
   * @throws {Error} If layout resolution fails
   */
  async resolveLayoutSubform(options) {
    this.throwIfDisposed();
    return JsonEvalRs.resolveLayoutSubform(this.handle, options.subformPath, options.evaluate || false);
  }

  /**
   * Get evaluated schema from subform
   * @param options - Options including subform path and resolveLayout flag
   * @returns Promise resolving to evaluated schema
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaSubform(options) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchemaSubform(this.handle, options.subformPath, options.resolveLayout || false);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get schema value from subform (all .value fields)
   * @param options - Options including subform path
   * @returns Promise resolving to schema values
   * @throws {Error} If operation fails
   */
  async getSchemaValueSubform(options) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaValueSubform(this.handle, options.subformPath);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get schema values from subform as a flat array of path-value pairs.
   * Returns an array like `[{path: "field.sub", value: 123}, ...]`.
   * @param options - Options including subform path
   * @returns Promise resolving to array of SchemaValueItem objects
   * @throws {Error} If operation fails
   */
  async getSchemaValueArraySubform(options) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaValueArraySubform(this.handle, options.subformPath);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get schema values from subform as a flat object with dotted path keys.
   * Returns an object like `{"field.sub": 123, ...}`.
   * @param options - Options including subform path
   * @returns Promise resolving to flat object with dotted paths
   * @throws {Error} If operation fails
   */
  async getSchemaValueObjectSubform(options) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaValueObjectSubform(this.handle, options.subformPath);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get evaluated schema without $params from subform
   * @param options - Options including subform path and resolveLayout flag
   * @returns Promise resolving to evaluated schema without $params
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaWithoutParamsSubform(options) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchemaWithoutParamsSubform(this.handle, options.subformPath, options.resolveLayout || false);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get evaluated schema by specific path from subform
   * @param options - Options including subform path, schema path, and skipLayout flag
   * @returns Promise resolving to value at path or null if not found
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPathSubform(options) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchemaByPathSubform(this.handle, options.subformPath, options.schemaPath, options.skipLayout || false);
    return resultStr ? (0, _jsonWithBigint.JSONParse)(resultStr) : null;
  }

  /**
   * Get evaluated schema by multiple paths from subform
   * Returns data in the specified format (skips paths that are not found)
   * @param options - Options including subform path, array of schema paths, skipLayout flag, and format
   * @returns Promise resolving to data in the specified format
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPathsSubform(options) {
    this.throwIfDisposed();
    const pathsJson = typeof options.schemaPaths === 'string' ? options.schemaPaths : (0, _jsonWithBigint.JSONStringify)(options.schemaPaths);
    const resultStr = await JsonEvalRs.getEvaluatedSchemaByPathsSubform(this.handle, options.subformPath, pathsJson, options.skipLayout || false, options.format !== undefined ? options.format : ReturnFormat.Nested);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get list of available subform paths
   * @returns Promise resolving to array of subform paths
   * @throws {Error} If operation fails
   */
  async getSubformPaths() {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSubformPaths(this.handle);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Get schema value by specific path from subform
   * @param options - Options including subform path and schema path
   * @returns Promise resolving to value at path or null if not found
   * @throws {Error} If operation fails
   */
  async getSchemaByPathSubform(options) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaByPathSubform(this.handle, options.subformPath, options.schemaPath);
    return resultStr ? (0, _jsonWithBigint.JSONParse)(resultStr) : null;
  }

  /**
   * Get schema values by multiple paths from subform
   * Returns data in the specified format (skips paths that are not found)
   * @param options - Options including subform path, array of schema paths, and format
   * @returns Promise resolving to data in the specified format
   * @throws {Error} If operation fails
   */
  async getSchemaByPathsSubform(options) {
    this.throwIfDisposed();
    const pathsJson = typeof options.schemaPaths === 'string' ? options.schemaPaths : (0, _jsonWithBigint.JSONStringify)(options.schemaPaths);
    const resultStr = await JsonEvalRs.getSchemaByPathsSubform(this.handle, options.subformPath, pathsJson, options.format !== undefined ? options.format : ReturnFormat.Nested);
    return (0, _jsonWithBigint.JSONParse)(resultStr);
  }

  /**
   * Check if a subform exists at the given path
   * @param subformPath - Path to check
   * @returns Promise resolving to true if subform exists, false otherwise
   * @throws {Error} If operation fails
   */
  async hasSubform(subformPath) {
    this.throwIfDisposed();
    return JsonEvalRs.hasSubform(this.handle, subformPath);
  }

  /**
   * Dispose of the native resources
   * Must be called when done using the instance
   * @returns Promise that resolves when disposal is complete
   */
  async dispose() {
    if (this.disposed) return;
    await JsonEvalRs.dispose(this.handle);
    this.disposed = true;
  }

  /**
   * Get the library version
   * @returns Promise resolving to version string
   */
  static async version() {
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
exports.JSONEval = JSONEval;
function useJSONEval(options) {
  const [evalInstance, setEvalInstance] = _react.default.useState(null);
  _react.default.useEffect(() => {
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
var _default = exports.default = JSONEval; // For backwards compatibility
const multiply = (a, b) => {
  return JsonEvalRs.multiply(a, b);
};
exports.multiply = multiply;
//# sourceMappingURL=index.js.map