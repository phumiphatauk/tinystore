//! Object operation handlers

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderName, Response, StatusCode},
    response::IntoResponse,
};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tinystore_shared::ObjectMetadata;
use tinystore_storage::{ListObjectsParams, StorageBackend};
use tracing::{debug, info};

use crate::error::S3Result;
use crate::xml::{ListBucketResult, ObjectXml};

/// Put an object (PUT /{bucket}/{key})
pub async fn put_object<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> S3Result<Response<Body>>
where
    B: StorageBackend,
{
    // Check for copy operation
    if let Some(copy_source) = headers.get("x-amz-copy-source").cloned() {
        return copy_object_impl(backend, bucket, key, copy_source).await;
    }

    // Extract metadata from headers
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let content_length = body.len() as u64;

    // Compute ETag (MD5 hash)
    let etag = format!("{:x}", md5::compute(&body));

    let mut metadata = ObjectMetadata::new(content_length, etag.clone());
    if let Some(ct) = content_type {
        metadata = metadata.with_content_type(ct);
    }

    // Extract custom metadata (x-amz-meta-* headers)
    for (key_name, value) in headers.iter() {
        if let Some(meta_key) = key_name.as_str().strip_prefix("x-amz-meta-") {
            if let Ok(meta_value) = value.to_str() {
                metadata = metadata.with_metadata(meta_key, meta_value);
            }
        }
    }

    // Store the object
    let result = backend
        .put_object(&bucket, &key, body, metadata)
        .await?;

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::ETAG,
        result.etag.parse().unwrap(),
    );

    Ok(response)
}

/// Get an object (GET /{bucket}/{key})
pub async fn get_object<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
    headers: HeaderMap,
) -> S3Result<Response<Body>>
where
    B: StorageBackend,
{
    // Parse range header if present
    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_range);

    // Get the object
    let result = backend.get_object(&bucket, &key, range).await?;

    let mut response = Response::new(Body::from(result.data));
    *response.status_mut() = StatusCode::OK;

    response.headers_mut().insert(
        header::CONTENT_LENGTH,
        result.metadata.content_length.to_string().parse().unwrap(),
    );
    response.headers_mut().insert(
        header::ETAG,
        result.metadata.etag.parse().unwrap(),
    );
    response.headers_mut().insert(
        header::LAST_MODIFIED,
        result.metadata.last_modified.to_rfc2822().parse().unwrap(),
    );

    if let Some(content_type) = result.metadata.content_type {
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            content_type.parse().unwrap(),
        );
    }

    // Add custom metadata as x-amz-meta-* headers
    for (key, value) in result.metadata.metadata.iter() {
        let header_name = HeaderName::from_bytes(format!("x-amz-meta-{}", key).as_bytes()).unwrap();
        response.headers_mut().insert(
            header_name,
            value.parse().unwrap(),
        );
    }

    Ok(response)
}

/// Get object metadata (HEAD /{bucket}/{key})
pub async fn head_object<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
) -> S3Result<Response<Body>>
where
    B: StorageBackend,
{
    let metadata = backend.head_object(&bucket, &key).await?;

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::OK;

    response.headers_mut().insert(
        header::CONTENT_LENGTH,
        metadata.content_length.to_string().parse().unwrap(),
    );
    response.headers_mut().insert(
        header::ETAG,
        metadata.etag.parse().unwrap(),
    );
    response.headers_mut().insert(
        header::LAST_MODIFIED,
        metadata.last_modified.to_rfc2822().parse().unwrap(),
    );

    if let Some(content_type) = metadata.content_type {
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            content_type.parse().unwrap(),
        );
    }

    // Add custom metadata as x-amz-meta-* headers
    for (key, value) in metadata.metadata.iter() {
        let header_name = HeaderName::from_bytes(format!("x-amz-meta-{}", key).as_bytes()).unwrap();
        response.headers_mut().insert(
            header_name,
            value.parse().unwrap(),
        );
    }

    Ok(response)
}

/// Delete an object (DELETE /{bucket}/{key})
pub async fn delete_object<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    backend.delete_object(&bucket, &key).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// List objects in a bucket (GET /{bucket})
pub async fn list_objects<B>(
    State(backend): State<Arc<B>>,
    Path(bucket): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    // Parse query parameters
    let prefix = params.get("prefix").cloned();
    let delimiter = params.get("delimiter").cloned();
    let max_keys = params
        .get("max-keys")
        .and_then(|s| s.parse::<usize>().ok());
    let continuation_token = params.get("continuation-token").cloned();

    let list_params = ListObjectsParams {
        prefix: prefix.clone(),
        delimiter,
        max_keys,
        continuation_token,
    };

    let result = backend.list_objects(&bucket, list_params).await?;

    let response = ListBucketResult {
        name: bucket,
        prefix,
        max_keys: max_keys.unwrap_or(1000),
        is_truncated: result.is_truncated,
        contents: result.objects.into_iter().map(ObjectXml::from).collect(),
        next_continuation_token: result.next_continuation_token,
    };

    let xml = crate::xml::to_xml_string(&response)
        .map_err(|e| tinystore_shared::StorageError::SerializationError(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [("Content-Type", "application/xml")],
        xml,
    ))
}

/// Copy an object implementation
async fn copy_object_impl<B>(
    backend: Arc<B>,
    dest_bucket: String,
    dest_key: String,
    copy_source: header::HeaderValue,
) -> S3Result<Response<Body>>
where
    B: StorageBackend,
{
    // Parse x-amz-copy-source header (format: /source-bucket/source-key)
    let copy_source_str = copy_source
        .to_str()
        .map_err(|_| tinystore_shared::StorageError::InvalidObjectKey("Invalid copy source".to_string()))?;

    let copy_source_str = copy_source_str.trim_start_matches('/');
    let parts: Vec<&str> = copy_source_str.splitn(2, '/').collect();

    if parts.len() != 2 {
        return Err(tinystore_shared::StorageError::InvalidObjectKey(
            "Invalid copy source format".to_string(),
        )
        .into());
    }

    let source_bucket = parts[0];
    let source_key = parts[1];

    // Perform the copy
    let result = backend
        .copy_object(source_bucket, source_key, &dest_bucket, &dest_key)
        .await?;

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::ETAG,
        result.etag.parse().unwrap(),
    );

    Ok(response)
}

/// Parse HTTP Range header
fn parse_range(range_str: &str) -> Option<tinystore_storage::Range> {
    // Format: "bytes=start-end" or "bytes=start-"
    let range_str = range_str.strip_prefix("bytes=")?;

    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return None;
    }

    let start = parts[0].parse::<u64>().ok()?;
    let end = if parts[1].is_empty() {
        None
    } else {
        parts[1].parse::<u64>().ok()
    };

    Some(tinystore_storage::Range { start, end })
}
