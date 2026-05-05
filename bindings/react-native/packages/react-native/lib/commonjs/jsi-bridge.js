"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.getJSIGlobal = getJSIGlobal;
exports.isJSIAvailable = isJSIAvailable;
/**
 * JSI Bridge for @json-eval-rs/react-native
 *
 * Provides direct synchronous access to native JSONEval via JSI,
 * bypassing the React Native bridge for near-zero overhead.
 *
 * Falls back to the bridge-based NativeModules API when JSI
 * is unavailable (e.g., during debugging with remote debugger).
 */

// Type definition for the JSI global installed by native code

// Cached reference to the JSI global
let _jsi = null;
let _jsiAttempted = false;

/**
 * Get the JSI global object if available.
 * Returns null if JSI is not installed (bridge fallback mode).
 */
function getJSIGlobal() {
  if (_jsiAttempted) return _jsi;
  _jsiAttempted = true;
  try {
    const g = global;
    if (g.jsonEval && typeof g.jsonEval.create === 'function') {
      _jsi = g.jsonEval;
    }
  } catch {
    // JSI not available
  }
  return _jsi;
}

/**
 * Check if JSI is available and installed.
 */
function isJSIAvailable() {
  return getJSIGlobal() !== null;
}
//# sourceMappingURL=jsi-bridge.js.map