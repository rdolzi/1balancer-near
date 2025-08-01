# Fusion+ HTLC Contract

This is the NEAR-side implementation of the Hash Time Locked Contract (HTLC) for the 1Balancer cross-chain atomic swap protocol.

## Overview

The Fusion+ HTLC contract enables atomic swaps between NEAR Protocol and EVM chains (primarily BASE) by implementing:

- **Hashlock**: SHA-256 based secret validation
- **Timelock**: Time-based expiration for refunds
- **Cross-chain coordination**: Events and state synchronization with Ethereum Hub
- **NEP-141 support**: Full fungible token standard compatibility
- **Native NEAR support**: Direct NEAR token swaps

## Architecture Alignment

This contract is part of the three-layer architecture:

```
Application Layer (Portfolio Management)
           ↓
Orchestration Layer (Cross-chain Coordination)
           ↓
Protocol Layer (This Contract + Ethereum Hub)
```

## Key Features

### 1. Cross-Chain Timeout Coordination

The contract implements the critical timeout coordination pattern:

```
NEAR (Destination): |--Withdraw--|--Cancel-->
                    0           T1         T2

BASE (Source):      |--Withdraw--|--Public--|--Cancel-->
                    0          T1'        T2'       T3'

Where: T2 < T1' (NEAR expires before BASE withdrawal)
```

This ensures atomicity - if NEAR fails, BASE can be safely refunded.

### 2. NEP-141 Token Support

The contract implements `ft_on_transfer` to receive tokens atomically:

```rust
pub fn ft_on_transfer(
    &mut self,
    sender_id: AccountId,
    amount: U128,
    msg: String, // Contains HTLC parameters
) -> PromiseOrValue<U128>
```

### 3. Event Emission for Monitoring

All state changes emit structured events for the orchestration service:

- `HTLCCreatedEvent`: New HTLC created
- `HTLCWithdrawnEvent`: Secret revealed and funds withdrawn
- `HTLCRefundedEvent`: Timeout refund executed
- `SECRET_REVEALED`: Critical for cross-chain coordination

## Contract Interface

### Core Functions

```rust
// Create HTLC (for native NEAR)
pub fn create_htlc(&mut self, args: HTLCCreateArgs) -> String

// Withdraw with secret revelation
pub fn withdraw(&mut self, htlc_id: String, secret: String) -> Promise

// Refund after timeout
pub fn refund(&mut self, htlc_id: String) -> Promise

// Query functions
pub fn get_htlc(&self, htlc_id: String) -> Option<HTLC>
pub fn get_cross_chain_info(&self, htlc_id: String) -> Option<CrossChainInfo>
pub fn get_active_htlcs(&self, from_index: u64, limit: u64) -> Vec<HTLCInfo>
```

### Admin Functions

```rust
// Set Ethereum orchestrator address
pub fn set_eth_orchestrator(&mut self, orchestrator: String)

// Token management
pub fn add_supported_token(&mut self, token: AccountId)
pub fn remove_supported_token(&mut self, token: AccountId)
```

## Building

**IMPORTANT**: Requires Rust 1.86.0 exactly

```bash
# Check Rust version
rustc --version  # Must be 1.86.0

# Build the contract
./build.sh

# Or manually
cargo build --target wasm32-unknown-unknown --release
```

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --features integration-tests
```

## Deployment

```bash
# Deploy to NEAR testnet
near deploy --wasmFile target/wasm32-unknown-unknown/release/fusion_plus_htlc.wasm --accountId YOUR_ACCOUNT.testnet

# Initialize
near call YOUR_ACCOUNT.testnet new '{"owner": "YOUR_ACCOUNT.testnet"}' --accountId YOUR_ACCOUNT.testnet
```

## Integration with Orchestration Service

The orchestration service monitors this contract's events to coordinate cross-chain swaps:

1. **Monitor Creation**: Watch for `HTLCCreatedEvent`
2. **Track State**: Use `get_cross_chain_info` for status
3. **Detect Secret**: Listen for `SECRET_REVEALED` events
4. **Complete Swap**: Coordinate with Ethereum Hub

## Security Considerations

1. **Hashlock Validation**: Only SHA-256 hashes accepted
2. **Timelock Enforcement**: Strict timeout checking
3. **Access Control**: Only receiver can withdraw, only sender can refund
4. **State Machine**: One-way state transitions prevent replay
5. **Event Verification**: All critical actions emit verifiable events

## Example Usage

### Creating an HTLC

```bash
# For native NEAR
near call fusion-htlc.testnet create_htlc '{
  "args": {
    "receiver": "alice.testnet",
    "token": "near",
    "amount": "1000000000000000000000000",
    "hashlock": "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b",
    "timelock": 1234567890,
    "order_hash": "0x123"
  }
}' --accountId bob.testnet --deposit 1

# For NEP-141 tokens
near call token.testnet ft_transfer_call '{
  "receiver_id": "fusion-htlc.testnet",
  "amount": "1000000",
  "msg": "{\"receiver\":\"alice.testnet\",\"hashlock\":\"...\",\"timelock\":1234567890}"
}' --accountId bob.testnet --depositYocto 1
```

### Withdrawing with Secret

```bash
near call fusion-htlc.testnet withdraw '{
  "htlc_id": "abc123",
  "secret": "mysecret"
}' --accountId alice.testnet
```

### Refunding after Timeout

```bash
near call fusion-htlc.testnet refund '{
  "htlc_id": "abc123"
}' --accountId bob.testnet
```

## Cross-Chain Flow

1. **User initiates swap** on frontend
2. **Orchestration service** generates secret and coordinates
3. **BASE escrow** is created first (source chain)
4. **NEAR HTLC** is created with same hashlock
5. **Secret revealed** on NEAR (destination)
6. **Orchestration** detects secret and completes BASE side
7. **Atomic swap** completed!

## Hackathon Notes

- Testnet only deployment for demo
- Orchestration service simulates resolver behavior (no KYC)
- Focus on demonstrating atomic cross-chain execution
- All complexity hidden from frontend users