//! LazyQMK Web Server Binary
//!
//! This binary starts the LazyQMK web server that provides a REST API
//! for the web-based layout editor frontend.
//!
//! # Usage
//!
//! ```bash
//! # Start with default settings (port 3001, current directory)
//! lazyqmk-web
//!
//! # Specify port and workspace
//! lazyqmk-web --port 8080 --workspace ~/my-layouts
//! ```

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use lazyqmk::config::Config;
use lazyqmk::web;

/// LazyQMK Web Server - REST API for the layout editor
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "3001")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Workspace directory containing layout files
    #[arg(short, long)]
    workspace: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let filter = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load or create configuration
    let config = Config::load().unwrap_or_default();

    // Determine workspace root
    let workspace_root = args
        .workspace
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    info!("Workspace root: {}", workspace_root.display());

    // Build socket address
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;

    // Start the server
    web::run_server(config, workspace_root, addr).await
}
