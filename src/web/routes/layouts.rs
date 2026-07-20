//! Layout CRUD, key swap, firmware generation, template save, render metadata,
//! create layout, and switch variant endpoints.

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::{
    ComboAction, ComboSettings, IdleEffectSettings, KeyDefinition, Layer, Layout, Position,
    RgbColor, RgbOverlayRippleSettings, TapDanceAction, TapHoldSettings,
};
use crate::models::{
    HoldDecisionMode, PaletteFxEffect, PaletteFxPalette, RippleColorMode, TapHoldPreset,
};
use crate::models::{PaletteFxSettings, RgbMatrixEffect};
use crate::parser;

use super::super::dto::{
    ComboActionDto, ComboSettingsDto, IdleEffectSettingsDto, PaletteFxSettingsDto,
    RgbOverlayRippleSettingsDto, TapDanceDto, TapHoldSettingsDto,
};
use crate::services::LayoutService;

use super::super::dto::{
    KeyAssignmentDto, KeyDetailActionDto, KeyDisplayDto, KeyRenderMetadata, LayerDto,
    LayerRenderMetadata, LayoutDto, LayoutListResponse, LayoutSaveDto, LayoutSummary,
    RenderMetadataResponse,
};
use super::super::error::ApiError;
use super::super::validation::{validate_filename, with_json_ext};
use super::super::AppState;
use super::templates::{
    get_template_dir, sanitize_template_filename, SaveTemplateRequest, TemplateInfo,
};

/// Parses a `RippleColorMode` from its display name.
fn parse_ripple_color_mode(name: &str) -> RippleColorMode {
    match name {
        "Fixed Color" | "Fixed" | "fixed" => RippleColorMode::Fixed,
        "Key Color" | "Key Based" | "key_based" => RippleColorMode::KeyBased,
        "Hue Shift" | "hue_shift" => RippleColorMode::HueShift,
        _ => RippleColorMode::default(),
    }
}

/// Parses a `HoldDecisionMode` from its display name.
fn parse_hold_decision_mode(name: &str) -> HoldDecisionMode {
    match name {
        "Default (Timing Only)" | "Default" | "default" => HoldDecisionMode::Default,
        "Permissive Hold" | "permissive_hold" => HoldDecisionMode::PermissiveHold,
        "Hold On Other Key" | "hold_on_other_key_press" => HoldDecisionMode::HoldOnOtherKeyPress,
        _ => HoldDecisionMode::default(),
    }
}

/// Parses a `TapHoldPreset` from its display name.
fn parse_tap_hold_preset(name: &str) -> TapHoldPreset {
    match name {
        "Default" | "default" => TapHoldPreset::Default,
        "Home Row Mods" | "home_row_mods" => TapHoldPreset::HomeRowMods,
        "Responsive" | "responsive" => TapHoldPreset::Responsive,
        "Deliberate" | "deliberate" => TapHoldPreset::Deliberate,
        "Custom" | "custom" => TapHoldPreset::Custom,
        _ => TapHoldPreset::default(),
    }
}

/// Parses a `PaletteFxEffect` from its display name.
fn parse_palette_fx_effect(name: &str) -> PaletteFxEffect {
    PaletteFxEffect::from_name(name).unwrap_or_default()
}

/// Parses a `PaletteFxPalette` from its display name.
fn parse_palette_fx_palette(name: &str) -> PaletteFxPalette {
    PaletteFxPalette::from_name(name).unwrap_or_default()
}

/// Converts a `LayoutSaveDto` (from frontend) back to the internal Layout model.
fn convert_dto_to_layout(dto: LayoutSaveDto) -> Layout {
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
            wave_count: ripple_dto.wave_count,
            wave_delay_ms: ripple_dto.wave_delay_ms,
            color_mode,
            fixed_color: ripple_dto.fixed_color,
            hue_shift_deg: ripple_dto.hue_shift_deg,
            trigger_on_press: ripple_dto.trigger_on_press,
            trigger_on_release: ripple_dto.trigger_on_release,
            ignore_transparent: ripple_dto.ignore_transparent,
            ignore_modifiers: ripple_dto.ignore_modifiers,
            ignore_layer_switch: ripple_dto.ignore_layer_switch,
            key_action_palette: if ripple_dto.key_action_palette.is_empty() {
                None
            } else {
                PaletteFxPalette::from_name(&ripple_dto.key_action_palette)
            },
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
        let combos = combo_dto
            .combos
            .into_iter()
            .map(|c_dto| {
                let action = match c_dto.action {
                    ComboActionDto::DisableEffects => ComboAction::DisableEffects,
                    ComboActionDto::DisableLighting => ComboAction::DisableLighting,
                    ComboActionDto::Bootloader => ComboAction::Bootloader,
                };
                crate::models::ComboDefinition::with_duration(
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

    // Convert PaletteFX settings
    let palette_fx = if let Some(pf_dto) = dto.palette_fx {
        PaletteFxSettings {
            enabled: pf_dto.enabled,
            default_effect: parse_palette_fx_effect(&pf_dto.default_effect),
            default_palette: parse_palette_fx_palette(&pf_dto.default_palette),
            enable_all_effects: pf_dto.enable_all_effects,
            enable_all_palettes: pf_dto.enable_all_palettes,
        }
    } else {
        PaletteFxSettings::default()
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
        palette_fx,
        tap_hold_settings,
        combo_settings,
        tap_dances,
    }
}

/// GET /api/layouts - List all layout files in the workspace.
pub(super) async fn list_layouts(
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

        // Process both .json and legacy .md files
        if path
            .extension()
            .is_some_and(|ext| ext == "json" || ext == "md")
        {
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
pub(super) async fn get_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<LayoutDto>, (StatusCode, Json<ApiError>)> {
    // Validate filename to prevent path traversal
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .json extension
    let filename = with_json_ext(filename);

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
    let qmk_path = state
        .config
        .read()
        .expect("config lock poisoned")
        .paths
        .qmk_firmware
        .clone();
    let position_to_geometry: HashMap<String, (u8, [u8; 2], u8)> =
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
            HashMap::new()
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
        palette_fx: PaletteFxSettingsDto::from(&layout.palette_fx),
        tap_hold_settings: TapHoldSettingsDto::from(&layout.tap_hold_settings),
        tap_dances: layout.tap_dances.iter().map(TapDanceDto::from).collect(),
        combo_settings: ComboSettingsDto::from(&layout.combo_settings),
    };

    Ok(Json(layout_dto))
}

/// PUT /api/layouts/{filename} - Save a layout file.
pub(super) async fn save_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(layout_dto): Json<LayoutSaveDto>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    // Validate filename to prevent path traversal
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .json extension
    let filename = with_json_ext(filename);

    let path = state.workspace_root.join(&filename);

    // Convert LayoutDto back to Layout model
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
pub(super) async fn swap_keys(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(request): Json<crate::web::dto::SwapKeysRequest>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    // Validate filename to prevent path traversal
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .json extension
    let filename = with_json_ext(filename);

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

    // Find indices of both keys by position
    let first_idx = layer.keys.iter().position(|k| {
        k.position.row == request.first_position.row && k.position.col == request.first_position.col
    });
    let second_idx = layer.keys.iter().position(|k| {
        k.position.row == request.second_position.row
            && k.position.col == request.second_position.col
    });

    match (first_idx, second_idx) {
        (Some(idx1), Some(idx2)) => {
            layer.keys.swap(idx1, idx2);

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

/// GET /api/layouts/{filename}/render-metadata - Get key display metadata for rendering.
pub(super) async fn get_render_metadata(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<RenderMetadataResponse>, (StatusCode, Json<ApiError>)> {
    // Validate filename to prevent path traversal
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    // Ensure .json extension
    let filename = with_json_ext(filename);

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
    let qmk_path = state
        .config
        .read()
        .expect("config lock poisoned")
        .paths
        .qmk_firmware
        .clone();
    let position_to_visual_index: HashMap<String, u8> = if let (Some(keyboard), Some(qmk_path)) =
        (layout.metadata.keyboard.as_ref(), qmk_path.as_ref())
    {
        let layout_variant = layout
            .metadata
            .layout_variant
            .clone()
            .unwrap_or_else(|| "LAYOUT".to_string());

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
        HashMap::new()
    };

    // Build tap dance lookup for display info
    let tap_dance_map: HashMap<String, &TapDanceAction> = layout
        .tap_dances
        .iter()
        .map(|td| (td.name.clone(), td))
        .collect();

    // Build layer ID to number mapping for resolving @uuid references
    let layer_id_to_number: HashMap<String, u8> = layout
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
                    let pos_key = format!("{},{}", key.position.row, key.position.col);
                    let visual_index = position_to_visual_index
                        .get(&pos_key)
                        .copied()
                        .unwrap_or(idx as u8);

                    let td_info = state
                        .keycode_db
                        .parse_tap_dance_keycode(&key.keycode)
                        .and_then(|td_name| tap_dance_map.get(&td_name))
                        .map(|td| crate::keycode_db::TapDanceDisplayInfo {
                            single_tap: td.single_tap.clone(),
                            double_tap: td.double_tap.clone(),
                            hold: td.hold.clone(),
                        });

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

/// POST /api/layouts/{filename}/save-as-template - Save layout as template.
pub(super) async fn save_as_template(
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

    // Ensure .json extension
    let filename = with_json_ext(filename);

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
    let template_path = template_dir.join(format!("{template_filename}.json"));

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
        filename: format!("{template_filename}.json"),
        name: layout.metadata.name.clone(),
        description: layout.metadata.description.clone(),
        author: layout.metadata.author.clone(),
        tags: layout.metadata.tags.clone(),
        created: layout.metadata.created.to_rfc3339(),
        layer_count: layout.layers.len(),
    }))
}
