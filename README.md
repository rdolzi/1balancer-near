# 1Balancer NEAR Challenges

This repository contains the NEAR Protocol implementation for 1Balancer's cross-chain atomic swap functionality, focusing on the 1inch Fusion+ hackathon challenges.

## Overview

1Balancer NEAR implements:
- **Challenge 1**: Fusion+ HTLC integration for cross-chain atomic swaps
- **Challenge 2**: Decentralized solver with TEE attestation

## Prerequisites

- Rust 1.86.0 (CRITICAL - specified in rust-toolchain.toml)
- Node.js 20+
- Docker for TEE solver deployment
- NEAR CLI tools

## Quick Start

```bash
# Install dependencies
make check-rust

# Build all contracts
make build

# Run tests
make test

# Deploy to testnet
make deploy-testnet
```

## Repository Structure

```
1balancer-near/
├── contracts/               # NEAR smart contracts
│   ├── fusion-plus-htlc/   # Challenge 1: Fusion+ integration
│   └── solver-registry/    # Challenge 2: TEE solver registry
├── shade-agent-solver/     # Decentralized solver implementation
├── integration-tests/      # Cross-chain integration tests
├── scripts/               # Deployment and utility scripts
└── docs/                  # NEAR-specific documentation
```

## Integration with Main Application

This repository works in conjunction with the main 1balancer application repository, providing the NEAR side of cross-chain atomic swaps. The Ethereum Hub on BASE chain communicates with these NEAR contracts through the orchestration layer.

## Development

See the [documentation](docs/) for detailed development guides and API references.