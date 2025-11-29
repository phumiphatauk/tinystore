//! Application state for SSR

#[cfg(feature = "ssr")]
use tinystore_storage::StorageBackend;
#[cfg(feature = "ssr")]
use std::sync::Arc;

/// Shared application state (SSR only)
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AppState<B: StorageBackend> {
    pub storage: Arc<B>,
    pub credentials: tinystore_auth::CredentialStore,
}
