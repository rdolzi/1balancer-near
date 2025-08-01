#!/bin/bash
set -e

# Solver Registry Contract Build Script
echo "Building Solver Registry contract..."

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

# Build the contract using cargo-near
echo "Building contract with cargo-near..."
cargo near build non-reproducible-wasm

# Check if build was successful
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Contract built successfully${NC}"
    
    # Create output directory
    mkdir -p ../../target/wasm32-unknown-unknown/release/
    
    # Copy the wasm file
    cp target/wasm32-unknown-unknown/release/solver_registry.wasm ../../target/wasm32-unknown-unknown/release/
    
    # Get file size
    SIZE=$(ls -lh target/wasm32-unknown-unknown/release/solver_registry.wasm | awk '{print $5}')
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