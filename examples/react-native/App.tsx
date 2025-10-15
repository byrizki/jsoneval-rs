import React, { useState } from 'react';
import {
  SafeAreaView,
  ScrollView,
  StatusBar,
  StyleSheet,
  Text,
  View,
  TouchableOpacity,
  useColorScheme,
} from 'react-native';
import FormValidationScreen from './src/screens/FormValidationScreen';
import DependentFieldsScreen from './src/screens/DependentFieldsScreen';

type TabType = 'validation' | 'dependent';

function App(): JSX.Element {
  const isDarkMode = useColorScheme() === 'dark';
  const [activeTab, setActiveTab] = useState<TabType>('validation');

  const backgroundStyle = {
    backgroundColor: isDarkMode ? '#1a1a1a' : '#f5f5f5',
    flex: 1,
  };

  return (
    <SafeAreaView style={backgroundStyle}>
      <StatusBar
        barStyle={isDarkMode ? 'light-content' : 'dark-content'}
        backgroundColor={backgroundStyle.backgroundColor}
      />
      <ScrollView
        contentInsetAdjustmentBehavior="automatic"
        style={backgroundStyle}>
        <View style={styles.container}>
          <View style={styles.header}>
            <Text style={[styles.title, { color: isDarkMode ? '#fff' : '#000' }]}>
              JSON Eval RS
            </Text>
            <Text style={[styles.subtitle, { color: isDarkMode ? '#aaa' : '#666' }]}>
              React Native Example
            </Text>
            <View style={styles.badges}>
              <View style={styles.badge}>
                <Text style={styles.badgeText}>Native</Text>
              </View>
              <View style={[styles.badge, { backgroundColor: '#10b981' }]}>
                <Text style={styles.badgeText}>Rust-Powered</Text>
              </View>
              <View style={[styles.badge, { backgroundColor: '#8b5cf6' }]}>
                <Text style={styles.badgeText}>Sequential</Text>
              </View>
            </View>
          </View>

          <View style={styles.tabs}>
            <TouchableOpacity
              style={[
                styles.tab,
                activeTab === 'validation' && styles.activeTab,
              ]}
              onPress={() => setActiveTab('validation')}>
              <Text
                style={[
                  styles.tabText,
                  activeTab === 'validation' && styles.activeTabText,
                  { color: isDarkMode ? '#fff' : '#000' },
                ]}>
                Form Validation
              </Text>
            </TouchableOpacity>
            <TouchableOpacity
              style={[
                styles.tab,
                activeTab === 'dependent' && styles.activeTab,
              ]}
              onPress={() => setActiveTab('dependent')}>
              <Text
                style={[
                  styles.tabText,
                  activeTab === 'dependent' && styles.activeTabText,
                  { color: isDarkMode ? '#fff' : '#000' },
                ]}>
                Dependent Fields
              </Text>
            </TouchableOpacity>
          </View>

          <View style={styles.content}>
            {activeTab === 'validation' ? (
              <FormValidationScreen />
            ) : (
              <DependentFieldsScreen />
            )}
          </View>

          <View style={styles.footer}>
            <Text style={[styles.footerText, { color: isDarkMode ? '#666' : '#999' }]}>
              Built with json-eval-rs
            </Text>
          </View>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 16,
  },
  header: {
    marginBottom: 24,
  },
  title: {
    fontSize: 32,
    fontWeight: 'bold',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 16,
    marginBottom: 16,
  },
  badges: {
    flexDirection: 'row',
    gap: 8,
    flexWrap: 'wrap',
  },
  badge: {
    backgroundColor: '#3b82f6',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
  },
  badgeText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '600',
  },
  tabs: {
    flexDirection: 'row',
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
    marginBottom: 24,
  },
  tab: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    borderBottomWidth: 2,
    borderBottomColor: 'transparent',
  },
  activeTab: {
    borderBottomColor: '#3b82f6',
  },
  tabText: {
    fontSize: 16,
  },
  activeTabText: {
    fontWeight: '600',
    color: '#3b82f6',
  },
  content: {
    flex: 1,
  },
  footer: {
    marginTop: 32,
    alignItems: 'center',
  },
  footerText: {
    fontSize: 14,
  },
});

export default App;
