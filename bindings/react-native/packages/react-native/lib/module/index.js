import React from 'react';
import { NativeModules, Platform } from 'react-native';
const LINKING_ERROR = `The package '@json-eval-rs/react-native' doesn't seem to be linked. Make sure: \n\n` + Platform.select({
  ios: "- You have run 'pod install'\n",
  default: ''
}) + '- You rebuilt the app after installing the package\n' + '- You are not using Expo managed workflow\n';
const JsonEvalRs = NativeModules.JsonEvalRs ? NativeModules.JsonEvalRs : new Proxy({}, {
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
    const contextStr = context ? typeof context === 'string' ? context : JSON.stringify(context) : null;
    const dataStr = data ? typeof data === 'string' ? data : JSON.stringify(data) : null;
    const handle = JsonEvalRs.createFromCache(cacheKey, contextStr, dataStr);
    return new JSONEval({
      schema: {},
      _handle: handle
    });
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
      const schemaStr = typeof schema === 'string' ? schema : JSON.stringify(schema);
      const contextStr = context ? typeof context === 'string' ? context : JSON.stringify(context) : null;
      const dataStr = data ? typeof data === 'string' ? data : JSON.stringify(data) : null;
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
   * instead of an object to avoid the JSON.stringify overhead
   */
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
    try {
      const dataStr = this.toJsonString(options.data);
      const contextStr = options.context ? this.toJsonString(options.context) : null;
      const resultStr = await JsonEvalRs.evaluate(this.handle, dataStr, contextStr);
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
  async validate(options) {
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
   * Re-evaluate fields that depend on changed paths
   * @param options - Dependent evaluation options
   * @returns Promise resolving to updated evaluated schema object
   * @throws {Error} If evaluation fails
   */
  async evaluateDependents(options) {
    this.throwIfDisposed();
    try {
      const {
        changedPaths,
        data,
        context,
        nested = true
      } = options;
      const dataStr = this.toJsonString(data);
      const contextStr = context ? this.toJsonString(context) : null;
      const resultStr = await JsonEvalRs.evaluateDependents(this.handle, changedPaths, dataStr, contextStr, nested);
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
   * Get the evaluated schema without $params field
   * @param skipLayout - Whether to skip layout resolution (default: false)
   * @returns Promise resolving to evaluated schema object
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaWithoutParams(skipLayout = false) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchemaWithoutParams(this.handle, skipLayout);
    return JSON.parse(resultStr);
  }

  /**
   * Get a value from the evaluated schema using dotted path notation
   * @param path - Dotted path to the value (e.g., "properties.field.value")
   * @param skipLayout - Whether to skip layout resolution (default: false)
   * @returns Promise resolving to the value at the path, or null if not found
   * @throws {Error} If operation fails
   */
  async getEvaluatedSchemaByPath(path, skipLayout = false) {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getEvaluatedSchemaByPath(this.handle, path, skipLayout);
    return resultStr ? JSON.parse(resultStr) : null;
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
      const schemaStr = typeof schema === 'string' ? schema : JSON.stringify(schema);
      const contextStr = context ? typeof context === 'string' ? context : JSON.stringify(context) : null;
      const dataStr = data ? typeof data === 'string' ? data : JSON.stringify(data) : null;
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
      const contextStr = context ? typeof context === 'string' ? context : JSON.stringify(context) : null;
      const dataStr = data ? typeof data === 'string' ? data : JSON.stringify(data) : null;
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
      const contextStr = context ? typeof context === 'string' ? context : JSON.stringify(context) : null;
      const dataStr = data ? typeof data === 'string' ? data : JSON.stringify(data) : null;
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
   * Compile and run JSON logic from a JSON logic string
   * @param logicStr - JSON logic expression as a string or object
   * @param data - Optional data to evaluate against (uses existing data if not provided)
   * @returns Promise resolving to the result of the evaluation
   * @throws {Error} If compilation or evaluation fails
   */
  async compileAndRunLogic(logicStr, data) {
    this.throwIfDisposed();
    const logic = this.toJsonString(logicStr);
    const dataStr = data ? this.toJsonString(data) : null;
    const resultStr = await JsonEvalRs.compileAndRunLogic(this.handle, logic, dataStr);
    return JSON.parse(resultStr);
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
  async evaluateSubform(options) {
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
  async validateSubform(options) {
    this.throwIfDisposed();
    const dataStr = this.toJsonString(options.data);
    const contextStr = options.context ? this.toJsonString(options.context) : null;
    const resultStr = await JsonEvalRs.validateSubform(this.handle, options.subformPath, dataStr, contextStr);
    return JSON.parse(resultStr);
  }

  /**
   * Evaluate dependents in subform when a field changes
   * @param options - Options including subform path, changed path, and optional data
   * @returns Promise resolving to dependent evaluation results
   * @throws {Error} If evaluation fails
   */
  async evaluateDependentsSubform(options) {
    this.throwIfDisposed();
    const dataStr = options.data ? this.toJsonString(options.data) : null;
    const contextStr = options.context ? this.toJsonString(options.context) : null;
    const resultStr = await JsonEvalRs.evaluateDependentsSubform(this.handle, options.subformPath, options.changedPath, dataStr, contextStr);
    return JSON.parse(resultStr);
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
    return JSON.parse(resultStr);
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
    return JSON.parse(resultStr);
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
    return JSON.parse(resultStr);
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
    return resultStr ? JSON.parse(resultStr) : null;
  }

  /**
   * Get list of available subform paths
   * @returns Promise resolving to array of subform paths
   * @throws {Error} If operation fails
   */
  async getSubformPaths() {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getSubformPaths(this.handle);
    return JSON.parse(resultStr);
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
export function useJSONEval(options) {
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
export default JSONEval;

// For backwards compatibility
export const multiply = (a, b) => {
  return JsonEvalRs.multiply(a, b);
};
//# sourceMappingURL=index.js.map