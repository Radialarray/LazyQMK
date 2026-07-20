//! Layout validation endpoint.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::services::LayoutService;

use super::super::dto::ValidationResponse;
use super::super::error::AppError;
use super::super::validation::{validate_filename, with_json_ext};
use super::super::AppState;

/// GET /api/layouts/{filename}/validate - Validate a layout.
pub(super) async fn validate_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<ValidationResponse>, AppError> {
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

    let mut warnings = Vec::new();
    for name in &layout.get_orphaned_tap_dances() {
        warnings.push(format!("Tap dance '{name}' is defined but not used"));
    }

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
