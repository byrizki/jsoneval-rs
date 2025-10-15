/**
 * Web Worker for JSON Eval RS
 * Runs WASM operations off the main thread for better performance
 */

let wasmInstance: any = null;

type MessageType =
  | 'INIT'
  | 'VALIDATE'
  | 'EVALUATE'
  | 'EVALUATE_DEPENDENTS'
  | 'GET_EVALUATED_SCHEMA'
  | 'GET_SCHEMA_VALUE'
  | 'RELOAD_SCHEMA'
  | 'CACHE_STATS'
  | 'CLEAR_CACHE'
  | 'CACHE_LEN'
  | 'FREE';

interface WorkerMessage {
  id: number;
  type: MessageType;
  payload?: any;
}

interface WorkerResponse {
  id: number;
  type: string;
  result?: any;
  error?: { message: string; stack?: string };
}

// Listen for messages from main thread
self.addEventListener('message', async (event: MessageEvent<WorkerMessage>) => {
  const { id, type, payload } = event.data;

  try {
    switch (type) {
      case 'INIT': {
        // Dynamically import WASM module
        const { JSONEval } = await import('@json-eval-rs/bundler');
        
        wasmInstance = new JSONEval({
          schema: JSON.parse(payload.schema),
          context: payload.context ? JSON.parse(payload.context) : undefined,
          data: payload.data ? JSON.parse(payload.data) : undefined,
        });

        await wasmInstance.init();
        
        self.postMessage({ id, type: 'INIT_SUCCESS' } as WorkerResponse);
        break;
      }

      case 'VALIDATE': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        // validate() now uses validateJS internally for proper serialization
        const result = await wasmInstance.validate({
          data: JSON.parse(payload.data),
          context: payload.context ? JSON.parse(payload.context) : undefined,
        });
        
        // Result is already a plain JS object, safe for postMessage
        self.postMessage({ id, type: 'VALIDATE_SUCCESS', result } as WorkerResponse);
        break;
      }

      case 'EVALUATE': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.evaluate({
          data: JSON.parse(payload.data),
          context: payload.context ? JSON.parse(payload.context) : undefined,
        });
        
        self.postMessage({ id, type: 'EVALUATE_SUCCESS', result } as WorkerResponse);
        break;
      }

      case 'EVALUATE_DEPENDENTS': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.evaluateDependents({
          changedPaths: payload.changedPaths,
          data: JSON.parse(payload.data),
          context: payload.context ? JSON.parse(payload.context) : undefined,
          nested: payload.nested !== undefined ? payload.nested : true,
        });
        
        self.postMessage({ id, type: 'EVALUATE_DEPENDENTS_SUCCESS', result } as WorkerResponse);
        break;
      }

      case 'GET_EVALUATED_SCHEMA': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.getEvaluatedSchema({
          skipLayout: payload.skipLayout || false,
        });
        
        self.postMessage({ id, type: 'GET_EVALUATED_SCHEMA_SUCCESS', result } as WorkerResponse);
        break;
      }

      case 'GET_SCHEMA_VALUE': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.getSchemaValue();
        
        self.postMessage({ id, type: 'GET_SCHEMA_VALUE_SUCCESS', result } as WorkerResponse);
        break;
      }

      case 'RELOAD_SCHEMA': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        await wasmInstance.reloadSchema({
          schema: JSON.parse(payload.schema),
          context: payload.context ? JSON.parse(payload.context) : undefined,
          data: payload.data ? JSON.parse(payload.data) : undefined,
        });
        
        self.postMessage({ id, type: 'RELOAD_SCHEMA_SUCCESS' } as WorkerResponse);
        break;
      }

      case 'CACHE_STATS': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.cacheStats();
        
        self.postMessage({ id, type: 'CACHE_STATS_SUCCESS', result } as WorkerResponse);
        break;
      }

      case 'CLEAR_CACHE': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        await wasmInstance.clearCache();
        
        self.postMessage({ id, type: 'CLEAR_CACHE_SUCCESS' } as WorkerResponse);
        break;
      }

      case 'CACHE_LEN': {
        if (!wasmInstance) {
          throw new Error('WASM not initialized. Call init() first.');
        }
        
        const result = await wasmInstance.cacheLen();
        
        self.postMessage({ id, type: 'CACHE_LEN_SUCCESS', result } as WorkerResponse);
        break;
      }

      case 'FREE': {
        if (wasmInstance) {
          wasmInstance.free();
          wasmInstance = null;
        }
        self.postMessage({ id, type: 'FREE_SUCCESS' } as WorkerResponse);
        break;
      }

      default:
        throw new Error(`Unknown message type: ${type}`);
    }
  } catch (error: any) {
    self.postMessage({
      id,
      type: 'ERROR',
      error: {
        message: error.message,
        stack: error.stack,
      },
    } as WorkerResponse);
  }
});

export {};
