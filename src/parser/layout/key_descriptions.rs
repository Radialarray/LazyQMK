//! Key Descriptions phase: parse `- layer:row:col: description` lines.

use crate::models::Position;
use anyhow::Result;

/// Parses the key descriptions section.
///
/// Format: `- layer:row:col: description text`
/// Example: `- 0:1:3: Primary thumb key - hold for symbols, tap for space`
#[allow(clippy::unnecessary_wraps)]
pub(super) fn parse_key_descriptions(
    lines: &[&str],
    start_line: usize,
    layout: &mut crate::models::Layout,
) -> Result<usize> {
    let mut line_num = start_line + 1; // Skip "## Key Descriptions" header

    // Regex compiled once via super::desc_regex() helper (cached with OnceLock)
    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Skip empty lines
        if line.is_empty() {
            line_num += 1;
            continue;
        }

        // Stop at next section
        if line.starts_with("##") || line.starts_with("---") {
            break;
        }

        // Parse description line: - layer:row:col: description text
        if let Some(captures) = super::desc_regex().captures(line) {
            let layer_idx: usize = captures[1].parse().unwrap_or(0);
            let row: u8 = captures[2].parse().unwrap_or(0);
            let col: u8 = captures[3].parse().unwrap_or(0);
            let description = captures[4].trim().to_string();

            // Find the key and set its description
            if let Some(layer) = layout.layers.get_mut(layer_idx) {
                let pos = Position::new(row, col);
                if let Some(key) = layer.get_key_mut(pos) {
                    key.description = Some(description);
                }
            }
        }

        line_num += 1;
    }

    Ok(line_num)
}
