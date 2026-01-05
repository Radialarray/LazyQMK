//! Static file serving for LazyQMK Web UI.
//!
//! This module provides embedded static file serving with SPA fallback support.
//! In release builds, static files are embedded directly in the binary for
//! easy distribution. In development, files can be served from disk.

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;
use std::path::PathBuf;

/// Embedded static files from the web frontend build.
///
/// The files are embedded at compile time from the `web/build` directory.
/// If the directory doesn't exist at compile time, the struct will be empty
/// and static serving will return 404 for all requests.
#[derive(Embed)]
#[folder = "web/build"]
#[include = "*.html"]
#[include = "*.js"]
#[include = "*.css"]
#[include = "*.json"]
#[include = "*.png"]
#[include = "*.ico"]
#[include = "*.svg"]
#[include = "*.woff"]
#[include = "*.woff2"]
#[include = "*.ttf"]
#[include = "_app/*"]
#[include = "_app/**/*"]
pub struct StaticAssets;

/// Serves static files with SPA fallback.
///
/// This handler:
/// 1. First tries to serve the exact requested path
/// 2. If not found, tries adding `.html` extension
/// 3. If still not found and path doesn't look like a file, serves `index.html` (SPA fallback)
///
/// # Arguments
///
/// * `request` - The incoming HTTP request
///
/// # Returns
///
/// The static file content or appropriate error response.
pub async fn serve_static(request: Request) -> Response {
    let path = request.uri().path();

    // Remove leading slash for embed lookup
    let path = path.trim_start_matches('/');

    // If path is empty or root, serve index.html
    if path.is_empty() {
        return serve_file("index.html");
    }

    // Try to serve the exact path first
    if let Some(content) = StaticAssets::get(path) {
        return file_response(path, content.data.as_ref());
    }

    // Try with .html extension for clean URLs
    let html_path = format!("{path}.html");
    if let Some(content) = StaticAssets::get(&html_path) {
        return file_response(&html_path, content.data.as_ref());
    }

    // Check if this looks like a file request (has extension)
    let looks_like_file = PathBuf::from(path)
        .extension()
        .is_some_and(|ext| !ext.is_empty());

    // If it looks like a file but wasn't found, return 404
    if looks_like_file {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    // SPA fallback: serve index.html for all other routes
    serve_file("index.html")
}

/// Serves a specific file from embedded assets.
fn serve_file(path: &str) -> Response {
    match StaticAssets::get(path) {
        Some(content) => file_response(path, content.data.as_ref()),
        None => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

/// Creates an HTTP response for a file with appropriate content type.
fn file_response(path: &str, content: &[u8]) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .header(header::CACHE_CONTROL, cache_control_for_path(path))
        .body(Body::from(content.to_vec()))
        .unwrap_or_else(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create response",
            )
                .into_response()
        })
}

/// Returns appropriate Cache-Control header based on file path.
///
/// - Immutable assets (with content hash in filename): long cache (1 year)
/// - HTML files: no cache (always revalidate)
/// - Other files: short cache (1 hour)
fn cache_control_for_path(path: &str) -> &'static str {
    // SvelteKit generates immutable assets with hashes in _app directory
    if path.starts_with("_app/") {
        "public, max-age=31536000, immutable"
    } else if std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("html"))
    {
        "no-cache, must-revalidate"
    } else {
        "public, max-age=3600"
    }
}

/// Returns true if embedded assets are available.
///
/// This can be used to check if the web UI was built and embedded
/// before attempting to serve it.
#[must_use]
pub fn has_embedded_assets() -> bool {
    // Check if we have at least the index.html file
    StaticAssets::get("index.html").is_some()
}

/// Lists all embedded asset paths for debugging.
#[cfg(test)]
pub fn list_embedded_assets() -> Vec<String> {
    StaticAssets::iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_control_for_path() {
        // Immutable assets get long cache
        assert_eq!(
            cache_control_for_path("_app/immutable/chunks/0.abc123.js"),
            "public, max-age=31536000, immutable"
        );

        // HTML gets no-cache
        assert_eq!(
            cache_control_for_path("index.html"),
            "no-cache, must-revalidate"
        );
        assert_eq!(
            cache_control_for_path("about.html"),
            "no-cache, must-revalidate"
        );

        // Other files get short cache
        assert_eq!(
            cache_control_for_path("favicon.png"),
            "public, max-age=3600"
        );
        assert_eq!(
            cache_control_for_path("manifest.json"),
            "public, max-age=3600"
        );
    }

    #[test]
    fn test_has_embedded_assets() {
        // This test checks the function works - result depends on whether
        // web/build exists at compile time
        let _ = has_embedded_assets();
    }
}
