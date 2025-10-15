/**
 * @json-eval-rs/webworker
 * Run JSON Eval RS in a Web Worker for better performance
 */

let messageId = 0;

/**
 * JSONEvalWorker - Run WASM operations in a Web Worker (off main thread)
 * 
 * @example
 * ```js
 * import { JSONEvalWorker } from '@json-eval-rs/webworker';
 * 
 * const evaluator = new JSONEvalWorker({
 *   schema: { type: 'object', properties: { ... } }
 * });
 * 
 * await evaluator.init();
 * const result = await evaluator.validate({ data: { name: 'John' } });
 * ```
 */
export class JSONEvalWorker {
  /**
   * @param {object} options
   * @param {object} options.schema - JSON schema
   * @param {object} [options.context] - Optional context data
   * @param {object} [options.data] - Optional initial data
   * @param {string} [options.workerUrl] - Optional custom worker URL
   */
  constructor({ schema, context, data, workerUrl }) {
    this._schema = schema;
    this._context = context;
    this._data = data;
    this._worker = null;
    this._ready = false;
    this._pending = new Map();
    this._workerUrl = workerUrl || new URL('./worker.js', import.meta.url);
  }

  /**
   * Initialize the worker and WASM module
   */
  async init() {
    if (this._ready) return;

    // Create worker
    this._worker = new Worker(this._workerUrl, { type: 'module' });

    // Set up message handler
    this._worker.addEventListener('message', (event) => {
      const { id, type, result, error } = event.data;
      const pending = this._pending.get(id);
      
      if (!pending) return;

      this._pending.delete(id);

      if (type === 'ERROR') {
        pending.reject(new Error(error.message));
      } else {
        pending.resolve(result);
      }
    });

    // Initialize WASM in worker
    await this._sendMessage('INIT', {
      schema: JSON.stringify(this._schema),
      context: this._context ? JSON.stringify(this._context) : null,
      data: this._data ? JSON.stringify(this._data) : null
    });

    this._ready = true;
  }

  /**
   * Send message to worker and wait for response
   * @private
   */
  _sendMessage(type, payload) {
    return new Promise((resolve, reject) => {
      const id = messageId++;
      
      this._pending.set(id, { resolve, reject });
      this._worker.postMessage({ id, type, payload });

      // Timeout after 30 seconds
      setTimeout(() => {
        if (this._pending.has(id)) {
          this._pending.delete(id);
          reject(new Error('Worker timeout'));
        }
      }, 30000);
    });
  }

  /**
   * Validate data against schema
   * @param {object} options
   * @param {object} options.data - Data to validate
   * @param {object} [options.context] - Optional context
   * @returns {Promise<{has_error: boolean, errors: Array}>}
   */
  async validate({ data, context }) {
    if (!this._ready) {
      throw new Error('Worker not initialized. Call init() first.');
    }

    return this._sendMessage('VALIDATE', {
      data: JSON.stringify(data),
      context: context ? JSON.stringify(context) : null
    });
  }

  /**
   * Evaluate schema with data
   * @param {object} options
   * @param {object} options.data - Data to evaluate
   * @param {object} [options.context] - Optional context
   * @returns {Promise<any>}
   */
  async evaluate({ data, context }) {
    if (!this._ready) {
      throw new Error('Worker not initialized. Call init() first.');
    }

    return this._sendMessage('EVALUATE', {
      data: JSON.stringify(data),
      context: context ? JSON.stringify(context) : null
    });
  }

  /**
   * Evaluate dependent fields
   * @param {object} options
   * @param {string[]} options.changedPaths - Array of changed field paths
   * @param {object} options.data - Current data
   * @param {object} [options.context] - Optional context
   * @param {boolean} [options.nested=true] - Follow dependency chains
   * @returns {Promise<any>}
   */
  async evaluateDependents({ changedPaths, data, context, nested = true }) {
    if (!this._ready) {
      throw new Error('Worker not initialized. Call init() first.');
    }

    return this._sendMessage('EVALUATE_DEPENDENTS', {
      changedPaths,
      data: JSON.stringify(data),
      context: context ? JSON.stringify(context) : null,
      nested
    });
  }

  /**
   * Get evaluated schema
   * @param {object} [options]
   * @param {boolean} [options.skipLayout=false] - Skip layout resolution
   * @returns {Promise<any>}
   */
  async getEvaluatedSchema({ skipLayout = false } = {}) {
    if (!this._ready) {
      throw new Error('Worker not initialized. Call init() first.');
    }

    return this._sendMessage('GET_EVALUATED_SCHEMA', { skipLayout });
  }

  /**
   * Get schema values
   * @returns {Promise<object>}
   */
  async getSchemaValue() {
    if (!this._ready) {
      throw new Error('Worker not initialized. Call init() first.');
    }

    return this._sendMessage('GET_SCHEMA_VALUE', {});
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
    if (!this._ready) {
      throw new Error('Worker not initialized. Call init() first.');
    }

    await this._sendMessage('RELOAD_SCHEMA', {
      schema: JSON.stringify(schema),
      context: context ? JSON.stringify(context) : null,
      data: data ? JSON.stringify(data) : null
    });

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
    if (!this._ready) {
      throw new Error('Worker not initialized. Call init() first.');
    }

    return this._sendMessage('CACHE_STATS', {});
  }

  /**
   * Clear the evaluation cache
   * @returns {Promise<void>}
   */
  async clearCache() {
    if (!this._ready) {
      throw new Error('Worker not initialized. Call init() first.');
    }

    await this._sendMessage('CLEAR_CACHE', {});
  }

  /**
   * Get the number of cached entries
   * @returns {Promise<number>}
   */
  async cacheLen() {
    if (!this._ready) {
      throw new Error('Worker not initialized. Call init() first.');
    }

    return this._sendMessage('CACHE_LEN', {});
  }

  /**
   * Terminate worker and free resources
   */
  async free() {
    if (this._worker) {
      await this._sendMessage('FREE', {});
      this._worker.terminate();
      this._worker = null;
      this._ready = false;
      this._pending.clear();
    }
  }
}

export default JSONEvalWorker;
