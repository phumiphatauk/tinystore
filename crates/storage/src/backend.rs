//! Storage backend trait definition

use async_trait::async_trait;
use bytes::Bytes;
use tinystore_shared::{BucketInfo, ObjectInfo, ObjectMetadata, StorageError, StorageResult};

/// Parameters for listing objects
#[derive(Debug, Clone, Default)]
pub struct ListObjectsParams {
    /// Filter by key prefix
    pub prefix: Option<String>,
    /// Group keys by delimiter
    pub delimiter: Option<String>,
    /// Maximum number of keys to return
    pub max_keys: Option<usize>,
    /// Continuation token for pagination
    pub continuation_token: Option<String>,
}

/// Range of bytes to retrieve
#[derive(Debug, Clone)]
pub struct Range {
    pub start: u64,
    pub end: Option<u64>,
}

/// Result of a GetObject operation
#[derive(Debug)]
pub struct GetObjectResult {
    pub data: Bytes,
    pub metadata: ObjectMetadata,
    pub range: Option<Range>,
}

/// Result of a PutObject operation
#[derive(Debug)]
pub struct PutObjectResult {
    pub etag: String,
}

/// Result of a CopyObject operation
#[derive(Debug)]
pub struct CopyObjectResult {
    pub etag: String,
}

/// Result of listing objects
#[derive(Debug)]
pub struct ListObjectsResult {
    pub objects: Vec<ObjectInfo>,
    pub common_prefixes: Vec<String>,
    pub is_truncated: bool,
    pub next_continuation_token: Option<String>,
}

/// Information about an uploaded part
#[derive(Debug, Clone)]
pub struct PartInfo {
    pub part_number: u32,
    pub etag: String,
    pub size: u64,
}

/// Completed part information
#[derive(Debug, Clone)]
pub struct CompletedPart {
    pub part_number: u32,
    pub etag: String,
}

/// Result of completing a multipart upload
#[derive(Debug)]
pub struct CompleteMultipartResult {
    pub etag: String,
}

/// Storage statistics
#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    pub total_buckets: u64,
    pub total_objects: u64,
    pub total_size_bytes: u64,
}

/// Trait for storage backends
#[async_trait]
pub trait StorageBackend: Send + Sync {
    // ===== Bucket operations =====

    /// Create a new bucket
    async fn create_bucket(&self, bucket: &str) -> StorageResult<()>;

    /// Delete a bucket (must be empty)
    async fn delete_bucket(&self, bucket: &str) -> StorageResult<()>;

    /// List all buckets
    async fn list_buckets(&self) -> StorageResult<Vec<BucketInfo>>;

    /// Check if a bucket exists
    async fn bucket_exists(&self, bucket: &str) -> StorageResult<bool>;

    /// Get bucket information
    async fn get_bucket_info(&self, bucket: &str) -> StorageResult<BucketInfo>;

    // ===== Object operations =====

    /// Put an object
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Bytes,
        metadata: ObjectMetadata,
    ) -> StorageResult<PutObjectResult>;

    /// Get an object
    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
        range: Option<Range>,
    ) -> StorageResult<GetObjectResult>;

    /// Delete an object
    async fn delete_object(&self, bucket: &str, key: &str) -> StorageResult<()>;

    /// Get object metadata (HEAD)
    async fn head_object(&self, bucket: &str, key: &str) -> StorageResult<ObjectMetadata>;

    /// List objects in a bucket
    async fn list_objects(
        &self,
        bucket: &str,
        params: ListObjectsParams,
    ) -> StorageResult<ListObjectsResult>;

    /// Copy an object
    async fn copy_object(
        &self,
        source_bucket: &str,
        source_key: &str,
        dest_bucket: &str,
        dest_key: &str,
    ) -> StorageResult<CopyObjectResult>;

    // ===== Multipart upload operations =====

    /// Create a multipart upload
    async fn create_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        metadata: ObjectMetadata,
    ) -> StorageResult<String>;

    /// Upload a part
    async fn upload_part(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> StorageResult<PartInfo>;

    /// Complete a multipart upload
    async fn complete_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        parts: Vec<CompletedPart>,
    ) -> StorageResult<CompleteMultipartResult>;

    /// Abort a multipart upload
    async fn abort_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> StorageResult<()>;

    /// List parts of a multipart upload
    async fn list_parts(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> StorageResult<Vec<PartInfo>>;

    // ===== Statistics =====

    /// Get storage statistics
    async fn get_stats(&self) -> StorageResult<StorageStats>;
}
