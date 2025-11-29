//! API client utilities

use serde::{Deserialize, Serialize};
use tinystore_shared::api::*;
use wasm_bindgen::JsValue;

/// Makes a GET request to the API
pub async fn get<T>(url: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let response = gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Makes a POST request to the API
pub async fn post<T, R>(url: &str, body: &T) -> Result<R, String>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let response = gloo_net::http::Request::post(url)
        .json(body)
        .map_err(|e| format!("Failed to serialize request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Makes a DELETE request to the API
pub async fn delete(url: &str) -> Result<(), String> {
    let response = gloo_net::http::Request::delete(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
    }

    Ok(())
}

/// Makes a PUT request to the API
pub async fn put(url: &str, body: Vec<u8>) -> Result<(), String> {
    // Convert Vec<u8> to Uint8Array for WASM
    let uint8_array = js_sys::Uint8Array::from(body.as_slice());
    let js_value: JsValue = uint8_array.into();

    let response = gloo_net::http::Request::put(url)
        .body(js_value)
        .map_err(|e| format!("Failed to prepare request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
    }

    Ok(())
}

/// Fetches server status
pub async fn fetch_server_status() -> Result<ServerStatus, String> {
    get("/api/status").await
}

/// Fetches storage statistics
pub async fn fetch_storage_stats() -> Result<StorageStats, String> {
    get("/api/stats").await
}

/// Fetches the list of buckets
pub async fn fetch_buckets() -> Result<ListBucketsResponse, String> {
    get("/api/buckets").await
}

/// Creates a new bucket
pub async fn create_bucket(name: String) -> Result<(), String> {
    let req = CreateBucketRequest { name };
    post::<_, ()>("/api/buckets", &req).await
}

/// Deletes a bucket
pub async fn delete_bucket(name: &str) -> Result<(), String> {
    delete(&format!("/api/buckets/{}", name)).await
}

/// Fetches objects in a bucket
pub async fn fetch_objects(bucket: &str, prefix: Option<&str>) -> Result<ListObjectsResponse, String> {
    let url = if let Some(p) = prefix {
        format!("/api/buckets/{}/objects?prefix={}", bucket, p)
    } else {
        format!("/api/buckets/{}/objects", bucket)
    };
    get(&url).await
}

/// Uploads an object to a bucket
pub async fn upload_object(bucket: &str, key: &str, data: Vec<u8>) -> Result<(), String> {
    let url = format!("/api/buckets/{}/objects/{}", bucket, key);
    put(&url, data).await
}

/// Deletes an object from a bucket
pub async fn delete_object(bucket: &str, key: &str) -> Result<(), String> {
    delete(&format!("/api/buckets/{}/objects/{}", bucket, key)).await
}

/// Fetches list of credentials
pub async fn fetch_credentials() -> Result<Vec<CredentialInfo>, String> {
    get("/api/credentials").await
}

/// Creates a new credential
pub async fn create_credential(req: CreateCredentialRequest) -> Result<(), String> {
    post::<_, ()>("/api/credentials", &req).await
}

/// Deletes a credential
pub async fn delete_credential(id: &str) -> Result<(), String> {
    delete(&format!("/api/credentials/{}", id)).await
}
