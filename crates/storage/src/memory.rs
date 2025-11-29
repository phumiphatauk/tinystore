//! In-memory storage backend for testing

use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use md5::{Md5, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tinystore_shared::{BucketInfo, ObjectInfo, ObjectMetadata, StorageError, StorageResult};

use crate::backend::{
    StorageBackend, ListObjectsParams, GetObjectResult, PutObjectResult,
    CopyObjectResult, CompleteMultipartResult, PartInfo, CompletedPart,
    Range, ListObjectsResult, StorageStats,
};

/// Stored object data
#[derive(Debug, Clone)]
struct StoredObject {
    data: Bytes,
    metadata: ObjectMetadata,
}

/// Information about a multipart upload
#[derive(Debug, Clone)]
struct MultipartUpload {
    bucket: String,
    key: String,
    upload_id: String,
    metadata: ObjectMetadata,
    parts: HashMap<u32, (Bytes, PartInfo)>,
}

/// In-memory storage backend (for testing)
pub struct MemoryBackend {
    buckets: Arc<RwLock<HashMap<String, BucketInfo>>>,
    objects: Arc<RwLock<HashMap<String, HashMap<String, StoredObject>>>>,
    multipart_uploads: Arc<RwLock<HashMap<String, MultipartUpload>>>,
}

impl MemoryBackend {
    /// Create a new in-memory backend
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            objects: Arc::new(RwLock::new(HashMap::new())),
            multipart_uploads: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Validate bucket name (S3 naming rules)
    fn validate_bucket_name(&self, bucket: &str) -> StorageResult<()> {
        if bucket.is_empty() || bucket.len() > 63 {
            return Err(StorageError::InvalidBucketName(bucket.to_string()));
        }
        if !bucket.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(StorageError::InvalidBucketName(bucket.to_string()));
        }
        Ok(())
    }

    /// Validate object key
    fn validate_object_key(&self, key: &str) -> StorageResult<()> {
        if key.is_empty() || key.len() > 1024 {
            return Err(StorageError::InvalidObjectKey(key.to_string()));
        }
        Ok(())
    }

    /// Calculate MD5 hash of data
    fn calculate_etag(&self, data: &[u8]) -> String {
        let hash = Md5::digest(data);
        format!("\"{}\"", hex::encode(hash))
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for MemoryBackend {
    async fn create_bucket(&self, bucket: &str) -> StorageResult<()> {
        self.validate_bucket_name(bucket)?;

        let mut buckets = self.buckets.write().await;
        if buckets.contains_key(bucket) {
            return Err(StorageError::BucketAlreadyExists(bucket.to_string()));
        }

        let bucket_info = BucketInfo::new(bucket.to_string());
        buckets.insert(bucket.to_string(), bucket_info);

        // Initialize empty objects map for this bucket
        self.objects.write().await.insert(bucket.to_string(), HashMap::new());

        Ok(())
    }

    async fn delete_bucket(&self, bucket: &str) -> StorageResult<()> {
        let mut buckets = self.buckets.write().await;
        if !buckets.contains_key(bucket) {
            return Err(StorageError::BucketNotFound(bucket.to_string()));
        }

        // Check if bucket is empty
        let objects = self.objects.read().await;
        if let Some(bucket_objects) = objects.get(bucket) {
            if !bucket_objects.is_empty() {
                return Err(StorageError::BucketNotEmpty(bucket.to_string()));
            }
        }

        buckets.remove(bucket);
        self.objects.write().await.remove(bucket);

        Ok(())
    }

    async fn list_buckets(&self) -> StorageResult<Vec<BucketInfo>> {
        let buckets = self.buckets.read().await;
        Ok(buckets.values().cloned().collect())
    }

    async fn bucket_exists(&self, bucket: &str) -> StorageResult<bool> {
        Ok(self.buckets.read().await.contains_key(bucket))
    }

    async fn get_bucket_info(&self, bucket: &str) -> StorageResult<BucketInfo> {
        let buckets = self.buckets.read().await;
        buckets.get(bucket)
            .cloned()
            .ok_or_else(|| StorageError::BucketNotFound(bucket.to_string()))
    }

    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Bytes,
        mut metadata: ObjectMetadata,
    ) -> StorageResult<PutObjectResult> {
        self.validate_object_key(key)?;

        if !self.bucket_exists(bucket).await? {
            return Err(StorageError::BucketNotFound(bucket.to_string()));
        }

        // Calculate ETag
        let etag = self.calculate_etag(&data);
        metadata.etag = etag.clone();
        metadata.content_length = data.len() as u64;
        metadata.last_modified = Utc::now();

        let stored_object = StoredObject {
            data,
            metadata,
        };

        let mut objects = self.objects.write().await;
        objects.get_mut(bucket)
            .unwrap()
            .insert(key.to_string(), stored_object);

        Ok(PutObjectResult { etag })
    }

    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
        range: Option<Range>,
    ) -> StorageResult<GetObjectResult> {
        let objects = self.objects.read().await;
        let bucket_objects = objects.get(bucket)
            .ok_or_else(|| StorageError::BucketNotFound(bucket.to_string()))?;

        let stored = bucket_objects.get(key)
            .ok_or_else(|| StorageError::ObjectNotFound {
                bucket: bucket.to_string(),
                key: key.to_string(),
            })?;

        let mut data = stored.data.clone();

        // Handle range request
        if let Some(range) = &range {
            let start = range.start as usize;
            let end = range.end.map(|e| e as usize).unwrap_or(data.len());

            if start >= data.len() || end > data.len() || start >= end {
                return Err(StorageError::InvalidRange(format!("{}-{}", start, end)));
            }

            data = data.slice(start..end);
        }

        Ok(GetObjectResult {
            data,
            metadata: stored.metadata.clone(),
            range,
        })
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> StorageResult<()> {
        let mut objects = self.objects.write().await;
        let bucket_objects = objects.get_mut(bucket)
            .ok_or_else(|| StorageError::BucketNotFound(bucket.to_string()))?;

        bucket_objects.remove(key)
            .ok_or_else(|| StorageError::ObjectNotFound {
                bucket: bucket.to_string(),
                key: key.to_string(),
            })?;

        Ok(())
    }

    async fn head_object(&self, bucket: &str, key: &str) -> StorageResult<ObjectMetadata> {
        let objects = self.objects.read().await;
        let bucket_objects = objects.get(bucket)
            .ok_or_else(|| StorageError::BucketNotFound(bucket.to_string()))?;

        let stored = bucket_objects.get(key)
            .ok_or_else(|| StorageError::ObjectNotFound {
                bucket: bucket.to_string(),
                key: key.to_string(),
            })?;

        Ok(stored.metadata.clone())
    }

    async fn list_objects(
        &self,
        bucket: &str,
        params: ListObjectsParams,
    ) -> StorageResult<ListObjectsResult> {
        let objects = self.objects.read().await;
        let bucket_objects = objects.get(bucket)
            .ok_or_else(|| StorageError::BucketNotFound(bucket.to_string()))?;

        let mut filtered_objects = Vec::new();
        let mut common_prefixes = std::collections::HashSet::new();

        for (key, stored) in bucket_objects {
            // Apply prefix filter
            if let Some(prefix) = &params.prefix {
                if !key.starts_with(prefix) {
                    continue;
                }
            }

            // Handle delimiter (common prefixes)
            if let Some(delimiter) = &params.delimiter {
                let search_start = params.prefix.as_ref().map(|p| p.len()).unwrap_or(0);
                if let Some(pos) = key[search_start..].find(delimiter) {
                    let prefix = &key[..search_start + pos + delimiter.len()];
                    common_prefixes.insert(prefix.to_string());
                    continue;
                }
            }

            filtered_objects.push(ObjectInfo {
                key: key.clone(),
                last_modified: stored.metadata.last_modified,
                etag: stored.metadata.etag.clone(),
                size: stored.metadata.content_length,
                storage_class: "STANDARD".to_string(),
            });
        }

        // Sort by key
        filtered_objects.sort_by(|a, b| a.key.cmp(&b.key));

        // Apply max_keys limit
        let max_keys = params.max_keys.unwrap_or(1000);
        let is_truncated = filtered_objects.len() > max_keys;
        filtered_objects.truncate(max_keys);

        Ok(ListObjectsResult {
            objects: filtered_objects,
            common_prefixes: common_prefixes.into_iter().collect(),
            is_truncated,
            next_continuation_token: None,
        })
    }

    async fn copy_object(
        &self,
        source_bucket: &str,
        source_key: &str,
        dest_bucket: &str,
        dest_key: &str,
    ) -> StorageResult<CopyObjectResult> {
        // Read source object
        let source_result = self.get_object(source_bucket, source_key, None).await?;

        // Put as destination
        let put_result = self.put_object(
            dest_bucket,
            dest_key,
            source_result.data,
            source_result.metadata,
        ).await?;

        Ok(CopyObjectResult { etag: put_result.etag })
    }

    async fn create_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        metadata: ObjectMetadata,
    ) -> StorageResult<String> {
        if !self.bucket_exists(bucket).await? {
            return Err(StorageError::BucketNotFound(bucket.to_string()));
        }

        let upload_id = uuid::Uuid::new_v4().to_string();

        let upload = MultipartUpload {
            bucket: bucket.to_string(),
            key: key.to_string(),
            upload_id: upload_id.clone(),
            metadata,
            parts: HashMap::new(),
        };

        self.multipart_uploads.write().await.insert(upload_id.clone(), upload);

        Ok(upload_id)
    }

    async fn upload_part(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> StorageResult<PartInfo> {
        let etag = self.calculate_etag(&data);
        let size = data.len() as u64;

        let part_info = PartInfo {
            part_number,
            etag: etag.clone(),
            size,
        };

        let mut uploads = self.multipart_uploads.write().await;
        let upload = uploads.get_mut(upload_id)
            .ok_or_else(|| StorageError::InternalError("Upload not found".to_string()))?;

        upload.parts.insert(part_number, (data, part_info.clone()));

        Ok(part_info)
    }

    async fn complete_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
        parts: Vec<CompletedPart>,
    ) -> StorageResult<CompleteMultipartResult> {
        // Get upload info
        let mut uploads = self.multipart_uploads.write().await;
        let upload = uploads.remove(upload_id)
            .ok_or_else(|| StorageError::InternalError("Upload not found".to_string()))?;
        drop(uploads);

        // Verify all parts and combine data
        let mut combined_data = Vec::new();
        for part in parts {
            let (data, _) = upload.parts.get(&part.part_number)
                .ok_or_else(|| StorageError::InternalError(
                    format!("Part {} not found", part.part_number)
                ))?;
            combined_data.extend_from_slice(data);
        }

        // Put combined object
        let result = self.put_object(
            bucket,
            key,
            Bytes::from(combined_data),
            upload.metadata,
        ).await?;

        Ok(CompleteMultipartResult { etag: result.etag })
    }

    async fn abort_multipart_upload(
        &self,
        _bucket: &str,
        _key: &str,
        upload_id: &str,
    ) -> StorageResult<()> {
        self.multipart_uploads.write().await.remove(upload_id);
        Ok(())
    }

    async fn list_parts(
        &self,
        _bucket: &str,
        _key: &str,
        upload_id: &str,
    ) -> StorageResult<Vec<PartInfo>> {
        let uploads = self.multipart_uploads.read().await;
        let upload = uploads.get(upload_id)
            .ok_or_else(|| StorageError::InternalError("Upload not found".to_string()))?;

        let mut parts: Vec<_> = upload.parts.values()
            .map(|(_, info)| info.clone())
            .collect();
        parts.sort_by_key(|p| p.part_number);

        Ok(parts)
    }

    async fn get_stats(&self) -> StorageResult<StorageStats> {
        let buckets = self.list_buckets().await?;
        let mut total_objects = 0u64;
        let mut total_size_bytes = 0u64;

        for bucket in &buckets {
            let objects = self.list_objects(&bucket.name, ListObjectsParams::default()).await?;
            total_objects += objects.objects.len() as u64;
            for obj in objects.objects {
                total_size_bytes += obj.size;
            }
        }

        Ok(StorageStats {
            total_buckets: buckets.len() as u64,
            total_objects,
            total_size_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[tokio::test]
    async fn test_bucket_operations() {
        let backend = MemoryBackend::new();

        // Test create bucket
        backend.create_bucket("test-bucket").await.unwrap();

        // Test bucket exists
        assert!(backend.bucket_exists("test-bucket").await.unwrap());

        // Test duplicate bucket creation fails
        assert!(backend.create_bucket("test-bucket").await.is_err());

        // Test list buckets
        let buckets = backend.list_buckets().await.unwrap();
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].name, "test-bucket");

        // Test get bucket info
        let info = backend.get_bucket_info("test-bucket").await.unwrap();
        assert_eq!(info.name, "test-bucket");

        // Test delete non-empty bucket should fail (after we add objects)
        // First delete should succeed since bucket is empty
        backend.delete_bucket("test-bucket").await.unwrap();
        assert!(!backend.bucket_exists("test-bucket").await.unwrap());
    }

    #[tokio::test]
    async fn test_invalid_bucket_names() {
        let backend = MemoryBackend::new();

        // Empty name
        assert!(backend.create_bucket("").await.is_err());

        // Too long (>63 chars)
        let long_name = "a".repeat(64);
        assert!(backend.create_bucket(&long_name).await.is_err());

        // Invalid characters (uppercase)
        assert!(backend.create_bucket("TestBucket").await.is_err());

        // Valid bucket names
        assert!(backend.create_bucket("test-bucket-123").await.is_ok());
        assert!(backend.create_bucket("another-bucket").await.is_ok());
    }

    #[tokio::test]
    async fn test_object_operations() {
        let backend = MemoryBackend::new();
        backend.create_bucket("test-bucket").await.unwrap();

        // Test put object
        let data = Bytes::from("Hello, World!");
        let metadata = ObjectMetadata::default();
        let result = backend.put_object("test-bucket", "test.txt", data.clone(), metadata).await.unwrap();
        assert!(!result.etag.is_empty());

        // Test head object
        let meta = backend.head_object("test-bucket", "test.txt").await.unwrap();
        assert_eq!(meta.content_length, 13);
        assert_eq!(meta.etag, result.etag);

        // Test get object
        let get_result = backend.get_object("test-bucket", "test.txt", None).await.unwrap();
        assert_eq!(get_result.data, data);
        assert_eq!(get_result.metadata.etag, result.etag);

        // Test delete object
        backend.delete_object("test-bucket", "test.txt").await.unwrap();

        // Verify object is deleted
        assert!(backend.get_object("test-bucket", "test.txt", None).await.is_err());
    }

    #[tokio::test]
    async fn test_object_not_found() {
        let backend = MemoryBackend::new();
        backend.create_bucket("test-bucket").await.unwrap();

        // Get non-existent object
        assert!(backend.get_object("test-bucket", "nonexistent.txt", None).await.is_err());

        // Head non-existent object
        assert!(backend.head_object("test-bucket", "nonexistent.txt").await.is_err());

        // Delete non-existent object
        assert!(backend.delete_object("test-bucket", "nonexistent.txt").await.is_err());
    }

    #[tokio::test]
    async fn test_range_requests() {
        let backend = MemoryBackend::new();
        backend.create_bucket("test-bucket").await.unwrap();

        let data = Bytes::from("0123456789");
        let metadata = ObjectMetadata::default();
        backend.put_object("test-bucket", "test.txt", data, metadata).await.unwrap();

        // Test range request
        let range = Range { start: 2, end: Some(5) };
        let result = backend.get_object("test-bucket", "test.txt", Some(range)).await.unwrap();
        assert_eq!(result.data, Bytes::from("234"));

        // Test range to end
        let range = Range { start: 5, end: None };
        let result = backend.get_object("test-bucket", "test.txt", Some(range)).await.unwrap();
        assert_eq!(result.data, Bytes::from("56789"));
    }

    #[tokio::test]
    async fn test_invalid_range() {
        let backend = MemoryBackend::new();
        backend.create_bucket("test-bucket").await.unwrap();

        let data = Bytes::from("0123456789");
        let metadata = ObjectMetadata::default();
        backend.put_object("test-bucket", "test.txt", data, metadata).await.unwrap();

        // Invalid range (start >= end)
        let range = Range { start: 5, end: Some(5) };
        assert!(backend.get_object("test-bucket", "test.txt", Some(range)).await.is_err());

        // Range beyond data length
        let range = Range { start: 100, end: None };
        assert!(backend.get_object("test-bucket", "test.txt", Some(range)).await.is_err());
    }

    #[tokio::test]
    async fn test_list_objects() {
        let backend = MemoryBackend::new();
        backend.create_bucket("test-bucket").await.unwrap();

        // Put multiple objects
        let metadata = ObjectMetadata::default();
        backend.put_object("test-bucket", "file1.txt", Bytes::from("data1"), metadata.clone()).await.unwrap();
        backend.put_object("test-bucket", "file2.txt", Bytes::from("data2"), metadata.clone()).await.unwrap();
        backend.put_object("test-bucket", "dir/file3.txt", Bytes::from("data3"), metadata.clone()).await.unwrap();

        // List all objects
        let params = ListObjectsParams::default();
        let result = backend.list_objects("test-bucket", params).await.unwrap();
        assert_eq!(result.objects.len(), 3);

        // List with prefix
        let params = ListObjectsParams {
            prefix: Some("file".to_string()),
            ..Default::default()
        };
        let result = backend.list_objects("test-bucket", params).await.unwrap();
        assert_eq!(result.objects.len(), 2);

        // List with delimiter
        let params = ListObjectsParams {
            delimiter: Some("/".to_string()),
            ..Default::default()
        };
        let result = backend.list_objects("test-bucket", params).await.unwrap();
        assert_eq!(result.objects.len(), 2); // file1.txt, file2.txt
        assert_eq!(result.common_prefixes.len(), 1); // dir/
    }

    #[tokio::test]
    async fn test_list_objects_max_keys() {
        let backend = MemoryBackend::new();
        backend.create_bucket("test-bucket").await.unwrap();

        // Put multiple objects
        let metadata = ObjectMetadata::default();
        for i in 0..10 {
            backend.put_object(
                "test-bucket",
                &format!("file{}.txt", i),
                Bytes::from(format!("data{}", i)),
                metadata.clone()
            ).await.unwrap();
        }

        // List with max_keys
        let params = ListObjectsParams {
            max_keys: Some(5),
            ..Default::default()
        };
        let result = backend.list_objects("test-bucket", params).await.unwrap();
        assert_eq!(result.objects.len(), 5);
        assert!(result.is_truncated);
    }

    #[tokio::test]
    async fn test_copy_object() {
        let backend = MemoryBackend::new();
        backend.create_bucket("source-bucket").await.unwrap();
        backend.create_bucket("dest-bucket").await.unwrap();

        // Put source object
        let data = Bytes::from("Copy me!");
        let metadata = ObjectMetadata::default();
        backend.put_object("source-bucket", "source.txt", data.clone(), metadata).await.unwrap();

        // Copy object
        let result = backend.copy_object(
            "source-bucket", "source.txt",
            "dest-bucket", "dest.txt"
        ).await.unwrap();
        assert!(!result.etag.is_empty());

        // Verify copied object
        let get_result = backend.get_object("dest-bucket", "dest.txt", None).await.unwrap();
        assert_eq!(get_result.data, data);

        // Copy within same bucket
        backend.copy_object(
            "source-bucket", "source.txt",
            "source-bucket", "source-copy.txt"
        ).await.unwrap();

        let get_result = backend.get_object("source-bucket", "source-copy.txt", None).await.unwrap();
        assert_eq!(get_result.data, data);
    }

    #[tokio::test]
    async fn test_multipart_upload() {
        let backend = MemoryBackend::new();
        backend.create_bucket("test-bucket").await.unwrap();

        // Create multipart upload
        let metadata = ObjectMetadata::default();
        let upload_id = backend.create_multipart_upload(
            "test-bucket",
            "large-file.bin",
            metadata
        ).await.unwrap();
        assert!(!upload_id.is_empty());

        // Upload parts
        let part1 = Bytes::from("Part 1 data");
        let part2 = Bytes::from("Part 2 data");
        let part3 = Bytes::from("Part 3 data");

        let part1_info = backend.upload_part(
            "test-bucket", "large-file.bin", &upload_id, 1, part1.clone()
        ).await.unwrap();
        let part2_info = backend.upload_part(
            "test-bucket", "large-file.bin", &upload_id, 2, part2.clone()
        ).await.unwrap();
        let part3_info = backend.upload_part(
            "test-bucket", "large-file.bin", &upload_id, 3, part3.clone()
        ).await.unwrap();

        // List parts
        let parts = backend.list_parts("test-bucket", "large-file.bin", &upload_id).await.unwrap();
        assert_eq!(parts.len(), 3);

        // Complete multipart upload
        let completed_parts = vec![
            CompletedPart { part_number: 1, etag: part1_info.etag },
            CompletedPart { part_number: 2, etag: part2_info.etag },
            CompletedPart { part_number: 3, etag: part3_info.etag },
        ];
        let result = backend.complete_multipart_upload(
            "test-bucket",
            "large-file.bin",
            &upload_id,
            completed_parts
        ).await.unwrap();
        assert!(!result.etag.is_empty());

        // Verify combined object
        let get_result = backend.get_object("test-bucket", "large-file.bin", None).await.unwrap();
        let expected_data = Bytes::from("Part 1 dataPart 2 dataPart 3 data");
        assert_eq!(get_result.data, expected_data);
    }

    #[tokio::test]
    async fn test_abort_multipart_upload() {
        let backend = MemoryBackend::new();
        backend.create_bucket("test-bucket").await.unwrap();

        // Create multipart upload
        let metadata = ObjectMetadata::default();
        let upload_id = backend.create_multipart_upload(
            "test-bucket",
            "abandoned-file.bin",
            metadata
        ).await.unwrap();

        // Upload a part
        let part1 = Bytes::from("Part 1 data");
        backend.upload_part(
            "test-bucket", "abandoned-file.bin", &upload_id, 1, part1
        ).await.unwrap();

        // Abort upload
        backend.abort_multipart_upload(
            "test-bucket",
            "abandoned-file.bin",
            &upload_id
        ).await.unwrap();

        // Verify upload is gone
        let result = backend.list_parts("test-bucket", "abandoned-file.bin", &upload_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bucket_not_empty() {
        let backend = MemoryBackend::new();
        backend.create_bucket("test-bucket").await.unwrap();

        // Add an object
        let metadata = ObjectMetadata::default();
        backend.put_object("test-bucket", "file.txt", Bytes::from("data"), metadata).await.unwrap();

        // Try to delete non-empty bucket
        let result = backend.delete_bucket("test-bucket").await;
        assert!(result.is_err());

        // Delete object then bucket
        backend.delete_object("test-bucket", "file.txt").await.unwrap();
        backend.delete_bucket("test-bucket").await.unwrap();
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let backend = MemoryBackend::new();
        backend.create_bucket("bucket1").await.unwrap();
        backend.create_bucket("bucket2").await.unwrap();

        let metadata = ObjectMetadata::default();
        backend.put_object("bucket1", "file1.txt", Bytes::from("12345"), metadata.clone()).await.unwrap();
        backend.put_object("bucket1", "file2.txt", Bytes::from("67890"), metadata.clone()).await.unwrap();
        backend.put_object("bucket2", "file3.txt", Bytes::from("abc"), metadata).await.unwrap();

        let stats = backend.get_stats().await.unwrap();
        assert_eq!(stats.total_buckets, 2);
        assert_eq!(stats.total_objects, 3);
        assert_eq!(stats.total_size_bytes, 13); // 5 + 5 + 3
    }

    #[tokio::test]
    async fn test_object_key_validation() {
        let backend = MemoryBackend::new();
        backend.create_bucket("test-bucket").await.unwrap();

        let metadata = ObjectMetadata::default();

        // Empty key should fail
        let result = backend.put_object("test-bucket", "", Bytes::from("data"), metadata.clone()).await;
        assert!(result.is_err());

        // Very long key (>1024) should fail
        let long_key = "a".repeat(1025);
        let result = backend.put_object("test-bucket", &long_key, Bytes::from("data"), metadata).await;
        assert!(result.is_err());
    }
}
