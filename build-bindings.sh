#!/bin/bash
# Build script for all JSON Eval RS bindings
# Usage: ./build-bindings.sh [target]
# Targets: all, csharp, web, react-native

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}$1${NC}"
    echo -e "${GREEN}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "ℹ $1"
}

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed. Please install Rust from https://rustup.rs"
    exit 1
fi

TARGET="${1:-all}"

# Build C# bindings
build_csharp() {
    print_header "Building C# Bindings (FFI)"
    
    print_warning "Building without parallel feature (optimized for sequential workloads)"
    print_warning "To enable parallel processing: cargo build --release --features ffi,parallel"
    
    # Build for Linux
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        cargo build --release --features ffi
        print_success "Built Linux x64 library: target/release/libjson_eval_rs.so"
    fi
    
    # Build for macOS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        cargo build --release --features ffi
        print_success "Built macOS library: target/release/libjson_eval_rs.dylib"
    fi
    
    # Build for Windows (if cross-compiling)
    if command -v cargo-xwin &> /dev/null || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        cargo build --release --features ffi --target x86_64-pc-windows-msvc 2>/dev/null || \
        cargo build --release --features ffi
        print_success "Built Windows library: target/release/json_eval_rs.dll"
    fi
    
    # Try to build C# project if dotnet is available
    if command -v dotnet &> /dev/null; then
        cd bindings/csharp
        dotnet build -c Release
        print_success "Built C# wrapper library"
        cd ../..
    else
        print_warning "dotnet CLI not found. Skipping C# project build"
        print_warning "Install .NET SDK from https://dotnet.microsoft.com/download"
    fi
}

# Build Web bindings
build_web() {
    print_header "Building Web Bindings (WASM)"
    
    print_warning "Building without parallel feature (WASM doesn't support rayon)"
    print_warning "Sequential processing is optimal for web workloads"
    
    # Check if wasm-pack is installed
    if ! command -v wasm-pack &> /dev/null; then
        print_error "wasm-pack is not installed"
        print_warning "Install with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
        return 1
    fi
    
    # Build for bundler target (Webpack, Next.js, Vite, etc.)
    print_info "Building for bundlers (Webpack, Vite, Next.js, etc.)..."
    wasm-pack build --target bundler --out-dir bindings/web/packages/bundler/pkg --features wasm
    print_success "Built WASM for bundlers → bindings/web/packages/bundler/pkg/"
        
    # Build for Node.js target
    print_info "Building for Node.js/SSR..."
    wasm-pack build --target nodejs --out-dir bindings/web/packages/node/pkg --features wasm
    print_success "Built WASM for Node.js → bindings/web/packages/node/pkg/"
    
    # Install web dependencies
    if command -v npm &> /dev/null; then
        cd bindings/web
        npm install
        print_success "Installed web dependencies"
        cd ../..
    fi
    
    print_success "Web bindings built successfully!"
    print_info "Packages:"
    print_info "  - @json-eval-rs/bundler: packages/bundler/ (for Webpack, Vite, Next.js, etc.)"
    print_info "  - @json-eval-rs/node: packages/node/ (for Node.js/SSR)"
    print_info "  - @json-eval-rs/core: packages/core/ (internal API wrapper)"
}

# Build React Native bindings
build_react_native() {
    print_header "Building React Native Bindings"
    
    print_warning "Building without parallel feature (optimized for mobile devices)"
    print_warning "Sequential processing is optimal for React Native workloads"
    
    # Build Android JNI libraries
    print_warning "React Native bindings require additional setup:"
    print_warning "1. For Android: Configure NDK in android/build.gradle"
    print_warning "2. For iOS: Configure Rust bindings in ios/*.podspec"
    print_warning "3. See bindings/react-native/README.md for detailed instructions"
    
    # Build for Android (all architectures)
    if command -v cargo-ndk &> /dev/null; then
        print_info "Building Android JNI libraries for all architectures..."
        cargo ndk -t arm64-v8a -o bindings/react-native/android/src/main/jniLibs build --release --features ffi
        print_success "Built Android arm64-v8a library"
        
        cargo ndk -t armeabi-v7a -o bindings/react-native/android/src/main/jniLibs build --release --features ffi
        print_success "Built Android armeabi-v7a library"
        
        cargo ndk -t x86 -o bindings/react-native/android/src/main/jniLibs build --release --features ffi
        print_success "Built Android x86 library"
        
        cargo ndk -t x86_64 -o bindings/react-native/android/src/main/jniLibs build --release --features ffi
        print_success "Built Android x86_64 library"
    else
        print_warning "cargo-ndk not installed. Skipping Android build"
        print_warning "Install with: cargo install cargo-ndk"
    fi
    
    # Build for iOS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        cargo build --release --features ffi --target aarch64-apple-ios
        cargo build --release --features ffi --target x86_64-apple-ios
        print_success "Built iOS libraries"
    else
        print_warning "iOS builds require macOS. Skipping"
    fi
    
    # Install npm dependencies
    if command -v npm &> /dev/null; then
        cd bindings/react-native
        if [ -f "package.json" ]; then
            npm install
            npm run prepare 2>/dev/null || true
            print_success "Installed React Native dependencies"
        fi
        cd ../..
        
        # Install example dependencies
        print_info "Installing example dependencies..."
        cd bindings/react-native/examples/rncli
        if [ -f "package.json" ]; then
            npm install 2>/dev/null || true
            print_success "Installed rncli example dependencies"
        fi
        cd ../../../..
    fi
}

# Package for publishing
package_csharp() {
    print_header "Packaging C# NuGet Package"
    
    if ! command -v dotnet &> /dev/null; then
        print_error "dotnet CLI not found"
        return 1
    fi
    
    cd bindings/csharp
    dotnet pack -c Release -o ../../dist/nuget
    cd ../..
    print_success "NuGet package created in dist/nuget/"
}

package_web() {
    print_header "Packaging Web npm Package"
    
    if ! command -v npm &> /dev/null; then
        print_error "npm not found"
        return 1
    fi
    
    cd bindings/web
    npm pack --pack-destination ../../dist/npm
    cd ../..
    print_success "npm package created in dist/npm/"
}

package_react_native() {
    print_header "Packaging React Native npm Package"
    
    if ! command -v npm &> /dev/null; then
        print_error "npm not found"
        return 1
    fi
    
    cd bindings/react-native
    npm pack --pack-destination ../../dist/npm
    cd ../..
    print_success "npm package created in dist/npm/"
}

# Create distribution directory
mkdir -p dist/nuget dist/npm

# Build based on target
case "$TARGET" in
    all)
        build_csharp
        build_web
        build_react_native
        ;;
    csharp)
        build_csharp
        ;;
    web)
        build_web
        ;;
    react-native)
        build_react_native
        ;;
    package)
        package_csharp 2>/dev/null || print_warning "C# packaging skipped"
        package_web 2>/dev/null || print_warning "Web packaging skipped"
        package_react_native 2>/dev/null || print_warning "React Native packaging skipped"
        ;;
    *)
        print_error "Unknown target: $TARGET"
        echo "Usage: $0 [all|csharp|web|react-native|package]"
        exit 1
        ;;
esac

print_header "Build Complete!"
print_success "All requested bindings have been built"

echo ""
echo "Next steps:"
echo "  - For C#: See bindings/csharp/README.md"
echo "  - For Web: See bindings/web/README.md"
echo "  - For React Native: See bindings/react-native/README.md"
echo ""
echo "To package for publishing, run: $0 package"
