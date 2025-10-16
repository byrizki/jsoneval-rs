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
 * Validation error for a specific field
 */
export interface ValidationError {
  /** Field path with the error */
  path: string;
  /** Type of validation rule that failed */
  ruleType: string;
  /** Error message */
  message: string;
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
  data: string | object;
  /** Optional context data (string or object) */
  context?: string | object;
  /** Whether to recursively follow dependency chains */
  nested?: boolean;
}

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
export class JSONEval {
  private handle: string;
  private disposed = false;

  /**
   * Create a new JSONEval instance
   * @param options - Configuration options
   * @throws {Error} If schema is invalid
   */
  constructor(options: JSONEvalOptions) {
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
   * Re-evaluate fields that depend on changed paths
   * @param options - Dependent evaluation options
   * @returns Promise resolving to updated evaluated schema object
   * @throws {Error} If evaluation fails
   */
  async evaluateDependents(options: EvaluateDependentsOptions): Promise<any> {
    this.throwIfDisposed();
    
    try {
      const { changedPaths, data, context, nested = true } = options;
      const dataStr = this.toJsonString(data);
      const contextStr = context ? this.toJsonString(context) : null;
      
      const resultStr = await JsonEvalRs.evaluateDependents(
        this.handle,
        changedPaths,
        dataStr,
        contextStr,
        nested
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
   * @param skipLayout - Whether to skip layout resolution (default: false)
   * @returns Promise resolving to the value at the path, or null if not found
   * @throws {Error} If operation fails
   */
  async getValueByPath(path: string, skipLayout: boolean = false): Promise<any | null> {
    this.throwIfDisposed();
    const resultStr = await JsonEvalRs.getValueByPath(this.handle, path, skipLayout);
    return resultStr ? JSON.parse(resultStr) : null;
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
