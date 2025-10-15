/**
 * @json-eval-rs/bundler
 * JSON Eval RS for modern bundlers (Webpack, Vite, Rollup, Next.js, etc.)
 */

import { JSONEvalCore, getVersion } from '@json-eval-rs/core';
import * as wasm from './pkg/json_eval_rs.js';

/**
 * JSONEval class with bundler WASM pre-configured
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
