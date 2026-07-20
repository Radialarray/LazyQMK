//! Static file serving and SPA fallback for the web API.
//!
//! Extracted from src/web/mod.rs as part of LazyQMK-2rf6.2.

use axum::{
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};

/// Embedded static files from the web frontend build.
#[derive(rust_embed::RustEmbed)]
#[folder = "web/build/"]
struct Assets;

/// Serves static files from the embedded assets.
///
/// Returns the file content with appropriate MIME type headers,
/// or None if the file is not found.
fn serve_static(uri: &str) -> Option<Response> {
    // Remove leading slash
    let path = uri.trim_start_matches('/');

    // Try to get the file from embedded assets
    let file = Assets::get(path)?;

    // Guess MIME type from file extension
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .as_ref()
        .to_string();

    // Build response with appropriate headers
    Some(([(header::CONTENT_TYPE, mime_type)], file.data).into_response())
}

/// Fallback handler for SPA routing.
///
/// Serves index.html for any non-API routes that don't match static files.
/// This enables client-side routing in the SPA.
fn spa_fallback() -> Response {
    // Try to get index.html from embedded assets
    if let Some(index) = Assets::get("index.html") {
        Html(index.data).into_response()
    } else {
        // If index.html is not embedded (dev mode), return 404
        (
            StatusCode::NOT_FOUND,
            "Frontend not embedded - use Vite dev server",
        )
            .into_response()
    }
}

/// Handles static file requests and SPA fallback.
///
/// First tries to serve the requested file from embedded assets.
/// If not found and it's not an API route, serves index.html for SPA routing.
pub async fn static_handler(uri: axum::http::Uri) -> Response {
    let path = uri.path();

    // Try to serve the static file first
    if let Some(response) = serve_static(path) {
        return response;
    }

    // If file not found and path doesn't start with /api, serve index.html (SPA fallback)
    if !path.starts_with("/api") {
        return spa_fallback();
    }

    // For API routes that don't match, return 404
    (StatusCode::NOT_FOUND, "Not found").into_response()
}