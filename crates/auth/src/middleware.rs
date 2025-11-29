//! Axum middleware for authentication

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Authentication layer for Axum
#[derive(Clone)]
pub struct AuthLayer {
    // TODO: Add credential store and configuration in Step 5
}

impl AuthLayer {
    /// Create a new authentication layer
    pub fn new() -> Self {
        Self {}
    }

    /// Middleware function to authenticate requests
    pub async fn authenticate(
        _request: Request<Body>,
        _next: Next,
    ) -> Result<Response, StatusCode> {
        // TODO: Implement in Step 5
        todo!("Implement in Step 5")
    }
}

impl Default for AuthLayer {
    fn default() -> Self {
        Self::new()
    }
}
