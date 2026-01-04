//! Layer references command for displaying inbound layer references and transparency warnings.

use crate::cli::common::{CliError, CliResult};
use crate::parser::layout::parse_markdown_layout;
use crate::services::layer_refs::{build_layer_ref_index, is_transparent};
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;

/// Show layer references and transparency warnings
#[derive(Debug, Clone, Args)]
pub struct LayerRefsArgs {
    /// Path to layout markdown file
    #[arg(short, long, value_name = "FILE")]
    pub layout: PathBuf,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// JSON response for layer references
#[derive(Debug, Serialize)]
struct LayerRefsResponse {
    layers: Vec<LayerRefData>,
}

/// Layer reference data for JSON output
#[derive(Debug, Serialize)]
struct LayerRefData {
    number: usize,
    name: String,
    inbound_refs: Vec<InboundRefData>,
    warnings: Vec<WarningData>,
}

/// Individual inbound reference for JSON output
#[derive(Debug, Serialize)]
struct InboundRefData {
    from_layer: usize,
    position: PositionData,
    kind: String,
    keycode: String,
}

/// Position data for JSON output
#[derive(Debug, Serialize)]
struct PositionData {
    row: u8,
    col: u8,
}

/// Warning data for JSON output
#[derive(Debug, Serialize)]
struct WarningData {
    position: PositionData,
    message: String,
}

impl LayerRefsArgs {
    /// Execute the layer-refs command
    pub fn execute(&self) -> CliResult<()> {
        // Parse layout file
        let layout = parse_markdown_layout(&self.layout)
            .map_err(|e| CliError::io(format!("Failed to load layout: {e}")))?;

        // Build layer reference index
        let layer_ref_index = build_layer_ref_index(&layout.layers);

        if self.json {
            // JSON output
            let mut layers_data = Vec::new();

            for (layer_idx, layer) in layout.layers.iter().enumerate() {
                // Get inbound references for this layer
                let refs = layer_ref_index.get(&layer_idx).cloned().unwrap_or_default();

                // Collect inbound references
                let inbound_refs: Vec<InboundRefData> = refs
                    .iter()
                    .map(|r| InboundRefData {
                        from_layer: r.from_layer,
                        position: PositionData {
                            row: r.position.row,
                            col: r.position.col,
                        },
                        kind: r.kind.display_name().to_string(),
                        keycode: r.keycode.clone(),
                    })
                    .collect();

                // Check for transparency conflicts
                let mut warnings = Vec::new();
                for r in &refs {
                    // Only check hold-like references
                    if !r.kind.is_hold_like() {
                        continue;
                    }

                    // Find the key at this position in the target layer
                    if let Some(target_key) = layer.get_key(r.position) {
                        // Check if the key is non-transparent
                        if !is_transparent(&target_key.keycode) {
                            let message = format!(
                                "Non-transparent key ({}) conflicts with hold-like reference from Layer {} {}",
                                target_key.keycode,
                                r.from_layer,
                                r.kind.display_name()
                            );
                            warnings.push(WarningData {
                                position: PositionData {
                                    row: r.position.row,
                                    col: r.position.col,
                                },
                                message,
                            });
                        }
                    }
                }

                layers_data.push(LayerRefData {
                    number: layer_idx,
                    name: layer.name.clone(),
                    inbound_refs,
                    warnings,
                });
            }

            let response = LayerRefsResponse {
                layers: layers_data,
            };

            println!(
                "{}",
                serde_json::to_string_pretty(&response)
                    .map_err(|e| CliError::io(format!("Failed to serialize JSON: {e}")))?
            );
        } else {
            // Human-readable text output
            for (layer_idx, layer) in layout.layers.iter().enumerate() {
                println!("Layer {}: {}", layer_idx, layer.name);

                // Get inbound references
                if let Some(refs) = layer_ref_index.get(&layer_idx) {
                    if refs.is_empty() {
                        println!("  No inbound references");
                    } else {
                        println!("  Inbound References:");
                        for r in refs {
                            println!(
                                "    - Layer {} [{},{}] {} ({}): {}",
                                r.from_layer,
                                r.position.row,
                                r.position.col,
                                r.kind.display_name(),
                                r.keycode,
                                r.keycode
                            );
                        }
                    }
                } else {
                    println!("  No inbound references");
                }

                // Check for warnings
                let mut has_warnings = false;
                for r in layer_ref_index.get(&layer_idx).cloned().unwrap_or_default() {
                    // Only check hold-like references
                    if !r.kind.is_hold_like() {
                        continue;
                    }

                    // Find the key at this position in the target layer
                    if let Some(target_key) = layer.get_key(r.position) {
                        // Check if the key is non-transparent
                        if !is_transparent(&target_key.keycode) {
                            if !has_warnings {
                                println!("  Warnings:");
                                has_warnings = true;
                            }
                            println!(
                                "    - Position [{},{}]: Non-transparent key ({}) conflicts with hold-like reference from Layer {} {}",
                                r.position.row,
                                r.position.col,
                                target_key.keycode,
                                r.from_layer,
                                r.kind.display_name()
                            );
                        }
                    }
                }

                println!();
            }
        }

        Ok(())
    }
}
