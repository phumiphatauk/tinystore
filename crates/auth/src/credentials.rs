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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get_credentials() {
        let store = CredentialStore::new();
        let creds = Credentials::new(
            "test-access-key".to_string(),
            "test-secret-key".to_string(),
            false,
        );

        store.add(creds.clone()).await;

        let retrieved = store.get("test-access-key").await;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.access_key, "test-access-key");
        assert_eq!(retrieved.secret_key, "test-secret-key");
        assert!(!retrieved.is_admin);
    }

    #[tokio::test]
    async fn test_get_nonexistent_credentials() {
        let store = CredentialStore::new();
        let result = store.get("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_remove_credentials() {
        let store = CredentialStore::new();
        let creds = Credentials::new(
            "remove-me".to_string(),
            "secret".to_string(),
            false,
        );

        store.add(creds).await;
        assert!(store.get("remove-me").await.is_some());

        let removed = store.remove("remove-me").await;
        assert!(removed);
        assert!(store.get("remove-me").await.is_none());
    }

    #[tokio::test]
    async fn test_remove_nonexistent_credentials() {
        let store = CredentialStore::new();
        let removed = store.remove("nonexistent").await;
        assert!(!removed);
    }

    #[tokio::test]
    async fn test_list_keys() {
        let store = CredentialStore::new();

        store.add(Credentials::new("key1".to_string(), "secret1".to_string(), false)).await;
        store.add(Credentials::new("key2".to_string(), "secret2".to_string(), true)).await;
        store.add(Credentials::new("key3".to_string(), "secret3".to_string(), false)).await;

        let keys = store.list_keys().await;
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[tokio::test]
    async fn test_update_credentials() {
        let store = CredentialStore::new();

        // Add initial credentials
        store.add(Credentials::new("key1".to_string(), "secret1".to_string(), false)).await;

        // Update with new credentials (same access key)
        store.add(Credentials::new("key1".to_string(), "new-secret".to_string(), true)).await;

        let creds = store.get("key1").await.unwrap();
        assert_eq!(creds.secret_key, "new-secret");
        assert!(creds.is_admin);
    }

    #[tokio::test]
    async fn test_admin_flag() {
        let admin_creds = Credentials::new("admin".to_string(), "secret".to_string(), true);
        let user_creds = Credentials::new("user".to_string(), "secret".to_string(), false);

        assert!(admin_creds.is_admin);
        assert!(!user_creds.is_admin);
    }
}
