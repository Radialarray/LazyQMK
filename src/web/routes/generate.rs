//! Generate job endpoints.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::services::LayoutService;

use super::super::error::AppError;
use super::super::generate_jobs;
use super::super::validation::{validate_filename, with_json_ext};
use super::super::AppState;

/// POST /api/layouts/{filename}/generate - Generate firmware and return job info.
pub(super) async fn generate_firmware(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<generate_jobs::StartGenerateResponse>, AppError> {
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

    let keyboard = layout.metadata.keyboard.clone().ok_or_else(|| {
        AppError::bad_request("Layout has no keyboard defined - cannot generate firmware")
    })?;

    let layout_variant = layout.metadata.layout_variant.ok_or_else(|| {
        AppError::bad_request("Layout has no layout variant defined - cannot generate firmware")
    })?;

    let job = state
        .generate_manager
        .start_generate(filename.clone(), keyboard, layout_variant)
        .map_err(|e| {
            AppError::with_details(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to start generation",
                Some(e),
            )
        })?;

    Ok(Json(generate_jobs::StartGenerateResponse {
        status: "started".to_string(),
        message: format!("Firmware generation started for {filename}"),
        job,
    }))
}

/// Query parameters for fetching generate logs.
#[derive(Debug, Deserialize)]
pub(super) struct GenerateLogsQuery {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_log_limit")]
    pub limit: usize,
}

fn default_log_limit() -> usize {
    100
}

/// GET /api/generate/jobs - List all generate jobs.
pub(super) async fn list_generate_jobs(
    State(state): State<AppState>,
) -> Json<Vec<generate_jobs::GenerateJob>> {
    Json(state.generate_manager.list_jobs())
}

/// GET /api/generate/jobs/{job_id} - Get generate job status.
pub(super) async fn get_generate_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<generate_jobs::GenerateJobStatusResponse>, AppError> {
    let job = state
        .generate_manager
        .get_job(&job_id)
        .ok_or_else(|| AppError::not_found(format!("Generate job not found: {job_id}")))?;

    Ok(Json(generate_jobs::GenerateJobStatusResponse { job }))
}

/// GET /api/generate/jobs/{job_id}/logs - Get generate job logs.
pub(super) async fn get_generate_logs(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Query(query): Query<GenerateLogsQuery>,
) -> Result<Json<generate_jobs::GenerateJobLogsResponse>, AppError> {
    let logs = state
        .generate_manager
        .get_logs(&job_id, query.offset, query.limit)
        .ok_or_else(|| AppError::not_found(format!("Generate job not found: {job_id}")))?;

    Ok(Json(logs))
}

/// POST /api/generate/jobs/{job_id}/cancel - Cancel a generate job.
pub(super) async fn cancel_generate_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Json<generate_jobs::CancelGenerateJobResponse> {
    Json(state.generate_manager.cancel_job(&job_id))
}

/// GET /api/generate/jobs/{job_id}/download - Download the generated zip file.
pub(super) async fn download_generate_zip(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Response, AppError> {
    let job = state
        .generate_manager
        .get_job(&job_id)
        .ok_or_else(|| AppError::not_found(format!("Generate job not found: {job_id}")))?;

    if job.status != generate_jobs::GenerateJobStatus::Completed {
        return Err(AppError::bad_request(format!(
            "Job is not completed. Current status: {}",
            job.status
        )));
    }

    let zip_path = state
        .generate_manager
        .get_zip_path(&job_id)
        .ok_or_else(|| AppError::internal("Zip file path not found"))?;

    if !zip_path.exists() {
        return Err(AppError::not_found("Zip file no longer exists"));
    }

    let file_content = std::fs::read(&zip_path).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read zip file",
            Some(e.to_string()),
        )
    })?;

    let filename = zip_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("firmware.zip");

    let response = (
        [
            (header::CONTENT_TYPE, "application/zip"),
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

/// GET /api/generate/health - Get generate job system health.
pub(super) async fn get_generate_health(
    State(state): State<AppState>,
) -> Json<generate_jobs::GenerateJobHealth> {
    Json(state.generate_manager.health())
}
