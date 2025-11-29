//! S3 API router setup

use axum::{
    routing::{delete, get, head, post, put},
    Router,
};
use std::sync::Arc;
use tinystore_storage::StorageBackend;

/// Create the S3 API router
pub fn create_s3_router<B>(backend: Arc<B>) -> Router
where
    B: StorageBackend + 'static,
{
    // TODO: Implement in Step 4
    let _ = backend;
    Router::new()
        .route("/", get(|| async { "TinyStore S3 API - TODO: Implement in Step 4" }))
}
