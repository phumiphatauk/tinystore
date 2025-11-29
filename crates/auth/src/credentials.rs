//! Credential management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Credentials for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_key: String,
    pub secret_key: String,
    pub is_admin: bool,
}

impl Credentials {
    /// Create new credentials
    pub fn new(access_key: String, secret_key: String, is_admin: bool) -> Self {
        Self {
            access_key,
            secret_key,
            is_admin,
        }
    }
}

/// Store for managing credentials
#[derive(Debug, Clone)]
pub struct CredentialStore {
    credentials: Arc<RwLock<HashMap<String, Credentials>>>,
}

impl CredentialStore {
    /// Create a new credential store
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add credentials to the store
    pub async fn add(&self, creds: Credentials) {
        let mut store = self.credentials.write().await;
        store.insert(creds.access_key.clone(), creds);
    }

    /// Get credentials by access key
    pub async fn get(&self, access_key: &str) -> Option<Credentials> {
        let store = self.credentials.read().await;
        store.get(access_key).cloned()
    }

    /// Remove credentials by access key
    pub async fn remove(&self, access_key: &str) -> bool {
        let mut store = self.credentials.write().await;
        store.remove(access_key).is_some()
    }

    /// List all access keys
    pub async fn list_keys(&self) -> Vec<String> {
        let store = self.credentials.read().await;
        store.keys().cloned().collect()
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}
