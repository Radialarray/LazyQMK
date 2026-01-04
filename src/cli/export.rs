//! Export command for generating markdown documentation.

use crate::cli::common::{CliError, CliResult};
use crate::config::Config;
use crate::export;
use crate::keycode_db::KeycodeDb;
use crate::models::Layout;
use crate::services::geometry;
use crate::services::LayoutService;
use clap::Args;
use std::fs;
use std::path::PathBuf;

/// Export keyboard layout to markdown documentation
#[derive(Debug, Clone, Args)]
pub struct ExportArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Path to QMK firmware repository
    #[arg(long, value_name = "PATH")]
    pub qmk_path: PathBuf,

    /// Output path for markdown file (defaults to [layout_name]_export_[date].md)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// QMK layout variant (auto-detected from metadata if omitted)
    #[arg(long, value_name = "NAME")]
    pub layout_name: Option<String>,
}

impl ExportArgs {
    /// Execute the export command
    pub fn execute(&self) -> CliResult<()> {
        // Load layout
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

        // Load keycode database (needed for tap dance docs)
        let keycode_db = KeycodeDb::load()
            .map_err(|e| CliError::io(format!("Failed to load keycode database: {e}")))?;

        // Generate markdown content using export module
        let markdown = export::export_to_markdown(&layout, &geometry, &keycode_db)
            .map_err(|e| CliError::io(format!("Failed to generate markdown: {e}")))?;

        // Determine output path
        let output_path = self.get_output_path(&layout);

        // Write to file
        fs::write(&output_path, markdown)
            .map_err(|e| CliError::io(format!("Failed to write output file: {e}")))?;

        println!("✓ Exported layout to: {}", output_path.display());

        Ok(())
    }

    /// Get the output file path (either user-specified or auto-generated)
    fn get_output_path(&self, layout: &Layout) -> PathBuf {
        if let Some(ref path) = self.output {
            return path.clone();
        }

        // Auto-generate filename: [layout_name]_export_[date].md
        let date = chrono::Local::now().format("%Y-%m-%d");
        let layout_name = layout.metadata.name.replace(' ', "_").to_lowercase();

        PathBuf::from(format!("{}_export_{}.md", layout_name, date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_output_path_default() {
        let args = ExportArgs {
            layout: PathBuf::from("test.md"),
            qmk_path: PathBuf::from("/qmk"),
            output: None,
            layout_name: None,
        };

        let layout = Layout::new("My Test Layout").unwrap();
        let path = args.get_output_path(&layout);

        let path_str = path.to_string_lossy();
        assert!(path_str.contains("my_test_layout_export_"));
        assert!(path_str.ends_with(".md"));
    }

    #[test]
    fn test_get_output_path_custom() {
        let custom_path = PathBuf::from("/tmp/my_export.md");
        let args = ExportArgs {
            layout: PathBuf::from("test.md"),
            qmk_path: PathBuf::from("/qmk"),
            output: Some(custom_path.clone()),
            layout_name: None,
        };

        let layout = Layout::new("Test").unwrap();
        let path = args.get_output_path(&layout);

        assert_eq!(path, custom_path);
    }
}
