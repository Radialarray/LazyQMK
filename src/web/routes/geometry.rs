//! Keyboard geometry, keyboard listing, and layout variant endpoints.

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use serde::Deserialize;

use crate::models::PaletteFxSettings;
use crate::models::{
    ComboSettings, IdleEffectSettings, KeyDefinition, Layer, Layout, LayoutMetadata, Position,
    RgbBrightness, RgbColor, RgbOverlayRippleSettings, RgbSaturation, TapHoldSettings,
    UncoloredKeyBehavior,
};
use crate::parser;
use crate::services::LayoutService;

use super::super::error::ApiError;
use super::super::validation::{validate_filename, validate_keyboard_path, with_json_ext};
use super::super::AppState;

/// Keyboard geometry response.
#[derive(Debug, Serialize)]
pub(super) struct GeometryResponse {
    /// Keyboard name/path (e.g., "crkbd" or "splitkb/halcyon/corne").
    pub keyboard: String,
    /// Layout variant name (e.g., "`LAYOUT_split_3x6_3`").
    pub layout: String,
    /// List of key geometries.
    pub keys: Vec<KeyGeometryInfo>,
    /// Number of matrix rows.
    pub matrix_rows: u8,
    /// Number of matrix columns.
    pub matrix_cols: u8,
    /// Number of rotary encoders.
    pub encoder_count: u8,
    /// Mapping from visual position ("row,col") to `visual_index` (layout array index).
    /// This allows the frontend to look up the `visual_index` for keys that only have
    /// position data, avoiding brittle coordinate inference logic.
    pub position_to_visual_index: HashMap<String, u8>,
}

/// Key geometry information for API response.
#[derive(Debug, Serialize)]
pub(super) struct KeyGeometryInfo {
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
    /// This matches the `visual_index` in `KeyAssignment` and should be used for mapping keycodes.
    pub visual_index: u8,
}

/// Keyboard summary for listing.
#[derive(Debug, Serialize)]
pub(super) struct KeyboardInfo {
    /// Keyboard path (e.g., "crkbd", "splitkb/halcyon/corne").
    pub path: String,
    /// Number of available layout variants.
    pub layout_count: usize,
}

/// Keyboard list response.
#[derive(Debug, Serialize)]
pub(super) struct KeyboardListResponse {
    /// List of keyboards.
    pub keyboards: Vec<KeyboardInfo>,
}

/// Layout variant info.
#[derive(Debug, Serialize)]
pub(super) struct LayoutVariantInfo {
    /// Layout name (e.g., "`LAYOUT_split_3x6_3`").
    pub name: String,
    /// Number of keys in this layout.
    pub key_count: usize,
}

/// Layout variants response.
#[derive(Debug, Serialize)]
pub(super) struct LayoutVariantsResponse {
    /// Keyboard path.
    pub keyboard: String,
    /// Available layout variants.
    pub variants: Vec<LayoutVariantInfo>,
}

/// GET /api/keyboards/{keyboard}/geometry/{layout} - Get keyboard geometry.
///
/// The keyboard path can contain slashes (e.g., "`keebart/corne_choc_pro`").
pub(super) async fn get_geometry(
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
    let position_to_visual_index: HashMap<String, u8> = geometry
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

/// GET /api/keyboards - List available keyboards by scanning QMK keyboards directory.
pub(super) async fn list_keyboards(
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

/// GET /api/keyboards/{keyboard}/layouts - Get layout variants for a keyboard.
pub(super) async fn list_keyboard_layouts(
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

/// Create layout request.
#[derive(Debug, Deserialize)]
pub(super) struct CreateLayoutRequest {
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
pub(super) struct SwitchVariantRequest {
    /// New layout variant name.
    pub layout_variant: String,
}

/// Switch variant response.
#[derive(Debug, Serialize)]
pub(super) struct SwitchVariantResponse {
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

/// POST /api/layouts - Create a new layout.
pub(super) async fn create_layout(
    State(state): State<AppState>,
    Json(request): Json<CreateLayoutRequest>,
) -> Result<Json<Layout>, (StatusCode, Json<ApiError>)> {
    // Validate filename
    let filename =
        validate_filename(&request.filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .json extension
    let filename = with_json_ext(filename);

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
        palette_fx: PaletteFxSettings::default(),
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
pub(super) async fn switch_layout_variant(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(request): Json<SwitchVariantRequest>,
) -> Result<Json<SwitchVariantResponse>, (StatusCode, Json<ApiError>)> {
    // Validate filename
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .json extension
    let filename = with_json_ext(filename);

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
