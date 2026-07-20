//! Layout inspection endpoint.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use regex::Regex;

use crate::services::LayoutService;

use super::super::dto::{
    InspectLayer, InspectMetadata, InspectResponse, InspectSettings, InspectTapDance,
};
use super::super::error::AppError;
use super::super::validation::{validate_filename, with_json_ext};
use super::super::AppState;

/// GET /api/layouts/{filename}/inspect - Get detailed layout information.
pub(super) async fn inspect_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<InspectResponse>, AppError> {
    let filename = validate_filename(&filename)?;
    let filename = with_json_ext(filename);
    let path = state.workspace_root.join(&filename);

    if !path.exists() {
        return Err(AppError::not_found(format!(
            "Layout file not found: {filename}"
        )));
    }

    let layout = LayoutService::load(&path)
        .map_err(|e| AppError::with_details(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load layout", Some(e.to_string())))?;

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

    let td_pattern = Regex::new(r"TD\(([^)]+)\)").unwrap();
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
