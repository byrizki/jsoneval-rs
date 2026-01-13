/**
 * @json-eval-rs/node
 * JSON Eval RS for Node.js and Server-Side Rendering (SSR)
 */

import { JSONEvalCore, getVersion, JSONEvalOptions } from '@json-eval-rs/webcore';
// @ts-ignore - implicitly loaded, file exists after build
import * as wasm from '../pkg/json_eval_rs.js';

/**
 * JSONEval class with Node.js WASM pre-configured
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

  /**
   * Evaluate logic expression without creating an instance
   * @param logicStr - JSON Logic expression (string or object)
   * @param data - Optional data (string or object)
   * @param context - Optional context (string or object)
   * @returns Evaluation result
   */
  static evaluateLogic(logicStr: string | object, data?: any, context?: any): any {
    return JSONEvalCore.evaluateLogic(wasm, logicStr, data, context);
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
