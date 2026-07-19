//! Tap-Hold settings: preset, tapping term, quick tap term, hold mode,
//! retro tapping, tapping toggle, flow tap term, chordal hold.

use crate::models::{HoldDecisionMode, Layout, TapHoldPreset};

/// Parses Tap-Hold Preset (also returns the explicit preset so the dispatcher
/// can re-apply it after individual settings may have reset it).
pub(super) fn try_parse_tap_hold_preset(line: &str, layout: &mut Layout) -> Option<TapHoldPreset> {
    if !line.starts_with("**Tap-Hold Preset**:") {
        return None;
    }

    let value = line
        .strip_prefix("**Tap-Hold Preset**:")
        .unwrap()
        .trim()
        .to_lowercase();

    let preset = match value.as_str() {
        "home row mods" | "homerowmods" => TapHoldPreset::HomeRowMods,
        "responsive" => TapHoldPreset::Responsive,
        "deliberate" => TapHoldPreset::Deliberate,
        "custom" => TapHoldPreset::Custom,
        _ => TapHoldPreset::Default,
    };
    layout.tap_hold_settings.preset = preset;
    Some(preset)
}

/// Parses Tapping Term (ms).
pub(super) fn try_parse_tapping_term(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Tapping Term**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Tapping Term**:")
        .unwrap()
        .trim()
        .trim_end_matches("ms")
        .trim();
    if let Ok(term) = value.parse::<u16>() {
        layout.tap_hold_settings.tapping_term = term;
    }
    true
}

/// Parses Quick Tap Term (ms, "auto", "same as tapping term", or "none").
pub(super) fn try_parse_quick_tap_term(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Quick Tap Term**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Quick Tap Term**:")
        .unwrap()
        .trim()
        .to_lowercase();

    if value == "auto" || value == "same as tapping term" || value == "none" {
        layout.tap_hold_settings.quick_tap_term = None;
    } else {
        let term_str = value.trim_end_matches("ms").trim();
        if let Ok(term) = term_str.parse::<u16>() {
            layout.tap_hold_settings.quick_tap_term = Some(term);
        }
    }
    true
}

/// Parses Hold Mode (permissive, hold on other key, etc).
pub(super) fn try_parse_hold_mode(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Hold Mode**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Hold Mode**:")
        .unwrap()
        .trim()
        .to_lowercase();

    layout.tap_hold_settings.hold_mode = match value.as_str() {
        "permissive" | "permissive hold" => HoldDecisionMode::PermissiveHold,
        "hold on other key" | "hold on other key press" | "aggressive" => {
            HoldDecisionMode::HoldOnOtherKeyPress
        }
        _ => HoldDecisionMode::Default,
    };
    true
}

/// Parses Retro Tapping (on/off).
pub(super) fn try_parse_retro_tapping(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Retro Tapping**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Retro Tapping**:")
        .unwrap()
        .trim()
        .to_lowercase();

    layout.tap_hold_settings.retro_tapping =
        value == "on" || value == "true" || value == "yes" || value == "enabled";
    true
}

/// Parses Tapping Toggle (number of taps).
pub(super) fn try_parse_tapping_toggle(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Tapping Toggle**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Tapping Toggle**:")
        .unwrap()
        .trim()
        .trim_end_matches(" taps")
        .trim();
    if let Ok(count) = value.parse::<u8>() {
        layout.tap_hold_settings.tapping_toggle = count;
    }
    true
}

/// Parses Flow Tap Term (ms, or "disabled"/"off"/"none").
pub(super) fn try_parse_flow_tap_term(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Flow Tap Term**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Flow Tap Term**:")
        .unwrap()
        .trim()
        .to_lowercase();

    if value == "disabled" || value == "off" || value == "none" {
        layout.tap_hold_settings.flow_tap_term = None;
    } else {
        let term_str = value.trim_end_matches("ms").trim();
        if let Ok(term) = term_str.parse::<u16>() {
            layout.tap_hold_settings.flow_tap_term = Some(term);
        }
    }
    true
}

/// Parses Chordal Hold (on/off).
pub(super) fn try_parse_chordal_hold(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Chordal Hold**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Chordal Hold**:")
        .unwrap()
        .trim()
        .to_lowercase();

    layout.tap_hold_settings.chordal_hold =
        value == "on" || value == "true" || value == "yes" || value == "enabled";
    true
}
