//! Object operation handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tinystore_storage::StorageBackend;

/// Put an object (PUT /{bucket}/{key})
pub async fn put_object<B>(
    State(_backend): State<Arc<B>>,
    Path((_bucket, _key)): Path<(String, String)>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4
    StatusCode::NOT_IMPLEMENTED
}

/// Get an object (GET /{bucket}/{key})
pub async fn get_object<B>(
    State(_backend): State<Arc<B>>,
    Path((_bucket, _key)): Path<(String, String)>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4
    StatusCode::NOT_IMPLEMENTED
}

/// Get object metadata (HEAD /{bucket}/{key})
pub async fn head_object<B>(
    State(_backend): State<Arc<B>>,
    Path((_bucket, _key)): Path<(String, String)>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4
    StatusCode::NOT_IMPLEMENTED
}

/// Delete an object (DELETE /{bucket}/{key})
pub async fn delete_object<B>(
    State(_backend): State<Arc<B>>,
    Path((_bucket, _key)): Path<(String, String)>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4
    StatusCode::NOT_IMPLEMENTED
}

/// List objects in a bucket (GET /{bucket})
pub async fn list_objects<B>(
    State(_backend): State<Arc<B>>,
    Path(_bucket): Path<String>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4
    StatusCode::NOT_IMPLEMENTED
}

/// Copy an object (PUT /{bucket}/{key} with x-amz-copy-source header)
pub async fn copy_object<B>(
    State(_backend): State<Arc<B>>,
    Path((_bucket, _key)): Path<(String, String)>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4
    StatusCode::NOT_IMPLEMENTED
}
