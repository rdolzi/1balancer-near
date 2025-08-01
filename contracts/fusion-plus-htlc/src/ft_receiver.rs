use near_sdk::{near, AccountId, Balance, PromiseOrValue};
use near_sdk::json_types::U128;
use crate::types::*;
use crate::FusionPlusContract;

/// NEP-141 FungibleTokenReceiver trait implementation
/// This allows the contract to receive tokens and create HTLCs atomically
#[near]
impl FusionPlusContract {
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
            token: AccountId::new_unchecked("placeholder".to_string()), // Will be replaced by actual token
            amount: 0, // Will be replaced by actual amount
            hashlock,
            timelock,
            order_hash,
        };
        
        near_sdk::serde_json::to_string(&args).unwrap()
    }
}