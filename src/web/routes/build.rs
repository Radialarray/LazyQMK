//! Build job endpoints.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::services::LayoutService;

use super::super::build_jobs;
use super::super::error::AppError;
use super::super::validation::{validate_filename, with_json_ext};
use super::super::AppState;

/// Query parameters for fetching build logs.
#[derive(Debug, Deserialize)]
pub(super) struct BuildLogsQuery {
    /// Offset to start reading logs from.
    #[serde(default)]
    pub offset: usize,
    /// Maximum number of log lines to return.
    #[serde(default = "default_log_limit")]
    pub limit: usize,
}

fn default_log_limit() -> usize {
    100
}

/// Response for listing build artifacts.
#[derive(Debug, Serialize)]
pub(super) struct BuildArtifactsResponse {
    /// Job ID.
    pub job_id: String,
    /// List of artifacts.
    pub artifacts: Vec<build_jobs::BuildArtifact>,
}

/// POST /api/build/start - Start a firmware build job.
pub(super) async fn start_build(
    State(state): State<AppState>,
    Json(request): Json<build_jobs::StartBuildRequest>,
) -> Result<Json<build_jobs::StartBuildResponse>, AppError> {
    let filename = validate_filename(&request.layout_filename)?;
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

    let keyboard = layout.metadata.keyboard.clone().ok_or_else(|| {
        AppError::bad_request("Layout has no keyboard defined - cannot build firmware")
    })?;

    let keymap = layout
        .metadata
        .keymap_name
        .unwrap_or_else(|| "default".to_string());

    let job = state
        .build_manager
        .start_build(filename, keyboard, keymap, path)
        .map_err(|e| {
            AppError::with_details(StatusCode::SERVICE_UNAVAILABLE, e, Option::<String>::None)
        })?;

    Ok(Json(build_jobs::StartBuildResponse { job }))
}

/// GET /api/build/jobs - List all build jobs.
pub(super) async fn list_build_jobs(
    State(state): State<AppState>,
) -> Json<Vec<build_jobs::BuildJob>> {
    Json(state.build_manager.list_jobs())
}

/// GET /api/build/jobs/{job_id} - Get build job status.
pub(super) async fn get_build_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<build_jobs::JobStatusResponse>, AppError> {
    let job = state
        .build_manager
        .get_job(&job_id)
        .ok_or_else(|| AppError::not_found(format!("Build job not found: {job_id}")))?;

    Ok(Json(build_jobs::JobStatusResponse { job }))
}

/// GET /api/build/jobs/{job_id}/logs - Get build job logs.
pub(super) async fn get_build_logs(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Query(query): Query<BuildLogsQuery>,
) -> Result<Json<build_jobs::JobLogsResponse>, AppError> {
    let logs = state
        .build_manager
        .get_logs(&job_id, query.offset, query.limit)
        .ok_or_else(|| AppError::not_found(format!("Build job not found: {job_id}")))?;

    Ok(Json(logs))
}

/// POST /api/build/jobs/{job_id}/cancel - Cancel a build job.
pub(super) async fn cancel_build_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Json<build_jobs::CancelJobResponse> {
    Json(state.build_manager.cancel_job(&job_id))
}

/// GET /api/build/jobs/{job_id}/artifacts - List artifacts for a build job.
pub(super) async fn get_build_artifacts(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<BuildArtifactsResponse>, AppError> {
    let artifacts = state
        .build_manager
        .get_artifacts(&job_id)
        .ok_or_else(|| AppError::not_found(format!("Build job not found: {job_id}")))?;

    Ok(Json(BuildArtifactsResponse { job_id, artifacts }))
}

/// GET /api/build/jobs/{job_id}/artifacts/{artifact_id}/download - Download a build artifact.
pub(super) async fn download_build_artifact(
    State(state): State<AppState>,
    Path((job_id, artifact_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let artifact_path = state
        .build_manager
        .get_artifact_path(&job_id, &artifact_id)
        .ok_or_else(|| {
            AppError::not_found(format!(
                "Artifact '{artifact_id}' not found for job '{job_id}'"
            ))
        })?;

    if !artifact_path.exists() {
        return Err(AppError::not_found("Artifact file not found on disk"));
    }

    let file_content = std::fs::read(&artifact_path).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read artifact file",
            Some(e.to_string()),
        )
    })?;

    let filename = artifact_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("firmware");

    let content_type = match artifact_id.as_str() {
        "uf2" | "bin" => "application/octet-stream",
        "hex" => "text/plain",
        _ => "application/octet-stream",
    };

    let response = (
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{filename}\""),
            ),
        ],
        Body::from(file_content),
    )
        .into_response();

    Ok(response)
}
