//! AWS Signature Version 4 implementation

use crate::error::{AuthError, AuthResult};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::{debug, trace};

type HmacSha256 = Hmac<Sha256>;

const SERVICE: &str = "s3";
const ALGORITHM: &str = "AWS4-HMAC-SHA256";

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
    ) -> AuthResult<String> {
        // Step 1: Create canonical request
        let canonical_request = Self::create_canonical_request(
            method,
            uri,
            query_string,
            headers,
            payload_hash,
        );

        // Step 2: Create string to sign
        let date_stamp = timestamp.format("%Y%m%d").to_string();
        let amz_date = timestamp.format("%Y%m%dT%H%M%SZ").to_string();
        let credential_scope = format!("{}/{}/{}/aws4_request", date_stamp, region, SERVICE);

        let canonical_request_hash = Self::hash_string(&canonical_request);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            ALGORITHM,
            amz_date,
            credential_scope,
            canonical_request_hash
        );

        // Step 3: Calculate signature
        let signature = Self::calculate_signature_value(secret_key, &date_stamp, region, &string_to_sign)?;

        // Step 4: Create authorization header
        let signed_headers = Self::get_signed_headers(headers);
        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            ALGORITHM,
            access_key,
            credential_scope,
            signed_headers,
            signature
        );

        Ok(authorization)
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
    ) -> AuthResult<bool> {
        // Parse authorization header
        trace!("Parsing authorization header");
        let parts = Self::parse_authorization_header(authorization_header)?;
        let credential = parts.get("Credential").ok_or(AuthError::MissingCredential)?;
        let provided_signature = parts.get("Signature")
            .ok_or_else(|| AuthError::InvalidAuthorizationHeader("Missing Signature".to_string()))?;

        // Extract date from credential scope
        let credential_parts: Vec<&str> = credential.split('/').collect();
        if credential_parts.len() < 2 {
            return Err(AuthError::InvalidCredentialFormat(credential.to_string()));
        }
        let date_stamp = credential_parts[1];

        // Parse timestamp from date stamp
        let timestamp = chrono::NaiveDate::parse_from_str(date_stamp, "%Y%m%d")
            .map_err(|e| AuthError::InvalidDateFormat(e.to_string()))?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AuthError::InvalidDateFormat(date_stamp.to_string()))?
            .and_utc();

        // Calculate expected signature
        let canonical_request = Self::create_canonical_request(
            method,
            uri,
            query_string,
            headers,
            payload_hash,
        );

        let credential_scope = format!("{}/{}/{}/aws4_request", date_stamp, region, SERVICE);
        let amz_date = timestamp.format("%Y%m%dT%H%M%SZ").to_string();
        let canonical_request_hash = Self::hash_string(&canonical_request);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            ALGORITHM,
            amz_date,
            credential_scope,
            canonical_request_hash
        );

        let expected_signature = Self::calculate_signature_value(secret_key, date_stamp, region, &string_to_sign)?;

        let is_valid = expected_signature == *provided_signature;
        if is_valid {
            debug!("Signature verified successfully");
        } else {
            debug!("Signature verification failed");
        }

        Ok(is_valid)
    }

    /// Calculate SHA256 hash of payload
    pub fn hash_payload(payload: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        hex::encode(hasher.finalize())
    }

    /// Create canonical request
    fn create_canonical_request(
        method: &str,
        uri: &str,
        query_string: &str,
        headers: &HashMap<String, String>,
        payload_hash: &str,
    ) -> String {
        // Canonical URI
        let canonical_uri = if uri.is_empty() { "/" } else { uri };

        // Canonical query string
        let canonical_query = Self::canonicalize_query_string(query_string);

        // Canonical headers
        let canonical_headers = Self::canonicalize_headers(headers);

        // Signed headers
        let signed_headers = Self::get_signed_headers(headers);

        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method,
            canonical_uri,
            canonical_query,
            canonical_headers,
            signed_headers,
            payload_hash
        )
    }

    /// Canonicalize query string
    fn canonicalize_query_string(query_string: &str) -> String {
        if query_string.is_empty() {
            return String::new();
        }

        let mut params: Vec<(&str, &str)> = query_string
            .split('&')
            .filter_map(|param| {
                let mut parts = param.splitn(2, '=');
                match (parts.next(), parts.next()) {
                    (Some(key), Some(value)) => Some((key, value)),
                    (Some(key), None) => Some((key, "")),
                    _ => None,
                }
            })
            .collect();

        params.sort_by(|a, b| a.0.cmp(b.0));

        params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Canonicalize headers
    fn canonicalize_headers(headers: &HashMap<String, String>) -> String {
        let mut canonical_headers: Vec<(String, String)> = headers
            .iter()
            .map(|(k, v)| (k.to_lowercase(), v.trim().to_string()))
            .collect();

        canonical_headers.sort_by(|a, b| a.0.cmp(&b.0));

        canonical_headers
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    /// Get signed headers list
    fn get_signed_headers(headers: &HashMap<String, String>) -> String {
        let mut header_names: Vec<String> = headers
            .keys()
            .map(|k| k.to_lowercase())
            .collect();

        header_names.sort();
        header_names.join(";")
    }

    /// Hash a string using SHA256
    fn hash_string(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Calculate signature value using HMAC-SHA256
    fn calculate_signature_value(
        secret_key: &str,
        date_stamp: &str,
        region: &str,
        string_to_sign: &str,
    ) -> AuthResult<String> {
        let k_secret = format!("AWS4{}", secret_key);
        let k_date = Self::hmac_sha256(k_secret.as_bytes(), date_stamp.as_bytes())?;
        let k_region = Self::hmac_sha256(&k_date, region.as_bytes())?;
        let k_service = Self::hmac_sha256(&k_region, SERVICE.as_bytes())?;
        let k_signing = Self::hmac_sha256(&k_service, b"aws4_request")?;

        let signature = Self::hmac_sha256(&k_signing, string_to_sign.as_bytes())?;
        Ok(hex::encode(signature))
    }

    /// Calculate HMAC-SHA256
    fn hmac_sha256(key: &[u8], data: &[u8]) -> AuthResult<Vec<u8>> {
        let mut mac = HmacSha256::new_from_slice(key)?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// Parse authorization header
    fn parse_authorization_header(header: &str) -> AuthResult<HashMap<String, String>> {
        let mut parts = HashMap::new();

        // Remove "AWS4-HMAC-SHA256 " prefix
        let header = header.trim_start_matches(ALGORITHM).trim();

        for part in header.split(", ") {
            if let Some((key, value)) = part.split_once('=') {
                parts.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(parts)
    }
}
