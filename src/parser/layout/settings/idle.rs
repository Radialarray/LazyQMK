//! Idle Effect settings: enabled flag, idle timeout, duration, mode.

use crate::models::Layout;

/// Parses Idle Effect enabled/disabled.
pub(super) fn try_parse_idle_effect_enabled(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Idle Effect**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Idle Effect**:")
        .unwrap()
        .trim()
        .to_lowercase();
    layout.idle_effect_settings.enabled =
        matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses Idle Timeout ("1 min", "60 sec", "60000ms", or "disabled"/"off"/"0").
pub(super) fn try_parse_idle_timeout(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Idle Timeout**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Idle Timeout**:")
        .unwrap()
        .trim()
        .to_lowercase();

    // Parse various formats: "1 min", "60 sec", "60000ms", "disabled", "off", "0"
    if value == "disabled" || value == "off" || value == "0" {
        layout.idle_effect_settings.idle_timeout_ms = 0;
    } else if let Some(mins) = value.strip_suffix(" min").or(value.strip_suffix("min")) {
        if let Ok(m) = mins.trim().parse::<u32>() {
            layout.idle_effect_settings.idle_timeout_ms = m * 60000;
        }
    } else if let Some(secs) = value.strip_suffix(" sec").or(value.strip_suffix("sec")) {
        if let Ok(s) = secs.trim().parse::<u32>() {
            layout.idle_effect_settings.idle_timeout_ms = s * 1000;
        }
    } else if let Some(ms) = value.strip_suffix("ms") {
        if let Ok(m) = ms.trim().parse::<u32>() {
            layout.idle_effect_settings.idle_timeout_ms = m;
        }
    } else if let Ok(ms) = value.parse::<u32>() {
        // Plain number, assume milliseconds
        layout.idle_effect_settings.idle_timeout_ms = ms;
    }
    true
}

/// Parses Idle Effect Duration ("5 min", "300 sec", "300000ms", or "0"/"off").
pub(super) fn try_parse_idle_effect_duration(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Idle Effect Duration**:") {
        return false;
    }
    let value = line
        .strip_prefix("**Idle Effect Duration**:")
        .unwrap()
        .trim()
        .to_lowercase();

    // Parse various formats: "5 min", "300 sec", "300000ms", "0"
    if value == "0" || value == "off" {
        layout.idle_effect_settings.idle_effect_duration_ms = 0;
    } else if let Some(mins) = value.strip_suffix(" min").or(value.strip_suffix("min")) {
        if let Ok(m) = mins.trim().parse::<u32>() {
            layout.idle_effect_settings.idle_effect_duration_ms = m * 60000;
        }
    } else if let Some(secs) = value.strip_suffix(" sec").or(value.strip_suffix("sec")) {
        if let Ok(s) = secs.trim().parse::<u32>() {
            layout.idle_effect_settings.idle_effect_duration_ms = s * 1000;
        }
    } else if let Some(ms) = value.strip_suffix("ms") {
        if let Ok(m) = ms.trim().parse::<u32>() {
            layout.idle_effect_settings.idle_effect_duration_ms = m;
        }
    } else if let Ok(ms) = value.parse::<u32>() {
        // Plain number, assume milliseconds
        layout.idle_effect_settings.idle_effect_duration_ms = ms;
    }
    true
}

/// Parses Idle Effect Mode (RgbMatrixEffect name).
pub(super) fn try_parse_idle_effect_mode(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**Idle Effect Mode**:") {
        return false;
    }
    let value = line.strip_prefix("**Idle Effect Mode**:").unwrap().trim();

    if let Some(effect) = crate::models::RgbMatrixEffect::from_name(value) {
        layout.idle_effect_settings.idle_effect_mode = effect;
    }
    true
}
