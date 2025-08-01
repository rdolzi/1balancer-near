use near_sdk::{near, AccountId};
use crate::types::*;
use crate::FusionPlusContract;

#[near]
impl FusionPlusContract {
    /// Set the Ethereum orchestrator address for cross-chain coordination
    pub fn set_eth_orchestrator(&mut self, orchestrator: String) {
        self.assert_owner();
        self.eth_orchestrator = orchestrator;
    }
    
    /// Get cross-chain coordination info for an HTLC
    pub fn get_cross_chain_info(&self, htlc_id: String) -> Option<CrossChainInfo> {
        self.htlcs.get(&htlc_id).map(|htlc| CrossChainInfo {
            htlc_id,
            order_hash: htlc.order_hash.clone(),
            hashlock: htlc.hashlock.clone(),
            state: htlc.state.clone(),
            secret: htlc.secret.clone(),
        })
    }
    
    /// Validate cross-chain timelock coordination
    /// NEAR timelock must be shorter than BASE timelock for atomicity
    pub fn validate_cross_chain_timelock(&self, near_timelock: u64, base_timelock: u64) -> bool {
        // NEAR cancellation must happen before BASE withdrawal
        // This ensures atomicity - if NEAR fails, BASE can be safely refunded
        near_timelock < base_timelock
    }
    
    /// Get all active HTLCs for monitoring
    pub fn get_active_htlcs(&self, from_index: u64, limit: u64) -> Vec<HTLCInfo> {
        let mut result = Vec::new();
        let mut count = 0u64;
        
        for (htlc_id, htlc) in self.htlcs.iter() {
            if htlc.state == HTLCState::Active {
                if count >= from_index {
                    result.push(HTLCInfo {
                        htlc_id,
                        sender: htlc.sender.clone(),
                        receiver: htlc.receiver.clone(),
                        amount: htlc.amount,
                        hashlock: htlc.hashlock.clone(),
                        timelock: htlc.timelock,
                        state: htlc.state.clone(),
                        order_hash: htlc.order_hash.clone(),
                    });
                    
                    if result.len() as u64 >= limit {
                        break;
                    }
                }
                count += 1;
            }
        }
        
        result
    }
}

/// Cross-chain coordination info
#[derive(near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CrossChainInfo {
    pub htlc_id: String,
    pub order_hash: Option<String>,
    pub hashlock: String,
    pub state: HTLCState,
    pub secret: Option<String>,
}

/// HTLC info for monitoring
#[derive(near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HTLCInfo {
    pub htlc_id: String,
    pub sender: AccountId,
    pub receiver: AccountId,
    pub amount: Balance,
    pub hashlock: String,
    pub timelock: u64,
    pub state: HTLCState,
    pub order_hash: Option<String>,
}