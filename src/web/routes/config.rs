//! Configuration endpoints.

use std::path::PathBuf;

use axum::{extract::State, http::StatusCode, Json};

use super::super::dto::{ConfigResponse, ConfigUpdateRequest, PreflightResponse};
use super::super::error::ApiError;
use super::super::AppState;

/// GET /api/config - Get current configuration.
pub(super) async fn get_config(State(state): State<AppState>) -> Json<ConfigResponse> {
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
pub(super) async fn update_config(
    State(state): State<AppState>,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    // Create a mutable copy of the config
    let mut config = (*state.config.read().expect("config lock poisoned")).clone();

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

    *state.config.write().expect("config lock poisoned") = config;

    // Propagate QMK path update to build and generate managers
    let new_qmk_path = state
        .config
        .read()
        .expect("config lock poisoned")
        .paths
        .qmk_firmware
        .clone();
    state.build_manager.set_qmk_path(new_qmk_path.clone());
    state.generate_manager.set_qmk_path(new_qmk_path);

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/preflight - Check application state for onboarding flow.
pub(super) async fn get_preflight(State(state): State<AppState>) -> Json<PreflightResponse> {
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
            entries.filter_map(Result::ok).any(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext == "json" || ext == "md")
            })
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
