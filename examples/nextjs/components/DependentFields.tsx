'use client';

import { useState, useEffect } from 'react';

const schema = {
  type: 'object',
  properties: {
    quantity: {
      type: 'number',
    },
    price: {
      type: 'number',
    },
    subtotal: {
      type: 'number',
      $evaluation: {
        '*': [{ var: 'quantity' }, { var: 'price' }],
      },
    },
    taxRate: {
      type: 'number',
    },
    tax: {
      type: 'number',
      $evaluation: {
        '*': [{ var: 'subtotal' }, { var: 'taxRate' }],
      },
    },
    total: {
      type: 'number',
      $evaluation: {
        '+': [{ var: 'subtotal' }, { var: 'tax' }],
      },
    },
  },
};

export default function DependentFields() {
  const [quantity, setQuantity] = useState(1);
  const [price, setPrice] = useState(10);
  const [taxRate, setTaxRate] = useState(0.1);
  const [subtotal, setSubtotal] = useState(0);
  const [tax, setTax] = useState(0);
  const [total, setTotal] = useState(0);
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

  useEffect(() => {
    if (!evalInstance) return;

    const updateDependents = async () => {
      const data = {
        quantity,
        price,
        taxRate,
      };

      try {
        const result = await evalInstance.evaluateDependents({
          changedPaths: ['quantity', 'price', 'taxRate'],
          data,
          nested: true
        });

        setSubtotal(result.subtotal || 0);
        setTax(result.tax || 0);
        setTotal(result.total || 0);
      } catch (error) {
        console.error('Evaluation error:', error);
      }
    };

    updateDependents();
  }, [quantity, price, taxRate, evalInstance]);

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
        <h2 className="text-2xl font-bold mb-4">Dependent Fields Example</h2>
        <p className="text-gray-600 dark:text-gray-400 mb-6">
          This example shows automatic calculation of dependent fields. Change quantity, price, or tax rate to see
          real-time updates.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div>
          <label htmlFor="quantity" className="block text-sm font-medium mb-2">
            Quantity
          </label>
          <input
            id="quantity"
            type="number"
            value={quantity}
            onChange={(e) => setQuantity(Number(e.target.value))}
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            min="0"
          />
        </div>

        <div>
          <label htmlFor="price" className="block text-sm font-medium mb-2">
            Price ($)
          </label>
          <input
            id="price"
            type="number"
            value={price}
            onChange={(e) => setPrice(Number(e.target.value))}
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            min="0"
            step="0.01"
          />
        </div>

        <div>
          <label htmlFor="taxRate" className="block text-sm font-medium mb-2">
            Tax Rate (%)
          </label>
          <input
            id="taxRate"
            type="number"
            value={taxRate * 100}
            onChange={(e) => setTaxRate(Number(e.target.value) / 100)}
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            min="0"
            max="100"
            step="0.1"
          />
        </div>
      </div>

      <div className="border-t border-gray-200 dark:border-gray-700 pt-6 space-y-4">
        <div className="flex justify-between items-center text-lg">
          <span className="font-medium">Subtotal:</span>
          <span className="text-2xl font-bold">${subtotal.toFixed(2)}</span>
        </div>

        <div className="flex justify-between items-center text-lg">
          <span className="font-medium">Tax ({(taxRate * 100).toFixed(1)}%):</span>
          <span className="text-2xl font-bold">${tax.toFixed(2)}</span>
        </div>

        <div className="flex justify-between items-center text-xl border-t border-gray-200 dark:border-gray-700 pt-4">
          <span className="font-bold">Total:</span>
          <span className="text-3xl font-bold text-blue-600 dark:text-blue-400">
            ${total.toFixed(2)}
          </span>
        </div>
      </div>

      <div className="mt-6 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
        <h3 className="font-semibold mb-2">Evaluation Logic:</h3>
        <pre className="text-sm overflow-auto">
{`subtotal = quantity × price
tax = subtotal × taxRate  
total = subtotal + tax`}
        </pre>
      </div>
    </div>
  );
}
