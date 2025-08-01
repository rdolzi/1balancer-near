#!/bin/bash
set -e

# Build script that bypasses ABI generation for NEAR SDK 5.x compatibility issues

echo "Building Fusion+ HTLC contract (without ABI)..."

# Build the contract directly with cargo
cargo build --target wasm32-unknown-unknown --release

# Create output directory
mkdir -p ../../target/near/fusion_plus_htlc/

# Copy the wasm file to expected location
cp ../../target/wasm32-unknown-unknown/release/fusion_plus_htlc.wasm ../../target/near/fusion_plus_htlc/fusion_plus_htlc.wasm

# Get file size
SIZE=$(ls -lh ../../target/near/fusion_plus_htlc/fusion_plus_htlc.wasm | awk '{print $5}')
echo "✓ Contract built successfully (without ABI)"
echo "  Contract size: $SIZE"
echo ""
echo "Note: ABI generation is currently disabled due to JsonSchema compatibility issues with AccountId in NEAR SDK 5.x"
echo "This contract will work normally but without automatic ABI generation."