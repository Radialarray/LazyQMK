//! `qmk geometry` — display matrix/LED/visual coordinate mappings.

use crate::cli::common::{CliError, CliResult};
use crate::parser::keyboard_json::{
    build_keyboard_geometry_with_rgb, build_matrix_to_led_map, parse_keyboard_info_json,
    parse_variant_keyboard_json,
};
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Args)]
/// Display matrix, LED, and visual coordinate mappings for a keyboard layout
pub struct GeometryArgs {
    /// Path to QMK firmware repository
    #[arg(long, value_name = "PATH")]
    pub qmk_path: PathBuf,

    /// Keyboard name (e.g., "crkbd/rev1", "ferris/sweep")
    #[arg(long, value_name = "NAME")]
    pub keyboard: String,

    /// Layout variant name (e.g., "LAYOUT", "`LAYOUT_split_3x6_3`")
    #[arg(long, value_name = "NAME")]
    pub layout_name: String,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

/// JSON response for geometry command
#[derive(Debug, Clone, Serialize)]
struct GeometryResponse {
    /// Keyboard name
    keyboard: String,
    /// Layout name
    layout: String,
    /// Matrix dimensions
    matrix: MatrixInfo,
    /// Total key count
    key_count: usize,
    /// Coordinate mappings
    mappings: Vec<KeyMapping>,
}

/// Matrix dimension information
#[derive(Debug, Clone, Serialize)]
struct MatrixInfo {
    /// Number of rows
    rows: u8,
    /// Number of columns
    cols: u8,
}

/// Key coordinate mapping
#[derive(Debug, Clone, Serialize)]
struct KeyMapping {
    /// Visual position index
    visual_index: u8,
    /// Matrix position [row, col]
    matrix: [u8; 2],
    /// LED index
    led_index: u8,
    /// Visual coordinates (x, y)
    visual_position: [f32; 2],
}

impl GeometryArgs {
    /// Execute the geometry command
    pub fn execute(&self) -> CliResult<()> {
        // Check for test fixture override (for testing without full QMK submodule)
        let qmk_path = if let Ok(fixture_path) = std::env::var("LAZYQMK_QMK_FIXTURE") {
            PathBuf::from(fixture_path)
        } else {
            self.qmk_path.clone()
        };

        // Validate QMK path
        if !qmk_path.exists() {
            return Err(CliError::io(format!(
                "QMK path does not exist: {}",
                qmk_path.display()
            )));
        }

        // Parse keyboard info.json
        let info = parse_keyboard_info_json(&qmk_path, &self.keyboard)
            .map_err(|e| CliError::validation(format!("Invalid keyboard: {e}")))?;

        // Check if layout exists
        if !info.layouts.contains_key(&self.layout_name) {
            let available: Vec<String> = info.layouts.keys().cloned().collect();
            return Err(CliError::validation(format!(
                "Layout '{}' not found. Available layouts: {:?}",
                self.layout_name, available
            )));
        }

        // Try to load RGB matrix mapping for accurate LED indices
        let matrix_to_led = parse_variant_keyboard_json(&qmk_path, &self.keyboard)
            .and_then(|variant| variant.rgb_matrix.map(|rgb| build_matrix_to_led_map(&rgb)));

        // Build keyboard geometry
        let geometry = build_keyboard_geometry_with_rgb(
            &info,
            &self.keyboard,
            &self.layout_name,
            matrix_to_led.as_ref(),
        )
        .map_err(|e| CliError::validation(format!("Invalid layout: {e}")))?;

        // Build response
        let mappings: Vec<KeyMapping> = geometry
            .keys
            .iter()
            .map(|key| KeyMapping {
                visual_index: key.layout_index,
                matrix: [key.matrix_position.0, key.matrix_position.1],
                led_index: key.led_index,
                visual_position: [key.visual_x, key.visual_y],
            })
            .collect();

        let response = GeometryResponse {
            keyboard: self.keyboard.clone(),
            layout: self.layout_name.clone(),
            matrix: MatrixInfo {
                rows: geometry.matrix_rows,
                cols: geometry.matrix_cols,
            },
            key_count: geometry.keys.len(),
            mappings,
        };

        // Output results
        if self.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
            );
        } else {
            println!("Keyboard: {}", response.keyboard);
            println!("Layout: {}", response.layout);
            println!(
                "\nMatrix: {}x{}",
                response.matrix.rows, response.matrix.cols
            );
            println!("Keys: {}", response.key_count);
            println!("\nCoordinate Mappings:");
            println!("  Visual | Matrix    | LED | Position");
            println!("  -------|-----------|-----|----------");
            for mapping in &response.mappings {
                println!(
                    "  {:6} | ({:2}, {:2}) | {:3} | ({:5.1}, {:5.1})",
                    mapping.visual_index,
                    mapping.matrix[0],
                    mapping.matrix[1],
                    mapping.led_index,
                    mapping.visual_position[0],
                    mapping.visual_position[1]
                );
            }
        }

        Ok(())
    }
}
