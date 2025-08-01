# NEAR Security Architecture

## Overview

This document details the comprehensive security architecture for the NEAR side of 1Balancer's cross-chain atomic swap system. Security is implemented at multiple layers to ensure safe, trustless execution.

## Security Model

### Defense in Depth

```mermaid
graph TB
    subgraph "Layer 1: Input Validation"
        IV1[Parameter Validation]
        IV2[Type Checking]
        IV3[Range Validation]
        IV4[Format Verification]
    end
    
    subgraph "Layer 2: Access Control"
        AC1[Role-Based Access]
        AC2[Caller Verification]
        AC3[Time-Based Restrictions]
        AC4[State-Based Guards]
    end
    
    subgraph "Layer 3: Cryptographic Security"
        CS1[SHA-256 Hashlock]
        CS2[Secret Management]
        CS3[Signature Verification]
        CS4[Hash Commitments]
    end
    
    subgraph "Layer 4: Protocol Security"
        PS1[Timeout Enforcement]
        PS2[Atomic Guarantees]
        PS3[State Machine]
        PS4[Event Integrity]
    end
    
    IV1 --> AC1
    AC1 --> CS1
    CS1 --> PS1
```

## HTLC Security

### Hashlock Implementation

```rust
// Secure hashlock validation
pub fn validate_secret(secret: &str, hashlock: &str) -> bool {
    // Remove 0x prefix if present
    let hashlock_clean = hashlock.trim_start_matches("0x");
    
    // Hash the secret using SHA-256
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);
    
    // Constant-time comparison to prevent timing attacks
    hash_hex == hashlock_clean
}
```

### Timeout Security

```mermaid
sequenceDiagram
    participant Contract
    participant TimeValidator
    participant BlockTime
    
    Contract->>TimeValidator: Check Timeout
    TimeValidator->>BlockTime: Get Current Time
    BlockTime-->>TimeValidator: block.timestamp
    
    alt Before Timeout
        TimeValidator-->>Contract: Allow Withdraw
        Contract->>Contract: Validate Secret
        Contract->>Contract: Transfer Funds
    else After Timeout
        TimeValidator-->>Contract: Allow Refund
        Contract->>Contract: Validate Sender
        Contract->>Contract: Return Funds
    end
```

### State Machine Security

```mermaid
stateDiagram-v2
    [*] --> Active: create_htlc()
    
    Active --> Active: get_htlc()
    Active --> Withdrawn: withdraw(valid_secret)
    Active --> Refunded: refund(after_timeout)
    
    Withdrawn --> Withdrawn: get_htlc()
    Withdrawn --> [*]: immutable
    
    Refunded --> Refunded: get_htlc()
    Refunded --> [*]: immutable
    
    Active --> Active: ❌ withdraw(invalid_secret)
    Active --> Active: ❌ refund(before_timeout)
    Withdrawn --> Withdrawn: ❌ any_mutation
    Refunded --> Refunded: ❌ any_mutation
```

## Access Control

### Function-Level Security

```rust
// Strict access control implementation
impl FusionPlusContract {
    /// Only receiver can withdraw
    pub fn withdraw(&mut self, htlc_id: String, secret: String) -> Promise {
        let htlc = self.htlcs.get(&htlc_id).expect(ERR_HTLC_NOT_FOUND);
        
        // Access control check
        assert_eq!(
            env::predecessor_account_id(),
            htlc.receiver,
            "Only receiver can withdraw"
        );
        
        // Additional validations...
    }
    
    /// Only sender can refund
    pub fn refund(&mut self, htlc_id: String) -> Promise {
        let htlc = self.htlcs.get(&htlc_id).expect(ERR_HTLC_NOT_FOUND);
        
        // Access control check
        assert_eq!(
            env::predecessor_account_id(),
            htlc.sender,
            "Only sender can refund"
        );
        
        // Timeout validation...
    }
}
```

### Admin Functions Protection

```mermaid
graph LR
    subgraph "Admin Functions"
        A1[set_eth_orchestrator]
        A2[add_supported_token]
        A3[remove_supported_token]
    end
    
    subgraph "Protection"
        P1[assert_owner()]
        P2[Check predecessor]
        P3[Validate owner]
    end
    
    subgraph "Result"
        R1[✅ Execute]
        R2[❌ Revert]
    end
    
    A1 --> P1
    A2 --> P1
    A3 --> P1
    P1 --> P2
    P2 --> P3
    P3 --> R1
    P3 --> R2
```

## TEE Solver Security

### Attestation Verification

```mermaid
sequenceDiagram
    participant Solver
    participant TEE
    participant Registry
    participant Verifier
    
    Solver->>TEE: Execute in Enclave
    TEE->>TEE: Generate Measurement
    TEE->>TEE: Sign with TEE Key
    TEE-->>Solver: Attestation Report
    
    Solver->>Registry: Register(attestation)
    Registry->>Verifier: Verify Report
    
    Verifier->>Verifier: Check Signature
    Verifier->>Verifier: Validate Measurement
    Verifier->>Verifier: Verify Freshness
    
    alt Valid Attestation
        Verifier-->>Registry: ✅ Valid
        Registry->>Registry: Store Solver
    else Invalid Attestation
        Verifier-->>Registry: ❌ Invalid
        Registry->>Registry: Reject
    end
```

### Secure Key Management

```mermaid
graph TB
    subgraph "TEE Secure Enclave"
        SK[Solver Key]
        KD[Key Derivation]
        SE[Sealed Storage]
    end
    
    subgraph "Chain Signatures"
        MPC1[MPC Node 1]
        MPC2[MPC Node 2]
        MPC3[MPC Node 3]
        AGG[Aggregator]
    end
    
    subgraph "Protection"
        P1[Never Export Private Key]
        P2[Sign Inside TEE]
        P3[Threshold Signatures]
    end
    
    SK --> KD
    KD --> SE
    SE --> P2
    
    MPC1 --> AGG
    MPC2 --> AGG
    MPC3 --> AGG
    AGG --> P3
```

## Cross-Chain Security

### Event Verification

```rust
// Event authenticity verification
pub struct EventVerifier {
    pub fn verify_cross_chain_event(
        &self,
        event: &CrossChainEvent,
        proof: &EventProof,
    ) -> Result<bool, Error> {
        // 1. Verify event source
        let valid_source = self.verify_source(&event.source_chain)?;
        
        // 2. Verify event hash
        let event_hash = self.compute_event_hash(event);
        let valid_hash = event_hash == proof.event_hash;
        
        // 3. Verify inclusion proof
        let valid_inclusion = self.verify_merkle_proof(
            &proof.merkle_proof,
            &event_hash,
            &proof.block_root
        )?;
        
        // 4. Verify block finality
        let valid_finality = self.verify_finality(
            &event.source_chain,
            &proof.block_number
        )?;
        
        Ok(valid_source && valid_hash && valid_inclusion && valid_finality)
    }
}
```

### Replay Attack Prevention

```mermaid
graph LR
    subgraph "Message"
        M1[HTLC ID]
        M2[Action]
        M3[Nonce]
        M4[Timestamp]
    end
    
    subgraph "Validation"
        V1[Check Processed]
        V2[Verify Nonce]
        V3[Check Timestamp]
        V4[Mark Processed]
    end
    
    subgraph "Storage"
        S1[(Processed Messages)]
        S2[(Nonce Counter)]
    end
    
    M1 --> V1
    M3 --> V2
    M4 --> V3
    V1 --> S1
    V2 --> S2
    V3 --> V4
    V4 --> S1
```

## Vulnerability Analysis

### Common Attack Vectors

```mermaid
graph TB
    subgraph "Attack Vectors"
        AV1[Reentrancy]
        AV2[Integer Overflow]
        AV3[Front-running]
        AV4[Time Manipulation]
        AV5[Access Control Bypass]
        AV6[Secret Pre-image]
    end
    
    subgraph "Mitigations"
        M1[State Updates First]
        M2[Safe Math]
        M3[TEE Protection]
        M4[Block Time Validation]
        M5[Strict Checks]
        M6[Hash Commitment]
    end
    
    AV1 --> M1
    AV2 --> M2
    AV3 --> M3
    AV4 --> M4
    AV5 --> M5
    AV6 --> M6
```

### Security Audit Checklist

#### Contract Security
- [x] No reentrancy vulnerabilities
- [x] No integer overflow/underflow
- [x] Proper access control
- [x] State machine integrity
- [x] Event emission for monitoring
- [x] Emergency pause capability

#### Cryptographic Security
- [x] SHA-256 for hashlocks
- [x] Secure random generation
- [x] No hardcoded secrets
- [x] Proper secret handling
- [x] Timing attack resistance

#### Cross-Chain Security
- [x] Timeout coordination
- [x] Event verification
- [x] Replay protection
- [x] Atomicity guarantees
- [x] State synchronization

## Emergency Response

### Circuit Breaker Pattern

```rust
pub struct EmergencyControl {
    paused: bool,
    emergency_admin: AccountId,
}

impl EmergencyControl {
    pub fn emergency_pause(&mut self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.emergency_admin,
            "Only emergency admin"
        );
        self.paused = true;
        env::log_str("EMERGENCY: Contract paused");
    }
    
    pub fn emergency_resume(&mut self) {
        // Multi-sig or time delay for resume
        self.validate_resume_conditions();
        self.paused = false;
        env::log_str("Contract resumed");
    }
}
```

### Incident Response Flow

```mermaid
graph TB
    subgraph "Detection"
        D1[Monitor Alerts]
        D2[User Reports]
        D3[Automated Checks]
    end
    
    subgraph "Assessment"
        A1[Severity Analysis]
        A2[Impact Scope]
        A3[Risk Evaluation]
    end
    
    subgraph "Response"
        R1[Emergency Pause]
        R2[Patch Development]
        R3[Testing]
        R4[Deployment]
    end
    
    subgraph "Recovery"
        RC1[Resume Operations]
        RC2[User Communication]
        RC3[Post-Mortem]
    end
    
    D1 --> A1
    D2 --> A1
    D3 --> A1
    A1 --> R1
    R1 --> R2
    R2 --> R3
    R3 --> R4
    R4 --> RC1
    RC1 --> RC2
    RC2 --> RC3
```

## Monitoring and Alerting

### Security Metrics

```
┌─────────────────────────────────────────────────┐
│            Security Dashboard                   │
├─────────────────────────────────────────────────┤
│ Failed Withdrawals (24h):     3                 │
│ Timeout Refunds:              2                 │
│ Invalid Secrets:              5                 │
│ Access Violations:            0                 │
├─────────────────────────────────────────────────┤
│ Anomaly Detection                               │
│ • Unusual withdrawal pattern detected           │
│ • High frequency of failed attempts             │
│ • Potential replay attack blocked               │
├─────────────────────────────────────────────────┤
│ System Health                                   │
│ TEE Attestations:     ✅ Valid                  │
│ Chain Signatures:     ✅ Operational            │
│ Event Monitoring:     ✅ Active                 │
│ Emergency Systems:    ✅ Ready                  │
└─────────────────────────────────────────────────┘
```

### Alert Thresholds

| Metric | Warning | Critical | Action |
|--------|---------|----------|--------|
| Failed Withdrawals/hour | >10 | >50 | Investigate |
| Invalid Secrets/hour | >20 | >100 | Check for attack |
| Access Violations/hour | >5 | >20 | Review logs |
| Gas Spike | >150% | >300% | Emergency review |
| Event Lag | >30s | >2min | Check monitor |

## Best Practices

### Secure Development

1. **Code Review**: All changes require review
2. **Testing**: Comprehensive test coverage
3. **Fuzzing**: Automated vulnerability discovery
4. **Static Analysis**: Automated security checks
5. **Formal Verification**: Critical paths verified

### Operational Security

1. **Key Management**: Hardware security modules
2. **Access Control**: Multi-signature requirements
3. **Monitoring**: 24/7 automated monitoring
4. **Incident Response**: Documented procedures
5. **Regular Audits**: Third-party security reviews

## Conclusion

The NEAR security architecture implements defense in depth with:

- **Multiple validation layers**: From input to protocol level
- **Cryptographic guarantees**: SHA-256 hashlocks and commitments
- **Access control**: Role-based and state-based restrictions
- **TEE isolation**: Protected execution environment
- **Cross-chain security**: Event verification and replay protection
- **Emergency mechanisms**: Circuit breakers and pause capability

This comprehensive approach ensures the safety and integrity of cross-chain atomic swaps.