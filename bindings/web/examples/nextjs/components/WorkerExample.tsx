'use client';

import { useState } from 'react';
import { useJSONEvalWorker } from '@/hooks/useJSONEvalWorker';

interface ValidationError {
  path: string;
  rule_type: string;
  message: string;
}

const schema = {
  type: 'object',
  properties: {
    username: {
      type: 'string',
      rules: {
        required: { value: true, message: 'Username is required' },
        minLength: { value: 4, message: 'Username must be at least 4 characters' },
      },
    },
    password: {
      type: 'string',
      rules: {
        required: { value: true, message: 'Password is required' },
        minLength: { value: 8, message: 'Password must be at least 8 characters' },
      },
    },
    confirmPassword: {
      type: 'string',
      rules: {
        required: { value: true, message: 'Please confirm password' },
      },
    },
  },
};

export default function WorkerExample() {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [cacheInfo, setCacheInfo] = useState<{ hits: number; misses: number; entries: number } | null>(
    null
  );

  // Use the reusable hook - WASM runs in Web Worker (off main thread)
  const {
    isReady,
    isLoading,
    error: workerError,
    validate,
    cacheStats,
    clearCache,
  } = useJSONEvalWorker({ schema });

  const handleValidate = async () => {
    if (!isReady) {
      console.error('Worker not ready');
      return;
    }

    const data = {
      username,
      password,
      confirmPassword,
    };

    try {
      // Validation runs in worker (non-blocking)
      const result = await validate(data);
      
      if (result.has_error) {
        const errorMap: Record<string, string> = {};
        result.errors.forEach((error: ValidationError) => {
          errorMap[error.path] = error.message;
        });

        // Custom validation: passwords must match
        if (password !== confirmPassword) {
          errorMap.confirmPassword = 'Passwords do not match';
        }

        setErrors(errorMap);
      } else {
        // Check password match
        if (password !== confirmPassword) {
          setErrors({ confirmPassword: 'Passwords do not match' });
        } else {
          setErrors({});
          alert('✅ Registration form is valid!');
        }
      }

      // Get cache stats
      const stats = await cacheStats();
      setCacheInfo(stats);
    } catch (error) {
      console.error('Validation error:', error);
    }
  };

  const handleClearCache = async () => {
    try {
      await clearCache();
      const stats = await cacheStats();
      setCacheInfo(stats);
      alert('Cache cleared!');
    } catch (error) {
      console.error('Clear cache error:', error);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
        <p className="ml-4 text-gray-600 dark:text-gray-400">Initializing Web Worker...</p>
      </div>
    );
  }

  if (workerError) {
    return (
      <div className="p-4 bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-700 rounded-lg">
        <p className="text-red-800 dark:text-red-200">Worker Error: {workerError}</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold mb-4">Web Worker Example</h2>
        <p className="text-gray-600 dark:text-gray-400 mb-4">
          This example runs WASM validation in a Web Worker (off the main thread) for better
          performance. The UI stays responsive even during heavy computations.
        </p>
        <div className="flex items-center gap-2 p-3 bg-green-50 dark:bg-green-900 rounded-lg">
          <span className="text-green-600 dark:text-green-300 text-xl">⚡</span>
          <span className="text-sm text-green-800 dark:text-green-200">
            Worker Status: <strong>{isReady ? 'Ready' : 'Not Ready'}</strong>
          </span>
        </div>
      </div>

      <div className="space-y-4">
        <div>
          <label htmlFor="username" className="block text-sm font-medium mb-2">
            Username
          </label>
          <input
            id="username"
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
              errors.username ? 'border-red-500' : 'border-gray-300 dark:border-gray-600'
            } bg-white dark:bg-gray-700 text-gray-900 dark:text-white`}
            placeholder="Enter username"
          />
          {errors.username && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.username}</p>
          )}
        </div>

        <div>
          <label htmlFor="password" className="block text-sm font-medium mb-2">
            Password
          </label>
          <input
            id="password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
              errors.password ? 'border-red-500' : 'border-gray-300 dark:border-gray-600'
            } bg-white dark:bg-gray-700 text-gray-900 dark:text-white`}
            placeholder="Enter password"
          />
          {errors.password && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.password}</p>
          )}
        </div>

        <div>
          <label htmlFor="confirmPassword" className="block text-sm font-medium mb-2">
            Confirm Password
          </label>
          <input
            id="confirmPassword"
            type="password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
              errors.confirmPassword ? 'border-red-500' : 'border-gray-300 dark:border-gray-600'
            } bg-white dark:bg-gray-700 text-gray-900 dark:text-white`}
            placeholder="Confirm password"
          />
          {errors.confirmPassword && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">
              {errors.confirmPassword}
            </p>
          )}
        </div>

        <button
          onClick={handleValidate}
          disabled={!isReady}
          className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-semibold py-3 px-6 rounded-lg transition-colors focus:ring-4 focus:ring-blue-300"
        >
          Validate (Web Worker)
        </button>
      </div>

      {cacheInfo && (
        <div className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg space-y-2">
          <div className="flex justify-between items-center">
            <h3 className="font-semibold">Cache Statistics:</h3>
            <button
              onClick={handleClearCache}
              className="text-sm px-3 py-1 bg-red-500 hover:bg-red-600 text-white rounded"
            >
              Clear Cache
            </button>
          </div>
          <div className="grid grid-cols-3 gap-4 text-sm">
            <div>
              <span className="text-gray-600 dark:text-gray-400">Hits:</span>
              <span className="ml-2 font-bold">{cacheInfo.hits}</span>
            </div>
            <div>
              <span className="text-gray-600 dark:text-gray-400">Misses:</span>
              <span className="ml-2 font-bold">{cacheInfo.misses}</span>
            </div>
            <div>
              <span className="text-gray-600 dark:text-gray-400">Entries:</span>
              <span className="ml-2 font-bold">{cacheInfo.entries}</span>
            </div>
          </div>
        </div>
      )}

      <div className="mt-6 p-4 bg-blue-50 dark:bg-blue-900 rounded-lg">
        <h3 className="font-semibold mb-2 text-blue-900 dark:text-blue-100">
          Why Use Web Workers?
        </h3>
        <ul className="text-sm space-y-1 text-blue-800 dark:text-blue-200">
          <li>✅ Non-blocking - UI stays responsive</li>
          <li>✅ Better performance for heavy computations</li>
          <li>✅ Smooth animations during validation</li>
          <li>✅ No frame drops on form interactions</li>
        </ul>
      </div>
    </div>
  );
}
