//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 9000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: String,
    pub data_dir: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: "filesystem".to_string(),
            data_dir: PathBuf::from("./data"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub enabled: bool,
    pub path: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/ui".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
    pub ui: UiConfig,
}

impl Config {
    /// Load configuration from file with environment variable overrides
    pub fn load(path: Option<&std::path::Path>) -> Result<Self, config::ConfigError> {
        let mut config = if let Some(p) = path {
            let content = std::fs::read_to_string(p)
                .map_err(|e| config::ConfigError::Message(e.to_string()))?;
            serde_yaml::from_str(&content)
                .map_err(|e| config::ConfigError::Message(e.to_string()))?
        } else {
            Config::default()
        };

        // Apply environment variable overrides
        if let Ok(host) = std::env::var("TINYSTORE_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("TINYSTORE_PORT") {
            if let Ok(p) = port.parse() {
                config.server.port = p;
            }
        }
        if let Ok(data_dir) = std::env::var("TINYSTORE_DATA_DIR") {
            config.storage.data_dir = std::path::PathBuf::from(data_dir);
        }

        Ok(config)
    }
}
