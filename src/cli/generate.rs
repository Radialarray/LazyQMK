//! Generate command for firmware files.

use crate::cli::common::{CliError, CliResult};
use crate::config::Config;
use crate::firmware::generator::FirmwareGenerator;
use crate::keycode_db::KeycodeDb;
use crate::services::geometry;
use crate::services::LayoutService;
use clap::Args;
use std::path::PathBuf;

/// Generate QMK firmware files from a layout
#[derive(Debug, Clone, Args)]
pub struct GenerateArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Path to QMK firmware repository
    #[arg(long, value_name = "PATH")]
    pub qmk_path: PathBuf,

    /// Output directory for generated files
    #[arg(short, long, value_name = "DIR")]
    pub out_dir: PathBuf,

    /// QMK layout variant (auto-detected from metadata if omitted)
    #[arg(long, value_name = "NAME")]
    pub layout_name: Option<String>,

    /// Output format: keymap, config, or all
    #[arg(long, value_name = "TYPE", default_value = "all")]
    pub format: String,

    /// Use stable timestamps/UUIDs for deterministic output (for testing)
    #[arg(long)]
    pub deterministic: bool,
}

impl GenerateArgs {
    /// Execute the generate command
    pub fn execute(&self) -> CliResult<()> {
        // Validate format
        if !matches!(self.format.as_str(), "keymap" | "config" | "all") {
            return Err(CliError::validation(format!(
                "Invalid format '{}'. Must be 'keymap', 'config', or 'all'",
                self.format
            )));
        }

        // Load layout
        let layout = LayoutService::load(&self.layout)
            .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

        // Build config with QMK path
        let mut config = Config::load().unwrap_or_default();
        config.paths.qmk_firmware = Some(self.qmk_path.clone());
        config.build.output_dir.clone_from(&self.out_dir);

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
        let mapping = geo_result.mapping;

        // Load keycode database
        let keycode_db = KeycodeDb::load()
            .map_err(|e| CliError::io(format!("Failed to load keycode database: {e}")))?;

        // Validate before generating
        let validator = crate::firmware::validator::FirmwareValidator::new(
            &layout,
            &geometry,
            &mapping,
            &keycode_db,
        );
        let report = validator
            .validate()
            .map_err(|e| CliError::io(format!("Validation failed: {e}")))?;

        if !report.is_valid() {
            return Err(CliError::validation(format!(
                "Layout validation failed:\n{}",
                report.format_message()
            )));
        }

        // Create output directory
        std::fs::create_dir_all(&self.out_dir)
            .map_err(|e| CliError::io(format!("Failed to create output directory: {e}")))?;

        // Generate files
        let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);

        match self.format.as_str() {
            "all" => {
                // Generate both files
                let keymap_c = generator
                    .generate_keymap_c()
                    .map_err(|e| CliError::io(format!("Failed to generate keymap.c: {e}")))?;
                let config_h = generator
                    .generate_merged_config_h()
                    .map_err(|e| CliError::io(format!("Failed to generate config.h: {e}")))?;

                // Apply deterministic transformations if requested
                let keymap_c = if self.deterministic {
                    normalize_for_deterministic(&keymap_c)
                } else {
                    keymap_c
                };
                let config_h = if self.deterministic {
                    normalize_for_deterministic(&config_h)
                } else {
                    config_h
                };

                // Write files
                std::fs::write(self.out_dir.join("keymap.c"), keymap_c)
                    .map_err(|e| CliError::io(format!("Failed to write keymap.c: {e}")))?;
                std::fs::write(self.out_dir.join("config.h"), config_h)
                    .map_err(|e| CliError::io(format!("Failed to write config.h: {e}")))?;

                println!("✓ Generated keymap.c and config.h");
                println!("  Output: {}", self.out_dir.display());
            }
            "keymap" => {
                let keymap_c = generator
                    .generate_keymap_c()
                    .map_err(|e| CliError::io(format!("Failed to generate keymap.c: {e}")))?;

                let keymap_c = if self.deterministic {
                    normalize_for_deterministic(&keymap_c)
                } else {
                    keymap_c
                };

                std::fs::write(self.out_dir.join("keymap.c"), keymap_c)
                    .map_err(|e| CliError::io(format!("Failed to write keymap.c: {e}")))?;

                println!("✓ Generated keymap.c");
                println!("  Output: {}", self.out_dir.display());
            }
            "config" => {
                let config_h = generator
                    .generate_merged_config_h()
                    .map_err(|e| CliError::io(format!("Failed to generate config.h: {e}")))?;

                let config_h = if self.deterministic {
                    normalize_for_deterministic(&config_h)
                } else {
                    config_h
                };

                std::fs::write(self.out_dir.join("config.h"), config_h)
                    .map_err(|e| CliError::io(format!("Failed to write config.h: {e}")))?;

                println!("✓ Generated config.h");
                println!("  Output: {}", self.out_dir.display());
            }
            _ => unreachable!("Format already validated"),
        }

        Ok(())
    }
}

/// Normalize generated code for deterministic output (remove timestamps)
fn normalize_for_deterministic(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            if line.contains("Generated:") || line.contains("Generated at:") {
                "// Generated: <timestamp>"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
