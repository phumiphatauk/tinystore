//! Filesystem-based storage backend

use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use md5::{Md5, Digest};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use tinystore_shared::{BucketInfo, ObjectInfo, ObjectMetadata, StorageError, StorageResult};
use tracing::{debug, error, info, warn};

use crate::backend::{
    StorageBackend, ListObjectsParams, GetObjectResult, PutObjectResult,
    CopyObjectResult, CompleteMultipartResult, PartInfo, CompletedPart,
    Range, ListObjectsResult, StorageStats,
};

/// Information about a multipart upload
#[derive(Debug, Clone)]
struct MultipartUpload {
    bucket: String,
    key: String,
    upload_id: String,
    metadata: ObjectMetadata,
    parts: HashMap<u32, PartInfo>,
}

/// Filesystem-based storage backend
#[derive(Clone)]
pub struct FilesystemBackend {
    data_dir: PathBuf,
    multipart_uploads: Arc<RwLock<HashMap<String, MultipartUpload>>>,
}

impl FilesystemBackend {
    /// Create a new filesystem backend
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            multipart_uploads: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the path to a bucket directory
    fn bucket_path(&self, bucket: &str) -> PathBuf {
        self.data_dir.join("buckets").join(bucket)
    }

    /// Get the path to a bucket metadata file
    fn bucket_metadata_path(&self, bucket: &str) -> PathBuf {
        self.bucket_path(bucket).join(".metadata.json")
    }

    /// Get the path to an object
    fn object_path(&self, bucket: &str, key: &str) -> PathBuf {
        self.bucket_path(bucket).join("objects").join(key)
    }

    /// Get the path to an object metadata file
    fn object_metadata_path(&self, bucket: &str, key: &str) -> PathBuf {
        self.bucket_path(bucket).join("metadata").join(format!("{}.json", key))
    }

    /// Get the path to a multipart upload directory
    fn multipart_path(&self, bucket: &str, upload_id: &str) -> PathBuf {
        self.bucket_path(bucket).join("multipart").join(upload_id)
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

    /// Read metadata from file
    async fn read_metadata(&self, path: &PathBuf) -> StorageResult<ObjectMetadata> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        serde_json::from_str(&content)
            .map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    /// Write metadata to file
    async fn write_metadata(&self, path: &PathBuf, metadata: &ObjectMetadata) -> StorageResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }
        let content = serde_json::to_string_pretty(metadata)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        fs::write(path, content)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))
    }
}

#[async_trait]
impl StorageBackend for FilesystemBackend {
    async fn create_bucket(&self, bucket: &str) -> StorageResult<()> {
        self.validate_bucket_name(bucket)?;

        let bucket_path = self.bucket_path(bucket);
        if bucket_path.exists() {
            return Err(StorageError::BucketAlreadyExists(bucket.to_string()));
        }

        // Create bucket directories
        fs::create_dir_all(&bucket_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        fs::create_dir_all(bucket_path.join("objects"))
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        fs::create_dir_all(bucket_path.join("metadata"))
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        fs::create_dir_all(bucket_path.join("multipart"))
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Write bucket metadata
        let bucket_info = BucketInfo::new(bucket.to_string());
        let metadata_path = self.bucket_metadata_path(bucket);
        let content = serde_json::to_string_pretty(&bucket_info)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        fs::write(metadata_path, content)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn delete_bucket(&self, bucket: &str) -> StorageResult<()> {
        let bucket_path = self.bucket_path(bucket);
        if !bucket_path.exists() {
            return Err(StorageError::BucketNotFound(bucket.to_string()));
        }

        // Check if bucket is empty
        let objects_path = bucket_path.join("objects");
        let mut entries = fs::read_dir(&objects_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        if entries.next_entry().await
            .map_err(|e| StorageError::IoError(e.to_string()))?
            .is_some() {
            return Err(StorageError::BucketNotEmpty(bucket.to_string()));
        }

        // Delete bucket directory
        fs::remove_dir_all(&bucket_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn list_buckets(&self) -> StorageResult<Vec<BucketInfo>> {
        let buckets_dir = self.data_dir.join("buckets");

        // Create buckets directory if it doesn't exist
        if !buckets_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&buckets_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let mut buckets = Vec::new();
        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))? {
            let metadata_path = entry.path().join(".metadata.json");
            if metadata_path.exists() {
                let content = fs::read_to_string(&metadata_path)
                    .await
                    .map_err(|e| StorageError::IoError(e.to_string()))?;
                let bucket_info: BucketInfo = serde_json::from_str(&content)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                buckets.push(bucket_info);
            }
        }

        Ok(buckets)
    }

    async fn bucket_exists(&self, bucket: &str) -> StorageResult<bool> {
        Ok(self.bucket_path(bucket).exists())
    }

    async fn get_bucket_info(&self, bucket: &str) -> StorageResult<BucketInfo> {
        let metadata_path = self.bucket_metadata_path(bucket);
        if !metadata_path.exists() {
            return Err(StorageError::BucketNotFound(bucket.to_string()));
        }

        let content = fs::read_to_string(&metadata_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        serde_json::from_str(&content)
            .map_err(|e| StorageError::SerializationError(e.to_string()))
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

        // Write object data
        let object_path = self.object_path(bucket, key);
        if let Some(parent) = object_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }
        fs::write(&object_path, &data)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Write metadata
        let metadata_path = self.object_metadata_path(bucket, key);
        self.write_metadata(&metadata_path, &metadata).await?;

        Ok(PutObjectResult { etag })
    }

    async fn get_object(
        &self,
        bucket: &str,
        key: &str,
        range: Option<Range>,
    ) -> StorageResult<GetObjectResult> {
        let object_path = self.object_path(bucket, key);
        if !object_path.exists() {
            return Err(StorageError::ObjectNotFound {
                bucket: bucket.to_string(),
                key: key.to_string(),
            });
        }

        let metadata_path = self.object_metadata_path(bucket, key);
        let metadata = self.read_metadata(&metadata_path).await?;

        let mut data = fs::read(&object_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Handle range request
        if let Some(range) = &range {
            let start = range.start as usize;
            let end = range.end.map(|e| e as usize).unwrap_or(data.len());

            if start >= data.len() || end > data.len() || start >= end {
                return Err(StorageError::InvalidRange(format!("{}-{}", start, end)));
            }

            data = data[start..end].to_vec();
        }

        Ok(GetObjectResult {
            data: Bytes::from(data),
            metadata,
            range,
        })
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> StorageResult<()> {
        let object_path = self.object_path(bucket, key);
        if !object_path.exists() {
            return Err(StorageError::ObjectNotFound {
                bucket: bucket.to_string(),
                key: key.to_string(),
            });
        }

        // Delete object file
        fs::remove_file(&object_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Delete metadata file
        let metadata_path = self.object_metadata_path(bucket, key);
        if metadata_path.exists() {
            fs::remove_file(&metadata_path)
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    async fn head_object(&self, bucket: &str, key: &str) -> StorageResult<ObjectMetadata> {
        let metadata_path = self.object_metadata_path(bucket, key);
        if !metadata_path.exists() {
            return Err(StorageError::ObjectNotFound {
                bucket: bucket.to_string(),
                key: key.to_string(),
            });
        }

        self.read_metadata(&metadata_path).await
    }

    async fn list_objects(
        &self,
        bucket: &str,
        params: ListObjectsParams,
    ) -> StorageResult<ListObjectsResult> {
        let objects_dir = self.bucket_path(bucket).join("objects");
        if !objects_dir.exists() {
            return Err(StorageError::BucketNotFound(bucket.to_string()));
        }

        let mut objects = Vec::new();
        let mut common_prefixes = std::collections::HashSet::new();

        // Recursively walk the objects directory
        let walker = walkdir::WalkDir::new(&objects_dir);
        for entry in walker {
            let entry = entry.map_err(|e| StorageError::IoError(e.to_string()))?;
            if entry.file_type().is_file() {
                let path = entry.path();
                let key = path.strip_prefix(&objects_dir)
                    .map_err(|e| StorageError::IoError(e.to_string()))?
                    .to_string_lossy()
                    .to_string();

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

                // Read metadata
                let metadata_path = self.object_metadata_path(bucket, &key);
                if let Ok(metadata) = self.read_metadata(&metadata_path).await {
                    objects.push(ObjectInfo {
                        key,
                        last_modified: metadata.last_modified,
                        etag: metadata.etag,
                        size: metadata.content_length,
                        storage_class: "STANDARD".to_string(),
                    });
                }
            }
        }

        // Sort by key
        objects.sort_by(|a, b| a.key.cmp(&b.key));

        // Apply max_keys limit
        let max_keys = params.max_keys.unwrap_or(1000);
        let is_truncated = objects.len() > max_keys;
        objects.truncate(max_keys);

        Ok(ListObjectsResult {
            objects,
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

        // Create multipart upload directory
        let upload_dir = self.multipart_path(bucket, &upload_id);
        fs::create_dir_all(&upload_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Store upload info
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
        // Verify upload exists
        let uploads = self.multipart_uploads.read().await;
        if !uploads.contains_key(upload_id) {
            return Err(StorageError::InternalError("Upload not found".to_string()));
        }
        drop(uploads);

        // Calculate ETag
        let etag = self.calculate_etag(&data);
        let size = data.len() as u64;

        // Write part file
        let part_path = self.multipart_path(bucket, upload_id).join(format!("part-{}", part_number));
        fs::write(&part_path, &data)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Store part info
        let part_info = PartInfo {
            part_number,
            etag: etag.clone(),
            size,
        };

        self.multipart_uploads.write().await
            .get_mut(upload_id)
            .unwrap()
            .parts
            .insert(part_number, part_info.clone());

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

        // Verify all parts
        for part in &parts {
            if !upload.parts.contains_key(&part.part_number) {
                return Err(StorageError::InternalError(
                    format!("Part {} not found", part.part_number)
                ));
            }
        }

        // Combine parts
        let mut combined_data = Vec::new();
        for part in parts {
            let part_path = self.multipart_path(bucket, upload_id)
                .join(format!("part-{}", part.part_number));
            let part_data = fs::read(&part_path)
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?;
            combined_data.extend_from_slice(&part_data);
        }

        // Put combined object
        let result = self.put_object(
            bucket,
            key,
            Bytes::from(combined_data),
            upload.metadata,
        ).await?;

        // Clean up multipart directory
        let upload_dir = self.multipart_path(bucket, upload_id);
        if upload_dir.exists() {
            fs::remove_dir_all(&upload_dir)
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }

        Ok(CompleteMultipartResult { etag: result.etag })
    }

    async fn abort_multipart_upload(
        &self,
        bucket: &str,
        _key: &str,
        upload_id: &str,
    ) -> StorageResult<()> {
        // Remove from uploads map
        self.multipart_uploads.write().await.remove(upload_id);

        // Clean up multipart directory
        let upload_dir = self.multipart_path(bucket, upload_id);
        if upload_dir.exists() {
            fs::remove_dir_all(&upload_dir)
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }

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

        let mut parts: Vec<_> = upload.parts.values().cloned().collect();
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
