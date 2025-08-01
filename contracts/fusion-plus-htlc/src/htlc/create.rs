use near_sdk::{env, log, near, Promise, PromiseError, AccountId, Balance};
use crate::types::*;
use crate::utils::{validate_hashlock, validate_timelock, generate_htlc_id, current_timestamp_sec};
use crate::FusionPlusContract;

#[near]
impl FusionPlusContract {
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
        self.htlcs.insert(&htlc_id, &htlc);

        // Add to active HTLCs
        let mut active_htlcs = self.active_htlcs.get(&sender).unwrap_or_else(|| {
            near_sdk::collections::UnorderedSet::new(
                format!("active_htlcs_{}", sender).as_bytes()
            )
        });
        active_htlcs.insert(&htlc_id);
        self.active_htlcs.insert(&sender, &active_htlcs);

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
        assert!(deposit > 0, "Must attach deposit");

        // Use NEAR as token ID for native token
        let mut modified_args = args;
        modified_args.token = AccountId::new_unchecked("near".to_string());
        modified_args.amount = deposit;

        self.internal_create_htlc(
            modified_args,
            env::predecessor_account_id(),
            deposit
        )
    }
}