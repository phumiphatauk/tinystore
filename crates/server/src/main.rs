//! TinyStore - Lightweight S3-compatible object storage server

mod config;
mod cli;
mod state;
mod ui_router;
mod api;

use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::info;

use tinystore_storage::FilesystemBackend;
use tinystore_auth::CredentialStore;
use tinystore_s3_api::create_s3_router;
use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    info!("Starting TinyStore v{}", env!("CARGO_PKG_VERSION"));

    // Parse CLI arguments
    let cli = cli::parse_cli();

    match cli.command {
        cli::Commands::Serve {
            host,
            port,
            data_dir,
            access_key,
            secret_key,
            no_ui,
            ..
        } => {
            // Create storage backend
            let storage = Arc::new(FilesystemBackend::new(data_dir.clone()));
            info!("Storage backend initialized at {:?}", data_dir);

            // Create credential store
            let credentials = CredentialStore::new();
            if let (Some(access), Some(secret)) = (access_key, secret_key) {
                credentials.add(tinystore_auth::Credentials::new(access, secret, true)).await;
                info!("Authentication enabled");
            } else {
                credentials.add(tinystore_auth::Credentials::new("tinystore".to_string(), "tinystore123".to_string(), true)).await;
                info!("Using default credentials (tinystore/tinystore123)");
            }

            // Create application state
            let app_state = AppState::new(storage.clone(), credentials.clone());

            // Create API state with start time tracking
            let api_state = api::ApiState {
                storage: storage.clone(),
                credentials: credentials.clone(),
                start_time: std::time::Instant::now(),
            };

            // Create S3 API router
            let s3_router = create_s3_router(storage.clone());
            info!("S3 API router initialized");

            // Create JSON API router
            let api_router = api::create_api_router(api_state);
            info!("JSON API router initialized");

            // Build the main application router
            let app = Router::new();

            // Add JSON API routes
            let app = app.nest("/api/v1", api_router);

            // Add S3 API routes
            let app = app.nest("/", s3_router);

            // Add UI routes if enabled
            let app = if !no_ui {
                info!("Web UI enabled at /ui");
                app.nest("/ui", ui_router::create_ui_router(app_state).await)
                    .nest_service("/pkg", ServeDir::new("public/pkg"))
                    .nest_service("/assets", ServeDir::new("public/assets"))
            } else {
                info!("Web UI disabled");
                app
            };

            // Add middleware
            let app = app.layer(TraceLayer::new_for_http());

            // Start the server
            let addr = format!("{}:{}", host, port);
            let listener = tokio::net::TcpListener::bind(&addr).await?;

            info!("TinyStore is ready!");
            info!("S3 API listening on http://{}", addr);
            if !no_ui {
                info!("Web UI available at http://{}/ui", addr);
            }

            axum::serve(listener, app).await?;
        }
        cli::Commands::InitConfig => {
            info!("Generating example configuration...");
            let config = config::Config::default();
            let yaml = serde_yaml::to_string(&config)?;
            std::fs::write("config.yaml", yaml)?;
            info!("Configuration written to config.yaml");
        }
        cli::Commands::ValidateConfig { config: config_path } => {
            info!("Validating configuration at {:?}", config_path);
            let content = std::fs::read_to_string(config_path)?;
            let _config: config::Config = serde_yaml::from_str(&content)?;
            info!("Configuration is valid!");
        }
    }

    Ok(())
}
