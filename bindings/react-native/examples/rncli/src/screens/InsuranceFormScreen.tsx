import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  ScrollView,
  useColorScheme,
  Switch,
  TouchableOpacity,
} from 'react-native';
import { useJSONEval } from '@json-eval-rs/react-native';

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

export default function InsuranceFormScreen() {
  const isDarkMode = useColorScheme() === 'dark';
  const [name, setName] = useState('John Doe');
  const [dateOfBirth, setDateOfBirth] = useState('1990-01-01');
  const [age, setAge] = useState<number | null>(null);
  const [isSmoker, setIsSmoker] = useState(false);
  const [occupation, setOccupation] = useState('OFFICE');
  const [occupationClass, setOccupationClass] = useState('');
  const [riskCategory, setRiskCategory] = useState('');
  const [productInfo, setProductInfo] = useState<any>(null);
  const [evaluatedSchema, setEvaluatedSchema] = useState<any>(null);
  const [showParams, setShowParams] = useState(false);
  
  const evalInstance = useJSONEval({ schema });

  // Initial evaluation
  useEffect(() => {
    if (!evalInstance) return;

    const initialize = async () => {
      try {
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

        await evalInstance.evaluate({ data });
        
        // Get evaluated schema WITHOUT $params
        const schemaWithoutParams = await evalInstance.getEvaluatedSchemaWithoutParams({ skipLayout: true });
        setEvaluatedSchema(schemaWithoutParams);

        // Get $params by path using dot notation
        const params = await evalInstance.getValueByPath({ 
          path: '$params', 
          skipLayout: true 
        });
        setProductInfo(params);

        // Get calculated values
        const ageValue = await evalInstance.getValueByPath({ 
          path: 'illustration.properties.insured.properties.age.value',
          skipLayout: true 
        });
        const classValue = await evalInstance.getValueByPath({ 
          path: 'illustration.properties.insured.properties.occupation_class.value',
          skipLayout: true 
        });
        const riskValue = await evalInstance.getValueByPath({ 
          path: 'illustration.properties.insured.properties.risk_category.value',
          skipLayout: true 
        });

        setAge(ageValue);
        setOccupationClass(classValue || '');
        setRiskCategory(riskValue || '');
      } catch (error) {
        console.error('Initialization error:', error);
      }
    };

    initialize();
  }, [evalInstance]);

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

      // Process dependent changes
      if (result && Array.isArray(result)) {
        result.forEach((change: any) => {
          if (change.$ref?.includes('age') && change.value != null) {
            setAge(change.value);
          }
        });
      }
    } catch (error) {
      console.error('Date change error:', error);
    }
  };

  // Handle smoker change
  const handleSmokerChange = async (value: boolean) => {
    setIsSmoker(value);
    if (!evalInstance) return;

    try {
      const data = {
        illustration: {
          insured: {
            name,
            date_of_birth: dateOfBirth,
            is_smoker: value,
            occupation,
          }
        }
      };

      // Use dot notation for path
      const result = await evalInstance.evaluateDependents({
        changedPath: 'illustration.insured.is_smoker',  // Dot notation!
        data,
      });

      // Process dependent changes (clears occupation, updates risk)
      if (result && Array.isArray(result)) {
        result.forEach((change: any) => {
          if (change.$ref?.includes('occupation') && change.clear) {
            setOccupation('');
          }
          if (change.$ref?.includes('risk_category') && change.value) {
            setRiskCategory(change.value);
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

      // Process transitive changes (occupation -> occupation_class -> risk_category)
      if (result && Array.isArray(result)) {
        result.forEach((change: any) => {
          if (change.$ref?.includes('occupation_class') && change.value) {
            setOccupationClass(change.value);
          }
          if (change.$ref?.includes('risk_category') && change.value) {
            setRiskCategory(change.value);
          }
        });
      }
    } catch (error) {
      console.error('Occupation change error:', error);
    }
  };

  const inputStyle = [
    styles.input,
    { 
      backgroundColor: isDarkMode ? '#2a2a2a' : '#fff',
      color: isDarkMode ? '#fff' : '#000',
      borderColor: isDarkMode ? '#444' : '#ddd',
    },
  ];

  const readOnlyStyle = [
    ...inputStyle,
    { backgroundColor: isDarkMode ? '#1a1a1a' : '#f5f5f5' },
  ];

  return (
    <ScrollView style={[styles.container, { backgroundColor: isDarkMode ? '#000' : '#f9fafb' }]}>
      <View style={styles.content}>
        <Text style={[styles.title, { color: isDarkMode ? '#fff' : '#000' }]}>
          Insurance Form with Dependencies
        </Text>
        <Text style={[styles.description, { color: isDarkMode ? '#aaa' : '#666' }]}>
          Using minimal_form.json with dot notation paths
        </Text>

        {/* Product Info Section */}
        {productInfo && (
          <View style={[styles.infoBox, { backgroundColor: isDarkMode ? '#1a1a1a' : '#f0f9ff' }]}>
            <Text style={[styles.infoTitle, { color: isDarkMode ? '#60a5fa' : '#2563eb' }]}>
              Product Information ($params)
            </Text>
            <Text style={[styles.infoText, { color: isDarkMode ? '#aaa' : '#666' }]}>
              {productInfo.productName} ({productInfo.productCode})
            </Text>
            <Text style={[styles.infoText, { color: isDarkMode ? '#aaa' : '#666' }]}>
              Type: {productInfo.type}
            </Text>
            <TouchableOpacity onPress={() => setShowParams(!showParams)}>
              <Text style={[styles.linkText, { color: isDarkMode ? '#60a5fa' : '#2563eb' }]}>
                {showParams ? 'Hide' : 'Show'} Full Params
              </Text>
            </TouchableOpacity>
            {showParams && (
              <Text style={[styles.jsonText, { color: isDarkMode ? '#aaa' : '#666' }]}>
                {JSON.stringify(productInfo, null, 2)}
              </Text>
            )}
          </View>
        )}

        {/* Form Fields */}
        <View style={styles.section}>
          <Text style={[styles.sectionTitle, { color: isDarkMode ? '#fff' : '#000' }]}>
            Insured Person Details
          </Text>

          <View style={styles.field}>
            <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
              Name
            </Text>
            <TextInput
              style={inputStyle}
              value={name}
              onChangeText={setName}
            />
          </View>

          <View style={styles.field}>
            <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
              Date of Birth
            </Text>
            <TextInput
              style={inputStyle}
              value={dateOfBirth}
              onChangeText={handleDateChange}
              placeholder="YYYY-MM-DD"
            />
          </View>

          <View style={styles.field}>
            <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
              Age (calculated via DATEDIF)
            </Text>
            <TextInput
              style={readOnlyStyle}
              value={age?.toString() || 'Calculating...'}
              editable={false}
            />
            <Text style={[styles.helperText, { color: isDarkMode ? '#888' : '#999' }]}>
              Automatically calculated from date of birth
            </Text>
          </View>

          <View style={styles.field}>
            <View style={styles.switchRow}>
              <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
                Is Smoker?
              </Text>
              <Switch
                value={isSmoker}
                onValueChange={handleSmokerChange}
              />
            </View>
            <Text style={[styles.helperText, { color: isDarkMode ? '#888' : '#999' }]}>
              Clears occupation and updates risk category
            </Text>
          </View>

          <View style={styles.field}>
            <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
              Occupation
            </Text>
            <View style={styles.buttonRow}>
              {['OFFICE', 'PROFESSIONAL', 'MANUAL', 'HIGH_RISK'].map((occ) => (
                <TouchableOpacity
                  key={occ}
                  style={[
                    styles.button,
                    occupation === occ && styles.buttonActive,
                    { borderColor: isDarkMode ? '#444' : '#ddd' }
                  ]}
                  onPress={() => handleOccupationChange(occ)}
                >
                  <Text style={[
                    styles.buttonText,
                    { color: isDarkMode ? '#fff' : '#000' },
                    occupation === occ && styles.buttonTextActive
                  ]}>
                    {occ.replace('_', ' ')}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
            <Text style={[styles.helperText, { color: isDarkMode ? '#888' : '#999' }]}>
              Triggers transitive dependency chain
            </Text>
          </View>

          <View style={styles.field}>
            <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
              Occupation Class (calculated)
            </Text>
            <TextInput
              style={readOnlyStyle}
              value={occupationClass || 'N/A'}
              editable={false}
            />
            <Text style={[styles.helperText, { color: isDarkMode ? '#888' : '#999' }]}>
              Depends on occupation
            </Text>
          </View>

          <View style={styles.field}>
            <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
              Risk Category (calculated)
            </Text>
            <TextInput
              style={readOnlyStyle}
              value={riskCategory || 'N/A'}
              editable={false}
            />
            <Text style={[styles.helperText, { color: isDarkMode ? '#888' : '#999' }]}>
              Depends on occupation class and smoker status
            </Text>
          </View>
        </View>

        {/* API Demo Box */}
        <View style={[styles.demoBox, { backgroundColor: isDarkMode ? '#1a1a1a' : '#f9fafb' }]}>
          <Text style={[styles.demoTitle, { color: isDarkMode ? '#fff' : '#000' }]}>
            🎯 API Features Demonstrated
          </Text>
          <Text style={[styles.demoText, { color: isDarkMode ? '#aaa' : '#666' }]}>
            ✅ Dot notation paths: "illustration.insured.name"{'\n'}
            ✅ getEvaluatedSchemaWithoutParams(){'\n'}
            ✅ getValueByPath() for $params access{'\n'}
            ✅ Transitive dependencies (auto-processed){'\n'}
            ✅ Clear and value dependents{'\n'}
            ✅ Real-time field calculations
          </Text>
        </View>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 8,
  },
  description: {
    fontSize: 14,
    marginBottom: 24,
  },
  infoBox: {
    padding: 16,
    borderRadius: 8,
    marginBottom: 24,
  },
  infoTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 14,
    marginBottom: 4,
  },
  linkText: {
    fontSize: 14,
    fontWeight: '600',
    marginTop: 8,
  },
  jsonText: {
    fontSize: 12,
    fontFamily: 'monospace',
    marginTop: 8,
  },
  section: {
    marginBottom: 24,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 16,
  },
  field: {
    marginBottom: 20,
  },
  label: {
    fontSize: 14,
    fontWeight: '600',
    marginBottom: 8,
  },
  input: {
    borderWidth: 1,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 16,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  buttonRow: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  button: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 6,
    borderWidth: 1,
  },
  buttonActive: {
    backgroundColor: '#3b82f6',
    borderColor: '#3b82f6',
  },
  buttonText: {
    fontSize: 12,
  },
  buttonTextActive: {
    color: '#fff',
  },
  helperText: {
    fontSize: 12,
    marginTop: 4,
    fontStyle: 'italic',
  },
  demoBox: {
    padding: 16,
    borderRadius: 8,
    marginTop: 24,
  },
  demoTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  demoText: {
    fontSize: 13,
    lineHeight: 20,
  },
});
