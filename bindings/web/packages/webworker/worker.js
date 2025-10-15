/**
 * Web Worker script for JSON Eval RS
 * This runs WASM operations off the main thread
 */

let wasmInstance = null;

// Listen for messages from main thread
self.addEventListener('message', async (event) => {
  const { id, type, payload } = event.data;

  try {
    switch (type) {
      case 'INIT': {
        // Import and initialize WASM module
        const wasmModule = await import('@json-eval-rs/bundler');
        const { JSONEvalWasm } = wasmModule;
        
        wasmInstance = new JSONEvalWasm(
          payload.schema,
          payload.context || null,
          payload.data || null
        );
        
        self.postMessage({ id, type: 'INIT_SUCCESS' });
        break;
      }

      case 'VALIDATE': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.validate(
          payload.data,
          payload.context || null
        );
        
        self.postMessage({ id, type: 'VALIDATE_SUCCESS', result });
        break;
      }

      case 'EVALUATE': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.evaluateJS(
          payload.data,
          payload.context || null
        );
        
        self.postMessage({ id, type: 'EVALUATE_SUCCESS', result });
        break;
      }

      case 'EVALUATE_DEPENDENTS': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.evaluateDependentsJS(
          payload.changedPaths,
          payload.data,
          payload.context || null,
          payload.nested !== undefined ? payload.nested : true
        );
        
        self.postMessage({ id, type: 'EVALUATE_DEPENDENTS_SUCCESS', result });
        break;
      }

      case 'GET_EVALUATED_SCHEMA': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.getEvaluatedSchemaJS(
          payload.skipLayout || false
        );
        
        self.postMessage({ id, type: 'GET_EVALUATED_SCHEMA_SUCCESS', result });
        break;
      }

      case 'GET_SCHEMA_VALUE': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.getSchemaValue();
        
        self.postMessage({ id, type: 'GET_SCHEMA_VALUE_SUCCESS', result });
        break;
      }

      case 'RELOAD_SCHEMA': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        await wasmInstance.reloadSchema(
          payload.schema,
          payload.context || null,
          payload.data || null
        );
        
        self.postMessage({ id, type: 'RELOAD_SCHEMA_SUCCESS' });
        break;
      }

      case 'CACHE_STATS': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.cacheStats();
        
        self.postMessage({ id, type: 'CACHE_STATS_SUCCESS', result });
        break;
      }

      case 'CLEAR_CACHE': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        wasmInstance.clearCache();
        
        self.postMessage({ id, type: 'CLEAR_CACHE_SUCCESS' });
        break;
      }

      case 'CACHE_LEN': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = wasmInstance.cacheLen();
        
        self.postMessage({ id, type: 'CACHE_LEN_SUCCESS', result });
        break;
      }

      case 'FREE': {
        if (wasmInstance) {
          wasmInstance.free();
          wasmInstance = null;
        }
        self.postMessage({ id, type: 'FREE_SUCCESS' });
        break;
      }

      default:
        throw new Error(`Unknown message type: ${type}`);
    }
  } catch (error) {
    self.postMessage({
      id,
      type: 'ERROR',
      error: {
        message: error.message,
        stack: error.stack
      }
    });
  }
});
