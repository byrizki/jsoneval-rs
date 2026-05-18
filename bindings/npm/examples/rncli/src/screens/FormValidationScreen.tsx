import React, { useState } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  useColorScheme,
} from 'react-native';
import { useJSONEval, type ValidationError } from '@json-eval-rs/react-native';

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

export default function FormValidationScreen() {
  const isDarkMode = useColorScheme() === 'dark';
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [age, setAge] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});
  const evalInstance = useJSONEval({ schema });

  const handleValidate = async () => {
    if (!evalInstance) {
      Alert.alert('Error', 'Evaluator not initialized');
      return;
    }

    try {
      const data = { name, email, age: age ? Number(age) : null };
      const result = await evalInstance.validate({ data });
      
      if (result.hasError) {
        const newErrors: Record<string, string> = {};
        result.errors.forEach((error: ValidationError) => {
          // Extract field name from path (e.g., "name" from "name" or "user.name")
          const fieldName = error.path.split('.').pop() || error.path;
          newErrors[fieldName] = error.message;
        });
        setErrors(newErrors);
      } else {
        setErrors({});
        Alert.alert('Success', '✅ Form is valid!');
      }
    } catch (error) {
      Alert.alert('Error', `Validation failed: ${error}`);
    }
  };

  const inputStyle = (fieldName: string) => [
    styles.input,
    { 
      backgroundColor: isDarkMode ? '#2a2a2a' : '#fff',
      color: isDarkMode ? '#fff' : '#000',
      borderColor: errors[fieldName] ? '#ef4444' : (isDarkMode ? '#444' : '#ddd'),
    },
  ];

  return (
    <View style={styles.container}>
      <Text style={[styles.title, { color: isDarkMode ? '#fff' : '#000' }]}>
        Form Validation Example
      </Text>
      <Text style={[styles.description, { color: isDarkMode ? '#aaa' : '#666' }]}>
        This demonstrates native form validation using JSON schema rules.
      </Text>

      <View style={styles.formGroup}>
        <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
          Name
        </Text>
        <TextInput
          style={inputStyle('name')}
          value={name}
          onChangeText={setName}
          placeholder="Enter your name"
          placeholderTextColor={isDarkMode ? '#666' : '#999'}
        />
        {errors.name && (
          <Text style={styles.errorText}>{errors.name}</Text>
        )}
      </View>

      <View style={styles.formGroup}>
        <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
          Email
        </Text>
        <TextInput
          style={inputStyle('email')}
          value={email}
          onChangeText={setEmail}
          placeholder="your.email@example.com"
          placeholderTextColor={isDarkMode ? '#666' : '#999'}
          keyboardType="email-address"
          autoCapitalize="none"
        />
        {errors.email && (
          <Text style={styles.errorText}>{errors.email}</Text>
        )}
      </View>

      <View style={styles.formGroup}>
        <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
          Age
        </Text>
        <TextInput
          style={inputStyle('age')}
          value={age}
          onChangeText={setAge}
          placeholder="Enter your age"
          placeholderTextColor={isDarkMode ? '#666' : '#999'}
          keyboardType="number-pad"
        />
        {errors.age && (
          <Text style={styles.errorText}>{errors.age}</Text>
        )}
      </View>

      <TouchableOpacity style={styles.button} onPress={handleValidate}>
        <Text style={styles.buttonText}>Validate Form</Text>
      </TouchableOpacity>

      <View style={[styles.dataBox, { backgroundColor: isDarkMode ? '#2a2a2a' : '#f9fafb' }]}>
        <Text style={[styles.dataTitle, { color: isDarkMode ? '#fff' : '#000' }]}>
          Current Data:
        </Text>
        <Text style={[styles.dataText, { color: isDarkMode ? '#aaa' : '#666' }]}>
          {JSON.stringify({ name, email, age: age ? Number(age) : null }, null, 2)}
        </Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
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
  formGroup: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    fontWeight: '600',
    marginBottom: 8,
  },
  input: {
    borderWidth: 1,
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 12,
    fontSize: 16,
  },
  errorText: {
    color: '#ef4444',
    fontSize: 12,
    marginTop: 4,
  },
  button: {
    backgroundColor: '#3b82f6',
    paddingVertical: 16,
    borderRadius: 8,
    alignItems: 'center',
    marginTop: 8,
  },
  buttonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  dataBox: {
    marginTop: 24,
    padding: 16,
    borderRadius: 8,
  },
  dataTitle: {
    fontSize: 14,
    fontWeight: '600',
    marginBottom: 8,
  },
  dataText: {
    fontSize: 12,
    fontFamily: 'monospace',
  },
});
