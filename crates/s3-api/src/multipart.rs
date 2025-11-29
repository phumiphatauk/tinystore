//! Multipart upload handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tinystore_storage::StorageBackend;

/// Create a multipart upload (POST /{bucket}/{key}?uploads)
pub async fn create_multipart_upload<B>(
    State(_backend): State<Arc<B>>,
    Path((_bucket, _key)): Path<(String, String)>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4 (Phase 2)
    StatusCode::NOT_IMPLEMENTED
}

/// Upload a part (PUT /{bucket}/{key}?partNumber={n}&uploadId={id})
pub async fn upload_part<B>(
    State(_backend): State<Arc<B>>,
    Path((_bucket, _key)): Path<(String, String)>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4 (Phase 2)
    StatusCode::NOT_IMPLEMENTED
}

/// Complete a multipart upload (POST /{bucket}/{key}?uploadId={id})
pub async fn complete_multipart_upload<B>(
    State(_backend): State<Arc<B>>,
    Path((_bucket, _key)): Path<(String, String)>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4 (Phase 2)
    StatusCode::NOT_IMPLEMENTED
}

/// Abort a multipart upload (DELETE /{bucket}/{key}?uploadId={id})
pub async fn abort_multipart_upload<B>(
    State(_backend): State<Arc<B>>,
    Path((_bucket, _key)): Path<(String, String)>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4 (Phase 2)
    StatusCode::NOT_IMPLEMENTED
}

/// List parts (GET /{bucket}/{key}?uploadId={id})
pub async fn list_parts<B>(
    State(_backend): State<Arc<B>>,
    Path((_bucket, _key)): Path<(String, String)>,
) -> impl IntoResponse
where
    B: StorageBackend,
{
    // TODO: Implement in Step 4 (Phase 2)
    StatusCode::NOT_IMPLEMENTED
}
