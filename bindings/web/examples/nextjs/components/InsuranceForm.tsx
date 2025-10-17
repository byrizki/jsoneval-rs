'use client';

import { useState, useEffect } from 'react';

// Minimal form schema based on tests/fixtures/minimal_form.json
const schema = {
  "$schema": "https://raw.githubusercontent.com/QuadrantSynergyInternational/form-schema/refs/heads/main/schema.json",
  "$params": {
    "type": "illustration",
    "productCode": "MIN001",
    "productName": "Minimal Insurance Product",
    "constants": {
      "MAX_AGE": 100,
      "MIN_AGE": 1
    },
    "references": {
      "OCCUPATION_TABLE": [
        { "occupation": "OFFICE", "class": "1", "risk": "Low" },
        { "occupation": "PROFESSIONAL", "class": "1", "risk": "Low" },
        { "occupation": "MANUAL", "class": "2", "risk": "Medium" },
        { "occupation": "HIGH_RISK", "class": "3", "risk": "High" }
      ]
    }
  },
  "properties": {
    "illustration": {
      "type": "object",
      "properties": {
        "insured": {
          "type": "object",
          "properties": {
            "name": {
              "type": "string",
              "title": "Name"
            },
            "date_of_birth": {
              "type": "string",
              "title": "Date of Birth",
              "dependents": [
                {
                  "$ref": "#/illustration/properties/insured/properties/age",
                  "value": {
                    "$evaluation": {
                      "DATEDIF": [
                        { "$ref": "$value" },
                        { "NOW": [] },
                        "Y"
                      ]
                    }
                  }
                }
              ]
            },
            "age": {
              "type": "number",
              "title": "Age (calculated)"
            },
            "is_smoker": {
              "type": "boolean",
              "title": "Is Smoker?",
              "dependents": [
                {
                  "$ref": "#/illustration/properties/insured/properties/occupation",
                  "clear": { "$evaluation": true }
                },
                {
                  "$ref": "#/illustration/properties/insured/properties/risk_category",
                  "value": {
                    "$evaluation": {
                      "if": [
                        { "$ref": "$value" },
                        "High",
                        "Standard"
                      ]
                    }
                  }
                }
              ]
            },
            "occupation": {
              "type": "string",
              "title": "Occupation",
              "dependents": [
                {
                  "$ref": "#/illustration/properties/insured/properties/occupation_class",
                  "value": {
                    "$evaluation": {
                      "if": [
                        { "==": [{ "$ref": "$value" }, "OFFICE"] },
                        "1",
                        {
                          "if": [
                            { "==": [{ "$ref": "$value" }, "PROFESSIONAL"] },
                            "1",
                            {
                              "if": [
                                { "==": [{ "$ref": "$value" }, "MANUAL"] },
                                "2",
                                "3"
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  }
                }
              ]
            },
            "occupation_class": {
              "type": "string",
              "title": "Occupation Class (calculated)",
              "dependents": [
                {
                  "$ref": "#/illustration/properties/insured/properties/risk_category",
                  "value": {
                    "$evaluation": {
                      "if": [
                        { "==": [{ "$ref": "$value" }, "1"] },
                        "Low",
                        {
                          "if": [
                            { "==": [{ "$ref": "$value" }, "2"] },
                            "Medium",
                            "High"
                          ]
                        }
                      ]
                    }
                  }
                }
              ]
            },
            "risk_category": {
              "type": "string",
              "title": "Risk Category (calculated)"
            }
          }
        }
      }
    }
  }
};

export default function InsuranceForm() {
  const [name, setName] = useState('John Doe');
  const [dateOfBirth, setDateOfBirth] = useState('1990-01-01');
  const [age, setAge] = useState<number | null>(null);
  const [isSmoker, setIsSmoker] = useState(false);
  const [occupation, setOccupation] = useState('OFFICE');
  const [occupationClass, setOccupationClass] = useState('');
  const [riskCategory, setRiskCategory] = useState('');
  const [productInfo, setProductInfo] = useState<any>(null);
  const [showParams, setShowParams] = useState(false);
  const [showSchema, setShowSchema] = useState(false);
  const [evaluatedSchema, setEvaluatedSchema] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [evalInstance, setEvalInstance] = useState<any>(null);
  const [dependencyLog, setDependencyLog] = useState<string[]>([]);

  // Initialize JSONEval instance
  useEffect(() => {
    let instance: any = null;
    
    // Dynamically import bundler package (client-side only)
    import('@json-eval-rs/bundler').then(async ({ JSONEval }) => {
      instance = new JSONEval({ schema });
      setEvalInstance(instance);

      // Initial evaluation
      const data = {
        illustration: {
          insured: {
            name,
            date_of_birth: dateOfBirth,
            is_smoker: isSmoker,
            occupation,
          }
        }
      };

      await instance.evaluate({ data });
      
      // Get evaluated schema WITHOUT $params
      const schemaWithoutParams = await instance.getEvaluatedSchemaWithoutParams({ skipLayout: true });
      setEvaluatedSchema(schemaWithoutParams);

      // Get $params by path using dot notation
      const params = await instance.getValueByPath({ 
        path: '$params', 
        skipLayout: true 
      });
      setProductInfo(params);

      // Get calculated values
      const ageValue = await instance.getValueByPath({ 
        path: 'illustration.properties.insured.properties.age.value',
        skipLayout: true 
      });
      const classValue = await instance.getValueByPath({ 
        path: 'illustration.properties.insured.properties.occupation_class.value',
        skipLayout: true 
      });
      const riskValue = await instance.getValueByPath({ 
        path: 'illustration.properties.insured.properties.risk_category.value',
        skipLayout: true 
      });

      setAge(ageValue);
      setOccupationClass(classValue || '');
      setRiskCategory(riskValue || '');
      setIsLoading(false);
    }).catch(err => {
      console.error('Failed to load WASM:', err);
      setIsLoading(false);
    });

    return () => {
      if (instance) {
        instance.free();
      }
    };
  }, []);

  // Handle date of birth change with dot notation
  const handleDateChange = async (newDate: string) => {
    setDateOfBirth(newDate);
    if (!evalInstance) return;

    try {
      const data = {
        illustration: {
          insured: {
            name,
            date_of_birth: newDate,
            is_smoker: isSmoker,
            occupation,
          }
        }
      };

      // Use dot notation for path! Much simpler than full schema path
      const result = await evalInstance.evaluateDependents({
        changedPath: 'illustration.insured.date_of_birth',  // Dot notation!
        data,
      });

      setDependencyLog(prev => [...prev, `📅 Date changed → Age calculated`]);

      // Process dependent changes
      if (result && Array.isArray(result)) {
        result.forEach((change: any) => {
          if (change.$ref?.includes('age') && change.value != null) {
            setAge(change.value);
            setDependencyLog(prev => [...prev, `  ✅ Age: ${change.value}`]);
          }
        });
      }
    } catch (error) {
      console.error('Date change error:', error);
    }
  };

  // Handle smoker change
  const handleSmokerChange = async (checked: boolean) => {
    setIsSmoker(checked);
    if (!evalInstance) return;

    try {
      const data = {
        illustration: {
          insured: {
            name,
            date_of_birth: dateOfBirth,
            is_smoker: checked,
            occupation,
          }
        }
      };

      // Use dot notation for path
      const result = await evalInstance.evaluateDependents({
        changedPath: 'illustration.insured.is_smoker',  // Dot notation!
        data,
      });

      setDependencyLog(prev => [...prev, `🚬 Smoker status changed → Clear occupation, update risk`]);

      // Process dependent changes (clears occupation, updates risk)
      if (result && Array.isArray(result)) {
        result.forEach((change: any) => {
          if (change.$ref?.includes('occupation') && change.clear) {
            setOccupation('');
            setDependencyLog(prev => [...prev, `  ✅ Occupation cleared`]);
          }
          if (change.$ref?.includes('risk_category') && change.value) {
            setRiskCategory(change.value);
            setDependencyLog(prev => [...prev, `  ✅ Risk: ${change.value}`]);
          }
        });
      }
    } catch (error) {
      console.error('Smoker change error:', error);
    }
  };

  // Handle occupation change (triggers transitive dependencies)
  const handleOccupationChange = async (value: string) => {
    setOccupation(value);
    if (!evalInstance) return;

    try {
      const data = {
        illustration: {
          insured: {
            name,
            date_of_birth: dateOfBirth,
            is_smoker: isSmoker,
            occupation: value,
          }
        }
      };

      // Use dot notation - automatically processes transitively!
      const result = await evalInstance.evaluateDependents({
        changedPath: 'illustration.insured.occupation',  // Dot notation!
        data,
      });

      setDependencyLog(prev => [...prev, `💼 Occupation → Class → Risk (transitive chain)`]);

      // Process transitive changes (occupation -> occupation_class -> risk_category)
      if (result && Array.isArray(result)) {
        result.forEach((change: any) => {
          const isTransitive = change.transitive;
          if (change.$ref?.includes('occupation_class') && change.value) {
            setOccupationClass(change.value);
            setDependencyLog(prev => [...prev, `  ${isTransitive ? '🔗' : '✅'} Class: ${change.value}`]);
          }
          if (change.$ref?.includes('risk_category') && change.value) {
            setRiskCategory(change.value);
            setDependencyLog(prev => [...prev, `  ${isTransitive ? '🔗' : '✅'} Risk: ${change.value}`]);
          }
        });
      }
    } catch (error) {
      console.error('Occupation change error:', error);
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
        <h2 className="text-3xl font-bold mb-2">Insurance Form with Dependencies</h2>
        <p className="text-gray-600 dark:text-gray-400">
          Using minimal_form.json with dot notation paths and transitive dependency chains
        </p>
      </div>

      {/* Product Info Section */}
      {productInfo && (
        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
          <h3 className="text-lg font-semibold text-blue-900 dark:text-blue-300 mb-2">
            📦 Product Information ($params)
          </h3>
          <div className="text-sm text-gray-700 dark:text-gray-300">
            <p><strong>{productInfo.productName}</strong> ({productInfo.productCode})</p>
            <p>Type: {productInfo.type}</p>
            <button
              onClick={() => setShowParams(!showParams)}
              className="mt-2 text-blue-600 dark:text-blue-400 hover:underline text-sm"
            >
              {showParams ? 'Hide' : 'Show'} Full Params
            </button>
            {showParams && (
              <pre className="mt-2 p-2 bg-white dark:bg-gray-800 rounded text-xs overflow-auto max-h-60">
                {JSON.stringify(productInfo, null, 2)}
              </pre>
            )}
          </div>
        </div>
      )}

      {/* Form Fields */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <label htmlFor="name" className="block text-sm font-medium mb-2">
            Name
          </label>
          <input
            id="name"
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700"
          />
        </div>

        <div>
          <label htmlFor="dob" className="block text-sm font-medium mb-2">
            Date of Birth
          </label>
          <input
            id="dob"
            type="date"
            value={dateOfBirth}
            onChange={(e) => handleDateChange(e.target.value)}
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700"
          />
        </div>

        <div>
          <label htmlFor="age" className="block text-sm font-medium mb-2">
            Age (calculated via DATEDIF)
          </label>
          <input
            id="age"
            type="text"
            value={age?.toString() || 'Calculating...'}
            readOnly
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-100 dark:bg-gray-800"
          />
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Automatically calculated from date of birth
          </p>
        </div>

        <div>
          <label className="flex items-center space-x-2 cursor-pointer">
            <input
              type="checkbox"
              checked={isSmoker}
              onChange={(e) => handleSmokerChange(e.target.checked)}
              className="w-5 h-5 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="text-sm font-medium">Is Smoker?</span>
          </label>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Clears occupation and updates risk category
          </p>
        </div>
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">
          Occupation
        </label>
        <div className="flex flex-wrap gap-2">
          {['OFFICE', 'PROFESSIONAL', 'MANUAL', 'HIGH_RISK'].map((occ) => (
            <button
              key={occ}
              onClick={() => handleOccupationChange(occ)}
              className={`px-4 py-2 rounded-lg border-2 transition-colors ${
                occupation === occ
                  ? 'bg-blue-500 border-blue-500 text-white'
                  : 'border-gray-300 dark:border-gray-600 hover:border-blue-400'
              }`}
            >
              {occ.replace('_', ' ')}
            </button>
          ))}
        </div>
        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
          Triggers transitive dependency chain
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <label htmlFor="occClass" className="block text-sm font-medium mb-2">
            Occupation Class (calculated)
          </label>
          <input
            id="occClass"
            type="text"
            value={occupationClass || 'N/A'}
            readOnly
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-100 dark:bg-gray-800"
          />
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Depends on occupation
          </p>
        </div>

        <div>
          <label htmlFor="risk" className="block text-sm font-medium mb-2">
            Risk Category (calculated)
          </label>
          <input
            id="risk"
            type="text"
            value={riskCategory || 'N/A'}
            readOnly
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-100 dark:bg-gray-800"
          />
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Depends on occupation class and smoker status
          </p>
        </div>
      </div>

      {/* Dependency Log */}
      {dependencyLog.length > 0 && (
        <div className="bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
          <div className="flex justify-between items-center mb-2">
            <h3 className="text-sm font-semibold">📊 Dependency Execution Log</h3>
            <button
              onClick={() => setDependencyLog([])}
              className="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
            >
              Clear
            </button>
          </div>
          <div className="space-y-1 max-h-40 overflow-auto">
            {dependencyLog.map((log, i) => (
              <p key={i} className="text-xs font-mono text-gray-700 dark:text-gray-300">
                {log}
              </p>
            ))}
          </div>
        </div>
      )}

      {/* API Demo Box */}
      <div className="bg-gradient-to-r from-purple-50 to-blue-50 dark:from-purple-900/20 dark:to-blue-900/20 border border-purple-200 dark:border-purple-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold mb-4">🎯 API Features Demonstrated</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
          <div className="flex items-start space-x-2">
            <span className="text-green-500">✅</span>
            <span>Dot notation paths: <code className="text-xs bg-white dark:bg-gray-800 px-1 rounded">illustration.insured.name</code></span>
          </div>
          <div className="flex items-start space-x-2">
            <span className="text-green-500">✅</span>
            <span>getEvaluatedSchemaWithoutParams()</span>
          </div>
          <div className="flex items-start space-x-2">
            <span className="text-green-500">✅</span>
            <span>getValueByPath() for $params access</span>
          </div>
          <div className="flex items-start space-x-2">
            <span className="text-green-500">✅</span>
            <span>Transitive dependencies (auto-processed)</span>
          </div>
          <div className="flex items-start space-x-2">
            <span className="text-green-500">✅</span>
            <span>Clear and value dependents</span>
          </div>
          <div className="flex items-start space-x-2">
            <span className="text-green-500">✅</span>
            <span>Real-time field calculations (DATEDIF)</span>
          </div>
        </div>

        <button
          onClick={() => setShowSchema(!showSchema)}
          className="mt-4 text-sm text-purple-600 dark:text-purple-400 hover:underline"
        >
          {showSchema ? 'Hide' : 'Show'} Evaluated Schema (without $params)
        </button>
        
        {showSchema && evaluatedSchema && (
          <pre className="mt-2 p-3 bg-white dark:bg-gray-800 rounded text-xs overflow-auto max-h-96">
            {JSON.stringify(evaluatedSchema, null, 2)}
          </pre>
        )}
      </div>
    </div>
  );
}
