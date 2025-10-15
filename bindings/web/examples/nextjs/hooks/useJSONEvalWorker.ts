/**
 * React Hook for JSON Eval RS Web Worker
 * Provides all JSONEval methods running off the main thread
 */

import { useEffect, useRef, useState, useCallback } from 'react';

interface ValidationError {
  path: string;
  rule_type: string;
  message: string;
}

interface ValidationResult {
  has_error: boolean;
  errors: ValidationError[];
}

interface CacheStats {
  hits: number;
  misses: number;
  entries: number;
}

interface UseJSONEvalWorkerOptions {
  schema: any;
  context?: any;
  data?: any;
}

interface UseJSONEvalWorkerReturn {
  isReady: boolean;
  isLoading: boolean;
  error: string | null;
  validate: (data: any, context?: any) => Promise<ValidationResult>;
  evaluate: (data: any, context?: any) => Promise<any>;
  evaluateDependents: (
    changedPaths: string[],
    data: any,
    context?: any,
    nested?: boolean
  ) => Promise<any>;
  getEvaluatedSchema: (skipLayout?: boolean) => Promise<any>;
  getSchemaValue: () => Promise<any>;
  reloadSchema: (schema: any, context?: any, data?: any) => Promise<void>;
  cacheStats: () => Promise<CacheStats>;
  clearCache: () => Promise<void>;
  cacheLen: () => Promise<number>;
}

let messageId = 0;

export function useJSONEvalWorker({
  schema,
  context,
  data,
}: UseJSONEvalWorkerOptions): UseJSONEvalWorkerReturn {
  const workerRef = useRef<Worker | null>(null);
  const pendingRef = useRef<Map<number, { resolve: (value: any) => void; reject: (error: Error) => void }>>(
    new Map()
  );

  const [isReady, setIsReady] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Send message to worker
  const sendMessage = useCallback((type: string, payload?: any): Promise<any> => {
    return new Promise((resolve, reject) => {
      if (!workerRef.current) {
        reject(new Error('Worker not initialized'));
        return;
      }

      const id = messageId++;
      pendingRef.current.set(id, { resolve, reject });

      workerRef.current.postMessage({ id, type, payload });

      // Timeout after 30 seconds
      setTimeout(() => {
        if (pendingRef.current.has(id)) {
          pendingRef.current.delete(id);
          reject(new Error('Worker timeout'));
        }
      }, 30000);
    });
  }, []);

  // Initialize worker
  useEffect(() => {
    try {
      // Create worker
      const worker = new Worker(new URL('../workers/json-eval.worker.ts', import.meta.url));

      // Handle messages from worker
      worker.addEventListener('message', (event) => {
        const { id, type, result, error: workerError } = event.data;
        const pending = pendingRef.current.get(id);

        if (!pending) return;

        pendingRef.current.delete(id);

        if (type === 'ERROR') {
          pending.reject(new Error(workerError.message));
        } else {
          pending.resolve(result);
        }
      });

      // Handle worker errors
      worker.addEventListener('error', (event) => {
        console.error('Worker error:', event);
        setError(event.message);
        setIsLoading(false);
      });

      workerRef.current = worker;

      // Initialize WASM in worker
      sendMessage('INIT', {
        schema: JSON.stringify(schema),
        context: context ? JSON.stringify(context) : null,
        data: data ? JSON.stringify(data) : null,
      })
        .then(() => {
          setIsReady(true);
          setIsLoading(false);
        })
        .catch((err) => {
          console.error('Failed to initialize worker:', err);
          setError(err.message);
          setIsLoading(false);
        });

      // Cleanup
      return () => {
        if (workerRef.current) {
          sendMessage('FREE', {}).catch(console.error);
          workerRef.current.terminate();
          workerRef.current = null;
        }
        pendingRef.current.clear();
      };
    } catch (err: any) {
      console.error('Failed to create worker:', err);
      setError(err.message);
      setIsLoading(false);
    }
  }, []); // Empty deps - only initialize once

  // API methods
  const validate = useCallback(
    async (data: any, context?: any): Promise<ValidationResult> => {
      return sendMessage('VALIDATE', {
        data: JSON.stringify(data),
        context: context ? JSON.stringify(context) : null,
      });
    },
    [sendMessage]
  );

  const evaluate = useCallback(
    async (data: any, context?: any): Promise<any> => {
      return sendMessage('EVALUATE', {
        data: JSON.stringify(data),
        context: context ? JSON.stringify(context) : null,
      });
    },
    [sendMessage]
  );

  const evaluateDependents = useCallback(
    async (
      changedPaths: string[],
      data: any,
      context?: any,
      nested: boolean = true
    ): Promise<any> => {
      return sendMessage('EVALUATE_DEPENDENTS', {
        changedPaths,
        data: JSON.stringify(data),
        context: context ? JSON.stringify(context) : null,
        nested,
      });
    },
    [sendMessage]
  );

  const getEvaluatedSchema = useCallback(
    async (skipLayout: boolean = false): Promise<any> => {
      return sendMessage('GET_EVALUATED_SCHEMA', { skipLayout });
    },
    [sendMessage]
  );

  const getSchemaValue = useCallback(async (): Promise<any> => {
    return sendMessage('GET_SCHEMA_VALUE', {});
  }, [sendMessage]);

  const reloadSchema = useCallback(
    async (schema: any, context?: any, data?: any): Promise<void> => {
      await sendMessage('RELOAD_SCHEMA', {
        schema: JSON.stringify(schema),
        context: context ? JSON.stringify(context) : null,
        data: data ? JSON.stringify(data) : null,
      });
    },
    [sendMessage]
  );

  const cacheStats = useCallback(async (): Promise<CacheStats> => {
    return sendMessage('CACHE_STATS', {});
  }, [sendMessage]);

  const clearCache = useCallback(async (): Promise<void> => {
    await sendMessage('CLEAR_CACHE', {});
  }, [sendMessage]);

  const cacheLen = useCallback(async (): Promise<number> => {
    return sendMessage('CACHE_LEN', {});
  }, [sendMessage]);

  return {
    isReady,
    isLoading,
    error,
    validate,
    evaluate,
    evaluateDependents,
    getEvaluatedSchema,
    getSchemaValue,
    reloadSchema,
    cacheStats,
    clearCache,
    cacheLen,
  };
}
