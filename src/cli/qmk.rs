//! QMK metadata commands for keyboard discovery and inspection.
//!
//! This module provides CLI commands to interact with QMK firmware metadata
//! without requiring the QMK CLI to be installed. Commands parse info.json
//! and keyboard.json files directly.

use crate::cli::common::{CliError, CliResult};
use crate::parser::keyboard_json::{
    discover_keyboard_config, extract_layout_variants, parse_keyboard_info_json,
    parse_variant_keyboard_json, build_keyboard_geometry_with_rgb, build_matrix_to_led_map,
};
use clap::Args;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// List all compilable keyboards in QMK firmware directory
#[derive(Debug, Clone, Args)]
pub struct ListKeyboardsArgs {
    /// Path to QMK firmware repository
    #[arg(long, value_name = "PATH")]
    pub qmk_path: PathBuf,

    /// Optional regex filter for keyboard names
    #[arg(long, value_name = "REGEX")]
    pub filter: Option<String>,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

/// JSON response for list-keyboards command
#[derive(Debug, Clone, Serialize)]
struct ListKeyboardsResponse {
    /// List of keyboard paths
    keyboards: Vec<String>,
    /// Total count
    count: usize,
}

impl ListKeyboardsArgs {
    /// Execute the list-keyboards command
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

        let keyboards_dir = qmk_path.join("keyboards");
        if !keyboards_dir.exists() {
            return Err(CliError::io(format!(
                "QMK keyboards directory not found: {}",
                keyboards_dir.display()
            )));
        }

        // Scan keyboards directory recursively
        let mut keyboards = scan_keyboards_directory(&keyboards_dir)
            .map_err(|e| CliError::io(format!("Failed to scan keyboards directory: {e}")))?;

        if keyboards.is_empty() {
            return Err(CliError::validation("No keyboards found in QMK directory"));
        }

        // Apply regex filter if provided
        if let Some(filter_str) = &self.filter {
            let regex = Regex::new(filter_str)
                .map_err(|e| CliError::validation(format!("Invalid regex pattern: {e}")))?;
            keyboards.retain(|kb| regex.is_match(kb));

            if keyboards.is_empty() {
                return Err(CliError::validation(format!(
                    "No keyboards match filter: {}",
                    filter_str
                )));
            }
        }

        // Sort alphabetically for consistent ordering
        keyboards.sort();

        // Output results
        if self.json {
            let response = ListKeyboardsResponse {
                keyboards: keyboards.clone(),
                count: keyboards.len(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
            );
        } else {
            println!("Found {} keyboards:", keyboards.len());
            for kb in &keyboards {
                println!("  {kb}");
            }
        }

        Ok(())
    }
}

/// Scans the keyboards directory recursively for keyboards with info.json or keyboard.json
fn scan_keyboards_directory(keyboards_dir: &PathBuf) -> anyhow::Result<Vec<String>> {
    use anyhow::Context;
    use std::collections::HashSet;

    let mut keyboards = HashSet::new();

    fn visit_directory(
        dir: &std::path::Path,
        keyboards_root: &std::path::Path,
        keyboards: &mut HashSet<String>,
    ) -> anyhow::Result<()> {
        let entries = fs::read_dir(dir).context(format!("Failed to read directory: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and common non-keyboard directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "lib" || name == "template" {
                        continue;
                    }
                }
                // Recursively visit subdirectories
                visit_directory(&path, keyboards_root, keyboards)?;
            } else if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str());
                if file_name == Some("info.json") || file_name == Some("keyboard.json") {
                    // Found a keyboard config file - compute relative path
                    if let Ok(rel_path) = path.parent().unwrap().strip_prefix(keyboards_root) {
                        let keyboard_name = rel_path.to_str().unwrap_or("").replace('\\', "/");
                        if !keyboard_name.is_empty() {
                            keyboards.insert(keyboard_name);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    visit_directory(keyboards_dir, keyboards_dir, &mut keyboards)?;

    Ok(keyboards.into_iter().collect())
}

/// List layout variants for a specific keyboard
#[derive(Debug, Clone, Args)]
pub struct ListLayoutsArgs {
    /// Path to QMK firmware repository
    #[arg(long, value_name = "PATH")]
    pub qmk_path: PathBuf,

    /// Keyboard name (e.g., "crkbd/rev1", "ferris/sweep")
    #[arg(long, value_name = "NAME")]
    pub keyboard: String,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

/// JSON response for list-layouts command
#[derive(Debug, Clone, Serialize)]
struct ListLayoutsResponse {
    /// Keyboard name
    keyboard: String,
    /// List of layout names
    layouts: Vec<LayoutInfo>,
    /// Total count
    count: usize,
}

/// Layout information including name and key count
#[derive(Debug, Clone, Serialize)]
struct LayoutInfo {
    /// Layout name (e.g., "LAYOUT", "LAYOUT_split_3x6_3")
    name: String,
    /// Number of keys in this layout
    key_count: usize,
}

impl ListLayoutsArgs {
    /// Execute the list-layouts command
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

        // Discover keyboard config files
        let config = discover_keyboard_config(&qmk_path, &self.keyboard)
            .map_err(|e| CliError::validation(format!("Keyboard not found: {e}")))?;

        if !config.has_layouts {
            return Err(CliError::validation(format!(
                "Keyboard '{}' has no layouts defined",
                self.keyboard
            )));
        }

        // Parse keyboard info.json
        let info = parse_keyboard_info_json(&qmk_path, &self.keyboard)
            .map_err(|e| CliError::io(format!("Failed to parse keyboard info: {e}")))?;

        // Extract layout variants with key counts
        let variants = extract_layout_variants(&info);

        if variants.is_empty() {
            return Err(CliError::validation(format!(
                "No layouts found for keyboard '{}'",
                self.keyboard
            )));
        }

        // Output results
        if self.json {
            let layouts: Vec<LayoutInfo> = variants
                .iter()
                .map(|v| LayoutInfo {
                    name: v.name.clone(),
                    key_count: v.key_count,
                })
                .collect();

            let response = ListLayoutsResponse {
                keyboard: self.keyboard.clone(),
                layouts,
                count: variants.len(),
            };

            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
            );
        } else {
            println!("Keyboard: {}", self.keyboard);
            println!("\nAvailable layouts ({}):", variants.len());
            for variant in &variants {
                println!("  {} ({} keys)", variant.name, variant.key_count);
            }
        }

        Ok(())
    }
}

/// Display matrix, LED, and visual coordinate mappings for a keyboard layout
#[derive(Debug, Clone, Args)]
pub struct GeometryArgs {
    /// Path to QMK firmware repository
    #[arg(long, value_name = "PATH")]
    pub qmk_path: PathBuf,

    /// Keyboard name (e.g., "crkbd/rev1", "ferris/sweep")
    #[arg(long, value_name = "NAME")]
    pub keyboard: String,

    /// Layout variant name (e.g., "LAYOUT", "LAYOUT_split_3x6_3")
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
            println!("\nMatrix: {}x{}", response.matrix.rows, response.matrix.cols);
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
