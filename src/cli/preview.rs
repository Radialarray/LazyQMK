//! `lazyqmk preview` — render a single layer as an ASCII/Unicode sketch.
//!
//! Used by agents to embed inline code-block previews in chat next to a
//! proposed keycode change so the user can see where the mutation will land
//! before it runs.

use crate::cli::common::{CliError, CliResult};
use crate::config::Config;
use crate::export::render_layer_diagram_with_markers;
use crate::services::geometry;
use crate::services::LayoutService;
use clap::Args;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Render a single layer of a layout as an ASCII/Unicode sketch.
#[derive(Debug, Clone, Args)]
pub struct PreviewArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Path to QMK firmware repository
    #[arg(long, value_name = "PATH")]
    pub qmk_path: PathBuf,

    /// QMK layout variant (auto-detected from metadata if omitted)
    #[arg(long, value_name = "NAME")]
    pub layout_name: Option<String>,

    /// Layer index to render (default 0)
    #[arg(long, value_name = "N", default_value = "0")]
    pub layer: usize,

    /// Highlight marker in `(row,col)=CHAR` form, e.g. `(0,2)=B`. Repeatable.
    #[arg(long = "highlight", value_name = "(R,C)=CHAR", value_parser = parse_highlight)]
    pub highlights: Vec<((u8, u8), char)>,

    /// Optional legend text printed after the diagram
    #[arg(long, value_name = "TEXT")]
    pub legend: Option<String>,

    /// Output as JSON: `{"diagram", "legend", "layer", "layer_name", "highlights"}`
    #[arg(long)]
    pub json: bool,
}

/// Parsed highlight entry for JSON output.
#[derive(Debug, Serialize)]
struct HighlightEntry {
    position: [u8; 2],
    marker: String,
}

/// JSON response shape.
#[derive(Debug, Serialize)]
struct PreviewResponse {
    diagram: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    legend: Option<String>,
    layer: usize,
    layer_name: String,
    highlights: Vec<HighlightEntry>,
}

/// Parses a `--highlight` value of the form `(r,c)=X` where X is a single char.
fn parse_highlight(s: &str) -> Result<((u8, u8), char), String> {
    let (pos, marker) = s.split_once('=').ok_or_else(|| {
        format!("expected '(row,col)=CHAR' (e.g. '(0,2)=B'), got: {s}")
    })?;

    let pos = pos.trim();
    let pos = pos
        .strip_prefix('(')
        .and_then(|p| p.strip_suffix(')'))
        .ok_or_else(|| format!("expected '(row,col)', got: {pos}"))?;
    let (r, c) = pos
        .split_once(',')
        .ok_or_else(|| format!("expected 'row,col', got: {pos}"))?;
    let row: u8 = r
        .trim()
        .parse()
        .map_err(|e| format!("invalid row '{r}': {e}"))?;
    let col: u8 = c
        .trim()
        .parse()
        .map_err(|e| format!("invalid col '{c}': {e}"))?;

    let marker = marker.trim();
    let mut chars = marker.chars();
    let ch = chars.next().ok_or_else(|| "marker char is empty".to_string())?;
    if chars.next().is_some() {
        return Err(format!(
            "marker must be a single character, got: {marker}"
        ));
    }

    Ok(((row, col), ch))
}

impl PreviewArgs {
    /// Execute the preview command.
    pub fn execute(&self) -> CliResult<()> {
        let layout = LayoutService::load(&self.layout)
            .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

        // Build config with QMK path
        let mut config = Config::load().unwrap_or_default();
        config.paths.qmk_firmware = Some(self.qmk_path.clone());

        // Determine layout variant
        let layout_variant = self
            .layout_name
            .clone()
            .or_else(|| layout.metadata.layout_variant.clone())
            .ok_or_else(|| {
                CliError::validation(
                    "Layout variant not specified. Use --layout-name or set in metadata",
                )
            })?;

        // Build geometry
        let geo_context = geometry::GeometryContext {
            config: &config,
            metadata: &layout.metadata,
        };
        let geo_result = geometry::build_geometry_for_layout(geo_context, &layout_variant)
            .map_err(|e| CliError::io(format!("Failed to build geometry: {e}")))?;
        let geometry = geo_result.geometry;

        // Validate layer index
        let layer_name = layout
            .layers
            .get(self.layer)
            .map(|l| l.name.clone())
            .ok_or_else(|| {
                CliError::validation(format!(
                    "Layer index {} out of bounds (layout has {} layer{})",
                    self.layer,
                    layout.layers.len(),
                    if layout.layers.len() == 1 { "" } else { "s" }
                ))
            })?;

        // Build marker map
        let markers: HashMap<(u8, u8), char> = self.highlights.iter().copied().collect();

        let diagram = render_layer_diagram_with_markers(
            &layout,
            self.layer,
            &geometry,
            markers,
            self.legend.as_deref(),
        )
        .map_err(|e| CliError::io(format!("Failed to render preview: {e}")))?;

        if self.json {
            let response = PreviewResponse {
                diagram,
                legend: self.legend.clone(),
                layer: self.layer,
                layer_name,
                highlights: self
                    .highlights
                    .iter()
                    .map(|((row, col), marker)| HighlightEntry {
                        position: [*row, *col],
                        marker: marker.to_string(),
                    })
                    .collect(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
            );
        } else {
            print!("{diagram}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
