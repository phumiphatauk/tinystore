//! In-memory storage backend for testing

use async_trait::async_trait;
use bytes::Bytes;
use tinystore_shared::{BucketInfo, ObjectInfo, ObjectMetadata, StorageResult};

use crate::backend::{
    StorageBackend, ListObjectsParams, GetObjectResult, PutObjectResult,
    CopyObjectResult, CompleteMultipartResult, PartInfo, CompletedPart,
    Range, ListObjectsResult, StorageStats,
};

/// In-memory storage backend (for testing)
pub struct MemoryBackend {
    // TODO: Add internal data structures in Step 3
}

impl MemoryBackend {
    /// Create a new in-memory backend
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for MemoryBackend {
    async fn create_bucket(&self, _bucket: &str) -> StorageResult<()> {
        todo!("Implement in Step 3")
    }

    async fn delete_bucket(&self, _bucket: &str) -> StorageResult<()> {
        todo!("Implement in Step 3")
    }

    async fn list_buckets(&self) -> StorageResult<Vec<BucketInfo>> {
        todo!("Implement in Step 3")
    }

    async fn bucket_exists(&self, _bucket: &str) -> StorageResult<bool> {
        todo!("Implement in Step 3")
    }

    async fn get_bucket_info(&self, _bucket: &str) -> StorageResult<BucketInfo> {
        todo!("Implement in Step 3")
    }

    async fn put_object(
        &self,
        _bucket: &str,
        _key: &str,
        _data: Bytes,
        _metadata: ObjectMetadata,
    ) -> StorageResult<PutObjectResult> {
        todo!("Implement in Step 3")
    }

    async fn get_object(
        &self,
        _bucket: &str,
        _key: &str,
        _range: Option<Range>,
    ) -> StorageResult<GetObjectResult> {
        todo!("Implement in Step 3")
    }

    async fn delete_object(&self, _bucket: &str, _key: &str) -> StorageResult<()> {
        todo!("Implement in Step 3")
    }

    async fn head_object(&self, _bucket: &str, _key: &str) -> StorageResult<ObjectMetadata> {
        todo!("Implement in Step 3")
    }

    async fn list_objects(
        &self,
        _bucket: &str,
        _params: ListObjectsParams,
    ) -> StorageResult<ListObjectsResult> {
        todo!("Implement in Step 3")
    }

    async fn copy_object(
        &self,
        _source_bucket: &str,
        _source_key: &str,
        _dest_bucket: &str,
        _dest_key: &str,
    ) -> StorageResult<CopyObjectResult> {
        todo!("Implement in Step 3")
    }

    async fn create_multipart_upload(
        &self,
        _bucket: &str,
        _key: &str,
        _metadata: ObjectMetadata,
    ) -> StorageResult<String> {
        todo!("Implement in Step 3")
    }

    async fn upload_part(
        &self,
        _bucket: &str,
        _key: &str,
        _upload_id: &str,
        _part_number: u32,
        _data: Bytes,
    ) -> StorageResult<PartInfo> {
        todo!("Implement in Step 3")
    }

    async fn complete_multipart_upload(
        &self,
        _bucket: &str,
        _key: &str,
        _upload_id: &str,
        _parts: Vec<CompletedPart>,
    ) -> StorageResult<CompleteMultipartResult> {
        todo!("Implement in Step 3")
    }

    async fn abort_multipart_upload(
        &self,
        _bucket: &str,
        _key: &str,
        _upload_id: &str,
    ) -> StorageResult<()> {
        todo!("Implement in Step 3")
    }

    async fn list_parts(
        &self,
        _bucket: &str,
        _key: &str,
        _upload_id: &str,
    ) -> StorageResult<Vec<PartInfo>> {
        todo!("Implement in Step 3")
    }

    async fn get_stats(&self) -> StorageResult<StorageStats> {
        todo!("Implement in Step 3")
    }
}
