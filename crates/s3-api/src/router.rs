//! S3 API router setup

use axum::{
    routing::{delete, get, head, post, put},
    Router,
};
use std::sync::Arc;
use tinystore_storage::StorageBackend;

use crate::bucket;
use crate::object;

/// Create the S3 API router
pub fn create_s3_router<B>(backend: Arc<B>) -> Router
where
    B: StorageBackend + 'static,
{
    Router::new()
        // Root - List all buckets (GET /)
        .route("/", get(bucket::list_buckets::<B>))
        // Bucket operations (/{bucket})
        .route(
            "/:bucket",
            get(object::list_objects::<B>)
                .put(bucket::create_bucket::<B>)
                .delete(bucket::delete_bucket::<B>)
                .head(bucket::head_bucket::<B>),
        )
        // Object operations (/{bucket}/{*key})
        .route(
            "/:bucket/*key",
            get(object::get_object::<B>)
                .put(object::put_object::<B>)
                .delete(object::delete_object::<B>)
                .head(object::head_object::<B>),
        )
        .with_state(backend)
}
