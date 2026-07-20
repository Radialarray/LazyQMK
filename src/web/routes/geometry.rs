//! Keyboard geometry, keyboard listing, and layout variant endpoints.

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::models::PaletteFxSettings;
use crate::models::{
    ComboSettings, IdleEffectSettings, KeyDefinition, Layer, Layout, LayoutMetadata, Position,
    RgbBrightness, RgbColor, RgbOverlayRippleSettings, RgbSaturation, TapHoldSettings,
    UncoloredKeyBehavior,
};
use crate::parser;
use crate::services::LayoutService;

use super::super::error::AppError;
use super::super::validation::{validate_filename, validate_keyboard_path, with_json_ext};
use super::super::AppState;

#[derive(Debug, Serialize)]
pub(super) struct GeometryResponse {
    pub keyboard: String,
    pub layout: String,
    pub keys: Vec<KeyGeometryInfo>,
    pub matrix_rows: u8,
    pub matrix_cols: u8,
    pub encoder_count: u8,
    pub position_to_visual_index: HashMap<String, u8>,
}

#[derive(Debug, Serialize)]
pub(super) struct KeyGeometryInfo {
    pub matrix_row: u8,
    pub matrix_col: u8,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub led_index: Option<u8>,
    pub visual_index: u8,
}

#[derive(Debug, Serialize)]
pub(super) struct KeyboardInfo {
    pub path: String,
    pub layout_count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct KeyboardListResponse {
    pub keyboards: Vec<KeyboardInfo>,
}

#[derive(Debug, Serialize)]
pub(super) struct LayoutVariantInfo {
    pub name: String,
    pub key_count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct LayoutVariantsResponse {
    pub keyboard: String,
    pub variants: Vec<LayoutVariantInfo>,
}

/// GET /api/keyboards/{keyboard}/geometry/{layout} - Get keyboard geometry.
pub(super) async fn get_geometry(
    State(state): State<AppState>,
    Path((keyboard, layout)): Path<(String, String)>,
) -> Result<Json<GeometryResponse>, AppError> {
    validate_keyboard_path(&keyboard)?;

    let qmk_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .clone()
        .ok_or_else(|| AppError::bad_request("QMK firmware path not configured"))?;

    let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(&qmk_path, &keyboard)
        .map_err(|e| {
            AppError::with_details(
                StatusCode::NOT_FOUND,
                format!("Failed to parse keyboard info for '{keyboard}'"),
                Some(e.to_string()),
            )
        })?;

    let _layout_def = keyboard_info
        .layouts
        .get(&layout)
        .ok_or_else(|| {
            AppError::not_found(format!(
                "Layout '{layout}' not found in keyboard '{keyboard}'"
            ))
        })?;

    let geometry = parser::keyboard_json::build_keyboard_geometry_with_rgb(
        &keyboard_info,
        &keyboard,
        &layout,
        None,
    )
    .map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to build keyboard geometry",
            Some(e.to_string()),
        )
    })?;

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

    let position_to_visual_index: HashMap<String, u8> = geometry
        .keys
        .iter()
        .map(|k| {
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

        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if dir_name.starts_with('.') || dir_name == "keymaps" {
            continue;
        }

        let info_json = path.join("info.json");
        let keyboard_json = path.join("keyboard.json");

        if info_json.exists() || keyboard_json.exists() {
            if let Ok(rel_path) = path.strip_prefix(base_dir) {
                let keyboard_path = rel_path.to_string_lossy().replace('\\', "/");
                let layout_count = get_keyboard_layout_count(&path);
                if layout_count > 0 {
                    keyboards.push(KeyboardInfo {
                        path: keyboard_path,
                        layout_count,
                    });
                }
            }
        }

        let depth = current_dir
            .strip_prefix(base_dir)
            .map(|p| p.components().count())
            .unwrap_or(0);
        if depth < 4 {
            scan_keyboard_directory(base_dir, &path, keyboards);
        }
    }
}

fn get_keyboard_layout_count(keyboard_dir: &std::path::Path) -> usize {
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
) -> Result<Json<KeyboardListResponse>, AppError> {
    let qmk_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .clone()
        .ok_or_else(|| AppError::bad_request("QMK firmware path not configured"))?;

    let keyboards_dir = qmk_path.join("keyboards");
    if !keyboards_dir.exists() {
        return Err(AppError::not_found("QMK keyboards directory not found"));
    }

    let mut keyboards = Vec::new();
    scan_keyboard_directory(&keyboards_dir, &keyboards_dir, &mut keyboards);
    keyboards.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(Json(KeyboardListResponse { keyboards }))
}

/// GET /api/keyboards/{keyboard}/layouts - Get layout variants for a keyboard.
pub(super) async fn list_keyboard_layouts(
    State(state): State<AppState>,
    Path(keyboard): Path<String>,
) -> Result<Json<LayoutVariantsResponse>, AppError> {
    validate_keyboard_path(&keyboard)?;

    let qmk_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .clone()
        .ok_or_else(|| AppError::bad_request("QMK firmware path not configured"))?;

    let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(&qmk_path, &keyboard)
        .map_err(|e| {
            AppError::with_details(
                StatusCode::NOT_FOUND,
                format!("Failed to parse keyboard info for '{keyboard}'"),
                Some(e.to_string()),
            )
        })?;

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

#[derive(Debug, Deserialize)]
pub(super) struct CreateLayoutRequest {
    pub filename: String,
    pub name: String,
    pub keyboard: String,
    pub layout_variant: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct SwitchVariantRequest {
    pub layout_variant: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SwitchVariantResponse {
    pub layout: Layout,
    pub keys_added: usize,
    pub keys_removed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// POST /api/layouts - Create a new layout.
pub(super) async fn create_layout(
    State(state): State<AppState>,
    Json(request): Json<CreateLayoutRequest>,
) -> Result<Json<Layout>, AppError> {
    let filename = validate_filename(&request.filename)?;
    let filename = with_json_ext(filename);
    validate_keyboard_path(&request.keyboard)?;

    let target_path = state.workspace_root.join(&filename);
    if target_path.exists() {
        return Err(AppError::with_details(
            StatusCode::CONFLICT,
            "Layout file already exists",
            Some(format!("Layout file already exists: {filename}")),
        ));
    }

    let qmk_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .clone()
        .ok_or_else(|| AppError::bad_request("QMK firmware path not configured"))?;

    let keyboard_info =
        parser::keyboard_json::parse_keyboard_info_json(&qmk_path, &request.keyboard).map_err(
            |e| {
                AppError::with_details(
                    StatusCode::NOT_FOUND,
                    format!("Failed to parse keyboard info for '{}'", request.keyboard),
                    Some(e.to_string()),
                )
            },
        )?;

    let layout_def = keyboard_info
        .layouts
        .get(&request.layout_variant)
        .ok_or_else(|| {
            AppError::not_found(format!(
                "Layout variant '{}' not found in keyboard '{}'",
                request.layout_variant, request.keyboard
            ))
        })?;

    let key_count = layout_def.layout.len();
    let now = chrono::Utc::now();

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

    LayoutService::save(&layout, &target_path).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save layout",
            Some(e.to_string()),
        )
    })?;

    Ok(Json(layout))
}

/// POST /api/layouts/{filename}/switch-variant - Switch layout to a different variant.
pub(super) async fn switch_layout_variant(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(request): Json<SwitchVariantRequest>,
) -> Result<Json<SwitchVariantResponse>, AppError> {
    let filename = validate_filename(&filename)?;
    let filename = with_json_ext(filename);
    let path = state.workspace_root.join(&filename);

    if !path.exists() {
        return Err(AppError::not_found(format!(
            "Layout file not found: {filename}"
        )));
    }

    let mut layout = LayoutService::load(&path).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load layout",
            Some(e.to_string()),
        )
    })?;

    let keyboard = layout.metadata.keyboard.clone().ok_or_else(|| {
        AppError::bad_request("Layout has no keyboard defined - cannot switch variant")
    })?;

    let qmk_path = state
        .config
        .read()
        .unwrap()
        .paths
        .qmk_firmware
        .clone()
        .ok_or_else(|| AppError::bad_request("QMK firmware path not configured"))?;

    let keyboard_info = parser::keyboard_json::parse_keyboard_info_json(&qmk_path, &keyboard)
        .map_err(|e| {
            AppError::with_details(
                StatusCode::NOT_FOUND,
                format!("Failed to parse keyboard info for '{keyboard}'"),
                Some(e.to_string()),
            )
        })?;

    let new_layout_def = keyboard_info
        .layouts
        .get(&request.layout_variant)
        .ok_or_else(|| {
            AppError::not_found(format!(
                "Layout variant '{}' not found in keyboard '{keyboard}'",
                request.layout_variant
            ))
        })?;

    let new_key_count = new_layout_def.layout.len();
    let old_key_count = layout.layers.first().map_or(0, |l| l.keys.len());

    let keys_added = new_key_count.saturating_sub(old_key_count);
    let keys_removed = old_key_count.saturating_sub(new_key_count);

    let warning = if keys_removed > 0 {
        Some(format!(
            "Layout variant has fewer keys ({new_key_count} vs {old_key_count}). \
             {keys_removed} keys were removed from each layer."
        ))
    } else {
        None
    };

    layout.metadata.layout_variant = Some(request.layout_variant.clone());
    layout.metadata.modified = chrono::Utc::now();

    for layer in &mut layout.layers {
        if new_key_count > layer.keys.len() {
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
            layer.keys.truncate(new_key_count);
        }

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

    LayoutService::save(&layout, &path).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save layout",
            Some(e.to_string()),
        )
    })?;

    Ok(Json(SwitchVariantResponse {
        layout,
        keys_added,
        keys_removed,
        warning,
    }))
}
