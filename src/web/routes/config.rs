//! Configuration endpoints.

use std::path::PathBuf;

use axum::{extract::State, http::StatusCode, Json};

use super::super::dto::{ConfigResponse, ConfigUpdateRequest, PreflightResponse};
use super::super::error::AppError;
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
) -> Result<StatusCode, AppError> {
    let mut config = (*state.config.read().expect("config lock poisoned")).clone();

    if let Some(path_str) = request.qmk_firmware_path {
        let path = PathBuf::from(path_str);

        if !path.exists() {
            return Err(AppError::bad_request("QMK firmware path does not exist"));
        }

        config.paths.qmk_firmware = Some(path);
    }

    config.validate().map_err(|e| {
        AppError::with_details(
            StatusCode::BAD_REQUEST,
            "Invalid configuration",
            Some(e.to_string()),
        )
    })?;

    config.save().map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save configuration",
            Some(e.to_string()),
        )
    })?;

    *state.config.write().expect("config lock poisoned") = config;

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

    let first_run = !has_layouts && !qmk_configured;

    Json(PreflightResponse {
        qmk_configured,
        has_layouts,
        first_run,
        qmk_firmware_path,
    })
}
