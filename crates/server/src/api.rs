//! JSON API endpoints for the Web UI

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tinystore_auth::CredentialStore;
use tinystore_storage::StorageBackend;

#[derive(Clone)]
pub struct ApiState<B: StorageBackend> {
    pub storage: Arc<B>,
    pub credentials: CredentialStore,
    pub start_time: Instant,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

#[derive(Serialize)]
pub struct ServerStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
}

#[derive(Serialize)]
pub struct StorageStats {
    pub total_buckets: u64,
    pub total_objects: u64,
    pub total_size_bytes: u64,
}

#[derive(Serialize)]
pub struct BucketInfo {
    pub name: String,
    pub created: String,
    pub object_count: u64,
}

#[derive(Deserialize)]
pub struct CreateBucketRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct ObjectInfo {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
    pub etag: String,
}

#[derive(Serialize)]
pub struct CredentialInfo {
    pub id: String,
    pub access_key: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct CreateCredentialRequest {
    pub description: String,
}

pub async fn get_status<B: StorageBackend>(
    State(state): State<ApiState<B>>,
) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();

    // Get memory usage (Linux-specific)
    let memory_mb = get_memory_usage_mb();

    let status = ServerStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        memory_usage_mb: memory_mb,
    };

    Json(ApiResponse::success(status))
}

pub async fn get_stats<B: StorageBackend>(
    State(state): State<ApiState<B>>,
) -> impl IntoResponse {
    match state.storage.get_stats().await {
        Ok(stats) => Json(ApiResponse::success(StorageStats {
            total_buckets: stats.total_buckets,
            total_objects: stats.total_objects,
            total_size_bytes: stats.total_size_bytes,
        })),
        Err(e) => Json(ApiResponse::<StorageStats>::error(e.to_string())),
    }
}

pub async fn list_buckets<B: StorageBackend>(
    State(state): State<ApiState<B>>,
) -> impl IntoResponse {
    match state.storage.list_buckets().await {
        Ok(buckets) => {
            let bucket_infos: Vec<BucketInfo> = buckets
                .into_iter()
                .map(|b| BucketInfo {
                    name: b.name,
                    created: b.creation_date.to_rfc3339(),
                    object_count: 0, // TODO: Implement object counting
                })
                .collect();
            Json(ApiResponse::success(bucket_infos))
        }
        Err(e) => Json(ApiResponse::<Vec<BucketInfo>>::error(e.to_string())),
    }
}

pub async fn create_bucket<B: StorageBackend>(
    State(state): State<ApiState<B>>,
    Json(req): Json<CreateBucketRequest>,
) -> impl IntoResponse {
    match state.storage.create_bucket(&req.name).await {
        Ok(_) => (StatusCode::CREATED, Json(ApiResponse::success(()))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e.to_string())),
        ),
    }
}

pub async fn delete_bucket<B: StorageBackend>(
    State(state): State<ApiState<B>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.storage.delete_bucket(&name).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(()))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e.to_string())),
        ),
    }
}

pub async fn list_objects<B: StorageBackend>(
    State(state): State<ApiState<B>>,
    Path(bucket): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let list_params = tinystore_storage::ListObjectsParams {
        prefix: params.get("prefix").cloned(),
        delimiter: params.get("delimiter").cloned(),
        max_keys: params
            .get("max_keys")
            .and_then(|s| s.parse::<usize>().ok()),
        continuation_token: params.get("continuation_token").cloned(),
    };

    match state.storage.list_objects(&bucket, list_params).await {
        Ok(result) => {
            let object_infos: Vec<ObjectInfo> = result
                .objects
                .into_iter()
                .map(|o| ObjectInfo {
                    key: o.key,
                    size: o.size,
                    last_modified: o.last_modified.to_rfc3339(),
                    etag: o.etag,
                })
                .collect();
            Json(ApiResponse::success(object_infos))
        }
        Err(e) => Json(ApiResponse::<Vec<ObjectInfo>>::error(e.to_string())),
    }
}

pub async fn upload_object<B: StorageBackend>(
    State(state): State<ApiState<B>>,
    Path((bucket, key)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let etag = format!("{:x}", md5::compute(&body));
    let metadata = tinystore_shared::ObjectMetadata::new(body.len() as u64, etag);

    match state.storage.put_object(&bucket, &key, body, metadata).await {
        Ok(_) => (StatusCode::CREATED, Json(ApiResponse::success(()))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e.to_string())),
        ),
    }
}

pub async fn delete_object<B: StorageBackend>(
    State(state): State<ApiState<B>>,
    Path((bucket, key)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.storage.delete_object(&bucket, &key).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(()))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e.to_string())),
        ),
    }
}

pub async fn list_credentials<B: StorageBackend>(
    State(state): State<ApiState<B>>,
) -> impl IntoResponse {
    let keys = state.credentials.list_keys().await;
    let mut cred_infos = Vec::new();

    for key in keys {
        if let Some(cred) = state.credentials.get(&key).await {
            cred_infos.push(CredentialInfo {
                id: cred.access_key.clone(),
                access_key: cred.access_key,
                description: String::new(),
            });
        }
    }

    (StatusCode::OK, Json(ApiResponse::success(cred_infos)))
}

pub async fn create_credential<B: StorageBackend>(
    State(state): State<ApiState<B>>,
    Json(req): Json<CreateCredentialRequest>,
) -> impl IntoResponse {
    // Generate new credentials
    use uuid::Uuid;
    let access_key = format!("AKIA{}", Uuid::new_v4().simple());
    let secret_key = Uuid::new_v4().simple().to_string();

    let cred = tinystore_auth::Credentials::new(access_key.clone(), secret_key.clone(), false);
    state.credentials.add(cred).await;

    let cred_info = CredentialInfo {
        id: access_key.clone(),
        access_key,
        description: req.description,
    };

    (StatusCode::CREATED, Json(ApiResponse::success(cred_info)))
}

pub async fn delete_credential<B: StorageBackend>(
    State(state): State<ApiState<B>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let removed = state.credentials.remove(&id).await;
    if removed {
        (StatusCode::OK, Json(ApiResponse::success(())))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Credential not found".to_string())),
        )
    }
}

pub fn create_api_router<B: StorageBackend + Clone + 'static>(state: ApiState<B>) -> Router {
    Router::new()
        .route("/status", get(get_status::<B>))
        .route("/stats", get(get_stats::<B>))
        .route("/buckets", get(list_buckets::<B>).post(create_bucket::<B>))
        .route("/buckets/:name", delete(delete_bucket::<B>))
        .route("/buckets/:name/objects", get(list_objects::<B>))
        .route(
            "/buckets/:bucket/objects/*key",
            put(upload_object::<B>).delete(delete_object::<B>),
        )
        .route(
            "/credentials",
            get(list_credentials::<B>).post(create_credential::<B>),
        )
        .route("/credentials/:id", delete(delete_credential::<B>))
        .with_state(state)
}

fn get_memory_usage_mb() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/self/statm") {
            if let Some(pages) = content.split_whitespace().next() {
                if let Ok(pages) = pages.parse::<u64>() {
                    return (pages * 4096) as f64 / 1024.0 / 1024.0;
                }
            }
        }
    }
    0.0
}
