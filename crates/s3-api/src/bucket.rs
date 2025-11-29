//! Bucket operation handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tinystore_storage::StorageBackend;

/// Create a bucket (PUT /{bucket})
pub async fn create_bucket<B>(
    State(_backend): State<Arc<B>>,
    Path(_bucket): Path<String>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4
    StatusCode::NOT_IMPLEMENTED
}

/// Delete a bucket (DELETE /{bucket})
pub async fn delete_bucket<B>(
    State(_backend): State<Arc<B>>,
    Path(_bucket): Path<String>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4
    StatusCode::NOT_IMPLEMENTED
}

/// List all buckets (GET /)
pub async fn list_buckets<B>(
    State(_backend): State<Arc<B>>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4
    StatusCode::NOT_IMPLEMENTED
}

/// Check if bucket exists (HEAD /{bucket})
pub async fn head_bucket<B>(
    State(_backend): State<Arc<B>>,
    Path(_bucket): Path<String>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4
    StatusCode::NOT_IMPLEMENTED
}
