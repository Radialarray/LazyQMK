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

pub mod build_jobs;
pub mod generate_jobs;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
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
use crate::models::{
    ComboSettings, IdleEffectSettings, Layout, RgbColor, RgbMatrixEffect, RgbOverlayRippleSettings,
    TapDanceAction, TapHoldSettings,
};
use crate::parser;
use crate::services::LayoutService;

use build_jobs::BuildJobManager;
use generate_jobs::GenerateJobManager;

#[cfg(test)]
use build_jobs::MockFirmwareBuilder;

#[cfg(test)]
use generate_jobs::MockGenerateWorker;

// ============================================================================
// Static File Embedding
// ============================================================================

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
async fn static_handler(uri: axum::http::Uri) -> Response {
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

// ============================================================================
// Application State
// ============================================================================

/// Shared application state for the web API.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    config: Arc<RwLock<Config>>,
    /// Keycode database (immutable after load)
    keycode_db: Arc<KeycodeDb>,
    /// Working directory for layout files (defaults to current dir)
    workspace_root: PathBuf,
    /// Build job manager for background firmware builds
    build_manager: Arc<BuildJobManager>,
    /// Generate job manager for firmware generation and zip packaging
    generate_manager: Arc<GenerateJobManager>,
}

impl AppState {
    /// Creates a new application state.
    pub fn new(config: Config, workspace_root: PathBuf) -> anyhow::Result<Self> {
        let keycode_db = Arc::new(KeycodeDb::load()?);

        // Set up build job manager
        let logs_dir = workspace_root.join(".lazyqmk").join("build_logs");
        let output_dir = workspace_root.join(".lazyqmk").join("build_output");
        let qmk_path = config.paths.qmk_firmware.clone();
        let build_manager = BuildJobManager::new(
            logs_dir,
            output_dir,
            qmk_path.clone(),
            Arc::clone(&keycode_db),
        );

        // Set up generate job manager
        let gen_logs_dir = workspace_root.join(".lazyqmk").join("generate_logs");
        let gen_output_dir = workspace_root.join(".lazyqmk").join("generate_output");
        let generate_manager = GenerateJobManager::new(
            gen_logs_dir,
            gen_output_dir,
            workspace_root.clone(),
            qmk_path,
            Arc::clone(&keycode_db),
        );

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            keycode_db,
            workspace_root,
            build_manager,
            generate_manager,
        })
    }

    /// Creates a new application state with a mock builder (for testing).
    #[cfg(test)]
    pub fn with_mock_builder(config: Config, workspace_root: PathBuf) -> anyhow::Result<Self> {
        let keycode_db = Arc::new(KeycodeDb::load()?);

        // Set up build job manager with mock builder
        let logs_dir = workspace_root.join(".lazyqmk").join("build_logs");
        let output_dir = workspace_root.join(".lazyqmk").join("build_output");
        let qmk_path = config.paths.qmk_firmware.clone();
        let mock_builder = Arc::new(MockFirmwareBuilder::default());
        let build_manager = BuildJobManager::with_builder(
            logs_dir,
            output_dir,
            qmk_path.clone(),
            mock_builder,
            Arc::clone(&keycode_db),
        );

        // Set up generate job manager with mock worker
        let gen_logs_dir = workspace_root.join(".lazyqmk").join("generate_logs");
        let gen_output_dir = workspace_root.join(".lazyqmk").join("generate_output");
        let mock_worker = Arc::new(MockGenerateWorker::default());
        let generate_manager = GenerateJobManager::with_worker(
            gen_logs_dir,
            gen_output_dir,
            workspace_root.clone(),
            qmk_path,
            Arc::clone(&keycode_db),
            mock_worker,
        );

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            keycode_db,
            workspace_root,
            build_manager,
            generate_manager,
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
    /// Workspace root directory where layout files are stored.
    pub workspace_root: String,
}

/// Configuration update request.
#[derive(Debug, Deserialize)]
pub struct ConfigUpdateRequest {
    /// New path to QMK firmware directory.
    pub qmk_firmware_path: Option<String>,
}

/// Swap keys request.
#[derive(Debug, Deserialize)]
pub struct SwapKeysRequest {
    /// Layer number (0-based).
    pub layer: u8,
    /// First key position (row, col).
    pub first_position: KeyPosition,
    /// Second key position (row, col).
    pub second_position: KeyPosition,
}

/// Key position for swap operation.
#[derive(Debug, Deserialize)]
pub struct KeyPosition {
    /// Row number.
    pub row: u8,
    /// Column number.
    pub col: u8,
}

/// Preflight check response for onboarding flow.
#[derive(Debug, Serialize)]
pub struct PreflightResponse {
    /// Whether QMK firmware path is configured and valid.
    pub qmk_configured: bool,
    /// Whether any layouts exist in the workspace.
    pub has_layouts: bool,
    /// True if this appears to be a first-run (no layouts and no QMK config).
    pub first_run: bool,
    /// QMK firmware path if configured.
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
    /// RGB Matrix default speed (0-255).
    pub rgb_matrix_default_speed: u8,
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

// ============================================================================
// Render Metadata Types (for Key Details panel)
// ============================================================================

/// Display metadata for a single key.
#[derive(Debug, Clone, Serialize)]
pub struct KeyDisplayDto {
    /// Primary/main label for the key (short form for in-key display)
    pub primary: String,
    /// Secondary label (e.g., hold action) - optional
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<String>,
    /// Tertiary label (e.g., double-tap for tap-dance) - optional
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tertiary: Option<String>,
}

/// Type of action in a multi-action keycode.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKindDto {
    /// Single key press
    Tap,
    /// Hold action
    Hold,
    /// Double tap action (tap dance)
    DoubleTap,
    /// Layer switch
    Layer,
    /// Modifier
    Modifier,
    /// Simple keycode with no multi-action behavior
    Simple,
}

impl From<crate::keycode_db::ActionKind> for ActionKindDto {
    fn from(kind: crate::keycode_db::ActionKind) -> Self {
        match kind {
            crate::keycode_db::ActionKind::Tap => ActionKindDto::Tap,
            crate::keycode_db::ActionKind::Hold => ActionKindDto::Hold,
            crate::keycode_db::ActionKind::DoubleTap => ActionKindDto::DoubleTap,
            crate::keycode_db::ActionKind::Layer => ActionKindDto::Layer,
            crate::keycode_db::ActionKind::Modifier => ActionKindDto::Modifier,
            crate::keycode_db::ActionKind::Simple => ActionKindDto::Simple,
        }
    }
}

/// Detailed description of a single action within a keycode.
#[derive(Debug, Clone, Serialize)]
pub struct KeyDetailActionDto {
    /// Type of action
    pub kind: ActionKindDto,
    /// Raw keycode or parameter (e.g., "KC_A", "1", "MOD_LCTL")
    pub code: String,
    /// Human-readable description
    pub description: String,
}

/// Complete key render metadata for a single key.
#[derive(Debug, Clone, Serialize)]
pub struct KeyRenderMetadata {
    /// Visual index (layout array index from info.json)
    pub visual_index: u8,
    /// Short labels for in-key display
    pub display: KeyDisplayDto,
    /// Full action breakdown for Key Details panel
    pub details: Vec<KeyDetailActionDto>,
}

/// Response for layout render metadata.
#[derive(Debug, Serialize)]
pub struct RenderMetadataResponse {
    /// Layout filename
    pub filename: String,
    /// Layer-indexed key metadata (layer_index -> list of key metadata)
    pub layers: Vec<LayerRenderMetadata>,
}

/// Render metadata for a single layer.
#[derive(Debug, Serialize)]
pub struct LayerRenderMetadata {
    /// Layer number
    pub number: u8,
    /// Layer name
    pub name: String,
    /// Per-key render metadata
    pub keys: Vec<KeyRenderMetadata>,
}

/// Export response with markdown content.
#[derive(Debug, Serialize)]
pub struct ExportResponse {
    /// The exported markdown content.
    pub markdown: String,
    /// Suggested filename for download.
    pub suggested_filename: String,
}

/// Idle effect settings for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// RGB overlay ripple settings for API.
#[allow(clippy::struct_excessive_bools)] // DTO mirrors model; bools are config flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbOverlayRippleSettingsDto {
    /// Whether ripple overlay is enabled.
    pub enabled: bool,
    /// Maximum number of concurrent ripples (1-8).
    pub max_ripples: u8,
    /// Duration of each ripple in milliseconds.
    pub duration_ms: u16,
    /// Speed multiplier (0-255, higher = faster expansion).
    pub speed: u8,
    /// Band width in LED units.
    pub band_width: u8,
    /// Amplitude as percentage of base brightness (0-100).
    pub amplitude_pct: u8,
    /// Color mode for ripples.
    pub color_mode: String,
    /// Fixed color (used when color_mode = Fixed).
    pub fixed_color: RgbColor,
    /// Hue shift in degrees (used when color_mode = HueShift).
    pub hue_shift_deg: i16,
    /// Trigger on key press.
    pub trigger_on_press: bool,
    /// Trigger on key release.
    pub trigger_on_release: bool,
    /// Ignore transparent keys (KC_TRNS).
    pub ignore_transparent: bool,
    /// Ignore modifier keys.
    pub ignore_modifiers: bool,
    /// Ignore layer switch keys.
    pub ignore_layer_switch: bool,
}

impl From<&RgbOverlayRippleSettings> for RgbOverlayRippleSettingsDto {
    fn from(s: &RgbOverlayRippleSettings) -> Self {
        Self {
            enabled: s.enabled,
            max_ripples: s.max_ripples,
            duration_ms: s.duration_ms,
            speed: s.speed,
            band_width: s.band_width,
            amplitude_pct: s.amplitude_pct,
            color_mode: s.color_mode.display_name().to_string(),
            fixed_color: s.fixed_color,
            hue_shift_deg: s.hue_shift_deg,
            trigger_on_press: s.trigger_on_press,
            trigger_on_release: s.trigger_on_release,
            ignore_transparent: s.ignore_transparent,
            ignore_modifiers: s.ignore_modifiers,
            ignore_layer_switch: s.ignore_layer_switch,
        }
    }
}

/// Combo definition for API (legacy, unused).
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

/// Combo action for two-key hold combos.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ComboActionDto {
    /// Disable RGB effects and revert to TUI layer colors.
    DisableEffects,
    /// Turn off all RGB lighting completely.
    DisableLighting,
    /// Enter bootloader mode for firmware flashing.
    Bootloader,
}

impl From<&crate::models::ComboAction> for ComboActionDto {
    fn from(action: &crate::models::ComboAction) -> Self {
        match action {
            crate::models::ComboAction::DisableEffects => Self::DisableEffects,
            crate::models::ComboAction::DisableLighting => Self::DisableLighting,
            crate::models::ComboAction::Bootloader => Self::Bootloader,
        }
    }
}

/// Two-key hold combo definition for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboDefinitionDto {
    /// First key position (row, col).
    pub key1: crate::models::Position,
    /// Second key position (row, col).
    pub key2: crate::models::Position,
    /// Action to perform when combo is held.
    pub action: ComboActionDto,
    /// Duration in milliseconds both keys must be held to activate.
    pub hold_duration_ms: u16,
}

impl From<&crate::models::ComboDefinition> for ComboDefinitionDto {
    fn from(combo: &crate::models::ComboDefinition) -> Self {
        Self {
            key1: combo.key1,
            key2: combo.key2,
            action: ComboActionDto::from(&combo.action),
            hold_duration_ms: combo.hold_duration_ms,
        }
    }
}

/// Combo settings for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboSettingsDto {
    /// Whether combo feature is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// List of combo definitions (max 3).
    #[serde(default)]
    pub combos: Vec<ComboDefinitionDto>,
}

impl From<&ComboSettings> for ComboSettingsDto {
    fn from(settings: &ComboSettings) -> Self {
        Self {
            enabled: settings.enabled,
            combos: settings
                .combos
                .iter()
                .map(ComboDefinitionDto::from)
                .collect(),
        }
    }
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
// Layout DTO Types (for frontend compatibility)
// ============================================================================

/// Key assignment DTO enriched with geometry data for frontend.
///
/// This bridges the gap between Rust's `KeyDefinition` (which only has `position`)
/// and TypeScript's `KeyAssignment` interface (which expects `visual_index`,
/// `matrix_position`, and `led_index` for rendering and interaction).
#[derive(Debug, Clone, Serialize)]
pub struct KeyAssignmentDto {
    /// QMK keycode (e.g., "KC_A", "KC_TRNS", "MO(1)")
    pub keycode: String,
    /// Matrix position [row, col] derived from geometry
    pub matrix_position: [u8; 2],
    /// Visual index (layout array index from info.json)
    pub visual_index: u8,
    /// RGB LED index derived from geometry
    pub led_index: u8,
    /// Visual position (row, col) for swap API compatibility
    pub position: crate::models::Position,
    /// Individual key color override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_override: Option<RgbColor>,
    /// Category assignment for this key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    /// Optional user description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Layer DTO with enriched key data.
#[derive(Debug, Clone, Serialize)]
pub struct LayerDto {
    /// Unique layer identifier
    pub id: String,
    /// Layer number (0-based)
    pub number: u8,
    /// Human-readable name
    pub name: String,
    /// Base color for all keys on this layer
    pub default_color: RgbColor,
    /// Optional category assignment for entire layer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    /// Enriched key assignments
    pub keys: Vec<KeyAssignmentDto>,
    /// Whether layer-level RGB colors are enabled
    pub layer_colors_enabled: bool,
}

/// Complete layout DTO with enriched layer data.
#[derive(Debug, Clone, Serialize)]
pub struct LayoutDto {
    /// Layout metadata
    pub metadata: crate::models::LayoutMetadata,
    /// Enriched layers
    pub layers: Vec<LayerDto>,
    /// Category definitions
    pub categories: Vec<crate::models::Category>,
    /// RGB lighting enabled
    pub rgb_enabled: bool,
    /// RGB brightness (0-255)
    pub rgb_brightness: crate::models::RgbBrightness,
    /// RGB saturation (0-255)
    pub rgb_saturation: crate::models::RgbSaturation,
    /// RGB Matrix default animation speed (0-255)
    pub rgb_matrix_default_speed: u8,
    /// Idle effect settings
    pub idle_effect_settings: IdleEffectSettingsDto,
    /// RGB overlay ripple settings
    pub rgb_overlay_ripple: RgbOverlayRippleSettingsDto,
    /// Tap-hold settings
    pub tap_hold_settings: TapHoldSettingsDto,
    /// Tap dance definitions
    pub tap_dances: Vec<TapDanceDto>,
    /// Combo settings
    pub combo_settings: ComboSettingsDto,
}

/// Layout DTO for save requests (accepts optional fields from frontend).
///
/// The frontend sends back the LayoutDto it received from GET, but we need to be
/// flexible about which fields are required since the TypeScript interface has many
/// optional fields.
#[derive(Debug, Clone, Deserialize)]
pub struct LayoutSaveDto {
    /// Layout metadata
    pub metadata: crate::models::LayoutMetadata,
    /// Enriched layers
    pub layers: Vec<LayerSaveDto>,
    /// Category definitions
    #[serde(default)]
    pub categories: Vec<crate::models::Category>,
    /// RGB lighting enabled
    #[serde(default = "default_rgb_enabled_true")]
    pub rgb_enabled: bool,
    /// RGB brightness (0-100%)
    #[serde(default)]
    pub rgb_brightness: crate::models::RgbBrightness,
    /// RGB saturation (0-200%)
    #[serde(default)]
    pub rgb_saturation: crate::models::RgbSaturation,
    /// RGB Matrix default animation speed (0-255)
    #[serde(default = "default_rgb_matrix_speed")]
    pub rgb_matrix_default_speed: u8,
    /// RGB Matrix timeout in milliseconds
    #[serde(default)]
    pub rgb_timeout_ms: u32,
    /// Behavior for keys without individual or category colors
    #[serde(default)]
    pub uncolored_key_behavior: u8,
    /// Idle effect settings
    #[serde(default)]
    pub idle_effect_settings: Option<IdleEffectSettingsDto>,
    /// RGB overlay ripple settings
    #[serde(default)]
    pub rgb_overlay_ripple: Option<RgbOverlayRippleSettingsDto>,
    /// Tap-hold settings
    #[serde(default)]
    pub tap_hold_settings: Option<TapHoldSettingsDto>,
    /// Tap dance definitions
    #[serde(default)]
    pub tap_dances: Vec<TapDanceDto>,
    /// Combo settings
    #[serde(default)]
    pub combo_settings: Option<ComboSettingsDto>,
}

fn default_rgb_enabled_true() -> bool {
    true
}

fn default_rgb_matrix_speed() -> u8 {
    127
}

/// Layer DTO for save requests (accepts optional fields from frontend).
#[derive(Debug, Clone, Deserialize)]
pub struct LayerSaveDto {
    /// Unique layer identifier
    #[serde(default)]
    pub id: Option<String>,
    /// Layer number (0-based)
    #[serde(default)]
    pub number: Option<u8>,
    /// Human-readable name
    pub name: String,
    /// Base color for all keys on this layer
    #[serde(default)]
    pub default_color: Option<RgbColor>,
    /// Optional category assignment for entire layer
    pub category_id: Option<String>,
    /// Enriched key assignments
    pub keys: Vec<KeyAssignmentSaveDto>,
    /// Whether layer-level RGB colors are enabled
    #[serde(default = "default_layer_colors_true")]
    pub layer_colors_enabled: bool,
    /// Legacy field from TypeScript interface (ignored)
    #[serde(default, skip_deserializing)]
    pub color: Option<String>,
}

fn default_layer_colors_true() -> bool {
    true
}

/// Key assignment DTO for save requests (accepts optional fields from frontend).
#[derive(Debug, Clone, Deserialize)]
pub struct KeyAssignmentSaveDto {
    /// QMK keycode (e.g., "KC_A", "KC_TRNS", "MO(1)")
    pub keycode: String,
    /// Matrix position [row, col] - enriched field from GET response
    #[serde(default)]
    pub matrix_position: [u8; 2],
    /// Visual index - enriched field from GET response
    #[serde(default)]
    pub visual_index: u8,
    /// RGB LED index - enriched field from GET response
    #[serde(default)]
    pub led_index: u8,
    /// Visual position (row, col) - the authoritative position field
    pub position: Option<crate::models::Position>,
    /// Individual key color override
    pub color_override: Option<RgbColor>,
    /// Category assignment for this key
    pub category_id: Option<String>,
    /// Optional user description
    pub description: Option<String>,
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
) -> Result<Json<LayoutDto>, (StatusCode, Json<ApiError>)> {
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

    // Build position_to_geometry mapping from keyboard geometry (if available)
    // This provides the authoritative mapping from visual position to geometry data
    let qmk_path = state.config.read().unwrap().paths.qmk_firmware.clone();
    let position_to_geometry: std::collections::HashMap<String, (u8, [u8; 2], u8)> =
        if let (Some(keyboard), Some(qmk_path)) =
            (layout.metadata.keyboard.as_ref(), qmk_path.as_ref())
        {
            // Determine layout variant (fall back to "LAYOUT" if not specified)
            let layout_variant = layout
                .metadata
                .layout_variant
                .clone()
                .unwrap_or_else(|| "LAYOUT".to_string());

            // Try to load geometry - if it fails, we'll fall back to array index
            parser::keyboard_json::parse_keyboard_info_json(qmk_path, keyboard)
                .ok()
                .and_then(|keyboard_info| {
                    parser::keyboard_json::build_keyboard_geometry_with_rgb(
                        &keyboard_info,
                        keyboard,
                        &layout_variant,
                        None,
                    )
                    .ok()
                })
                .map(|geometry| {
                    // Build mapping from position (row,col) to (visual_index, matrix_position, led_index)
                    geometry
                        .keys
                        .iter()
                        .map(|k| {
                            let row = k.visual_y.round() as u8;
                            let col = k.visual_x.round() as u8;
                            let pos_key = format!("{row},{col}");
                            (
                                pos_key,
                                (
                                    k.layout_index,
                                    [k.matrix_position.0, k.matrix_position.1],
                                    k.led_index,
                                ),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

    // Convert Layout to LayoutDto with enriched key data
    let layers: Vec<LayerDto> = layout
        .layers
        .iter()
        .map(|layer| {
            let keys: Vec<KeyAssignmentDto> = layer
                .keys
                .iter()
                .enumerate()
                .map(|(idx, key)| {
                    // Look up geometry data from position mapping, fall back to array index
                    let pos_key = format!("{},{}", key.position.row, key.position.col);
                    let (visual_index, matrix_position, led_index) = position_to_geometry
                        .get(&pos_key)
                        .copied()
                        .unwrap_or_else(|| {
                            // Fallback: use array index for all fields
                            let idx_u8 = idx as u8;
                            (idx_u8, [idx_u8, 0], idx_u8)
                        });

                    KeyAssignmentDto {
                        keycode: key.keycode.clone(),
                        matrix_position,
                        visual_index,
                        led_index,
                        position: key.position,
                        color_override: key.color_override,
                        category_id: key.category_id.clone(),
                        description: key.description.clone(),
                    }
                })
                .collect();

            LayerDto {
                id: layer.id.clone(),
                number: layer.number,
                name: layer.name.clone(),
                default_color: layer.default_color,
                category_id: layer.category_id.clone(),
                keys,
                layer_colors_enabled: layer.layer_colors_enabled,
            }
        })
        .collect();

    let layout_dto = LayoutDto {
        metadata: layout.metadata,
        layers,
        categories: layout.categories,
        rgb_enabled: layout.rgb_enabled,
        rgb_brightness: layout.rgb_brightness,
        rgb_saturation: layout.rgb_saturation,
        rgb_matrix_default_speed: layout.rgb_matrix_default_speed,
        idle_effect_settings: IdleEffectSettingsDto::from(&layout.idle_effect_settings),
        rgb_overlay_ripple: RgbOverlayRippleSettingsDto::from(&layout.rgb_overlay_ripple),
        tap_hold_settings: TapHoldSettingsDto::from(&layout.tap_hold_settings),
        tap_dances: layout.tap_dances.iter().map(TapDanceDto::from).collect(),
        combo_settings: ComboSettingsDto::from(&layout.combo_settings),
    };

    Ok(Json(layout_dto))
}

/// Parses a RippleColorMode from its display name.
fn parse_ripple_color_mode(name: &str) -> crate::models::RippleColorMode {
    use crate::models::RippleColorMode;
    match name {
        "Fixed Color" | "fixed" => RippleColorMode::Fixed,
        "Key Color" | "key_based" => RippleColorMode::KeyBased,
        "Hue Shift" | "hue_shift" => RippleColorMode::HueShift,
        _ => RippleColorMode::default(),
    }
}

/// Parses a HoldDecisionMode from its display name.
fn parse_hold_decision_mode(name: &str) -> crate::models::HoldDecisionMode {
    use crate::models::HoldDecisionMode;
    match name {
        "Default (Timing Only)" | "Default" | "default" => HoldDecisionMode::Default,
        "Permissive Hold" | "permissive_hold" => HoldDecisionMode::PermissiveHold,
        "Hold On Other Key" | "hold_on_other_key_press" => HoldDecisionMode::HoldOnOtherKeyPress,
        _ => HoldDecisionMode::default(),
    }
}

/// Parses a TapHoldPreset from its display name.
fn parse_tap_hold_preset(name: &str) -> crate::models::TapHoldPreset {
    use crate::models::TapHoldPreset;
    match name {
        "Default" | "default" => TapHoldPreset::Default,
        "Home Row Mods" | "home_row_mods" => TapHoldPreset::HomeRowMods,
        "Responsive" | "responsive" => TapHoldPreset::Responsive,
        "Deliberate" | "deliberate" => TapHoldPreset::Deliberate,
        "Custom" | "custom" => TapHoldPreset::Custom,
        _ => TapHoldPreset::default(),
    }
}

/// Converts a LayoutSaveDto (from frontend) back to the internal Layout model.
///
/// This strips the enriched fields (visual_index, matrix_position, led_index)
/// that were added during the GET response and keeps only the core data needed
/// for persistence (position, keycode, color_override, etc.).
fn convert_dto_to_layout(dto: LayoutSaveDto) -> Layout {
    use crate::models::layer::{KeyDefinition, Layer, Position};

    // Convert DTOs back to internal models
    let layers: Vec<Layer> = dto
        .layers
        .into_iter()
        .enumerate()
        .map(|(idx, layer_dto)| {
            let keys: Vec<KeyDefinition> = layer_dto
                .keys
                .into_iter()
                .map(|key_dto| {
                    // Extract position - use the position field if present,
                    // otherwise infer from matrix_position
                    let position = key_dto.position.unwrap_or_else(|| Position {
                        row: key_dto.matrix_position[0],
                        col: key_dto.matrix_position[1],
                    });

                    KeyDefinition {
                        position,
                        keycode: key_dto.keycode,
                        label: None, // Not exposed in DTO
                        color_override: key_dto.color_override,
                        category_id: key_dto.category_id,
                        combo_participant: false, // Not exposed in DTO
                        description: key_dto.description,
                    }
                })
                .collect();

            Layer {
                id: layer_dto
                    .id
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                number: layer_dto.number.unwrap_or(idx as u8),
                name: layer_dto.name,
                default_color: layer_dto
                    .default_color
                    .unwrap_or_else(|| RgbColor::new(0, 0, 0)),
                category_id: layer_dto.category_id,
                keys,
                layer_colors_enabled: layer_dto.layer_colors_enabled,
            }
        })
        .collect();

    // Convert idle effect settings
    let idle_effect_settings = if let Some(settings_dto) = dto.idle_effect_settings {
        IdleEffectSettings {
            enabled: settings_dto.enabled,
            idle_timeout_ms: settings_dto.idle_timeout_ms,
            idle_effect_duration_ms: settings_dto.idle_effect_duration_ms,
            idle_effect_mode: RgbMatrixEffect::from_name(&settings_dto.idle_effect_mode)
                .unwrap_or_default(),
        }
    } else {
        IdleEffectSettings::default()
    };

    // Convert RGB overlay ripple settings
    let rgb_overlay_ripple = if let Some(ripple_dto) = dto.rgb_overlay_ripple {
        // Parse color mode from display name
        let color_mode = parse_ripple_color_mode(&ripple_dto.color_mode);

        RgbOverlayRippleSettings {
            enabled: ripple_dto.enabled,
            max_ripples: ripple_dto.max_ripples,
            duration_ms: ripple_dto.duration_ms,
            speed: ripple_dto.speed,
            band_width: ripple_dto.band_width,
            amplitude_pct: ripple_dto.amplitude_pct,
            color_mode,
            fixed_color: ripple_dto.fixed_color,
            hue_shift_deg: ripple_dto.hue_shift_deg,
            trigger_on_press: ripple_dto.trigger_on_press,
            trigger_on_release: ripple_dto.trigger_on_release,
            ignore_transparent: ripple_dto.ignore_transparent,
            ignore_modifiers: ripple_dto.ignore_modifiers,
            ignore_layer_switch: ripple_dto.ignore_layer_switch,
        }
    } else {
        RgbOverlayRippleSettings::default()
    };

    // Convert tap-hold settings
    let tap_hold_settings = if let Some(th_dto) = dto.tap_hold_settings {
        // Parse hold mode and preset from display names
        let hold_mode = parse_hold_decision_mode(&th_dto.hold_mode);
        let preset = parse_tap_hold_preset(&th_dto.preset);

        TapHoldSettings {
            tapping_term: th_dto.tapping_term,
            quick_tap_term: th_dto.quick_tap_term,
            hold_mode,
            retro_tapping: th_dto.retro_tapping,
            tapping_toggle: th_dto.tapping_toggle,
            flow_tap_term: th_dto.flow_tap_term,
            chordal_hold: th_dto.chordal_hold,
            preset,
        }
    } else {
        TapHoldSettings::default()
    };

    // Convert tap dances
    let tap_dances: Vec<TapDanceAction> = dto
        .tap_dances
        .into_iter()
        .map(|td_dto| TapDanceAction {
            name: td_dto.name,
            single_tap: td_dto.single_tap,
            double_tap: td_dto.double_tap,
            hold: td_dto.hold,
        })
        .collect();

    // Convert combo settings
    let combo_settings = if let Some(combo_dto) = dto.combo_settings {
        use crate::models::{ComboAction, ComboDefinition};

        let combos: Vec<ComboDefinition> = combo_dto
            .combos
            .into_iter()
            .map(|c_dto| {
                let action = match c_dto.action {
                    ComboActionDto::DisableEffects => ComboAction::DisableEffects,
                    ComboActionDto::DisableLighting => ComboAction::DisableLighting,
                    ComboActionDto::Bootloader => ComboAction::Bootloader,
                };
                ComboDefinition::with_duration(
                    c_dto.key1,
                    c_dto.key2,
                    action,
                    c_dto.hold_duration_ms,
                )
            })
            .collect();

        ComboSettings {
            enabled: combo_dto.enabled,
            combos,
        }
    } else {
        ComboSettings::default()
    };

    Layout {
        metadata: dto.metadata,
        layers,
        categories: dto.categories,
        rgb_enabled: dto.rgb_enabled,
        rgb_brightness: dto.rgb_brightness,
        rgb_saturation: dto.rgb_saturation,
        rgb_matrix_default_speed: dto.rgb_matrix_default_speed,
        rgb_timeout_ms: dto.rgb_timeout_ms,
        uncolored_key_behavior: dto.uncolored_key_behavior.into(),
        idle_effect_settings,
        rgb_overlay_ripple,
        tap_hold_settings,
        combo_settings,
        tap_dances,
    }
}

/// PUT /api/layouts/{filename} - Save a layout file.
async fn save_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(layout_dto): Json<LayoutSaveDto>,
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

    // Convert LayoutDto back to Layout model
    // The DTO contains enriched fields (visual_index, matrix_position, led_index)
    // that we need to discard when converting back to the internal model
    let layout = convert_dto_to_layout(layout_dto);

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

/// POST /api/layouts/{filename}/swap-keys - Swap two keys in a layout.
async fn swap_keys(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(request): Json<SwapKeysRequest>,
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

    // Check if file exists
    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Layout file not found: {filename}"))),
        ));
    }

    // Load the layout
    let mut layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load layout",
                e.to_string(),
            )),
        )
    })?;

    // Validate layer number
    if request.layer as usize >= layout.layers.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(format!(
                "Invalid layer number: {}",
                request.layer
            ))),
        ));
    }

    // Check if trying to swap a key with itself
    if request.first_position.row == request.second_position.row
        && request.first_position.col == request.second_position.col
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("Cannot swap a key with itself")),
        ));
    }

    // Get the layer
    let layer = &mut layout.layers[request.layer as usize];

    // Debug logging
    eprintln!(
        "Swap request: layer={}, first=({},{}), second=({},{})",
        request.layer,
        request.first_position.row,
        request.first_position.col,
        request.second_position.row,
        request.second_position.col
    );
    eprintln!("Layer has {} keys", layer.keys.len());
    for (i, k) in layer.keys.iter().enumerate() {
        eprintln!(
            "  Key {}: pos=({},{}) keycode={}",
            i, k.position.row, k.position.col, k.keycode
        );
    }

    // Find indices of both keys by position
    let first_idx = layer.keys.iter().position(|k| {
        k.position.row == request.first_position.row && k.position.col == request.first_position.col
    });
    let second_idx = layer.keys.iter().position(|k| {
        k.position.row == request.second_position.row
            && k.position.col == request.second_position.col
    });

    eprintln!(
        "Found first_idx={:?}, second_idx={:?}",
        first_idx, second_idx
    );

    match (first_idx, second_idx) {
        (Some(idx1), Some(idx2)) => {
            // Swap all properties: keycode, color_override, category_id
            let first_key = layer.keys[idx1].clone();
            let second_key = layer.keys[idx2].clone();

            layer.keys[idx1].keycode = second_key.keycode;
            layer.keys[idx1].color_override = second_key.color_override;
            layer.keys[idx1].category_id = second_key.category_id;

            layer.keys[idx2].keycode = first_key.keycode;
            layer.keys[idx2].color_override = first_key.color_override;
            layer.keys[idx2].category_id = first_key.category_id;

            // Save the layout
            LayoutService::save(&layout, &path).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError::with_details(
                        "Failed to save layout after swap",
                        e.to_string(),
                    )),
                )
            })?;

            Ok(StatusCode::NO_CONTENT)
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("One or both key positions not found")),
        )),
    }
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
            .read()
            .unwrap()
            .paths
            .qmk_firmware
            .as_ref()
            .map(|p| p.display().to_string()),
        output_dir: state
            .config
            .read()
            .unwrap()
            .build
            .output_dir
            .display()
            .to_string(),
        workspace_root: state.workspace_root.display().to_string(),
    })
}

/// PUT /api/config - Update configuration.
async fn update_config(
    State(state): State<AppState>,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    // Create a mutable copy of the config
    let mut config = (*state.config.read().unwrap()).clone();

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

    *state.config.write().unwrap() = config;

    // Propagate QMK path update to build and generate managers
    let new_qmk_path = state.config.read().unwrap().paths.qmk_firmware.clone();
    state.build_manager.set_qmk_path(new_qmk_path.clone());
    state.generate_manager.set_qmk_path(new_qmk_path);

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/preflight - Check application state for onboarding flow.
async fn get_preflight(State(state): State<AppState>) -> Json<PreflightResponse> {
    // Check if QMK firmware path is configured and valid
    let qmk_configured = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .as_ref()
        .is_some_and(|p| p.exists());

    let qmk_firmware_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .as_ref()
        .map(|p| p.display().to_string());

    // Check if any layouts exist in the workspace
    let has_layouts = std::fs::read_dir(&state.workspace_root)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .any(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
        })
        .unwrap_or(false);

    // First run if no layouts AND no QMK config
    let first_run = !has_layouts && !qmk_configured;

    Json(PreflightResponse {
        qmk_configured,
        has_layouts,
        first_run,
        qmk_firmware_path,
    })
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
    /// Mapping from visual position ("row,col") to visual_index (layout array index).
    /// This allows the frontend to look up the visual_index for keys that only have
    /// position data, avoiding brittle coordinate inference logic.
    pub position_to_visual_index: std::collections::HashMap<String, u8>,
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
    /// Visual index (layout array index from info.json).
    /// This matches the visual_index in KeyAssignment and should be used for mapping keycodes.
    pub visual_index: u8,
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
    let qmk_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .clone()
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError::new("QMK firmware path not configured")),
            )
        })?;

    // Parse keyboard info.json
    let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(&qmk_path, &keyboard)
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
            visual_index: k.layout_index,
        })
        .collect();

    // Build position_to_visual_index mapping from geometry
    // This provides the authoritative mapping from visual position to layout array index,
    // so the frontend doesn't need to infer it from coordinates.
    let position_to_visual_index: std::collections::HashMap<String, u8> = geometry
        .keys
        .iter()
        .map(|k| {
            // Quantize visual coordinates to grid position (same logic as VisualLayoutMapping::build)
            let row = k.visual_y.round() as u8;
            let col = k.visual_x.round() as u8;
            let pos_key = format!("{row},{col}");
            (pos_key, k.layout_index)
        })
        .collect();

    Ok(Json(GeometryResponse {
        keyboard,
        layout,
        keys,
        matrix_rows: geometry.matrix_rows,
        matrix_cols: geometry.matrix_cols,
        encoder_count: geometry.encoder_count,
        position_to_visual_index,
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
        rgb_matrix_default_speed: layout.rgb_matrix_default_speed,
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

/// GET /api/layouts/{filename}/render-metadata - Get key display metadata for rendering.
///
/// Returns per-key display information (primary/secondary/tertiary labels and
/// full action details) for all layers. This data is used by the frontend to
/// render multi-action keycodes (tap-hold, layer-tap, mod-tap, tap-dance, etc.)
/// with compact in-key labels and full descriptions in a Key Details panel.
async fn get_render_metadata(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<RenderMetadataResponse>, (StatusCode, Json<ApiError>)> {
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

    // Build position_to_visual_index mapping from geometry (if keyboard info is available)
    // This mapping converts key positions (row,col) to the visual_index expected by the frontend
    let qmk_path = state.config.read().unwrap().paths.qmk_firmware.clone();
    let position_to_visual_index: std::collections::HashMap<String, u8> =
        if let (Some(keyboard), Some(qmk_path)) =
            (layout.metadata.keyboard.as_ref(), qmk_path.as_ref())
        {
            // Determine layout variant (fall back to default if not specified)
            let layout_variant = layout
                .metadata
                .layout_variant
                .clone()
                .unwrap_or_else(|| "LAYOUT".to_string());

            // Try to load geometry - if it fails, we'll fall back to array index
            parser::keyboard_json::parse_keyboard_info_json(qmk_path, keyboard)
                .ok()
                .and_then(|keyboard_info| {
                    parser::keyboard_json::build_keyboard_geometry_with_rgb(
                        &keyboard_info,
                        keyboard,
                        &layout_variant,
                        None,
                    )
                    .ok()
                })
                .map(|geometry| {
                    // Build mapping from position (row,col) to visual_index
                    geometry
                        .keys
                        .iter()
                        .map(|k| {
                            let row = k.visual_y.round() as u8;
                            let col = k.visual_x.round() as u8;
                            let pos_key = format!("{row},{col}");
                            (pos_key, k.layout_index)
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

    // Build tap dance lookup for display info
    let tap_dance_map: std::collections::HashMap<String, &TapDanceAction> = layout
        .tap_dances
        .iter()
        .map(|td| (td.name.clone(), td))
        .collect();

    // Build layer ID to number mapping for resolving @uuid references
    let layer_id_to_number: std::collections::HashMap<String, u8> = layout
        .layers
        .iter()
        .map(|layer| (layer.id.clone(), layer.number))
        .collect();

    // Build render metadata for each layer
    let layers: Vec<LayerRenderMetadata> = layout
        .layers
        .iter()
        .map(|layer| {
            let keys: Vec<KeyRenderMetadata> = layer
                .keys
                .iter()
                .enumerate()
                .map(|(idx, key)| {
                    // Look up visual_index from position mapping, fall back to array index
                    let pos_key = format!("{},{}", key.position.row, key.position.col);
                    let visual_index = position_to_visual_index
                        .get(&pos_key)
                        .copied()
                        .unwrap_or(idx as u8);

                    // Check if this is a tap dance keycode and get its info
                    let td_info = state
                        .keycode_db
                        .parse_tap_dance_keycode(&key.keycode)
                        .and_then(|td_name| tap_dance_map.get(&td_name))
                        .map(|td| crate::keycode_db::TapDanceDisplayInfo {
                            single_tap: td.single_tap.clone(),
                            double_tap: td.double_tap.clone(),
                            hold: td.hold.clone(),
                        });

                    // Get display metadata from keycode_db, passing layer ID map
                    let meta = state.keycode_db.get_display_metadata(
                        &key.keycode,
                        td_info.as_ref(),
                        Some(&layer_id_to_number),
                    );

                    KeyRenderMetadata {
                        visual_index,
                        display: KeyDisplayDto {
                            primary: meta.display.primary,
                            secondary: meta.display.secondary,
                            tertiary: meta.display.tertiary,
                        },
                        details: meta
                            .details
                            .into_iter()
                            .map(|d| KeyDetailActionDto {
                                kind: d.kind.into(),
                                code: d.code,
                                description: d.description,
                            })
                            .collect(),
                    }
                })
                .collect();

            LayerRenderMetadata {
                number: layer.number,
                name: layer.name.clone(),
                keys,
            }
        })
        .collect();

    Ok(Json(RenderMetadataResponse { filename, layers }))
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
        let qmk_path = state.config.read().unwrap().paths.qmk_firmware.clone();
        if let Some(qmk_path) = qmk_path {
            parser::keyboard_json::parse_keyboard_info_json(&qmk_path, keyboard)
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

/// POST /api/layouts/{filename}/generate - Generate firmware and return job info.
async fn generate_firmware(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<generate_jobs::StartGenerateResponse>, (StatusCode, Json<ApiError>)> {
    // Validate filename
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

    // Load the layout to get keyboard/layout variant info
    let layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load layout",
                e.to_string(),
            )),
        )
    })?;

    // Get keyboard from layout metadata
    let keyboard = layout.metadata.keyboard.clone().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(
                "Layout has no keyboard defined - cannot generate firmware",
            )),
        )
    })?;

    // Get layout variant from layout metadata
    let layout_variant = layout.metadata.layout_variant.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(
                "Layout has no layout variant defined - cannot generate firmware",
            )),
        )
    })?;

    // Start the generate job
    let job = state
        .generate_manager
        .start_generate(filename.clone(), keyboard, layout_variant)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::with_details("Failed to start generation", e)),
            )
        })?;

    Ok(Json(generate_jobs::StartGenerateResponse {
        status: "started".to_string(),
        message: format!("Firmware generation started for {filename}"),
        job,
    }))
}

// ============================================================================
// Build Job Endpoints
// ============================================================================

/// Query parameters for fetching build logs.
#[derive(Debug, Deserialize)]
pub struct BuildLogsQuery {
    /// Offset to start reading logs from.
    #[serde(default)]
    pub offset: usize,
    /// Maximum number of log lines to return.
    #[serde(default = "default_log_limit")]
    pub limit: usize,
}

fn default_log_limit() -> usize {
    100
}

/// POST /api/build/start - Start a firmware build job.
async fn start_build(
    State(state): State<AppState>,
    Json(request): Json<build_jobs::StartBuildRequest>,
) -> Result<Json<build_jobs::StartBuildResponse>, (StatusCode, Json<ApiError>)> {
    // Validate layout filename
    let filename = validate_filename(&request.layout_filename)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

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

    // Load the layout to get keyboard/keymap info
    let layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load layout",
                e.to_string(),
            )),
        )
    })?;

    // Get keyboard and keymap from layout metadata
    let keyboard = layout.metadata.keyboard.clone().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(
                "Layout has no keyboard defined - cannot build firmware",
            )),
        )
    })?;

    let keymap = layout
        .metadata
        .keymap_name
        .unwrap_or_else(|| "default".to_string());

    // Start the build job
    let job = state
        .build_manager
        .start_build(filename, keyboard, keymap, path)
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, Json(ApiError::new(e))))?;

    Ok(Json(build_jobs::StartBuildResponse { job }))
}

/// GET /api/build/jobs - List all build jobs.
async fn list_build_jobs(State(state): State<AppState>) -> Json<Vec<build_jobs::BuildJob>> {
    Json(state.build_manager.list_jobs())
}

/// GET /api/build/jobs/{job_id} - Get build job status.
async fn get_build_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<build_jobs::JobStatusResponse>, (StatusCode, Json<ApiError>)> {
    let job = state.build_manager.get_job(&job_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Build job not found: {job_id}"))),
        )
    })?;

    Ok(Json(build_jobs::JobStatusResponse { job }))
}

/// GET /api/build/jobs/{job_id}/logs - Get build job logs.
async fn get_build_logs(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Query(query): Query<BuildLogsQuery>,
) -> Result<Json<build_jobs::JobLogsResponse>, (StatusCode, Json<ApiError>)> {
    let logs = state
        .build_manager
        .get_logs(&job_id, query.offset, query.limit)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::new(format!("Build job not found: {job_id}"))),
            )
        })?;

    Ok(Json(logs))
}

/// POST /api/build/jobs/{job_id}/cancel - Cancel a build job.
async fn cancel_build_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Json<build_jobs::CancelJobResponse> {
    Json(state.build_manager.cancel_job(&job_id))
}

/// Response for listing build artifacts.
#[derive(Debug, Serialize)]
pub struct BuildArtifactsResponse {
    /// Job ID.
    pub job_id: String,
    /// List of artifacts.
    pub artifacts: Vec<build_jobs::BuildArtifact>,
}

/// GET /api/build/jobs/{job_id}/artifacts - List artifacts for a build job.
async fn get_build_artifacts(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<BuildArtifactsResponse>, (StatusCode, Json<ApiError>)> {
    let artifacts = state.build_manager.get_artifacts(&job_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Build job not found: {job_id}"))),
        )
    })?;

    Ok(Json(BuildArtifactsResponse { job_id, artifacts }))
}

/// GET /api/build/jobs/{job_id}/artifacts/{artifact_id}/download - Download a build artifact.
async fn download_build_artifact(
    State(state): State<AppState>,
    Path((job_id, artifact_id)): Path<(String, String)>,
) -> Result<axum::response::Response, (StatusCode, Json<ApiError>)> {
    use axum::body::Body;
    use axum::http::header;
    use axum::response::IntoResponse;

    // Get the artifact path (includes security validation)
    let artifact_path = state
        .build_manager
        .get_artifact_path(&job_id, &artifact_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::new(format!(
                    "Artifact '{artifact_id}' not found for job '{job_id}'"
                ))),
            )
        })?;

    // Check file exists
    if !artifact_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Artifact file not found on disk")),
        ));
    }

    // Read the file
    let file_content = std::fs::read(&artifact_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to read artifact file",
                e.to_string(),
            )),
        )
    })?;

    // Get filename for Content-Disposition header
    let filename = artifact_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("firmware");

    // Determine content type based on extension
    let content_type = match artifact_id.as_str() {
        "uf2" => "application/octet-stream",
        "bin" => "application/octet-stream",
        "hex" => "text/plain",
        _ => "application/octet-stream",
    };

    // Build response with proper headers for file download
    let response = (
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{filename}\""),
            ),
        ],
        Body::from(file_content),
    )
        .into_response();

    Ok(response)
}

// ============================================================================
// Generate Job Endpoints
// ============================================================================

/// Query parameters for fetching generate logs.
#[derive(Debug, Deserialize)]
pub struct GenerateLogsQuery {
    /// Offset to start reading logs from.
    #[serde(default)]
    pub offset: usize,
    /// Maximum number of log lines to return.
    #[serde(default = "default_log_limit")]
    pub limit: usize,
}

/// GET /api/generate/jobs - List all generate jobs.
async fn list_generate_jobs(
    State(state): State<AppState>,
) -> Json<Vec<generate_jobs::GenerateJob>> {
    Json(state.generate_manager.list_jobs())
}

/// GET /api/generate/jobs/{job_id} - Get generate job status.
async fn get_generate_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<generate_jobs::GenerateJobStatusResponse>, (StatusCode, Json<ApiError>)> {
    let job = state.generate_manager.get_job(&job_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Generate job not found: {job_id}"))),
        )
    })?;

    Ok(Json(generate_jobs::GenerateJobStatusResponse { job }))
}

/// GET /api/generate/jobs/{job_id}/logs - Get generate job logs.
async fn get_generate_logs(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Query(query): Query<GenerateLogsQuery>,
) -> Result<Json<generate_jobs::GenerateJobLogsResponse>, (StatusCode, Json<ApiError>)> {
    let logs = state
        .generate_manager
        .get_logs(&job_id, query.offset, query.limit)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::new(format!("Generate job not found: {job_id}"))),
            )
        })?;

    Ok(Json(logs))
}

/// POST /api/generate/jobs/{job_id}/cancel - Cancel a generate job.
async fn cancel_generate_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Json<generate_jobs::CancelGenerateJobResponse> {
    Json(state.generate_manager.cancel_job(&job_id))
}

/// GET /api/generate/jobs/{job_id}/download - Download the generated zip file.
async fn download_generate_zip(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::response::Response, (StatusCode, Json<ApiError>)> {
    use axum::body::Body;
    use axum::http::header;
    use axum::response::IntoResponse;

    // Get the job
    let job = state.generate_manager.get_job(&job_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Generate job not found: {job_id}"))),
        )
    })?;

    // Check job is completed
    if job.status != generate_jobs::GenerateJobStatus::Completed {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(format!(
                "Job is not completed. Current status: {}",
                job.status
            ))),
        ));
    }

    // Get the zip path
    let zip_path = state
        .generate_manager
        .get_zip_path(&job_id)
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("Zip file path not found")),
            )
        })?;

    // Check file exists
    if !zip_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Zip file no longer exists")),
        ));
    }

    // Read the file
    let file_content = std::fs::read(&zip_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to read zip file",
                e.to_string(),
            )),
        )
    })?;

    // Extract filename for Content-Disposition
    let filename = zip_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("firmware.zip");

    // Build response with proper headers
    let response = (
        [
            (header::CONTENT_TYPE, "application/zip"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{filename}\""),
            ),
        ],
        Body::from(file_content),
    )
        .into_response();

    Ok(response)
}

/// GET /api/generate/health - Get generate job system health.
async fn get_generate_health(
    State(state): State<AppState>,
) -> Json<generate_jobs::GenerateJobHealth> {
    Json(state.generate_manager.health())
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
// Template Endpoints
// ============================================================================

/// Template info for API response.
#[derive(Debug, Serialize)]
pub struct TemplateInfo {
    /// Template filename.
    pub filename: String,
    /// Template name.
    pub name: String,
    /// Template description.
    pub description: String,
    /// Template author.
    pub author: String,
    /// Template tags.
    pub tags: Vec<String>,
    /// Creation timestamp (RFC 3339).
    pub created: String,
    /// Number of layers in template.
    pub layer_count: usize,
}

/// Template list response.
#[derive(Debug, Serialize)]
pub struct TemplateListResponse {
    /// List of templates.
    pub templates: Vec<TemplateInfo>,
}

/// Template save request.
#[derive(Debug, Deserialize)]
pub struct SaveTemplateRequest {
    /// Template name.
    pub name: String,
    /// Template tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Apply template request.
#[derive(Debug, Deserialize)]
pub struct ApplyTemplateRequest {
    /// Target filename for new layout.
    pub target_filename: String,
}

/// Get the platform-specific template directory.
fn get_template_dir() -> Result<PathBuf, (StatusCode, Json<ApiError>)> {
    Config::config_dir()
        .map(|dir| dir.join("templates"))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::with_details(
                    "Failed to get template directory",
                    e.to_string(),
                )),
            )
        })
}

/// Sanitize a string to be a valid filename.
fn sanitize_template_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '_'
            } else {
                '_'
            }
        })
        .collect()
}

/// GET /api/templates - List all available templates.
async fn list_templates() -> Result<Json<TemplateListResponse>, (StatusCode, Json<ApiError>)> {
    let template_dir = get_template_dir()?;

    // Create template directory if it doesn't exist
    if !template_dir.exists() {
        std::fs::create_dir_all(&template_dir).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::with_details(
                    "Failed to create template directory",
                    e.to_string(),
                )),
            )
        })?;
    }

    let entries = std::fs::read_dir(&template_dir).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to read template directory",
                e.to_string(),
            )),
        )
    })?;

    let mut templates = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            if let Ok(layout) = LayoutService::load(&path) {
                if layout.metadata.is_template {
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown.md")
                        .to_string();

                    templates.push(TemplateInfo {
                        filename,
                        name: layout.metadata.name.clone(),
                        description: layout.metadata.description.clone(),
                        author: layout.metadata.author.clone(),
                        tags: layout.metadata.tags.clone(),
                        created: layout.metadata.created.to_rfc3339(),
                        layer_count: layout.layers.len(),
                    });
                }
            }
        }
    }

    // Sort by name
    templates.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Json(TemplateListResponse { templates }))
}

/// GET /api/templates/{filename} - Get a specific template.
async fn get_template(
    Path(filename): Path<String>,
) -> Result<Json<Layout>, (StatusCode, Json<ApiError>)> {
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;
    let template_dir = get_template_dir()?;

    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        filename.to_string()
    } else {
        format!("{filename}.md")
    };

    let path = template_dir.join(&filename);

    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Template not found: {filename}"))),
        ));
    }

    let layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load template",
                e.to_string(),
            )),
        )
    })?;

    if !layout.metadata.is_template {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("File is not a template")),
        ));
    }

    Ok(Json(layout))
}

/// POST /api/layouts/{filename}/save-as-template - Save layout as template.
async fn save_as_template(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(request): Json<SaveTemplateRequest>,
) -> Result<Json<TemplateInfo>, (StatusCode, Json<ApiError>)> {
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;
    let template_dir = get_template_dir()?;

    // Create template directory if it doesn't exist
    if !template_dir.exists() {
        std::fs::create_dir_all(&template_dir).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::with_details(
                    "Failed to create template directory",
                    e.to_string(),
                )),
            )
        })?;
    }

    // Ensure .md extension
    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        filename.to_string()
    } else {
        format!("{filename}.md")
    };

    // Load the source layout from workspace
    let source_path = state.workspace_root.join(&filename);

    if !source_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!(
                "Source layout not found: {filename}"
            ))),
        ));
    }

    let mut layout = LayoutService::load(&source_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load source layout",
                e.to_string(),
            )),
        )
    })?;

    // Update metadata with validation
    // Validate and set name
    if request.name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("Template name cannot be empty")),
        ));
    }
    if request.name.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(format!(
                "Template name exceeds maximum length of 100 bytes (got {})",
                request.name.len()
            ))),
        ));
    }
    layout.metadata.name.clone_from(&request.name);

    // Validate and set tags
    for tag in &request.tags {
        if tag.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError::new("Tag cannot be empty")),
            ));
        }
        if !tag
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError::new(format!(
                    "Invalid tag '{}': must be lowercase ASCII letters, digits, and hyphens only",
                    tag
                ))),
            ));
        }
    }
    layout.metadata.tags = request.tags;
    layout.metadata.is_template = true;
    layout.metadata.modified = chrono::Utc::now();

    // Generate safe filename
    let template_filename = sanitize_template_filename(&request.name);
    let template_path = template_dir.join(format!("{template_filename}.md"));

    // Check if template already exists
    if template_path.exists() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiError::new(format!(
                "Template '{}' already exists",
                request.name
            ))),
        ));
    }

    // Save the template
    LayoutService::save(&layout, &template_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to save template",
                e.to_string(),
            )),
        )
    })?;

    Ok(Json(TemplateInfo {
        filename: format!("{template_filename}.md"),
        name: layout.metadata.name.clone(),
        description: layout.metadata.description.clone(),
        author: layout.metadata.author.clone(),
        tags: layout.metadata.tags.clone(),
        created: layout.metadata.created.to_rfc3339(),
        layer_count: layout.layers.len(),
    }))
}

/// POST /api/templates/{filename}/apply - Apply template to create new layout.
async fn apply_template(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(request): Json<ApplyTemplateRequest>,
) -> Result<Json<Layout>, (StatusCode, Json<ApiError>)> {
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;
    let target_filename = validate_filename(&request.target_filename)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;
    let template_dir = get_template_dir()?;

    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        filename.to_string()
    } else {
        format!("{filename}.md")
    };

    let template_path = template_dir.join(&filename);

    if !template_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Template not found: {filename}"))),
        ));
    }

    // Load the template
    let mut layout = LayoutService::load(&template_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load template",
                e.to_string(),
            )),
        )
    })?;

    if !layout.metadata.is_template {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("File is not a template")),
        ));
    }

    // Update metadata for new layout
    layout.metadata.is_template = false;
    layout.metadata.created = chrono::Utc::now();
    layout.metadata.modified = chrono::Utc::now();

    // Prepare target path
    let target_filename = if std::path::Path::new(target_filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        target_filename.to_string()
    } else {
        format!("{target_filename}.md")
    };

    let target_path = state.workspace_root.join(&target_filename);

    // Check if target already exists
    if target_path.exists() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiError::new(format!(
                "Layout file already exists: {target_filename}"
            ))),
        ));
    }

    // Save the new layout
    LayoutService::save(&layout, &target_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to save layout",
                e.to_string(),
            )),
        )
    })?;

    Ok(Json(layout))
}

// ============================================================================
// Keyboard & Layout Variant Endpoints (Setup Wizard Support)
// ============================================================================

/// Keyboard summary for listing.
#[derive(Debug, Serialize)]
pub struct KeyboardInfo {
    /// Keyboard path (e.g., "crkbd", "splitkb/halcyon/corne").
    pub path: String,
    /// Number of available layout variants.
    pub layout_count: usize,
}

/// Keyboard list response.
#[derive(Debug, Serialize)]
pub struct KeyboardListResponse {
    /// List of keyboards.
    pub keyboards: Vec<KeyboardInfo>,
}

/// Layout variant info.
#[derive(Debug, Serialize)]
pub struct LayoutVariantInfo {
    /// Layout name (e.g., "LAYOUT_split_3x6_3").
    pub name: String,
    /// Number of keys in this layout.
    pub key_count: usize,
}

/// Layout variants response.
#[derive(Debug, Serialize)]
pub struct LayoutVariantsResponse {
    /// Keyboard path.
    pub keyboard: String,
    /// Available layout variants.
    pub variants: Vec<LayoutVariantInfo>,
}

/// Create layout request.
#[derive(Debug, Deserialize)]
pub struct CreateLayoutRequest {
    /// Filename for the new layout (without path).
    pub filename: String,
    /// Layout name (display name).
    pub name: String,
    /// Keyboard path.
    pub keyboard: String,
    /// Layout variant name.
    pub layout_variant: String,
    /// Optional description.
    #[serde(default)]
    pub description: String,
    /// Optional author.
    #[serde(default)]
    pub author: String,
}

/// Switch variant request.
#[derive(Debug, Deserialize)]
pub struct SwitchVariantRequest {
    /// New layout variant name.
    pub layout_variant: String,
}

/// Switch variant response.
#[derive(Debug, Serialize)]
pub struct SwitchVariantResponse {
    /// Updated layout.
    pub layout: Layout,
    /// Number of keys added (if new variant has more keys).
    pub keys_added: usize,
    /// Number of keys removed (if new variant has fewer keys).
    pub keys_removed: usize,
    /// Warning message if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// GET /api/keyboards - List available keyboards by scanning QMK keyboards directory.
///
/// This endpoint scans the QMK firmware keyboards directory without using the QMK CLI,
/// which avoids external dependencies. It identifies valid keyboards by looking for
/// info.json or keyboard.json files.
async fn list_keyboards(
    State(state): State<AppState>,
) -> Result<Json<KeyboardListResponse>, (StatusCode, Json<ApiError>)> {
    // Get QMK path from config
    let qmk_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .clone()
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError::new("QMK firmware path not configured")),
            )
        })?;

    let keyboards_dir = qmk_path.join("keyboards");
    if !keyboards_dir.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("QMK keyboards directory not found")),
        ));
    }

    // Scan for keyboards with layout definitions
    let mut keyboards = Vec::new();
    scan_keyboard_directory(&keyboards_dir, &keyboards_dir, &mut keyboards);

    // Sort by path
    keyboards.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(Json(KeyboardListResponse { keyboards }))
}

/// Recursively scans keyboard directory for valid keyboards.
fn scan_keyboard_directory(
    base_dir: &std::path::Path,
    current_dir: &std::path::Path,
    keyboards: &mut Vec<KeyboardInfo>,
) {
    let Ok(entries) = std::fs::read_dir(current_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Skip hidden directories
        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if dir_name.starts_with('.') || dir_name == "keymaps" {
            continue;
        }

        // Check if this directory has keyboard config files
        let info_json = path.join("info.json");
        let keyboard_json = path.join("keyboard.json");

        if info_json.exists() || keyboard_json.exists() {
            // Get relative path from keyboards directory
            if let Ok(rel_path) = path.strip_prefix(base_dir) {
                let keyboard_path = rel_path.to_string_lossy().replace('\\', "/");

                // Try to get layout count
                let layout_count = get_keyboard_layout_count(&path);

                if layout_count > 0 {
                    keyboards.push(KeyboardInfo {
                        path: keyboard_path,
                        layout_count,
                    });
                }
            }
        }

        // Recurse into subdirectories (but not too deep)
        let depth = current_dir
            .strip_prefix(base_dir)
            .map(|p| p.components().count())
            .unwrap_or(0);
        if depth < 4 {
            scan_keyboard_directory(base_dir, &path, keyboards);
        }
    }
}

/// Gets the number of layouts defined for a keyboard.
fn get_keyboard_layout_count(keyboard_dir: &std::path::Path) -> usize {
    // Try info.json first
    let info_json = keyboard_dir.join("info.json");
    if info_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&info_json) {
            if let Ok(info) = json5::from_str::<parser::keyboard_json::QmkInfoJson>(&content) {
                if !info.layouts.is_empty() {
                    return info.layouts.len();
                }
            }
        }
    }

    // Try keyboard.json
    let keyboard_json = keyboard_dir.join("keyboard.json");
    if keyboard_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&keyboard_json) {
            if let Ok(variant) =
                json5::from_str::<parser::keyboard_json::VariantKeyboardJson>(&content)
            {
                if !variant.layouts.is_empty() {
                    return variant.layouts.len();
                }
            }
        }
    }

    0
}

/// GET /api/keyboards/{keyboard}/layouts - Get layout variants for a keyboard.
async fn list_keyboard_layouts(
    State(state): State<AppState>,
    Path(keyboard): Path<String>,
) -> Result<Json<LayoutVariantsResponse>, (StatusCode, Json<ApiError>)> {
    // Validate keyboard path
    validate_keyboard_path(&keyboard).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Get QMK path from config
    let qmk_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .clone()
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError::new("QMK firmware path not configured")),
            )
        })?;

    // Parse keyboard info.json
    let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(&qmk_path, &keyboard)
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::with_details(
                    format!("Failed to parse keyboard info for '{keyboard}'"),
                    e.to_string(),
                )),
            )
        })?;

    // Extract layout variants
    let variants: Vec<LayoutVariantInfo> =
        parser::keyboard_json::extract_layout_variants(&keyboard_info)
            .into_iter()
            .map(|v| LayoutVariantInfo {
                name: v.name,
                key_count: v.key_count,
            })
            .collect();

    Ok(Json(LayoutVariantsResponse { keyboard, variants }))
}

/// POST /api/layouts - Create a new layout.
async fn create_layout(
    State(state): State<AppState>,
    Json(request): Json<CreateLayoutRequest>,
) -> Result<Json<Layout>, (StatusCode, Json<ApiError>)> {
    // Validate filename
    let filename =
        validate_filename(&request.filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .md extension
    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        filename.to_string()
    } else {
        format!("{filename}.md")
    };

    // Validate keyboard path
    validate_keyboard_path(&request.keyboard).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Check if file already exists
    let target_path = state.workspace_root.join(&filename);
    if target_path.exists() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiError::new(format!(
                "Layout file already exists: {filename}"
            ))),
        ));
    }

    // Get QMK path from config
    let qmk_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .clone()
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError::new("QMK firmware path not configured")),
            )
        })?;

    // Parse keyboard info to get geometry
    let keyboard_info =
        parser::keyboard_json::parse_keyboard_info_json(&qmk_path, &request.keyboard).map_err(
            |e| {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiError::with_details(
                        format!("Failed to parse keyboard info for '{}'", request.keyboard),
                        e.to_string(),
                    )),
                )
            },
        )?;

    // Validate layout variant exists
    let layout_def = keyboard_info
        .layouts
        .get(&request.layout_variant)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::new(format!(
                    "Layout variant '{}' not found in keyboard '{}'",
                    request.layout_variant, request.keyboard
                ))),
            )
        })?;

    // Create layout with proper geometry
    let key_count = layout_def.layout.len();
    let now = chrono::Utc::now();

    use crate::models::{
        IdleEffectSettings, KeyDefinition, Layer, LayoutMetadata, Position, RgbBrightness,
        RgbColor, RgbOverlayRippleSettings, RgbSaturation, TapHoldSettings, UncoloredKeyBehavior,
    };

    // Build key definitions from geometry
    let mut base_keys = Vec::with_capacity(key_count);
    for key_pos in layout_def.layout.iter() {
        let matrix = key_pos.matrix.unwrap_or([0, 0]);
        base_keys.push(KeyDefinition {
            position: Position {
                row: matrix[0],
                col: matrix[1],
            },
            keycode: "KC_NO".to_string(),
            label: None,
            color_override: None,
            category_id: None,
            combo_participant: false,
            description: None,
        });
    }

    // Create base layer
    let base_layer = Layer {
        number: 0,
        name: "Base".to_string(),
        id: uuid::Uuid::new_v4().to_string(),
        default_color: RgbColor::new(255, 255, 255),
        category_id: None,
        keys: base_keys,
        layer_colors_enabled: true,
    };

    let metadata = LayoutMetadata {
        name: request.name,
        description: request.description,
        author: request.author,
        created: now,
        modified: now,
        tags: vec![],
        is_template: false,
        version: "1.0".to_string(),
        layout_variant: Some(request.layout_variant),
        keyboard: Some(request.keyboard),
        keymap_name: Some("default".to_string()),
        output_format: Some("uf2".to_string()),
    };

    let layout = Layout {
        metadata,
        layers: vec![base_layer],
        categories: vec![],
        rgb_enabled: true,
        rgb_brightness: RgbBrightness::default(),
        rgb_saturation: RgbSaturation::default(),
        rgb_matrix_default_speed: 127,
        rgb_timeout_ms: 0,
        uncolored_key_behavior: UncoloredKeyBehavior::default(),
        idle_effect_settings: IdleEffectSettings::default(),
        rgb_overlay_ripple: RgbOverlayRippleSettings::default(),
        tap_hold_settings: TapHoldSettings::default(),
        combo_settings: ComboSettings::default(),
        tap_dances: vec![],
    };

    // Save the layout
    LayoutService::save(&layout, &target_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to save layout",
                e.to_string(),
            )),
        )
    })?;

    Ok(Json(layout))
}

/// POST /api/layouts/{filename}/switch-variant - Switch layout to a different variant.
///
/// This endpoint performs an authoritative transformation:
/// - Updates metadata.layout_variant
/// - Rebuilds geometry based on new variant
/// - Adjusts all layers to new key count (preserves existing keys where possible)
async fn switch_layout_variant(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(request): Json<SwitchVariantRequest>,
) -> Result<Json<SwitchVariantResponse>, (StatusCode, Json<ApiError>)> {
    // Validate filename
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

    // Load existing layout
    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Layout file not found: {filename}"))),
        ));
    }

    let mut layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load layout",
                e.to_string(),
            )),
        )
    })?;

    // Get keyboard from layout metadata
    let keyboard = layout.metadata.keyboard.clone().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(
                "Layout has no keyboard defined - cannot switch variant",
            )),
        )
    })?;

    // Get QMK path from config
    let qmk_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .clone()
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError::new("QMK firmware path not configured")),
            )
        })?;

    // Parse keyboard info.json
    let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(&qmk_path, &keyboard)
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::with_details(
                    format!("Failed to parse keyboard info for '{keyboard}'"),
                    e.to_string(),
                )),
            )
        })?;

    // Validate new layout variant exists
    let new_layout_def = keyboard_info
        .layouts
        .get(&request.layout_variant)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::new(format!(
                    "Layout variant '{}' not found in keyboard '{keyboard}'",
                    request.layout_variant
                ))),
            )
        })?;

    let new_key_count = new_layout_def.layout.len();
    let old_key_count = layout.layers.first().map_or(0, |l| l.keys.len());

    // Calculate keys added/removed
    let keys_added = new_key_count.saturating_sub(old_key_count);
    let keys_removed = old_key_count.saturating_sub(new_key_count);

    // Build warning message if keys are being lost
    let warning = if keys_removed > 0 {
        Some(format!(
            "Layout variant has fewer keys ({new_key_count} vs {old_key_count}). \
             {keys_removed} keys were removed from each layer."
        ))
    } else {
        None
    };

    // Update metadata
    layout.metadata.layout_variant = Some(request.layout_variant.clone());
    layout.metadata.modified = chrono::Utc::now();

    // Adjust each layer to new key count
    use crate::models::{KeyDefinition, Position};

    for layer in &mut layout.layers {
        if new_key_count > layer.keys.len() {
            // Add new keys with default keycodes
            for idx in layer.keys.len()..new_key_count {
                let key_pos = &new_layout_def.layout[idx];
                let matrix = key_pos.matrix.unwrap_or([0, 0]);
                layer.keys.push(KeyDefinition {
                    position: Position {
                        row: matrix[0],
                        col: matrix[1],
                    },
                    keycode: "KC_NO".to_string(),
                    label: None,
                    color_override: None,
                    category_id: None,
                    combo_participant: false,
                    description: None,
                });
            }
        } else if new_key_count < layer.keys.len() {
            // Truncate keys (preserves first N keys)
            layer.keys.truncate(new_key_count);
        }

        // Update matrix positions based on new layout geometry
        for (idx, key) in layer.keys.iter_mut().enumerate() {
            if idx < new_layout_def.layout.len() {
                let key_pos = &new_layout_def.layout[idx];
                if let Some(matrix) = key_pos.matrix {
                    key.position.row = matrix[0];
                    key.position.col = matrix[1];
                }
            }
        }
    }

    // Save the updated layout
    LayoutService::save(&layout, &path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to save layout",
                e.to_string(),
            )),
        )
    })?;

    Ok(Json(SwitchVariantResponse {
        layout,
        keys_added,
        keys_removed,
        warning,
    }))
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
        .route(
            "/api/layouts/{filename}/swap-keys",
            axum::routing::post(swap_keys),
        )
        .route("/api/layouts/{filename}/validate", get(validate_layout))
        .route("/api/layouts/{filename}/inspect", get(inspect_layout))
        .route("/api/layouts/{filename}/export", get(export_layout))
        .route(
            "/api/layouts/{filename}/render-metadata",
            get(get_render_metadata),
        )
        .route(
            "/api/layouts/{filename}/generate",
            axum::routing::post(generate_firmware),
        )
        .route(
            "/api/layouts/{filename}/save-as-template",
            axum::routing::post(save_as_template),
        )
        // Template endpoints
        .route("/api/templates", get(list_templates))
        .route("/api/templates/{filename}", get(get_template))
        .route(
            "/api/templates/{filename}/apply",
            axum::routing::post(apply_template),
        )
        // Keycode endpoints
        .route("/api/keycodes", get(list_keycodes))
        .route("/api/keycodes/categories", get(list_categories))
        // Config endpoints
        .route("/api/config", get(get_config).put(update_config))
        // Preflight endpoint for onboarding
        .route("/api/preflight", get(get_preflight))
        // Effects endpoint
        .route("/api/effects", get(list_effects))
        // Geometry endpoint
        .route(
            "/api/keyboards/{keyboard}/geometry/{layout}",
            get(get_geometry),
        )
        // Keyboard & Setup Wizard endpoints
        .route("/api/keyboards", get(list_keyboards))
        .route(
            "/api/keyboards/{keyboard}/layouts",
            get(list_keyboard_layouts),
        )
        .route("/api/layouts", axum::routing::post(create_layout))
        .route(
            "/api/layouts/{filename}/switch-variant",
            axum::routing::post(switch_layout_variant),
        )
        // Build job endpoints
        .route("/api/build/start", axum::routing::post(start_build))
        .route("/api/build/jobs", get(list_build_jobs))
        .route("/api/build/jobs/{job_id}", get(get_build_job))
        .route("/api/build/jobs/{job_id}/logs", get(get_build_logs))
        .route(
            "/api/build/jobs/{job_id}/cancel",
            axum::routing::post(cancel_build_job),
        )
        .route(
            "/api/build/jobs/{job_id}/artifacts",
            get(get_build_artifacts),
        )
        .route(
            "/api/build/jobs/{job_id}/artifacts/{artifact_id}/download",
            get(download_build_artifact),
        )
        // Generate job endpoints
        .route("/api/generate/jobs", get(list_generate_jobs))
        .route("/api/generate/jobs/{job_id}", get(get_generate_job))
        .route("/api/generate/jobs/{job_id}/logs", get(get_generate_logs))
        .route(
            "/api/generate/jobs/{job_id}/cancel",
            axum::routing::post(cancel_generate_job),
        )
        .route(
            "/api/generate/jobs/{job_id}/download",
            get(download_generate_zip),
        )
        .route("/api/generate/health", get(get_generate_health))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        // Fallback for static files and SPA routing (must be last)
        // This serves embedded frontend files or returns 404 if not embedded (dev mode)
        .fallback(static_handler)
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
