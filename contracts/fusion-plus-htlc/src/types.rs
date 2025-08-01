use near_sdk::{AccountId, Balance, Timestamp};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

/// HTLC State
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum HTLCState {
    Active,
    Withdrawn,
    Refunded,
    Expired,
}

/// Core HTLC structure that mirrors Ethereum Hub's Immutables
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HTLC {
    /// The sender (maker in cross-chain terminology)
    pub sender: AccountId,
    /// The receiver (taker in cross-chain terminology)
    pub receiver: AccountId,
    /// The token contract (NEP-141 token)
    pub token: AccountId,
    /// Amount locked in the HTLC
    pub amount: Balance,
    /// SHA-256 hashlock (32 bytes)
    pub hashlock: String,
    /// Timelock as Unix timestamp
    pub timelock: Timestamp,
    /// Secret (revealed when withdrawing)
    pub secret: Option<String>,
    /// Current state of the HTLC
    pub state: HTLCState,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Linked order hash from Ethereum Hub
    pub order_hash: Option<String>,
}

/// Cross-chain timelock structure matching TimelocksLib.sol
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CrossChainTimelocks {
    /// Source chain withdrawal time (BASE)
    pub src_withdrawal: u64,
    /// Source chain cancellation time (BASE)
    pub src_cancellation: u64,
    /// Destination chain withdrawal time (NEAR)
    pub dst_withdrawal: u64,
    /// Destination chain cancellation time (NEAR)
    pub dst_cancellation: u64,
}

/// HTLC creation parameters
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HTLCCreateArgs {
    pub receiver: AccountId,
    pub token: AccountId,
    pub amount: Balance,
    pub hashlock: String,
    pub timelock: Timestamp,
    pub order_hash: Option<String>,
}

/// Event structures for cross-chain coordination
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HTLCCreatedEvent {
    pub htlc_id: String,
    pub sender: AccountId,
    pub receiver: AccountId,
    pub token: AccountId,
    pub amount: Balance,
    pub hashlock: String,
    pub timelock: Timestamp,
    pub order_hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HTLCWithdrawnEvent {
    pub htlc_id: String,
    pub receiver: AccountId,
    pub secret: String,
    pub amount: Balance,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HTLCRefundedEvent {
    pub htlc_id: String,
    pub sender: AccountId,
    pub amount: Balance,
}

/// Error messages
pub const ERR_HTLC_NOT_FOUND: &str = "HTLC not found";
pub const ERR_HTLC_NOT_ACTIVE: &str = "HTLC is not active";
pub const ERR_INVALID_SECRET: &str = "Invalid secret";
pub const ERR_TIMELOCK_NOT_EXPIRED: &str = "Timelock has not expired";
pub const ERR_UNAUTHORIZED: &str = "Unauthorized";
pub const ERR_INVALID_HASHLOCK: &str = "Invalid hashlock format";
pub const ERR_INVALID_TIMELOCK: &str = "Invalid timelock";
pub const ERR_INSUFFICIENT_DEPOSIT: &str = "Insufficient deposit";