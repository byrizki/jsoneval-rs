/**
 * High-performance JSON Logic evaluator with schema validation
 * @module @json-eval-rs/web
 */

import init, { JSONEvalWasm, version as wasmVersion, init as wasmInit } from './json_eval_rs.js';

// Initialize WASM module
let initialized = false;
let initPromise = null;

async function ensureInitialized() {
  if (initialized) return;
  if (initPromise) return initPromise;
  
  initPromise = init().then(() => {
    wasmInit();
    initialized = true;
  });
  
  return initPromise;
}

/**
 * Wrapper class for better JavaScript ergonomics
 */
export class JSONEval {
  /**
   * @param {object} options
   * @param {string} options.schema - JSON schema string
   * @param {string} [options.context] - Optional context data (JSON string)
   * @param {string} [options.data] - Optional initial data (JSON string)
   */
  constructor({ schema, context, data }) {
    this._initPromise = ensureInitialized().then(() => {
      this._inner = new JSONEvalWasm(schema, context || null, data || null);
    });
    this._ready = false;
  }

  async _ensureReady() {
    if (!this._ready) {
      await this._initPromise;
      this._ready = true;
    }
  }

  /**
   * Evaluate schema with provided data (returns JSON string)
   * @param {object} options
   * @param {string} options.data - JSON data string
   * @param {string} [options.context] - Optional context data (JSON string)
   * @returns {Promise<string>} Evaluated schema as JSON string
   */
  async evaluate({ data, context }) {
    await this._ensureReady();
    return this._inner.evaluate(data, context || null);
  }

  /**
   * Evaluate schema with provided data (returns JavaScript object)
   * @param {object} options
   * @param {string} options.data - JSON data string
   * @param {string} [options.context] - Optional context data (JSON string)
   * @returns {Promise<any>} Evaluated schema as JavaScript object
   */
  async evaluateJS({ data, context }) {
    await this._ensureReady();
    return this._inner.evaluateJS(data, context || null);
  }

  /**
   * Validate data against schema rules
   * @param {object} options
   * @param {string} options.data - JSON data string
   * @param {string} [options.context] - Optional context data (JSON string)
   * @returns {Promise<object>} ValidationResult
   */
  async validate({ data, context }) {
    await this._ensureReady();
    return this._inner.validate(data, context || null);
  }

  /**
   * Re-evaluate fields that depend on changed paths (returns JSON string)
   * @param {object} options
   * @param {string[]} options.changedPaths - Array of field paths that changed
   * @param {string} options.data - Updated JSON data string
   * @param {string} [options.context] - Optional context data (JSON string)
   * @param {boolean} [options.nested=true] - Whether to recursively follow dependency chains
   * @returns {Promise<string>} Updated evaluated schema as JSON string
   */
  async evaluateDependents({ changedPaths, data, context, nested = true }) {
    await this._ensureReady();
    return this._inner.evaluateDependents(changedPaths, data, context || null, nested);
  }

  /**
   * Re-evaluate fields that depend on changed paths (returns JavaScript object)
   * @param {object} options
   * @param {string[]} options.changedPaths - Array of field paths that changed
   * @param {string} options.data - Updated JSON data string
   * @param {string} [options.context] - Optional context data (JSON string)
   * @param {boolean} [options.nested=true] - Whether to recursively follow dependency chains
   * @returns {Promise<any>} Updated evaluated schema as JavaScript object
   */
  async evaluateDependentsJS({ changedPaths, data, context, nested = true }) {
    await this._ensureReady();
    return this._inner.evaluateDependentsJS(changedPaths, data, context || null, nested);
  }

  /**
   * Get the evaluated schema with optional layout resolution (returns JSON string)
   * @param {object} [options]
   * @param {boolean} [options.skipLayout=false] - Whether to skip layout resolution
   * @returns {Promise<string>} Evaluated schema as JSON string
   */
  async getEvaluatedSchema({ skipLayout = false } = {}) {
    await this._ensureReady();
    return this._inner.getEvaluatedSchema(skipLayout);
  }

  /**
   * Get the evaluated schema with optional layout resolution (returns JavaScript object)
   * @param {object} [options]
   * @param {boolean} [options.skipLayout=false] - Whether to skip layout resolution
   * @returns {Promise<any>} Evaluated schema as JavaScript object
   */
  async getEvaluatedSchemaJS({ skipLayout = false } = {}) {
    await this._ensureReady();
    return this._inner.getEvaluatedSchemaJS(skipLayout);
  }

  /**
   * Get all schema values (evaluations ending with .value)
   * @returns {Promise<object>} Map of path -> value as JavaScript object
   */
  async getSchemaValue() {
    await this._ensureReady();
    return this._inner.getSchemaValue();
  }

  /**
   * Free the underlying WebAssembly resources
   */
  free() {
    if (this._inner) {
      this._inner.free();
      this._inner = null;
    }
  }
}

/**
 * Get the library version
 * @returns {Promise<string>} Version string
 */
export async function version() {
  await ensureInitialized();
  return wasmVersion();
}

export { init, wasmInit as initWasm };
export default JSONEval;
