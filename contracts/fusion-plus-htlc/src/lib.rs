use near_sdk::{near, env, AccountId, Balance, Promise, PanicOnDefault};
use near_sdk::collections::{LookupMap, UnorderedSet};

mod types;
mod utils;
mod htlc;
mod cross_chain;
mod ft_receiver;

use types::*;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct FusionPlusContract {
    /// Contract owner
    owner: AccountId,
    /// All HTLCs by ID
    htlcs: LookupMap<String, HTLC>,
    /// Active HTLCs per sender for efficient queries
    active_htlcs: LookupMap<AccountId, UnorderedSet<String>>,
    /// Ethereum orchestrator address for cross-chain coordination
    eth_orchestrator: String,
    /// Supported tokens
    supported_tokens: UnorderedSet<AccountId>,
}

#[near]
impl FusionPlusContract {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self {
            owner,
            htlcs: LookupMap::new(b"h"),
            active_htlcs: LookupMap::new(b"a"),
            eth_orchestrator: String::new(),
            supported_tokens: UnorderedSet::new(b"t"),
        }
    }
    
    /// Get contract owner
    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }
    
    /// Assert caller is owner
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only owner can call this method"
        );
    }
    
    /// Add supported token
    pub fn add_supported_token(&mut self, token: AccountId) {
        self.assert_owner();
        self.supported_tokens.insert(&token);
    }
    
    /// Remove supported token
    pub fn remove_supported_token(&mut self, token: AccountId) {
        self.assert_owner();
        self.supported_tokens.remove(&token);
    }
    
    /// Check if token is supported
    pub fn is_token_supported(&self, token: &AccountId) -> bool {
        self.supported_tokens.contains(token)
    }
    
    /// Get HTLC by ID
    pub fn get_htlc(&self, htlc_id: String) -> Option<HTLC> {
        self.htlcs.get(&htlc_id)
    }
    
    /// Get all HTLCs for a sender
    pub fn get_sender_htlcs(&self, sender: AccountId) -> Vec<String> {
        self.active_htlcs.get(&sender)
            .map(|set| set.to_vec())
            .unwrap_or_default()
    }
    
    /// Get contract stats
    pub fn get_stats(&self) -> ContractStats {
        ContractStats {
            total_htlcs: self.htlcs.len(),
            active_htlcs: self.active_htlcs.len(),
            supported_tokens: self.supported_tokens.len(),
            eth_orchestrator: self.eth_orchestrator.clone(),
        }
    }
}

#[derive(near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractStats {
    pub total_htlcs: u64,
    pub active_htlcs: u64,
    pub supported_tokens: u64,
    pub eth_orchestrator: String,
}