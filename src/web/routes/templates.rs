//! Template endpoints.

use std::path::PathBuf;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::models::Layout;
use crate::services::LayoutService;

use super::super::error::AppError;
use super::super::validation::validate_filename;
use super::super::AppState;

/// Template info for API response.
#[derive(Debug, Serialize)]
pub(super) struct TemplateInfo {
    pub filename: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub created: String,
    pub layer_count: usize,
}

/// Template list response.
#[derive(Debug, Serialize)]
pub(super) struct TemplateListResponse {
    pub templates: Vec<TemplateInfo>,
}

/// Template save request.
#[derive(Debug, Deserialize)]
pub(super) struct SaveTemplateRequest {
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Apply template request.
#[derive(Debug, Deserialize)]
pub(super) struct ApplyTemplateRequest {
    pub target_filename: String,
}

/// Get the platform-specific template directory.
pub(super) fn get_template_dir() -> Result<PathBuf, AppError> {
    Config::config_dir()
        .map(|dir| dir.join("templates"))
        .map_err(|e| {
            AppError::with_details(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get template directory",
                Some(e.to_string()),
            )
        })
}

/// Sanitize a string to be a valid filename.
pub(super) fn sanitize_template_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '_'
            } else {
                '_'
            }
        })
        .collect()
}

/// GET /api/templates - List all available templates.
pub(super) async fn list_templates() -> Result<Json<TemplateListResponse>, AppError> {
    let template_dir = get_template_dir()?;

    if !template_dir.exists() {
        std::fs::create_dir_all(&template_dir).map_err(|e| {
            AppError::with_details(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create template directory",
                Some(e.to_string()),
            )
        })?;
    }

    let entries = std::fs::read_dir(&template_dir).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read template directory",
            Some(e.to_string()),
        )
    })?;

    let mut templates = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|ext| ext == "json" || ext == "md")
        {
            if let Ok(layout) = LayoutService::load(&path) {
                if layout.metadata.is_template {
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown.json")
                        .to_string();

                    templates.push(TemplateInfo {
                        filename,
                        name: layout.metadata.name.clone(),
                        description: layout.metadata.description.clone(),
                        author: layout.metadata.author.clone(),
                        tags: layout.metadata.tags.clone(),
                        created: layout.metadata.created.to_rfc3339(),
                        layer_count: layout.layers.len(),
                    });
                }
            }
        }
    }

    templates.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Json(TemplateListResponse { templates }))
}

/// GET /api/templates/{filename} - Get a specific template.
pub(super) async fn get_template(Path(filename): Path<String>) -> Result<Json<Layout>, AppError> {
    let filename = validate_filename(&filename)?;
    let template_dir = get_template_dir()?;

    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        filename.to_string()
    } else {
        format!("{filename}.json")
    };

    let path = template_dir.join(&filename);

    if !path.exists() {
        return Err(AppError::not_found(format!(
            "Template not found: {filename}"
        )));
    }

    let layout = LayoutService::load(&path).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load template",
            Some(e.to_string()),
        )
    })?;

    if !layout.metadata.is_template {
        return Err(AppError::bad_request("File is not a template"));
    }

    Ok(Json(layout))
}

/// POST /api/templates/{filename}/apply - Apply template to create new layout.
pub(super) async fn apply_template(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(request): Json<ApplyTemplateRequest>,
) -> Result<Json<Layout>, AppError> {
    let filename = validate_filename(&filename)?;
    let target_filename = validate_filename(&request.target_filename)?;
    let template_dir = get_template_dir()?;

    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        filename.to_string()
    } else {
        format!("{filename}.json")
    };

    let template_path = template_dir.join(&filename);

    if !template_path.exists() {
        return Err(AppError::not_found(format!(
            "Template not found: {filename}"
        )));
    }

    let mut layout = LayoutService::load(&template_path).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load template",
            Some(e.to_string()),
        )
    })?;

    if !layout.metadata.is_template {
        return Err(AppError::bad_request("File is not a template"));
    }

    layout.metadata.is_template = false;
    layout.metadata.created = chrono::Utc::now();
    layout.metadata.modified = chrono::Utc::now();

    let target_filename = if std::path::Path::new(target_filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        target_filename.to_string()
    } else {
        format!("{target_filename}.json")
    };

    let target_path = state.workspace_root.join(&target_filename);

    if target_path.exists() {
        return Err(AppError::with_details(
            StatusCode::CONFLICT,
            "Layout file already exists",
            Some(format!("Layout file already exists: {target_filename}")),
        ));
    }

    LayoutService::save(&layout, &target_path).map_err(|e| {
        AppError::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save layout",
            Some(e.to_string()),
        )
    })?;

    Ok(Json(layout))
}
