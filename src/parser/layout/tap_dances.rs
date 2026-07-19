//! Tap Dances phase: parse `- **name**:` blocks with `Single Tap`, `Double Tap`, `Hold`.

use crate::models::TapDanceAction;
use anyhow::Result;

/// Parses the tap dances section.
///
/// Format:
/// ```markdown
/// ## Tap Dances
///
/// - **name**:
///   - Single Tap: KC_ESC
///   - Double Tap: KC_CAPS
///   - Hold: KC_LCTL
/// ```
#[allow(clippy::unnecessary_wraps)]
pub(super) fn parse_tap_dances(
    lines: &[&str],
    start_line: usize,
    layout: &mut crate::models::Layout,
) -> Result<usize> {
    let mut line_num = start_line + 1; // Skip "## Tap Dances" header

    // State for parsing current tap dance
    let mut current_name: Option<String> = None;
    let mut current_single: Option<String> = None;
    let mut current_double: Option<String> = None;
    let mut current_hold: Option<String> = None;

    // Helper to finish current tap dance
    fn finish_current_td(
        name: Option<String>,
        single: Option<String>,
        double: Option<String>,
        hold: Option<String>,
        layout: &mut crate::models::Layout,
    ) -> Result<()> {
        if let (Some(name), Some(single)) = (name, single) {
            let mut td = TapDanceAction::new(name, single);
            if let Some(double) = double {
                td = td.with_double_tap(double);
            }
            if let Some(hold) = hold {
                td = td.with_hold(hold);
            }
            // Validate before adding
            td.validate()?;
            layout.tap_dances.push(td);
        }
        Ok(())
    }

    while line_num < lines.len() {
        let line = lines[line_num];
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            line_num += 1;
            continue;
        }

        // Stop at next section
        if trimmed.starts_with("##") || trimmed.starts_with("---") {
            // Finish current tap dance if any
            finish_current_td(
                current_name.take(),
                current_single.take(),
                current_double.take(),
                current_hold.take(),
                layout,
            )?;
            break;
        }

        // Parse tap dance name: - **name**: (check trimmed version)
        if trimmed.starts_with("- **") && trimmed.ends_with("**:") {
            // Finish previous tap dance if any
            finish_current_td(
                current_name.take(),
                current_single.take(),
                current_double.take(),
                current_hold.take(),
                layout,
            )?;

            // Extract name from: - **name**:
            let name = trimmed
                .strip_prefix("- **")
                .and_then(|s| s.strip_suffix("**:"))
                .map(str::to_string);
            current_name = name;
            line_num += 1;
            continue;
        }

        // Parse properties: "  - Single Tap: KC_ESC" (use original line to preserve indent)
        if line.starts_with("  - ") {
            if let Some(keycode_part) = line.strip_prefix("  - Single Tap: ") {
                current_single = Some(keycode_part.trim().to_string());
            } else if let Some(keycode_part) = line.strip_prefix("  - Double Tap: ") {
                current_double = Some(keycode_part.trim().to_string());
            } else if let Some(keycode_part) = line.strip_prefix("  - Hold: ") {
                current_hold = Some(keycode_part.trim().to_string());
            }
        }

        line_num += 1;
    }

    // Finish last tap dance if any
    finish_current_td(
        current_name,
        current_single,
        current_double,
        current_hold,
        layout,
    )?;

    Ok(line_num)
}
