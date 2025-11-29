//! API request and response types for UI

use serde::{Deserialize, Serialize};
use crate::{BucketInfo, BucketStats, ObjectInfo};

/// Response for listing buckets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBucketsResponse {
    pub buckets: Vec<BucketInfo>,
}

/// Response for listing objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObjectsResponse {
    pub objects: Vec<ObjectInfo>,
    pub is_truncated: bool,
    pub next_continuation_token: Option<String>,
}

/// Request to create a bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBucketRequest {
    pub name: String,
}

/// Server status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_buckets: u64,
    pub total_objects: u64,
    pub total_size_bytes: u64,
}

/// Credential information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialInfo {
    pub id: String,
    pub access_key: String,
    pub is_admin: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create a new credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCredentialRequest {
    pub access_key: String,
    pub secret_key: String,
    pub is_admin: bool,
}
