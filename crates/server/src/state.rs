//! Application state

use std::sync::Arc;
use tinystore_storage::StorageBackend;
use tinystore_auth::CredentialStore;

/// Shared application state
#[derive(Clone)]
pub struct AppState<B>
where
    B: StorageBackend,
{
    /// Storage backend
    pub storage: Arc<B>,
    /// Credential store
    pub credentials: CredentialStore,
}

impl<B> AppState<B>
where
    B: StorageBackend,
{
    /// Create new application state
    pub fn new(storage: Arc<B>, credentials: CredentialStore) -> Self {
        Self {
            storage,
            credentials,
        }
    }
}
