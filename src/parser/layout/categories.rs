//! Categories phase: parse `- id: Name (#RRGGBB)` lines.

use crate::models::{Category, RgbColor};
use anyhow::Result;

/// Parses the categories section.
pub(super) fn parse_categories(
    lines: &[&str],
    start_line: usize,
    layout: &mut crate::models::Layout,
) -> Result<usize> {
    let mut line_num = start_line + 1; // Skip "## Categories" header

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Skip empty lines
        if line.is_empty() {
            line_num += 1;
            continue;
        }

        // Stop at next section
        if line.starts_with("##") {
            break;
        }

        // Parse category line: - id: Name (#RRGGBB)
        if let Some(captures) = super::category_regex().captures(line) {
            let id = captures[1].to_string();
            let name = captures[2].to_string();
            let color_hex = format!("#{}", &captures[3]);
            let color = RgbColor::from_hex(&color_hex)?;

            let category = Category::new(&id, &name, color)?;
            layout.add_category(category)?;
        }

        line_num += 1;
    }

    Ok(line_num)
}
