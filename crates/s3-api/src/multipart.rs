//! Multipart upload handlers

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use tinystore_shared::ObjectMetadata;
use tinystore_storage::StorageBackend;

use crate::error::S3Result;
use crate::xml;

/// Create a multipart upload (POST /{bucket}/{key}?uploads)
pub async fn create_multipart_upload<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    let metadata = ObjectMetadata::default();
    let upload_id = backend.create_multipart_upload(&bucket, &key, metadata).await?;

    let response = xml::InitiateMultipartUploadResult {
        bucket: bucket.clone(),
        key: key.clone(),
        upload_id,
    };

    let xml_body = xml::to_xml_string(&response)
        .map_err(|e| tinystore_shared::StorageError::SerializationError(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        xml_body,
    ))
}

/// Upload a part (PUT /{bucket}/{key}?partNumber={n}&uploadId={id})
pub async fn upload_part<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    body: Bytes,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    let part_number: u32 = params
        .get("partNumber")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| tinystore_shared::StorageError::InvalidObjectKey("Missing partNumber".to_string()))?;

    let upload_id = params
        .get("uploadId")
        .ok_or_else(|| tinystore_shared::StorageError::InvalidObjectKey("Missing uploadId".to_string()))?;

    let part_info = backend.upload_part(&bucket, &key, upload_id, part_number, body).await?;

    Ok((
        StatusCode::OK,
        [(header::ETAG, part_info.etag)],
        "",
    ))
}

/// Complete a multipart upload (POST /{bucket}/{key}?uploadId={id})
pub async fn complete_multipart_upload<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    body: Bytes,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    let upload_id = params
        .get("uploadId")
        .ok_or_else(|| tinystore_shared::StorageError::InvalidObjectKey("Missing uploadId".to_string()))?;

    // Parse the XML body
    let completed_parts = xml::parse_complete_multipart_upload(&body)
        .map_err(|e| tinystore_shared::StorageError::SerializationError(e.to_string()))?;

    let result = backend.complete_multipart_upload(&bucket, &key, upload_id, completed_parts).await?;

    let response = xml::CompleteMultipartUploadResult {
        location: format!("/{}/{}", bucket, key),
        bucket: bucket.clone(),
        key: key.clone(),
        etag: result.etag,
    };

    let xml_body = xml::to_xml_string(&response)
        .map_err(|e| tinystore_shared::StorageError::SerializationError(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        xml_body,
    ))
}

/// Abort a multipart upload (DELETE /{bucket}/{key}?uploadId={id})
pub async fn abort_multipart_upload<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    let upload_id = params
        .get("uploadId")
        .ok_or_else(|| tinystore_shared::StorageError::InvalidObjectKey("Missing uploadId".to_string()))?;

    backend.abort_multipart_upload(&bucket, &key, upload_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List parts (GET /{bucket}/{key}?uploadId={id})
pub async fn list_parts<B>(
    State(backend): State<Arc<B>>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    let upload_id = params
        .get("uploadId")
        .ok_or_else(|| tinystore_shared::StorageError::InvalidObjectKey("Missing uploadId".to_string()))?;

    let parts = backend.list_parts(&bucket, &key, upload_id).await?;

    let response = xml::ListPartsResult {
        bucket: bucket.clone(),
        key: key.clone(),
        upload_id: upload_id.clone(),
        parts: parts.into_iter().map(|p| xml::PartXml {
            part_number: p.part_number,
            etag: p.etag,
            size: p.size,
        }).collect(),
    };

    let xml_body = xml::to_xml_string(&response)
        .map_err(|e| tinystore_shared::StorageError::SerializationError(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        xml_body,
    ))
}

/// List multipart uploads (GET /{bucket}?uploads)
pub async fn list_multipart_uploads<B>(
    State(_backend): State<Arc<B>>,
    Path(bucket): Path<String>,
) -> S3Result<impl IntoResponse>
where
    B: StorageBackend,
{
    // Return empty list for now - full implementation requires tracking active uploads
    let response = xml::ListMultipartUploadsResult {
        bucket,
        uploads: vec![],
    };

    let xml_body = xml::to_xml_string(&response)
        .map_err(|e| tinystore_shared::StorageError::SerializationError(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        xml_body,
    ))
}
