//! Combos settings: enabled flag and individual combo definitions.

use crate::models::{ComboDefinition, Layout, Position};

/// Parses the `**Combos**` / `**Combos Enabled**` master switch.
pub(super) fn try_parse_combos_enabled(line: &str, layout: &mut Layout) -> bool {
    if !(line.starts_with("**Combos**:") || line.starts_with("**Combos Enabled**:")) {
        return false;
    }
    let value = line
        .strip_prefix("**Combos**:")
        .or_else(|| line.strip_prefix("**Combos Enabled**:"))
        .unwrap()
        .trim()
        .to_lowercase();
    layout.combo_settings.enabled = matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses an individual combo definition line.
///
/// Format: `**Combo N**: (row1,col1)+(row2,col2) → Action [duration]`
/// Example: `**Combo 1**: (0,0)+(0,1) → Disable Effects [500ms]`
///
/// Returns `true` if the line matched the combo regex (regardless of whether
/// the action name resolved to a known `ComboAction`). This matches the
/// original parser's behavior — any line matching the regex was consumed.
pub(super) fn try_parse_combo_definition(line: &str, layout: &mut Layout) -> bool {
    let Some(captures) = super::super::combo_regex().captures(line) else {
        return false;
    };

    let combo_num: usize = captures[1].parse().unwrap_or(0);
    let row1: u8 = captures[2].parse().unwrap_or(0);
    let col1: u8 = captures[3].parse().unwrap_or(0);
    let row2: u8 = captures[4].parse().unwrap_or(0);
    let col2: u8 = captures[5].parse().unwrap_or(0);
    let action_str = captures[6].trim();
    let duration_ms: u16 = captures
        .get(7)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(500);

    // Parse action
    if let Some(action) = crate::models::ComboAction::from_name(action_str) {
        let combo = ComboDefinition::with_duration(
            Position::new(row1, col1),
            Position::new(row2, col2),
            action,
            duration_ms,
        );

        // Place the combo at its 0-based index (combo_num 1→idx 0, etc.).
        // Grow the vec only as far as needed — never push placeholder combos
        // for gaps, which was the source of phantom COMBO_0 entries when
        // only Combo 2 and Combo 3 were defined.
        if combo_num > 0 && combo_num <= 3 {
            let idx = combo_num - 1;
            if layout.combo_settings.combos.len() <= idx {
                layout
                    .combo_settings
                    .combos
                    .resize_with(idx + 1, ComboDefinition::new_placeholder);
            }
            layout.combo_settings.combos[idx] = combo;
        }
    }
    true
}
