/**
 * @json-eval-rs/vanilla
 * JSON Eval RS for direct browser usage or environments needing manual WASM loading (e.g. Next.js Turbopack)
 */

import { JSONEvalCore, getVersion, JSONEvalOptions } from '@json-eval-rs/webcore';
// @ts-ignore - implicitly loaded, file exists after build
import init, * as wasm from '../pkg/json_eval_rs.js';

/**
 * JSONEval class for vanilla/web usage.
 * Requires calling `init(wasmUrl)` before use.
 */
export class JSONEval extends JSONEvalCore {
  constructor(options: JSONEvalOptions) {
    super(wasm, options);
  }

  /**
   * Initialize the WASM module.
   * Shortcut for the default export from pkg.
   * @param input - Path to WASM file or WASM binary
   * @returns Promise resolving to WASM module
   */
  static async initWasm(input?: string | Request | Response | BufferSource | WebAssembly.Module): Promise<any> {
    return init(input);
  }

  /**
   * Evaluate logic expression without creating an instance.
   * NOTE: You MUST call `JSONEval.initWasm()` or `init()` before using this method.
   * 
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

// Re-export init for direct usage
export { init };

// Re-export types for convenience
export * from '@json-eval-rs/webcore';

export default JSONEval;
