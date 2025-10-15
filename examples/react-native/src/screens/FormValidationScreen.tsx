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

// Note: In a real app, you would import from @json-eval-rs/react-native
// For this example, we'll use mock functions
interface ValidationError {
  path: string;
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

export default function FormValidationScreen() {
  const isDarkMode = useColorScheme() === 'dark';
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [age, setAge] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleValidate = async () => {
    try {
      // In a real app, you would use:
      // const eval = new JSONEval({ schema: JSON.stringify(schema) });
      // const result = await eval.validate({ 
      //   data: { name, email, age: age ? Number(age) : null }
      // });
      
      // Mock validation for demonstration
      const newErrors: Record<string, string> = {};
      
      if (!name) {
        newErrors.name = 'Name is required';
      } else if (name.length < 3) {
        newErrors.name = 'Name must be at least 3 characters';
      }
      
      if (!email) {
        newErrors.email = 'Email is required';
      } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
        newErrors.email = 'Invalid email format';
      }
      
      if (!age) {
        newErrors.age = 'Age is required';
      } else {
        const ageNum = Number(age);
        if (ageNum < 18) {
          newErrors.age = 'Must be at least 18 years old';
        } else if (ageNum > 120) {
          newErrors.age = 'Age must be less than 120';
        }
      }
      
      setErrors(newErrors);
      
      if (Object.keys(newErrors).length === 0) {
        Alert.alert('Success', '✅ Form is valid!');
      }
    } catch (error) {
      Alert.alert('Error', 'Validation failed');
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
