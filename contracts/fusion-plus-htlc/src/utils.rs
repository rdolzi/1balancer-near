use near_sdk::env;
use sha2::{Digest, Sha256};

/// Validates that a secret matches a hashlock
pub fn validate_secret(secret: &str, hashlock: &str) -> bool {
    // Remove any 0x prefix if present
    let hashlock_clean = hashlock.trim_start_matches("0x");
    
    // Hash the secret
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);
    
    // Compare with hashlock
    hash_hex == hashlock_clean
}

/// Validates hashlock format (should be 64 hex chars)
pub fn validate_hashlock(hashlock: &str) -> bool {
    let hashlock_clean = hashlock.trim_start_matches("0x");
    hashlock_clean.len() == 64 && hashlock_clean.chars().all(|c| c.is_ascii_hexdigit())
}

/// Validates timelock is in the future
pub fn validate_timelock(timelock: u64) -> bool {
    timelock > env::block_timestamp() / 1_000_000_000 // Convert nanoseconds to seconds
}

/// Generates a unique HTLC ID
pub fn generate_htlc_id(sender: &str, receiver: &str, timestamp: u64) -> String {
    let data = format!("{}-{}-{}", sender, receiver, timestamp);
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let hash = hasher.finalize();
    hex::encode(&hash[..16]) // Use first 16 bytes for shorter ID
}

/// Gets current timestamp in seconds
pub fn current_timestamp_sec() -> u64 {
    env::block_timestamp() / 1_000_000_000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_secret() {
        let secret = "mysecret";
        let correct_hashlock = "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b";
        let wrong_hashlock = "wrong_hash";
        
        assert!(validate_secret(secret, correct_hashlock));
        assert!(!validate_secret(secret, wrong_hashlock));
    }

    #[test]
    fn test_validate_hashlock() {
        let valid_hashlock = "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b";
        let valid_with_prefix = "0x2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b";
        let invalid_length = "2bb80d537b1da3e38bd30361aa855686";
        let invalid_chars = "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25g";
        
        assert!(validate_hashlock(valid_hashlock));
        assert!(validate_hashlock(valid_with_prefix));
        assert!(!validate_hashlock(invalid_length));
        assert!(!validate_hashlock(invalid_chars));
    }
}