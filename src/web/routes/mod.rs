//! Route modules and router wiring.
//!
//! Each submodule contains the handlers for a group of related endpoints.
//! This module exposes a single `router()` function that wires all handlers.

pub mod build;
pub mod config;
pub mod export;
pub mod generate;
pub mod geometry;
pub mod health;
pub mod inspect;
pub mod keycodes;
pub mod layouts;
pub mod templates;
pub mod validate;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use super::static_files::static_handler;
use super::AppState;

/// Creates the API router with all endpoints.
pub fn router(state: AppState) -> Router {
    // CORS configuration - allow all origins for development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        // Effects
        .route("/api/effects", get(health::list_effects))
        // Layout endpoints
        .route("/api/layouts", get(layouts::list_layouts))
        .route("/api/layouts/{filename}", get(layouts::get_layout).put(layouts::save_layout))
        .route(
            "/api/layouts/{filename}/swap-keys",
            axum::routing::post(layouts::swap_keys),
        )
        .route("/api/layouts/{filename}/validate", get(validate::validate_layout))
        .route("/api/layouts/{filename}/inspect", get(inspect::inspect_layout))
        .route("/api/layouts/{filename}/export", get(export::export_layout))
        .route(
            "/api/layouts/{filename}/render-metadata",
            get(layouts::get_render_metadata),
        )
        .route(
            "/api/layouts/{filename}/generate",
            axum::routing::post(generate::generate_firmware),
        )
        .route(
            "/api/layouts/{filename}/save-as-template",
            axum::routing::post(layouts::save_as_template),
        )
        // Template endpoints
        .route("/api/templates", get(templates::list_templates))
        .route("/api/templates/{filename}", get(templates::get_template))
        .route(
            "/api/templates/{filename}/apply",
            axum::routing::post(templates::apply_template),
        )
        // Keycode endpoints
        .route("/api/keycodes", get(keycodes::list_keycodes))
        .route("/api/keycodes/categories", get(keycodes::list_categories))
        // Config endpoints
        .route("/api/config", get(config::get_config).put(config::update_config))
        // Preflight endpoint for onboarding
        .route("/api/preflight", get(config::get_preflight))
        // Geometry endpoint
        .route(
            "/api/keyboards/{keyboard}/geometry/{layout}",
            get(geometry::get_geometry),
        )
        // Keyboard & Setup Wizard endpoints
        .route("/api/keyboards", get(geometry::list_keyboards))
        .route(
            "/api/keyboards/{keyboard}/layouts",
            get(geometry::list_keyboard_layouts),
        )
        .route("/api/layouts", axum::routing::post(geometry::create_layout))
        .route(
            "/api/layouts/{filename}/switch-variant",
            axum::routing::post(geometry::switch_layout_variant),
        )
        // Build job endpoints
        .route("/api/build/start", axum::routing::post(build::start_build))
        .route("/api/build/jobs", get(build::list_build_jobs))
        .route("/api/build/jobs/{job_id}", get(build::get_build_job))
        .route("/api/build/jobs/{job_id}/logs", get(build::get_build_logs))
        .route(
            "/api/build/jobs/{job_id}/cancel",
            axum::routing::post(build::cancel_build_job),
        )
        .route(
            "/api/build/jobs/{job_id}/artifacts",
            get(build::get_build_artifacts),
        )
        .route(
            "/api/build/jobs/{job_id}/artifacts/{artifact_id}/download",
            get(build::download_build_artifact),
        )
        // Generate job endpoints
        .route("/api/generate/jobs", get(generate::list_generate_jobs))
        .route("/api/generate/jobs/{job_id}", get(generate::get_generate_job))
        .route("/api/generate/jobs/{job_id}/logs", get(generate::get_generate_logs))
        .route(
            "/api/generate/jobs/{job_id}/cancel",
            axum::routing::post(generate::cancel_generate_job),
        )
        .route(
            "/api/generate/jobs/{job_id}/download",
            get(generate::download_generate_zip),
        )
        .route("/api/generate/health", get(generate::get_generate_health))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        // Fallback for static files and SPA routing (must be last)
        .fallback(static_handler)
}
