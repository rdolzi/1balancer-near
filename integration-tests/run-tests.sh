#!/bin/bash

# Integration test runner for 1Balancer NEAR cross-chain tests

set -e

echo "🧪 1Balancer NEAR Integration Tests"
echo "==================================="

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Check required environment variables
required_vars=(
    "NEAR_NETWORK"
    "NEAR_NODE_URL" 
    "NEAR_HTLC_CONTRACT"
    "ETH_RPC_URL"
    "ETH_HUB_CONTRACT"
)

for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "❌ Missing required environment variable: $var"
        exit 1
    fi
done

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

# Run tests based on argument
case "$1" in
    "happy")
        echo "✅ Running happy path tests..."
        npm run test:happy
        ;;
    "edge")
        echo "⚠️  Running edge case tests..."
        npm run test:edge
        ;;
    "security")
        echo "🔒 Running security tests..."
        npm run test:security
        ;;
    "all")
        echo "🚀 Running all integration tests..."
        npm test
        ;;
    *)
        echo "Usage: $0 {happy|edge|security|all}"
        echo ""
        echo "Options:"
        echo "  happy    - Run happy path cross-chain swap tests"
        echo "  edge     - Run edge case and timeout tests"
        echo "  security - Run security validation tests"
        echo "  all      - Run all test suites"
        exit 1
        ;;
esac

echo ""
echo "✅ Integration tests completed!"