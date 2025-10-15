import { useState, useEffect } from 'react';
import Head from 'next/head';
import FormValidator from '@/components/FormValidator';
import DependentFields from '@/components/DependentFields';

export default function Home() {
  const [activeTab, setActiveTab] = useState<'validator' | 'dependent'>('validator');

  return (
    <>
      <Head>
        <title>JSON Eval RS - Next.js Example</title>
        <meta name="description" content="JSON Eval RS Next.js example with form validation and dependent fields" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="icon" href="/favicon.ico" />
      </Head>
      <main className="min-h-screen p-8">
        <div className="max-w-4xl mx-auto">
          <main className="container mx-auto px-4 py-8">
            <h1 className="text-4xl font-bold text-center mb-8">
              JSON Eval RS - Next.js Example
            </h1>
            <p className="text-center text-gray-600 mb-8">
              High-performance JSON Logic evaluation with schema validation
              <br />
              <span className="text-sm text-blue-600">✨ Powered by Web Workers - Runs off main thread for smooth UI</span>
            </p>
            <div className="mt-4 flex gap-2">
              <span className="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm">
                WebAssembly
              </span>
              <span className="px-3 py-1 bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 rounded-full text-sm">
                Rust-Powered
              </span>
              <span className="px-3 py-1 bg-purple-100 dark:bg-purple-900 text-purple-800 dark:text-purple-200 rounded-full text-sm">
                Sequential Processing
              </span>
            </div>
          </header>

          <div className="mb-6 border-b border-gray-200 dark:border-gray-700">
            <nav className="flex gap-4">
              <button
                onClick={() => setActiveTab('validator')}
                className={`px-4 py-2 border-b-2 transition-colors ${
                  activeTab === 'validator'
                    ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                    : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'
                }`}
              >
                Form Validation
              </button>
              <button
                onClick={() => setActiveTab('dependent')}
                className={`px-4 py-2 border-b-2 transition-colors ${
                  activeTab === 'dependent'
                    ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                    : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'
                }`}
              >
                Dependent Fields
              </button>
            </nav>
          </div>

          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
            {activeTab === 'validator' ? <FormValidator /> : <DependentFields />}
          </div>

          <footer className="mt-8 text-center text-sm text-gray-500 dark:text-gray-400">
            <p>
              Built with{' '}
              <a
                href="https://github.com/yourusername/json-eval-rs"
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 hover:underline"
              >
                json-eval-rs
              </a>
            </p>
          </footer>
        </div>
      </main>
    </>
  );
}
