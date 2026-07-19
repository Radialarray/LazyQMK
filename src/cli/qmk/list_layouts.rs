//! `qmk list-layouts` — list layout variants for a keyboard.

use crate::cli::common::{CliError, CliResult};
use crate::parser::keyboard_json::{
    discover_keyboard_config, extract_layout_variants, parse_keyboard_info_json,
};
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Args)]
/// List layout variants for a specific keyboard
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
    /// Layout name (e.g., "LAYOUT", "`LAYOUT_split_3x6_3`")
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
