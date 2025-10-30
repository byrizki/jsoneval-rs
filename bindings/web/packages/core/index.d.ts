/**
 * @json-eval-rs/webcore - TypeScript definitions
 */

/**
 * Get the library version from the WASM module
 * @param wasmModule - WASM module
 * @returns Version string
 */
export function getVersion(wasmModule: any): string;

export interface ValidationError {
  path: string;
  rule_type: string;
  message: string;
}

export interface ValidationResult {
  has_error: boolean;
  errors: ValidationError[];
}

export interface DependentChange {
  path: string;
  value: any;
}

export interface JSONEvalOptions {
  schema: any;
  context?: any;
  data?: any;
  wasmModule?: any;
}

export interface ValidateOptions {
  data: any;
  context?: any;
}

export interface EvaluateOptions {
  data: any;
  context?: any;
}

export interface EvaluateDependentsOptions {
  changedPaths: string[];
  data?: any;
  context?: any;
  reEvaluate?: boolean;
}

export interface GetEvaluatedSchemaOptions {
  skipLayout?: boolean;
}

export interface GetValueByPathOptions {
  path: string;
  skipLayout?: boolean;
}

export interface GetValueByPathsOptions {
  paths: string[];
  skipLayout?: boolean;
}

export interface GetSchemaByPathOptions {
  path: string;
}

export interface GetSchemaByPathsOptions {
  paths: string[];
}

export interface ReloadSchemaOptions {
  schema: any;
  context?: any;
  data?: any;
}

export interface CacheStats {
  hits: number;
  misses: number;
  entries: number;
}

export interface EvaluateSubformOptions {
  subformPath: string;
  data: any;
  context?: any;
}

export interface ValidateSubformOptions {
  subformPath: string;
  data: any;
  context?: any;
}

export interface EvaluateDependentsSubformOptions {
  subformPath: string;
  changedPaths: string[];
  data?: any;
  context?: any;
  reEvaluate?: boolean;
}

export interface ResolveLayoutSubformOptions {
  subformPath: string;
  evaluate?: boolean;
}

export interface GetEvaluatedSchemaSubformOptions {
  subformPath: string;
  resolveLayout?: boolean;
}

export interface GetSchemaValueSubformOptions {
  subformPath: string;
}

export interface GetEvaluatedSchemaByPathSubformOptions {
  subformPath: string;
  schemaPath: string;
  skipLayout?: boolean;
}

export interface GetEvaluatedSchemaByPathsSubformOptions {
  subformPath: string;
  schemaPaths: string[];
  skipLayout?: boolean;
}

export interface GetSchemaByPathSubformOptions {
  subformPath: string;
  schemaPath: string;
}

export interface GetSchemaByPathsSubformOptions {
  subformPath: string;
  schemaPaths: string[];
}

export interface CompileAndRunLogicOptions {
  logicStr: string | object;
  data?: any;
  context?: any;
}

export class JSONEval {
  constructor(options: JSONEvalOptions);
  static fromCache(cacheKey: string, context?: any, data?: any): JSONEval;
  init(): Promise<void>;
  validate(options: ValidateOptions): Promise<ValidationResult>;
  evaluate(options: EvaluateOptions): Promise<any>;
  evaluateDependents(options: EvaluateDependentsOptions): Promise<DependentChange[]>;
  compileAndRunLogic(options: CompileAndRunLogicOptions): Promise<any>;
  compileLogic(logicStr: string | object): Promise<number>;
  runLogic(logicId: number, data?: any, context?: any): Promise<any>;
  getEvaluatedSchema(options?: GetEvaluatedSchemaOptions): Promise<any>;
  getSchemaValue(): Promise<any>;
  getEvaluatedSchemaWithoutParams(options?: GetEvaluatedSchemaOptions): Promise<any>;
  getValueByPath(options: GetValueByPathOptions): Promise<any | null>;
  getEvaluatedSchemaByPath(options: GetValueByPathOptions): Promise<any | null>;
  getEvaluatedSchemaByPaths(options: GetValueByPathsOptions): Promise<any>;
  getSchemaByPath(options: GetSchemaByPathOptions): Promise<any | null>;
  getSchemaByPaths(options: GetSchemaByPathsOptions): Promise<any>;
  reloadSchema(options: ReloadSchemaOptions): Promise<void>;
  reloadSchemaMsgpack(schemaMsgpack: Uint8Array, context?: any, data?: any): Promise<void>;
  reloadSchemaFromCache(cacheKey: string, context?: any, data?: any): Promise<void>;
  cacheStats(): Promise<CacheStats>;
  clearCache(): Promise<void>;
  cacheLen(): Promise<number>;
  enableCache(): Promise<void>;
  disableCache(): Promise<void>;
  isCacheEnabled(): boolean;
  
  // Subform methods
  evaluateSubform(options: EvaluateSubformOptions): Promise<void>;
  validateSubform(options: ValidateSubformOptions): Promise<ValidationResult>;
  evaluateDependentsSubform(options: EvaluateDependentsSubformOptions): Promise<DependentChange[]>;
  resolveLayoutSubform(options: ResolveLayoutSubformOptions): Promise<void>;
  getEvaluatedSchemaSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any>;
  getSchemaValueSubform(options: GetSchemaValueSubformOptions): Promise<any>;
  getEvaluatedSchemaWithoutParamsSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any>;
  getEvaluatedSchemaByPathSubform(options: GetEvaluatedSchemaByPathSubformOptions): Promise<any | null>;
  getEvaluatedSchemaByPathsSubform(options: GetEvaluatedSchemaByPathsSubformOptions): Promise<any>;
  getSchemaByPathSubform(options: GetSchemaByPathSubformOptions): Promise<any | null>;
  getSchemaByPathsSubform(options: GetSchemaByPathsSubformOptions): Promise<any>;
  getSubformPaths(): Promise<string[]>;
  hasSubform(subformPath: string): Promise<boolean>;
  
  free(): void;
}

export function version(wasmModule?: any): Promise<string>;

export default JSONEval;
