//! Keycodes utility command for listing available keycodes.

use crate::cli::common::{CliError, CliResult};
use crate::keycode_db::KeycodeDb;
use clap::Args;
use serde::Serialize;

/// List available keycodes from the embedded keycode database
#[derive(Debug, Clone, Args)]
pub struct KeycodesArgs {
    /// Filter by category name (e.g., "basic", "navigation", "modifiers")
    #[arg(long, value_name = "NAME")]
    pub category: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct KeycodeOutput {
    /// QMK keycode (e.g., "`KC_A`")
    code: String,
    /// Display name (e.g., "A")
    label: String,
    /// Category name
    category: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct KeycodesJsonResponse {
    /// List of keycodes
    keycodes: Vec<KeycodeOutput>,
    /// Total count of keycodes in response
    count: usize,
}

impl KeycodesArgs {
    /// Execute the keycodes command
    pub fn execute(&self) -> CliResult<()> {
        // Load database
        let db = KeycodeDb::load()
            .map_err(|e| CliError::io(format!("Failed to load keycode database: {e}")))?;

        // Get keycodes (filtered or all)
        let keycodes = if let Some(cat) = &self.category {
            // Validate category exists
            if db.get_category(cat).is_none() {
                return Err(CliError::validation(format!("Unknown category: {}", cat)));
            }
            db.get_category_keycodes(cat)
        } else {
            // Get all keycodes using search with empty query
            db.search("")
        };

        if self.json {
            self.output_json(&keycodes)?;
        } else {
            self.output_table(&keycodes);
        }

        Ok(())
    }

    /// Output keycodes as JSON
    fn output_json(&self, keycodes: &[&crate::keycode_db::KeycodeDefinition]) -> CliResult<()> {
        let output_keycodes = keycodes
            .iter()
            .map(|kc| KeycodeOutput {
                code: kc.code.clone(),
                label: kc.name.clone(),
                category: kc.category.clone(),
                description: kc.description.clone(),
            })
            .collect::<Vec<_>>();

        let response = KeycodesJsonResponse {
            count: output_keycodes.len(),
            keycodes: output_keycodes,
        };

        println!(
            "{}",
            serde_json::to_string_pretty(&response)
                .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
        );

        Ok(())
    }

    /// Output keycodes as a formatted table
    fn output_table(&self, keycodes: &[&crate::keycode_db::KeycodeDefinition]) {
        if keycodes.is_empty() {
            println!("No keycodes found.");
            return;
        }

        // Calculate column widths
        let max_code_len = keycodes
            .iter()
            .map(|kc| kc.code.len())
            .max()
            .unwrap_or(0)
            .max(4); // At least "CODE"

        let max_label_len = keycodes
            .iter()
            .map(|kc| kc.name.len())
            .max()
            .unwrap_or(0)
            .max(5); // At least "LABEL"

        let max_category_len = keycodes
            .iter()
            .map(|kc| kc.category.len())
            .max()
            .unwrap_or(0)
            .max(8); // At least "CATEGORY"

        // Print header
        println!(
            "{:<width_code$}  {:<width_label$}  {:<width_category$}  DESCRIPTION",
            "CODE",
            "LABEL",
            "CATEGORY",
            width_code = max_code_len,
            width_label = max_label_len,
            width_category = max_category_len
        );

        // Print separator
        println!(
            "{}  {}  {}  {}",
            "─".repeat(max_code_len),
            "─".repeat(max_label_len),
            "─".repeat(max_category_len),
            "─".repeat(20)
        );

        // Print keycodes
        for keycode in keycodes {
            let description = keycode
                .description
                .as_ref()
                .map(|d| d.chars().take(50).collect::<String>())
                .unwrap_or_default();

            println!(
                "{:<width_code$}  {:<width_label$}  {:<width_category$}  {}",
                keycode.code,
                keycode.name,
                keycode.category,
                description,
                width_code = max_code_len,
                width_label = max_label_len,
                width_category = max_category_len
            );
        }

        println!();
        println!("Total: {} keycodes", keycodes.len());
    }
}
