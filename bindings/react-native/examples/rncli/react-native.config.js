const path = require('path');

module.exports = {
  // Point to the package in the monorepo
  dependencies: {
    '@json-eval-rs/react-native': {
      root: path.join(__dirname, '../../packages/react-native'),
    },
  },
};
