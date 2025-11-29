//! Bucket operation handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tinystore_storage::StorageBackend;
use tracing::{debug, info};

use crate::error::S3Result;
use crate::xml::{BucketXml, Buckets, ListAllMyBucketsResult, Owner};

/// Create a bucket (PUT /{bucket})
pub async fn create_bucket<B>(
    State(backend): State<Arc<B>>,
    Path(bucket): Path<String>,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    info!("S3 API: CreateBucket request for bucket: {}", bucket);
    backend.create_bucket(&bucket).await?;
    debug!("S3 API: CreateBucket succeeded for bucket: {}", bucket);
    Ok(StatusCode::OK)
}

/// Delete a bucket (DELETE /{bucket})
pub async fn delete_bucket<B>(
    State(backend): State<Arc<B>>,
    Path(bucket): Path<String>,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    info!("S3 API: DeleteBucket request for bucket: {}", bucket);
    backend.delete_bucket(&bucket).await?;
    debug!("S3 API: DeleteBucket succeeded for bucket: {}", bucket);
    Ok(StatusCode::NO_CONTENT)
}

/// List all buckets (GET /)
pub async fn list_buckets<B>(
    State(backend): State<Arc<B>>,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    debug!("S3 API: ListBuckets request");
    let buckets = backend.list_buckets().await?;

    let response = ListAllMyBucketsResult {
        owner: Owner {
            id: "tinystore".to_string(),
            display_name: "TinyStore".to_string(),
        },
        buckets: Buckets {
            bucket: buckets.into_iter().map(BucketXml::from).collect(),
        },
    };

    let xml = crate::xml::to_xml_string(&response)
        .map_err(|e| tinystore_shared::StorageError::SerializationError(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [("Content-Type", "application/xml")],
        xml,
    ))
}

/// Check if bucket exists (HEAD /{bucket})
pub async fn head_bucket<B>(
    State(backend): State<Arc<B>>,
    Path(bucket): Path<String>,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    let exists = backend.bucket_exists(&bucket).await?;

    if exists {
        Ok(StatusCode::OK)
    } else {
        Err(tinystore_shared::StorageError::BucketNotFound(bucket).into())
    }
}
