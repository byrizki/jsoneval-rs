/**
 * @json-eval-rs/common
 * Shared utility functions for Web and React Native bindings.
 */

/**
 * Convert a value to JSON string.
 * If already a string, returns as-is.
 * Otherwise serializes with JSON.stringify.
 */
export function stringifyValue(value: string | object): string {
  return typeof value === 'string' ? value : JSON.stringify(value);
}

/**
 * Parse a value from JSON string.
 * If already a string, returns as-is.
 * Otherwise parses with JSON.parse.
 */
export function parseValue(value: string) {
  return typeof value === 'string' ? JSON.parse(value) : value;
}

/**
 * Serialize a value to JSON string, or return null if null/undefined.
 */
export function stringifyOrNull(value: any): string | null {
  if (value == null) return null;
  return typeof value === 'string' ? value : JSON.stringify(value);
}

/**
 * Extract error message from unknown error.
 */
export function extractErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
