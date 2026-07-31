//! Generate command for firmware files.

use crate::cli::common::{CliError, CliResult};
use crate::config::Config;
use crate::firmware::generator::FirmwareGenerator;
use crate::keycode_db::KeycodeDb;
use crate::services::geometry;
use crate::services::layout_versions::LayoutVersionService;
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

        // Auto-snapshot before compile (only if the layout is in folder layout).
        // `LayoutService::load` may have just migrated a legacy flat file; in
        // that case we don't snapshot — the user can do it manually next time.
        if let Some(layout_name) = self.layout.file_stem().and_then(|s| s.to_str()) {
                if let Some(layouts_dir) = std::path::Path::new(&self.layout)
                    .parent()
                    .map(std::path::Path::to_path_buf)
                    .or_else(|| Config::config_dir().ok().map(|c| c.join("layouts")))
            {
                let svc = LayoutVersionService::new(layouts_dir);
                let layout_dir = svc.layout_dir(layout_name);
                if layout_dir.join("current.json").exists() {
                    // Only snapshot when we have a real folder layout to write into.
                    if let Err(e) = svc.create_auto_snapshot(layout_name, &layout, None) {
                        return Err(CliError::io(format!(
                            "Failed to create pre-compile snapshot: {e}"
                        )));
                    }
                }
            }
        }

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
                let rules_mk = generator.generate_rules_mk();
                let keymap_json = generator.generate_keymap_json();

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

                // Write keymap.json (for QMK community modules)
                if !keymap_json.is_empty() {
                    std::fs::write(self.out_dir.join("keymap.json"), &keymap_json)
                        .map_err(|e| CliError::io(format!("Failed to write keymap.json: {e}")))?;
                } else {
                    let json_path = self.out_dir.join("keymap.json");
                    if json_path.exists() {
                        std::fs::remove_file(&json_path).map_err(|e| {
                            CliError::io(format!("Failed to remove stale keymap.json: {e}"))
                        })?;
                    }
                }

                // rules.mk is always emitted (placeholder body when no features
                // need enabling) so the deploy step overwrites any stale
                // rules.mk left in the destination by a previous feature-enabled
                // generation. See LazyQMK-msc8.
                std::fs::write(self.out_dir.join("rules.mk"), &rules_mk)
                    .map_err(|e| CliError::io(format!("Failed to write rules.mk: {e}")))?;

                if !keymap_json.is_empty() {
                    println!("✓ Generated keymap.c, config.h, rules.mk, and keymap.json");
                } else {
                    println!("✓ Generated keymap.c, config.h, and rules.mk");
                }
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
