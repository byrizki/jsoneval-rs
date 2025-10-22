/**
 * @json-eval-rs/core
 * High-level JavaScript API for JSON Eval RS WASM bindings
 * 
 * This package provides a clean, ergonomic API that works with any WASM target:
 */

/**
 * Get the library version from the WASM module
 * @param {any} wasmModule - WASM module
 * @returns {string} Version string
 */
export function getVersion(wasmModule) {
  if (wasmModule && typeof wasmModule.getVersion === 'function') {
    return wasmModule.getVersion();
  }
  return 'unknown';
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
 * import { JSONEval } from '@json-eval-rs/core';
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
  /**
   * @param {any} wasmModule - WASM module (injected by wrapper package)
   * @param {object} options
   * @param {object|Uint8Array|string} options.schema - JSON schema, MessagePack bytes, or cache key
   * @param {object} [options.context] - Optional context data
   * @param {object} [options.data] - Optional initial data
   * @param {boolean} [options.fromCache] - If true, schema is treated as a cache key
   */
  constructor(wasmModule, { schema, context, data, fromCache = false }) {
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
  async init() {
    if (this._ready) return;

    // If WASM module not provided, throw error - user must provide it or install peer dependency
    if (!this._wasmModule) {
      throw new Error(
        'No WASM module provided. Please either:\n' +
        '1. Pass wasmModule in constructor: new JSONEval({ schema, wasmModule: await import("@json-eval-rs/bundler") })\n' +
        '2. Or install a peer dependency: yarn install @json-eval-rs/bundler (or @json-eval-rs/vanilla or @json-eval-rs/node)'
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
    } catch (error) {
      throw new Error(`Failed to create JSONEval instance: ${error.message || error}`);
    }
  }

  /**
   * Create a new JSONEval instance from a cached ParsedSchema
   * Static factory method for convenience
   * 
   * @param {any} wasmModule - WASM module
   * @param {string} cacheKey - Cache key to lookup in ParsedSchemaCache
   * @param {object} [context] - Optional context data
   * @param {object} [data] - Optional initial data
   * @returns {JSONEvalCore} New instance
   */
  static fromCache(wasmModule, cacheKey, context, data) {
    return new JSONEvalCore(wasmModule, {
      schema: cacheKey,
      context,
      data,
      fromCache: true
    });
  }

  /**
   * Validate data against schema (returns parsed JavaScript object)
   * Uses validateJS for Worker-safe serialization
   * @param {object} options
   * @param {object} options.data - Data to validate
   * @param {object} [options.context] - Optional context
   * @returns {Promise<{has_error: boolean, errors: Array<{path: string, rule_type: string, message: string}>}>}
   */
  async validate({ data, context }) {
    await this.init();
    try {
      // Use validateJS for proper serialization (Worker-safe)
      return this._instance.validateJS(
        JSON.stringify(data),
        context ? JSON.stringify(context) : null
      );
    } catch (error) {
      throw new Error(`Validation failed: ${error.message || error}`);
    }
  }

  /**
   * Evaluate schema with data (returns parsed JavaScript object)
   * @param {object} options
   * @param {object} options.data - Data to evaluate
   * @param {object} [options.context] - Optional context
   * @returns {Promise<any>}
   */
  async evaluate({ data, context }) {
    await this.init();
    try {
      return this._instance.evaluateJS(
        JSON.stringify(data),
        context ? JSON.stringify(context) : null
      );
    } catch (error) {
      throw new Error(`Evaluation failed: ${error.message || error}`);
    }
  }

  /**
   * Evaluate dependent fields (returns parsed JavaScript object, processes transitively)
   * @param {object} options
   * @param {string} options.changedPath - Single changed field path (e.g., "#/illustration/properties/field")
   * @param {object} [options.data] - Optional updated data (null to use existing)
   * @param {object} [options.context] - Optional context
   * @returns {Promise<Array>} Array of dependent change objects
   */
  async evaluateDependents({ changedPath, data, context }) {
    await this.init();
    try {
      return this._instance.evaluateDependentsJS(
        changedPath,
        data ? JSON.stringify(data) : null,
        context ? JSON.stringify(context) : null
      );
    } catch (error) {
      throw new Error(`Dependent evaluation failed: ${error.message || error}`);
    }
  }

  /**
   * Get evaluated schema
   * @param {object} [options]
   * @param {boolean} [options.skipLayout=false] - Skip layout resolution
   * @returns {Promise<any>}
   */
  async getEvaluatedSchema({ skipLayout = false } = {}) {
    await this.init();
    return this._instance.getEvaluatedSchemaJS(skipLayout);
  }

  /**
   * Get evaluated schema as MessagePack binary data
   * @param {object} [options]
   * @param {boolean} [options.skipLayout=false] - Skip layout resolution
   * @returns {Promise<Uint8Array>} MessagePack-encoded schema bytes
   */
  async getEvaluatedSchemaMsgpack({ skipLayout = false } = {}) {
    await this.init();
    return this._instance.getEvaluatedSchemaMsgpack(skipLayout);
  }

  /**
   * Get schema values (evaluations ending with .value)
   * @returns {Promise<object>}
   */
  async getSchemaValue() {
    await this.init();
    return this._instance.getSchemaValue();
  }

  /**
   * Get evaluated schema without $params field
   * @param {object} [options]
   * @param {boolean} [options.skipLayout=false] - Skip layout resolution
   * @returns {Promise<any>}
   */
  async getEvaluatedSchemaWithoutParams({ skipLayout = false } = {}) {
    await this.init();
    return this._instance.getEvaluatedSchemaWithoutParamsJS(skipLayout);
  }

  /**
   * Get a value from the evaluated schema using dotted path notation
   * @param {object} options
   * @param {string} options.path - Dotted path to the value (e.g., "properties.field.value")
   * @param {boolean} [options.skipLayout=false] - Skip layout resolution
   * @returns {Promise<any|null>} Value at the path, or null if not found
   */
  async getEvaluatedSchemaByPath({ path, skipLayout = false }) {
    await this.init();
    return this._instance.getEvaluatedSchemaByPathJS(path, skipLayout);
  }

  /**
   * Reload schema with new data
   * @param {object} options
   * @param {object} options.schema - New JSON schema
   * @param {object} [options.context] - Optional new context
   * @param {object} [options.data] - Optional new data
   * @returns {Promise<void>}
   */
  async reloadSchema({ schema, context, data }) {
    if (!this._instance) {
      throw new Error('Instance not initialized. Call init() first.');
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
    } catch (error) {
      throw new Error(`Failed to reload schema: ${error.message || error}`);
    }
  }

  /**
   * Reload schema from MessagePack bytes
   * @param {Uint8Array} schemaMsgpack - MessagePack-encoded schema bytes
   * @param {object} [context] - Optional new context
   * @param {object} [data] - Optional new data
   * @returns {Promise<void>}
   */
  async reloadSchemaMsgpack(schemaMsgpack, context, data) {
    if (!this._instance) {
      throw new Error('Instance not initialized. Call init() first.');
    }

    if (!(schemaMsgpack instanceof Uint8Array)) {
      throw new Error('schemaMsgpack must be a Uint8Array');
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
    } catch (error) {
      throw new Error(`Failed to reload schema from MessagePack: ${error.message || error}`);
    }
  }

  /**
   * Reload schema from ParsedSchemaCache using a cache key
   * @param {string} cacheKey - Cache key to lookup in the global ParsedSchemaCache
   * @param {object} [context] - Optional new context
   * @param {object} [data] - Optional new data
   * @returns {Promise<void>}
   */
  async reloadSchemaFromCache(cacheKey, context, data) {
    if (!this._instance) {
      throw new Error('Instance not initialized. Call init() first.');
    }

    if (typeof cacheKey !== 'string' || !cacheKey) {
      throw new Error('cacheKey must be a non-empty string');
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
    } catch (error) {
      throw new Error(`Failed to reload schema from cache: ${error.message || error}`);
    }
  }

  /**
   * Get cache statistics
   * @returns {Promise<{hits: number, misses: number, entries: number}>}
   */
  async cacheStats() {
    await this.init();
    return this._instance.cacheStats();
  }

  /**
   * Clear the evaluation cache
   * @returns {Promise<void>}
   */
  async clearCache() {
    await this.init();
    this._instance.clearCache();
  }

  /**
   * Get the number of cached entries
   * @returns {Promise<number>}
   */
  async cacheLen() {
    await this.init();
    return this._instance.cacheLen();
  }

  /**
   * Resolve layout with optional evaluation
   * @param {object} [options]
   * @param {boolean} [options.evaluate=false] - If true, runs evaluation before resolving layout
   * @returns {Promise<void>}
   */
  async resolveLayout({ evaluate = false } = {}) {
    await this.init();
    return this._instance.resolveLayout(evaluate);
  }

  /**
   * Compile and run JSON logic from a JSON logic string
   * @param {object} options
   * @param {string|object} options.logicStr - JSON logic expression as a string or object
   * @param {object} [options.data] - Optional data to evaluate against (uses existing data if not provided)
   * @returns {Promise<any>} Result of the evaluation
   */
  async compileAndRunLogic({ logicStr, data }) {
    await this.init();
    const logic = typeof logicStr === 'string' ? logicStr : JSON.stringify(logicStr);
    return this._instance.compileAndRunLogicJS(
      logic,
      data ? JSON.stringify(data) : null
    );
  }

  /**
   * Validate data against schema rules with optional path filtering
   * @param {object} options
   * @param {object} options.data - Data to validate
   * @param {object} [options.context] - Optional context
   * @param {Array<string>} [options.paths] - Optional array of paths to validate (null for all)
   * @returns {Promise<{has_error: boolean, errors: Array<{path: string, rule_type: string, message: string}>}>}
   */
  async validatePaths({ data, context, paths }) {
    await this.init();
    try {
      // Use validatePathsJS for proper serialization (Worker-safe)
      return this._instance.validatePathsJS(
        JSON.stringify(data),
        context ? JSON.stringify(context) : null,
        paths || null
      );
    } catch (error) {
      throw new Error(`Validation failed: ${error.message || error}`);
    }
  }

  // ============================================================================
  // Subform Methods
  // ============================================================================

  /**
   * Evaluate a subform with data
   * @param {object} options
   * @param {string} options.subformPath - Path to the subform (e.g., "#/riders")
   * @param {object} options.data - Data for the subform
   * @param {object} [options.context] - Optional context
   * @returns {Promise<void>}
   */
  async evaluateSubform({ subformPath, data, context }) {
    await this.init();
    return this._instance.evaluateSubform(
      subformPath,
      JSON.stringify(data),
      context ? JSON.stringify(context) : null
    );
  }

  /**
   * Validate subform data against its schema rules
   * @param {object} options
   * @param {string} options.subformPath - Path to the subform
   * @param {object} options.data - Data for the subform
   * @param {object} [options.context] - Optional context
   * @returns {Promise<{has_error: boolean, errors: Array}>}
   */
  async validateSubform({ subformPath, data, context }) {
    await this.init();
    return this._instance.validateSubform(
      subformPath,
      JSON.stringify(data),
      context ? JSON.stringify(context) : null
    );
  }

  /**
   * Evaluate dependents in subform when a field changes
   * @param {object} options
   * @param {string} options.subformPath - Path to the subform
   * @param {string} options.changedPath - Path of the field that changed
   * @param {object} [options.data] - Optional updated data
   * @param {object} [options.context] - Optional context
   * @returns {Promise<any>}
   */
  async evaluateDependentsSubform({ subformPath, changedPath, data, context }) {
    await this.init();
    return this._instance.evaluateDependentsSubformJS(
      subformPath,
      changedPath,
      data ? JSON.stringify(data) : null,
      context ? JSON.stringify(context) : null
    );
  }

  /**
   * Resolve layout for subform
   * @param {object} options
   * @param {string} options.subformPath - Path to the subform
   * @param {boolean} [options.evaluate=false] - If true, runs evaluation before resolving layout
   * @returns {Promise<void>}
   */
  async resolveLayoutSubform({ subformPath, evaluate = false }) {
    await this.init();
    return this._instance.resolveLayoutSubform(subformPath, evaluate);
  }

  /**
   * Get evaluated schema from subform
   * @param {object} options
   * @param {string} options.subformPath - Path to the subform
   * @param {boolean} [options.resolveLayout=false] - Whether to resolve layout
   * @returns {Promise<any>}
   */
  async getEvaluatedSchemaSubform({ subformPath, resolveLayout = false }) {
    await this.init();
    return this._instance.getEvaluatedSchemaSubformJS(subformPath, resolveLayout);
  }

  /**
   * Get schema value from subform (all .value fields)
   * @param {object} options
   * @param {string} options.subformPath - Path to the subform
   * @returns {Promise<any>}
   */
  async getSchemaValueSubform({ subformPath }) {
    await this.init();
    return this._instance.getSchemaValueSubform(subformPath);
  }

  /**
   * Get evaluated schema without $params from subform
   * @param {object} options
   * @param {string} options.subformPath - Path to the subform
   * @param {boolean} [options.resolveLayout=false] - Whether to resolve layout
   * @returns {Promise<any>}
   */
  async getEvaluatedSchemaWithoutParamsSubform({ subformPath, resolveLayout = false }) {
    await this.init();
    return this._instance.getEvaluatedSchemaWithoutParamsSubformJS(subformPath, resolveLayout);
  }

  /**
   * Get evaluated schema by specific path from subform
   * @param {object} options
   * @param {string} options.subformPath - Path to the subform
   * @param {string} options.schemaPath - Path within the subform
   * @param {boolean} [options.skipLayout=false] - Whether to skip layout resolution
   * @returns {Promise<any|null>}
   */
  async getEvaluatedSchemaByPathSubform({ subformPath, schemaPath, skipLayout = false }) {
    await this.init();
    return this._instance.getEvaluatedSchemaByPathSubformJS(subformPath, schemaPath, skipLayout);
  }

  /**
   * Get list of available subform paths
   * @returns {Promise<Array<string>>}
   */
  async getSubformPaths() {
    await this.init();
    return this._instance.getSubformPaths();
  }

  /**
   * Check if a subform exists at the given path
   * @param {string} subformPath - Path to check
   * @returns {Promise<boolean>}
   */
  async hasSubform(subformPath) {
    await this.init();
    return this._instance.hasSubform(subformPath);
  }

  /**
   * Free WASM resources
   */
  free() {
    if (this._instance) {
      this._instance.free();
      this._instance = null;
      this._ready = false;
    }
  }
}

/**
 * Get library version (internal - use from specific packages)
 * @param {any} wasmModule - WASM module
 * @returns {string}
 */
export function getVersion(wasmModule) {
  return wasmModule.version();
}

export default JSONEvalCore;
