//! General settings: uncolored key behavior, RGB master switch, RGB timeout.

use crate::models::Layout;

/// Parses Uncolored Key Behavior / Inactive Key Behavior / Uncolored Key Brightness.
pub(super) fn try_parse_uncolored_key_behavior(line: &str, layout: &mut Layout) -> bool {
    if !(line.starts_with("**Inactive Key Behavior**:")
        || line.starts_with("**Uncolored Key Behavior**:")
        || line.starts_with("**Uncolored Key Brightness**:"))
    {
        return false;
    }

    let value = line
        .strip_prefix("**Inactive Key Behavior**:")
        .or_else(|| line.strip_prefix("**Uncolored Key Behavior**:"))
        .or_else(|| line.strip_prefix("**Uncolored Key Brightness**:"))
        .unwrap()
        .trim()
        .to_lowercase();

    // Parse as percentage (legacy format support)
    let percent = if value.contains('%') {
        value
            .trim_end_matches('%')
            .trim()
            .parse::<u8>()
            .unwrap_or(100)
    } else {
        match value.as_str() {
            "off" | "black" | "off (black)" => 0,
            "show color" | "full" => 100,
            _ => value.parse::<u8>().unwrap_or(100),
        }
    }
    .min(100);
    layout.uncolored_key_behavior = crate::models::UncoloredKeyBehavior::from(percent);
    true
}

/// Parses RGB Enabled / RGB Master Switch.
pub(super) fn try_parse_rgb_enabled(line: &str, layout: &mut Layout) -> bool {
    if !(line.starts_with("**RGB Enabled**:") || line.starts_with("**RGB Master Switch**:")) {
        return false;
    }

    let value = line
        .strip_prefix("**RGB Enabled**:")
        .or_else(|| line.strip_prefix("**RGB Master Switch**:"))
        .unwrap()
        .trim()
        .to_lowercase();
    layout.rgb_enabled = matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
    true
}

/// Parses RGB Timeout ("5 min", "300 sec", "300000ms", "disabled", "off").
pub(super) fn try_parse_rgb_timeout(line: &str, layout: &mut Layout) -> bool {
    if !line.starts_with("**RGB Timeout**:") {
        return false;
    }

    let value = line
        .strip_prefix("**RGB Timeout**:")
        .unwrap()
        .trim()
        .to_lowercase();

    // Parse various formats: "5 min", "300 sec", "300000ms", "disabled", "off"
    if value == "disabled" || value == "off" || value == "0" {
        layout.rgb_timeout_ms = 0;
    } else if let Some(mins) = value.strip_suffix(" min").or(value.strip_suffix("min")) {
        if let Ok(m) = mins.trim().parse::<u32>() {
            layout.rgb_timeout_ms = m * 60000;
        }
    } else if let Some(secs) = value.strip_suffix(" sec").or(value.strip_suffix("sec")) {
        if let Ok(s) = secs.trim().parse::<u32>() {
            layout.rgb_timeout_ms = s * 1000;
        }
    } else if let Some(ms) = value.strip_suffix("ms") {
        if let Ok(m) = ms.trim().parse::<u32>() {
            layout.rgb_timeout_ms = m;
        }
    } else if let Ok(ms) = value.parse::<u32>() {
        // Plain number, assume milliseconds
        layout.rgb_timeout_ms = ms;
    }
    true
}
