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
    dispose(handle: string): void;
    evaluateOnly(handle: string, data: string, context: string | null, paths: string | null): void;
    evaluate(handle: string, data: string, context: string | null, paths: string | null): string;
    validate(handle: string, data: string, context: string | null): string;
    validatePaths(handle: string, data: string, context: string | null, paths: string | null): string;
    evaluateDependents(handle: string, changedPaths: string, data: string | null, context: string | null, reEvaluate: boolean, includeSubforms: boolean): string;
    getEvaluatedSchema(handle: string, skipLayout: boolean): string;
    getSchemaValue(handle: string): string;
    getSchemaValueArray(handle: string): string;
    getSchemaValueObject(handle: string): string;
    getEvaluatedSchemaWithoutParams(handle: string, skipLayout: boolean): string;
    getEvaluatedSchemaByPath(handle: string, path: string, skipLayout: boolean): string;
    getEvaluatedSchemaByPaths(handle: string, pathsJson: string, skipLayout: boolean, format: number): string;
    getSchemaByPath(handle: string, path: string): string;
    getSchemaByPaths(handle: string, pathsJson: string, format: number): string;
    resolveLayout(handle: string, evaluate: boolean): void;
    reloadSchema(handle: string, schema: string, context: string | null, data: string | null): void;
    reloadSchemaFromCache(handle: string, cacheKey: string, context: string | null, data: string | null): void;
    setTimezoneOffset(handle: string, offsetMinutes: number): void;
    cancel(handle: string): void;
    compileAndRunLogic(handle: string, logicStr: string, data: string | null, context: string | null): string;
    compileLogic(handle: string, logicStr: string): number;
    runLogic(handle: string, logicId: number, data: string | null, context: string | null): string;
    evaluateSubform(handle: string, subformPath: string, data: string, context: string | null, paths: string | null): void;
    validateSubform(handle: string, subformPath: string, data: string, context: string | null): string;
    evaluateDependentsSubform(handle: string, subformPath: string, changedPath: string, data: string | null, context: string | null, reEvaluate: boolean, includeSubforms: boolean): string;
    resolveLayoutSubform(handle: string, subformPath: string, evaluate: boolean): void;
    getEvaluatedSchemaSubform(handle: string, subformPath: string, resolveLayout: boolean): string;
    getSchemaValueSubform(handle: string, subformPath: string): string;
    getSchemaValueArraySubform(handle: string, subformPath: string): string;
    getSchemaValueObjectSubform(handle: string, subformPath: string): string;
    getEvaluatedSchemaWithoutParamsSubform(handle: string, subformPath: string, resolveLayout: boolean): string;
    getEvaluatedSchemaByPathSubform(handle: string, subformPath: string, schemaPath: string, skipLayout: boolean): string;
    getEvaluatedSchemaByPathsSubform(handle: string, subformPath: string, schemaPathsJson: string, skipLayout: boolean, format: number): string;
    getSchemaByPathSubform(handle: string, subformPath: string, schemaPath: string): string;
    getSchemaByPathsSubform(handle: string, subformPath: string, schemaPathsJson: string, format: number): string;
    getSubformPaths(handle: string): string;
    hasSubform(handle: string, subformPath: string): boolean;
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