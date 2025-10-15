/**
 * @json-eval-rs/core
 * High-level JavaScript API for JSON Eval RS WASM bindings
 * 
 * This package provides a clean, ergonomic API that works with any WASM target:
 * - @json-eval-rs/bundler (for webpack, vite, etc)
 * - @json-eval-rs/vanilla (for direct browser use)
 * - @json-eval-rs/node (for Node.js)
 */

/**
 * JSONEval - High-level wrapper for better JavaScript ergonomics
 * 
 * @example
 * ```js
 * import { JSONEval } from '@json-eval-rs/core';
 * 
 * const evaluator = new JSONEval({
 *   schema: { type: 'object', properties: { ... } },
 *   data: { name: 'John' }
 * });
 * 
 * const result = await evaluator.validate({ data: { name: '' } });
 * ```
 */
export class JSONEval {
  /**
   * @param {object} options
   * @param {object} options.schema - JSON schema
   * @param {object} [options.context] - Optional context data
   * @param {object} [options.data] - Optional initial data
   * @param {any} [options.wasmModule] - Optional pre-loaded WASM module
   */
  constructor({ schema, context, data, wasmModule }) {
    this._schema = schema;
    this._context = context;
    this._data = data;
    this._wasmModule = wasmModule;
    this._instance = null;
    this._ready = false;
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
        '2. Or install a peer dependency: npm install @json-eval-rs/bundler (or @json-eval-rs/vanilla or @json-eval-rs/node)'
      );
    }

    const { JSONEvalWasm } = this._wasmModule;
    this._instance = new JSONEvalWasm(
      JSON.stringify(this._schema),
      this._context ? JSON.stringify(this._context) : null,
      this._data ? JSON.stringify(this._data) : null
    );
    this._ready = true;
  }

  /**
   * Validate data against schema
   * @param {object} options
   * @param {object} options.data - Data to validate
   * @param {object} [options.context] - Optional context
   * @returns {Promise<{has_error: boolean, errors: Array}>}
   */
  async validate({ data, context }) {
    await this.init();
    return this._instance.validate(
      JSON.stringify(data),
      context ? JSON.stringify(context) : null
    );
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
    return this._instance.evaluateJS(
      JSON.stringify(data),
      context ? JSON.stringify(context) : null
    );
  }

  /**
   * Evaluate dependent fields (returns parsed JavaScript object)
   * @param {object} options
   * @param {string[]} options.changedPaths - Array of changed field paths
   * @param {object} options.data - Current data
   * @param {object} [options.context] - Optional context
   * @param {boolean} [options.nested=true] - Follow dependency chains
   * @returns {Promise<any>}
   */
  async evaluateDependents({ changedPaths, data, context, nested = true }) {
    await this.init();
    return this._instance.evaluateDependentsJS(
      changedPaths,
      JSON.stringify(data),
      context ? JSON.stringify(context) : null,
      nested
    );
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
   * Get schema values (evaluations ending with .value)
   * @returns {Promise<object>}
   */
  async getSchemaValue() {
    await this.init();
    return this._instance.getSchemaValue();
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

    await this._instance.reloadSchema(
      JSON.stringify(schema),
      context ? JSON.stringify(context) : null,
      data ? JSON.stringify(data) : null
    );

    // Update internal state
    this._schema = schema;
    this._context = context;
    this._data = data;
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
 * Get library version
 * @param {any} wasmModule - WASM module (required)
 * @returns {Promise<string>}
 * 
 * @example
 * import { version } from '@json-eval-rs/core';
 * import * as wasmModule from '@json-eval-rs/bundler';
 * const v = await version(wasmModule);
 */
export async function version(wasmModule) {
  if (!wasmModule) {
    throw new Error(
      'WASM module is required. Usage:\n' +
      'import { version } from "@json-eval-rs/core";\n' +
      'import * as wasmModule from "@json-eval-rs/bundler";\n' +
      'const v = await version(wasmModule);'
    );
  }
  return wasmModule.version();
}

export default JSONEval;
