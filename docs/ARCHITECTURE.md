# 1Balancer NEAR Architecture

## Overview

This repository implements the NEAR Protocol side of 1Balancer's cross-chain atomic swap functionality, specifically targeting the 1inch Fusion+ hackathon challenges.

## Challenge Implementations

### Challenge 1: Fusion+ HTLC Integration

**Location**: `contracts/fusion-plus-htlc/`

Key features:
- Hashlock and timelock functionality preservation
- Bidirectional swap support (NEAR ↔ Ethereum)
- NEP-141 (FT) token support
- Cross-chain event coordination

Architecture:
```
fusion-plus-htlc/
├── htlc/           # Core HTLC logic
├── cross_chain/    # Cross-chain coordination
└── ft_receiver/    # Token reception handling
```

### Challenge 2: Decentralized Solver

**Location**: `shade-agent-solver/`

Components:
- TEE-based solver running on Phala Cloud
- Chain Signatures for secure cross-chain signing
- Liquidity pool management
- Fusion+ order processing

Architecture:
```
shade-agent-solver/
├── solver/         # Core solver logic
├── tee/           # TEE attestation
├── near/          # NEAR blockchain interaction
└── fusion/        # 1inch Fusion+ integration
```

## Integration Points

### With Ethereum Hub (BASE)

Communication flow:
1. Ethereum Hub initiates swap with Fusion+ order
2. NEAR HTLC contract creates mirror swap
3. Orchestration layer coordinates execution
4. Atomic completion through hashlock reveal

### With TEE Solver

The solver:
1. Monitors Fusion+ orders
2. Provides liquidity and quotes
3. Executes swaps with TEE attestation
4. Uses Chain Signatures for cross-chain operations

## Security Considerations

- Rust 1.86.0 required (specified in rust-toolchain.toml)
- Hashlock verification across chains
- Timeout coordination for safety
- TEE attestation for solver trust

## Development Workflow

1. **Contract Development**: Use `cargo near` commands
2. **Solver Development**: TypeScript with TEE constraints
3. **Testing**: Unit tests + integration tests + cross-chain scenarios
4. **Deployment**: Testnet first, then mainnet with security audit