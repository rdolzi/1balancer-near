#!/bin/bash
set -e

# NEAR HTLC Contract Build Script
echo "Building Fusion+ HTLC contract..."

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
REQUIRED_VERSION="1.86.0"

if [ "$RUST_VERSION" != "$REQUIRED_VERSION" ]; then
    echo -e "${RED}Error: Rust version $REQUIRED_VERSION is required, but $RUST_VERSION is installed${NC}"
    echo "Please install the correct version using:"
    echo "  rustup install $REQUIRED_VERSION"
    echo "  rustup default $REQUIRED_VERSION"
    exit 1
fi

echo "✓ Rust version $RUST_VERSION"

# Check if cargo-near is installed
if ! command -v cargo-near &> /dev/null; then
    echo -e "${RED}Error: cargo-near is not installed${NC}"
    echo "Please install cargo-near using:"
    echo "  cargo install cargo-near"
    exit 1
fi

# Build the contract using custom build script (ABI generation disabled)
echo "Building contract (ABI generation temporarily disabled)..."
./build-without-abi.sh
exit $?

# Check if build was successful
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Contract built successfully${NC}"
    
    # Create output directory
    mkdir -p ../../target/wasm32-unknown-unknown/release/
    
    # Copy the wasm file from cargo-near output location
    CARGO_NEAR_FILE="target/near/fusion_plus_htlc/fusion_plus_htlc.wasm"
    
    if [ -f "$CARGO_NEAR_FILE" ]; then
        cp "$CARGO_NEAR_FILE" ../../target/wasm32-unknown-unknown/release/
        
        # Get file size
        SIZE=$(ls -lh "$CARGO_NEAR_FILE" | awk '{print $5}')
        echo -e "${GREEN}✓ Contract size: $SIZE${NC}"
    else
        echo -e "${RED}✗ WASM file not found at expected location${NC}"
        exit 1
    fi
    
    # Run tests (skip for now as they need special setup for NEAR contracts)
    echo "Skipping tests in build script (run 'make test-unit' separately)"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

echo -e "${GREEN}Build complete!${NC}"