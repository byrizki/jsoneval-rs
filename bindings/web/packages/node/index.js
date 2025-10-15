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
