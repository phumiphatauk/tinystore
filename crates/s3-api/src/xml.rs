//! XML serialization helpers for S3 API

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tinystore_shared::{BucketInfo, ObjectInfo};
use tinystore_storage::CompletedPart;

/// XML response for ListBuckets
#[derive(Debug, Serialize)]
#[serde(rename = "ListAllMyBucketsResult")]
pub struct ListAllMyBucketsResult {
    #[serde(rename = "Owner")]
    pub owner: Owner,
    #[serde(rename = "Buckets")]
    pub buckets: Buckets,
}

#[derive(Debug, Serialize)]
pub struct Owner {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "DisplayName")]
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct Buckets {
    #[serde(rename = "Bucket")]
    pub bucket: Vec<BucketXml>,
}

#[derive(Debug, Serialize)]
pub struct BucketXml {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "CreationDate")]
    pub creation_date: String,
}

impl From<BucketInfo> for BucketXml {
    fn from(info: BucketInfo) -> Self {
        Self {
            name: info.name,
            creation_date: info.creation_date.to_rfc3339(),
        }
    }
}

/// XML response for ListObjectsV2
#[derive(Debug, Serialize)]
#[serde(rename = "ListBucketResult")]
pub struct ListBucketResult {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Prefix")]
    pub prefix: Option<String>,
    #[serde(rename = "MaxKeys")]
    pub max_keys: usize,
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    #[serde(rename = "Contents")]
    pub contents: Vec<ObjectXml>,
    #[serde(rename = "NextContinuationToken", skip_serializing_if = "Option::is_none")]
    pub next_continuation_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ObjectXml {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "LastModified")]
    pub last_modified: String,
    #[serde(rename = "ETag")]
    pub etag: String,
    #[serde(rename = "Size")]
    pub size: u64,
    #[serde(rename = "StorageClass")]
    pub storage_class: String,
}

impl From<ObjectInfo> for ObjectXml {
    fn from(info: ObjectInfo) -> Self {
        Self {
            key: info.key,
            last_modified: info.last_modified.to_rfc3339(),
            etag: format!("\"{}\"", info.etag),
            size: info.size,
            storage_class: info.storage_class,
        }
    }
}

/// Serialize a type to XML string
pub fn to_xml_string<T: Serialize>(value: &T) -> Result<String, quick_xml::de::DeError> {
    let xml = quick_xml::se::to_string(value)?;
    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml))
}

// ============ Request Structures (Deserialization) ============

/// DeleteObjects request
#[derive(Debug, Deserialize)]
#[serde(rename = "Delete")]
pub struct DeleteObjectsRequest {
    #[serde(rename = "Object", default)]
    pub objects: Vec<ObjectIdentifier>,
}

#[derive(Debug, Deserialize)]
pub struct ObjectIdentifier {
    #[serde(rename = "Key")]
    pub key: String,
}

/// CompleteMultipartUpload request
#[derive(Debug, Deserialize)]
#[serde(rename = "CompleteMultipartUpload")]
pub struct CompleteMultipartUploadRequest {
    #[serde(rename = "Part", default)]
    pub parts: Vec<CompletedPartXml>,
}

#[derive(Debug, Deserialize)]
pub struct CompletedPartXml {
    #[serde(rename = "PartNumber")]
    pub part_number: u32,
    #[serde(rename = "ETag")]
    pub etag: String,
}

// ============ Response Structures (Serialization) ============

/// DeleteResult response
#[derive(Debug, Serialize)]
#[serde(rename = "DeleteResult")]
pub struct DeleteResult {
    #[serde(rename = "Deleted", default)]
    pub deleted: Vec<DeletedObject>,
    #[serde(rename = "Error", default)]
    pub errors: Vec<DeleteError>,
}

#[derive(Debug, Serialize)]
pub struct DeletedObject {
    #[serde(rename = "Key")]
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteError {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
}

/// InitiateMultipartUploadResult
#[derive(Debug, Serialize)]
#[serde(rename = "InitiateMultipartUploadResult")]
pub struct InitiateMultipartUploadResult {
    #[serde(rename = "Bucket")]
    pub bucket: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "UploadId")]
    pub upload_id: String,
}

/// CompleteMultipartUploadResult
#[derive(Debug, Serialize)]
#[serde(rename = "CompleteMultipartUploadResult")]
pub struct CompleteMultipartUploadResult {
    #[serde(rename = "Location")]
    pub location: String,
    #[serde(rename = "Bucket")]
    pub bucket: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "ETag")]
    pub etag: String,
}

/// ListPartsResult
#[derive(Debug, Serialize)]
#[serde(rename = "ListPartsResult")]
pub struct ListPartsResult {
    #[serde(rename = "Bucket")]
    pub bucket: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "UploadId")]
    pub upload_id: String,
    #[serde(rename = "Part")]
    pub parts: Vec<PartXml>,
}

#[derive(Debug, Serialize)]
pub struct PartXml {
    #[serde(rename = "PartNumber")]
    pub part_number: u32,
    #[serde(rename = "ETag")]
    pub etag: String,
    #[serde(rename = "Size")]
    pub size: u64,
}

/// ListMultipartUploadsResult
#[derive(Debug, Serialize)]
#[serde(rename = "ListMultipartUploadsResult")]
pub struct ListMultipartUploadsResult {
    #[serde(rename = "Bucket")]
    pub bucket: String,
    #[serde(rename = "Upload", default)]
    pub uploads: Vec<MultipartUploadXml>,
}

#[derive(Debug, Serialize)]
pub struct MultipartUploadXml {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "UploadId")]
    pub upload_id: String,
}

/// LocationConstraint for GetBucketLocation
#[derive(Debug, Serialize)]
#[serde(rename = "LocationConstraint")]
pub struct LocationConstraint {
    #[serde(rename = "$value")]
    pub location: String,
}

// ============ Parsing Functions ============

/// Parse DeleteObjects XML request
pub fn parse_delete_objects(body: &[u8]) -> Result<Vec<String>, quick_xml::de::DeError> {
    let request: DeleteObjectsRequest = quick_xml::de::from_reader(body)?;
    Ok(request.objects.into_iter().map(|o| o.key).collect())
}

/// Parse CompleteMultipartUpload XML request
pub fn parse_complete_multipart_upload(body: &[u8]) -> Result<Vec<CompletedPart>, quick_xml::de::DeError> {
    let request: CompleteMultipartUploadRequest = quick_xml::de::from_reader(body)?;
    Ok(request.parts.into_iter().map(|p| CompletedPart {
        part_number: p.part_number,
        etag: p.etag,
    }).collect())
}
