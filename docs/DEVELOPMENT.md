# NEAR Development Guide

## Prerequisites

- Rust 1.86.0 (CRITICAL - enforced by rust-toolchain.toml)
- Node.js 20+
- NEAR CLI tools
- Docker (for TEE solver)

## Initial Setup

1. Install Rust toolchain:
```bash
rustup default 1.86.0
rustup target add wasm32-unknown-unknown
```

2. Install NEAR tools:
```bash
cargo install cargo-near
npm install -g near-cli
```

3. Build contracts:
```bash
make build
```

## Contract Development

### Fusion+ HTLC Contract

Location: `contracts/fusion-plus-htlc/`

Key methods:
- `create_htlc`: Initialize cross-chain swap
- `withdraw`: Complete swap with secret
- `refund`: Reclaim funds on timeout

### Solver Registry

Location: `contracts/solver-registry/`

Features:
- TEE attestation verification
- Liquidity pool management
- Solver registration

## Solver Development

### Setup
```bash
cd shade-agent-solver
npm install
cp .env.example .env
# Edit .env with your configuration
```

### Local Testing
```bash
npm run dev
```

### TEE Deployment
```bash
docker build -t shade-agent-solver .
./scripts/deploy/deploy-solver-tee.sh
```

## Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cd integration-tests
npm test
```

### Cross-Chain Tests
Requires both 1balancer and 1balancer-near running:
```bash
./scripts/test/test-cross-chain.sh
```

## Deployment

### Testnet
```bash
make deploy-testnet
```

### Mainnet
```bash
# After security audit only
make deploy-mainnet
```