//! Validation command for layout files.

use crate::cli::common::{
    CliError, CliResult, ValidationChecks, ValidationLocation, ValidationMessage,
    ValidationPosition, ValidationResponse,
};
use crate::config::Config;
use crate::firmware::validator::FirmwareValidator;
use crate::keycode_db::KeycodeDb;
use crate::models::keyboard_geometry::KeyboardGeometry;
use crate::models::visual_layout_mapping::VisualLayoutMapping;
use crate::services::{geometry, LayoutService};
use clap::Args;
use std::path::PathBuf;

/// Validate a layout file for errors and warnings
#[derive(Debug, Clone, Args)]
pub struct ValidateArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,

    /// Treat warnings as errors (exit non-zero)
    #[arg(long)]
    pub strict: bool,
}

impl ValidateArgs {
    /// Execute the validate command
    pub fn execute(&self) -> CliResult<()> {
        // Load layout
        let layout = LayoutService::load(&self.layout)
            .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

        let (geometry, mapping) = build_geometry_and_mapping_for_layout(&layout)?;

        // Load keycode database
        let keycode_db = KeycodeDb::load()
            .map_err(|e| CliError::io(format!("Failed to load keycode database: {e}")))?;

        // Validate
        let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
        let report = validator
            .validate()
            .map_err(|e| CliError::io(format!("Validation failed: {e}")))?;

        // Build response
        let mut checks = ValidationChecks::all_passed();
        let mut messages = Vec::new();

        // Convert errors
        for error in &report.errors {
            // Update check status based on error kind
            match error.kind {
                crate::firmware::validator::ValidationErrorKind::InvalidKeycode => {
                    checks.keycodes = "failed".to_string();
                }
                crate::firmware::validator::ValidationErrorKind::MissingPosition
                | crate::firmware::validator::ValidationErrorKind::DuplicatePosition
                | crate::firmware::validator::ValidationErrorKind::MatrixOutOfBounds
                | crate::firmware::validator::ValidationErrorKind::MismatchedKeyCount => {
                    checks.positions = "failed".to_string();
                }
                crate::firmware::validator::ValidationErrorKind::EmptyLayer => {
                    checks.layer_refs = "failed".to_string();
                }
            }

            let location =
                if let (Some(layer), Some(row), Some(col)) = (error.layer, error.row, error.col) {
                    Some(ValidationLocation {
                        layer,
                        position: ValidationPosition { row, col },
                    })
                } else {
                    None
                };

            messages.push(ValidationMessage {
                severity: "error".to_string(),
                message: error.message.clone(),
                location,
            });
        }

        // Convert warnings
        for warning in &report.warnings {
            let msg = warning.message.clone();

            // Update check status for tap dance warnings
            if msg.contains("Tap dance") || msg.contains("tap dance") {
                checks.tap_dances = "warning".to_string();
            }

            messages.push(ValidationMessage {
                severity: "warning".to_string(),
                message: msg,
                location: None,
            });
        }

        let response = ValidationResponse {
            valid: report.is_valid(),
            errors: messages,
            checks,
        };

        // Output results
        if self.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
            );
        } else {
            // Human-readable output
            if response.valid {
                println!("✓ Validation passed");
            } else {
                println!("✗ Validation failed");
            }

            println!("\nChecks:");
            println!("  Keycodes:   {}", response.checks.keycodes);
            println!("  Positions:  {}", response.checks.positions);
            println!("  Layer refs: {}", response.checks.layer_refs);
            println!("  Tap dances: {}", response.checks.tap_dances);

            if !response.errors.is_empty() {
                println!("\nIssues:");
                for msg in &response.errors {
                    let prefix = if msg.severity == "error" {
                        "  ✗"
                    } else {
                        "  ⚠"
                    };
                    if let Some(loc) = &msg.location {
                        println!(
                            "{} [Layer {} ({}, {})] {}",
                            prefix, loc.layer, loc.position.row, loc.position.col, msg.message
                        );
                    } else {
                        println!("{} {}", prefix, msg.message);
                    }
                }
            }
        }

        // Exit code
        if !response.valid {
            return Err(CliError::validation("Validation failed"));
        }

        if self.strict && !response.errors.is_empty() {
            let has_warnings = response.errors.iter().any(|m| m.severity == "warning");
            if has_warnings {
                return Err(CliError::validation("Warnings found in strict mode"));
            }
        }

        Ok(())
    }
}

fn build_geometry_and_mapping_for_layout(
    layout: &crate::models::Layout,
) -> CliResult<(KeyboardGeometry, VisualLayoutMapping)> {
    let config = Config::load().unwrap_or_default();
    let layout_name = layout
        .metadata
        .layout_variant
        .as_deref()
        .unwrap_or("LAYOUT");
    let context = geometry::GeometryContext {
        config: &config,
        metadata: &layout.metadata,
    };

    if let Ok(result) = geometry::build_geometry_for_layout(context, layout_name) {
        return Ok((result.geometry, result.mapping));
    }

    let geometry = build_minimal_geometry_for_layout(layout)?;
    let mapping = VisualLayoutMapping::build(&geometry);
    Ok((geometry, mapping))
}

/// Build minimal geometry for a layout based on its key count
fn build_minimal_geometry_for_layout(
    layout: &crate::models::Layout,
) -> CliResult<KeyboardGeometry> {
    use crate::models::keyboard_geometry::KeyGeometry;

    if layout.layers.is_empty() {
        return Err(CliError::validation("Layout has no layers"));
    }

    let key_count = layout.layers[0].keys.len();
    if key_count == 0 {
        return Err(CliError::validation("Layout has no keys"));
    }

    let rows = layout.layers[0]
        .keys
        .iter()
        .map(|key| key.position.row)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let cols = layout.layers[0]
        .keys
        .iter()
        .map(|key| key.position.col)
        .max()
        .unwrap_or(0)
        .saturating_add(1);

    let mut geometry = KeyboardGeometry::new(
        layout.metadata.keyboard.as_deref().unwrap_or("unknown"),
        layout
            .metadata
            .layout_variant
            .as_deref()
            .unwrap_or("LAYOUT"),
        rows,
        cols,
    );

    // Add keys in visual order
    for (idx, key_def) in layout.layers[0].keys.iter().enumerate() {
        let pos = key_def.position;
        #[allow(clippy::cast_possible_truncation)]
        let led_index = idx as u8;
        geometry.add_key(KeyGeometry::new(
            (pos.row, pos.col),
            led_index,
            f32::from(pos.col),
            f32::from(pos.row),
        ));
    }

    Ok(geometry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{KeyDefinition, Layer, Position, RgbColor};

    #[test]
    fn minimal_geometry_accepts_split_visual_coordinates() {
        let mut layout = crate::models::Layout::new("split").unwrap();
        let mut layer = Layer::new(0, "Globals", RgbColor::new(0, 0, 0)).unwrap();
        layer
            .keys
            .push(KeyDefinition::new(Position::new(1, 8), "KC_VOLU"));
        layer
            .keys
            .push(KeyDefinition::new(Position::new(2, 6), "KC_MUTE"));
        layout.add_layer(layer).unwrap();

        let geometry = build_minimal_geometry_for_layout(&layout).unwrap();
        let mapping = VisualLayoutMapping::build(&geometry);
        assert_eq!(mapping.visual_to_matrix_pos(1, 8), Some((1, 8)));
        assert_eq!(mapping.visual_to_matrix_pos(2, 6), Some((2, 6)));
    }
}
