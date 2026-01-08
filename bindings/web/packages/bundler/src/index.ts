/**
 * @json-eval-rs/bundler
 * JSON Eval RS for modern bundlers (Webpack, Vite, Rollup, Next.js, etc.)
 */

import { JSONEvalCore, getVersion, JSONEvalOptions } from '@json-eval-rs/webcore';
// @ts-ignore - implicitly loaded by bundler, file exists after build
import * as wasm from '../pkg/json_eval_rs.js';

/**
 * JSONEval class with bundler WASM pre-configured
 */
export class JSONEval extends JSONEvalCore {
  constructor(options: JSONEvalOptions) {
    super(wasm, options);
  }

  /**
   * Create a new JSONEval instance from a cached ParsedSchema
   * @param cacheKey - Cache key to lookup in ParsedSchemaCache
   * @param context - Optional context data
   * @param data - Optional initial data
   * @returns New instance
   */
  static fromCache(cacheKey: string, context?: any, data?: any): JSONEval {
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
 */
export function version(): string {
  return getVersion(wasm);
}

// Re-export types for convenience
export * from '@json-eval-rs/webcore';

export default JSONEval;
