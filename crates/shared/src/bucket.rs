//! Bucket types and utilities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Information about a bucket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BucketInfo {
    /// Bucket name
    pub name: String,
    /// Creation timestamp
    pub creation_date: DateTime<Utc>,
}

/// Statistics about a bucket
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BucketStats {
    /// Number of objects in the bucket
    pub object_count: u64,
    /// Total size of all objects in bytes
    pub total_size: u64,
}

impl BucketInfo {
    /// Create a new BucketInfo
    pub fn new(name: String) -> Self {
        Self {
            name,
            creation_date: Utc::now(),
        }
    }
}
