//! Layout validation endpoint.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::services::LayoutService;

use super::super::dto::ValidationResponse;
use super::super::error::ApiError;
use super::super::validation::{validate_filename, with_json_ext};
use super::super::AppState;

/// GET /api/layouts/{filename}/validate - Validate a layout.
pub(super) async fn validate_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<ValidationResponse>, (StatusCode, Json<ApiError>)> {
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
