//! Web API routes for layout versioning.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::{LayoutDiff, LayoutRevision, RevisionSummary};
use crate::services::layout_versions::{LayoutVersionService, RevisionEventHook};
use crate::web::app_state::AppState;
use crate::web::error::AppError;
use crate::web::events::LayoutEvent;

/// Builds a service wired to publish `LayoutEvent::RevisionChanged` events.
fn service_with_sse(state: &AppState) -> LayoutVersionService {
    let sender = state.layout_event_sender();
    let hook: RevisionEventHook = Arc::new(move |layout_name, revision, kind| {
        let _ = sender.send(LayoutEvent::RevisionChanged {
            layout_name: layout_name.to_string(),
            revision,
            action: kind.as_str().to_string(),
        });
    });
    LayoutVersionService::new(state.workspace_root.clone()).with_event_hook(hook)
}

/// Query parameters for the list endpoint.
#[derive(Debug, Deserialize)]
pub(super) struct ListVersionsQuery {
    #[serde(default)]
    pub layout: Option<String>,
}

/// Response for the version list endpoint.
#[derive(Debug, Serialize)]
pub(super) struct ListVersionsResponse {
    pub revisions: Vec<RevisionSummary>,
}

/// Request body for creating a snapshot.
#[derive(Debug, Deserialize)]
pub(super) struct CreateSnapshotRequest {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// Request body for renaming a revision.
#[derive(Debug, Deserialize)]
pub(super) struct RenameRevisionRequest {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// Query for diff endpoint.
#[derive(Debug, Deserialize)]
pub(super) struct DiffQuery {
    pub from: u32,
    pub to: u32,
}

/// Response for the diff endpoint.
#[derive(Debug, Serialize)]
pub(super) struct DiffResponse {
    pub diff: LayoutDiff,
}

/// GET /api/versions?layout=NAME — list revisions for a layout.
pub(super) async fn list_revisions(
    State(state): State<AppState>,
    Query(q): Query<ListVersionsQuery>,
) -> Result<Json<ListVersionsResponse>, AppError> {
    let layout_name = q.layout.ok_or_else(|| {
        AppError::bad_request("Missing required query parameter: layout".to_string())
    })?;
    let svc = service_with_sse(&state);
    let revisions = svc.list(&layout_name).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to list revisions",
            Some(e.to_string()),
        )
    })?;
    Ok(Json(ListVersionsResponse { revisions }))
}

/// GET /api/versions/{layout}/{revision} — get a full revision snapshot.
pub(super) async fn get_revision(
    State(state): State<AppState>,
    Path((layout_name, revision)): Path<(String, u32)>,
) -> Result<Json<LayoutRevision>, AppError> {
    let svc = service_with_sse(&state);
    let snapshot = svc.get(&layout_name, revision).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get revision",
            Some(e.to_string()),
        )
    })?;
    Ok(Json(snapshot))
}

/// POST /api/versions/{layout} — create a new snapshot of the current layout.
pub(super) async fn create_snapshot(
    State(state): State<AppState>,
    Path(layout_name): Path<String>,
    Json(req): Json<CreateSnapshotRequest>,
) -> Result<Json<RevisionSummary>, AppError> {
    let layout_path = state.workspace_root.join(format!("{layout_name}.json"));
    let layout = crate::services::LayoutService::load(&layout_path).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load layout",
            Some(e.to_string()),
        )
    })?;
    let svc = service_with_sse(&state);
    let summary = svc
        .create_snapshot(
            &layout_name,
            &layout,
            req.label.as_deref(),
            req.note.as_deref(),
            None,
        )
        .map_err(|e| {
            AppError::with_details(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create snapshot",
                Some(e.to_string()),
            )
        })?;
    Ok(Json(summary))
}

/// POST /api/versions/{layout}/{revision}/restore — restore a revision as the
/// current layout. Auto-snapshots the current layout first.
pub(super) async fn restore_revision(
    State(state): State<AppState>,
    Path((layout_name, revision)): Path<(String, u32)>,
) -> Result<StatusCode, AppError> {
    let svc = service_with_sse(&state);
    svc.restore(&layout_name, revision, None).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to restore revision",
            Some(e.to_string()),
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /api/versions/{layout}/{revision} — rename a revision's label/note.
pub(super) async fn rename_revision(
    State(state): State<AppState>,
    Path((layout_name, revision)): Path<(String, u32)>,
    Json(req): Json<RenameRevisionRequest>,
) -> Result<Json<RevisionSummary>, AppError> {
    let svc = service_with_sse(&state);
    let summary = svc
        .rename(
            &layout_name,
            revision,
            req.label.as_deref(),
            req.note.as_deref(),
        )
        .map_err(|e| {
            AppError::with_details(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to rename revision",
                Some(e.to_string()),
            )
        })?;
    Ok(Json(summary))
}

/// DELETE /api/versions/{layout}/{revision} — delete a revision.
pub(super) async fn delete_revision(
    State(state): State<AppState>,
    Path((layout_name, revision)): Path<(String, u32)>,
) -> Result<StatusCode, AppError> {
    let svc = service_with_sse(&state);
    svc.delete(&layout_name, revision).map_err(|e| {
        if e.to_string().contains("active revision") {
            AppError::with_details(
                StatusCode::CONFLICT,
                "Cannot delete the active revision",
                Some(e.to_string()),
            )
        } else {
            AppError::with_details(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete revision",
                Some(e.to_string()),
            )
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/versions/{layout}/diff?from=N&to=M — diff two revisions.
pub(super) async fn diff_revisions(
    State(state): State<AppState>,
    Path(layout_name): Path<String>,
    Query(q): Query<DiffQuery>,
) -> Result<Json<DiffResponse>, AppError> {
    let svc = service_with_sse(&state);
    let diff = svc.diff(&layout_name, q.from, q.to).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to compute diff",
            Some(e.to_string()),
        )
    })?;
    Ok(Json(DiffResponse { diff }))
}
