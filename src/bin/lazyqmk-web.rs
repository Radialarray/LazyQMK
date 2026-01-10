//! LazyQMK Web Server Binary
//!
//! This binary starts the LazyQMK web server that provides a REST API
//! for the web-based layout editor frontend.
//!
//! # Usage
//!
//! ```bash
//! # Start with default settings (port 3001, uses ~/.config/LazyQMK/layouts/)
//! lazyqmk-web
//!
//! # Specify port and workspace
//! lazyqmk-web --port 8080 --workspace ~/my-layouts
//! ```

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use lazyqmk::config::Config;
use lazyqmk::web;

/// LazyQMK Web Server - REST API for the layout editor
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "LazyQMK Web Server - Browser-based keyboard layout editor",
    long_about = "\
LazyQMK Web Server - Browser-based keyboard layout editor

DESCRIPTION:
  Provides a modern browser-based interface for editing QMK keyboard layouts
  with full feature parity to the terminal UI. Includes REST API backend and
  embedded web frontend in a single binary.

USAGE EXAMPLES:
  Start with default settings:
    lazyqmk-web
  
  Custom port and host:
    lazyqmk-web --port 8080 --host 0.0.0.0
  
  Custom workspace directory:
    lazyqmk-web --workspace ~/my-layouts
  
  Enable verbose logging:
    lazyqmk-web --verbose

ACCESSING THE WEB EDITOR:
  After starting the server, open your browser to:
    http://localhost:3001 (default)
  
  Or your custom host/port:
    http://localhost:8080 (if using --port 8080)

FEATURES:
  - Visual keyboard layout editor
  - Layer management and color coding
  - Category system for key organization
  - Firmware generation and compilation
  - Build history with artifact management
  - Real-time build logs with SSE streaming
  - Template system for layout sharing
  - Dark mode support
  
  For detailed feature documentation, see: docs/WEB_FEATURES.md

BUILDING FROM SOURCE:
  This binary requires the 'web' feature flag:
    cargo build --features web --release --bin lazyqmk-web

SECURITY:
  By default, the server binds to 127.0.0.1 (localhost only) for security.
  To expose the server to your local network:
    lazyqmk-web --host 0.0.0.0
  
  For production deployment with HTTPS and authentication, see:
    docs/WEB_DEPLOYMENT.md
"
)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "3001")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Workspace directory containing layout files.
    /// Defaults to the platform-specific layouts directory:
    /// - Linux: ~/.config/LazyQMK/layouts/
    /// - macOS: ~/Library/Application Support/LazyQMK/layouts/
    /// - Windows: %APPDATA%\LazyQMK\layouts\
    #[arg(short, long)]
    workspace: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

/// Gets the default layouts directory, creating it if it doesn't exist.
fn get_default_layouts_dir() -> anyhow::Result<PathBuf> {
    let layouts_dir = Config::config_dir()?.join("layouts");

    // Create directory if it doesn't exist
    if !layouts_dir.exists() {
        std::fs::create_dir_all(&layouts_dir).context(format!(
            "Failed to create layouts directory: {}",
            layouts_dir.display()
        ))?;
    }

    Ok(layouts_dir)
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

    // Determine workspace root:
    // 1. Use --workspace if provided
    // 2. Otherwise, use Config::config_dir()/layouts (same as TUI)
    let workspace_root = match args.workspace {
        Some(path) => path,
        None => get_default_layouts_dir()?,
    };

    info!("Workspace root: {}", workspace_root.display());

    // Build socket address
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;

    // Start the server
    web::run_server(config, workspace_root, addr).await
}
