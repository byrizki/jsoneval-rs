'use client';

import { useState, useEffect } from 'react';

interface ValidationResult {
  has_error: boolean;
  errors: ValidationError[];
}

interface ValidationError {
  path: string;
  rule_type: string;
  message: string;
}

const schema = {
  type: 'object',
  properties: {
    name: {
      type: 'string',
      rules: {
        required: { value: true, message: 'Name is required' },
        minLength: { value: 3, message: 'Name must be at least 3 characters' },
      },
    },
    email: {
      type: 'string',
      rules: {
        required: { value: true, message: 'Email is required' },
        pattern: {
          value: '^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$',
          message: 'Invalid email format',
        },
      },
    },
    age: {
      type: 'number',
      rules: {
        required: { value: true, message: 'Age is required' },
        minValue: { value: 18, message: 'Must be at least 18 years old' },
        maxValue: { value: 120, message: 'Age must be less than 120' },
      },
    },
  },
};

export default function FormValidator() {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [age, setAge] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isLoading, setIsLoading] = useState(true);
  const [evalInstance, setEvalInstance] = useState<any>(null);

  useEffect(() => {
    let instance: any = null;
    
    // Dynamically import webworker (runs WASM off main thread)
    import('@json-eval-rs/webworker').then(async ({ JSONEvalWorker }) => {
      instance = new JSONEvalWorker({ schema });
      await instance.init();
      setEvalInstance(instance);
      setIsLoading(false);
    }).catch(err => {
      console.error('Failed to load WASM worker:', err);
      setIsLoading(false);
    });

    return () => {
      if (instance) {
        instance.free();
      }
    };
  }, []);

  const handleValidate = async () => {
    if (!evalInstance) {
      console.error('WASM module not loaded');
      return;
    }

    const data = {
      name,
      email,
      age: age ? Number(age) : null,
    };

    try {
      const result = await evalInstance.validate({ data });
      
      if (result.has_error) {
        const errorMap: Record<string, string> = {};
        result.errors.forEach((error: ValidationError) => {
          errorMap[error.path] = error.message;
        });
        setErrors(errorMap);
      } else {
        setErrors({});
        alert('✅ Form is valid!');
      }
    } catch (error) {
      console.error('Validation error:', error);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold mb-4">Form Validation Example</h2>
        <p className="text-gray-600 dark:text-gray-400 mb-6">
          This example demonstrates real-time form validation using JSON schema rules powered by WebAssembly.
        </p>
      </div>

      <div className="space-y-4">
        <div>
          <label htmlFor="name" className="block text-sm font-medium mb-2">
            Name
          </label>
          <input
            id="name"
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
              errors.name ? 'border-red-500' : 'border-gray-300 dark:border-gray-600'
            } bg-white dark:bg-gray-700 text-gray-900 dark:text-white`}
            placeholder="Enter your name"
          />
          {errors.name && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.name}</p>
          )}
        </div>

        <div>
          <label htmlFor="email" className="block text-sm font-medium mb-2">
            Email
          </label>
          <input
            id="email"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
              errors.email ? 'border-red-500' : 'border-gray-300 dark:border-gray-600'
            } bg-white dark:bg-gray-700 text-gray-900 dark:text-white`}
            placeholder="your.email@example.com"
          />
          {errors.email && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.email}</p>
          )}
        </div>

        <div>
          <label htmlFor="age" className="block text-sm font-medium mb-2">
            Age
          </label>
          <input
            id="age"
            type="number"
            value={age}
            onChange={(e) => setAge(e.target.value)}
            className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
              errors.age ? 'border-red-500' : 'border-gray-300 dark:border-gray-600'
            } bg-white dark:bg-gray-700 text-gray-900 dark:text-white`}
            placeholder="Enter your age"
          />
          {errors.age && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.age}</p>
          )}
        </div>

        <button
          onClick={handleValidate}
          className="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors focus:ring-4 focus:ring-blue-300"
        >
          Validate Form
        </button>
      </div>

      <div className="mt-6 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
        <h3 className="font-semibold mb-2">Current Data:</h3>
        <pre className="text-sm overflow-auto">
          {JSON.stringify({ name, email, age: age ? Number(age) : null }, null, 2)}
        </pre>
      </div>
    </div>
  );
}
