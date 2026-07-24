//! Request/response DTOs and From conversions for the web API.
//!
//! Extracted from src/web/mod.rs as part of LazyQMK-2rf6.2.

use serde::{Deserialize, Serialize};

use crate::keycode_db::{KeycodeCategory, KeycodeDefinition};
use crate::models::{
    ComboSettings, IdleEffectSettings, RgbColor, RgbOverlayRippleSettings, TapDanceAction,
    TapHoldSettings,
};

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
    /// Keycode string (e.g., "`KC_A`").
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
    /// Raw keycode or parameter (e.g., "`KC_A`", "1", "`MOD_LCTL`")
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
    /// Layer-indexed key metadata (`layer_index` -> list of key metadata)
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

/// `PaletteFX` settings for API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteFxSettingsDto {
    /// Master switch for `PaletteFX` effects.
    pub enabled: bool,
    /// Default effect name.
    pub default_effect: String,
    /// Default palette name.
    pub default_palette: String,
    /// Enable all effects at compile time.
    pub enable_all_effects: bool,
    /// Enable all palettes at compile time.
    pub enable_all_palettes: bool,
}

impl From<&crate::models::PaletteFxSettings> for PaletteFxSettingsDto {
    fn from(s: &crate::models::PaletteFxSettings) -> Self {
        Self {
            enabled: s.enabled,
            default_effect: s.default_effect.display_name().to_string(),
            default_palette: s.default_palette.display_name().to_string(),
            enable_all_effects: s.enable_all_effects,
            enable_all_palettes: s.enable_all_palettes,
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
    /// Speed multiplier (0-255, higher = faster expansion in physical LED coordinate space).
    pub speed: u8,
    /// Band width in physical LED distance units.
    pub band_width: u8,
    /// Amplitude as percentage of base brightness (0-100).
    pub amplitude_pct: u8,
    /// Number of concentric waves per keypress (1-5).
    pub wave_count: u8,
    /// Delay between consecutive waves in milliseconds (50-500).
    pub wave_delay_ms: u16,
    /// Color mode for ripples.
    pub color_mode: String,
    /// Fixed color (used when `color_mode` = Fixed).
    pub fixed_color: RgbColor,
    /// Hue shift in degrees (used when `color_mode` = `HueShift`).
    pub hue_shift_deg: i16,
    /// Trigger on key press.
    pub trigger_on_press: bool,
    /// Trigger on key release.
    pub trigger_on_release: bool,
    /// Ignore transparent keys (`KC_TRNS`).
    pub ignore_transparent: bool,
    /// Ignore modifier keys.
    pub ignore_modifiers: bool,
    /// Ignore layer switch keys.
    pub ignore_layer_switch: bool,
    /// `PaletteFX` palette for key-action reactive bursts (display name or empty).
    /// Empty string means use the current palette.
    #[serde(default)]
    pub key_action_palette: String,
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
            wave_count: s.wave_count,
            wave_delay_ms: s.wave_delay_ms,
            color_mode: s.color_mode.display_name().to_string(),
            fixed_color: s.fixed_color,
            hue_shift_deg: s.hue_shift_deg,
            trigger_on_press: s.trigger_on_press,
            trigger_on_release: s.trigger_on_release,
            ignore_transparent: s.ignore_transparent,
            ignore_modifiers: s.ignore_modifiers,
            ignore_layer_switch: s.ignore_layer_switch,
            key_action_palette: s
                .key_action_palette
                .map_or_else(String::new, |p| p.display_name().to_string()),
        }
    }
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
    /// List of combo definitions (max [`crate::models::layout::MAX_COMBOS`]).
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
    /// QMK keycode (e.g., "`KC_A`", "`KC_TRNS`", "MO(1)")
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
    /// `PaletteFX` settings
    pub palette_fx: PaletteFxSettingsDto,
    /// Tap-hold settings
    pub tap_hold_settings: TapHoldSettingsDto,
    /// Tap dance definitions
    pub tap_dances: Vec<TapDanceDto>,
    /// Combo settings
    pub combo_settings: ComboSettingsDto,
}

/// Layout DTO for save requests (accepts optional fields from frontend).
///
/// The frontend sends back the `LayoutDto` it received from GET, but we need to be
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
    /// `PaletteFX` settings
    #[serde(default)]
    pub palette_fx: Option<PaletteFxSettingsDto>,
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
    /// QMK keycode (e.g., "`KC_A`", "`KC_TRNS`", "MO(1)")
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
