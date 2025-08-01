use near_sdk::{near, env, log, AccountId, Promise, NearToken, PromiseOrValue, PanicOnDefault};
use near_sdk::store::{LookupMap, UnorderedSet};
use near_sdk::json_types::U128;

mod types;
mod utils;

// Keep the module structure for types and events only
mod cross_chain {
    pub mod events;
}

use types::*;
use utils::{validate_hashlock, validate_timelock, generate_htlc_id, current_timestamp_sec, validate_secret};

// Type alias for backward compatibility
pub type Balance = u128;

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
        self.supported_tokens.insert(token);
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
        self.htlcs.get(&htlc_id).cloned()
    }
    
    /// Get all HTLCs for a sender
    pub fn get_sender_htlcs(&self, sender: AccountId) -> Vec<String> {
        self.active_htlcs.get(&sender)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get contract stats
    pub fn get_stats(&self) -> ContractStats {
        // Note: store module doesn't provide len() method
        // These would need to be tracked separately if exact counts are needed
        ContractStats {
            total_htlcs: 0, // Would need manual tracking
            active_htlcs: 0, // Would need manual tracking
            supported_tokens: 0, // Would need manual tracking
            eth_orchestrator: self.eth_orchestrator.clone(),
        }
    }

    // ===== CREATE HTLC METHODS =====
    
    /// Creates a new HTLC
    /// Called when NEP-141 tokens are transferred to this contract
    pub(crate) fn internal_create_htlc(&mut self, args: HTLCCreateArgs, sender: AccountId, amount: Balance) -> String {
        // Validate inputs
        assert!(validate_hashlock(&args.hashlock), "{}", ERR_INVALID_HASHLOCK);
        assert!(validate_timelock(args.timelock), "{}", ERR_INVALID_TIMELOCK);
        assert_eq!(amount, args.amount, "{}", ERR_INSUFFICIENT_DEPOSIT);
        assert_eq!(args.token, env::predecessor_account_id(), "Token mismatch");

        // Generate HTLC ID
        let htlc_id = generate_htlc_id(
            sender.as_str(),
            args.receiver.as_str(),
            env::block_timestamp()
        );

        // Create HTLC
        let htlc = HTLC {
            sender: sender.clone(),
            receiver: args.receiver.clone(),
            token: args.token.clone(),
            amount: args.amount,
            hashlock: args.hashlock.clone(),
            timelock: args.timelock,
            secret: None,
            state: HTLCState::Active,
            created_at: current_timestamp_sec(),
            order_hash: args.order_hash.clone(),
        };

        // Store HTLC
        self.htlcs.insert(htlc_id.clone(), htlc.clone());

        // Add to active HTLCs
        if let Some(mut active_htlcs) = self.active_htlcs.get_mut(&sender) {
            active_htlcs.insert(htlc_id.clone());
        } else {
            let mut active_htlcs = near_sdk::store::UnorderedSet::new(
                format!("active_htlcs_{}", sender).as_bytes()
            );
            active_htlcs.insert(htlc_id.clone());
            self.active_htlcs.insert(sender.clone(), active_htlcs);
        }

        // Emit event for cross-chain coordination
        let event = HTLCCreatedEvent {
            htlc_id: htlc_id.clone(),
            sender,
            receiver: args.receiver,
            token: args.token,
            amount: args.amount,
            hashlock: args.hashlock,
            timelock: args.timelock,
            order_hash: args.order_hash,
        };

        log!(
            "EVENT_JSON:{}",
            near_sdk::serde_json::to_string(&event).unwrap()
        );

        // Log for monitoring
        log!(
            "HTLC created: {} from {} to {} for {} tokens",
            htlc_id,
            htlc.sender,
            htlc.receiver,
            htlc.amount
        );

        htlc_id
    }

    /// Direct HTLC creation (for testing or direct calls)
    /// Requires attaching the tokens as deposit
    #[payable]
    pub fn create_htlc(&mut self, args: HTLCCreateArgs) -> String {
        // For native NEAR token HTLCs
        let deposit = env::attached_deposit();
        assert!(deposit.as_yoctonear() > 0, "Must attach deposit");

        // Use NEAR as token ID for native token
        let mut modified_args = args;
        modified_args.token = "near".parse::<AccountId>().unwrap();
        modified_args.amount = deposit.as_yoctonear();

        self.internal_create_htlc(
            modified_args,
            env::predecessor_account_id(),
            deposit.as_yoctonear()
        )
    }

    // ===== WITHDRAW HTLC METHODS =====
    
    /// Withdraw HTLC by revealing the secret
    /// Only the receiver can withdraw before the timelock expires
    pub fn withdraw(&mut self, htlc_id: String, secret: String) -> Promise {
        // Get HTLC
        let mut htlc = self.htlcs.get(&htlc_id)
            .cloned()
            .expect(ERR_HTLC_NOT_FOUND);
        
        // Validate state
        assert_eq!(htlc.state, HTLCState::Active, "{}", ERR_HTLC_NOT_ACTIVE);
        
        // Validate caller is receiver
        assert_eq!(
            env::predecessor_account_id(),
            htlc.receiver,
            "{}", ERR_UNAUTHORIZED
        );
        
        // Validate secret
        assert!(
            validate_secret(&secret, &htlc.hashlock),
            "{}", ERR_INVALID_SECRET
        );
        
        // Check timelock hasn't expired for destination chain
        // This is where we implement the cross-chain coordination
        let current_time = current_timestamp_sec();
        assert!(
            current_time < htlc.timelock,
            "Withdrawal period has expired"
        );
        
        // Update state
        htlc.state = HTLCState::Withdrawn;
        htlc.secret = Some(secret.clone());
        self.htlcs.insert(htlc_id.clone(), htlc.clone());
        
        // Remove from active HTLCs
        if let Some(mut active_htlcs) = self.active_htlcs.get_mut(&htlc.sender) {
            active_htlcs.remove(&htlc_id);
        }
        // Check if we should remove the empty set
        if let Some(active_htlcs) = self.active_htlcs.get(&htlc.sender) {
            if active_htlcs.iter().next().is_none() {
                self.active_htlcs.remove(&htlc.sender);
            }
        }
        
        // Emit event
        let event = HTLCWithdrawnEvent {
            htlc_id: htlc_id.clone(),
            receiver: htlc.receiver.clone(),
            secret: secret.clone(),
            amount: htlc.amount,
        };
        
        log!(
            "EVENT_JSON:{}",
            near_sdk::serde_json::to_string(&event).unwrap()
        );
        
        log!(
            "HTLC {} withdrawn by {} with secret {}",
            htlc_id,
            htlc.receiver,
            secret
        );
        
        // Transfer tokens
        if htlc.token.as_str() == "near" {
            // Native NEAR transfer
            Promise::new(htlc.receiver).transfer(NearToken::from_yoctonear(htlc.amount))
        } else {
            // NEP-141 token transfer
            let transfer_args = near_sdk::serde_json::json!({
                "receiver_id": htlc.receiver,
                "amount": htlc.amount.to_string(),
                "memo": format!("HTLC {} withdrawn", htlc_id)
            });
            
            Promise::new(htlc.token)
                .function_call(
                    "ft_transfer".to_string(),
                    transfer_args.to_string().into_bytes(),
                    NearToken::from_yoctonear(1), // 1 yoctoNEAR for storage
                    near_sdk::Gas::from_tgas(10)
                )
        }
    }
    
    /// Get the secret after withdrawal (for cross-chain coordination)
    pub fn get_secret(&self, htlc_id: String) -> Option<String> {
        self.htlcs.get(&htlc_id)
            .and_then(|htlc| htlc.secret.clone())
    }

    // ===== REFUND HTLC METHODS =====
    
    /// Refund HTLC after timelock expiry
    /// Only the sender can refund after the timelock expires
    pub fn refund(&mut self, htlc_id: String) -> Promise {
        // Get HTLC
        let mut htlc = self.htlcs.get(&htlc_id)
            .cloned()
            .expect(ERR_HTLC_NOT_FOUND);
        
        // Validate state
        assert_eq!(htlc.state, HTLCState::Active, "{}", ERR_HTLC_NOT_ACTIVE);
        
        // Validate caller is sender
        assert_eq!(
            env::predecessor_account_id(),
            htlc.sender,
            "{}", ERR_UNAUTHORIZED
        );
        
        // Check timelock has expired
        let current_time = current_timestamp_sec();
        assert!(
            current_time >= htlc.timelock,
            "{}", ERR_TIMELOCK_NOT_EXPIRED
        );
        
        // Update state
        htlc.state = HTLCState::Refunded;
        self.htlcs.insert(htlc_id.clone(), htlc.clone());
        
        // Remove from active HTLCs
        if let Some(mut active_htlcs) = self.active_htlcs.get_mut(&htlc.sender) {
            active_htlcs.remove(&htlc_id);
        }
        // Check if we should remove the empty set
        if let Some(active_htlcs) = self.active_htlcs.get(&htlc.sender) {
            if active_htlcs.iter().next().is_none() {
                self.active_htlcs.remove(&htlc.sender);
            }
        }
        
        // Emit event
        let event = HTLCRefundedEvent {
            htlc_id: htlc_id.clone(),
            sender: htlc.sender.clone(),
            amount: htlc.amount,
        };
        
        log!(
            "EVENT_JSON:{}",
            near_sdk::serde_json::to_string(&event).unwrap()
        );
        
        log!(
            "HTLC {} refunded to {}",
            htlc_id,
            htlc.sender
        );
        
        // Transfer tokens back to sender
        if htlc.token.as_str() == "near" {
            // Native NEAR transfer
            Promise::new(htlc.sender).transfer(NearToken::from_yoctonear(htlc.amount))
        } else {
            // NEP-141 token transfer
            let transfer_args = near_sdk::serde_json::json!({
                "receiver_id": htlc.sender,
                "amount": htlc.amount.to_string(),
                "memo": format!("HTLC {} refunded", htlc_id)
            });
            
            Promise::new(htlc.token)
                .function_call(
                    "ft_transfer".to_string(),
                    transfer_args.to_string().into_bytes(),
                    NearToken::from_yoctonear(1), // 1 yoctoNEAR for storage
                    near_sdk::Gas::from_tgas(10)
                )
        }
    }
    
    /// Cancel multiple HTLCs (batch operation)
    pub fn batch_refund(&mut self, htlc_ids: Vec<String>) -> Vec<Promise> {
        htlc_ids.into_iter()
            .map(|htlc_id| self.refund(htlc_id))
            .collect()
    }

    // ===== CROSS-CHAIN COORDINATION METHODS =====
    
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
        // Note: store module LookupMap doesn't provide iteration
        // This would need to be tracked separately with an index
        // For now, return empty vec
        Vec::new()
    }

    // ===== FT RECEIVER METHODS =====
    
    /// Called by NEP-141 token contracts when tokens are transferred to this contract
    /// The msg parameter should contain the HTLC creation parameters
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Parse the message to get HTLC parameters
        let args: HTLCCreateArgs = near_sdk::serde_json::from_str(&msg)
            .expect("Invalid HTLC parameters in msg");
        
        // The token contract is the predecessor
        let token_contract = near_sdk::env::predecessor_account_id();
        
        // Validate token matches
        assert_eq!(
            token_contract, 
            args.token,
            "Token contract mismatch"
        );
        
        // Create HTLC with the received tokens
        let htlc_id = self.internal_create_htlc(
            args,
            sender_id,
            amount.0
        );
        
        near_sdk::log!(
            "HTLC {} created via ft_on_transfer with {} tokens",
            htlc_id,
            amount.0
        );
        
        // Return 0 to indicate all tokens were used
        PromiseOrValue::Value(U128(0))
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

/// Helper trait for creating HTLCs with NEP-141 tokens
pub trait HTLCTokenReceiver {
    fn create_htlc_msg(
        receiver: AccountId,
        hashlock: String,
        timelock: u64,
        order_hash: Option<String>,
    ) -> String;
}

impl HTLCTokenReceiver for HTLCCreateArgs {
    /// Creates a properly formatted message for ft_transfer_call
    fn create_htlc_msg(
        receiver: AccountId,
        hashlock: String,
        timelock: u64,
        order_hash: Option<String>,
    ) -> String {
        let args = HTLCCreateArgs {
            receiver,
            token: "placeholder".parse::<AccountId>().unwrap(), // Will be replaced by actual token
            amount: 0, // Will be replaced by actual amount
            hashlock,
            timelock,
            order_hash,
        };
        
        near_sdk::serde_json::to_string(&args).unwrap()
    }
}