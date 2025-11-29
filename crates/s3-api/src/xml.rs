//! XML serialization helpers for S3 API

use chrono::{DateTime, Utc};
use serde::Serialize;
use tinystore_shared::{BucketInfo, ObjectInfo};

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
