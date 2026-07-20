//! Layout export endpoint.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::export;
use crate::models::Layout;
use crate::parser;
use crate::services::LayoutService;

use super::super::dto::ExportResponse;
use super::super::error::ApiError;
use super::super::validation::validate_filename;
use super::super::AppState;

/// Generate simple markdown without geometry diagrams.
fn generate_simple_markdown(layout: &Layout) -> String {
    use std::fmt::Write;
    let mut output = String::new();

    let _ = writeln!(output, "# {}\n", layout.metadata.name);
    let _ = writeln!(output, "**Description:** {}\n", layout.metadata.description);
    let _ = writeln!(output, "**Author:** {}\n", layout.metadata.author);

    if let Some(keyboard) = &layout.metadata.keyboard {
        let _ = writeln!(output, "**Keyboard:** {keyboard}\n");
    }

    let _ = writeln!(output, "## Layers\n");
    for layer in &layout.layers {
        let _ = writeln!(output, "### Layer {}: {}\n", layer.number, layer.name);
        let _ = writeln!(output, "- Keys: {}", layer.keys.len());
        let _ = writeln!(
            output,
            "- Color: #{:02X}{:02X}{:02X}\n",
            layer.default_color.r, layer.default_color.g, layer.default_color.b
        );
    }

    if !layout.tap_dances.is_empty() {
        let _ = writeln!(output, "## Tap Dances\n");
        for td in &layout.tap_dances {
            let _ = writeln!(output, "- **{}**: tap={}", td.name, td.single_tap);
        }
    }

    output
}

/// GET /api/layouts/{filename}/export - Export layout to markdown.
pub(super) async fn export_layout(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<ExportResponse>, (StatusCode, Json<ApiError>)> {
    // Validate filename
    let filename = validate_filename(&filename).map_err(|e| (StatusCode::BAD_REQUEST, Json(e)))?;

    let filename = if std::path::Path::new(filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        filename.to_string()
    } else {
        format!("{filename}.json")
    };

    let path = state.workspace_root.join(&filename);

    if !path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new(format!("Layout file not found: {filename}"))),
        ));
    }

    let layout = LayoutService::load(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::with_details(
                "Failed to load layout",
                e.to_string(),
            )),
        )
    })?;

    // Get keyboard geometry for export
    let geometry = if let (Some(keyboard), Some(layout_variant)) = (
        layout.metadata.keyboard.as_ref(),
        layout.metadata.layout_variant.as_ref(),
    ) {
        let qmk_path = state
            .config
            .read()
            .expect("config lock poisoned")
            .paths
            .qmk_firmware
            .clone();
        if let Some(qmk_path) = qmk_path {
            parser::keyboard_json::parse_keyboard_info_json(&qmk_path, keyboard)
                .ok()
                .and_then(|info| {
                    parser::keyboard_json::build_keyboard_geometry_with_rgb(
                        &info,
                        keyboard,
                        layout_variant,
                        None,
                    )
                    .ok()
                })
        } else {
            None
        }
    } else {
        None
    };

    // Generate markdown (with or without geometry)
    let markdown = if let Some(geom) = geometry {
        export::export_to_markdown(&layout, &geom, &state.keycode_db).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::with_details(
                    "Failed to export layout",
                    e.to_string(),
                )),
            )
        })?
    } else {
        // Generate simpler markdown without geometry
        generate_simple_markdown(&layout)
    };

    // Generate suggested filename
    let layout_name = layout
        .metadata
        .name
        .to_lowercase()
        .replace(' ', "_")
        .replace(|c: char| !c.is_alphanumeric() && c != '_', "");
    let date = chrono::Utc::now().format("%Y%m%d");
    let suggested_filename = format!("{layout_name}_export_{date}.md");

    Ok(Json(ExportResponse {
        markdown,
        suggested_filename,
    }))
}
