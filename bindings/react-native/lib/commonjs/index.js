"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.multiply = exports.default = exports.JSONEval = void 0;
exports.useJSONEval = useJSONEval;
var _reactNative = require("react-native");
var React = _interopRequireWildcard(require("react"));
function _interopRequireWildcard(e, t) { if ("function" == typeof WeakMap) var r = new WeakMap(), n = new WeakMap(); return (_interopRequireWildcard = function (e, t) { if (!t && e && e.__esModule) return e; var o, i, f = { __proto__: null, default: e }; if (null === e || "object" != typeof e && "function" != typeof e) return f; if (o = t ? n : r) { if (o.has(e)) return o.get(e); o.set(e, f); } for (const t in e) "default" !== t && {}.hasOwnProperty.call(e, t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e, t)) && (i.get || i.set) ? o(f, t, i) : f[t] = e[t]); return f; })(e, t); }
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
 * Validation error for a specific field
 */

/**
 * Result of validation operation
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
 * High-performance JSON Logic evaluator with schema validation for React Native
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
   * Create a new JSONEval instance
   * @param options - Configuration options
   * @throws {Error} If schema is invalid
   */
  constructor(options) {
    const {
      schema,
      context,
      data
    } = options;
    const schemaStr = typeof schema === 'string' ? schema : JSON.stringify(schema);
    const contextStr = context ? typeof context === 'string' ? context : JSON.stringify(context) : undefined;
    const dataStr = data ? typeof data === 'string' ? data : JSON.stringify(data) : undefined;
    this.handle = JsonEvalRs.create(schemaStr, contextStr, dataStr);
  }
  throwIfDisposed() {
    if (this.disposed) {
      throw new Error('JSONEval instance has been disposed');
    }
  }
  toJsonString(value) {
    return typeof value === 'string' ? value : JSON.stringify(value);
  }

  /**
   * Evaluate schema with provided data
   * @param options - Evaluation options
   * @returns Promise resolving to evaluated schema object
   * @throws {Error} If evaluation fails
   */
  async evaluate(options) {
    this.throwIfDisposed();
    const dataStr = this.toJsonString(options.data);
    const contextStr = options.context ? this.toJsonString(options.context) : undefined;
    const resultStr = await JsonEvalRs.evaluate(this.handle, dataStr, contextStr);
    return JSON.parse(resultStr);
  }

  /**
   * Validate data against schema rules
   * @param options - Validation options
   * @returns Promise resolving to ValidationResult
   * @throws {Error} If validation operation fails
   */
  async validate(options) {
    this.throwIfDisposed();
    const dataStr = this.toJsonString(options.data);
    const contextStr = options.context ? this.toJsonString(options.context) : undefined;
    const resultStr = await JsonEvalRs.validate(this.handle, dataStr, contextStr);
    return JSON.parse(resultStr);
  }

  /**
   * Re-evaluate fields that depend on changed paths
   * @param options - Dependent evaluation options
   * @returns Promise resolving to updated evaluated schema object
   * @throws {Error} If evaluation fails
   */
  async evaluateDependents(options) {
    this.throwIfDisposed();
    const {
      changedPaths,
      data,
      context,
      nested = true
    } = options;
    const dataStr = this.toJsonString(data);
    const contextStr = context ? this.toJsonString(context) : undefined;
    const resultStr = await JsonEvalRs.evaluateDependents(this.handle, changedPaths, dataStr, contextStr, nested);
    return JSON.parse(resultStr);
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
    return JSON.parse(resultStr);
  }

  /**
   * Get all schema values (evaluations ending with .value)
   * @returns Promise resolving to map of path -> value
   * @throws {Error} If operation fails
   */
  async getSchemaValue() {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSchemaValue(this.handle);
    return JSON.parse(resultStr);
  }

  /**
   * Reload schema with new data
   * @param options - Configuration options with new schema, context, and data
   * @throws {Error} If reload fails
   */
  async reloadSchema(options) {
    this.throwIfDisposed();
    const {
      schema,
      context,
      data
    } = options;
    const schemaStr = typeof schema === 'string' ? schema : JSON.stringify(schema);
    const contextStr = context ? typeof context === 'string' ? context : JSON.stringify(context) : undefined;
    const dataStr = data ? typeof data === 'string' ? data : JSON.stringify(data) : undefined;
    await JsonEvalRs.reloadSchema(this.handle, schemaStr, contextStr, dataStr);
  }

  /**
   * Get cache statistics
   * @returns Promise resolving to cache statistics
   * @throws {Error} If operation fails
   */
  async cacheStats() {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.cacheStats(this.handle);
    return JSON.parse(resultStr);
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
   * Validate data against schema rules with optional path filtering
   * @param options - Validation options with optional path filtering
   * @returns Promise resolving to ValidationResult
   * @throws {Error} If validation operation fails
   */
  async validatePaths(options) {
    this.throwIfDisposed();
    const dataStr = this.toJsonString(options.data);
    const contextStr = options.context ? this.toJsonString(options.context) : undefined;
    const paths = options.paths || undefined;
    const resultStr = await JsonEvalRs.validatePaths(this.handle, dataStr, contextStr, paths);
    return JSON.parse(resultStr);
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
  const [evalInstance, setEvalInstance] = React.useState(null);
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
var _default = exports.default = JSONEval; // For backwards compatibility
const multiply = (a, b) => {
  return JsonEvalRs.multiply(a, b);
};
exports.multiply = multiply;
//# sourceMappingURL=index.js.map