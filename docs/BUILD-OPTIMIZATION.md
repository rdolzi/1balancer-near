# NEAR Contract Build Optimization

## Problem

NEAR contracts built with standard `cargo build` may fail with `CompilationError(PrepareError(Deserialization))` when deployed. This happens because:

1. The WASM binary contains unnecessary sections that NEAR VM doesn't accept
2. The contract size is not optimized, leading to higher deployment costs
3. Debug information and other metadata interfere with NEAR's deserialization

## Solution

Our automated build process includes:

1. **wasm-strip**: Removes unnecessary sections from WASM binary
2. **wasm-opt**: Optimizes the contract for size and performance

## Automated Build Process

### Using Make (Recommended)
```bash
make build
```

This automatically:
- Checks for required tools
- Installs missing dependencies (macOS/Linux)
- Builds all contracts
- Strips and optimizes WASM files
- Shows size reduction statistics

### Using Build Script Directly
```bash
./scripts/build-contract.sh
```

### Manual Process (If Needed)
```bash
# 1. Build the contract
cargo build --target wasm32-unknown-unknown --release

# 2. Strip unnecessary sections
wasm-strip target/wasm32-unknown-unknown/release/fusion_plus_htlc.wasm

# 3. Optimize for size
wasm-opt -Os target/wasm32-unknown-unknown/release/fusion_plus_htlc.wasm \
         -o target/wasm32-unknown-unknown/release/fusion_plus_htlc.wasm
```

## Tool Installation

### macOS
```bash
brew install wabt      # For wasm-strip
brew install binaryen  # For wasm-opt
```

### Ubuntu/Debian
```bash
sudo apt-get install wabt
sudo apt-get install binaryen
```

### Other Linux/Windows
- wabt: https://github.com/WebAssembly/wabt
- binaryen: https://github.com/WebAssembly/binaryen

## Size Comparison

Example optimization results:
- Before: 199KB (unoptimized)
- After: 172KB (stripped & optimized)
- Reduction: ~14%

## Troubleshooting

### "cargo-near" Issues
While `cargo-near` is the official build tool, it requires:
- TTY environment (doesn't work in CI/CD)
- Additional ABI generation setup
- JsonSchema trait implementations

Our build script provides a simpler alternative that works everywhere.

### Deployment Errors
If you still get deserialization errors:
1. Ensure you're using the optimized WASM file
2. Check Rust version matches `rust-toolchain.toml` (1.86.0)
3. Verify NEAR SDK version compatibility (5.0.0)

## CI/CD Integration

Add to your GitHub Actions:
```yaml
- name: Install WASM tools
  run: |
    sudo apt-get update
    sudo apt-get install -y wabt binaryen

- name: Build NEAR contracts
  run: make build
```