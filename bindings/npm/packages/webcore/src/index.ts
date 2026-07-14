import {
	type CompileAndRunLogicOptions,
	type DependentChange,
	type EvaluateDependentsOptions,
	type EvaluateDependentsSubformOptions,
	type EvaluateOptions,
	type EvaluateSubformOptions,
	type GetEvaluatedSchemaSubformOptions,
	type GetEvaluatedSchemaByPathSubformOptions,
	type GetEvaluatedSchemaByPathsSubformOptions,
	type GetFieldOptionsOptions,
	type GetSchemaByPathOptions,
	type GetSchemaByPathSubformOptions,
	type GetSchemaByPathsOptions,
	type GetSchemaByPathsSubformOptions,
	type GetSchemaValueSubformOptions,
	type GetValueByPathOptions,
	type GetValueByPathsOptions,
	type JSONEvalOptions,
	type ReloadSchemaOptions,
	type ResolveLayoutSubformOptions,
	type SchemaValueItem,
	type ValidateOptions,
	type ValidateSubformOptions,
	type ValidationResult,
	extractErrorMessage,
	resolveEvaluatedLayout,
	stringifyOrNull,
	stringifyValue,
} from "@json-eval-rs/common";

// Re-export shared types for downstream consumers
export {
	DependentChange,
	EvaluateDependentsOptions,
	EvaluateDependentsSubformOptions,
	EvaluateOptions,
	EvaluateSubformOptions,
	GetEvaluatedSchemaByPathSubformOptions,
	GetEvaluatedSchemaByPathsSubformOptions,
	GetEvaluatedSchemaSubformOptions,
	GetFieldOptionsOptions,
	GetSchemaByPathOptions,
	GetSchemaByPathSubformOptions,
	GetSchemaByPathsOptions,
	GetSchemaByPathsSubformOptions,
	GetSchemaValueSubformOptions,
	GetValueByPathOptions,
	GetValueByPathsOptions,
	JSONEvalOptions,
	LayoutOverlayEntry,
	ReloadSchemaOptions,
	ResolveLayoutSubformOptions,
	ReturnFormat,
	SchemaValueItem,
	ValidateOptions,
	ValidateSubformOptions,
	ValidationError,
	ValidationResult,
	extractErrorMessage,
	mergeLayoutOverlay,
	parseValue,
	resolveEvaluatedLayout,
	stringifyOrNull,
	stringifyValue,
} from "@json-eval-rs/common";

/**
 * @json-eval-rs/webcore
 * High-level JavaScript API for JSON Eval RS WASM bindings
 *
 * This package provides a clean, ergonomic API that works with any WASM target:
 */

/**
 * Get the library version from the WASM module
 * @param wasmModule - WASM module
 * @returns Version string
 */
export function getVersion(wasmModule: any): string {
	if (wasmModule && typeof wasmModule.getVersion === "function") {
		return wasmModule.getVersion();
	}
	return "unknown";
}

/**
 * JSONEval - High-level JavaScript API for JSON Eval RS
 *
 * This is an internal abstraction layer. Use specific packages instead:
 * - @json-eval-rs/bundler (for bundlers like Webpack, Vite, Next.js)
 * - @json-eval-rs/vanilla (for direct browser usage)
 * - @json-eval-rs/node (for Node.js/SSR)
 *
 * @example
 * ```js
 * import { JSONEval } from '@json-eval-rs/webcore';
 *
 * const evaluator = new JSONEval({
 *   schema: { type: 'object', properties: { ... } }
 * });
 *
 * await evaluator.init();
 * const result = await evaluator.validate({ data: { name: 'John' } });
 * ```
 */
export class JSONEvalCore {
	/** Internal storage for the schema (JSON, MsgPack, or Cache Key) */
	private _schema: any;
	/** Reference to the loaded WASM module */
	private _wasmModule: any;
	/** Current context data */
	private _context: any;
	/** Current evaluation data */
	private _data: any;
	/** The underlying WASM JSONEval instance */
	private _instance: any = null;
	/** Initialization state flag */
	private _ready: boolean = false;
	/** Flag indicating if schema provided is binary MessagePack */
	private _isMsgpackSchema: boolean;
	/** Flag indicating if schema is a cache key reference */
	private _isFromCache: boolean;

	/**
	 * Create a new JSONEval Core instance.
	 * Does not initialize WASM immediately; wait for `init()` or call async methods.
	 *
	 * @param wasmModule - The loaded WASM module (provided by wrapper packages like @json-eval-rs/node or /vanilla)
	 * @param options - Configuration options containing schema, data, and context
	 */
	constructor(
		wasmModule: any,
		{ schema, context, data, fromCache = false }: JSONEvalOptions,
	) {
		this._schema = schema;
		this._wasmModule = wasmModule;
		this._context = context;
		this._data = data;
		this._instance = null;
		this._ready = false;
		this._isMsgpackSchema = schema instanceof Uint8Array;
		this._isFromCache = fromCache;
	}

	/**
	 * Initialize the WASM instance
	 * Call this before using other methods, or use the async methods which call it automatically
	 */
	async init(): Promise<void> {
		if (this._ready) return;

		if (!this._wasmModule) {
			throw new Error(
				"No WASM module provided. Please either:\n" +
					'1. Pass wasmModule in constructor: new JSONEval({ schema, wasmModule: await import("@json-eval-rs/bundler") })\n' +
					"2. Or install a peer dependency: yarn install @json-eval-rs/bundler (or @json-eval-rs/vanilla or @json-eval-rs/node)",
			);
		}

		try {
			const { JSONEvalWasm } = this._wasmModule;

			// Create instance from cache, MessagePack, or JSON
			if (this._isFromCache) {
				this._instance = JSONEvalWasm.newFromCache(
					this._schema,
					stringifyOrNull(this._context),
					stringifyOrNull(this._data),
				);
			} else if (this._isMsgpackSchema) {
				this._instance = JSONEvalWasm.newFromMsgpack(
					this._schema,
					stringifyOrNull(this._context),
					stringifyOrNull(this._data),
				);
			} else {
				this._instance = new JSONEvalWasm(
					typeof this._schema === "string"
						? this._schema
						: stringifyValue(this._schema),
					stringifyOrNull(this._context),
					stringifyOrNull(this._data),
				);
			}
			this._ready = true;
		} catch (error: any) {
			throw new Error(
				`Failed to create JSONEval instance: ${extractErrorMessage(error)}`,
			);
		}
	}

	/**
	 * Create a new JSONEval instance from a cached ParsedSchema
	 */
	static fromCache(
		wasmModule: any,
		cacheKey: string,
		context?: any,
		data?: any,
	): JSONEvalCore {
		return new JSONEvalCore(wasmModule, {
			schema: cacheKey,
			context,
			data,
			fromCache: true,
		});
	}

	/**
	 * Create a new JSONEval instance from a MessagePack schema
	 */
	static fromMsgpack(
		wasmModule: any,
		schemaMsgpack: Uint8Array,
		context?: any,
		data?: any,
	): JSONEvalCore {
		return new JSONEvalCore(wasmModule, {
			schema: schemaMsgpack,
			context,
			data,
		});
	}

	/**
	 * Get the WASM library version
	 */
	static version(wasmModule: any): string {
		return getVersion(wasmModule);
	}

	/**
	 * Evaluate logic expression without creating an instance
	 */
	static evaluateLogic(
		wasmModule: any,
		logicStr: string | object,
		data?: any,
		context?: any,
	): any {
		if (!wasmModule) {
			throw new Error("No WASM module provided.");
		}
		const { JSONEvalWasm } = wasmModule;
		if (!JSONEvalWasm || typeof JSONEvalWasm.evaluateLogic !== "function") {
			throw new Error("WASM module does not support evaluateLogic.");
		}

		const logic =
			typeof logicStr === "string" ? logicStr : stringifyValue(logicStr);

		return JSONEvalWasm.evaluateLogic(
			logic,
			stringifyOrNull(data),
			stringifyOrNull(context),
		);
	}

	/**
	 * Validate data against schema (returns parsed JavaScript object)
	 * Uses validateJS for Worker-safe serialization
	 */
	async validate({
		data,
		context,
	}: ValidateOptions): Promise<ValidationResult> {
		await this.init();
		try {
			return this._instance.validateJS(
				stringifyOrNull(data)!,
				stringifyOrNull(context),
			);
		} catch (error: any) {
			throw new Error(`Validation failed: ${extractErrorMessage(error)}`);
		}
	}

	/**
	 * Evaluate schema with data (returns parsed JavaScript object)
	 */
	async evaluate({ data, context, paths }: EvaluateOptions): Promise<any> {
		await this.init();
		try {
			return this._instance.evaluateJS(
				stringifyOrNull(data)!,
				stringifyOrNull(context),
				paths || null,
			);
		} catch (error: any) {
			throw new Error(`Evaluation failed: ${extractErrorMessage(error)}`);
		}
	}

	/**
	 * Evaluate schema with data (only updates internal state, returns void)
	 */
	async evaluateOnly({ data, context, paths }: EvaluateOptions): Promise<void> {
		await this.init();
		try {
			this._instance.evaluate(
				stringifyOrNull(data)!,
				stringifyOrNull(context),
				paths || null,
			);
		} catch (error: any) {
			throw new Error(`Evaluation failed: ${extractErrorMessage(error)}`);
		}
	}

	/**
	 * Evaluate dependent fields (returns parsed JavaScript object, processes transitively)
	 */
	async evaluateDependents({
		changedPaths,
		data,
		context,
		reEvaluate = true,
		includeSubforms = true,
	}: EvaluateDependentsOptions): Promise<DependentChange[]> {
		await this.init();
		try {
			const paths = Array.isArray(changedPaths) ? changedPaths : [changedPaths];

			return this._instance.evaluateDependentsJS(
				typeof paths === "string" ? paths : stringifyValue(paths),
				stringifyOrNull(data),
				stringifyOrNull(context),
				reEvaluate,
				includeSubforms,
			);
		} catch (error: any) {
			throw new Error(
				`Dependent evaluation failed: ${extractErrorMessage(error)}`,
			);
		}
	}

	/**
	 * Evaluate dependents (returns JSON string)
	 */
	async evaluateDependentsString({
		changedPath,
		data,
		context,
		reEvaluate = true,
		includeSubforms = true,
	}: {
		changedPath: string;
		data?: any;
		context?: any;
		reEvaluate?: boolean;
		includeSubforms?: boolean;
	}): Promise<string> {
		await this.init();
		try {
			return this._instance.evaluateDependents(
				changedPath,
				stringifyOrNull(data),
				stringifyOrNull(context),
				reEvaluate,
				includeSubforms,
			);
		} catch (error: any) {
			throw new Error(
				`Dependent evaluation failed: ${extractErrorMessage(error)}`,
			);
		}
	}

	/**
	 * Get evaluated schema (compact, without $layout resolution)
	 */
	async getEvaluatedSchema(): Promise<any> {
		await this.init();
		return this._instance.getEvaluatedSchemaJS();
	}

	/**
	 * Get resolved layout overlay entries
	 */
	async getResolvedLayout(): Promise<any[]> {
		await this.init();
		return this._instance.getResolvedLayout();
	}

	/**
	 * Get evaluated schema with layout fully resolved
	 */
	async getEvaluatedSchemaResolved(): Promise<any> {
		await this.init();
		return resolveEvaluatedLayout(
			() => this.getEvaluatedSchemaWithoutParams(),
			() => this.getResolvedLayout(),
		);
	}

	/**
	 * Get evaluated schema as MessagePack binary data
	 */
	async getEvaluatedSchemaMsgpack(): Promise<Uint8Array> {
		await this.init();
		return this._instance.getEvaluatedSchemaMsgpack();
	}

	/**
	 * Get schema values (evaluations ending with .value)
	 */
	async getSchemaValue(): Promise<any> {
		await this.init();
		return this._instance.getSchemaValue();
	}

	/**
	 * Get all schema values as array of path-value pairs
	 */
	async getSchemaValueArray(): Promise<SchemaValueItem[]> {
		await this.init();
		return this._instance.getSchemaValueArray();
	}

	/**
	 * Get all schema values as object with dotted path keys
	 */
	async getSchemaValueObject(): Promise<Record<string, any>> {
		await this.init();
		return this._instance.getSchemaValueObject();
	}

	/**
	 * Get evaluated schema without $params field (compact)
	 */
	async getEvaluatedSchemaWithoutParams(): Promise<any> {
		await this.init();
		return this._instance.getEvaluatedSchemaWithoutParamsJS();
	}

	/**
	 * Get a value from the evaluated schema using dotted path notation (compact)
	 */
	async getEvaluatedSchemaByPath({
		path,
	}: GetValueByPathOptions): Promise<any | null> {
		await this.init();
		return this._instance.getEvaluatedSchemaByPathJS(path);
	}

	/**
	 * Get values from the evaluated schema using multiple dotted path notations (compact)
	 */
	async getEvaluatedSchemaByPaths({
		paths,
		format = 0,
	}: GetValueByPathsOptions): Promise<any> {
		await this.init();
		return this._instance.getEvaluatedSchemaByPathsJS(
			typeof paths === "string" ? paths : stringifyValue(paths),
			format,
		);
	}

	/**
	 * Get a value from the schema using dotted path notation
	 */
	async getSchemaByPath({ path }: GetSchemaByPathOptions): Promise<any | null> {
		await this.init();
		return this._instance.getSchemaByPathJS(path);
	}

	/**
	 * Get a value from the schema using dotted path notation
	 * Alias for getSchemaByPath to match React Native API
	 */
	async get(path: string): Promise<any | null> {
		return this.getSchemaByPath({ path });
	}

	/**
	 * Get values from the schema using multiple dotted path notations
	 */
	async getSchemaByPaths({
		paths,
		format = 0,
	}: GetSchemaByPathsOptions): Promise<any> {
		await this.init();
		return this._instance.getSchemaByPathsJS(
			typeof paths === "string" ? paths : stringifyValue(paths),
			format,
		);
	}

	/**
	 * Reload schema with new data
	 */
	async reloadSchema({
		schema,
		context,
		data,
	}: ReloadSchemaOptions): Promise<void> {
		if (!this._instance) {
			throw new Error("Instance not initialized. Call init() first.");
		}

		try {
			await this._instance.reloadSchema(
				typeof schema === "string" ? schema : stringifyValue(schema),
				stringifyOrNull(context),
				stringifyOrNull(data),
			);

			this._schema = schema;
			this._context = context;
			this._data = data;
		} catch (error: any) {
			throw new Error(`Failed to reload schema: ${extractErrorMessage(error)}`);
		}
	}

	/**
	 * Reload schema from MessagePack bytes
	 */
	async reloadSchemaMsgpack(
		schemaMsgpack: Uint8Array,
		context?: any,
		data?: any,
	): Promise<void> {
		if (!this._instance) {
			throw new Error("Instance not initialized. Call init() first.");
		}

		if (!(schemaMsgpack instanceof Uint8Array)) {
			throw new Error("schemaMsgpack must be a Uint8Array");
		}

		try {
			await this._instance.reloadSchemaMsgpack(
				schemaMsgpack,
				stringifyOrNull(context),
				stringifyOrNull(data),
			);

			this._schema = schemaMsgpack;
			this._context = context;
			this._data = data;
			this._isMsgpackSchema = true;
		} catch (error: any) {
			throw new Error(
				`Failed to reload schema from MessagePack: ${extractErrorMessage(error)}`,
			);
		}
	}

	/**
	 * Reload schema from ParsedSchemaCache using a cache key
	 */
	async reloadSchemaFromCache(
		cacheKey: string,
		context?: any,
		data?: any,
	): Promise<void> {
		if (!this._instance) {
			throw new Error("Instance not initialized. Call init() first.");
		}

		if (typeof cacheKey !== "string" || !cacheKey) {
			throw new Error("cacheKey must be a non-empty string");
		}

		try {
			await this._instance.reloadSchemaFromCache(
				cacheKey,
				stringifyOrNull(context),
				stringifyOrNull(data),
			);

			this._context = context;
			this._data = data;
		} catch (error: any) {
			throw new Error(
				`Failed to reload schema from cache: ${extractErrorMessage(error)}`,
			);
		}
	}

	/**
	 * Resolve layout with optional evaluation, returning overlay entries
	 */
	async resolveLayout(options: { evaluate?: boolean } = {}): Promise<any[]> {
		const { evaluate = false } = options;
		await this.init();
		return this._instance.resolveLayout(evaluate);
	}

	/**
	 * Set timezone offset for datetime operations (TODAY, NOW)
	 */
	setTimezoneOffset(offsetMinutes: number | null | undefined): void {
		if (!this._instance) {
			throw new Error("Instance not initialized. Call init() first.");
		}
		this._instance.setTimezoneOffset(offsetMinutes);
	}

	/**
	 * Compile and run JSON logic from a JSON logic string
	 */
	async compileAndRunLogic({
		logicStr,
		data,
		context,
	}: CompileAndRunLogicOptions): Promise<any> {
		await this.init();
		const logic =
			typeof logicStr === "string" ? logicStr : stringifyValue(logicStr);
		const result = await this._instance.compileAndRunLogic(
			logic,
			stringifyOrNull(data),
			stringifyOrNull(context),
		);
		return result;
	}

	/**
	 * Compile JSON logic and return a global ID
	 */
	async compileLogic(logicStr: string | object): Promise<number> {
		await this.init();
		const logic =
			typeof logicStr === "string" ? logicStr : stringifyValue(logicStr);
		return this._instance.compileLogic(logic);
	}

	/**
	 * Run pre-compiled logic by ID
	 */
	async runLogic(logicId: number, data?: any, context?: any): Promise<any> {
		await this.init();
		const result = await this._instance.runLogic(
			logicId,
			stringifyOrNull(data),
			stringifyOrNull(context),
		);
		return result;
	}

	/**
	 * Validate data against schema rules with optional path filtering
	 */
	async validatePaths({
		data,
		context,
		paths,
	}: EvaluateOptions): Promise<ValidationResult> {
		await this.init();
		try {
			return this._instance.validatePathsJS(
				stringifyOrNull(data)!,
				stringifyOrNull(context),
				paths || null,
			);
		} catch (error: any) {
			throw new Error(`Validation failed: ${extractErrorMessage(error)}`);
		}
	}

	/**
	 * Validate data with path filtering (returns WASM ValidationResult object)
	 * Note: You must call .free() on the returned object when done.
	 */
	async validatePathsOnly({
		data,
		context,
		paths,
	}: EvaluateOptions): Promise<any> {
		await this.init();
		try {
			return this._instance.validatePaths(
				stringifyOrNull(data)!,
				stringifyOrNull(context),
				paths || null,
			);
		} catch (error: any) {
			throw new Error(`Validation failed: ${extractErrorMessage(error)}`);
		}
	}

	/**
	 * Cancel any running evaluation
	 */
	async cancel(): Promise<void> {
		if (this._ready && this._instance) {
			this._instance.cancel();
		}
	}

	// ============================================================================
	// Subform Methods
	// ============================================================================

	/**
	 * Evaluate a subform with data
	 */
	async evaluateSubform({
		subformPath,
		data,
		context,
		paths,
	}: EvaluateSubformOptions): Promise<void> {
		await this.init();
		return this._instance.evaluateSubform(
			subformPath,
			stringifyOrNull(data)!,
			stringifyOrNull(context),
			paths || null,
		);
	}

	/**
	 * Validate subform data against its schema rules
	 */
	async validateSubform({
		subformPath,
		data,
		context,
	}: ValidateSubformOptions): Promise<ValidationResult> {
		await this.init();
		try {
			return this._instance.validateSubformJS(
				subformPath,
				stringifyOrNull(data)!,
				stringifyOrNull(context),
			);
		} catch (error: any) {
			throw new Error(
				`Subform validation failed: ${extractErrorMessage(error)}`,
			);
		}
	}

	/**
	 * Evaluate dependent fields in subform
	 */
	async evaluateDependentsSubform({
		subformPath,
		changedPaths,
		data,
		context,
		reEvaluate = true,
		includeSubforms = true,
	}: EvaluateDependentsSubformOptions): Promise<DependentChange[]> {
		await this.init();

		const paths = Array.isArray(changedPaths) ? changedPaths : [changedPaths];

		return this._instance.evaluateDependentsSubformJS(
			subformPath,
			typeof paths === "string" ? paths : stringifyValue(paths),
			stringifyOrNull(data),
			stringifyOrNull(context),
			reEvaluate,
			includeSubforms,
		);
	}

	/**
	 * Evaluate dependent fields in subform (returns JSON string)
	 */
	async evaluateDependentsSubformString({
		subformPath,
		changedPaths,
		data,
		context,
		includeSubforms = true,
	}: EvaluateDependentsSubformOptions): Promise<string> {
		await this.init();
		const paths = Array.isArray(changedPaths) ? changedPaths : [changedPaths];
		return this._instance.evaluateDependentsSubform(
			subformPath,
			typeof paths === "string" ? paths : stringifyValue(paths),
			stringifyOrNull(data),
			stringifyOrNull(context),
			false,
			includeSubforms,
		);
	}

	/**
	 * Resolve layout for subform, returning overlay entries
	 */
	async resolveLayoutSubform({
		subformPath,
		evaluate = false,
	}: ResolveLayoutSubformOptions): Promise<any[]> {
		await this.init();
		return this._instance.resolveLayoutSubform(subformPath, evaluate);
	}

	/**
	 * Get evaluated schema from subform (compact, without $layout resolution)
	 */
	async getEvaluatedSchemaSubform({
		subformPath,
	}: GetEvaluatedSchemaSubformOptions): Promise<any> {
		await this.init();
		return this._instance.getEvaluatedSchemaSubformJS(subformPath);
	}

	/**
	 * Get resolved layout overlay entries for subform
	 */
	async getResolvedLayoutSubform({
		subformPath,
	}: GetEvaluatedSchemaSubformOptions): Promise<any[]> {
		await this.init();
		return this._instance.getResolvedLayoutSubform(subformPath);
	}

	/**
	 * Get evaluated schema with layout fully resolved for subform
	 */
	async getEvaluatedSchemaResolvedSubform({
		subformPath,
	}: GetEvaluatedSchemaSubformOptions): Promise<any> {
		await this.init();
		return resolveEvaluatedLayout(
			() => this.getEvaluatedSchemaWithoutParamsSubform({ subformPath }),
			() => this.getResolvedLayoutSubform({ subformPath }),
		);
	}

	/**
	 * Get schema value from subform
	 */
	async getSchemaValueSubform({
		subformPath,
	}: GetSchemaValueSubformOptions): Promise<any> {
		await this.init();
		return this._instance.getSchemaValueSubform(subformPath);
	}

	/**
	 * Get schema values from subform as array of path-value pairs
	 */
	async getSchemaValueArraySubform({
		subformPath,
	}: GetSchemaValueSubformOptions): Promise<SchemaValueItem[]> {
		await this.init();
		return this._instance.getSchemaValueArraySubform(subformPath);
	}

	/**
	 * Get schema values from subform as object with dotted path keys
	 */
	async getSchemaValueObjectSubform({
		subformPath,
	}: GetSchemaValueSubformOptions): Promise<Record<string, any>> {
		await this.init();
		return this._instance.getSchemaValueObjectSubform(subformPath);
	}

	/**
	 * Get evaluated schema without $params from subform (compact)
	 */
	async getEvaluatedSchemaWithoutParamsSubform({
		subformPath,
	}: GetEvaluatedSchemaSubformOptions): Promise<any> {
		await this.init();
		return this._instance.getEvaluatedSchemaWithoutParamsSubformJS(subformPath);
	}

	/**
	 * Get evaluated schema by specific path from subform (compact)
	 */
	async getEvaluatedSchemaByPathSubform({
		subformPath,
		schemaPath,
	}: GetEvaluatedSchemaByPathSubformOptions): Promise<any | null> {
		await this.init();
		return this._instance.getEvaluatedSchemaByPathSubformJS(
			subformPath,
			schemaPath,
		);
	}

	/**
	 * Get evaluated schema by multiple paths from subform (compact)
	 */
	async getEvaluatedSchemaByPathsSubform({
		subformPath,
		schemaPaths,
		format = 0,
	}: GetEvaluatedSchemaByPathsSubformOptions): Promise<any> {
		await this.init();
		return this._instance.getEvaluatedSchemaByPathsSubformJS(
			subformPath,
			typeof schemaPaths === "string"
				? schemaPaths
				: stringifyValue(schemaPaths),
			format,
		);
	}

	/**
	 * Get list of available subform paths
	 */
	async getSubformPaths(): Promise<string[]> {
		await this.init();
		return this._instance.getSubformPaths();
	}

	/**
	 * Get schema by specific path from subform
	 */
	async getSchemaByPathSubform({
		subformPath,
		schemaPath,
	}: GetSchemaByPathSubformOptions): Promise<any | null> {
		await this.init();
		return this._instance.getSchemaByPathSubformJS(subformPath, schemaPath);
	}

	/**
	 * Get schema by multiple paths from subform
	 */
	async getSchemaByPathsSubform({
		subformPath,
		schemaPaths,
		format = 0,
	}: GetSchemaByPathsSubformOptions): Promise<any> {
		await this.init();
		return this._instance.getSchemaByPathsSubformJS(
			subformPath,
			typeof schemaPaths === "string"
				? schemaPaths
				: stringifyValue(schemaPaths),
			format,
		);
	}

	/**
	 * Check if a subform exists at the given path
	 */
	async hasSubform(subformPath: string): Promise<boolean> {
		await this.init();
		return this._instance.hasSubform(subformPath);
	}

	/**
	 * Evaluate and return the options for a specific field on demand.
	 *
	 * The field is identified by `path`, which can be:
	 * - Dotted notation: `"form.occupation"`
	 * - JSON pointer: `"/properties/form/properties/occupation"`
	 * - Schema ref: `"#/properties/form/properties/occupation"`
	 *
	 * @returns Resolved options value (array or URL string), or null if the field has no options
	 */
	async getFieldOptions({ path }: GetFieldOptionsOptions): Promise<any | null> {
		await this.init();
		return this._instance.getFieldOptionsJS(path);
	}

	/**
	 * Free WASM resources
	 */
	free(): void {
		if (this._instance) {
			this._instance.free();
			this._instance = null;
			this._ready = false;
		}
	}

	/**
	 * Alias for free() to match React Native API
	 */
	dispose(): void {
		this.free();
	}
}

export default JSONEvalCore;
