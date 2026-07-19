//! `qmk list-keyboards` — list compilable keyboards.

use crate::cli::common::{CliError, CliResult};
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
        let entries =
            fs::read_dir(dir).context(format!("Failed to read directory: {}", dir.display()))?;

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
