/**
 * JSI Bridge for @json-eval-rs/react-native
 *
 * Provides direct synchronous access to native JSONEval via JSI,
 * bypassing the React Native bridge for near-zero overhead.
 *
 * Falls back to the bridge-based NativeModules API when JSI
 * is unavailable (e.g., during debugging with remote debugger).
 */
export interface JsonEvalJSIGlobal {
    create(schema: string, context: string | null, data: string | null): string;
    createFromCache(cacheKey: string, context: string | null, data: string | null): string;
    createFromMsgpack(msgpack: number[], context: string | null, data: string | null): string;
    dispose(handle: string): void;
    evaluateOnly(handle: string, data: string, context: string | null, paths: string | null): void;
    evaluate(handle: string, data: string, context: string | null, paths: string | null): string;
    validate(handle: string, data: string, context: string | null): string;
    validatePaths(handle: string, data: string, context: string | null, paths: string | null): string;
    evaluateDependents(handle: string, changedPaths: string, data: string | null, context: string | null, reEvaluate: boolean, includeSubforms: boolean): string;
    getEvaluatedSchema(handle: string): string;
    getSchemaValue(handle: string): string;
    getSchemaValueArray(handle: string): string;
    getSchemaValueObject(handle: string): string;
    getEvaluatedSchemaWithoutParams(handle: string): string;
    getEvaluatedSchemaByPath(handle: string, path: string): string;
    getEvaluatedSchemaByPaths(handle: string, pathsJson: string, format: number): string;
    getSchemaByPath(handle: string, path: string): string;
    getSchemaByPaths(handle: string, pathsJson: string, format: number): string;
    getFieldOptions(handle: string, fieldPath: string): string;
    resolveLayout(handle: string, evaluate: boolean): string;
    getResolvedLayout(handle: string): string;
    getEvaluatedSchemaResolved(handle: string): string;
    reloadSchema(handle: string, schema: string, context: string | null, data: string | null): void;
    reloadSchemaFromCache(handle: string, cacheKey: string, context: string | null, data: string | null): void;
    reloadSchemaMsgpack(handle: string, msgpack: number[], context: string | null, data: string | null): void;
    setTimezoneOffset(handle: string, offsetMinutes: number): void;
    cancel(handle: string): void;
    compileAndRunLogic(handle: string, logicStr: string, data: string | null, context: string | null): string;
    compileLogic(handle: string, logicStr: string): number;
    runLogic(handle: string, logicId: number, data: string | null, context: string | null): string;
    evaluateSubform(handle: string, subformPath: string, data: string, context: string | null, paths: string | null): void;
    validateSubform(handle: string, subformPath: string, data: string, context: string | null): string;
    evaluateDependentsSubform(handle: string, subformPath: string, changedPath: string, data: string | null, context: string | null, reEvaluate: boolean, includeSubforms: boolean): string;
    resolveLayoutSubform(handle: string, subformPath: string, evaluate: boolean): string;
    getResolvedLayoutSubform(handle: string, subformPath: string): string;
    getEvaluatedSchemaResolvedSubform(handle: string, subformPath: string): string;
    getEvaluatedSchemaSubform(handle: string, subformPath: string): string;
    getSchemaValueSubform(handle: string, subformPath: string): string;
    getSchemaValueArraySubform(handle: string, subformPath: string): string;
    getSchemaValueObjectSubform(handle: string, subformPath: string): string;
    getEvaluatedSchemaWithoutParamsSubform(handle: string, subformPath: string): string;
    getEvaluatedSchemaByPathSubform(handle: string, subformPath: string, schemaPath: string): string;
    getEvaluatedSchemaByPathsSubform(handle: string, subformPath: string, schemaPathsJson: string, format: number): string;
    getSchemaByPathSubform(handle: string, subformPath: string, schemaPath: string): string;
    getSchemaByPathsSubform(handle: string, subformPath: string, schemaPathsJson: string, format: number): string;
    getSubformPaths(handle: string): string;
    hasSubform(handle: string, subformPath: string): boolean;
    /** Convert ArrayBuffer (UTF-8 encoded) to string — replaces TextDecoder */
    decodeArrayBuffer(buffer: ArrayBuffer): string;
    evaluateLogic(logic: string, data: string | null, context: string | null): string;
    version(): string;
}
/**
 * Get the JSI global object if available.
 * Returns null if JSI is not installed (bridge fallback mode).
 */
export declare function getJSIGlobal(): JsonEvalJSIGlobal | null;
/**
 * Check if JSI is available and installed.
 */
export declare function isJSIAvailable(): boolean;
//# sourceMappingURL=jsi-bridge.d.ts.map