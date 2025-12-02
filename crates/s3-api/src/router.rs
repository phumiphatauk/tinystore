//! S3 API router setup

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{delete, get, head, post, put},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tinystore_storage::StorageBackend;

use crate::{bucket, health, multipart, object};

/// Create the S3 API router
pub fn create_s3_router<B>(backend: Arc<B>) -> Router
where
    B: StorageBackend + 'static,
{
    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        // Root - List all buckets
        .route("/", get(bucket::list_buckets::<B>))
        // Bucket operations with query parameter handling
        .route(
            "/:bucket",
            get(handle_bucket_get::<B>)
                .put(bucket::create_bucket::<B>)
                .delete(bucket::delete_bucket::<B>)
                .head(bucket::head_bucket::<B>)
                .post(handle_bucket_post::<B>),
        )
        // Object operations with query parameter handling
        .route(
            "/:bucket/*key",
            get(handle_object_get::<B>)
                .put(handle_object_put::<B>)
                .delete(handle_object_delete::<B>)
                .head(object::head_object::<B>)
                .post(handle_object_post::<B>),
        )
        .with_state(backend)
}

/// Handle GET /{bucket} with query parameters
async fn handle_bucket_get<B>(
    State(backend): State<Arc<B>>,
    Path(bucket): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl axum::response::IntoResponse
where
    B: StorageBackend,
{
    // Check for special operations
    if params.contains_key("location") {
        return bucket::get_bucket_location::<B>(State(backend), Path(bucket)).await.into_response();
    }
    if params.contains_key("uploads") {
        return multipart::list_multipart_uploads::<B>(State(backend), Path(bucket)).await.into_response();
    }

    // Default: list objects
    object::list_objects::<B>(State(backend), Path(bucket), Query(params)).await.into_response()
}

/// Handle POST /{bucket} with query parameters (bulk delete)
async fn handle_bucket_post<B>(
    State(backend): State<Arc<B>>,
    Path(bucket): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    body: axum::body::Bytes,
) -> impl axum::response::IntoResponse
where
    B: StorageBackend,
{
    if params.contains_key("delete") {
        return object::delete_objects::<B>(State(backend), Path(bucket), body).await.into_response();
    }

    // Invalid operation
    axum::http::StatusCode::BAD_REQUEST.into_response()
}

/// Handle GET /{bucket}/{key} with query parameters
async fn handle_object_get<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> impl axum::response::IntoResponse
where
    B: StorageBackend,
{
    if params.contains_key("uploadId") {
        // List parts
        return multipart::list_parts::<B>(State(backend), Path((bucket, key)), Query(params)).await.into_response();
    }

    // Default: get object
    object::get_object::<B>(State(backend), Path((bucket, key)), headers).await.into_response()
}

/// Handle PUT /{bucket}/{key} with query parameters
async fn handle_object_put<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> impl axum::response::IntoResponse
where
    B: StorageBackend,
{
    if params.contains_key("partNumber") && params.contains_key("uploadId") {
        // Upload part
        return multipart::upload_part::<B>(State(backend), Path((bucket, key)), Query(params), body).await.into_response();
    }

    // Default: put object
    object::put_object::<B>(State(backend), Path((bucket, key)), headers, body).await.into_response()
}

/// Handle POST /{bucket}/{key} with query parameters (multipart)
async fn handle_object_post<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    body: axum::body::Bytes,
) -> impl axum::response::IntoResponse
where
    B: StorageBackend,
{
    if params.contains_key("uploads") {
        // Initiate multipart upload
        return multipart::create_multipart_upload::<B>(State(backend), Path((bucket, key))).await.into_response();
    }

    if params.contains_key("uploadId") {
        // Complete multipart upload
        return multipart::complete_multipart_upload::<B>(
            State(backend),
            Path((bucket, key)),
            Query(params),
            body,
        ).await.into_response();
    }

    axum::http::StatusCode::BAD_REQUEST.into_response()
}

/// Handle DELETE /{bucket}/{key} with query parameters
async fn handle_object_delete<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> impl axum::response::IntoResponse
where
    B: StorageBackend,
{
    if params.contains_key("uploadId") {
        // Abort multipart upload
        return multipart::abort_multipart_upload::<B>(
            State(backend),
            Path((bucket, key)),
            Query(params),
        ).await.into_response();
    }

    // Default: delete object
    object::delete_object::<B>(State(backend), Path((bucket, key))).await.into_response()
}
