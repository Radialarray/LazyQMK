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
use super::super::error::ApiError;
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
) -> Result<Json<build_jobs::StartBuildResponse>, (StatusCode, Json<ApiError>)> {
    // Validate layout filename
    let filename = validate_filename(&request.layout_filename)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

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

    // Load the layout to get keyboard/keymap info
    let layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load layout",
                e.to_string(),
            )),
        )
    })?;

    // Get keyboard and keymap from layout metadata
    let keyboard = layout.metadata.keyboard.clone().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError::new(
                "Layout has no keyboard defined - cannot build firmware",
            )),
        )
    })?;

    let keymap = layout
        .metadata
        .keymap_name
        .unwrap_or_else(|| "default".to_string());

    // Start the build job
    let job = state
        .build_manager
        .start_build(filename, keyboard, keymap, path)
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, Json(ApiError::new(e))))?;

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
) -> Result<Json<build_jobs::JobStatusResponse>, (StatusCode, Json<ApiError>)> {
    let job = state.build_manager.get_job(&job_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Build job not found: {job_id}"))),
        )
    })?;

    Ok(Json(build_jobs::JobStatusResponse { job }))
}

/// GET /api/build/jobs/{job_id}/logs - Get build job logs.
pub(super) async fn get_build_logs(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Query(query): Query<BuildLogsQuery>,
) -> Result<Json<build_jobs::JobLogsResponse>, (StatusCode, Json<ApiError>)> {
    let logs = state
        .build_manager
        .get_logs(&job_id, query.offset, query.limit)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::new(format!("Build job not found: {job_id}"))),
            )
        })?;

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
) -> Result<Json<BuildArtifactsResponse>, (StatusCode, Json<ApiError>)> {
    let artifacts = state.build_manager.get_artifacts(&job_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Build job not found: {job_id}"))),
        )
    })?;

    Ok(Json(BuildArtifactsResponse { job_id, artifacts }))
}

/// GET /api/build/jobs/{job_id}/artifacts/{artifact_id}/download - Download a build artifact.
pub(super) async fn download_build_artifact(
    State(state): State<AppState>,
    Path((job_id, artifact_id)): Path<(String, String)>,
) -> Result<Response, (StatusCode, Json<ApiError>)> {
    // Get the artifact path (includes security validation)
    let artifact_path = state
        .build_manager
        .get_artifact_path(&job_id, &artifact_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::new(format!(
                    "Artifact '{artifact_id}' not found for job '{job_id}'"
                ))),
            )
        })?;

    // Check file exists
    if !artifact_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Artifact file not found on disk")),
        ));
    }

    // Read the file
    let file_content = std::fs::read(&artifact_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to read artifact file",
                e.to_string(),
            )),
        )
    })?;

    // Get filename for Content-Disposition header
    let filename = artifact_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("firmware");

    // Determine content type based on extension
    let content_type = match artifact_id.as_str() {
        "uf2" => "application/octet-stream",
        "bin" => "application/octet-stream",
        "hex" => "text/plain",
        _ => "application/octet-stream",
    };

    // Build response with proper headers for file download
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
