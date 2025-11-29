//! Error types for TinyStore

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Storage error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum StorageError {
    #[error("Bucket already exists: {0}")]
    BucketAlreadyExists(String),

    #[error("Bucket not found: {0}")]
    BucketNotFound(String),

    #[error("Bucket not empty: {0}")]
    BucketNotEmpty(String),

    #[error("Invalid bucket name: {0}")]
    InvalidBucketName(String),

    #[error("Object not found: {bucket}/{key}")]
    ObjectNotFound { bucket: String, key: String },

    #[error("Invalid object key: {0}")]
    InvalidObjectKey(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid range: {0}")]
    InvalidRange(String),

    #[error("Entity too large")]
    EntityTooLarge,

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

impl StorageError {
    /// Get the S3 error code for this error
    pub fn s3_code(&self) -> &'static str {
        match self {
            StorageError::BucketAlreadyExists(_) => "BucketAlreadyExists",
            StorageError::BucketNotFound(_) => "NoSuchBucket",
            StorageError::BucketNotEmpty(_) => "BucketNotEmpty",
            StorageError::InvalidBucketName(_) => "InvalidBucketName",
            StorageError::ObjectNotFound { .. } => "NoSuchKey",
            StorageError::InvalidObjectKey(_) => "InvalidArgument",
            StorageError::IoError(_) => "InternalError",
            StorageError::SerializationError(_) => "InternalError",
            StorageError::InvalidRange(_) => "InvalidRange",
            StorageError::EntityTooLarge => "EntityTooLarge",
            StorageError::InternalError(_) => "InternalError",
            StorageError::NotImplemented(_) => "NotImplemented",
        }
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            StorageError::BucketAlreadyExists(_) => 409,
            StorageError::BucketNotFound(_) => 404,
            StorageError::BucketNotEmpty(_) => 409,
            StorageError::InvalidBucketName(_) => 400,
            StorageError::ObjectNotFound { .. } => 404,
            StorageError::InvalidObjectKey(_) => 400,
            StorageError::IoError(_) => 500,
            StorageError::SerializationError(_) => 500,
            StorageError::InvalidRange(_) => 416,
            StorageError::EntityTooLarge => 400,
            StorageError::InternalError(_) => 500,
            StorageError::NotImplemented(_) => 501,
        }
    }
}
