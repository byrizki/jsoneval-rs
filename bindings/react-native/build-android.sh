#!/bin/bash
# Build Rust static libraries for React Native Android
# This script builds the .a files needed by CMake

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Building Rust static libraries for React Native Android${NC}"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo not found. Please install Rust from https://rustup.rs${NC}"
    exit 1
fi

# Android target mapping
declare -A TARGETS=(
    ["arm64-v8a"]="aarch64-linux-android"
    ["armeabi-v7a"]="armv7-linux-androideabi"
    ["x86"]="i686-linux-android"
    ["x86_64"]="x86_64-linux-android"
)

# Check which targets are installed
echo "Checking installed Android targets..."
INSTALLED_TARGETS=$(rustup target list --installed)

for abi in "${!TARGETS[@]}"; do
    target="${TARGETS[$abi]}"
    if echo "$INSTALLED_TARGETS" | grep -q "$target"; then
        echo -e "${GREEN}✓${NC} $target (for $abi)"
    else
        echo -e "${YELLOW}✗${NC} $target (for $abi) - not installed"
    fi
done

echo ""
echo "To install missing targets, run:"
echo "  rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android"
echo ""

# Ask which ABIs to build
if [ -z "$1" ]; then
    echo "Usage: $0 [ABI|all]"
    echo "Available ABIs: arm64-v8a, armeabi-v7a, x86, x86_64, all"
    echo ""
    echo "Building for arm64-v8a (default)..."
    BUILD_ABIS=("arm64-v8a")
elif [ "$1" == "all" ]; then
    BUILD_ABIS=("arm64-v8a" "armeabi-v7a" "x86" "x86_64")
else
    BUILD_ABIS=("$1")
fi

# Navigate to repo root
cd "$(dirname "$0")/../.."

# Build for each ABI
for abi in "${BUILD_ABIS[@]}"; do
    target="${TARGETS[$abi]}"
    
    if [ -z "$target" ]; then
        echo -e "${RED}Unknown ABI: $abi${NC}"
        continue
    fi
    
    echo -e "${GREEN}Building for $abi ($target)...${NC}"
    
    # Build the shared library (.so)
    cargo build --release --target "$target" --features ffi
    
    # Check if the shared library was created
    SO_PATH="target/$target/release/libjson_eval_rs.so"
    if [ -f "$SO_PATH" ]; then
        echo -e "${GREEN}✓${NC} Shared library created: $SO_PATH"
        ls -lh "$SO_PATH"
        
        # Copy to jniLibs directory for the npm package
        JNILIBS_DIR="bindings/react-native/packages/react-native/android/src/main/jniLibs/$abi"
        mkdir -p "$JNILIBS_DIR"
        cp "$SO_PATH" "$JNILIBS_DIR/"
        echo -e "${GREEN}✓${NC} Copied to: $JNILIBS_DIR/libjson_eval_rs.so"
    else
        echo -e "${RED}✗${NC} Shared library not found at: $SO_PATH"
        echo "Available files:"
        ls -la "target/$target/release/" | grep json_eval || true
    fi
    
    echo ""
done

echo -e "${GREEN}Build complete!${NC}"
echo ""
echo "Shared libraries (.so) have been copied to:"
for abi in "${BUILD_ABIS[@]}"; do
    echo "  $abi: bindings/react-native/packages/react-native/android/src/main/jniLibs/$abi/libjson_eval_rs.so"
done
echo ""
echo "These libraries are bundled with the npm package."
echo ""
echo "Now you can build the React Native Android app:"
echo "  cd bindings/react-native/examples/rncli/android"
echo "  ./gradlew assembleDebug"
