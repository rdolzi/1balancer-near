use near_sdk::test_utils::{accounts, VMContextBuilder};
use near_sdk::{testing_env, AccountId, Balance};
use fusion_plus_htlc::{FusionPlusContract, HTLCCreateArgs};

fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(accounts(0))
        .signer_account_id(predecessor_account_id.clone())
        .predecessor_account_id(predecessor_account_id);
    builder
}

#[test]
fn test_create_htlc_native() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    
    let mut contract = FusionPlusContract::new(accounts(1));
    
    // Create HTLC with native NEAR
    let args = HTLCCreateArgs {
        receiver: accounts(2),
        token: AccountId::new_unchecked("near".to_string()),
        amount: 1_000_000_000_000_000_000_000_000, // 1 NEAR
        hashlock: "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b".to_string(),
        timelock: 1_000_000_000 + 3600, // 1 hour from now
        order_hash: Some("0x123".to_string()),
    };
    
    // Attach deposit
    testing_env!(context
        .attached_deposit(1_000_000_000_000_000_000_000_000)
        .build());
    
    let htlc_id = contract.create_htlc(args);
    
    // Verify HTLC was created
    let htlc = contract.get_htlc(htlc_id.clone()).unwrap();
    assert_eq!(htlc.sender, accounts(1));
    assert_eq!(htlc.receiver, accounts(2));
    assert_eq!(htlc.amount, 1_000_000_000_000_000_000_000_000);
}

#[test]
fn test_withdraw_with_secret() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    
    let mut contract = FusionPlusContract::new(accounts(1));
    
    // Create HTLC
    let args = HTLCCreateArgs {
        receiver: accounts(2),
        token: AccountId::new_unchecked("near".to_string()),
        amount: 1_000_000_000_000_000_000_000_000,
        hashlock: "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b".to_string(),
        timelock: env::block_timestamp() / 1_000_000_000 + 3600,
        order_hash: None,
    };
    
    testing_env!(context
        .attached_deposit(1_000_000_000_000_000_000_000_000)
        .build());
    
    let htlc_id = contract.create_htlc(args);
    
    // Switch to receiver account
    testing_env!(context
        .predecessor_account_id(accounts(2))
        .attached_deposit(0)
        .build());
    
    // Withdraw with correct secret
    contract.withdraw(htlc_id.clone(), "mysecret".to_string());
    
    // Verify HTLC is withdrawn
    let htlc = contract.get_htlc(htlc_id).unwrap();
    assert_eq!(htlc.state, fusion_plus_htlc::types::HTLCState::Withdrawn);
    assert_eq!(htlc.secret, Some("mysecret".to_string()));
}

#[test]
#[should_panic(expected = "Invalid secret")]
fn test_withdraw_with_wrong_secret() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    
    let mut contract = FusionPlusContract::new(accounts(1));
    
    // Create HTLC
    let args = HTLCCreateArgs {
        receiver: accounts(2),
        token: AccountId::new_unchecked("near".to_string()),
        amount: 1_000_000_000_000_000_000_000_000,
        hashlock: "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b".to_string(),
        timelock: env::block_timestamp() / 1_000_000_000 + 3600,
        order_hash: None,
    };
    
    testing_env!(context
        .attached_deposit(1_000_000_000_000_000_000_000_000)
        .build());
    
    let htlc_id = contract.create_htlc(args);
    
    // Switch to receiver account
    testing_env!(context
        .predecessor_account_id(accounts(2))
        .attached_deposit(0)
        .build());
    
    // Try to withdraw with wrong secret
    contract.withdraw(htlc_id, "wrongsecret".to_string());
}

#[test]
fn test_refund_after_timeout() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    
    let mut contract = FusionPlusContract::new(accounts(1));
    
    // Create HTLC with short timelock
    let args = HTLCCreateArgs {
        receiver: accounts(2),
        token: AccountId::new_unchecked("near".to_string()),
        amount: 1_000_000_000_000_000_000_000_000,
        hashlock: "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b".to_string(),
        timelock: env::block_timestamp() / 1_000_000_000 + 10, // 10 seconds
        order_hash: None,
    };
    
    testing_env!(context
        .attached_deposit(1_000_000_000_000_000_000_000_000)
        .build());
    
    let htlc_id = contract.create_htlc(args);
    
    // Fast forward time
    testing_env!(context
        .block_timestamp((env::block_timestamp() / 1_000_000_000 + 20) * 1_000_000_000)
        .build());
    
    // Refund
    contract.refund(htlc_id.clone());
    
    // Verify HTLC is refunded
    let htlc = contract.get_htlc(htlc_id).unwrap();
    assert_eq!(htlc.state, fusion_plus_htlc::types::HTLCState::Refunded);
}

#[test]
#[should_panic(expected = "Timelock has not expired")]
fn test_refund_before_timeout() {
    let mut context = get_context(accounts(1));
    testing_env!(context.build());
    
    let mut contract = FusionPlusContract::new(accounts(1));
    
    // Create HTLC
    let args = HTLCCreateArgs {
        receiver: accounts(2),
        token: AccountId::new_unchecked("near".to_string()),
        amount: 1_000_000_000_000_000_000_000_000,
        hashlock: "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b".to_string(),
        timelock: env::block_timestamp() / 1_000_000_000 + 3600,
        order_hash: None,
    };
    
    testing_env!(context
        .attached_deposit(1_000_000_000_000_000_000_000_000)
        .build());
    
    let htlc_id = contract.create_htlc(args);
    
    // Try to refund before timeout
    contract.refund(htlc_id);
}