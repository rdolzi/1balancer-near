#!/bin/bash
set -e

echo "🚀 Deploying to NEAR testnet..."
echo ""

# Check for NEAR CLI
if ! command -v near &> /dev/null; then
    echo "❌ NEAR CLI not found. Install with: npm install -g near-cli"
    exit 1
fi

# Check for credentials
if [ -z "$NEAR_MASTER_ACCOUNT" ]; then
    echo "❌ NEAR_MASTER_ACCOUNT not set"
    echo "Please export NEAR_MASTER_ACCOUNT=your-account.testnet"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Must run from 1balancer-near directory"
    exit 1
fi

# Define contract names as subaccounts
HTLC_CONTRACT="fusion-htlc.$NEAR_MASTER_ACCOUNT"
# SOLVER_CONTRACT="solver-registry.$NEAR_MASTER_ACCOUNT"  # TODO: Uncomment when implemented

echo "📋 Deployment plan:"
echo "  Master account: $NEAR_MASTER_ACCOUNT"
echo "  HTLC contract:  $HTLC_CONTRACT"
# echo "  Solver contract: $SOLVER_CONTRACT"  # TODO: Uncomment when implemented
echo ""
echo "⚠️  Note: Only deploying HTLC contract (Solver Registry not yet implemented)"
echo ""

# Check if contracts already exist
echo "🔍 Checking if contracts already exist..."
if near state "$HTLC_CONTRACT" 2>/dev/null | grep -q "amount:"; then
    echo "⚠️  Contract $HTLC_CONTRACT already exists"
    echo "   To redeploy, delete it first with:"
    echo "   near delete-account $HTLC_CONTRACT $NEAR_MASTER_ACCOUNT"
    HTLC_EXISTS=true
else
    echo "✅ Contract $HTLC_CONTRACT does not exist, ready to deploy"
    HTLC_EXISTS=false
fi

# TODO: Uncomment when Solver Registry is implemented
# if near state "$SOLVER_CONTRACT" 2>/dev/null | grep -q "amount:"; then
#     echo "⚠️  Contract $SOLVER_CONTRACT already exists"
#     echo "   To redeploy, delete it first with:"
#     echo "   near delete-account $SOLVER_CONTRACT $NEAR_MASTER_ACCOUNT"
#     SOLVER_EXISTS=true
# else
#     echo "✅ Contract $SOLVER_CONTRACT does not exist, ready to deploy"
#     SOLVER_EXISTS=false
# fi

echo ""

# Check if WASM files are optimized
WASM_FILE="target/wasm32-unknown-unknown/release/fusion_plus_htlc.wasm"
if [ -f "$WASM_FILE" ]; then
    # Check file size - unoptimized contracts are typically > 190KB
    FILE_SIZE=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null || echo "0")
    FILE_SIZE_KB=$((FILE_SIZE / 1024))
    
    if [ $FILE_SIZE_KB -gt 190 ]; then
        echo "⚠️  WARNING: Contract appears to be unoptimized (${FILE_SIZE_KB}KB)"
        echo "   Running optimization..."
        echo ""
        
        # Try to run build script
        if [ -f "./scripts/build-contract.sh" ]; then
            ./scripts/build-contract.sh
        else
            echo "❌ Build script not found. Contract may fail to deploy."
            echo "   Please run: make build"
            exit 1
        fi
    else
        echo "✅ Contract appears to be optimized (${FILE_SIZE_KB}KB)"
    fi
else
    echo "❌ Contract WASM file not found"
    echo "   Please run: make build"
    exit 1
fi

echo ""

# Deploy HTLC contract
if [ "$HTLC_EXISTS" = false ]; then
    echo "📄 Step 1/2: Creating subaccount for HTLC contract..."
    echo "   Creating: $HTLC_CONTRACT"
    
    # Create the subaccount (minimum balance for contract storage - 3 NEAR for safety)
    near create-account "$HTLC_CONTRACT" --masterAccount "$NEAR_MASTER_ACCOUNT" --initialBalance 3 || {
        echo "❌ Failed to create HTLC subaccount"
        echo ""
        # Check if it's a balance issue
        BALANCE=$(near state "$NEAR_MASTER_ACCOUNT" 2>/dev/null | grep -o 'amount: "[0-9.]*"' | cut -d'"' -f2 || echo "0")
        if [ -n "$BALANCE" ]; then
            echo "💰 Current balance: $BALANCE NEAR"
            echo "   Required: ~3 NEAR per contract (for storage costs)"
            echo ""
            echo "🚰 Get testnet NEAR from faucets:"
            echo "   1. Stakely (0.002 NEAR/day): https://stakely.io/en/faucet/near-testnet"
            echo "   2. Thirdweb (0.01 NEAR/day): https://thirdweb.com/near-testnet"
            echo "   3. NEAR Discord: Join and ask in #dev-support"
            echo ""
            echo "💡 Tip: Contract storage requires more NEAR than expected."
        else
            echo "   Make sure you have enough NEAR balance in $NEAR_MASTER_ACCOUNT"
        fi
        exit 1
    }
    
    echo ""
    echo "📄 Step 2/2: Deploying WASM code to the account..."
    echo "   Deploying: fusion_plus_htlc.wasm"
    
    # Deploy the contract
    near deploy "$HTLC_CONTRACT" target/wasm32-unknown-unknown/release/fusion_plus_htlc.wasm || {
        echo "❌ Failed to deploy HTLC contract code"
        exit 1
    }
    
    echo ""
    echo "🔧 Initializing contract..."
    # Initialize the contract
    near call "$HTLC_CONTRACT" new "{\"owner\": \"$NEAR_MASTER_ACCOUNT\"}" --accountId "$NEAR_MASTER_ACCOUNT" || {
        echo "⚠️  Contract initialization failed (this may be normal if no init method exists)"
    }
    
    echo ""
    echo "✅ HTLC contract deployed successfully to $HTLC_CONTRACT!"
else
    echo "⏭️  Skipping HTLC deployment (already exists)"
fi

echo ""

# TODO: Uncomment and implement when Solver Registry is ready
# # Deploy Solver Registry contract
# if [ "$SOLVER_EXISTS" = false ]; then
#     echo "📄 Step 1/2: Creating subaccount for Solver Registry..."
#     echo "   Creating: $SOLVER_CONTRACT"
#     
#     # Create the subaccount (minimum balance for testing - 0.1 NEAR)
#     near create-account "$SOLVER_CONTRACT" --masterAccount "$NEAR_MASTER_ACCOUNT" --initialBalance 0.1 || {
#         echo "❌ Failed to create Solver Registry subaccount"
#         echo "   Make sure you have enough NEAR balance in $NEAR_MASTER_ACCOUNT"
#         exit 1
#     }
#     
#     echo ""
#     echo "📄 Step 2/2: Deploying WASM code to the account..."
#     echo "   Deploying: solver_registry.wasm"
#     
#     # Deploy the contract
#     near deploy "$SOLVER_CONTRACT" target/wasm32-unknown-unknown/release/solver_registry.wasm || {
#         echo "❌ Failed to deploy Solver Registry contract code"
#         exit 1
#     }
#     
#     echo ""
#     echo "🔧 Initializing contract..."
#     # Initialize the contract
#     near call "$SOLVER_CONTRACT" new "{\"owner\": \"$NEAR_MASTER_ACCOUNT\"}" --accountId "$NEAR_MASTER_ACCOUNT" || {
#         echo "⚠️  Contract initialization failed (this may be normal if no init method exists)"
#     }
#     
#     echo ""
#     echo "✅ Solver Registry deployed successfully to $SOLVER_CONTRACT!"
# else
#     echo "⏭️  Skipping Solver Registry deployment (already exists)"
# fi

echo ""

# Save deployment info
mkdir -p .near-credentials/testnet
cat > .near-credentials/testnet/deploy.json << EOF
{
  "contractId": "$HTLC_CONTRACT",
  "masterAccount": "$NEAR_MASTER_ACCOUNT",
  "network": "testnet",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "📋 Deployment summary:"
echo "  ✅ HTLC contract: $HTLC_CONTRACT"
echo ""
echo "🌐 View on NEAR Explorer:"
echo "  https://testnet.nearblocks.io/address/$HTLC_CONTRACT"
echo ""
echo "📡 Available contract methods:"
echo ""
echo "  VIEW METHODS (free, no gas):"
echo "    near view $HTLC_CONTRACT get_info"
echo "    near view $HTLC_CONTRACT get_owner"
echo ""
echo "  CHANGE METHODS (require gas):"
echo "    # Coming soon: create_htlc, claim_htlc, refund_htlc"
echo ""
echo "✅ Deployment complete!"