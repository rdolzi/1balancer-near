# Cross-Chain Workflow Documentation

## Overview

This document details the complete cross-chain atomic swap workflow between BASE (Ethereum L2) and NEAR Protocol,
showing how the two chains coordinate to ensure atomic execution.

## Workflow Architecture

### High-Level Flow

```mermaid
graph TB
    subgraph "User Journey"
        U1[User Initiates Swap]
        U2[Approve Tokens]
        U3[Monitor Progress]
        U4[Swap Complete]
    end
    
    subgraph "Orchestration"
        O1[Generate Secret]
        O2[Deploy Escrows]
        O3[Monitor Events]
        O4[Coordinate Execution]
    end
    
    subgraph "Blockchain Execution"
        B1[BASE: Lock Tokens]
        B2[NEAR: Lock Tokens]
        B3[Reveal Secret]
        B4[Complete Both Sides]
    end
    
    U1 --> O1
    O1 --> O2
    O2 --> B1
    O2 --> B2
    B1 --> O3
    B2 --> O3
    O3 --> O4
    O4 --> B3
    B3 --> B4
    B4 --> U4
    U3 -.-> O3
```

## Detailed Workflow Steps

### 1. Initialization Phase

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant Orchestrator
    participant SecretManager
    
    User->>Frontend: Request Swap (BASE→NEAR)
    Frontend->>Orchestrator: POST /api/sessions
    
    Orchestrator->>SecretManager: Generate Secret
    SecretManager-->>Orchestrator: secret + hashlock
    
    Orchestrator->>Orchestrator: Calculate Timelocks
    Note over Orchestrator: T_near = 24h<br/>T_base = 48h
    
    Orchestrator-->>Frontend: Session Created
    Frontend-->>User: Display Session ID
```

### 2. Source Chain Lock (BASE)

```mermaid
sequenceDiagram
    participant Orchestrator
    participant FusionHub as Fusion+ Hub
    participant EscrowFactory
    participant Escrow as EscrowSrc
    participant LOP as 1inch LOP
    
    Orchestrator->>FusionHub: Create Order
    FusionHub->>LOP: Build Limit Order
    
    Orchestrator->>EscrowFactory: deploySrc()
    EscrowFactory->>Escrow: CREATE2 Deploy
    
    Note over Escrow: Immutables:<br/>maker, taker, token<br/>amount, hashlock<br/>timelocks
    
    LOP->>Escrow: Fill Order (tokens)
    Escrow-->>Orchestrator: EVENT: SrcEscrowCreated
```

### 3. Destination Chain Lock (NEAR)

```mermaid
sequenceDiagram
    participant Orchestrator
    participant NEARContract as NEAR HTLC
    participant Token as NEP-141 Token
    participant EventStream
    
    Note over Orchestrator: Detected BASE Lock
    
    Orchestrator->>NEARContract: create_htlc()
    NEARContract->>NEARContract: Validate Params
    
    alt NEP-141 Token
        Token->>NEARContract: ft_on_transfer()
        NEARContract->>NEARContract: Create HTLC
    else Native NEAR
        Orchestrator->>NEARContract: Attach Deposit
        NEARContract->>NEARContract: Create HTLC
    end
    
    NEARContract-->>EventStream: EVENT: HTLCCreated
    EventStream-->>Orchestrator: Notify
```

### 4. Secret Revelation Phase

```mermaid
sequenceDiagram
    participant Orchestrator
    participant NEARContract
    participant BaseEscrow
    participant Monitor
    
    Note over Orchestrator: Both Chains Locked
    
    Orchestrator->>NEARContract: withdraw(htlc_id, secret)
    NEARContract->>NEARContract: Validate Secret
    NEARContract->>NEARContract: Transfer to Receiver
    NEARContract-->>Monitor: EVENT: SecretRevealed
    
    Monitor->>Orchestrator: Secret Available!
    
    Orchestrator->>BaseEscrow: withdraw(secret)
    BaseEscrow->>BaseEscrow: Validate & Transfer
    BaseEscrow-->>Monitor: EVENT: Withdrawn
    
    Note over Orchestrator: Swap Complete ✓
```

### 5. Timeout Handling

```mermaid
sequenceDiagram
    participant Timer
    participant NEAR as NEAR Chain
    participant BASE as BASE Chain
    participant Orchestrator
    
    Timer->>Timer: Monitor Timeouts
    
    alt NEAR Timeout First (24h)
        Timer->>NEAR: Check HTLC Status
        NEAR-->>Timer: Still Active
        Timer->>Orchestrator: Initiate Refund
        Orchestrator->>NEAR: refund()
        NEAR->>NEAR: Return Tokens
        Note over BASE: Safe to Refund
        Timer->>BASE: Wait for BASE timeout (48h)
        Orchestrator->>BASE: cancel()
        BASE->>BASE: Return Tokens
    else Success Before Timeout
        Note over Timer: Secret Revealed
        Timer->>Timer: Cancel Timeout
    end
```

## State Management

### Session States

```mermaid
stateDiagram-v2
    [*] --> Initialized: create_session
    Initialized --> SourceLocking: initiate_swap
    SourceLocking --> SourceLocked: escrow_created
    SourceLocked --> DestinationLocking: lock_destination
    DestinationLocking --> BothLocked: htlc_created
    BothLocked --> Revealing: reveal_secret
    Revealing --> Completed: both_withdrawn
    
    SourceLocking --> Failed: lock_failed
    DestinationLocking --> Failed: lock_failed
    BothLocked --> Timeout: timeout_reached
    Timeout --> Refunding: initiate_refund
    Refunding --> Refunded: refund_complete
    
    Failed --> [*]
    Completed --> [*]
    Refunded --> [*]
```

### Event Correlation

```mermaid
graph LR
    subgraph "BASE Events"
        BE1[SrcEscrowCreated]
        BE2[SecretRevealed]
        BE3[EscrowCancelled]
    end
    
    subgraph "NEAR Events"
        NE1[HTLCCreated]
        NE2[HTLCWithdrawn]
        NE3[HTLCRefunded]
    end
    
    subgraph "Correlation Engine"
        CE[Event Correlator]
        SM[State Machine]
        DB[(Session Store)]
    end
    
    BE1 --> CE
    BE2 --> CE
    BE3 --> CE
    NE1 --> CE
    NE2 --> CE
    NE3 --> CE
    
    CE --> SM
    SM --> DB
```

## Error Scenarios

### 1. NEAR Lock Failure

```mermaid
sequenceDiagram
    participant Orchestrator
    participant BASE
    participant NEAR
    
    Note over BASE: Tokens Locked
    
    Orchestrator->>NEAR: create_htlc()
    NEAR-->>Orchestrator: ERROR: Insufficient Balance
    
    Orchestrator->>Orchestrator: Mark Failed
    Orchestrator->>Orchestrator: Wait for Timeout
    
    Note over Orchestrator: After BASE timeout
    
    Orchestrator->>BASE: cancel()
    BASE->>BASE: Refund Tokens
```

### 2. Network Partition

```mermaid
sequenceDiagram
    participant Orchestrator
    participant BASE
    participant NEAR
    participant Backup as Backup Monitor
    
    Note over Orchestrator: Primary Monitor
    
    Orchestrator->>BASE: Monitor Events
    Orchestrator->>NEAR: Monitor Events
    
    Note over Orchestrator: Network Issue
    
    Backup->>Backup: Detect Primary Offline
    Backup->>BASE: Take Over Monitoring
    Backup->>NEAR: Take Over Monitoring
    
    Note over Backup: Continue Operation
```

## Performance Optimization

### Parallel Processing

```mermaid
graph TB
    subgraph "Sequential (Slow)"
        S1[Create BASE Escrow]
        S2[Wait Confirmation]
        S3[Create NEAR HTLC]
        S4[Wait Confirmation]
        S1 --> S2
        S2 --> S3
        S3 --> S4
    end
    
    subgraph "Parallel (Fast)"
        P1[Create BASE Escrow]
        P2[Create NEAR HTLC]
        P3[Monitor Both]
        P4[Coordinate]
        P1 --> P3
        P2 --> P3
        P3 --> P4
    end
```

### Event Batching

```mermaid
graph LR
    subgraph "Event Stream"
        E1[Event 1]
        E2[Event 2]
        E3[Event 3]
        E4[Event 4]
    end
    
    subgraph "Batch Processor"
        BP[Batch Events]
        PR[Process Together]
        UP[Update State Once]
    end
    
    E1 --> BP
    E2 --> BP
    E3 --> BP
    E4 --> BP
    BP --> PR
    PR --> UP
```

## Monitoring and Analytics

### Key Metrics

```mermaid
graph TB
    subgraph "Swap Metrics"
        M1[Total Swaps]
        M2[Success Rate]
        M3[Average Duration]
        M4[Timeout Rate]
    end
    
    subgraph "Chain Metrics"
        C1[BASE Gas Used]
        C2[NEAR Gas Used]
        C3[Event Lag]
        C4[Confirmation Time]
    end
    
    subgraph "System Health"
        H1[Orchestrator Uptime]
        H2[Monitor Status]
        H3[Secret Manager]
        H4[Error Rate]
    end
```

### Monitoring Dashboard

```
┌─────────────────────────────────────────────────┐
│            Cross-Chain Swap Monitor             │
├─────────────────────────────────────────────────┤
│ Active Swaps: 12    │ Completed Today: 156     │
│ Success Rate: 98.5% │ Avg Duration: 3m 24s     │
├─────────────────────────────────────────────────┤
│ Chain Status        │ Latest Events            │
│ BASE:   ✅ Connected │ SrcEscrowCreated  2m ago │
│ NEAR:   ✅ Connected │ HTLCWithdrawn     1m ago │
├─────────────────────────────────────────────────┤
│ Recent Swaps                                    │
│ #1234: BASE→NEAR  1000 USDC  ✅ Complete        │
│ #1235: NEAR→BASE  50 NEAR    ⏳ Processing     │
│ #1236: BASE→NEAR  500 USDT   ⏳ Processing     │
└─────────────────────────────────────────────────┘
```

## Security Considerations

### Attack Prevention

```mermaid
graph TB
    subgraph "Attack Vectors"
        A1[Front-running]
        A2[Replay Attack]
        A3[Timeout Manipulation]
        A4[Secret Pre-reveal]
    end
    
    subgraph "Mitigations"
        M1[TEE Execution]
        M2[Nonce Tracking]
        M3[Strict Validation]
        M4[Hash Commitment]
    end
    
    A1 --> M1
    A2 --> M2
    A3 --> M3
    A4 --> M4
```

### Security Checklist

- [ ] Secret generation uses cryptographically secure random
- [ ] Hashlock validation on both chains
- [ ] Timeout ordering enforced (T_near < T_base)
- [ ] Event authenticity verification
- [ ] Access control on critical functions
- [ ] Reentrancy protection
- [ ] Integer overflow protection
- [ ] Emergency pause mechanism

## Integration Guide

### For Frontend Developers

```typescript
// Simple swap initiation
const swap = await orchestrator.createSwap({
    from: 'base',
    to: 'near',
    token: 'USDC',
    amount: '1000',
    recipient: 'alice.near'
});

// Monitor progress
swap.on('status', (status) => {
    console.log(`Swap ${status.step}: ${status.message}`);
});

// Handle completion
swap.on('complete', (result) => {
    console.log('Swap completed!', result.txHashes);
});
```

### For Backend Integration

```typescript
// Orchestration service integration
class SwapOrchestrator {
    async handleSwap(params: SwapParams) {
        // 1. Generate secret
        const { secret, hashlock } = await this.generateSecret();
        
        // 2. Calculate timelocks
        const timelocks = this.calculateTimelocks();
        
        // 3. Deploy on both chains
        const [baseTx, nearTx] = await Promise.all([
            this.deployBaseEscrow(params, hashlock, timelocks),
            this.createNEARHTLC(params, hashlock, timelocks)
        ]);
        
        // 4. Monitor and coordinate
        await this.monitorAndComplete(baseTx, nearTx, secret);
    }
}
```

## Conclusion

The cross-chain workflow ensures atomic execution through careful coordination of:

1. **Timeout Management**: NEAR expires before BASE
2. **Secret Management**: One-time reveal ensures atomicity
3. **Event Monitoring**: Real-time state synchronization
4. **Error Handling**: Graceful degradation and recovery

This design enables trustless cross-chain swaps while maintaining security and performance.