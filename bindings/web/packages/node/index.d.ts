/**
 * @json-eval-rs/node - TypeScript definitions
 */

/**
 * Get the library version
 * @returns Version string
 */
export function version(): string;

export interface ValidationError {
  path: string;
  rule_type: string;
  message: string;
}

export interface ValidationResult {
  has_error: boolean;
  errors: ValidationError[];
}

export interface JSONEvalOptions {
  schema: any;
  context?: any;
  data?: any;
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
  changedPath: string;
  data?: any;
  context?: any;
}

export interface GetEvaluatedSchemaOptions {
  skipLayout?: boolean;
}

export interface GetValueByPathOptions {
  path: string;
  skipLayout?: boolean;
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
  changedPath: string;
  data?: any;
  context?: any;
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

export class JSONEval {
  constructor(options: JSONEvalOptions);
  init(): Promise<void>;
  validate(options: ValidateOptions): Promise<ValidationResult>;
  evaluate(options: EvaluateOptions): Promise<any>;
  evaluateDependents(options: EvaluateDependentsOptions): Promise<any>;
  getEvaluatedSchema(options?: GetEvaluatedSchemaOptions): Promise<any>;
  getSchemaValue(): Promise<any>;
  getEvaluatedSchemaWithoutParams(options?: GetEvaluatedSchemaOptions): Promise<any>;
  getValueByPath(options: GetValueByPathOptions): Promise<any | null>;
  reloadSchema(options: ReloadSchemaOptions): Promise<void>;
  cacheStats(): Promise<CacheStats>;
  clearCache(): Promise<void>;
  cacheLen(): Promise<number>;
  
  // Subform methods
  evaluateSubform(options: EvaluateSubformOptions): Promise<void>;
  validateSubform(options: ValidateSubformOptions): Promise<ValidationResult>;
  evaluateDependentsSubform(options: EvaluateDependentsSubformOptions): Promise<any>;
  resolveLayoutSubform(options: ResolveLayoutSubformOptions): Promise<void>;
  getEvaluatedSchemaSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any>;
  getSchemaValueSubform(options: GetSchemaValueSubformOptions): Promise<any>;
  getEvaluatedSchemaWithoutParamsSubform(options: GetEvaluatedSchemaSubformOptions): Promise<any>;
  getEvaluatedSchemaByPathSubform(options: GetEvaluatedSchemaByPathSubformOptions): Promise<any | null>;
  getSubformPaths(): Promise<string[]>;
  hasSubform(subformPath: string): Promise<boolean>;
  
  free(): void;
}

export function version(): string;

export default JSONEval;
