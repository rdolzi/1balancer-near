use near_sdk::log;

/// Emit standardized events for cross-chain monitoring
pub struct EventEmitter;

impl EventEmitter {
    /// Emit a cross-chain coordination event
    pub fn emit_coordination_event(event_type: &str, data: &str) {
        log!(
            "CROSS_CHAIN_EVENT:{}:{}",
            event_type,
            data
        );
    }
    
    /// Emit HTLC state change for monitoring
    pub fn emit_state_change(htlc_id: &str, old_state: &str, new_state: &str) {
        let event_data = near_sdk::serde_json::json!({
            "htlc_id": htlc_id,
            "old_state": old_state,
            "new_state": new_state,
            "timestamp": near_sdk::env::block_timestamp(),
        });
        
        log!(
            "STATE_CHANGE:{}",
            event_data.to_string()
        );
    }
    
    /// Emit secret revealed event (critical for cross-chain)
    pub fn emit_secret_revealed(htlc_id: &str, secret: &str, order_hash: &Option<String>) {
        let event_data = near_sdk::serde_json::json!({
            "event": "SecretRevealed",
            "htlc_id": htlc_id,
            "secret": secret,
            "order_hash": order_hash,
            "timestamp": near_sdk::env::block_timestamp(),
            "chain": "near",
        });
        
        log!(
            "SECRET_REVEALED:{}",
            event_data.to_string()
        );
    }
    
    /// Emit cross-chain sync request
    pub fn emit_sync_request(htlc_id: &str, action: &str) {
        let event_data = near_sdk::serde_json::json!({
            "event": "SyncRequest",
            "htlc_id": htlc_id,
            "action": action,
            "chain": "near",
            "timestamp": near_sdk::env::block_timestamp(),
        });
        
        log!(
            "SYNC_REQUEST:{}",
            event_data.to_string()
        );
    }
}