//! Web API module for LazyQMK.
//!
//! This module provides a REST API for the LazyQMK layout editor,
//! enabling a web-based frontend to interact with layout files,
//! keycodes, and QMK firmware configuration.
//!
//! # Endpoints
//!
//! - `GET /health` - Health check
//! - `GET /api/layouts` - List layout markdown files
//! - `GET /api/layouts/{filename}` - Load and parse a layout file
//! - `PUT /api/layouts/{filename}` - Save a layout file
//! - `GET /api/keycodes` - Query keycode database (optional ?search=)
//! - `GET /api/keycodes/categories` - List keycode categories
//! - `GET /api/config` - Get current configuration
//! - `PUT /api/config` - Update configuration
//! - `GET /api/keyboards/{keyboard}/geometry/{layout}` - Get keyboard geometry

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::Config;
use crate::export;
use crate::keycode_db::{KeycodeCategory, KeycodeDb, KeycodeDefinition};
use crate::models::{IdleEffectSettings, Layout, RgbMatrixEffect, TapDanceAction, TapHoldSettings};
use crate::parser;
use crate::services::LayoutService;

// ============================================================================
// Application State
// ============================================================================

/// Shared application state for the web API.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    config: Arc<Config>,
    /// Keycode database (immutable after load)
    keycode_db: Arc<KeycodeDb>,
    /// Working directory for layout files (defaults to current dir)
    workspace_root: PathBuf,
}

impl AppState {
    /// Creates a new application state.
    pub fn new(config: Config, workspace_root: PathBuf) -> anyhow::Result<Self> {
        let keycode_db = KeycodeDb::load()?;
        Ok(Self {
            config: Arc::new(config),
            keycode_db: Arc::new(keycode_db),
            workspace_root,
        })
    }

    /// Returns the workspace root directory.
    #[must_use]
    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Current health status (e.g., "healthy").
    pub status: String,
    /// Application version.
    pub version: String,
}

/// Layout list response.
#[derive(Debug, Serialize)]
pub struct LayoutListResponse {
    /// List of layout summaries.
    pub layouts: Vec<LayoutSummary>,
}

/// Summary of a layout file.
#[derive(Debug, Serialize)]
pub struct LayoutSummary {
    /// Filename of the layout.
    pub filename: String,
    /// Display name of the layout.
    pub name: String,
    /// Description of the layout.
    pub description: String,
    /// Last modified timestamp (RFC 3339 format).
    pub modified: String,
}

/// Query parameters for keycode search.
#[derive(Debug, Deserialize)]
pub struct KeycodeQuery {
    /// Search term to filter keycodes.
    pub search: Option<String>,
    /// Category ID to filter keycodes.
    pub category: Option<String>,
}

/// Keycode list response.
#[derive(Debug, Serialize)]
pub struct KeycodeListResponse {
    /// List of matching keycodes.
    pub keycodes: Vec<KeycodeInfo>,
    /// Total count of matching keycodes.
    pub total: usize,
}

/// Keycode information for API response.
#[derive(Debug, Serialize)]
pub struct KeycodeInfo {
    /// Keycode string (e.g., "KC_A").
    pub code: String,
    /// Human-readable name.
    pub name: String,
    /// Category this keycode belongs to.
    pub category: String,
    /// Optional description of the keycode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl From<&KeycodeDefinition> for KeycodeInfo {
    fn from(kc: &KeycodeDefinition) -> Self {
        Self {
            code: kc.code.clone(),
            name: kc.name.clone(),
            category: kc.category.clone(),
            description: kc.description.clone(),
        }
    }
}

/// Category list response.
#[derive(Debug, Serialize)]
pub struct CategoryListResponse {
    /// List of keycode categories.
    pub categories: Vec<CategoryInfo>,
}

/// Category information for API response.
#[derive(Debug, Serialize)]
pub struct CategoryInfo {
    /// Unique category identifier.
    pub id: String,
    /// Human-readable category name.
    pub name: String,
    /// Description of the category.
    pub description: String,
}

impl From<&KeycodeCategory> for CategoryInfo {
    fn from(cat: &KeycodeCategory) -> Self {
        Self {
            id: cat.id.clone(),
            name: cat.name.clone(),
            description: cat.description.clone(),
        }
    }
}

/// Configuration response.
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    /// Path to QMK firmware directory.
    pub qmk_firmware_path: Option<String>,
    /// Output directory for generated files.
    pub output_dir: String,
}

/// Configuration update request.
#[derive(Debug, Deserialize)]
pub struct ConfigUpdateRequest {
    /// New path to QMK firmware directory.
    pub qmk_firmware_path: Option<String>,
}

/// API error response.
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Error message.
    pub error: String,
    /// Optional additional details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }

    fn with_details(error: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: Some(details.into()),
        }
    }
}

// ============================================================================
// Validate/Inspect Response Types
// ============================================================================

/// Validation result response.
#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    /// Whether the layout is valid.
    pub valid: bool,
    /// Error message if invalid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// List of warnings (non-fatal issues).
    pub warnings: Vec<String>,
}

/// Inspect response with layout details.
#[derive(Debug, Serialize)]
pub struct InspectResponse {
    /// Layout metadata.
    pub metadata: InspectMetadata,
    /// Layer information.
    pub layers: Vec<InspectLayer>,
    /// Tap dance summary.
    pub tap_dances: Vec<InspectTapDance>,
    /// Settings summary.
    pub settings: InspectSettings,
}

/// Metadata section for inspect.
#[derive(Debug, Serialize)]
pub struct InspectMetadata {
    /// Layout name.
    pub name: String,
    /// Layout description.
    pub description: String,
    /// Layout author.
    pub author: String,
    /// Target keyboard identifier.
    pub keyboard: Option<String>,
    /// Layout variant name.
    pub layout_variant: Option<String>,
    /// Creation timestamp.
    pub created: String,
    /// Last modification timestamp.
    pub modified: String,
    /// Number of layers.
    pub layer_count: usize,
    /// Total number of keys.
    pub key_count: usize,
    /// Number of categories used.
    pub category_count: usize,
    /// Number of tap dance definitions.
    pub tap_dance_count: usize,
}

/// Layer info for inspect.
#[derive(Debug, Serialize)]
pub struct InspectLayer {
    /// Layer number (0-based).
    pub number: u8,
    /// Layer name.
    pub name: String,
    /// Number of keys in this layer.
    pub key_count: usize,
    /// Default color for this layer.
    pub default_color: String,
    /// Whether per-key colors are enabled.
    pub colors_enabled: bool,
}

/// Tap dance info for inspect.
#[derive(Debug, Serialize)]
pub struct InspectTapDance {
    /// Tap dance name.
    pub name: String,
    /// Keycode for single tap.
    pub single_tap: String,
    /// Keycode for double tap.
    pub double_tap: Option<String>,
    /// Keycode for hold action.
    pub hold: Option<String>,
    /// Layers where this tap dance is used.
    pub used_in_layers: Vec<u8>,
}

/// Settings summary for inspect.
#[derive(Debug, Serialize)]
pub struct InspectSettings {
    /// Whether RGB lighting is enabled.
    pub rgb_enabled: bool,
    /// RGB brightness level (0-255).
    pub rgb_brightness: u8,
    /// RGB saturation level (0-255).
    pub rgb_saturation: u8,
    /// Whether idle effect is enabled.
    pub idle_effect_enabled: bool,
    /// Idle timeout in milliseconds.
    pub idle_timeout_ms: u32,
    /// Idle effect mode name.
    pub idle_effect_mode: String,
    /// Tapping term in milliseconds.
    pub tapping_term: u16,
    /// Tap-hold preset name.
    pub tap_hold_preset: String,
}

/// Export response with markdown content.
#[derive(Debug, Serialize)]
pub struct ExportResponse {
    /// The exported markdown content.
    pub markdown: String,
    /// Suggested filename for download.
    pub suggested_filename: String,
}

/// Generate firmware response.
#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    /// Status of the generation job.
    pub status: String,
    /// Message describing the current state.
    pub message: String,
    /// Job ID for tracking (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
}

/// Idle effect settings for API.
#[derive(Debug, Serialize, Deserialize)]
pub struct IdleEffectSettingsDto {
    /// Whether idle effect is enabled.
    pub enabled: bool,
    /// Idle timeout in milliseconds.
    pub idle_timeout_ms: u32,
    /// Idle effect duration in milliseconds.
    pub idle_effect_duration_ms: u32,
    /// Effect mode name.
    pub idle_effect_mode: String,
}

impl From<&IdleEffectSettings> for IdleEffectSettingsDto {
    fn from(s: &IdleEffectSettings) -> Self {
        Self {
            enabled: s.enabled,
            idle_timeout_ms: s.idle_timeout_ms,
            idle_effect_duration_ms: s.idle_effect_duration_ms,
            idle_effect_mode: s.idle_effect_mode.display_name().to_string(),
        }
    }
}

/// Tap hold settings for API.
#[derive(Debug, Serialize, Deserialize)]
pub struct TapHoldSettingsDto {
    /// Tapping term in milliseconds.
    pub tapping_term: u16,
    /// Quick tap term in milliseconds.
    pub quick_tap_term: Option<u16>,
    /// Hold mode name.
    pub hold_mode: String,
    /// Whether retro tapping is enabled.
    pub retro_tapping: bool,
    /// Tapping toggle count.
    pub tapping_toggle: u8,
    /// Flow tap term in milliseconds.
    pub flow_tap_term: Option<u16>,
    /// Whether chordal hold is enabled.
    pub chordal_hold: bool,
    /// Preset name.
    pub preset: String,
}

impl From<&TapHoldSettings> for TapHoldSettingsDto {
    fn from(s: &TapHoldSettings) -> Self {
        Self {
            tapping_term: s.tapping_term,
            quick_tap_term: s.quick_tap_term,
            hold_mode: s.hold_mode.display_name().to_string(),
            retro_tapping: s.retro_tapping,
            tapping_toggle: s.tapping_toggle,
            flow_tap_term: s.flow_tap_term,
            chordal_hold: s.chordal_hold,
            preset: s.preset.display_name().to_string(),
        }
    }
}

/// Combo definition for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboDto {
    /// Unique combo identifier.
    pub id: String,
    /// Combo name.
    pub name: String,
    /// Trigger keys.
    pub keys: Vec<String>,
    /// Output keycode.
    pub output: String,
}

/// RGB matrix effects list.
#[derive(Debug, Serialize)]
pub struct EffectsListResponse {
    /// List of available effects.
    pub effects: Vec<EffectInfo>,
}

/// Effect information.
#[derive(Debug, Serialize)]
pub struct EffectInfo {
    /// Effect identifier.
    pub id: String,
    /// Display name.
    pub name: String,
}

/// Tap dance DTO for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapDanceDto {
    /// Tap dance name.
    pub name: String,
    /// Keycode for single tap.
    pub single_tap: String,
    /// Keycode for double tap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_tap: Option<String>,
    /// Keycode for hold action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<String>,
}

impl From<&TapDanceAction> for TapDanceDto {
    fn from(td: &TapDanceAction) -> Self {
        Self {
            name: td.name.clone(),
            single_tap: td.single_tap.clone(),
            double_tap: td.double_tap.clone(),
            hold: td.hold.clone(),
        }
    }
}

// ============================================================================
// Path Validation (Security)
// ============================================================================

/// Validates a filename to prevent path traversal attacks.
///
/// Returns the sanitized filename or an error if the filename is invalid.
fn validate_filename(filename: &str) -> Result<&str, ApiError> {
    // Reject empty filenames
    if filename.is_empty() {
        return Err(ApiError::new("Filename cannot be empty"));
    }

    // Reject path traversal attempts
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(ApiError::new(
            "Invalid filename: path traversal not allowed",
        ));
    }

    // Reject absolute paths
    if filename.starts_with('/') || filename.starts_with('\\') {
        return Err(ApiError::new(
            "Invalid filename: absolute paths not allowed",
        ));
    }

    // Reject hidden files
    if filename.starts_with('.') {
        return Err(ApiError::new("Invalid filename: hidden files not allowed"));
    }

    Ok(filename)
}

/// Validates a keyboard path to prevent path traversal attacks.
fn validate_keyboard_path(keyboard: &str) -> Result<(), ApiError> {
    if keyboard.is_empty() {
        return Err(ApiError::new("Keyboard path cannot be empty"));
    }

    // Reject path traversal attempts
    if keyboard.contains("..") {
        return Err(ApiError::new(
            "Invalid keyboard path: path traversal not allowed",
        ));
    }

    // Reject absolute paths
    if keyboard.starts_with('/') || keyboard.starts_with('\\') {
        return Err(ApiError::new(
            "Invalid keyboard path: absolute paths not allowed",
        ));
    }

    Ok(())
}

// ============================================================================
// Route Handlers
// ============================================================================

/// GET /health - Health check endpoint.
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// GET /api/layouts - List all layout files in the workspace.
async fn list_layouts(
    State(state): State<AppState>,
) -> Result<Json<LayoutListResponse>, (StatusCode, Json<ApiError>)> {
    let mut layouts = Vec::new();

    // Read directory entries
    let entries = std::fs::read_dir(&state.workspace_root).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to read workspace directory",
                e.to_string(),
            )),
        )
    })?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        // Only process .md files
        if path.extension().is_some_and(|ext| ext == "md") {
            let filename = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            // Try to parse the layout to get metadata
            if let Ok(layout) = LayoutService::load(&path) {
                layouts.push(LayoutSummary {
                    filename,
                    name: layout.metadata.name.clone(),
                    description: layout.metadata.description.clone(),
                    modified: layout.metadata.modified.to_rfc3339(),
                });
            }
            // Skip files that can't be parsed as layouts
        }
    }

    // Sort by modification time (newest first)
    layouts.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(Json(LayoutListResponse { layouts }))
}

/// GET /api/layouts/{filename} - Load a specific layout file.
async fn get_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<Layout>, (StatusCode, Json<ApiError>)> {
    // Validate filename to prevent path traversal
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .md extension (case-insensitive)
    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        filename.to_string()
    } else {
        format!("{filename}.md")
    };

    let path = state.workspace_root.join(&filename);

    // Check if file exists
    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Layout file not found: {filename}"))),
        ));
    }

    // Load and parse the layout
    let layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load layout",
                e.to_string(),
            )),
        )
    })?;

    Ok(Json(layout))
}

/// PUT /api/layouts/{filename} - Save a layout file.
async fn save_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(layout): Json<Layout>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    // Validate filename to prevent path traversal
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .md extension (case-insensitive)
    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        filename.to_string()
    } else {
        format!("{filename}.md")
    };

    let path = state.workspace_root.join(&filename);

    // Validate the layout
    layout.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::with_details("Invalid layout", e.to_string())),
        )
    })?;

    // Save the layout
    LayoutService::save(&layout, &path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to save layout",
                e.to_string(),
            )),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/keycodes - Query keycode database.
async fn list_keycodes(
    State(state): State<AppState>,
    Query(query): Query<KeycodeQuery>,
) -> Json<KeycodeListResponse> {
    let search = query.search.as_deref().unwrap_or("");

    let keycodes: Vec<KeycodeInfo> = match &query.category {
        Some(cat) => state
            .keycode_db
            .search_in_category(search, cat)
            .into_iter()
            .map(KeycodeInfo::from)
            .collect(),
        None => state
            .keycode_db
            .search(search)
            .into_iter()
            .map(KeycodeInfo::from)
            .collect(),
    };

    let total = keycodes.len();
    Json(KeycodeListResponse { keycodes, total })
}

/// GET /api/keycodes/categories - List keycode categories.
async fn list_categories(State(state): State<AppState>) -> Json<CategoryListResponse> {
    let categories = state
        .keycode_db
        .categories()
        .iter()
        .map(CategoryInfo::from)
        .collect();

    Json(CategoryListResponse { categories })
}

/// GET /api/config - Get current configuration.
async fn get_config(State(state): State<AppState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        qmk_firmware_path: state
            .config
            .paths
            .qmk_firmware
            .as_ref()
            .map(|p| p.display().to_string()),
        output_dir: state.config.build.output_dir.display().to_string(),
    })
}

/// PUT /api/config - Update configuration.
async fn update_config(
    State(state): State<AppState>,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    // Create a mutable copy of the config
    let mut config = (*state.config).clone();

    // Update QMK firmware path if provided
    if let Some(path_str) = request.qmk_firmware_path {
        let path = PathBuf::from(path_str);

        // Validate the path exists and is a valid QMK directory
        if !path.exists() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError::new("QMK firmware path does not exist")),
            ));
        }

        config.paths.qmk_firmware = Some(path);
    }

    // Validate and save the config
    config.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::with_details(
                "Invalid configuration",
                e.to_string(),
            )),
        )
    })?;

    config.save().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to save configuration",
                e.to_string(),
            )),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Keyboard geometry response.
#[derive(Debug, Serialize)]
pub struct GeometryResponse {
    /// Keyboard name/path (e.g., "crkbd" or "splitkb/halcyon/corne").
    pub keyboard: String,
    /// Layout variant name (e.g., "LAYOUT_split_3x6_3").
    pub layout: String,
    /// List of key geometries.
    pub keys: Vec<KeyGeometryInfo>,
    /// Number of matrix rows.
    pub matrix_rows: u8,
    /// Number of matrix columns.
    pub matrix_cols: u8,
    /// Number of rotary encoders.
    pub encoder_count: u8,
}

/// Key geometry information for API response.
#[derive(Debug, Serialize)]
pub struct KeyGeometryInfo {
    /// Matrix row position.
    pub matrix_row: u8,
    /// Matrix column position.
    pub matrix_col: u8,
    /// Visual X position (in key units).
    pub x: f32,
    /// Visual Y position (in key units).
    pub y: f32,
    /// Key width (in key units, typically 1.0).
    pub width: f32,
    /// Key height (in key units, typically 1.0).
    pub height: f32,
    /// Key rotation angle in degrees.
    pub rotation: f32,
    /// RGB LED index for this key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub led_index: Option<u8>,
}

/// GET /api/keyboards/{keyboard}/geometry/{layout} - Get keyboard geometry.
///
/// The keyboard path can contain slashes (e.g., "keebart/corne_choc_pro").
async fn get_geometry(
    State(state): State<AppState>,
    Path((keyboard, layout)): Path<(String, String)>,
) -> Result<Json<GeometryResponse>, (StatusCode, Json<ApiError>)> {
    // Validate keyboard path
    validate_keyboard_path(&keyboard).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Get QMK path from config
    let qmk_path = state.config.paths.qmk_firmware.as_ref().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("QMK firmware path not configured")),
        )
    })?;

    // Parse keyboard info.json
    let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(qmk_path, &keyboard)
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::with_details(
                    format!("Failed to parse keyboard info for '{keyboard}'"),
                    e.to_string(),
                )),
            )
        })?;

    // Validate that the layout exists
    let _layout_def = keyboard_info.layouts.get(&layout).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!(
                "Layout '{layout}' not found in keyboard '{keyboard}'"
            ))),
        )
    })?;

    // Build geometry from the layout
    let geometry = parser::keyboard_json::build_keyboard_geometry_with_rgb(
        &keyboard_info,
        &keyboard,
        &layout,
        None,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to build keyboard geometry",
                e.to_string(),
            )),
        )
    })?;

    // Convert to API response
    let keys: Vec<KeyGeometryInfo> = geometry
        .keys
        .iter()
        .map(|k| KeyGeometryInfo {
            matrix_row: k.matrix_position.0,
            matrix_col: k.matrix_position.1,
            x: k.visual_x,
            y: k.visual_y,
            width: k.width,
            height: k.height,
            rotation: k.rotation,
            led_index: Some(k.led_index),
        })
        .collect();

    Ok(Json(GeometryResponse {
        keyboard,
        layout,
        keys,
        matrix_rows: geometry.matrix_rows,
        matrix_cols: geometry.matrix_cols,
        encoder_count: geometry.encoder_count,
    }))
}

// ============================================================================
// Validate, Inspect, Export, Generate Handlers
// ============================================================================

/// GET /api/layouts/{filename}/validate - Validate a layout.
async fn validate_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<ValidationResponse>, (StatusCode, Json<ApiError>)> {
    // Validate filename to prevent path traversal
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .md extension
    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        filename.to_string()
    } else {
        format!("{filename}.md")
    };

    let path = state.workspace_root.join(&filename);

    // Check if file exists
    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Layout file not found: {filename}"))),
        ));
    }

    // Load and parse the layout
    let layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load layout",
                e.to_string(),
            )),
        )
    })?;

    // Collect warnings
    let mut warnings = Vec::new();

    // Check for orphaned tap dances
    let orphaned = layout.get_orphaned_tap_dances();
    for name in &orphaned {
        warnings.push(format!("Tap dance '{name}' is defined but not used"));
    }

    // Validate the layout
    match layout.validate() {
        Ok(()) => Ok(Json(ValidationResponse {
            valid: true,
            error: None,
            warnings,
        })),
        Err(e) => Ok(Json(ValidationResponse {
            valid: false,
            error: Some(e.to_string()),
            warnings,
        })),
    }
}

/// GET /api/layouts/{filename}/inspect - Get detailed layout information.
async fn inspect_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<InspectResponse>, (StatusCode, Json<ApiError>)> {
    // Validate filename to prevent path traversal
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .md extension
    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        filename.to_string()
    } else {
        format!("{filename}.md")
    };

    let path = state.workspace_root.join(&filename);

    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Layout file not found: {filename}"))),
        ));
    }

    let layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load layout",
                e.to_string(),
            )),
        )
    })?;

    // Build inspect response
    let key_count = layout.layers.first().map_or(0, |l| l.keys.len());

    let metadata = InspectMetadata {
        name: layout.metadata.name.clone(),
        description: layout.metadata.description.clone(),
        author: layout.metadata.author.clone(),
        keyboard: layout.metadata.keyboard.clone(),
        layout_variant: layout.metadata.layout_variant.clone(),
        created: layout.metadata.created.to_rfc3339(),
        modified: layout.metadata.modified.to_rfc3339(),
        layer_count: layout.layers.len(),
        key_count,
        category_count: layout.categories.len(),
        tap_dance_count: layout.tap_dances.len(),
    };

    let layers: Vec<InspectLayer> = layout
        .layers
        .iter()
        .map(|l| InspectLayer {
            number: l.number,
            name: l.name.clone(),
            key_count: l.keys.len(),
            default_color: format!(
                "#{:02X}{:02X}{:02X}",
                l.default_color.r, l.default_color.g, l.default_color.b
            ),
            colors_enabled: l.layer_colors_enabled,
        })
        .collect();

    // Build tap dance info with usage
    let td_pattern = regex::Regex::new(r"TD\(([^)]+)\)").unwrap();
    let tap_dances: Vec<InspectTapDance> = layout
        .tap_dances
        .iter()
        .map(|td| {
            let mut used_in_layers = Vec::new();
            for layer in &layout.layers {
                for key in &layer.keys {
                    if let Some(captures) = td_pattern.captures(&key.keycode) {
                        if captures[1] == td.name && !used_in_layers.contains(&layer.number) {
                            used_in_layers.push(layer.number);
                        }
                    }
                }
            }
            InspectTapDance {
                name: td.name.clone(),
                single_tap: td.single_tap.clone(),
                double_tap: td.double_tap.clone(),
                hold: td.hold.clone(),
                used_in_layers,
            }
        })
        .collect();

    let settings = InspectSettings {
        rgb_enabled: layout.rgb_enabled,
        rgb_brightness: layout.rgb_brightness.as_percent(),
        rgb_saturation: layout.rgb_saturation.as_percent(),
        idle_effect_enabled: layout.idle_effect_settings.enabled,
        idle_timeout_ms: layout.idle_effect_settings.idle_timeout_ms,
        idle_effect_mode: layout
            .idle_effect_settings
            .idle_effect_mode
            .display_name()
            .to_string(),
        tapping_term: layout.tap_hold_settings.tapping_term,
        tap_hold_preset: layout.tap_hold_settings.preset.display_name().to_string(),
    };

    Ok(Json(InspectResponse {
        metadata,
        layers,
        tap_dances,
        settings,
    }))
}

/// GET /api/layouts/{filename}/export - Export layout to markdown.
async fn export_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<ExportResponse>, (StatusCode, Json<ApiError>)> {
    // Validate filename
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        filename.to_string()
    } else {
        format!("{filename}.md")
    };

    let path = state.workspace_root.join(&filename);

    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Layout file not found: {filename}"))),
        ));
    }

    let layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load layout",
                e.to_string(),
            )),
        )
    })?;

    // Get keyboard geometry for export
    let geometry = if let (Some(keyboard), Some(layout_variant)) = (
        layout.metadata.keyboard.as_ref(),
        layout.metadata.layout_variant.as_ref(),
    ) {
        if let Some(qmk_path) = state.config.paths.qmk_firmware.as_ref() {
            parser::keyboard_json::parse_keyboard_info_json(qmk_path, keyboard)
                .ok()
                .and_then(|info| {
                    parser::keyboard_json::build_keyboard_geometry_with_rgb(
                        &info,
                        keyboard,
                        layout_variant,
                        None,
                    )
                    .ok()
                })
        } else {
            None
        }
    } else {
        None
    };

    // Generate markdown (with or without geometry)
    let markdown = if let Some(geom) = geometry {
        export::export_to_markdown(&layout, &geom, &state.keycode_db).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::with_details(
                    "Failed to export layout",
                    e.to_string(),
                )),
            )
        })?
    } else {
        // Generate simpler markdown without geometry
        generate_simple_markdown(&layout)
    };

    // Generate suggested filename
    let layout_name = layout
        .metadata
        .name
        .to_lowercase()
        .replace(' ', "_")
        .replace(|c: char| !c.is_alphanumeric() && c != '_', "");
    let date = chrono::Utc::now().format("%Y%m%d");
    let suggested_filename = format!("{layout_name}_export_{date}.md");

    Ok(Json(ExportResponse {
        markdown,
        suggested_filename,
    }))
}

/// Generate simple markdown without geometry diagrams.
fn generate_simple_markdown(layout: &Layout) -> String {
    use std::fmt::Write;
    let mut output = String::new();

    let _ = writeln!(output, "# {}\n", layout.metadata.name);
    let _ = writeln!(output, "**Description:** {}\n", layout.metadata.description);
    let _ = writeln!(output, "**Author:** {}\n", layout.metadata.author);

    if let Some(keyboard) = &layout.metadata.keyboard {
        let _ = writeln!(output, "**Keyboard:** {keyboard}\n");
    }

    let _ = writeln!(output, "## Layers\n");
    for layer in &layout.layers {
        let _ = writeln!(output, "### Layer {}: {}\n", layer.number, layer.name);
        let _ = writeln!(output, "- Keys: {}", layer.keys.len());
        let _ = writeln!(
            output,
            "- Color: #{:02X}{:02X}{:02X}\n",
            layer.default_color.r, layer.default_color.g, layer.default_color.b
        );
    }

    if !layout.tap_dances.is_empty() {
        let _ = writeln!(output, "## Tap Dances\n");
        for td in &layout.tap_dances {
            let _ = writeln!(output, "- **{}**: tap={}", td.name, td.single_tap);
        }
    }

    output
}

/// POST /api/layouts/{filename}/generate - Generate firmware (stub).
async fn generate_firmware(
    Path(filename): Path<String>,
) -> Result<Json<GenerateResponse>, (StatusCode, Json<ApiError>)> {
    // Validate filename
    let _filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Return "not implemented" response - firmware generation is too complex for web
    Ok(Json(GenerateResponse {
        status: "not_implemented".to_string(),
        message: "Firmware generation is not available in the web interface. \
                  Please use the CLI command: lazyqmk generate <layout.md>"
            .to_string(),
        job_id: None,
    }))
}

/// GET /api/effects - List available RGB matrix effects.
async fn list_effects() -> Json<EffectsListResponse> {
    let effects = RgbMatrixEffect::all()
        .iter()
        .map(|e| EffectInfo {
            id: format!("{:?}", e).to_lowercase(),
            name: e.display_name().to_string(),
        })
        .collect();

    Json(EffectsListResponse { effects })
}

// ============================================================================
// Router Setup
// ============================================================================

/// Creates the API router with all endpoints.
pub fn create_router(state: AppState) -> Router {
    // CORS configuration - allow all origins for development
    // NOTE: This permissive CORS policy is intended for local development only.
    // In production, restrict origins to trusted domains (e.g., specific localhost
    // ports or your deployed frontend domain). This is safe for LazyQMK since the
    // server is designed to run locally on the user's machine alongside the frontend.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Layout endpoints
        .route("/api/layouts", get(list_layouts))
        .route("/api/layouts/{filename}", get(get_layout).put(save_layout))
        .route("/api/layouts/{filename}/validate", get(validate_layout))
        .route("/api/layouts/{filename}/inspect", get(inspect_layout))
        .route("/api/layouts/{filename}/export", get(export_layout))
        .route(
            "/api/layouts/{filename}/generate",
            axum::routing::post(generate_firmware),
        )
        // Keycode endpoints
        .route("/api/keycodes", get(list_keycodes))
        .route("/api/keycodes/categories", get(list_categories))
        // Config endpoints
        .route("/api/config", get(get_config).put(update_config))
        // Effects endpoint
        .route("/api/effects", get(list_effects))
        // Geometry endpoint
        .route(
            "/api/keyboards/{keyboard}/geometry/{layout}",
            get(get_geometry),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
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
