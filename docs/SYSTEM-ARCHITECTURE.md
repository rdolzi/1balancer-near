# NEAR System Architecture

## Overview

This document provides comprehensive architectural diagrams and explanations of how the NEAR components integrate with the overall 1Balancer system, following the pattern established in the Ethereum Hub documentation.

## System Component Architecture

### Complete System Integration

```mermaid
graph TB
    subgraph "User Interface Layer"
        UI[Portfolio Dashboard]
        WC[Wallet Connect]
        AS[Account Selector]
    end
    
    subgraph "Application Services"
        API[REST API]
        WS[WebSocket Server]
        AUTH[Auth Service]
    end
    
    subgraph "Orchestration Layer"
        subgraph "Core Services"
            ORCH[Orchestration Engine]
            SIM[Swap Simulator]
            SM[Session Manager]
            EM[Event Monitor]
        end
        
        subgraph "Chain Coordinators"
            BCC[BASE Chain Coordinator]
            NCC[NEAR Chain Coordinator]
            CCC[Cross-Chain Coordinator]
        end
    end
    
    subgraph "Protocol Layer - EVM"
        subgraph "BASE Chain (L2)"
            HUB[Ethereum Hub]
            ESC[Escrow Factory]
            ESRC[EscrowSrc]
            EDST[EscrowDst]
            LOP[1inch LOP]
        end
    end
    
    subgraph "Protocol Layer - NEAR"
        subgraph "NEAR Protocol"
            HTLC[Fusion+ HTLC]
            REG[Solver Registry]
            CS[Chain Signatures]
        end
        
        subgraph "TEE Infrastructure"
            SOLVER[TEE Solver]
            ATT[Attestation Service]
            KMS[Key Management]
        end
    end
    
    subgraph "External Services"
        INCH[1inch APIs]
        NEAR_RPC[NEAR RPC]
        BASE_RPC[BASE RPC]
        PHALA[Phala Network]
    end
    
    %% User Flow
    UI --> API
    API --> ORCH
    WS --> SM
    
    %% Orchestration Flow
    ORCH --> SIM
    ORCH --> SM
    SM --> BCC
    SM --> NCC
    BCC --> CCC
    NCC --> CCC
    
    %% Chain Integration
    BCC --> HUB
    HUB --> ESC
    ESC --> ESRC
    ESC --> EDST
    HUB --> LOP
    
    NCC --> HTLC
    NCC --> REG
    HTLC --> CS
    
    %% External Integration
    LOP --> INCH
    BCC --> BASE_RPC
    NCC --> NEAR_RPC
    SOLVER --> PHALA
    
    %% Event Flow
    EM --> BCC
    EM --> NCC
    
    style HTLC fill:#ff9800
    style REG fill:#ff9800
    style SOLVER fill:#4caf50
    style CCC fill:#e91e63
```

## Cross-Chain Message Flow

### Message Passing Architecture

```mermaid
graph LR
    subgraph "Message Generation"
        MG1[User Action]
        MG2[Create Message]
        MG3[Sign Message]
        MG4[Add Metadata]
    end
    
    subgraph "Source Chain Processing"
        SC1[Validate Message]
        SC2[Lock Assets]
        SC3[Emit Event]
        SC4[Generate Proof]
    end
    
    subgraph "Cross-Chain Bridge"
        BR1[Event Monitor]
        BR2[Message Queue]
        BR3[Validator Set]
        BR4[Consensus]
    end
    
    subgraph "Destination Chain Processing"
        DC1[Verify Proof]
        DC2[Validate State]
        DC3[Execute Action]
        DC4[Emit Confirmation]
    end
    
    subgraph "Finalization"
        F1[Confirm Both Sides]
        F2[Update State]
        F3[Notify User]
    end
    
    MG1 --> MG2 --> MG3 --> MG4
    MG4 --> SC1
    SC1 --> SC2 --> SC3 --> SC4
    SC4 --> BR1
    BR1 --> BR2 --> BR3 --> BR4
    BR4 --> DC1
    DC1 --> DC2 --> DC3 --> DC4
    DC4 --> F1 --> F2 --> F3
```

## NEAR Contract Internal Architecture

### Contract Component Interaction

```mermaid
graph TB
    subgraph "Contract Entry Points"
        EP1[create_htlc]
        EP2[withdraw]
        EP3[refund]
        EP4[ft_on_transfer]
    end
    
    subgraph "Core Logic"
        subgraph "HTLC Management"
            HM1[HTLC Creation]
            HM2[Secret Validation]
            HM3[Timeout Check]
            HM4[State Transition]
        end
        
        subgraph "Token Handling"
            TH1[NEP-141 Receiver]
            TH2[Native NEAR]
            TH3[Token Transfer]
            TH4[Balance Tracking]
        end
        
        subgraph "Cross-Chain Coordination"
            CC1[Event Emission]
            CC2[Order Hash Link]
            CC3[Timeout Sync]
            CC4[State Query]
        end
    end
    
    subgraph "Storage Layer"
        ST1[(HTLC Storage)]
        ST2[(Active HTLCs)]
        ST3[(Token Registry)]
        ST4[(Config Storage)]
    end
    
    subgraph "Security Layer"
        SEC1[Access Control]
        SEC2[Input Validation]
        SEC3[State Guards]
        SEC4[Reentrancy Protection]
    end
    
    EP1 --> SEC2
    EP2 --> SEC1
    EP3 --> SEC1
    EP4 --> TH1
    
    SEC2 --> HM1
    SEC1 --> HM2
    SEC1 --> HM3
    
    HM1 --> ST1
    HM1 --> ST2
    HM2 --> HM4
    HM3 --> HM4
    HM4 --> ST1
    
    TH1 --> TH3
    TH2 --> TH3
    TH3 --> TH4
    
    HM4 --> CC1
    CC1 --> CC2
    
    style SEC1 fill:#f44336
    style SEC2 fill:#f44336
    style SEC3 fill:#f44336
    style SEC4 fill:#f44336
```

## Event-Driven Architecture

### Event Processing Pipeline

```mermaid
graph TB
    subgraph "Event Sources"
        ES1[NEAR Blockchain]
        ES2[BASE Blockchain]
        ES3[User Actions]
        ES4[System Timers]
    end
    
    subgraph "Event Collection"
        subgraph "NEAR Events"
            NE1[HTLCCreated]
            NE2[SecretRevealed]
            NE3[HTLCRefunded]
            NE4[SolverRegistered]
        end
        
        subgraph "BASE Events"
            BE1[EscrowCreated]
            BE2[SecretRevealed]
            BE3[EscrowCancelled]
            BE4[OrderFilled]
        end
    end
    
    subgraph "Event Processing"
        subgraph "Event Router"
            ER[Event Dispatcher]
            EF[Event Filter]
            EV[Event Validator]
        end
        
        subgraph "Event Handlers"
            EH1[HTLC Handler]
            EH2[Escrow Handler]
            EH3[Order Handler]
            EH4[Timeout Handler]
        end
        
        subgraph "State Machine"
            SM1[Load State]
            SM2[Validate Transition]
            SM3[Apply Changes]
            SM4[Persist State]
        end
    end
    
    subgraph "Action Execution"
        AE1[Chain Actions]
        AE2[Notifications]
        AE3[State Updates]
        AE4[Error Handling]
    end
    
    ES1 --> NE1
    ES1 --> NE2
    ES1 --> NE3
    ES1 --> NE4
    
    ES2 --> BE1
    ES2 --> BE2
    ES2 --> BE3
    ES2 --> BE4
    
    NE1 --> ER
    NE2 --> ER
    NE3 --> ER
    NE4 --> ER
    BE1 --> ER
    BE2 --> ER
    BE3 --> ER
    BE4 --> ER
    
    ER --> EF --> EV
    
    EV --> EH1
    EV --> EH2
    EV --> EH3
    EV --> EH4
    
    EH1 --> SM1
    EH2 --> SM1
    EH3 --> SM1
    EH4 --> SM1
    
    SM1 --> SM2 --> SM3 --> SM4
    
    SM4 --> AE1
    SM4 --> AE2
    SM4 --> AE3
    
    ES3 --> ER
    ES4 --> EH4
    
    style NE2 fill:#4caf50
    style BE2 fill:#4caf50
```

## TEE Solver Architecture Detail

### Solver Component Architecture

```mermaid
graph TB
    subgraph "External Interfaces"
        IF1[1inch Fusion API]
        IF2[NEAR RPC]
        IF3[Orchestrator API]
        IF4[Phala Network]
    end
    
    subgraph "TEE Secure Enclave"
        subgraph "Input Processing"
            IP1[Quote Receiver]
            IP2[Data Validator]
            IP3[Rate Limiter]
        end
        
        subgraph "Core Logic"
            CL1[Profit Calculator]
            CL2[Risk Analyzer]
            CL3[Order Builder]
            CL4[Strategy Engine]
        end
        
        subgraph "Secure Operations"
            SO1[Key Derivation]
            SO2[Signature Generation]
            SO3[Attestation Creation]
            SO4[Sealed Storage]
        end
        
        subgraph "Output Processing"
            OP1[Order Submitter]
            OP2[Event Logger]
            OP3[Metrics Collector]
        end
    end
    
    subgraph "Chain Integration"
        CI1[Solver Registry]
        CI2[Chain Signatures]
        CI3[HTLC Contract]
    end
    
    IF1 --> IP1
    IP1 --> IP2
    IP2 --> IP3
    
    IP3 --> CL1
    CL1 --> CL2
    CL2 --> CL3
    CL3 --> CL4
    
    CL4 --> SO1
    SO1 --> SO2
    SO2 --> SO3
    
    SO3 --> OP1
    OP1 --> IF1
    
    SO3 --> CI1
    SO2 --> CI2
    OP1 --> CI3
    
    IF2 --> IP1
    IF3 --> IP1
    IF4 --> SO3
    
    OP2 --> IF3
    OP3 --> IF3
    
    style SO1 fill:#2196f3
    style SO2 fill:#2196f3
    style SO3 fill:#2196f3
    style SO4 fill:#2196f3
```

## Data Flow Architecture

### Complete Data Flow Through System

```mermaid
graph TB
    subgraph "Data Sources"
        DS1[User Input]
        DS2[Blockchain State]
        DS3[Price Feeds]
        DS4[Order Book]
    end
    
    subgraph "Data Processing Pipeline"
        subgraph "Ingestion Layer"
            IL1[Input Validation]
            IL2[Data Normalization]
            IL3[Schema Validation]
        end
        
        subgraph "Transformation Layer"
            TL1[Business Logic]
            TL2[State Calculation]
            TL3[Risk Assessment]
        end
        
        subgraph "Storage Layer"
            SL1[(Session DB)]
            SL2[(Event Store)]
            SL3[(Cache Layer)]
            SL4[(Archive)]
        end
    end
    
    subgraph "Data Distribution"
        subgraph "Real-time Updates"
            RT1[WebSocket Streams]
            RT2[Event Notifications]
            RT3[State Sync]
        end
        
        subgraph "Query Interface"
            QI1[GraphQL API]
            QI2[REST Endpoints]
            QI3[RPC Methods]
        end
    end
    
    subgraph "Data Consumers"
        DC1[Frontend UI]
        DC2[Mobile Apps]
        DC3[Analytics]
        DC4[Monitoring]
    end
    
    DS1 --> IL1
    DS2 --> IL1
    DS3 --> IL2
    DS4 --> IL2
    
    IL1 --> IL3
    IL2 --> IL3
    IL3 --> TL1
    
    TL1 --> TL2
    TL2 --> TL3
    
    TL3 --> SL1
    TL3 --> SL2
    TL2 --> SL3
    SL2 --> SL4
    
    SL1 --> RT1
    SL2 --> RT2
    SL3 --> RT3
    
    SL1 --> QI1
    SL2 --> QI2
    SL3 --> QI3
    
    RT1 --> DC1
    RT2 --> DC2
    QI1 --> DC3
    QI2 --> DC4
```

## Security Architecture Layers

### Defense-in-Depth Security Model

```mermaid
graph TB
    subgraph "Perimeter Security"
        PS1[DDoS Protection]
        PS2[WAF Rules]
        PS3[Rate Limiting]
        PS4[IP Filtering]
    end
    
    subgraph "Application Security"
        subgraph "Authentication Layer"
            AL1[Wallet Auth]
            AL2[Session Management]
            AL3[2FA/MFA]
        end
        
        subgraph "Authorization Layer"
            AZ1[Role-Based Access]
            AZ2[Permission Matrix]
            AZ3[Resource Guards]
        end
        
        subgraph "Validation Layer"
            VL1[Input Sanitization]
            VL2[Type Checking]
            VL3[Business Rules]
        end
    end
    
    subgraph "Protocol Security"
        subgraph "Cryptographic Layer"
            CL1[SHA-256 Hashlocks]
            CL2[Digital Signatures]
            CL3[Key Management]
        end
        
        subgraph "Smart Contract Security"
            SC1[Access Control]
            SC2[Reentrancy Guards]
            SC3[Integer Safety]
            SC4[State Machine]
        end
        
        subgraph "Cross-Chain Security"
            CC1[Timeout Coordination]
            CC2[Event Verification]
            CC3[Replay Protection]
        end
    end
    
    subgraph "Infrastructure Security"
        IS1[TEE Isolation]
        IS2[Network Segmentation]
        IS3[Encrypted Storage]
        IS4[Secure Communication]
    end
    
    PS1 --> PS2 --> PS3 --> PS4
    PS4 --> AL1
    
    AL1 --> AL2 --> AL3
    AL3 --> AZ1
    AZ1 --> AZ2 --> AZ3
    AZ3 --> VL1
    VL1 --> VL2 --> VL3
    
    VL3 --> CL1
    CL1 --> CL2 --> CL3
    
    CL3 --> SC1
    SC1 --> SC2 --> SC3 --> SC4
    
    SC4 --> CC1
    CC1 --> CC2 --> CC3
    
    CC3 --> IS1
    IS1 --> IS2 --> IS3 --> IS4
    
    style PS1 fill:#f44336
    style CL1 fill:#2196f3
    style SC1 fill:#4caf50
    style IS1 fill:#ff9800
```

## State Management Architecture

### Distributed State Synchronization

```mermaid
graph LR
    subgraph "State Sources"
        SS1[User State]
        SS2[Chain State]
        SS3[Session State]
        SS4[System State]
    end
    
    subgraph "State Management Core"
        subgraph "State Store"
            ST1[In-Memory Cache]
            ST2[Redis State]
            ST3[Database State]
            ST4[Chain State]
        end
        
        subgraph "State Synchronizer"
            SY1[Event Listener]
            SY2[State Merger]
            SY3[Conflict Resolver]
            SY4[State Publisher]
        end
        
        subgraph "State Validators"
            SV1[Schema Validator]
            SV2[Business Rules]
            SV3[Consistency Check]
            SV4[Integrity Verify]
        end
    end
    
    subgraph "State Consumers"
        SC1[Frontend State]
        SC2[API State]
        SC3[Worker State]
        SC4[Monitor State]
    end
    
    SS1 --> SY1
    SS2 --> SY1
    SS3 --> SY1
    SS4 --> SY1
    
    SY1 --> SY2
    SY2 --> SY3
    SY3 --> SV1
    
    SV1 --> SV2
    SV2 --> SV3
    SV3 --> SV4
    
    SV4 --> ST1
    ST1 --> ST2
    ST2 --> ST3
    ST3 --> ST4
    
    SY4 --> SC1
    SY4 --> SC2
    SY4 --> SC3
    SY4 --> SC4
    
    ST1 --> SY4
```

## Integration Architecture

### Service Integration Map

```mermaid
graph TB
    subgraph "1Balancer Core"
        CORE[Core Services]
        ORCH[Orchestration]
        API[API Gateway]
    end
    
    subgraph "Blockchain Integrations"
        subgraph "EVM Chains"
            ETH[Ethereum]
            BASE[BASE L2]
            POLY[Polygon]
        end
        
        subgraph "Non-EVM Chains"
            NEAR[NEAR Protocol]
            SUI[Sui Network]
            APTOS[Aptos]
        end
    end
    
    subgraph "DeFi Integrations"
        INCH[1inch Protocol]
        UNI[Uniswap]
        CURVE[Curve]
    end
    
    subgraph "Infrastructure"
        IPFS[IPFS Storage]
        GRAPH[The Graph]
        CHAIN[Chainlink]
    end
    
    subgraph "External Services"
        AUTH[Auth Providers]
        ANALYTICS[Analytics]
        MONITOR[Monitoring]
    end
    
    CORE --> ORCH
    ORCH --> API
    
    API --> ETH
    API --> BASE
    API --> POLY
    API --> NEAR
    API --> SUI
    API --> APTOS
    
    ORCH --> INCH
    ORCH --> UNI
    ORCH --> CURVE
    
    CORE --> IPFS
    CORE --> GRAPH
    CORE --> CHAIN
    
    API --> AUTH
    API --> ANALYTICS
    API --> MONITOR
    
    style NEAR fill:#ff9800
    style INCH fill:#2196f3
```

## Performance Architecture

### Optimization Layers

```mermaid
graph TB
    subgraph "Request Layer"
        RL1[Load Balancer]
        RL2[Request Router]
        RL3[Priority Queue]
    end
    
    subgraph "Caching Strategy"
        CS1[CDN Cache]
        CS2[API Cache]
        CS3[Redis Cache]
        CS4[Local Cache]
    end
    
    subgraph "Processing Optimization"
        PO1[Parallel Processing]
        PO2[Batch Operations]
        PO3[Async Execution]
        PO4[Resource Pooling]
    end
    
    subgraph "Data Optimization"
        DO1[Query Optimization]
        DO2[Index Strategy]
        DO3[Data Compression]
        DO4[Lazy Loading]
    end
    
    subgraph "Network Optimization"
        NO1[Connection Pooling]
        NO2[Protocol Buffers]
        NO3[WebSocket Reuse]
        NO4[Multi-Region]
    end
    
    RL1 --> RL2 --> RL3
    RL3 --> CS1
    
    CS1 --> CS2 --> CS3 --> CS4
    
    CS4 --> PO1
    PO1 --> PO2 --> PO3 --> PO4
    
    PO4 --> DO1
    DO1 --> DO2 --> DO3 --> DO4
    
    DO4 --> NO1
    NO1 --> NO2 --> NO3 --> NO4
```

## Deployment Architecture

### Multi-Environment Deployment

```mermaid
graph TB
    subgraph "Development"
        DEV1[Local NEAR]
        DEV2[Local Hardhat]
        DEV3[Mock Services]
    end
    
    subgraph "Staging"
        STG1[NEAR Testnet]
        STG2[BASE Testnet]
        STG3[Test Services]
    end
    
    subgraph "Production"
        subgraph "NEAR Mainnet"
            NM1[HTLC Contract]
            NM2[Solver Registry]
            NM3[TEE Solver]
        end
        
        subgraph "BASE Mainnet"
            BM1[Ethereum Hub]
            BM2[Escrow Factory]
            BM3[Resolver]
        end
        
        subgraph "Infrastructure"
            INF1[Orchestrator]
            INF2[Monitoring]
            INF3[Analytics]
        end
    end
    
    subgraph "CI/CD Pipeline"
        CI1[Code Push]
        CI2[Build & Test]
        CI3[Security Scan]
        CI4[Deploy]
    end
    
    DEV1 --> STG1
    DEV2 --> STG2
    DEV3 --> STG3
    
    STG1 --> NM1
    STG2 --> BM1
    STG3 --> INF1
    
    CI1 --> CI2 --> CI3 --> CI4
    CI4 --> DEV1
    CI4 --> STG1
    CI4 --> NM1
```

## Conclusion

This architecture provides:

1. **Comprehensive Integration**: All components work together seamlessly
2. **Security at Every Layer**: Defense in depth approach
3. **Scalable Design**: Can handle growth and additional chains
4. **Clear Separation**: Each component has defined responsibilities
5. **Event-Driven**: Reactive architecture for real-time operations
6. **Performance Optimized**: Multiple optimization strategies

The NEAR integration extends the Ethereum Hub architecture to support non-EVM chains while maintaining the same security guarantees and architectural principles.