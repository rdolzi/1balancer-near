#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🚀 Building NEAR smart contracts..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Must run from 1balancer-near directory${NC}"
    exit 1
fi

# Function to check and install dependencies
check_dependencies() {
    local missing_deps=()
    
    # Check for wasm-strip
    if ! command -v wasm-strip &> /dev/null; then
        echo -e "${YELLOW}⚠️  wasm-strip not found${NC}"
        missing_deps+=("wabt")
    fi
    
    # Check for wasm-opt
    if ! command -v wasm-opt &> /dev/null; then
        echo -e "${YELLOW}⚠️  wasm-opt not found${NC}"
        missing_deps+=("binaryen")
    fi
    
    # Install missing dependencies
    if [ ${#missing_deps[@]} -gt 0 ]; then
        echo -e "${YELLOW}📦 Installing missing dependencies...${NC}"
        
        # Detect OS
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            if command -v brew &> /dev/null; then
                for dep in "${missing_deps[@]}"; do
                    echo "   Installing $dep..."
                    brew install $dep
                done
            else
                echo -e "${RED}❌ Homebrew not found. Please install dependencies manually:${NC}"
                echo "   brew install ${missing_deps[*]}"
                exit 1
            fi
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            # Linux
            if command -v apt-get &> /dev/null; then
                # Debian/Ubuntu
                for dep in "${missing_deps[@]}"; do
                    echo "   Installing $dep..."
                    sudo apt-get install -y $dep
                done
            elif command -v yum &> /dev/null; then
                # RedHat/CentOS
                echo -e "${RED}❌ Please install manually:${NC}"
                echo "   wabt: https://github.com/WebAssembly/wabt"
                echo "   binaryen: https://github.com/WebAssembly/binaryen"
                exit 1
            else
                echo -e "${RED}❌ Unsupported Linux distribution. Please install manually:${NC}"
                echo "   wabt: https://github.com/WebAssembly/wabt"
                echo "   binaryen: https://github.com/WebAssembly/binaryen"
                exit 1
            fi
        else
            echo -e "${RED}❌ Unsupported OS. Please install manually:${NC}"
            echo "   wabt: https://github.com/WebAssembly/wabt"
            echo "   binaryen: https://github.com/WebAssembly/binaryen"
            exit 1
        fi
    fi
}

# Check and install dependencies
check_dependencies

# Build contracts
echo ""
echo "📦 Building contracts..."

# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Process each contract
for contract_dir in contracts/*/; do
    if [ -f "$contract_dir/Cargo.toml" ]; then
        contract_name=$(basename "$contract_dir" | tr '-' '_')
        wasm_file="target/wasm32-unknown-unknown/release/${contract_name}.wasm"
        
        if [ -f "$wasm_file" ]; then
            echo ""
            echo "📄 Processing $contract_name..."
            
            # Get original size
            original_size=$(ls -lh "$wasm_file" | awk '{print $5}')
            echo "   Original size: $original_size"
            
            # Strip unnecessary sections
            echo "   Stripping unnecessary sections..."
            wasm-strip "$wasm_file"
            
            # Optimize for size
            echo "   Optimizing for size..."
            wasm-opt -Os "$wasm_file" -o "${wasm_file%.wasm}_optimized.wasm"
            
            # Replace with optimized version
            mv "${wasm_file%.wasm}_optimized.wasm" "$wasm_file"
            
            # Get final size
            final_size=$(ls -lh "$wasm_file" | awk '{print $5}')
            echo -e "   ${GREEN}✅ Final size: $final_size${NC}"
        fi
    fi
done

echo ""
echo -e "${GREEN}✅ Build complete!${NC}"
echo ""
echo "📋 Built contracts:"
ls -lh target/wasm32-unknown-unknown/release/*.wasm 2>/dev/null | grep -v "_optimized" || echo "   No contracts found"
echo ""
echo "💡 To deploy, run: ./scripts/deploy/deploy-testnet.sh"