/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  webpack: (config, { isServer }) => {
    // Fix for WebAssembly - only enable on client side
    if (!isServer) {
      config.experiments = {
        ...config.experiments,
        asyncWebAssembly: true,
        layers: true,
      };

      // Handle .wasm files
      config.module.rules.push({
        test: /\.wasm$/,
        type: 'webassembly/async',
      });

      config.output.webassemblyModuleFilename = 'static/wasm/[modulehash].wasm';
      
      // Suppress async/await warning - modern browsers support it
      config.output.environment = {
        ...config.output.environment,
        asyncFunction: true,
      };
    }

    // Fallback for Node.js modules
    config.resolve.fallback = {
      ...config.resolve.fallback,
      fs: false,
      path: false,
      crypto: false,
    };

    return config;
  },
};

module.exports = nextConfig;
