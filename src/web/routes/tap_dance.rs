//! Tap dance validator endpoint.
//!
//! Mirrors `lazyqmk tap-dance validate` (src/cli/tap_dance.rs:286). Reports
//! orphaned TD() references (used in layers but no definition exists) and
//! unused definitions (defined but never referenced).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;

use crate::services::LayoutService;

use super::super::error::AppError;
use super::super::validation::{validate_filename, with_json_ext};
use super::super::AppState;

/// GET /api/layouts/{filename}/tap-dance/validate
pub(super) async fn tap_dance_validate(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<TapDanceValidateResponse>, AppError> {
    let filename = validate_filename(&filename)?;
    let filename = with_json_ext(filename);
    let path = state.workspace_root.join(&filename);

    if !path.exists() {
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

    let td_pattern = Regex::new(r"TD\(([^)]+)\)").unwrap();
    let mut referenced: HashSet<String> = HashSet::new();

    for layer in &layout.layers {
        for key in &layer.keys {
            if let Some(captures) = td_pattern.captures(&key.keycode) {
                referenced.insert(captures[1].to_string());
            }
        }
    }

    let defined: HashSet<String> = layout
        .tap_dances
        .iter()
        .map(|td| td.name.clone())
        .collect();

    let orphaned: Vec<String> = referenced
        .iter()
        .filter(|name| !defined.contains(*name))
        .cloned()
        .collect();

    let unused: Vec<String> = layout
        .get_orphaned_tap_dances();

    Ok(Json(TapDanceValidateResponse {
        valid: orphaned.is_empty(),
        total_defined: defined.len(),
        total_referenced: referenced.len(),
        orphaned,
        unused,
    }))
}

/// Response shape.
#[derive(Debug, Serialize)]
pub struct TapDanceValidateResponse {
    /// True if no orphaned references (unused definitions are still warnings, not errors).
    pub valid: bool,
    /// Number of defined tap dances.
    pub total_defined: usize,
    /// Number of distinct tap dance names referenced from layers.
    pub total_referenced: usize,
    /// Names used in TD() but missing from layout.tap_dances (errors).
    pub orphaned: Vec<String>,
    /// Names defined but never referenced (warnings).
    pub unused: Vec<String>,
}