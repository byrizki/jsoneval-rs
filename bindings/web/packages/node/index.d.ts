/**
 * @json-eval-rs/node - TypeScript definitions
 */

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
  changedPaths: string[];
  data: any;
  context?: any;
  nested?: boolean;
}

export interface GetEvaluatedSchemaOptions {
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

export class JSONEval {
  constructor(options: JSONEvalOptions);
  init(): Promise<void>;
  validate(options: ValidateOptions): Promise<ValidationResult>;
  evaluate(options: EvaluateOptions): Promise<any>;
  evaluateDependents(options: EvaluateDependentsOptions): Promise<any>;
  getEvaluatedSchema(options?: GetEvaluatedSchemaOptions): Promise<any>;
  getSchemaValue(): Promise<any>;
  reloadSchema(options: ReloadSchemaOptions): Promise<void>;
  cacheStats(): Promise<CacheStats>;
  clearCache(): Promise<void>;
  cacheLen(): Promise<number>;
  free(): void;
}

export function version(): string;

export default JSONEval;
