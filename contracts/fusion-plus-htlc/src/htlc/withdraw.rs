use near_sdk::{env, log, near, Promise, AccountId};
use crate::types::*;
use crate::utils::{validate_secret, current_timestamp_sec};
use crate::FusionPlusContract;

#[near]
impl FusionPlusContract {
    /// Withdraw HTLC by revealing the secret
    /// Only the receiver can withdraw before the timelock expires
    pub fn withdraw(&mut self, htlc_id: String, secret: String) -> Promise {
        // Get HTLC
        let mut htlc = self.htlcs.get(&htlc_id).expect(ERR_HTLC_NOT_FOUND);
        
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
        self.htlcs.insert(&htlc_id, &htlc);
        
        // Remove from active HTLCs
        if let Some(mut active_htlcs) = self.active_htlcs.get(&htlc.sender) {
            active_htlcs.remove(&htlc_id);
            if active_htlcs.is_empty() {
                self.active_htlcs.remove(&htlc.sender);
            } else {
                self.active_htlcs.insert(&htlc.sender, &active_htlcs);
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
            Promise::new(htlc.receiver).transfer(htlc.amount)
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
                    1, // 1 yoctoNEAR for storage
                    near_sdk::Gas::from_tgas(10)
                )
        }
    }
    
    /// Get the secret after withdrawal (for cross-chain coordination)
    pub fn get_secret(&self, htlc_id: String) -> Option<String> {
        self.htlcs.get(&htlc_id)
            .and_then(|htlc| htlc.secret.clone())
    }
}