//! Axum middleware for authentication

use crate::{AuthError, CredentialStore, SignatureV4};
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use tracing::{debug, error, warn};

const DEFAULT_REGION: &str = "us-east-1";

/// Authentication layer for Axum
#[derive(Clone)]
pub struct AuthLayer {
    credential_store: CredentialStore,
    region: String,
    enabled: bool,
}

impl AuthLayer {
    /// Create a new authentication layer
    pub fn new(credential_store: CredentialStore, region: String, enabled: bool) -> Self {
        Self {
            credential_store,
            region,
            enabled,
        }
    }

    /// Create a new authentication layer with default settings
    pub fn with_defaults(credential_store: CredentialStore) -> Self {
        Self::new(credential_store, DEFAULT_REGION.to_string(), true)
    }

    /// Middleware function to authenticate requests
    pub async fn authenticate(
        credential_store: CredentialStore,
        region: String,
        enabled: bool,
        request: Request<Body>,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // If authentication is disabled, allow all requests
        if !enabled {
            debug!("Authentication disabled, allowing request");
            return Ok(next.run(request).await);
        }

        // Extract parts from request
        let (parts, body) = request.into_parts();
        let method = parts.method.as_str();
        let uri = parts.uri.path();
        let query_string = parts.uri.query().unwrap_or("");
        let headers = parts.headers.clone();

        // Check for authorization header
        let auth_header = match headers.get("authorization") {
            Some(h) => match h.to_str() {
                Ok(s) => s,
                Err(e) => {
                    error!("Invalid authorization header encoding: {}", e);
                    return Err(StatusCode::BAD_REQUEST);
                }
            },
            None => {
                warn!("Missing authorization header");
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

        // Extract access key from authorization header
        let access_key = match Self::extract_access_key(auth_header) {
            Ok(key) => key,
            Err(e) => {
                warn!("Failed to extract access key: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        // Get credentials from store
        let credentials = match credential_store.get(&access_key).await {
            Some(creds) => creds,
            None => {
                warn!("Unknown access key: {}", access_key);
                return Err(StatusCode::FORBIDDEN);
            }
        };

        // Get payload hash from headers
        let payload_hash = headers
            .get("x-amz-content-sha256")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("UNSIGNED-PAYLOAD");

        // Convert headers to HashMap for signature verification
        let header_map = Self::headers_to_map(&headers);

        // Verify signature
        match SignatureV4::verify_signature(
            auth_header,
            method,
            uri,
            query_string,
            &header_map,
            payload_hash,
            &credentials.secret_key,
            &region,
        ) {
            Ok(true) => {
                debug!("Request authenticated successfully for user: {}", access_key);
                // Reconstruct request and continue
                let request = Request::from_parts(parts, body);
                Ok(next.run(request).await)
            }
            Ok(false) => {
                warn!("Signature verification failed for user: {}", access_key);
                Err(StatusCode::FORBIDDEN)
            }
            Err(e) => {
                error!("Error verifying signature: {} (code: {})", e, e.s3_code());
                // Map auth error to appropriate HTTP status code
                Err(StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::FORBIDDEN))
            }
        }
    }

    /// Extract access key from authorization header
    fn extract_access_key(auth_header: &str) -> Result<String, String> {
        // Format: AWS4-HMAC-SHA256 Credential=ACCESS_KEY/date/region/service/aws4_request, ...
        let credential_part = auth_header
            .split("Credential=")
            .nth(1)
            .ok_or("Missing Credential in authorization header")?;

        let access_key = credential_part
            .split('/')
            .next()
            .ok_or("Invalid credential format")?;

        Ok(access_key.to_string())
    }

    /// Convert HeaderMap to HashMap
    fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (name, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                map.insert(name.as_str().to_string(), value_str.to_string());
            }
        }
        map
    }
}

impl Default for AuthLayer {
    fn default() -> Self {
        Self::new(CredentialStore::new(), DEFAULT_REGION.to_string(), true)
    }
}
