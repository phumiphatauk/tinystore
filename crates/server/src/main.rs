//! TinyStore - Lightweight S3-compatible object storage server

mod config;
mod cli;
mod state;

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

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

    // TODO: Load configuration in later steps
    // TODO: Create storage backend in later steps
    // TODO: Set up S3 API router in later steps
    // TODO: Set up Leptos UI router in later steps
    // TODO: Start server in later steps

    info!("TinyStore is ready!");
    info!("Workspace structure is set up successfully");

    Ok(())
}
