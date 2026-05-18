const {getDefaultConfig, mergeConfig} = require('@react-native/metro-config');
const path = require('path');
const pak = require('../../packages/react-native/package.json');

const root = path.resolve(__dirname, '../..');
const packageRoot = path.resolve(__dirname, '../../packages/react-native');

/**
 * Metro configuration
 * https://reactnative.dev/docs/metro
 *
 * @type {import('metro-config').MetroConfig}
 */
const config = {
  watchFolders: [root, packageRoot],

  // We need to make sure that only one version is loaded for peerDependencies
  // So we exclude them at the root, and alias them to the versions in example's node_modules
  resolver: {
    unstable_enableSymlinks: true,
    extraNodeModules: {
      '@json-eval-rs/react-native': path.resolve(
        __dirname,
        '../../packages/react-native',
      ),
    },
  },

  transformer: {
    getTransformOptions: async () => ({
      transform: {
        experimentalImportSupport: false,
        inlineRequires: true,
      },
    }),
  },
};

module.exports = mergeConfig(getDefaultConfig(__dirname), config);
