//! AWS Signature Version 4 implementation

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

/// AWS Signature V4 calculator
pub struct SignatureV4;

impl SignatureV4 {
    /// Calculate the signature for a request
    pub fn calculate_signature(
        access_key: &str,
        secret_key: &str,
        method: &str,
        uri: &str,
        query_string: &str,
        headers: &HashMap<String, String>,
        payload_hash: &str,
        timestamp: DateTime<Utc>,
        region: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // TODO: Implement in Step 5
        let _ = (access_key, secret_key, method, uri, query_string, headers, payload_hash, timestamp, region);
        todo!("Implement in Step 5")
    }

    /// Verify a request signature
    pub fn verify_signature(
        authorization_header: &str,
        method: &str,
        uri: &str,
        query_string: &str,
        headers: &HashMap<String, String>,
        payload_hash: &str,
        secret_key: &str,
        region: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // TODO: Implement in Step 5
        let _ = (authorization_header, method, uri, query_string, headers, payload_hash, secret_key, region);
        todo!("Implement in Step 5")
    }

    /// Calculate SHA256 hash of payload
    pub fn hash_payload(payload: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        hex::encode(hasher.finalize())
    }
}
