#[cfg(test)]
mod tests {
    use crate::{FusionPlusContract, HTLCCreateArgs, HTLCEvent, HTLC};
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, VMContext, NearToken, AccountId};
    use sha2::{Sha256, Digest};
    type Balance = u128;
    
    // Helper function to create testing context
    fn get_context(predecessor: AccountId, attached_deposit: Balance) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(accounts(0))
            .signer_account_id(predecessor.clone())
            .predecessor_account_id(predecessor)
            .attached_deposit(NearToken::from_yoctonear(attached_deposit))
            .block_timestamp(1_000_000_000 * 1_000_000_000) // 1 billion seconds in nanoseconds
            .build()
    }
    
    // Helper function to generate secret and hashlock
    fn generate_secret_pair() -> (String, String) {
        let secret = "my_secret_123";
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        let hashlock = format!("{:x}", hasher.finalize());
        (secret.to_string(), hashlock)
    }
    
    #[test]
    fn test_complete_atomic_swap_base_to_near() {
        println!("=== Testing Complete Atomic Swap: BASE → NEAR ===");
        
        // Setup accounts
        let owner = accounts(0);
        let alice_base = accounts(1); // Alice on BASE
        let bob_near = accounts(2);   // Bob on NEAR
        
        // Initialize contract
        testing_env!(get_context(owner.clone(), 0));
        let mut contract = FusionPlusContract::new(owner.clone());
        
        // Generate secret (Alice knows this)
        let (secret, hashlock) = generate_secret_pair();
        println!("1. Alice generates secret and hashlock");
        println!("   Secret: {}", secret);
        println!("   Hashlock: {}", hashlock);
        
        // Simulate: Alice locks USDC on BASE (orchestrator handles this)
        println!("\n2. Alice locks 100 USDC on BASE escrow");
        println!("   BASE escrow created with hashlock");
        
        // Bob creates HTLC on NEAR with NEAR tokens
        let htlc_amount = 10_000_000_000_000_000_000_000_000; // 10 NEAR
        let timelock = 2_000_000_000; // 2 billion seconds (future)
        
        testing_env!(get_context(bob_near.clone(), htlc_amount));
        println!("\n3. Bob creates HTLC on NEAR");
        let htlc_id = contract.create_htlc(HTLCCreateArgs {
            receiver: alice_base.clone(), // Alice will receive NEAR
            token: "near".to_string(),
            amount: htlc_amount,
            hashlock: hashlock.clone(),
            timelock,
            order_hash: "order_123".to_string(),
        });
        println!("   HTLC ID: {}", htlc_id);
        println!("   Amount: 10 NEAR locked");
        println!("   Receiver: Alice");
        
        // Verify HTLC was created
        let htlc = contract.get_htlc(htlc_id.clone()).unwrap();
        assert_eq!(htlc.sender, bob_near);
        assert_eq!(htlc.receiver, alice_base);
        assert_eq!(htlc.amount, htlc_amount);
        assert_eq!(htlc.hashlock, hashlock);
        println!("   ✅ HTLC verified on-chain");
        
        // Alice reveals secret to claim NEAR
        testing_env!(get_context(alice_base.clone(), 0));
        println!("\n4. Alice reveals secret to claim NEAR");
        contract.withdraw(htlc_id.clone(), secret.clone());
        println!("   ✅ Alice successfully claimed 10 NEAR");
        
        // Verify HTLC is withdrawn
        let htlc = contract.get_htlc(htlc_id.clone()).unwrap();
        assert!(htlc.withdrawn);
        assert_eq!(htlc.secret, Some(secret.clone()));
        
        // Simulate: Bob uses revealed secret to claim USDC on BASE
        println!("\n5. Bob sees revealed secret on NEAR");
        println!("   Secret from NEAR events: {}", secret);
        println!("   Bob claims 100 USDC on BASE using this secret");
        println!("   ✅ Atomic swap completed successfully!");
        
        // Check events
        let events = contract.get_recent_events(0);
        assert_eq!(events.len(), 2); // created + withdrawn
        assert_eq!(events[0].event_type, "created");
        assert_eq!(events[1].event_type, "withdrawn");
        assert_eq!(events[1].secret, Some(secret));
        println!("\n6. Events recorded for monitoring:");
        println!("   - HTLC created");
        println!("   - HTLC withdrawn with secret revealed");
    }
    
    #[test]
    fn test_complete_atomic_swap_near_to_base() {
        println!("=== Testing Complete Atomic Swap: NEAR → BASE ===");
        
        // Setup accounts
        let owner = accounts(0);
        let charlie_near = accounts(1); // Charlie on NEAR
        let dave_base = accounts(2);    // Dave on BASE
        
        // Initialize contract
        testing_env!(get_context(owner.clone(), 0));
        let mut contract = FusionPlusContract::new(owner.clone());
        
        // Generate secret (Dave knows this)
        let (secret, hashlock) = generate_secret_pair();
        println!("1. Dave generates secret and hashlock");
        println!("   Secret: {}", secret);
        println!("   Hashlock: {}", hashlock);
        
        // Charlie creates HTLC on NEAR
        let htlc_amount = 50_000_000_000_000_000_000_000_000; // 50 NEAR
        let timelock = 2_000_000_000;
        
        testing_env!(get_context(charlie_near.clone(), htlc_amount));
        println!("\n2. Charlie creates HTLC on NEAR");
        let htlc_id = contract.create_htlc(HTLCCreateArgs {
            receiver: dave_base.clone(), // Dave will receive NEAR
            token: "near".to_string(),
            amount: htlc_amount,
            hashlock: hashlock.clone(),
            timelock,
            order_hash: "order_456".to_string(),
        });
        println!("   HTLC ID: {}", htlc_id);
        println!("   Amount: 50 NEAR locked");
        println!("   Receiver: Dave");
        
        // Simulate: Dave locks USDC on BASE
        println!("\n3. Dave locks 500 USDC on BASE escrow");
        println!("   BASE escrow created with same hashlock");
        
        // Dave reveals secret to claim NEAR
        testing_env!(get_context(dave_base.clone(), 0));
        println!("\n4. Dave reveals secret to claim NEAR");
        contract.withdraw(htlc_id.clone(), secret.clone());
        println!("   ✅ Dave successfully claimed 50 NEAR");
        
        // Simulate: Charlie uses revealed secret to claim USDC
        println!("\n5. Charlie sees revealed secret on NEAR");
        println!("   Secret from NEAR events: {}", secret);
        println!("   Charlie claims 500 USDC on BASE using this secret");
        println!("   ✅ Atomic swap completed successfully!");
        
        // Verify final state
        let htlc = contract.get_htlc(htlc_id).unwrap();
        assert!(htlc.withdrawn);
        assert_eq!(htlc.secret, Some(secret));
    }
    
    #[test]
    fn test_refund_after_timeout() {
        println!("=== Testing Refund Mechanism After Timeout ===");
        
        let owner = accounts(0);
        let sender = accounts(1);
        let receiver = accounts(2);
        
        // Initialize contract
        testing_env!(get_context(owner.clone(), 0));
        let mut contract = FusionPlusContract::new(owner.clone());
        
        // Create HTLC with short timeout
        let (_, hashlock) = generate_secret_pair();
        let htlc_amount = 5_000_000_000_000_000_000_000_000; // 5 NEAR
        let timelock = 1_500_000_000; // 1.5 billion seconds (will expire)
        
        testing_env!(get_context(sender.clone(), htlc_amount));
        println!("1. Sender creates HTLC with 5 NEAR");
        let htlc_id = contract.create_htlc(HTLCCreateArgs {
            receiver: receiver.clone(),
            token: "near".to_string(),
            amount: htlc_amount,
            hashlock,
            timelock,
            order_hash: "order_timeout".to_string(),
        });
        println!("   HTLC ID: {}", htlc_id);
        println!("   Timelock: {} seconds", timelock);
        
        // Simulate time passing (move past timelock)
        let mut context = get_context(sender.clone(), 0);
        context.block_timestamp = 3_000_000_000 * 1_000_000_000; // 3 billion seconds
        testing_env!(context);
        
        println!("\n2. Time passes... Current time > Timelock");
        println!("   HTLC has expired without being claimed");
        
        // Sender refunds the HTLC
        println!("\n3. Sender initiates refund");
        contract.refund(htlc_id.clone());
        println!("   ✅ Refund successful - 5 NEAR returned to sender");
        
        // Verify HTLC is refunded
        let htlc = contract.get_htlc(htlc_id.clone()).unwrap();
        assert!(htlc.refunded);
        assert!(!htlc.withdrawn);
        println!("\n4. HTLC state verified:");
        println!("   - Refunded: true");
        println!("   - Withdrawn: false");
        
        // Check refund event
        let events = contract.get_recent_events(0);
        let refund_event = events.iter().find(|e| e.event_type == "refunded").unwrap();
        assert_eq!(refund_event.htlc_id, htlc_id);
        println!("\n5. Refund event recorded for monitoring");
    }
    
    #[test]
    fn test_cannot_withdraw_after_timeout() {
        println!("=== Testing Cannot Withdraw After Timeout ===");
        
        let owner = accounts(0);
        let sender = accounts(1);
        let receiver = accounts(2);
        
        testing_env!(get_context(owner.clone(), 0));
        let mut contract = FusionPlusContract::new(owner);
        
        // Create HTLC
        let (secret, hashlock) = generate_secret_pair();
        let htlc_amount = 1_000_000_000_000_000_000_000_000; // 1 NEAR
        let timelock = 1_500_000_000;
        
        testing_env!(get_context(sender.clone(), htlc_amount));
        let htlc_id = contract.create_htlc(HTLCCreateArgs {
            receiver: receiver.clone(),
            token: "near".to_string(),
            amount: htlc_amount,
            hashlock,
            timelock,
            order_hash: "order_late".to_string(),
        });
        
        // Move past timelock
        let mut context = get_context(receiver.clone(), 0);
        context.block_timestamp = 3_000_000_000 * 1_000_000_000;
        testing_env!(context);
        
        // Try to withdraw after timeout - should fail
        println!("Attempting to withdraw after timeout...");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract.withdraw(htlc_id, secret);
        }));
        
        assert!(result.is_err());
        println!("✅ Withdrawal correctly rejected after timeout");
    }
    
    #[test]
    fn test_cannot_refund_before_timeout() {
        println!("=== Testing Cannot Refund Before Timeout ===");
        
        let owner = accounts(0);
        let sender = accounts(1);
        let receiver = accounts(2);
        
        testing_env!(get_context(owner.clone(), 0));
        let mut contract = FusionPlusContract::new(owner);
        
        // Create HTLC with future timeout
        let (_, hashlock) = generate_secret_pair();
        let htlc_amount = 1_000_000_000_000_000_000_000_000; // 1 NEAR
        let timelock = 2_000_000_000; // Future
        
        testing_env!(get_context(sender.clone(), htlc_amount));
        let htlc_id = contract.create_htlc(HTLCCreateArgs {
            receiver,
            token: "near".to_string(),
            amount: htlc_amount,
            hashlock,
            timelock,
            order_hash: "order_early".to_string(),
        });
        
        // Try to refund before timeout - should fail
        println!("Attempting to refund before timeout...");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract.refund(htlc_id);
        }));
        
        assert!(result.is_err());
        println!("✅ Refund correctly rejected before timeout");
    }
    
    #[test]
    fn test_active_htlcs_tracking() {
        println!("=== Testing Active HTLCs Tracking ===");
        
        let owner = accounts(0);
        let sender = accounts(1);
        let receiver = accounts(2);
        
        testing_env!(get_context(owner.clone(), 0));
        let mut contract = FusionPlusContract::new(owner);
        
        // Initially no active HTLCs
        let active = contract.get_active_htlcs(0, 10);
        assert_eq!(active.len(), 0);
        println!("Initial active HTLCs: 0");
        
        // Create multiple HTLCs
        let (_, hashlock) = generate_secret_pair();
        let htlc_amount = 1_000_000_000_000_000_000_000_000; // 1 NEAR
        
        testing_env!(get_context(sender.clone(), htlc_amount * 3));
        
        // Create 3 HTLCs
        let mut htlc_ids = vec![];
        for i in 0..3 {
            let htlc_id = contract.create_htlc(HTLCCreateArgs {
                receiver: receiver.clone(),
                token: "near".to_string(),
                amount: htlc_amount,
                hashlock: hashlock.clone(),
                timelock: 2_000_000_000,
                order_hash: format!("order_{}", i),
            });
            htlc_ids.push(htlc_id);
        }
        
        // Check active HTLCs
        let active = contract.get_active_htlcs(0, 10);
        assert_eq!(active.len(), 3);
        println!("Active HTLCs after creation: 3");
        
        // Withdraw one
        let (secret, _) = generate_secret_pair();
        testing_env!(get_context(receiver.clone(), 0));
        contract.withdraw(htlc_ids[0].clone(), secret);
        
        // Check active HTLCs reduced
        let active = contract.get_active_htlcs(0, 10);
        assert_eq!(active.len(), 2);
        println!("Active HTLCs after withdrawal: 2");
        
        // Verify contract info
        let info = contract.get_info();
        assert!(info.contains("Active HTLCs: 2"));
        println!("Contract info: {}", info);
    }
}