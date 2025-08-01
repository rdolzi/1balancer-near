use near_sdk::{AccountId, Balance};
use near_workspaces::{Account, Contract, Worker};
use serde_json::json;

const HTLC_WASM_FILEPATH: &str = "./target/wasm32-unknown-unknown/release/fusion_plus_htlc.wasm";

#[tokio::test]
async fn test_cross_chain_htlc_flow() -> Result<(), Box<dyn std::error::Error>> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(HTLC_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    // Initialize contract
    let owner = worker.dev_create_account().await?;
    let outcome = contract
        .call("new")
        .args_json(json!({
            "owner": owner.id()
        }))
        .transact()
        .await?;
    assert!(outcome.is_success());

    // Create accounts
    let sender = worker.dev_create_account().await?;
    let receiver = worker.dev_create_account().await?;

    // Create HTLC (simulating cross-chain coordination)
    let hashlock = "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b"; // sha256("mysecret")
    let timelock = 1_000_000_000 + 3600; // 1 hour from epoch
    let order_hash = "0x123456"; // Simulated Ethereum order hash

    let htlc_args = json!({
        "args": {
            "receiver": receiver.id(),
            "token": "near",
            "amount": "1000000000000000000000000", // 1 NEAR
            "hashlock": hashlock,
            "timelock": timelock,
            "order_hash": order_hash
        }
    });

    let outcome = sender
        .call(contract.id(), "create_htlc")
        .args_json(htlc_args)
        .deposit(1_000_000_000_000_000_000_000_000) // 1 NEAR
        .transact()
        .await?;
    
    assert!(outcome.is_success());
    let htlc_id: String = outcome.json()?;

    // Get HTLC info
    let htlc_info = contract
        .call("get_htlc")
        .args_json(json!({ "htlc_id": htlc_id.clone() }))
        .view()
        .await?
        .json::<serde_json::Value>()?;

    assert_eq!(htlc_info["sender"], sender.id().to_string());
    assert_eq!(htlc_info["receiver"], receiver.id().to_string());
    assert_eq!(htlc_info["hashlock"], hashlock);

    // Simulate cross-chain coordination - receiver withdraws with secret
    let outcome = receiver
        .call(contract.id(), "withdraw")
        .args_json(json!({
            "htlc_id": htlc_id,
            "secret": "mysecret"
        }))
        .transact()
        .await?;

    assert!(outcome.is_success());

    // Verify HTLC is withdrawn
    let htlc_info = contract
        .call("get_htlc")
        .args_json(json!({ "htlc_id": htlc_id }))
        .view()
        .await?
        .json::<serde_json::Value>()?;

    assert_eq!(htlc_info["state"], "Withdrawn");
    assert_eq!(htlc_info["secret"], "mysecret");

    Ok(())
}

#[tokio::test]
async fn test_cross_chain_timeout_refund() -> Result<(), Box<dyn std::error::Error>> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(HTLC_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    // Initialize
    let owner = worker.dev_create_account().await?;
    contract
        .call("new")
        .args_json(json!({ "owner": owner.id() }))
        .transact()
        .await?;

    let sender = worker.dev_create_account().await?;
    let receiver = worker.dev_create_account().await?;

    // Create HTLC with very short timelock
    let htlc_args = json!({
        "args": {
            "receiver": receiver.id(),
            "token": "near",
            "amount": "1000000000000000000000000",
            "hashlock": "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b",
            "timelock": 1, // Already expired
            "order_hash": "0x789"
        }
    });

    let outcome = sender
        .call(contract.id(), "create_htlc")
        .args_json(htlc_args)
        .deposit(1_000_000_000_000_000_000_000_000)
        .transact()
        .await?;

    let htlc_id: String = outcome.json()?;

    // Fast forward time (simulate timeout)
    worker.fast_forward(100).await?;

    // Sender refunds after timeout
    let outcome = sender
        .call(contract.id(), "refund")
        .args_json(json!({ "htlc_id": htlc_id }))
        .transact()
        .await?;

    assert!(outcome.is_success());

    // Verify refunded
    let htlc_info = contract
        .call("get_htlc")
        .args_json(json!({ "htlc_id": htlc_id }))
        .view()
        .await?
        .json::<serde_json::Value>()?;

    assert_eq!(htlc_info["state"], "Refunded");

    Ok(())
}