"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.JSONEval = void 0;
Object.defineProperty(exports, "ReturnFormat", {
  enumerable: true,
  get: function () {
    return _common.ReturnFormat;
  }
});
exports.multiply = exports.default = void 0;
exports.useJSONEval = useJSONEval;
var _react = _interopRequireDefault(require("react"));
var _reactNative = require("react-native");
var _common = require("@json-eval-rs/common");
var _jsiBridge = require("./jsi-bridge");
function _interopRequireDefault(e) { return e && e.__esModule ? e : { default: e }; }
// Re-export shared types for downstream consumers

const LINKING_ERROR = `The package '@json-eval-rs/react-native' doesn't seem to be linked. Make sure: \n\n` + _reactNative.Platform.select({
  ios: "- You have run 'pod install'\n",
  default: ''
}) + '- You rebuilt the app after installing the package\n' + '- You are not using Expo managed workflow\n';
const JsonEvalRs = _reactNative.NativeModules.JsonEvalRs ? _reactNative.NativeModules.JsonEvalRs : new Proxy({}, {
  get() {
    throw new Error(LINKING_ERROR);
  }
});

// JSI bootstrap: install sync host object at module load.
let _jsi = null;
try {
  if (typeof JsonEvalRs.installJSI === 'function') {
    const installed = JsonEvalRs.installJSI();
    if (installed) {
      _jsi = (0, _jsiBridge.getJSIGlobal)();
      console.log('JSONEval is using JSI 🎉');
    }
  }
} catch {
  // JSI unavailable — fall back to bridge
}
const useJSI = _jsi !== null;

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
    var _jsi2;
    const contextStr = (0, _common.stringifyOrNull)(context);
    const dataStr = (0, _common.stringifyOrNull)(data);
    if (useJSI && (_jsi2 = _jsi) !== null && _jsi2 !== void 0 && _jsi2.createFromCache) {
      const handle = _jsi.createFromCache(cacheKey, contextStr, dataStr);
      return new JSONEval({
        schema: {},
        _handle: handle
      });
    }
    const handle = JsonEvalRs.createFromCache(cacheKey, contextStr, dataStr);
    return new JSONEval({
      schema: {},
      _handle: handle
    });
  }

  /**
   * Creates a new JSON evaluator instance from a MessagePack-encoded schema
   * @param schemaMsgpack - MessagePack-encoded schema bytes (Uint8Array or number array)
   * @param context - Optional context data
   * @param data - Optional initial data
   * @returns New JSONEval instance
   * @throws {Error} If creation fails
   */
  static fromMsgpack(schemaMsgpack, context, data) {
    const contextStr = (0, _common.stringifyOrNull)(context);
    const dataStr = (0, _common.stringifyOrNull)(data);
    try {
      var _jsi3;
      // Convert Uint8Array to number array if needed
      const msgpackArray = schemaMsgpack instanceof Uint8Array ? Array.from(schemaMsgpack) : schemaMsgpack;
      let handle;
      if (useJSI && (_jsi3 = _jsi) !== null && _jsi3 !== void 0 && _jsi3.createFromMsgpack) {
        handle = _jsi.createFromMsgpack(msgpackArray, contextStr, dataStr);
      } else {
        handle = JsonEvalRs.createFromMsgpack(msgpackArray, contextStr, dataStr);
      }
      return new JSONEval({
        schema: {},
        _handle: handle
      });
    } catch (error) {
      throw new Error(`Failed to create JSONEval instance from MessagePack: ${(0, _common.extractErrorMessage)(error)}`);
    }
  }

  /**
   * Evaluates logic expression without creating an instance
   * @param logicStr - JSON Logic expression as string or object
   * @param data - Optional data as string or object
   * @param context - Optional context as string or object
   * @returns Promise resolving to evaluation result
   */
  static async evaluateLogic(logicStr, data, context) {
    const logic = (0, _common.stringifyValue)(logicStr);
    const dataStr = (0, _common.stringifyOrNull)(data);
    const contextStr = (0, _common.stringifyOrNull)(context);
    if (useJSI && _jsi.evaluateLogic) {
      const resultStr = _jsi.evaluateLogic(logic, dataStr, contextStr);
      return (0, _common.parseValue)(resultStr);
    }
    const resultStr = await JsonEvalRs.evaluateLogic(logic, dataStr, contextStr);
    return (0, _common.parseValue)(resultStr);
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
      const schemaStr = (0, _common.stringifyValue)(schema);
      const contextStr = (0, _common.stringifyOrNull)(context);
      const dataStr = (0, _common.stringifyOrNull)(data);
      if (useJSI) {
        this.handle = _jsi.create(schemaStr, contextStr, dataStr);
      } else {
        this.handle = JsonEvalRs.create(schemaStr, contextStr, dataStr);
      }
    } catch (error) {
      throw new Error(`Failed to create JSONEval instance: ${(0, _common.extractErrorMessage)(error)}`);
    }
  }
  throwIfDisposed() {
    if (this.disposed) {
      throw new Error('JSONEval instance has been disposed');
    }
  }

  /**
   * Internal helper to call native methods with JSI fallback.
   * Handles synchronous JSI calls and asynchronous bridge calls.
   */
  async _callNative(methodName, ...args) {
    if (useJSI && _jsi[methodName]) {
      return _jsi[methodName](this.handle, ...args);
    }
    return await JsonEvalRs[methodName](this.handle, ...args);
  }

  /**
   * Internal helper to call native methods and parse JSON result.
   */
  async _callNativeJson(methodName, ...args) {
    const result = await this._callNative(methodName, ...args);
    if (!result) return null;

    // If it's an ArrayBuffer (Zero-Copy JSI result), decode it first
    if (result instanceof ArrayBuffer) {
      if (result.byteLength === 0) return null;
      const jsonStr = _jsi.decodeArrayBuffer(result);
      return (0, _common.parseValue)(jsonStr);
    }
    return typeof result === 'string' ? (0, _common.parseValue)(result) : result;
  }

  /**
   * Internal helper to call native methods and parse JSON result, or return null if empty.
   */
  async _callNativeJsonOrNull(methodName, ...args) {
    const result = await this._callNative(methodName, ...args);
    if (!result) return null;
    if (result instanceof ArrayBuffer) {
      if (result.byteLength === 0) return null;
      const jsonStr = _jsi.decodeArrayBuffer(result);
      return jsonStr === 'null' || jsonStr === '' ? null : (0, _common.parseValue)(jsonStr);
    }
    return result ? (0, _common.parseValue)(result) : null;
  }

  /**
   * Cancel any running evaluation
   */
  async cancel() {
    this.throwIfDisposed();
    await this._callNative('cancel');
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
      const dataStr = (0, _common.stringifyValue)(options.data);
      const contextStr = options.context ? (0, _common.stringifyValue)(options.context) : null;
      const pathsJson = options.paths ? typeof options.paths === 'string' ? options.paths : JSON.stringify(options.paths) : null;
      return await this._callNativeJson('evaluate', dataStr, contextStr, pathsJson);
    } catch (error) {
      throw new Error(`Evaluation failed: ${(0, _common.extractErrorMessage)(error)}`);
    }
  }

  /**
   * Evaluate schema with provided data (only updates internal state)
   * @param options - Evaluation options
   * @returns Promise that resolves when evaluation is complete
   * @throws {Error} If evaluation fails
   */
  async evaluateOnly(options) {
    this.throwIfDisposed();
    try {
      const dataStr = (0, _common.stringifyValue)(options.data);
      const contextStr = options.context ? (0, _common.stringifyValue)(options.context) : null;
      const pathsJson = options.paths ? typeof options.paths === 'string' ? options.paths : JSON.stringify(options.paths) : null;
      await this._callNative('evaluateOnly', dataStr, contextStr, pathsJson);
    } catch (error) {
      throw new Error(`Evaluation failed: ${(0, _common.extractErrorMessage)(error)}`);
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
      const dataStr = (0, _common.stringifyValue)(options.data);
      const contextStr = options.context ? (0, _common.stringifyValue)(options.context) : null;
      return await this._callNativeJson('validate', dataStr, contextStr);
    } catch (error) {
      throw new Error(`Validation failed: ${(0, _common.extractErrorMessage)(error)}`);
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
        reEvaluate = true,
        includeSubforms = true
      } = options;
      const changedPathsJson = typeof changedPaths === 'string' ? changedPaths : JSON.stringify(changedPaths);
      const dataStr = data ? (0, _common.stringifyValue)(data) : null;
      const contextStr = context ? (0, _common.stringifyValue)(context) : null;
      return await this._callNativeJson('evaluateDependents', changedPathsJson, dataStr, contextStr, reEvaluate, includeSubforms);
    } catch (error) {
      throw new Error(`Dependent evaluation failed: ${(0, _common.extractErrorMessage)(error)}`);
    }
  }

  /**
   * Re-evaluate fields that depend on a changed path (returns JSON string)
   * @param options - Dependent evaluation options
   * @returns Promise resolving to JSON string of dependent field changes
   * @throws {Error} If evaluation fails
   */
  async evaluateDependentsString(options) {
    this.throwIfDisposed();
    try {
      const {
        changedPaths,
        data,
        context,
        reEvaluate = true,
        includeSubforms = true
      } = options;
      const changedPathsJson = typeof changedPaths === 'string' ? changedPaths : JSON.stringify(changedPaths);
      const dataStr = data ? (0, _common.stringifyValue)(data) : null;
      const contextStr = context ? (0, _common.stringifyValue)(context) : null;
      return await this._callNative('evaluateDependents', changedPathsJson, dataStr, contextStr, reEvaluate, includeSubforms);
    } catch (error) {
      throw new Error(`Dependent evaluation failed: ${(0, _common.extractErrorMessage)(error)}`);
    }
  }

  /**
   * Get the evaluated schema (compact, without $layout resolution)
   * @returns Promise resolving to evaluated schema object
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchema() {
    this.throwIfDisposed();
    return await this._callNativeJson('getEvaluatedSchema');
  }

  /**
   * Get resolved layout overlay entries
   * @returns Promise resolving to array of overlay entries
   * @throws {Error} If operation fails
   */
  async getResolvedLayout() {
    this.throwIfDisposed();
    return await this._callNativeJson('getResolvedLayout');
  }

  /**
   * Get evaluated schema with layout fully resolved
   * @returns Promise resolving to evaluated schema with layout applied
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaResolved() {
    this.throwIfDisposed();
    return await this._callNativeJson('getEvaluatedSchemaResolved');
  }

  /**
   * Get all schema values (evaluations ending with .value)
   * @returns Promise resolving to map of path -> value
   * @throws {Error} If operation fails
   */
  async getSchemaValue() {
    this.throwIfDisposed();
    return await this._callNativeJson('getSchemaValue');
  }

  /**
   * Get all schema values as array of path-value pairs
   * Returns [{path: "", value: ""}, ...]
   * @returns Promise resolving to array of SchemaValueItem objects
   * @throws {Error} If operation fails
   */
  async getSchemaValueArray() {
    this.throwIfDisposed();
    return await this._callNativeJson('getSchemaValueArray');
  }

  /**
   * Get all schema values as object with dotted path keys
   * Returns {path: value, ...}
   * @returns Promise resolving to flat object with dotted paths as keys
   * @throws {Error} If operation fails
   */
  async getSchemaValueObject() {
    this.throwIfDisposed();
    return await this._callNativeJson('getSchemaValueObject');
  }

  /**
   * Get the evaluated schema without $params field (compact)
   * @returns Promise resolving to evaluated schema object
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaWithoutParams() {
    this.throwIfDisposed();
    return await this._callNativeJson('getEvaluatedSchemaWithoutParams');
  }

  /**
   * Get a value from the evaluated schema using dotted path notation (compact)
   * @param path - Dotted path to the value (e.g., "properties.field.value")
   * @returns Promise resolving to the value at the path, or null if not found
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPath(path) {
    this.throwIfDisposed();
    return await this._callNativeJsonOrNull('getEvaluatedSchemaByPath', path);
  }

  /**
   * Get values from the evaluated schema using multiple dotted path notations (compact)
   * Returns data in the specified format (skips paths that are not found)
   * @param paths - Array of dotted paths to retrieve
   * @param format - Return format (Nested, Flat, or Array)
   * @returns Promise resolving to data in the specified format
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPaths(paths, format = _common.ReturnFormat.Nested) {
    this.throwIfDisposed();
    const pathsJson = typeof paths === 'string' ? paths : JSON.stringify(paths);
    return await this._callNativeJson('getEvaluatedSchemaByPaths', pathsJson, format);
  }

  /**
   * Get a value from the schema using dotted path notation
   * @param path - Dotted path to the value (e.g., "properties.field.value")
   * @returns Promise resolving to the value at the path, or null if not found
   * @throws {Error} If operation fails
   */
  async getSchemaByPath(path) {
    this.throwIfDisposed();
    return await this._callNativeJsonOrNull('getSchemaByPath', path);
  }

  /**
   * Get values from the schema using multiple dotted path notations
   * Returns data in the specified format (skips paths that are not found)
   * @param paths - Array of dotted paths to retrieve
   * @param format - Return format (Nested, Flat, or Array)
   * @returns Promise resolving to data in the specified format
   * @throws {Error} If operation fails
   */
  async getSchemaByPaths(paths, format = _common.ReturnFormat.Nested) {
    this.throwIfDisposed();
    const pathsJson = typeof paths === 'string' ? paths : JSON.stringify(paths);
    return await this._callNativeJson('getSchemaByPaths', pathsJson, format);
  }

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
  async getFieldOptions(fieldPath) {
    this.throwIfDisposed();
    return await this._callNativeJsonOrNull('getFieldOptions', fieldPath);
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
      const schemaStr = (0, _common.stringifyValue)(schema);
      const contextStr = (0, _common.stringifyOrNull)(context);
      const dataStr = (0, _common.stringifyOrNull)(data);
      await this._callNative('reloadSchema', schemaStr, contextStr, dataStr);
    } catch (error) {
      throw new Error(`Failed to reload schema: ${(0, _common.extractErrorMessage)(error)}`);
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
      const contextStr = (0, _common.stringifyOrNull)(context);
      const dataStr = (0, _common.stringifyOrNull)(data);
      await this._callNative('reloadSchemaMsgpack', msgpackArray, contextStr, dataStr);
    } catch (error) {
      throw new Error(`Failed to reload schema from MessagePack: ${(0, _common.extractErrorMessage)(error)}`);
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
      const contextStr = (0, _common.stringifyOrNull)(context);
      const dataStr = (0, _common.stringifyOrNull)(data);
      await this._callNative('reloadSchemaFromCache', cacheKey, contextStr, dataStr);
    } catch (error) {
      throw new Error(`Failed to reload schema from cache: ${(0, _common.extractErrorMessage)(error)}`);
    }
  }

  /**
   * Resolve layout with optional evaluation
   * @param evaluate - If true, runs evaluation before resolving layout (default: false)
   * @returns Promise resolving to array of layout overlay entries
   * @throws {Error} If operation fails
   */
  async resolveLayout(evaluate = false) {
    this.throwIfDisposed();
    return await this._callNativeJson('resolveLayout', evaluate);
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
    await this._callNative('setTimezoneOffset', offsetMinutes);
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
    const logic = (0, _common.stringifyValue)(logicStr);
    const dataStr = data ? (0, _common.stringifyValue)(data) : null;
    const contextStr = context ? (0, _common.stringifyValue)(context) : null;
    return await this._callNativeJson('compileAndRunLogic', logic, dataStr, contextStr);
  }

  /**
   * Compile JSON logic and return a global ID
   * @param logicStr - JSON logic expression as a string or object
   * @returns Promise resolving to the compiled logic ID
   * @throws {Error} If compilation fails
   */
  async compileLogic(logicStr) {
    this.throwIfDisposed();
    const logic = (0, _common.stringifyValue)(logicStr);
    return await this._callNative('compileLogic', logic);
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
    const dataStr = data ? (0, _common.stringifyValue)(data) : null;
    const contextStr = context ? (0, _common.stringifyValue)(context) : null;
    return await this._callNativeJson('runLogic', logicId, dataStr, contextStr);
  }

  /**
   * Validate data against schema rules with optional path filtering
   * @param options - Validation options with optional path filtering
   * @returns Promise resolving to ValidationResult
   * @throws {Error} If validation operation fails
   */
  async validatePaths(options) {
    this.throwIfDisposed();
    const dataStr = (0, _common.stringifyValue)(options.data);
    const contextStr = options.context ? (0, _common.stringifyValue)(options.context) : null;
    const paths = options.paths || null;
    return await this._callNativeJson('validatePaths', dataStr, contextStr, paths);
  }

  /**
   * Validate data against schema rules with optional path filtering
   * (alias for validatePaths in RN)
   * @param options - Validation options with optional path filtering
   * @returns Promise resolving to ValidationResult
   * @throws {Error} If validation operation fails
   */
  async validatePathsOnly(options) {
    return this.validatePaths(options);
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
    const dataStr = (0, _common.stringifyValue)(options.data);
    const contextStr = options.context ? (0, _common.stringifyValue)(options.context) : null;
    return await this._callNative('evaluateSubform', options.subformPath, dataStr, contextStr, options.paths);
  }

  /**
   * Validate subform data against its schema rules
   * @param options - Validation options including subform path and data
   * @returns Promise resolving to ValidationResult
   * @throws {Error} If validation fails
   */
  async validateSubform(options) {
    this.throwIfDisposed();
    const dataStr = (0, _common.stringifyValue)(options.data);
    const contextStr = options.context ? (0, _common.stringifyValue)(options.context) : null;
    return await this._callNativeJson('validateSubform', options.subformPath, dataStr, contextStr);
  }

  /**
   * Evaluate dependents in a subform when fields change
   * @param options - Options including subform path, changed paths array, and optional data
   * @returns Promise resolving to dependent evaluation results
   * @throws {Error} If evaluation fails
   */
  async evaluateDependentsSubform(options) {
    this.throwIfDisposed();
    const dataStr = options.data ? (0, _common.stringifyValue)(options.data) : null;
    const contextStr = options.context ? (0, _common.stringifyValue)(options.context) : null;

    // For now, pass the first path since native bridge expects single path (wraps internally)
    const changedPath = options.changedPaths[0] || '';
    return await this._callNativeJson('evaluateDependentsSubform', options.subformPath, changedPath, dataStr, contextStr, options.reEvaluate ?? true, options.includeSubforms ?? true);
  }

  /**
   * Evaluate dependents in a subform when fields change (returns JSON string)
   * @param options - Options including subform path, changed paths array, and optional data
   * @returns Promise resolving to JSON string of dependent evaluation results
   * @throws {Error} If evaluation fails
   */
  async evaluateDependentsSubformString(options) {
    this.throwIfDisposed();
    const dataStr = options.data ? (0, _common.stringifyValue)(options.data) : null;
    const contextStr = options.context ? (0, _common.stringifyValue)(options.context) : null;
    const changedPath = options.changedPaths[0] || '';
    return await this._callNative('evaluateDependentsSubform', options.subformPath, changedPath, dataStr, contextStr, options.reEvaluate ?? true, options.includeSubforms ?? true);
  }

  /**
   * Resolve layout for subform
   * @param options - Options including subform path and evaluate flag
   * @returns Promise resolving to array of layout overlay entries
   * @throws {Error} If layout resolution fails
   */
  async resolveLayoutSubform(options) {
    this.throwIfDisposed();
    return await this._callNativeJson('resolveLayoutSubform', options.subformPath, options.evaluate || false);
  }

  /**
   * Get resolved layout overlay entries for subform
   * @param options - Options including subform path
   * @returns Promise resolving to array of layout overlay entries
   * @throws {Error} If operation fails
   */
  async getResolvedLayoutSubform(options) {
    this.throwIfDisposed();
    return await this._callNativeJson('getResolvedLayoutSubform', options.subformPath);
  }

  /**
   * Get evaluated schema with layout fully resolved for subform
   * @param options - Options including subform path
   * @returns Promise resolving to evaluated schema with layout applied
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaResolvedSubform(options) {
    this.throwIfDisposed();
    return await this._callNativeJson('getEvaluatedSchemaResolvedSubform', options.subformPath);
  }

  /**
   * Get evaluated schema from subform (compact, without $layout resolution)
   * @param options - Options including subform path
   * @returns Promise resolving to evaluated schema
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaSubform(options) {
    this.throwIfDisposed();
    return await this._callNativeJson('getEvaluatedSchemaSubform', options.subformPath);
  }

  /**
   * Get schema value from subform (all .value fields)
   * @param options - Options including subform path
   * @returns Promise resolving to schema values
   * @throws {Error} If operation fails
   */
  async getSchemaValueSubform(options) {
    this.throwIfDisposed();
    return await this._callNativeJson('getSchemaValueSubform', options.subformPath);
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
    return await this._callNativeJson('getSchemaValueArraySubform', options.subformPath);
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
    return await this._callNativeJson('getSchemaValueObjectSubform', options.subformPath);
  }

  /**
   * Get evaluated schema without $params from subform (compact)
   * @param options - Options including subform path
   * @returns Promise resolving to evaluated schema without $params
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaWithoutParamsSubform(options) {
    this.throwIfDisposed();
    return await this._callNativeJson('getEvaluatedSchemaWithoutParamsSubform', options.subformPath);
  }

  /**
   * Get evaluated schema by specific path from subform (compact)
   * @param options - Options including subform path and schema path
   * @returns Promise resolving to value at path or null if not found
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPathSubform(options) {
    this.throwIfDisposed();
    return await this._callNativeJsonOrNull('getEvaluatedSchemaByPathSubform', options.subformPath, options.schemaPath);
  }

  /**
   * Get evaluated schema by multiple paths from subform (compact)
   * Returns data in the specified format (skips paths that are not found)
   * @param options - Options including subform path, array of schema paths, and format
   * @returns Promise resolving to data in the specified format
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPathsSubform(options) {
    this.throwIfDisposed();
    const pathsJson = typeof options.schemaPaths === 'string' ? options.schemaPaths : JSON.stringify(options.schemaPaths);
    return await this._callNativeJson('getEvaluatedSchemaByPathsSubform', options.subformPath, pathsJson, options.format !== undefined ? options.format : _common.ReturnFormat.Nested);
  }

  /**
   * Get list of available subform paths
   * @returns Promise resolving to array of subform paths
   * @throws {Error} If operation fails
   */
  async getSubformPaths() {
    this.throwIfDisposed();
    return await this._callNativeJson('getSubformPaths');
  }

  /**
   * Get schema value by specific path from subform
   * @param options - Options including subform path and schema path
   * @returns Promise resolving to value at path or null if not found
   * @throws {Error} If operation fails
   */
  async getSchemaByPathSubform(options) {
    this.throwIfDisposed();
    return await this._callNativeJsonOrNull('getSchemaByPathSubform', options.subformPath, options.schemaPath);
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
    const pathsJson = typeof options.schemaPaths === 'string' ? options.schemaPaths : JSON.stringify(options.schemaPaths);
    return await this._callNativeJson('getSchemaByPathsSubform', options.subformPath, pathsJson, options.format !== undefined ? options.format : _common.ReturnFormat.Nested);
  }

  /**
   * Check if a subform exists at the given path
   * @param subformPath - Path to check
   * @returns Promise resolving to true if subform exists, false otherwise
   * @throws {Error} If operation fails
   */
  async hasSubform(subformPath) {
    this.throwIfDisposed();
    return await this._callNative('hasSubform', subformPath);
  }

  /**
   * Dispose of the native resources
   * Must be called when done using the instance
   * @returns Promise that resolves when disposal is complete
   */
  async dispose() {
    if (this.disposed) return;
    await this._callNative('dispose');
    this.disposed = true;
  }

  /**
   * Get the library version
   * @returns Promise resolving to version string
   */
  static async version() {
    var _jsi4;
    if (useJSI && (_jsi4 = _jsi) !== null && _jsi4 !== void 0 && _jsi4.version) {
      return _jsi.version();
    }
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