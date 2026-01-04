//! Keycode resolution command.

use crate::cli::common::{CliError, CliResult};
use crate::keycode_db::KeycodeDb;
use crate::services::LayoutService;
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;

/// Resolve parameterized keycodes with layer UUIDs
#[derive(Debug, Clone, Args)]
pub struct KeycodeArgs {
    /// Path to layout markdown file (for layer UUID context)
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Keycode expression to resolve (e.g., "LT(@uuid, KC_SPC)")
    #[arg(short, long, value_name = "EXPR")]
    pub expr: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct ResolveResult {
    input: String,
    resolved: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    layer_name: Option<String>,
    valid: bool,
}

impl KeycodeArgs {
    /// Execute the keycode resolve command
    pub fn execute(&self) -> CliResult<()> {
        // Load layout for layer context
        let layout = LayoutService::load(&self.layout)
            .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

        // Load keycode database
        let keycode_db = KeycodeDb::load()
            .map_err(|e| CliError::io(format!("Failed to load keycode database: {e}")))?;

        // Try to resolve the keycode
        let resolved = layout
            .resolve_layer_keycode(&self.expr, &keycode_db)
            .unwrap_or_else(|| self.expr.clone());

        // Check if it was actually resolved (different from input)
        let was_resolved = resolved != self.expr;

        // Try to extract layer name if it's a layer keycode
        let layer_name = if was_resolved {
            // Extract layer index from resolved keycode
            extract_layer_index(&resolved)
                .and_then(|idx| layout.layers.get(idx).map(|layer| layer.name.clone()))
        } else {
            None
        };

        let result = ResolveResult {
            input: self.expr.clone(),
            resolved,
            layer_name,
            valid: was_resolved || !is_layer_keycode(&self.expr),
        };

        if self.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&result)
                    .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
            );
        } else {
            println!("Input:    {}", result.input);
            println!("Resolved: {}", result.resolved);
            if let Some(name) = result.layer_name {
                println!("Layer:    {}", name);
            }
            if result.valid {
                println!("Status:   ✓ Valid");
            } else {
                println!("Status:   ✗ Could not resolve");
            }
        }

        // Exit with error if resolution failed
        if !result.valid {
            return Err(CliError::validation("Failed to resolve layer keycode"));
        }

        Ok(())
    }
}

/// Extract layer index from a resolved layer keycode (e.g., "MO(1)" -> Some(1))
fn extract_layer_index(keycode: &str) -> Option<usize> {
    // Handle simple layer keycodes: MO(n), TG(n), TO(n), TT(n), OSL(n), DF(n)
    for prefix in &["MO(", "TG(", "TO(", "TT(", "OSL(", "DF("] {
        if let Some(inner) = keycode.strip_prefix(prefix) {
            if let Some(num_str) = inner.strip_suffix(')') {
                return num_str.parse().ok();
            }
        }
    }

    // Handle compound layer keycodes: LT(n, key), LM(n, mod)
    for prefix in &["LT(", "LM("] {
        if let Some(inner) = keycode.strip_prefix(prefix) {
            if let Some(inner) = inner.strip_suffix(')') {
                if let Some((num_str, _)) = inner.split_once(',') {
                    return num_str.trim().parse().ok();
                }
            }
        }
    }

    None
}

/// Check if a keycode is a layer-switching keycode
fn is_layer_keycode(keycode: &str) -> bool {
    keycode.starts_with("MO(")
        || keycode.starts_with("TG(")
        || keycode.starts_with("TO(")
        || keycode.starts_with("TT(")
        || keycode.starts_with("OSL(")
        || keycode.starts_with("DF(")
        || keycode.starts_with("LT(")
        || keycode.starts_with("LM(")
}
