use near_sdk::{env, log, near, Promise};
use crate::types::*;
use crate::utils::current_timestamp_sec;
use crate::FusionPlusContract;

#[near]
impl FusionPlusContract {
    /// Refund HTLC after timelock expiry
    /// Only the sender can refund after the timelock expires
    pub fn refund(&mut self, htlc_id: String) -> Promise {
        // Get HTLC
        let mut htlc = self.htlcs.get(&htlc_id).expect(ERR_HTLC_NOT_FOUND);
        
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
            Promise::new(htlc.sender).transfer(htlc.amount)
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
                    1, // 1 yoctoNEAR for storage
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
}