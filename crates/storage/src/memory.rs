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
