# 1Balancer NEAR Architecture

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Cross-Chain Integration](#cross-chain-integration)
4. [NEAR Contract Architecture](#near-contract-architecture)
5. [TEE Solver Architecture](#tee-solver-architecture)
6. [Security Architecture](#security-architecture)
7. [Event Flow and Monitoring](#event-flow-and-monitoring)
8. [Integration Points](#integration-points)

## Overview

The 1Balancer NEAR repository implements the NEAR Protocol side of the cross-chain atomic swap system, enabling trustless portfolio rebalancing between NEAR and EVM chains (primarily BASE). This architecture extends the Ethereum Hub design to support non-EVM chains through the Fusion+ protocol.

## System Architecture

### Three-Layer Architecture Extension

Building upon the Ethereum Hub's three-layer pattern, NEAR integration adds:

```mermaid
graph TB
    subgraph "APPLICATION LAYER"
        UI[1Balancer Frontend]
        PM[Portfolio Manager]
        API[Backend API]
    end
    
    subgraph "ORCHESTRATION LAYER"
        ORCH[Orchestration Service]
        SIM[Swap Simulator]
        MON[Event Monitor]
        COORD[Cross-Chain Coordinator]
    end
    
    subgraph "PROTOCOL LAYER"
        subgraph "EVM Side (BASE)"
            HUB[Ethereum Hub]
            ESC[Escrow Contracts]
            LOP[1inch LOP]
        end
        
        subgraph "NEAR Side (This Repo)"
            HTLC[NEAR HTLC Contract]
            REG[Solver Registry]
            CS[Chain Signatures]
            TEE[TEE Solver]
        end
    end
    
    UI --> API
    API --> ORCH
    ORCH --> COORD
    COORD --> HUB
    COORD --> HTLC
    
    style HTLC fill:#ff9800
    style REG fill:#ff9800
    style TEE fill:#4caf50
```

### Component Relationships

```mermaid
graph LR
    subgraph "1balancer Repository"
        EH[Ethereum Hub]
        OS[Orchestration Service]
        FE[Frontend]
    end
    
    subgraph "1balancer-near Repository"
        NH[NEAR HTLC]
        SR[Solver Registry]
        TS[TEE Solver]
        CS[Chain Signatures]
    end
    
    subgraph "External Services"
        NEAR[NEAR Protocol]
        BASE[BASE L2]
        INCH[1inch APIs]
        PHALA[Phala Network]
    end
    
    OS <--> EH
    OS <--> NH
    NH <--> NEAR
    TS --> PHALA
    TS <--> SR
    SR <--> CS
    CS <--> NEAR
    
    style NH fill:#ff9800
    style SR fill:#ff9800
    style TS fill:#4caf50
```

## Cross-Chain Integration

### Atomic Swap Flow

The NEAR integration implements the destination chain side of the atomic swap protocol:

```mermaid
sequenceDiagram
    participant User
    participant Orchestrator
    participant BASE as BASE Chain
    participant NEAR as NEAR Chain
    participant TEE as TEE Solver
    
    User->>Orchestrator: Initiate Swap
    Orchestrator->>Orchestrator: Generate Secret
    
    Note over Orchestrator: Create Hashlock
    
    Orchestrator->>BASE: Deploy Source Escrow
    BASE-->>Orchestrator: Escrow Created
    
    Orchestrator->>NEAR: Create HTLC
    NEAR-->>Orchestrator: HTLC Created
    
    alt Success Path
        Orchestrator->>NEAR: Reveal Secret
        NEAR->>NEAR: Validate & Transfer
        NEAR-->>Orchestrator: Secret Revealed Event
        Orchestrator->>BASE: Complete Swap
        BASE->>BASE: Release Funds
    else Timeout Path
        Note over NEAR: Timelock Expires
        NEAR->>NEAR: Refund to Sender
        Note over BASE: Safe to Refund
        BASE->>BASE: Refund to Maker
    end
```

### Timeout Coordination

Critical for atomicity, the timeout structure ensures safe execution:

```mermaid
gantt
    title Cross-Chain Timeout Coordination
    dateFormat HH:mm
    axisFormat %H:%M
    
    section NEAR Chain
    Active Period           :active, near1, 00:00, 24:00
    Refund Available        :crit, near2, 24:00, 48:00
    
    section BASE Chain
    Active Period           :active, base1, 00:00, 48:00
    Public Withdrawal       :done, base2, 48:00, 60:00
    Refund Available        :crit, base3, 60:00, 72:00
```

**Key Rule**: NEAR timeout (T_near) < BASE withdrawal (T_base)
- If NEAR fails, BASE can safely refund
- If NEAR succeeds, secret is available for BASE completion

## NEAR Contract Architecture

### Contract Structure

```mermaid
classDiagram
    class FusionPlusContract {
        -owner: AccountId
        -htlcs: LookupMap~String, HTLC~
        -active_htlcs: LookupMap~AccountId, UnorderedSet~
        -eth_orchestrator: String
        -supported_tokens: UnorderedSet~AccountId~
        +new(owner: AccountId)
        +create_htlc(args: HTLCCreateArgs): String
        +withdraw(htlc_id: String, secret: String): Promise
        +refund(htlc_id: String): Promise
        +ft_on_transfer(sender: AccountId, amount: U128, msg: String): U128
    }
    
    class HTLC {
        +sender: AccountId
        +receiver: AccountId
        +token: AccountId
        +amount: Balance
        +hashlock: String
        +timelock: Timestamp
        +secret: Option~String~
        +state: HTLCState
        +order_hash: Option~String~
    }
    
    class HTLCState {
        <<enumeration>>
        Active
        Withdrawn
        Refunded
        Expired
    }
    
    FusionPlusContract --> HTLC
    HTLC --> HTLCState
```

### Module Organization

```
contracts/fusion-plus-htlc/
├── src/
│   ├── lib.rs              # Main contract logic
│   ├── types.rs            # Data structures
│   ├── utils.rs            # Helper functions
│   ├── htlc/               # HTLC operations
│   │   ├── create.rs       # Creation logic
│   │   ├── withdraw.rs     # Secret revelation
│   │   └── refund.rs       # Timeout refunds
│   ├── cross_chain/        # Cross-chain coordination
│   │   ├── coordinator.rs  # State sync
│   │   └── events.rs       # Event emission
│   └── ft_receiver.rs      # NEP-141 support
```

### State Machine

```mermaid
stateDiagram-v2
    [*] --> Active: create_htlc()
    Active --> Withdrawn: withdraw(secret)
    Active --> Refunded: refund() after timeout
    Active --> Expired: automatic after timeout
    Withdrawn --> [*]
    Refunded --> [*]
    Expired --> [*]
```

## TEE Solver Architecture

### Decentralized Solver Design

```mermaid
graph TB
    subgraph "Phala Network TEE"
        subgraph "Secure Enclave"
            QE[Quote Engine]
            PC[Profit Calculator]
            EX[Executor]
            KS[Key Storage]
        end
        ATT[Attestation Service]
    end
    
    subgraph "NEAR Protocol"
        REG[Solver Registry]
        CS[Chain Signatures]
        HTLC[HTLC Contract]
    end
    
    subgraph "1inch Fusion+"
        API[Fusion API]
        ORD[Order Stream]
    end
    
    ORD --> QE
    QE --> PC
    PC --> EX
    EX --> CS
    CS --> API
    
    ATT --> REG
    REG --> HTLC
    
    style QE fill:#4caf50
    style ATT fill:#2196f3
```

### Solver Registration Flow

```mermaid
sequenceDiagram
    participant Solver
    participant TEE as Phala TEE
    participant Registry
    participant ChainSig as Chain Signatures
    
    Solver->>TEE: Deploy Code
    TEE->>TEE: Generate Attestation
    TEE-->>Solver: Attestation Proof
    
    Solver->>Registry: Register(attestation)
    Registry->>Registry: Verify Attestation
    Registry->>Registry: Store Solver Info
    
    Note over Registry: Solver Active
    
    Solver->>ChainSig: Request Signing Key
    ChainSig-->>Solver: MPC Key Share
    
    loop Quote Processing
        Solver->>Solver: Monitor Quotes
        Solver->>Solver: Calculate Profit
        Solver->>ChainSig: Sign Order
        ChainSig-->>Solver: Signature
    end
```

## Security Architecture

### Multi-Layer Security Model

```mermaid
graph TB
    subgraph "Application Security"
        AS1[Input Validation]
        AS2[Rate Limiting]
        AS3[Authentication]
    end
    
    subgraph "Protocol Security"
        PS1[Hashlock SHA-256]
        PS2[Timelock Enforcement]
        PS3[Access Control]
        PS4[State Machine Guards]
    end
    
    subgraph "Cross-Chain Security"
        CS1[Event Verification]
        CS2[Timeout Coordination]
        CS3[Atomic Guarantees]
        CS4[Replay Protection]
    end
    
    subgraph "TEE Security"
        TS1[Attestation Verification]
        TS2[Secure Key Storage]
        TS3[Isolated Execution]
        TS4[Tamper Protection]
    end
    
    AS1 --> PS1
    PS1 --> CS1
    CS1 --> TS1
```

### Attack Vector Mitigation

| Attack Vector | Mitigation Strategy |
|--------------|-------------------|
| Front-running | TEE execution isolation |
| Replay attacks | One-time secret usage |
| Timeout manipulation | Strict timestamp validation |
| Cross-chain race conditions | Enforced timeout ordering |
| Malicious solvers | TEE attestation requirement |
| Secret pre-revelation | SHA-256 commitment scheme |

## Event Flow and Monitoring

### Event Architecture

```mermaid
graph LR
    subgraph "NEAR Events"
        E1[HTLCCreated]
        E2[SecretRevealed]
        E3[HTLCRefunded]
        E4[StateChanged]
    end
    
    subgraph "Event Monitor"
        M1[NEAR Indexer]
        M2[Event Parser]
        M3[State Tracker]
    end
    
    subgraph "Orchestration"
        O1[Event Correlator]
        O2[Action Trigger]
        O3[State Sync]
    end
    
    E1 --> M1
    E2 --> M1
    E3 --> M1
    E4 --> M1
    
    M1 --> M2
    M2 --> M3
    M3 --> O1
    O1 --> O2
    O2 --> O3
```

### Critical Event Sequence

```mermaid
sequenceDiagram
    participant NEAR
    participant Monitor
    participant Orchestrator
    participant BASE
    
    NEAR->>Monitor: EVENT: HTLCCreated
    Monitor->>Orchestrator: Parse & Forward
    Orchestrator->>Orchestrator: Update State
    
    NEAR->>Monitor: EVENT: SecretRevealed
    Monitor->>Orchestrator: CRITICAL: Secret Available
    Orchestrator->>BASE: Complete BASE Side
    BASE-->>Orchestrator: Swap Completed
    
    Orchestrator->>Monitor: Mark Complete
    Monitor->>Monitor: Archive Events
```

## Integration Points

### 1. Orchestration Service Integration

```typescript
interface NEARIntegration {
    // Contract calls
    createHTLC(params: HTLCParams): Promise<string>;
    getHTLCStatus(id: string): Promise<HTLCState>;
    
    // Event monitoring
    onHTLCCreated(callback: (event: HTLCCreatedEvent) => void): void;
    onSecretRevealed(callback: (event: SecretRevealedEvent) => void): void;
    
    // State queries
    getActiveHTLCs(): Promise<HTLCInfo[]>;
    getCrossChainInfo(htlcId: string): Promise<CrossChainInfo>;
}
```

### 2. Frontend Integration

```typescript
// Hooks for NEAR integration
const useNEARSwap = () => {
    const createSwap = async (params: SwapParams) => {
        // Call orchestration service
        const session = await orchestrator.createSession({
            sourceChain: 'base',
            destinationChain: 'near',
            ...params
        });
        return session;
    };
    
    const monitorSwap = (sessionId: string) => {
        // WebSocket monitoring
        return orchestrator.subscribe(sessionId);
    };
    
    return { createSwap, monitorSwap };
};
```

### 3. Chain Signatures Integration

```rust
// MPC signing for solver operations
impl ChainSignatureClient {
    pub async fn request_signature(
        &self,
        payload: &[u8],
        path: &str,
    ) -> Result<Signature, Error> {
        // MPC protocol execution
        let request = SignatureRequest {
            payload: payload.to_vec(),
            path: path.to_string(),
            key_version: self.key_version,
        };
        
        self.mpc_contract
            .sign(request)
            .await
    }
}
```

## Performance Considerations

### Optimization Strategies

1. **Event Batching**: Process multiple events in single transaction
2. **State Caching**: Minimize cross-contract calls
3. **Gas Optimization**: Efficient data structures and storage patterns
4. **Parallel Processing**: Independent HTLC operations

### Benchmarks

| Operation | Gas Cost | Time |
|-----------|----------|------|
| Create HTLC | ~5 TGas | <2s |
| Withdraw | ~10 TGas | <3s |
| Refund | ~5 TGas | <2s |
| TEE Attestation | ~20 TGas | <5s |

## Future Enhancements

1. **Multi-signature HTLCs**: Support for complex authorization
2. **Batch Operations**: Multiple swaps in single transaction
3. **Advanced TEE Features**: Zero-knowledge proofs, private swaps
4. **Additional Chains**: Cosmos, Solana integration
5. **Liquidity Aggregation**: Cross-chain liquidity pools

## Conclusion

The NEAR integration extends 1Balancer's cross-chain capabilities to non-EVM ecosystems while maintaining the security and atomicity guarantees of the Fusion+ protocol. The architecture prioritizes:

- **Security**: Multi-layer validation and TEE isolation
- **Scalability**: Efficient event processing and state management
- **Usability**: Hidden complexity behind simple interfaces
- **Extensibility**: Modular design for future enhancements