import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  useColorScheme,
} from 'react-native';
import { useJSONEval } from '@json-eval-rs/react-native';

const schema = {
  type: 'object',
  properties: {
    quantity: { type: 'number' },
    price: { type: 'number' },
    taxRate: { type: 'number' },
    subtotal: {
      type: 'number',
      $evaluation: {
        '*': [{ var: 'quantity' }, { var: 'price' }]
      }
    },
    tax: {
      type: 'number',
      $evaluation: {
        '*': [{ var: 'subtotal' }, { '/': [{ var: 'taxRate' }, 100] }]
      }
    },
    total: {
      type: 'number',
      $evaluation: {
        '+': [{ var: 'subtotal' }, { var: 'tax' }]
      }
    }
  }
};

export default function DependentFieldsScreen() {
  const isDarkMode = useColorScheme() === 'dark';
  const [quantity, setQuantity] = useState('1');
  const [price, setPrice] = useState('10');
  const [taxRate, setTaxRate] = useState('10');
  const [subtotal, setSubtotal] = useState(0);
  const [tax, setTax] = useState(0);
  const [total, setTotal] = useState(0);
  const evalInstance = useJSONEval({ schema });

  useEffect(() => {
    if (!evalInstance) return;

    const updateCalculations = async () => {
      try {
        const data = {
          quantity: Number(quantity) || 0,
          price: Number(price) || 0,
          taxRate: Number(taxRate) || 0,
        };

        const result = await evalInstance.evaluateDependents({
          changedPaths: ['quantity', 'price', 'taxRate'],
          data,
          nested: true,
        });

        setSubtotal(result.subtotal || 0);
        setTax(result.tax || 0);
        setTotal(result.total || 0);
      } catch (error) {
        console.error('Calculation error:', error);
      }
    };

    updateCalculations();
  }, [evalInstance, quantity, price, taxRate]);

  const inputStyle = [
    styles.input,
    { 
      backgroundColor: isDarkMode ? '#2a2a2a' : '#fff',
      color: isDarkMode ? '#fff' : '#000',
      borderColor: isDarkMode ? '#444' : '#ddd',
    },
  ];

  return (
    <View style={styles.container}>
      <Text style={[styles.title, { color: isDarkMode ? '#fff' : '#000' }]}>
        Dependent Fields Example
      </Text>
      <Text style={[styles.description, { color: isDarkMode ? '#aaa' : '#666' }]}>
        This shows automatic calculation of dependent fields. Change values to see real-time updates.
      </Text>

      <View style={styles.row}>
        <View style={styles.column}>
          <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
            Quantity
          </Text>
          <TextInput
            style={inputStyle}
            value={quantity}
            onChangeText={setQuantity}
            keyboardType="number-pad"
          />
        </View>

        <View style={styles.column}>
          <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
            Price ($)
          </Text>
          <TextInput
            style={inputStyle}
            value={price}
            onChangeText={setPrice}
            keyboardType="decimal-pad"
          />
        </View>

        <View style={styles.column}>
          <Text style={[styles.label, { color: isDarkMode ? '#fff' : '#000' }]}>
            Tax (%)
          </Text>
          <TextInput
            style={inputStyle}
            value={taxRate}
            onChangeText={setTaxRate}
            keyboardType="decimal-pad"
          />
        </View>
      </View>

      <View style={[styles.divider, { backgroundColor: isDarkMode ? '#444' : '#e5e7eb' }]} />

      <View style={styles.resultRow}>
        <Text style={[styles.resultLabel, { color: isDarkMode ? '#fff' : '#000' }]}>
          Subtotal:
        </Text>
        <Text style={[styles.resultValue, { color: isDarkMode ? '#fff' : '#000' }]}>
          ${subtotal.toFixed(2)}
        </Text>
      </View>

      <View style={styles.resultRow}>
        <Text style={[styles.resultLabel, { color: isDarkMode ? '#fff' : '#000' }]}>
          Tax ({taxRate}%):
        </Text>
        <Text style={[styles.resultValue, { color: isDarkMode ? '#fff' : '#000' }]}>
          ${tax.toFixed(2)}
        </Text>
      </View>

      <View style={[styles.divider, { backgroundColor: isDarkMode ? '#444' : '#e5e7eb' }]} />

      <View style={styles.resultRow}>
        <Text style={[styles.totalLabel, { color: isDarkMode ? '#fff' : '#000' }]}>
          Total:
        </Text>
        <Text style={styles.totalValue}>
          ${total.toFixed(2)}
        </Text>
      </View>

      <View style={[styles.infoBox, { backgroundColor: isDarkMode ? '#2a2a2a' : '#f9fafb' }]}>
        <Text style={[styles.infoTitle, { color: isDarkMode ? '#fff' : '#000' }]}>
          Evaluation Logic:
        </Text>
        <Text style={[styles.infoText, { color: isDarkMode ? '#aaa' : '#666' }]}>
          {`subtotal = quantity × price\ntax = subtotal × taxRate\ntotal = subtotal + tax`}
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
  row: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 24,
  },
  column: {
    flex: 1,
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
  divider: {
    height: 1,
    marginVertical: 16,
  },
  resultRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginVertical: 8,
  },
  resultLabel: {
    fontSize: 16,
    fontWeight: '500',
  },
  resultValue: {
    fontSize: 20,
    fontWeight: 'bold',
  },
  totalLabel: {
    fontSize: 20,
    fontWeight: 'bold',
  },
  totalValue: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#3b82f6',
  },
  infoBox: {
    marginTop: 24,
    padding: 16,
    borderRadius: 8,
  },
  infoTitle: {
    fontSize: 14,
    fontWeight: '600',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 12,
    fontFamily: 'monospace',
    lineHeight: 18,
  },
});
