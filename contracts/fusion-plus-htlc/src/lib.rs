use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, NearToken};
use sha2::{Digest, Sha256};

type Balance = u128;
type Timestamp = u64;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FusionPlusHTLC {
    owner: AccountId,
    htlcs: UnorderedMap<String, HTLC>,
    active_htlc_ids: Vec<String>,
    next_htlc_id: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct HTLC {
    pub sender: AccountId,
    pub receiver: AccountId,
    pub token: Option<AccountId>, // None for NEAR native token
    pub amount: Balance,
    pub hashlock: Base64VecU8, // SHA-256 hash
    pub timelock: Timestamp,
    pub order_hash: Base64VecU8,
    pub withdrawn: bool,
    pub refunded: bool,
    pub created_at: Timestamp,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    HTLCs,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HTLCView {
    pub htlc_id: String,
    pub sender: AccountId,
    pub receiver: AccountId,
    pub token: Option<AccountId>,
    pub amount: U128,
    pub hashlock: Base64VecU8,
    pub timelock: U128,
    pub order_hash: Base64VecU8,
    pub withdrawn: bool,
    pub refunded: bool,
    pub created_at: U128,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EventLog {
    pub event_type: String,
    pub htlc_id: String,
    pub sender: Option<AccountId>,
    pub receiver: Option<AccountId>,
    pub secret: Option<Base64VecU8>,
    pub amount: Option<U128>,
    pub hashlock: Option<Base64VecU8>,
    pub timelock: Option<U128>,
    pub timestamp: U128,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CreateHTLCArgs {
    pub receiver: AccountId,
    pub token: Option<AccountId>,
    pub amount: U128,
    pub hashlock: Base64VecU8,
    pub timelock: U128,
    pub order_hash: Base64VecU8,
}

#[near_bindgen]
impl FusionPlusHTLC {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner,
            htlcs: UnorderedMap::new(StorageKey::HTLCs),
            active_htlc_ids: Vec::new(),
            next_htlc_id: 1,
        }
    }

    // Create HTLC
    #[payable]
    pub fn create_htlc(&mut self, args: CreateHTLCArgs) -> String {
        let htlc_id = format!("htlc_{}", self.next_htlc_id);
        self.next_htlc_id += 1;

        let amount: Balance = args.amount.0;
        let timelock: Timestamp = args.timelock.0 as u64;

        // Validate inputs
        assert!(amount > 0, "Amount must be positive");
        assert!(timelock > env::block_timestamp(), "Timelock must be in future");
        assert!(args.hashlock.0.len() == 32, "Hashlock must be 32 bytes (SHA-256)");

        // Handle native NEAR token
        if args.token.is_none() {
            assert!(
                env::attached_deposit() >= NearToken::from_yoctonear(amount),
                "Attached deposit must match amount for NEAR"
            );
        } else {
            // For NEP-141 tokens, implement transfer_from logic here
            // For now, we'll focus on native NEAR
            assert!(args.token.is_none(), "NEP-141 tokens not yet implemented");
        }

        let htlc = HTLC {
            sender: env::predecessor_account_id(),
            receiver: args.receiver.clone(),
            token: args.token,
            amount,
            hashlock: args.hashlock.clone(),
            timelock,
            order_hash: args.order_hash,
            withdrawn: false,
            refunded: false,
            created_at: env::block_timestamp(),
        };

        self.htlcs.insert(&htlc_id, &htlc);
        self.active_htlc_ids.push(htlc_id.clone());

        // Emit event
        self.emit_event(EventLog {
            event_type: "htlc_created".to_string(),
            htlc_id: htlc_id.clone(),
            sender: Some(htlc.sender.clone()),
            receiver: Some(htlc.receiver.clone()),
            secret: None,
            amount: Some(U128(amount)),
            hashlock: Some(args.hashlock),
            timelock: Some(U128(timelock as u128)),
            timestamp: U128(env::block_timestamp() as u128),
        });

        htlc_id
    }

    // Withdraw with secret
    pub fn withdraw(&mut self, htlc_id: String, secret: Base64VecU8) {
        let htlc = self.htlcs.get(&htlc_id)
            .expect("HTLC does not exist");

        assert!(!htlc.withdrawn, "Already withdrawn");
        assert!(!htlc.refunded, "Already refunded");
        assert!(
            env::predecessor_account_id() == htlc.receiver,
            "Only receiver can withdraw"
        );

        // Verify secret
        let mut hasher = Sha256::new();
        hasher.update(&secret.0);
        let hash = hasher.finalize();
        
        assert!(
            hash.as_slice() == htlc.hashlock.0.as_slice(),
            "Invalid secret"
        );

        // Extract values before modifying
        let receiver = htlc.receiver.clone();
        let sender = htlc.sender.clone();
        let amount = htlc.amount;
        let token = htlc.token.clone();

        // Clone htlc and mark as withdrawn
        let mut htlc_updated = htlc.clone();
        htlc_updated.withdrawn = true;
        self.htlcs.insert(&htlc_id, &htlc_updated);

        // Transfer funds
        if token.is_none() {
            // Transfer NEAR
            Promise::new(receiver.clone()).transfer(NearToken::from_yoctonear(amount));
        } else {
            // Handle NEP-141 token transfer
            panic!("NEP-141 tokens not yet implemented");
        }

        // Emit events
        self.emit_event(EventLog {
            event_type: "secret_revealed".to_string(),
            htlc_id: htlc_id.clone(),
            sender: None,
            receiver: None,
            secret: Some(secret),
            amount: None,
            hashlock: None,
            timelock: None,
            timestamp: U128(env::block_timestamp() as u128),
        });

        self.emit_event(EventLog {
            event_type: "htlc_withdrawn".to_string(),
            htlc_id: htlc_id.clone(),
            sender: Some(sender.clone()),
            receiver: Some(receiver.clone()),
            secret: None,
            amount: Some(U128(amount)),
            hashlock: None,
            timelock: None,
            timestamp: U128(env::block_timestamp() as u128),
        });

        // Remove from active list
        self.active_htlc_ids.retain(|id| id != &htlc_id);
    }

    // Refund after timeout
    pub fn refund(&mut self, htlc_id: String) {
        let htlc = self.htlcs.get(&htlc_id)
            .expect("HTLC does not exist");

        assert!(!htlc.withdrawn, "Already withdrawn");
        assert!(!htlc.refunded, "Already refunded");
        assert!(
            env::predecessor_account_id() == htlc.sender,
            "Only sender can refund"
        );
        assert!(
            env::block_timestamp() >= htlc.timelock,
            "Timelock not expired"
        );

        // Extract all values before modifying
        let sender = htlc.sender.clone();
        let receiver = htlc.receiver.clone();
        let amount = htlc.amount;
        let token = htlc.token.clone();

        // Clone htlc and mark as refunded
        let mut htlc_updated = htlc.clone();
        htlc_updated.refunded = true;
        self.htlcs.insert(&htlc_id, &htlc_updated);

        // Transfer funds back
        if token.is_none() {
            // Transfer NEAR
            Promise::new(sender.clone()).transfer(NearToken::from_yoctonear(amount));
        } else {
            // Handle NEP-141 token transfer
            panic!("NEP-141 tokens not yet implemented");
        }

        // Emit event
        self.emit_event(EventLog {
            event_type: "htlc_refunded".to_string(),
            htlc_id: htlc_id.clone(),
            sender: Some(sender.clone()),
            receiver: Some(receiver.clone()),
            secret: None,
            amount: Some(U128(amount)),
            hashlock: None,
            timelock: None,
            timestamp: U128(env::block_timestamp() as u128),
        });

        // Remove from active list
        self.active_htlc_ids.retain(|id| id != &htlc_id);
    }

    // View methods
    pub fn get_htlc(&self, htlc_id: String) -> Option<HTLCView> {
        self.htlcs.get(&htlc_id).map(|htlc| HTLCView {
            htlc_id,
            sender: htlc.sender.clone(),
            receiver: htlc.receiver.clone(),
            token: htlc.token.clone(),
            amount: U128(htlc.amount),
            hashlock: htlc.hashlock.clone(),
            timelock: U128(htlc.timelock as u128),
            order_hash: htlc.order_hash.clone(),
            withdrawn: htlc.withdrawn,
            refunded: htlc.refunded,
            created_at: U128(htlc.created_at as u128),
        })
    }

    pub fn get_active_htlcs(&self, from_index: u64, limit: u64) -> Vec<HTLCView> {
        let start = from_index as usize;
        let end = std::cmp::min(start + limit as usize, self.active_htlc_ids.len());
        
        self.active_htlc_ids[start..end]
            .iter()
            .filter_map(|id| self.get_htlc(id.clone()))
            .collect()
    }

    pub fn get_recent_events(&self, _from_timestamp: U128) -> Vec<EventLog> {
        // In a real implementation, we would store events
        // For now, return empty as events are emitted to logs
        vec![]
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn get_info(&self) -> String {
        format!(
            r#"{{"owner":"{}","version":"2.0.0","total_htlcs":{},"active_htlcs":{}}}"#,
            self.owner,
            self.htlcs.len(),
            self.active_htlc_ids.len()
        )
    }

    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"owner":"{}","version":"2.0.0","total_htlcs":{},"active_htlcs":{}}}"#,
            self.owner,
            self.htlcs.len(),
            self.active_htlc_ids.len()
        )
    }

    // Internal helpers
    fn emit_event(&self, event: EventLog) {
        env::log_str(&format!(
            "EVENT_JSON:{}",
            near_sdk::serde_json::to_string(&event).unwrap()
        ));
    }
}

// Tests module
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, VMContext};

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id)
            .build()
    }

    #[test]
    fn test_new() {
        let context = get_context(accounts(1));
        testing_env!(context);
        let contract = FusionPlusHTLC::new(accounts(1));
        assert_eq!(contract.get_owner(), accounts(1));
    }

    #[test]
    fn test_create_htlc() {
        let mut context = get_context(accounts(1));
        context.attached_deposit = 1_000_000_000_000_000_000_000_000; // 1 NEAR
        testing_env!(context);
        
        let mut contract = FusionPlusHTLC::new(accounts(0));
        
        let hashlock = Base64VecU8(vec![0u8; 32]); // Mock hashlock
        let args = CreateHTLCArgs {
            receiver: accounts(2),
            token: None,
            amount: U128(1_000_000_000_000_000_000_000_000),
            hashlock: hashlock.clone(),
            timelock: U128(env::block_timestamp() + 3600_000_000_000), // 1 hour
            order_hash: Base64VecU8(vec![1u8; 32]),
        };
        
        let htlc_id = contract.create_htlc(args);
        assert_eq!(htlc_id, "htlc_1");
        
        let htlc = contract.get_htlc(htlc_id).unwrap();
        assert_eq!(htlc.sender, accounts(1));
        assert_eq!(htlc.receiver, accounts(2));
        assert!(!htlc.withdrawn);
        assert!(!htlc.refunded);
    }
}