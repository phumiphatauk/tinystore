//! CLI argument parsing

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tinystore")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the TinyStore server
    Serve {
        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Port to listen on
        #[arg(short, long, default_value = "9000")]
        port: u16,

        /// Data directory path
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,

        /// Access key for authentication
        #[arg(long, env = "TINYSTORE_ACCESS_KEY")]
        access_key: Option<String>,

        /// Secret key for authentication
        #[arg(long, env = "TINYSTORE_SECRET_KEY")]
        secret_key: Option<String>,

        /// Disable web UI
        #[arg(long)]
        no_ui: bool,

        /// Log level
        #[arg(long, default_value = "info")]
        log_level: String,
    },

    /// Generate example configuration file
    InitConfig,

    /// Validate configuration file
    ValidateConfig {
        /// Path to configuration file
        #[arg(short, long)]
        config: PathBuf,
    },
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}
