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

# Build the contract
echo "Building contract..."
cargo build --target wasm32-unknown-unknown --release

# Check if build was successful
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Contract built successfully${NC}"
    
    # Create output directory
    mkdir -p ../../target/wasm32-unknown-unknown/release/
    
    # Copy the wasm file
    cp target/wasm32-unknown-unknown/release/fusion_plus_htlc.wasm ../../target/wasm32-unknown-unknown/release/
    
    # Get file size
    SIZE=$(ls -lh target/wasm32-unknown-unknown/release/fusion_plus_htlc.wasm | awk '{print $5}')
    echo -e "${GREEN}✓ Contract size: $SIZE${NC}"
    
    # Run tests
    echo "Running tests..."
    cargo test
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ All tests passed${NC}"
    else
        echo -e "${RED}✗ Tests failed${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

echo -e "${GREEN}Build complete!${NC}"