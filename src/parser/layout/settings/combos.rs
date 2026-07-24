//! Combos settings: enabled flag and individual combo definitions.

use std::collections::BTreeMap;

use crate::models::layout::combo::MAX_COMBOS;
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
/// The 1-indexed slot number (`N`) keys the entry into `buffer`. Gaps are
/// permitted at parse time (older files written when only 3 slots were
/// supported may have non-contiguous numbering) — they are silently dropped
/// when [`combo_buffer_into_vec`] converts the map into a `Vec`.
///
/// Returns `true` if the line matched the combo regex (regardless of whether
/// the action name resolved to a known `ComboAction`). This matches the
/// original parser's behavior — any line matching the regex was consumed.
pub(super) fn try_parse_combo_definition(
    line: &str,
    buffer: &mut BTreeMap<usize, ComboDefinition>,
) -> bool {
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

    if combo_num == 0 || combo_num > MAX_COMBOS {
        return true;
    }

    if let Some(action) = crate::models::ComboAction::from_name(action_str) {
        let def = ComboDefinition::with_duration(
            Position::new(row1, col1),
            Position::new(row2, col2),
            action,
            duration_ms,
        );
        if def.validate().is_ok() {
            buffer.insert(combo_num, def);
        }
    }
    true
}

/// Drains a combo parse buffer into a slot-ordered `Vec`, skipping gaps.
pub(super) fn combo_buffer_into_vec(
    buffer: BTreeMap<usize, ComboDefinition>,
) -> Vec<ComboDefinition> {
    buffer.into_values().collect()
}
