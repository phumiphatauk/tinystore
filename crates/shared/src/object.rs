//! Object types and utilities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about an object (used for listing)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectInfo {
    /// Object key (path)
    pub key: String,
    /// Last modification timestamp
    pub last_modified: DateTime<Utc>,
    /// ETag (typically MD5 hash)
    pub etag: String,
    /// Size in bytes
    pub size: u64,
    /// Storage class (STANDARD, etc.)
    pub storage_class: String,
}

/// Metadata for an object
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectMetadata {
    /// Content type (MIME type)
    pub content_type: Option<String>,
    /// Content length in bytes
    pub content_length: u64,
    /// ETag (MD5 hash)
    pub etag: String,
    /// Last modification timestamp
    pub last_modified: DateTime<Utc>,
    /// Custom metadata (user-defined headers)
    pub metadata: HashMap<String, String>,
}

impl ObjectMetadata {
    /// Create new metadata with required fields
    pub fn new(content_length: u64, etag: String) -> Self {
        Self {
            content_type: None,
            content_length,
            etag,
            last_modified: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set content type
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Add custom metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}
