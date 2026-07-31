//! Layer references endpoint.
//!
//! Mirrors `lazyqmk layer-refs` (`src/cli/layer_refs.rs`). Computes a per-layer
//! reverse index of all layer-switching keycodes plus transparency-conflict
//! warnings where a hold-like reference points at a non-`KC_TRNS` slot.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::services::layer_refs::{build_layer_ref_index, is_transparent, LayerRefKind};
use crate::services::LayoutService;

use super::super::error::AppError;
use super::super::validation::{validate_filename, with_json_ext};
use super::super::AppState;

/// Single inbound reference (from layer X position Y to target layer).
#[derive(Debug, Serialize)]
pub struct InboundRefDto {
    /// Source layer index (where the key is).
    pub from_layer: usize,
    /// Matrix row of the source key.
    pub row: u8,
    /// Matrix column of the source key.
    pub col: u8,
    /// Human-readable kind name (e.g. "Tap-Hold (LT)").
    pub kind: String,
    /// Full keycode string from the source key.
    pub keycode: String,
}

/// Single transparency-conflict warning.
#[derive(Debug, Serialize)]
pub struct LayerRefWarningDto {
    /// Source layer index of the offending hold-like reference.
    pub from_layer: usize,
    /// Matrix row of the source key.
    pub row: u8,
    /// Matrix column of the source key.
    pub col: u8,
    /// Full keycode of the source hold-like reference.
    pub keycode: String,
    /// Keycode at the target position on the destination layer.
    pub target_keycode: String,
    /// Human-readable warning message.
    pub message: String,
}

/// Per-layer summary of inbound references and warnings.
#[derive(Debug, Serialize)]
pub struct LayerRefLayerDto {
    /// Layer index (0-based).
    pub number: usize,
    /// Layer display name.
    pub name: String,
    /// Number of inbound layer-switching keycodes targeting this layer.
    pub inbound_count: usize,
    /// Inbound reference rows.
    pub inbound_refs: Vec<InboundRefDto>,
    /// Transparency-conflict warnings for this layer.
    pub warnings: Vec<LayerRefWarningDto>,
}

/// Full response shape.
#[derive(Debug, Serialize)]
pub struct LayerRefsResponse {
    /// Per-layer summaries.
    pub layers: Vec<LayerRefLayerDto>,
    /// Total inbound references across all layers.
    pub total_inbound_refs: usize,
    /// Total transparency-conflict warnings across all layers.
    pub total_warnings: usize,
}

/// GET /api/layouts/{filename}/layer-refs
pub(super) async fn layer_refs(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<LayerRefsResponse>, AppError> {
    let filename = validate_filename(&filename)?;
    let filename = with_json_ext(filename);
    let path = state.workspace_root.join(&filename);

    if !crate::services::layouts::layout_exists_at(&path) {
        return Err(AppError::not_found(format!(
            "Layout file not found: {filename}"
        )));
    }

    let layout = LayoutService::load(&path).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load layout",
            Some(e.to_string()),
        )
    })?;

    let index = build_layer_ref_index(&layout.layers);

    let mut layers = Vec::new();
    let mut total_inbound = 0usize;
    let mut total_warnings = 0usize;

    for (layer_idx, layer) in layout.layers.iter().enumerate() {
        let refs = index.get(&layer_idx).cloned().unwrap_or_default();
        total_inbound += refs.len();

        let inbound_refs: Vec<InboundRefDto> = refs
            .iter()
            .map(|r| InboundRefDto {
                from_layer: r.from_layer,
                row: r.position.row,
                col: r.position.col,
                kind: r.kind.display_name().to_string(),
                keycode: r.keycode.clone(),
            })
            .collect();

        let mut warnings = Vec::new();
        for r in &refs {
            if !r.kind.is_hold_like() {
                continue;
            }
            if let Some(target_key) = layer.get_key(r.position) {
                if !is_transparent(&target_key.keycode) {
                    let message = format!(
                        "Non-transparent key ({}) conflicts with hold-like reference from Layer {} {}",
                        target_key.keycode,
                        r.from_layer,
                        r.kind.display_name()
                    );
                    warnings.push(LayerRefWarningDto {
                        from_layer: r.from_layer,
                        row: r.position.row,
                        col: r.position.col,
                        keycode: r.keycode.clone(),
                        target_keycode: target_key.keycode.clone(),
                        message,
                    });
                }
            }
        }
        total_warnings += warnings.len();

        layers.push(LayerRefLayerDto {
            number: layer_idx,
            name: layer.name.clone(),
            inbound_count: inbound_refs.len(),
            inbound_refs,
            warnings,
        });
    }

    // Surface the referenced LayerRefKind enum so consumers can match on it
    // without hardcoding strings. Touched at compile time only.
    let _ = LayerRefKind::Momentary;

    Ok(Json(LayerRefsResponse {
        layers,
        total_inbound_refs: total_inbound,
        total_warnings,
    }))
}