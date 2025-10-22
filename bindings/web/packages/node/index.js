/**
 * @json-eval-rs/node
 * JSON Eval RS for Node.js and Server-Side Rendering (SSR)
 */

import { JSONEvalCore, getVersion } from '@json-eval-rs/core';
import * as wasm from './pkg/json_eval_rs.js';

/**
 * JSONEval class with Node.js WASM pre-configured
 */
export class JSONEval extends JSONEvalCore {
  constructor(options) {
    super(wasm, options);
  }

  /**
   * Create a new JSONEval instance from a cached ParsedSchema
   * @param {string} cacheKey - Cache key to lookup in ParsedSchemaCache
   * @param {object} [context] - Optional context data
   * @param {object} [data] - Optional initial data
   * @returns {JSONEval} New instance
   */
  static fromCache(cacheKey, context, data) {
    return new JSONEval({
      schema: cacheKey,
      context,
      data,
      fromCache: true
    });
  }
}

/**
 * Get library version
 * @returns {string}
 */
export function version() {
  return getVersion(wasm);
}

// Re-export types for convenience
export * from '@json-eval-rs/core';

export default JSONEval;
