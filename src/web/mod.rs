//! Web API module for `LazyQMK`.
//!
//! This module provides a REST API for the `LazyQMK` layout editor,
//! enabling a web-based frontend to interact with layout files,
//! keycodes, and QMK firmware configuration.
//!
//! # Endpoints
//!
//! - `GET /health` - Health check
//! - `GET /api/layouts` - List layout files
//! - `GET /api/layouts/{filename}` - Load and parse a layout file
//! - `PUT /api/layouts/{filename}` - Save a layout file
//! - `POST /api/layouts/{filename}/swap-keys` - Swap two keys in a layout
//! - `POST /api/layouts/{filename}/generate` - Generate firmware and start job
//! - `POST /api/layouts/{filename}/save-as-template` - Save layout as template
//! - `GET /api/layouts/{filename}/render-metadata` - Get key display metadata for rendering
//! - `GET /api/templates` - List available templates
//! - `GET /api/templates/{filename}` - Get a specific template
//! - `POST /api/templates/{filename}/apply` - Apply template to create new layout
//! - `GET /api/keycodes` - Query keycode database (optional ?search=)
//! - `GET /api/keycodes/categories` - List keycode categories
//! - `GET /api/config` - Get current configuration
//! - `PUT /api/config` - Update configuration
//! - `GET /api/preflight` - Check application state for onboarding flow
//! - `GET /api/keyboards/{keyboard}/geometry/{layout}` - Get keyboard geometry
//! - `POST /api/build/start` - Start a firmware build job
//! - `GET /api/build/jobs` - List all build jobs
//! - `GET /api/build/jobs/{job_id}` - Get build job status
//! - `GET /api/build/jobs/{job_id}/logs` - Get build job logs
//! - `POST /api/build/jobs/{job_id}/cancel` - Cancel a build job
//! - `GET /api/build/jobs/{job_id}/artifacts` - List build artifacts
//! - `GET /api/build/jobs/{job_id}/artifacts/{artifact_id}/download` - Download build artifact
//! - `GET /api/generate/jobs` - List all generate jobs
//! - `GET /api/generate/jobs/{job_id}` - Get generate job status
//! - `GET /api/generate/jobs/{job_id}/logs` - Get generate job logs
//! - `POST /api/generate/jobs/{job_id}/cancel` - Cancel a generate job
//! - `GET /api/generate/jobs/{job_id}/download` - Download generated zip file
//! - `GET /api/generate/health` - Get generate job system health status

pub mod app_state;
pub mod build_jobs;
pub mod dto;
pub mod error;
pub mod generate_jobs;
pub mod routes;
pub mod static_files;
pub mod validation;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use tracing::info;

use crate::config::Config;

pub use app_state::AppState;
pub use error::{ApiError, AppError};

pub use crate::web::dto::*;

use static_files::static_handler;
#[cfg(test)]
use validation::{validate_filename, validate_keyboard_path};

/// Creates the API router with all endpoints.
///
/// Wires the per-prefix route modules in [`routes`] and adds the static-file
/// fallback (SPA routing) at the end.
pub fn create_router(state: AppState) -> Router {
    routes::router(state).fallback(static_handler)
}

/// Runs the web server.
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `workspace_root` - Directory containing layout files
/// * `addr` - Socket address to bind to
///
/// # Errors
///
/// Returns an error if the server fails to start.
pub async fn run_server(
    config: Config,
    workspace_root: PathBuf,
    addr: SocketAddr,
) -> anyhow::Result<()> {
    let state = AppState::new(config, workspace_root)?;
    let app = create_router(state);

    info!("Starting LazyQMK web server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_filename_valid() {
        assert!(validate_filename("my_layout.md").is_ok());
        assert!(validate_filename("layout123").is_ok());
        assert!(validate_filename("test-layout.md").is_ok());
    }

    #[test]
    fn test_validate_filename_path_traversal() {
        assert!(validate_filename("../secret.md").is_err());
        assert!(validate_filename("foo/../bar.md").is_err());
        assert!(validate_filename("..").is_err());
    }

    #[test]
    fn test_validate_filename_absolute_path() {
        assert!(validate_filename("/etc/passwd").is_err());
        assert!(validate_filename("\\Windows\\System32").is_err());
    }

    #[test]
    fn test_validate_filename_hidden_files() {
        assert!(validate_filename(".hidden").is_err());
        assert!(validate_filename(".env").is_err());
    }

    #[test]
    fn test_validate_filename_empty() {
        assert!(validate_filename("").is_err());
    }

    #[test]
    fn test_validate_keyboard_path_valid() {
        assert!(validate_keyboard_path("crkbd").is_ok());
        assert!(validate_keyboard_path("splitkb/halcyon/corne").is_ok());
        assert!(validate_keyboard_path("keebart/corne_choc_pro").is_ok());
    }

    #[test]
    fn test_validate_keyboard_path_traversal() {
        assert!(validate_keyboard_path("../secret").is_err());
        assert!(validate_keyboard_path("foo/../bar").is_err());
    }

    #[test]
    fn test_validate_keyboard_path_absolute() {
        assert!(validate_keyboard_path("/etc/keyboard").is_err());
    }

    #[test]
    fn test_validate_keyboard_path_empty() {
        assert!(validate_keyboard_path("").is_err());
    }
}
